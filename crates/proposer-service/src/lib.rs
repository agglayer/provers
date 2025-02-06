use std::task::{Context, Poll};

use futures::{future::BoxFuture, FutureExt};
use proposer_client::ProposerClient;

use crate::config::ProposerServiceConfig;

pub mod config;

pub mod error;

#[derive(Clone)]
pub struct ProposerService {
    pub client: ProposerClient,
}

impl ProposerService {
    pub fn new(config: ProposerServiceConfig) -> Result<Self, crate::error::Error> {
        Ok(Self {
            client: ProposerClient::new(config.client)?,
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
