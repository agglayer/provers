use proposer_client::error::Error as ProposerClientError;
use proposer_db_client::Error as ProposerDBError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    AlloyProviderError(eyre::Error),

    #[error("Proposer client error: {0}")]
    Client(#[from] ProposerClientError),

    #[error("Database operation failed: {0}")]
    Database(#[from] ProposerDBError),

    #[error("Unsupported aggregation proof mode {0:?}")]
    UnsupportedAggregationProofMode(sp1_sdk::SP1ProofMode),

    #[error("Failure on the deserialization of the FEP public values")]
    FepPublicValuesDeserializeFailure(#[source] alloy_sol_types::Error),

    #[error("Failed to initialize L2 consensus layer client")]
    L2ConsensusLayerClientInit(#[source] jsonrpsee::core::client::Error),

    #[error("Failed to fetch safe head at L1 block")]
    L2SafeHeadFetch(#[source] jsonrpsee::core::client::Error),

    #[error(transparent)]
    Other(eyre::Error),
}
