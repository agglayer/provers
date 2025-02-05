use std::task::{Context, Poll};

use futures::{future::BoxFuture, FutureExt};

pub struct Request {}
pub struct Response {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Noop")]
    Noop,
}

/// This service is responsible for building an Aggchain proof.
#[derive(Default, Clone)]
pub struct AggchainProofBuilderService {}

impl tower::Service<Request> for AggchainProofBuilderService {
    type Response = Response;

    type Error = Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, _req: Request) -> Self::Future {
        async move { Ok(Response {}) }.boxed()
    }
}
