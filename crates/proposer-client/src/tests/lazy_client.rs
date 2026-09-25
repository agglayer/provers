#![allow(clippy::result_large_err)]

use std::time::Duration;

use crate::{rpc::ProposerRpcClient, GrpcUri};

/// A syntactically valid endpoint with nothing listening on it. An eager dial
/// fails against it, a lazy dial cannot.
const UNREACHABLE: &str = "http://127.0.0.1:1";

fn unreachable_endpoint() -> GrpcUri {
    UNREACHABLE
        .parse()
        .expect("UNREACHABLE should be a valid gRPC endpoint")
}

#[tokio::test]
async fn new_dials_eagerly() {
    let result = ProposerRpcClient::new(unreachable_endpoint(), Duration::from_secs(1)).await;

    assert!(
        result.is_err(),
        "the network path must keep dialling the proposer eagerly"
    );
}

/// Mock mode builds its proposer client through this constructor, so it must
/// not require a reachable proposer to start up.
#[tokio::test]
async fn new_lazy_does_not_dial() {
    if let Err(error) =
        ProposerRpcClient::new_lazy(unreachable_endpoint(), Duration::from_secs(1)).await
    {
        panic!("mock mode must not require a reachable proposer: {error}");
    }
}
