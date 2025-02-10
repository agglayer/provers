#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Proposer service error: {0}")]
    ProposerService(#[from] proposer_service::Error),

    #[error("Aggchain proof builder error: {0}")]
    AggchainProofBuilder(#[from] aggchain_proof_builder::Error),
}
