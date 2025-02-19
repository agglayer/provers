use alloy::transports::{RpcError, TransportErrorKind};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to create aggchain contracts client")]
    ContractsClientInitializationFailed(#[source] aggchain_proof_contracts::Error),

    #[error(transparent)]
    AlloyRpcTransportError(#[from] RpcError<TransportErrorKind>),

    #[error(transparent)]
    ProofGenerationError(#[from] aggchain_proof_core::error::ProofError),

    #[error(transparent)]
    ProverExecutorError(#[from] prover_executor::Error),

    #[error("Prover service error:: {0}")]
    ProverServiceError(String),
}
