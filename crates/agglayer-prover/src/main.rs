use std::sync::Arc;

use prover_engine::ProverEngine;
use tokio_util::sync::CancellationToken;
use tracing::info;

fn main() -> anyhow::Result<()> {
    let config = Arc::new(agglayer_prover_config::ProverConfig::default());

    // Initialize the logger
    agglayer_prover::logging::tracing(&config.log);

    let global_cancellation_token = CancellationToken::new();

    info!("Starting Agglayer Prover on {}", config.grpc_endpoint);

    let prover_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("agglayer-prover-runtime")
        .enable_all()
        .build()?;

    let metrics_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("metrics-runtime")
        .worker_threads(2)
        .enable_all()
        .build()?;

    let pp_service =
        prover_runtime.block_on(async { agglayer_prover::prover::Prover::create_service(&config) });

    _ = ProverEngine::builder()
        .add_rpc_service(pp_service)
        .set_rpc_runtime(prover_runtime)
        .set_metrics_runtime(metrics_runtime)
        .set_cancellation_token(global_cancellation_token)
        .start();

    Ok(())
}
