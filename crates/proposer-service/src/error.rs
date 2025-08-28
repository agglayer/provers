use proposer_client::error::Error as ProposerClientError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    AlloyProviderError(anyhow::Error),

    #[error("Proposer client error: {0}")]
    Client(#[from] ProposerClientError),

    #[error("Unsupported aggregation proof mode {0:?}")]
    UnsupportedAggregationProofMode(sp1_sdk::SP1ProofMode),

    #[error("Failure on the deserialization of the FEP public values")]
    FepPublicValuesDeserializeFailure(#[source] alloy_sol_types::Error),

    #[error(transparent)]
    Other(eyre::Error),
}
