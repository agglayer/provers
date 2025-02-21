#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to create alloy provider")]
    AlloyProviderInitializationFailed(#[source] anyhow::Error),

    #[error("Unable to setup proposer service")]
    ProposerServiceInitFailed(#[source] proposer_service::Error),

    #[error("Unable to setup custom chain data builder")]
    CustomChainDataBuilderError(#[source] crate::service::customchaindata_builder::Error),

    #[error("Proposer service returned an error during operation")]
    ProposerServiceError(#[source] proposer_service::Error),

    #[error("Proposer service request failed")]
    ProposerServiceRequestFailed(#[source] proposer_service::Error),

    #[error("Unable to setup aggchain proof builder")]
    AggchainProofBuilderInitFailed(#[source] aggchain_proof_builder::Error),

    #[error("Aggchain proof builder service returned an error during operation")]
    AggchainProofBuilderServiceError(#[source] aggchain_proof_builder::Error),

    #[error("Aggchain proof builder service request failed")]
    AggchainProofBuilderRequestFailed(#[source] aggchain_proof_builder::Error),
}
