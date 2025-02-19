#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to create alloy provider")]
    ProviderInitializationError(#[source] anyhow::Error),

    #[error("Error with contract ABI file")]
    ContractAbiFileError(#[source] std::io::Error),

    #[error("Error processing contract ABI JSON")]
    ContractAbiJsonError(#[source] serde_json::Error),

    #[error("Invalid contract address")]
    InvalidContractAddress(#[source] alloy::hex::FromHexError),

    #[error("Unable to retrieve bridge address from the global exit root manager contract")]
    BridgeAddressError(#[source] alloy::contract::Error),
}
