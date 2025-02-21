#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AlloyProviderError(anyhow::Error),

    #[error(transparent)]
    CustomChainDataBuilderError(#[from] crate::service::customchaindata_builder::Error),

    #[error(transparent)]
    ProposerService(#[from] proposer_service::Error),

    #[error(transparent)]
    AggchainProofBuilder(#[from] aggchain_proof_builder::Error),
}
