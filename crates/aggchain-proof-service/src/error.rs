#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to create alloy provider")]
    AlloyProviderInitializationFailed(#[source] eyre::Error),

    #[error("Unable to setup proposer service")]
    ProposerServiceInitFailed(#[source] proposer_service::Error),

    #[error("Proposer service returned an error during operation")]
    ProposerServiceError(#[source] proposer_service::Error),

    #[error("Proposer service request failed")]
    ProposerServiceRequestFailed(#[source] proposer_service::Error),

    #[error("Unable to setup aggchain proof builder")]
    AggchainProofBuilderInitFailed(#[source] eyre::Error),

    #[error("Unable to poll for aggchain proof builder readiness")]
    AggchainProofBuilderPollReadyFailed(#[source] aggchain_proof_builder::Error),

    #[error("Aggchain proof builder service request failed")]
    AggchainProofBuilderRequestFailed(#[source] aggchain_proof_builder::Error),

    #[error("Unable to setup aggchain contracts client")]
    ContractsClientInitFailed(#[source] aggchain_proof_contracts::Error),

    #[error("Unable to resolve aggchain proof vkey")]
    AggchainProofVkeyResolveFailed(#[source] aggchain_proof_contracts::Error),

    #[error(
        "Proposer  breaks import/unclaim pair: end_block {new_end_block}, \
         global_index={global_index}, claim_block={import_block}, unclaim_block={unclaim_block} "
    )]
    BrokenImportUnclaimPair {
        global_index: alloy_primitives::U256,
        import_block: u64,
        unclaim_block: u64,
        new_end_block: u64,
    },
}
