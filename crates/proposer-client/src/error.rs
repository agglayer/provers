use crate::ProofId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error requesting proof")]
    Requesting(#[source] ProofRequestError),

    #[error("Proof request with proof_id {0} error: {1:?}")]
    Proving(ProofId, String),

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
