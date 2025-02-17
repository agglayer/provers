use alloy::transports::{RpcError, TransportErrorKind};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AlloyProviderError(anyhow::Error),

    #[error(transparent)]
    AlloyRpcTransportError(#[from] RpcError<TransportErrorKind>),

    #[error(transparent)]
    ProofGenerationError(#[from] aggchain_proof_core::error::ProofError),

    #[error(transparent)]
    ProverExecutorError(#[from] prover_executor::Error),

    #[error("Prover service error:: {0}")]
    ProverServiceError(String),
}
