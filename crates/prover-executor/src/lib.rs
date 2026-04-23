use std::{
    panic::AssertUnwindSafe,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

pub use error::Error;
use eyre::Context as _;
use futures::{Future, TryFutureExt};
use prover_config::{CpuProverConfig, ProverType};
use sp1_sdk::{
    network::FulfillmentStrategy, CpuProver, MockProver, NetworkProver, ProveRequest as _, Prover,
    ProverClient, ProvingKey as _, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin,
    SP1VerifyingKey,
};
use tower::{
    limit::ConcurrencyLimitLayer, timeout::TimeoutLayer, util::BoxCloneService, Service,
    ServiceBuilder, ServiceExt,
};
use tracing::{debug, error, info};

#[cfg(test)]
mod tests;

mod error;
mod utils;

pub use utils::*;

#[derive(Clone)]
pub struct Executor {
    vkey: Arc<SP1VerifyingKey>,
    primary: BoxCloneService<Request, Response, Error>,
    fallback: Option<BoxCloneService<Request, Response, Error>>,
}

impl Executor {
    pub fn get_vkey(&self) -> &Arc<SP1VerifyingKey> {
        &self.vkey
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
        vkey: Arc<SP1VerifyingKey>,
        primary: BoxCloneService<Request, Response, Error>,
        fallback: Option<BoxCloneService<Request, Response, Error>>,
    ) -> Self {
        Self {
            vkey,
            primary,
            fallback,
        }
    }

    pub async fn create_prover(
        prover_type: ProverType,
        program: &'static [u8],
    ) -> eyre::Result<(SP1VerifyingKey, BoxCloneService<Request, Response, Error>)> {
        match prover_type {
            ProverType::NetworkProver(network_prover_config) => {
                debug!("Creating network prover executor...");
                let network_prover = ProverClient::builder()
                    .network()
                    .rpc_url(network_prover_config.sp1_cluster_endpoint.as_str())
                    .build()
                    .await;
                let proving_key = network_prover
                    .setup(program.into())
                    .await
                    .map_err(|error| eyre::eyre!(error.to_string()))?;
                let verification_key = proving_key.verifying_key().clone();

                Ok((
                    verification_key.clone(),
                    Self::build_network_service(
                        network_prover_config.get_proving_request_timeout(),
                        NetworkExecutor {
                            prover: Arc::new(network_prover),
                            proving_key,
                            verification_key,
                            timeout: network_prover_config.proving_timeout,
                        },
                    ),
                ))
            }
            ProverType::CpuProver(cpu_prover_config) => {
                debug!("Creating CPU prover executor...");
                let prover = sp1_async(AssertUnwindSafe(CpuProver::new()))
                    .await
                    .context("CpuProver initialization panicked")?;
                let proving_key = prover
                    .setup(program.into())
                    .await
                    .map_err(|error| eyre::eyre!(error.to_string()))?;
                let verification_key = proving_key.verifying_key().clone();

                Ok((
                    verification_key.clone(),
                    Self::build_local_service(
                        cpu_prover_config.get_proving_request_timeout(),
                        cpu_prover_config.max_concurrency_limit,
                        LocalExecutor {
                            prover: Arc::new(LocalProver::Cpu(prover)),
                            proving_key,
                            verification_key,
                        },
                    ),
                ))
            }
            ProverType::MockProver(mock_prover_config) => {
                debug!("Creating Mock prover executor...");
                let prover = sp1_async(AssertUnwindSafe(MockProver::new()))
                    .await
                    .context("MockProver initialization panicked")?;
                let proving_key = prover
                    .setup(program.into())
                    .await
                    .map_err(|error| eyre::eyre!(error.to_string()))?;
                let verification_key = proving_key.verifying_key().clone();

                Ok((
                    verification_key.clone(),
                    Self::build_local_service(
                        mock_prover_config.get_proving_request_timeout(),
                        mock_prover_config.max_concurrency_limit,
                        LocalExecutor {
                            prover: Arc::new(LocalProver::Mock(prover)),
                            proving_key,
                            verification_key,
                        },
                    ),
                ))
            }
        }
    }

    pub async fn new(
        primary: ProverType,
        fallback: Option<ProverType>,
        program: &'static [u8],
    ) -> eyre::Result<Self> {
        let (vkey, primary) = Self::create_prover(primary, program)
            .await
            .context("Failed creating primary prover")?;
        let fallback = if let Some(config) = fallback {
            Some(
                Self::create_prover(config, program)
                    .await
                    .context("Failed creating secondary prover")?
                    .1,
            )
        } else {
            None
        };
        Ok(Self {
            vkey: Arc::new(vkey),
            primary,
            fallback,
        })
    }

    pub async fn compute_program_vkey(program: &'static [u8]) -> eyre::Result<SP1VerifyingKey> {
        let executor = Executor::new(
            ProverType::CpuProver(CpuProverConfig::default()),
            None,
            program,
        )
        .await
        .context("Failed creating fake prover to compute program vkey")?;
        Ok(SP1VerifyingKey::clone(executor.get_vkey()))
    }
}

#[derive(Debug, Clone)]
pub enum ProofType {
    Stark,
    Plonk,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub stdin: SP1Stdin,
    pub proof_type: ProofType,
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
        std::task::ready!(self
            .primary
            .poll_ready(cx)
            .map_err(|_| Error::UnableToInitializePrimaryProver)?);

        match &mut self.fallback {
            Some(fallback) => fallback
                .poll_ready(cx)
                .map_err(|_| Error::UnableToInitializeFallbackProver),
            None => Poll::Ready(Ok(())),
        }
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut primary = self.primary.clone();
        let fallback = self.fallback.clone();

        let fut = async move {
            let result = primary.ready().await?.call(req.clone()).await;
            match result {
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
enum LocalProver {
    Cpu(CpuProver),
    Mock(MockProver),
}

#[derive(Clone)]
struct LocalExecutor {
    proving_key: SP1ProvingKey,
    verification_key: SP1VerifyingKey,
    prover: Arc<LocalProver>,
}

async fn prove_with_local_prover<P>(
    prover: &P,
    proving_key: &SP1ProvingKey,
    verification_key: &SP1VerifyingKey,
    stdin: SP1Stdin,
    proof_type: ProofType,
    disable_deferred_proof_verification: bool,
) -> Result<SP1ProofWithPublicValues, Error>
where
    P: Prover<ProvingKey = SP1ProvingKey>,
    P::Error: std::fmt::Display,
{
    let mut proof_request = prover.prove(proving_key, stdin);

    proof_request = match proof_type {
        ProofType::Plonk => proof_request.plonk(),
        ProofType::Stark => proof_request.compressed(),
    };

    if disable_deferred_proof_verification {
        proof_request = proof_request.deferred_proof_verification(false);
    }

    let proof = proof_request
        .await
        .map_err(|error| Error::ProverFailed(error.to_string()))?;

    prover
        .verify(&proof, verification_key, None)
        .map_err(|error| Error::ProofVerificationFailed(error.into()))?;

    Ok(proof)
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
        let proof_type = req.proof_type;

        let proving_key = self.proving_key.clone();
        let verification_key = self.verification_key.clone();

        debug!("Proving with local prover");
        Box::pin(async move {
            debug!("Starting the proving of the requested MultiBatchHeader");

            let proof = match prover.as_ref() {
                LocalProver::Cpu(prover) => {
                    prove_with_local_prover(
                        prover,
                        &proving_key,
                        &verification_key,
                        stdin,
                        proof_type,
                        false,
                    )
                    .await
                }
                LocalProver::Mock(prover) => {
                    prove_with_local_prover(
                        prover,
                        &proving_key,
                        &verification_key,
                        stdin,
                        proof_type,
                        true,
                    )
                    .await
                }
            }?;

            debug!("Proof verification completed successfully");
            Ok(Response { proof })
        })
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
        let fut = sp1_async(AssertUnwindSafe(async move {
            // AssertUnwindSafe might be a lie, but we currently have a choice between
            // crashing the whole system and hoping for the best.
            // TODO: Figure out a way to kill only the NetworkExecutor service, marking it
            // as unhealthy and potentially restarting it automatically.
            debug!("Starting the proving of the requested MultiBatchHeader");
            let proof_request = prover.prove(&proving_key, stdin);

            let proof_request = match req.proof_type {
                ProofType::Plonk => proof_request.plonk(),
                ProofType::Stark => proof_request.compressed(),
            };

            let proof = proof_request
                .timeout(timeout)
                .strategy(FulfillmentStrategy::Reserved)
                .await
                .map_err(|error| Error::ProverFailed(error.to_string()))?;

            debug!("Proving completed. Verifying the proof...");
            prover
                .verify(&proof, &verification_key, None)
                .map_err(|error| Error::ProofVerificationFailed(error.into()))?;

            debug!("Proof verification completed successfully");
            Ok(Response { proof })
        }))
        .map_err(|_| Error::UnableToExecuteProver)
        .and_then(|res| async { res });

        Box::pin(fut)
    }
}
