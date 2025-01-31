use crate::ProofId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Reqwest http error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Invalid proof_id: {0}")]
    InvalidProofId(ProofId),
    #[error("Request timeout for the proof_id: {0}")]
    Timeout(ProofId),
    #[error("Proof request with proof_id: {0} is unfullfilable")]
    ProofRequestUnfullfilable(ProofId),
    #[error("Error proving proof_id {0}: {1:?}")]
    Proving(ProofId, String),
}
