use std::{convert::Infallible, sync::Arc, task::Poll};

use http::{Request, Response};
use prover_engine::ProverEngine;
use tokio_util::sync::CancellationToken;
use tonic::{
    body::{empty_body, BoxBody},
    server::NamedService,
};
use tower::Service;
use tracing::info;

fn main() -> anyhow::Result<()> {
    let config = Arc::new(aggkit_prover_config::ProverConfig::default());

    // Initialize the logger
    prover_logger::tracing(&config.log);

    let global_cancellation_token = CancellationToken::new();

    info!("Starting AggKit Prover on {}", config.grpc_endpoint);

    let prover_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("aggkit-prover-runtime")
        .enable_all()
        .build()?;

    let metrics_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("metrics-runtime")
        .worker_threads(2)
        .enable_all()
        .build()?;

    // TODO: implement the auth-proof service
    let auth_proof_service = AuthProofService;

    _ = ProverEngine::builder()
        .add_rpc_service(auth_proof_service)
        .set_rpc_runtime(prover_runtime)
        .set_metrics_runtime(metrics_runtime)
        .set_cancellation_token(global_cancellation_token)
        .start();

    Ok(())
}

#[derive(Clone, Copy)]
struct AuthProofService;

impl NamedService for AuthProofService {
    const NAME: &'static str = "auth-proof-service";
}

impl Service<Request<BoxBody>> for AuthProofService {
    type Response = Response<BoxBody>;

    type Error = Infallible;

    type Future = futures::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        info!("Received request: {:?}", request);
        futures::future::ok(Response::new(empty_body()))
    }
}
