use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

pub use error::Error;
use error::ProofVerificationError;
use futures::{Future, TryFutureExt};
use prover_config::ProverType;
use sindri::{
    client::SindriClient,
    integrations::sp1_v4::SP1ProofInfo,
    ProofInput,
};
use sp1_sdk::{
    network::{prover::NetworkProver, FulfillmentStrategy},
    CpuProver, Prover, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin,
    SP1VerifyingKey,
};
use tokio::task::spawn_blocking;
use tower::{
    limit::ConcurrencyLimitLayer, timeout::TimeoutLayer, util::BoxCloneService, Service,
    ServiceBuilder, ServiceExt,
};
use tracing::{debug, error, info};

#[cfg(test)]
mod tests;

mod error;

#[derive(Clone)]
pub struct Executor {
    primary: BoxCloneService<Request, Response, Error>,
    fallback: Option<BoxCloneService<Request, Response, Error>>,
}

impl Executor {
    pub fn get_vkey(elf: &[u8]) -> SP1VerifyingKey {
        let prover = CpuProver::new();
        let (_proving_key, verification_key) = prover.setup(elf);

        verification_key
    }

    pub fn build_network_service<S>(
        timeout: Duration,
        service: S,
    ) -> BoxCloneService<Request, Response, Error>
    where
        S: Service<Request, Response = Response, Error = Error> + Send + Clone + 'static,
        <S as Service<Request>>::Future: std::marker::Send,
    {
        BoxCloneService::new(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(timeout))
                .service(service)
                .map_err(|error| match error.downcast::<Error>() {
                    Ok(error) => *error,
                    Err(error) => Error::ProverFailed(error.to_string()),
                }),
        )
    }

    pub fn build_local_service<S>(
        timeout: Duration,
        concurrency: usize,
        service: S,
    ) -> BoxCloneService<Request, Response, Error>
    where
        S: Service<Request, Response = Response, Error = Error> + Send + Clone + 'static,
        <S as Service<Request>>::Future: std::marker::Send,
    {
        BoxCloneService::new(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(timeout))
                .layer(ConcurrencyLimitLayer::new(concurrency))
                .service(service)
                .map_err(|error| match error.downcast::<Error>() {
                    Ok(error) => *error,
                    Err(error) => Error::ProverFailed(error.to_string()),
                }),
        )
    }

    #[cfg(test)]
    pub fn new_with_services(
        primary: BoxCloneService<Request, Response, Error>,
        fallback: Option<BoxCloneService<Request, Response, Error>>,
    ) -> Self {
        Self { primary, fallback }
    }

    pub fn create_prover(
        prover_type: &ProverType,
        program: &[u8],
    ) -> BoxCloneService<Request, Response, Error> {
        match prover_type {
            ProverType::NetworkProver(network_prover_config) => {
                debug!("Creating network prover executor...");
                let network_prover = ProverClient::builder().network().build();
                let (proving_key, verification_key) = network_prover.setup(program);
                Self::build_network_service(
                    network_prover_config.get_proving_request_timeout(),
                    NetworkExecutor {
                        prover: Arc::new(network_prover),
                        proving_key,
                        verification_key,
                        timeout: network_prover_config.proving_timeout,
                    },
                )
            }
            ProverType::CpuProver(cpu_prover_config) => {
                debug!("Creating CPU prover executor...");
                let prover = CpuProver::new();
                let (proving_key, verification_key) = prover.setup(program);

                Self::build_local_service(
                    cpu_prover_config.get_proving_request_timeout(),
                    cpu_prover_config.max_concurrency_limit,
                    LocalExecutor {
                        prover: Arc::new(prover),
                        proving_key,
                        verification_key,
                    },
                )
            }
            ProverType::MockProver(mock_prover_config) => {
                debug!("Creating Mock prover executor...");
                let prover = CpuProver::mock();
                let (proving_key, verification_key) = prover.setup(program);

                Self::build_local_service(
                    mock_prover_config.get_proving_request_timeout(),
                    mock_prover_config.max_concurrency_limit,
                    LocalExecutor {
                        prover: Arc::new(prover),
                        proving_key,
                        verification_key,
                    },
                )
            }
            ProverType::GpuProver(_) => todo!(),
            ProverType::SindriProver(_) => todo!(),
        }
    }

    pub fn new(primary: &ProverType, fallback: &Option<ProverType>, program: &[u8]) -> Self {
        let primary = Self::create_prover(primary, program);
        let fallback = fallback
            .as_ref()
            .map(|config| Self::create_prover(config, program));
        Self { primary, fallback }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub stdin: SP1Stdin,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub proof: SP1ProofWithPublicValues,
}

impl Service<Request> for Executor {
    type Response = Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.primary.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let primary = self.primary.call(req.clone());
        let fallback = self.fallback.clone();
        let fut = async move {
            match primary.await {
                Ok(res) => Ok(res),
                Err(err) => {
                    error!("Primary prover failed: {:?}", err);
                    if let Some(mut _fallback) = fallback {
                        // If fallback prover is set, try to use it
                        info!("Repeating proving request with fallback prover...");
                        _fallback.ready().await?.call(req).await
                    } else {
                        // Return primary prover error
                        Err(err)
                    }
                }
            }
        };

        Box::pin(fut)
    }
}

#[derive(Clone)]
struct LocalExecutor {
    proving_key: SP1ProvingKey,
    verification_key: SP1VerifyingKey,
    prover: Arc<CpuProver>,
}

impl Service<Request> for LocalExecutor {
    type Response = Response;

    type Error = Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let prover = self.prover.clone();
        let stdin = req.stdin;

        let proving_key = self.proving_key.clone();
        let verification_key = self.verification_key.clone();

        debug!("Proving with CPU prover");
        Box::pin(
            spawn_blocking(move || {
                debug!("Starting the proving of the requested MultiBatchHeader");
                let proof = prover
                    .prove(&proving_key, &stdin)
                    .plonk()
                    .run()
                    .map_err(|error| Error::ProverFailed(error.to_string()))?;

                debug!("Proving completed. Verifying the proof...");
                prover
                    .verify(&proof, &verification_key)
                    .map_err(|error| Error::ProofVerificationFailed(error.into()))?;

                debug!("Proof verification completed successfully");

                Ok(Response { proof })
            })
            .map_err(|_| Error::UnableToExecuteProver)
            .and_then(|res| async { res }),
        )
    }
}

#[derive(Clone)]
struct NetworkExecutor {
    prover: Arc<NetworkProver>,
    proving_key: SP1ProvingKey,
    verification_key: SP1VerifyingKey,
    timeout: Duration,
}

impl Service<Request> for NetworkExecutor {
    type Response = Response;

    type Error = Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let prover = self.prover.clone();
        let stdin = req.stdin;

        let verification_key = self.verification_key.clone();
        let proving_key = self.proving_key.clone();
        let timeout = self.timeout;

        debug!("Proving with network prover with timeout: {:?}", timeout);
        let fut = async move {
            debug!("Starting the proving of the requested MultiBatchHeader");
            let proof = prover
                .prove(&proving_key, &stdin)
                .plonk()
                .timeout(timeout)
                .strategy(FulfillmentStrategy::Reserved)
                .run_async()
                .await
                .map_err(|error| Error::ProverFailed(error.to_string()))?;

            debug!("Proving completed. Verifying the proof...");
            prover
                .verify(&proof, &verification_key)
                .map_err(|error| Error::ProofVerificationFailed(error.into()))?;

            debug!("Proof verification completed successfully");
            Ok(Response { proof })
        };

        Box::pin(fut)
    }
}

#[derive(Clone)]
struct SindriExecutor {
    prover: Arc<SindriClient>,
    verification_key: SP1VerifyingKey,
    timeout: Duration,
}

impl Service<Request> for SindriExecutor {
    type Response = Response;

    type Error = Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let prover = self.prover.clone();
        let stdin = req.stdin;

        let verification_key = self.verification_key.clone();
        let timeout = self.timeout;

        debug!("Proving with network prover with timeout: {:?}", timeout);
        let fut = async move {
            debug!("Starting the proving of the requested MultiBatchHeader");

            let circuit_id = "pessimistic-proof";
            let proof_input = ProofInput::try_from(stdin).map_err(|error| Error::ProverFailed(error.to_string()))?;
            let proof_response = prover
                .prove_circuit_blocking(circuit_id, proof_input, None, None, None)
                .map_err(|error| Error::ProverFailed(error.to_string()))?;

            let proof = proof_response
                .to_sp1_proof_with_public()
                .map_err(|error| Error::ProverFailed(error.to_string()))?;

            debug!("Proving completed. Verifying the proof...");
            proof_response
                .verify_sp1_proof_locally(&verification_key)
                .map_err(|error| Error::ProofVerificationFailed(ProofVerificationError::Plonk(error.to_string())))?;

            debug!("Proof verification completed successfully");
            Ok(Response { proof })
        };

        Box::pin(fut)
    }
}
