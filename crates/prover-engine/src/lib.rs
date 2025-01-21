use std::{convert::Infallible, future::IntoFuture, net::SocketAddr};

use agglayer_telemetry::ServerBuilder as MetricsBuilder;
use http::{Request, Response};
use tokio::{net::TcpListener, runtime::Runtime};
use tokio_util::sync::CancellationToken;
use tonic::{
    body::{boxed, BoxBody},
    server::NamedService,
};
use tower::{Service, ServiceExt};
use tracing::info;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct ProverEngine {
    rpc_server: axum::Router,
    rpc_runtime: Option<Runtime>,
    metrics_runtime: Option<Runtime>,
    reflection: Vec<&'static [u8]>,
    healthy_service: Vec<&'static str>,
    cancellation_token: Option<CancellationToken>,
}

impl ProverEngine {
    pub fn builder() -> Self {
        Self {
            rpc_server: axum::Router::new(),
            reflection: vec![],
            healthy_service: vec![],
            rpc_runtime: None,
            metrics_runtime: None,
            cancellation_token: None,
        }
    }

    pub fn set_rpc_runtime(mut self, rpc_runtime: Runtime) -> Self {
        self.rpc_runtime = Some(rpc_runtime);

        self
    }

    pub fn set_metrics_runtime(mut self, metrics_runtime: Runtime) -> Self {
        self.metrics_runtime = Some(metrics_runtime);

        self
    }
}

impl ProverEngine {
    pub fn set_cancellation_token(mut self, cancellation_token: CancellationToken) -> Self {
        self.cancellation_token = Some(cancellation_token);
        self
    }

    pub fn add_rpc_service<S>(mut self, rpc_service: S) -> Self
    where
        S: Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Sync
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<BoxError> + Send,
    {
        self.rpc_server = add_rpc_service(self.rpc_server, rpc_service);
        self.healthy_service.push(S::NAME);

        self
    }

    pub fn add_reflection_service(mut self, descriptor: &'static [u8]) -> Self {
        self.reflection.push(descriptor);

        self
    }

    pub fn start(mut self) -> anyhow::Result<()> {
        let cancellation_token = self.cancellation_token.take().unwrap_or_default();

        let metrics_runtime = self
            .metrics_runtime
            .take()
            .map(Result::Ok)
            .unwrap_or_else(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .thread_name("metrics-runtime")
                    .worker_threads(2)
                    .enable_all()
                    .build()
            })?;

        let prover_runtime = self.rpc_runtime.take().map(Result::Ok).unwrap_or_else(|| {
            tokio::runtime::Builder::new_multi_thread()
                .thread_name("prover-runtime")
                .enable_all()
                .build()
        })?;

        let addr: SocketAddr = "[::1]:10000".parse().unwrap();
        let telemetry_addr: SocketAddr = "[::1]:3400".parse().unwrap();

        // Create the metrics server.
        let metric_server = metrics_runtime.block_on(
            MetricsBuilder::builder()
                .addr(telemetry_addr)
                .cancellation_token(cancellation_token.clone())
                .build(),
        )?;

        // Spawn the metrics server into the metrics runtime.
        let metrics_handle = {
            // This guard is used to ensure that the metrics runtime is entered
            // before the server is spawned. This is necessary because the `into_future`
            // of `WithGracefulShutdown` is spawning various tasks before returning the
            // actual server instance to spawn.
            let _guard = metrics_runtime.enter();
            // Spawn the metrics server
            metrics_runtime.spawn(metric_server.into_future())
        };
        let tcp_listener = prover_runtime.block_on(TcpListener::bind(addr))?;

        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

        let (reflection_v1, reflection_v1alpha) = self.reflection.iter().fold(
            (
                tonic_reflection::server::Builder::configure(),
                tonic_reflection::server::Builder::configure(),
            ),
            |(v1, v1alpha), descriptor| {
                (
                    v1.register_encoded_file_descriptor_set(descriptor),
                    v1alpha.register_encoded_file_descriptor_set(descriptor),
                )
            },
        );

        let reflection_v1 = reflection_v1
            .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
            .build_v1()
            .unwrap();

        let reflection_v1alpha = reflection_v1alpha
            .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
            .build_v1alpha()
            .unwrap();

        prover_runtime.block_on(async {
            for service_name in self.healthy_service.iter() {
                health_reporter
                    .set_service_status(service_name, tonic_health::ServingStatus::Serving)
                    .await;
            }
        });

        // Adding the reflection and health services to the RPC server
        let rpc_server = add_rpc_service(self.rpc_server, reflection_v1);
        let rpc_server = add_rpc_service(rpc_server, reflection_v1alpha);
        let rpc_server = add_rpc_service(rpc_server, health_service);

        let token = cancellation_token.clone();
        let prover_handle = prover_runtime.spawn(
            axum::serve(tcp_listener, rpc_server)
                .with_graceful_shutdown(async move { token.cancelled().await })
                .into_future(),
        );

        let terminate_signal = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Fail to setup SIGTERM signal")
                .recv()
                .await;
        };

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(async {
                tokio::select! {
                    _ = terminate_signal => {
                        info!("Received SIGTERM, shutting down...");
                        // Cancel the global cancellation token to start the shutdown process.
                        cancellation_token.cancel();
                        // Wait for the prover to shutdown.
                        _ = prover_handle.await;
                        // Wait for the metrics server to shutdown.
                        _ = metrics_handle.await;
                    }
                    _ = tokio::signal::ctrl_c() => {
                        info!("Received SIGINT (ctrl-c), shutting down...");
                        // Cancel the global cancellation token to start the shutdown process.
                        cancellation_token.cancel();
                        // Wait for the prover to shutdown.
                        _ = prover_handle.await;
                        // Wait for the metrics server to shutdown.
                        _ = metrics_handle.await;
                    }
                }
            });

        // prover_runtime.shutdown_timeout(config.shutdown.runtime_timeout);
        // metrics_runtime.shutdown_timeout(config.shutdown.runtime_timeout);

        Ok(())
    }
}

fn add_rpc_service<S>(rpc_server: axum::Router, rpc_service: S) -> axum::Router
where
    S: Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Sync
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError> + Send,
{
    rpc_server.route_service(
        &format!("/{}/*rest", S::NAME),
        rpc_service.map_request(|r: Request<axum::body::Body>| r.map(boxed)),
    )
}
