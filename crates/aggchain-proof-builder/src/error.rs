#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to setup aggchain contracts client")]
    ContractsClientInitFailed(#[source] aggchain_proof_contracts::Error),

    #[error("Failed to retrieve l2 chain data")]
    L2ChainDataRetrievalError(#[source] aggchain_proof_contracts::Error),

    #[error("Prover executor returned an error")]
    ProverExecutorError(#[source] prover_executor::Error),

    #[error("Prover service returned the error: {0}")]
    ProverServiceError(String),
}
