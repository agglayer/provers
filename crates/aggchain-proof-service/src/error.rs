#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to create alloy provider")]
    AlloyProviderInitializationFailed(#[source] anyhow::Error),

    #[error("Unable to setup proposer service")]
    ProposerServiceInitFailed(#[source] proposer_service::Error),

    #[error("Proposer service returned an error during operation")]
    ProposerServiceError(#[source] proposer_service::Error),

    #[error("Proposer service request failed")]
    ProposerServiceRequestFailed(#[source] proposer_service::Error),

    #[error("Unable to setup aggchain proof builder")]
    AggchainProofBuilderInitFailed(#[source] aggchain_proof_builder::Error),

    #[error("Aggchain proof builder service request failed")]
    AggchainProofBuilderRequestFailed(#[source] aggchain_proof_builder::Error),

    #[error("Unable to setup aggchain contracts client")]
    ContractsClientInitFailed(#[source] aggchain_proof_contracts::Error),

    #[error("Unable to serialize custom chain data")]
    UnableToSerializeCustomChainData(#[source] bincode::Error),

    #[error("Unable to resolve aggchain proof vkey")]
    AggchainProofVkeyResolveFailed(#[source] aggchain_proof_contracts::Error),
}
