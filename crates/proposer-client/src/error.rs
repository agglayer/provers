use crate::ProofId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to create RPC client")]
    UnableToCreateRPCClient(#[source] jsonrpsee::core::client::Error),

    #[error("An error occurred while requesting an aggregated proof")]
    AggProofRequestFailed(#[source] jsonrpsee::core::client::Error),

    #[error("Reqwest http error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Invalid proof_id: {0:?}")]
    InvalidProofId(String),

    #[error("Proof request with proof_id: {0} timeout")]
    Timeout(ProofId),

    #[error("Proof request with proof_id: {0} is unfulfillable")]
    ProofRequestUnfulfillable(ProofId),

    #[error("Proof request with proof_id {0} error: {1:?}")]
    Proving(ProofId, String),
}
