#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ProposerService(#[from] proposer_service::Error),

    #[error(transparent)]
    AggchainProofBuilder(#[from] aggchain_proof_builder::Error),
}
