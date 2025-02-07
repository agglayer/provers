use std::sync::Arc;
use std::task::{Context, Poll};

use futures::{future::BoxFuture, FutureExt};
use proposer_client::network_prover::new_network_prover;
use proposer_client::rpc::ProposerRpcClient;
use proposer_client::ProposerClient;
use sp1_sdk::NetworkProver;

use crate::config::ProposerServiceConfig;

pub mod config;

pub mod error;

#[derive(Clone)]
pub struct ProposerService {
    pub client: Arc<ProposerClient<ProposerRpcClient, NetworkProver>>,
}

impl ProposerService {
    pub fn new(config: ProposerServiceConfig) -> Result<Self, crate::error::Error> {
        let proposer_client = ProposerRpcClient::new(config.client.proposer_endpoint.as_str())?;
        let network_prover = new_network_prover(config.client.sp1_cluster_endpoint.as_str());
        Ok(Self {
            client: Arc::new(ProposerClient::new(
                proposer_client,
                network_prover,
                Some(config.client.proving_timeout),
            )?),
        })
    }
}

pub struct Request {}
pub struct Response {}

#[derive(thiserror::Error, Debug)]
pub enum Error {}

impl tower::Service<Request> for ProposerService {
    type Response = Response;

    type Error = Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request) -> Self::Future {
        async { Ok(Response {}) }.boxed()
    }
}
