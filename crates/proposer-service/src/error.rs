use alloy_primitives::B256;
use proposer_client::error::Error as ProposerClientError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    AlloyProviderError(anyhow::Error),

    #[error("Proposer client error: {0}")]
    Client(#[from] ProposerClientError),

    #[error("Unsupported aggregated span proof mode {0:?}")]
    UnsupportedAggProofMode(sp1_sdk::SP1ProofMode),

    #[error("Aggregated span proof vkey mismatch (got: {got:?}, expected: {expected:?})")]
    AggProofVKeyMismatch { got: B256, expected: B256 },
}
