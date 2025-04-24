use crate::RequestId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Proof request with request_id {0} error: {1:?}")]
    Proving(RequestId, String),

    #[error("Proof verification error")]
    Verification {
        request_id: RequestId,
        source: sp1_sdk::prover::SP1VerificationError,
    },

    #[error("Error requesting proof")]
    Requesting(#[source] ProofRequestError),

    #[error("Error initializing grpc connection")]
    Connect(tonic::transport::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ProofRequestError {
    #[error("Cannot parse grpc response")]
    ParsingResponse(#[source] GrpcConversionError),

    #[error("Request failed: {0}")]
    Failed(String),

    #[error("Grpc request error")]
    Grpc(#[source] tonic::Status),
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcConversionError {
    #[error("Conversion of `{field}` failed")]
    Conversion {
        field: &'static str,
        source: anyhow::Error,
    },
}
