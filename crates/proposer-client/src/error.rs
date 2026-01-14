use crate::RequestId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Proof request with request_id {0} error: {1:?}")]
    Proving(RequestId, String),

    #[error("Proof verification error")]
    Verification {
        request_id: RequestId,
        source: eyre::Report,
    },
}
