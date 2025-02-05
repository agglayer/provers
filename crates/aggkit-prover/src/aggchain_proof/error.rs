#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Proposer service error: {0}")]
    ProposerService(proposer_service::Error),

    #[error("Aggchain proof builder error: {0}")]
    AggchainProofBuilder(aggchain_proof_builder::Error),
}

impl From<proposer_service::Error> for Error {
    fn from(e: proposer_service::Error) -> Self {
        Self::ProposerService(e)
    }
}

impl From<aggchain_proof_builder::Error> for Error {
    fn from(e: aggchain_proof_builder::Error) -> Self {
        Self::AggchainProofBuilder(e)
    }
}
