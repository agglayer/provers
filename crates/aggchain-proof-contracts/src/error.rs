use aggchain_proof_core::bridge::static_call::StaticCallStage;
use sp1_cc_host_executor::HostError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to create alloy node provider")]
    ProviderInitializationError(#[source] eyre::Error),

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

    #[error("Error retrieving rollup config hash from the op succinct config")]
    RollupConfigHashError(#[source] alloy::contract::Error),

    #[error("Error retrieving aggchain vkey")]
    AggchainVKeyRetrievalError(#[source] alloy::contract::Error),

    #[error("Invalid host static call at stage: {stage:?}")]
    InvalidHostStaticCall {
        source: eyre::Report,
        stage: StaticCallStage,
    },

    #[error("Invalid sketch finalization for the pre L2 block.")]
    InvalidPreBlockSketchFinalization(#[source] HostError),

    #[error("Invalid sketch finalization for the new L2 block.")]
    InvalidNewBlockSketchFinalization(#[source] HostError),

    #[error("Failure on the initialization of the HostExecutor for pre L2 block.")]
    HostExecutorPreBlockInitialization(#[source] HostError),

    #[error("Failure on the initialization of the HostExecutor for new L2 block.")]
    HostExecutorNewBlockInitialization(#[source] HostError),

    #[error("Unable to retrieve trusted sequencer address")]
    UnableToRetrieveTrustedSequencerAddress(#[source] alloy::contract::Error),

    #[error("Invalid evm sketch genesis input: {0}")]
    InvalidEvmSketchGenesisInput(String),

    #[error("Invalid op succinct config name input: {0}")]
    InvalidOpSuccinctConfigName(String),

    #[error(transparent)]
    Other(eyre::Report),
}
