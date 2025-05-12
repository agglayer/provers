use std::time::Duration;

use op_succinct_grpc::proofs::{GetMockProofRequest, GetMockProofResponse};
use tonic::transport::server::TcpIncoming;
pub use tonic::transport::Error as TransportError;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::rpc::grpc::{self, proofs_server::Proofs};

mockall::mock! {
    /// Mock op-succinct proofs service.
    #[derive(Clone)]
    pub ProofsService {}

    #[tonic::async_trait]
    impl Proofs for ProofsService {
        async fn request_agg_proof(
            &self,
            request: tonic::Request<grpc::AggProofRequest>,
        ) -> Result<tonic::Response<grpc::AggProofResponse>, tonic::Status>;

        async fn get_mock_proof(
            &self,
            request: Request<GetMockProofRequest>
        ) -> Result<Response<GetMockProofResponse>, Status>;
    }
}

impl MockProofsService {
    /// Run a mock server.
    pub async fn run(self) -> Result<Handle, anyhow::Error> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let local_addr = listener.local_addr()?;

        info!("Starting mock server on {local_addr}");

        let cancellation_token = tokio_util::sync::CancellationToken::new();

        let task = tokio::spawn({
            let cancellation_token = cancellation_token.clone();
            let incoming =
                TcpIncoming::from_listener(listener, false, Some(Duration::from_secs(5)))
                    .map_err(anyhow::Error::from_boxed)?;

            async move {
                tonic::transport::Server::builder()
                    .add_service(grpc::proofs_server::ProofsServer::new(self))
                    .serve_with_incoming_shutdown(incoming, cancellation_token.cancelled())
                    .await
            }
        });

        info!("Started mock server on {local_addr}");

        Ok(Handle {
            task,
            local_addr,
            _drop_guard: cancellation_token.clone().drop_guard(),
            cancellation_token,
        })
    }
}

/// Handle to a running mock server.
///
/// The server is stopped if the handle goes out of scope.
pub struct Handle {
    task: tokio::task::JoinHandle<Result<(), TransportError>>,
    local_addr: std::net::SocketAddr,
    cancellation_token: tokio_util::sync::CancellationToken,
    _drop_guard: tokio_util::sync::DropGuard,
}

impl Handle {
    /// Get the mock server address.
    pub fn local_addr(&self) -> &std::net::SocketAddr {
        &self.local_addr
    }

    /// Get the URI to connect to the mock server.
    pub fn uri(&self) -> crate::GrpcUri {
        format!("http://{}", self.local_addr()).parse().unwrap()
    }

    /// Stop the server and get the return value.
    pub async fn stop(self) -> Result<(), TransportError> {
        self.cancellation_token.cancel();
        self.task.await.unwrap()
    }
}
