use proposer_client::error::Error as ProposerClientError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    AlloyProviderError(anyhow::Error),

    #[error("Proposer client error: {0}")]
    Client(#[from] ProposerClientError),
}
