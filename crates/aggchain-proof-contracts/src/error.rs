#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to create alloy node provider")]
    ProviderInitializationError(#[source] anyhow::Error),

    #[error("Error processing contract ABI file")]
    ContractAbiFileError(#[source] std::io::Error),

    #[error("Error processing contract ABI JSON")]
    ContractAbiJsonError(#[source] serde_json::Error),

    #[error("Invalid contract address")]
    InvalidContractAddress(#[source] alloy::hex::FromHexError),

    #[error("Unable to retrieve zkevm bridge address from the global exit root manager contract")]
    BridgeAddressError(#[source] alloy::contract::Error),

    #[error("Unable to retrieve aggchain fep address from the polygon rollup manager contract")]
    AggchainFepAddressError(#[source] alloy::contract::Error),

    #[error("Error retrieving local exit root")]
    LocalExitRootError(#[source] alloy::contract::Error),

    #[error("Unable to setup async engine")]
    AsyncEngineSetupError(#[source] std::io::Error),

    #[error("Unable to create HTTP RPC rollup node client")]
    RollupNodeInitError(#[source] jsonrpsee::core::ClientError),

    #[error("Error retrieving l2 output at block from the node")]
    L2OutputAtBlockRetrievalError(#[source] jsonrpsee::core::ClientError),

    #[error("L2 output at block value is missing, field {0}")]
    L2OutputAtBlockValueMissing(String),

    #[error("Invalid L2 output at block, field {0}")]
    L2OutputAtBlockInvalidValue(String, #[source] alloy::hex::FromHexError),

    #[error("Error performing rollup manager rollup id to rollup data call")]
    InvalidRollupIdToRollupData(#[source] alloy::contract::Error),

    #[error("Error retrieving rollup config hash")]
    RollupConfigHashError(#[source] alloy::contract::Error),
}
