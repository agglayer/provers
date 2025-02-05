mod provider;

use std::sync::Arc;
use std::task::{Context, Poll};

use aggkit_prover_config::aggchain_proof::{
    AggchainProofBuilderConfig, HTTP_RPC_NODE_BACKOFF_MAX_RETRIES, HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
};
use futures::{future::BoxFuture, FutureExt};

use crate::provider::json_rpc::{build_http_retry_provider, AlloyProvider};

pub struct AgghcainProofBuilderRequest {}
pub struct Response {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Alloy error: {0}")]
    AlloyProviderError(String),
}

/// This service is responsible for building an Aggchain proof.
#[derive(Debug, Clone)]
pub struct AggchainProofBuilderService {
    l1_client: Arc<AlloyProvider>,
    l2_client: Arc<AlloyProvider>,
}

impl AggchainProofBuilderService {
    pub fn new(config: AggchainProofBuilderConfig) -> Result<Self, Error> {
        Ok(AggchainProofBuilderService {
            l1_client: Arc::new(
                build_http_retry_provider(
                    config.l1_rpc_endpoint,
                    HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                    HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
                )
                .map_err(|e| Error::AlloyProviderError(e.to_string()))?,
            ),
            l2_client: Arc::new(
                build_http_retry_provider(
                    config.l2_rpc_endpoint,
                    HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                    HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
                )
                .map_err(|e| Error::AlloyProviderError(e.to_string()))?,
            ),
        })
    }
}

impl tower::Service<AgghcainProofBuilderRequest> for AggchainProofBuilderService {
    type Response = Response;

    type Error = Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, _req: AgghcainProofBuilderRequest) -> Self::Future {
        async move { Ok(Response {}) }.boxed()
    }
}
