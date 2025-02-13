use std::sync::Arc;

use aggkit_prover::aggchain_proof::GrpcService;
use aggkit_prover_types::v1::aggchain_proof_service_server::AggchainProofServiceServer;
use prover_engine::ProverEngine;
use tokio_util::sync::CancellationToken;
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

    // TODO: implement the aggchain-proof service
    let aggchain_proof_service =
        AggchainProofServiceServer::new(GrpcService::new(&config.aggchain_proof_service)?);

    _ = ProverEngine::builder()
        .add_rpc_service(aggchain_proof_service)
        .add_reflection_service(aggkit_prover_types::FILE_DESCRIPTOR_SET)
        .set_rpc_runtime(prover_runtime)
        .set_metrics_runtime(metrics_runtime)
        .set_cancellation_token(global_cancellation_token)
        .start();

    Ok(())
}
