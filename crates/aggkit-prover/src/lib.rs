use std::{path::PathBuf, sync::Arc};

use aggkit_prover_types::v1::aggchain_proof_service_server::AggchainProofServiceServer;
use prover_engine::ProverEngine;
use rpc::GrpcService;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub mod cli;
pub mod rpc;
pub(crate) mod utils;

#[cfg(test)]
mod tests;

pub fn runtime(cfg: PathBuf, version: &str) -> anyhow::Result<()> {
    let config = Arc::new(aggkit_prover_config::ProverConfig::try_load(&cfg)?);

    // Initialize the logger
    prover_logger::tracing(&config.log);

    let global_cancellation_token = CancellationToken::new();

    info!("Starting AggKit Prover version info: {}", version);

    let prover_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("aggkit-prover-runtime")
        .enable_all()
        .build()?;

    let metrics_runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("metrics-runtime")
        .worker_threads(2)
        .enable_all()
        .build()?;

    let aggchain_proof_service = prover_runtime.block_on(async {
        let grpc_service = GrpcService::new(&config.aggchain_proof_service).await?;
        Ok::<AggchainProofServiceServer<GrpcService>, aggchain_proof_service::Error>(
            AggchainProofServiceServer::new(grpc_service),
        )
    })?;

    _ = ProverEngine::builder()
        .add_rpc_service(aggchain_proof_service)
        .add_reflection_service(aggkit_prover_types::v1::FILE_DESCRIPTOR_SET)
        .set_rpc_runtime(prover_runtime)
        .set_metrics_runtime(metrics_runtime)
        .set_cancellation_token(global_cancellation_token)
        .set_rpc_socket_addr(config.grpc_endpoint)
        .set_metric_socket_addr(config.telemetry.addr)
        .start();

    Ok(())
}

/// Common version information about the executed agglayer binary.
pub fn version() -> String {
    let pkg_name = env!("CARGO_PKG_NAME");
    let git_describe = env!("VERGEN_GIT_DESCRIBE");
    let timestamp = env!("VERGEN_GIT_COMMIT_TIMESTAMP");
    format!("{pkg_name} ({git_describe}) [git commit timestamp: {timestamp}]")
}
