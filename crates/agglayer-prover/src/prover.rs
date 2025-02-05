use std::sync::Arc;

use agglayer_prover_config::ProverConfig;
use agglayer_prover_types::v1::pessimistic_proof_service_server::PessimisticProofServiceServer;
use anyhow::Result;
use prover_executor::Executor;
use tokio::join;
use tokio_util::sync::CancellationToken;
use tonic::{codec::CompressionEncoding, transport::Server};
use tower::{limit::ConcurrencyLimitLayer, ServiceExt as _};
use tracing::{debug, error};

use crate::rpc::ProverRPC;

pub struct Prover {
    handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
}

#[buildstructor::buildstructor]
impl Prover {
    pub fn create_service(
        config: &ProverConfig,
        program: &[u8],
    ) -> PessimisticProofServiceServer<ProverRPC> {
        let executor = tower::ServiceBuilder::new()
            .timeout(config.max_request_duration)
            .layer(ConcurrencyLimitLayer::new(config.max_concurrency_limit))
            .service(Executor::new(config, program))
            .into_inner()
            .boxed();

        let executor = tower::buffer::Buffer::new(executor, config.max_buffered_queries);

        let rpc = ProverRPC::new(executor);

        PessimisticProofServiceServer::new(rpc)
            .max_decoding_message_size(config.grpc.max_decoding_message_size)
            .max_encoding_message_size(config.grpc.max_encoding_message_size)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd)
    }

    /// Function that setups and starts the Agglayer Prover.
    ///
    /// The available methods are:
    ///
    /// - `builder`: Creates a new builder instance.
    /// - `config`: Sets the configuration.
    /// - `start`: Starts the Agglayer prover.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The gRPC server failed to start.
    #[builder(entry = "builder", exit = "start", visibility = "pub(crate)")]
    pub async fn start(
        config: Arc<ProverConfig>,
        cancellation_token: CancellationToken,
        program: &'static [u8],
    ) -> Result<Self> {
        let svc = Self::create_service(&config, program);
        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

        health_reporter
            .set_serving::<PessimisticProofServiceServer<ProverRPC>>()
            .await;

        let reflection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(agglayer_prover_types::FILE_DESCRIPTOR_SET)
            .build_v1alpha()
            .expect("Cannot build gRPC because of FILE_DESCRIPTOR_SET error");
        let layer = tower::ServiceBuilder::new().into_inner();

        let handle = tokio::spawn(async move {
            if let Err(error) = Server::builder()
                .layer(layer)
                .add_service(reflection)
                .add_service(health_service)
                .add_service(svc)
                .serve_with_shutdown(config.grpc_endpoint, cancellation_token.cancelled())
                .await
            {
                error!("Failed to start Agglayer Prover: {}", error);

                return Err(error);
            }

            Ok(())
        });
        Ok(Self { handle })
    }

    pub async fn await_shutdown(self) {
        debug!("Node shutdown started.");
        _ = join!(self.handle);
        debug!("Node shutdown completed.");
    }
}
