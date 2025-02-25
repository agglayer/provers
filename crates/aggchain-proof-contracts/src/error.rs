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

    #[error("Unable to retrieve bridge address from the global exit root manager contract")]
    BridgeAddressError(#[source] alloy::contract::Error),

    #[error("Error retrieving local exit root")]
    LocalExitRootError(#[source] alloy::contract::Error),

    #[error("Unable to setup async engine")]
    AsyncEngineSetupError(#[source] std::io::Error),
}
