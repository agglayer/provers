use crate::RequestId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to create RPC client")]
    UnableToCreateRPCClient(#[source] jsonrpsee::core::client::Error),

    #[error("An error occurred while requesting an aggregated proof")]
    AggProofRequestFailed(#[source] jsonrpsee::core::client::Error),

    #[error("Reqwest http error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Invalid request_id: {0:?}")]
    InvalidRequestId(String),

    #[error("Proof request with request_id: {0} timeout")]
    Timeout(RequestId),

    #[error("Proof request with request_id: {0} is unfulfillable")]
    ProofRequestUnfulfillable(RequestId),

    #[error("Proof request with request_id {0} error: {1:?}")]
    Proving(RequestId, String),
}
