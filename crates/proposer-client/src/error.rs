#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Reqwest http error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}
