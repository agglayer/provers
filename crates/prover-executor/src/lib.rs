use std::{
    panic::{AssertUnwindSafe, UnwindSafe},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

pub use error::Error;
use eyre::Context as _;
use futures::{Future, FutureExt, TryFutureExt};
use prover_config::{CpuProverConfig, ProverType};
use sp1_sdk::{
    network::{prover::NetworkProver, FulfillmentStrategy},
    CpuProver, Prover, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin,
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
                sp1_blocking(move || {
                    let network_prover = ProverClient::builder()
                        .network()
                        .rpc_url(network_prover_config.sp1_cluster_endpoint.as_str())
                        .build();
                    let (proving_key, verification_key) = network_prover.setup(program);
                    (
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
                    )
                })
                .await
            }
            ProverType::CpuProver(cpu_prover_config) => {
                debug!("Creating CPU prover executor...");
                sp1_blocking(move || {
                    let prover = CpuProver::new();
                    let (proving_key, verification_key) = prover.setup(program);

                    (
                        verification_key.clone(),
                        Self::build_local_service(
                            cpu_prover_config.get_proving_request_timeout(),
                            cpu_prover_config.max_concurrency_limit,
                            LocalExecutor {
                                prover: Arc::new(prover),
                                is_mock: false,
                                proving_key,
                                verification_key,
                            },
                        ),
                    )
                })
                .await
            }
            ProverType::MockProver(mock_prover_config) => {
                debug!("Creating Mock prover executor...");
                sp1_blocking(move || {
                    let prover = CpuProver::mock();
                    let (proving_key, verification_key) = prover.setup(program);

                    (
                        verification_key.clone(),
                        Self::build_local_service(
                            mock_prover_config.get_proving_request_timeout(),
                            mock_prover_config.max_concurrency_limit,
                            LocalExecutor {
                                prover: Arc::new(prover),
                                is_mock: true,
                                proving_key,
                                verification_key,
                            },
                        ),
                    )
                })
                .await
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
        let fallback = match fallback {
            None => None,
            Some(config) => Some(
                Self::create_prover(config, program)
                    .await
                    .context("Failed creating secondary prover")?
                    .1,
            ),
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
struct LocalExecutor {
    proving_key: SP1ProvingKey,
    verification_key: SP1VerifyingKey,
    is_mock: bool,
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
        let is_mock = self.is_mock;
        let stdin = req.stdin;

        let proving_key = self.proving_key.clone();
        let verification_key = self.verification_key.clone();

        debug!("Proving with CPU prover");
        Box::pin(
            sp1_blocking(move || {
                debug!("Starting the proving of the requested MultiBatchHeader");
                let mut proof_request = prover.prove(&proving_key, &stdin);

                proof_request = match req.proof_type {
                    ProofType::Plonk => proof_request.plonk(),
                    ProofType::Stark => proof_request.compressed(),
                };

                if is_mock {
                    proof_request = proof_request.deferred_proof_verification(false);
                }

                let proof = proof_request
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
        let fut = sp1_async(AssertUnwindSafe(async move {
            // AssertUnwindSafe might be a lie, but we currently have a choice between
            // crashing the whole system and hoping for the best.
            // TODO: Figure out a way to kill only the NetworkExecutor service, marking it
            // as unhealthy and potentially restarting it automatically.
            debug!("Starting the proving of the requested MultiBatchHeader");
            let proof_request = prover.prove(&proving_key, &stdin);

            let proof_request = match req.proof_type {
                ProofType::Plonk => proof_request.plonk(),
                ProofType::Stark => proof_request.compressed(),
            };

            let proof = proof_request
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
        }))
        .map_err(|_| Error::UnableToExecuteProver)
        .and_then(|res| async { res });

        Box::pin(fut)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Panic occurred during SP1 call: {message}")]
pub struct Sp1PanicError {
    message: String,
}

impl From<Box<dyn std::any::Any + Send>> for Sp1PanicError {
    fn from(original_error: Box<dyn std::any::Any + Send>) -> Self {
        let message = if let Some(message) = original_error.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = original_error.downcast_ref::<&str>() {
            message.to_string()
        } else {
            "<unknown message type>".to_string()
        };
        Sp1PanicError { message }
    }
}

pub fn sp1_fast<R>(f: impl UnwindSafe + FnOnce() -> R) -> eyre::Result<R> {
    std::panic::catch_unwind(f).map_err(|error| {
        let error = Sp1PanicError::from(error);
        let error = eyre::Report::from(error);
        tracing::error!(?error, "SP1 call panicked");
        error
    })
}

pub async fn sp1_blocking<F, R>(f: F) -> eyre::Result<R>
where
    F: 'static + Send + UnwindSafe + FnOnce() -> R,
    R: 'static + Send,
{
    tokio::task::spawn_blocking(|| sp1_fast(f))
        .await
        .context("Failed running blocking task for SP1 call")?
}

pub async fn sp1_async<F, R>(f: F) -> eyre::Result<R>
where
    F: UnwindSafe + Future<Output = R>,
{
    f.catch_unwind().await.map_err(|error| {
        let error = Sp1PanicError::from(error);
        let error = eyre::Report::from(error);
        tracing::error!(?error, "SP1 call panicked");
        error
    })
}
