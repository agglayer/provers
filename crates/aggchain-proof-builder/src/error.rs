use crate::WitnessGeneration;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to retrieve l2 chain data")]
    L2ChainDataRetrievalError(#[source] aggchain_proof_contracts::Error),

    #[error("Failed to retrieve l1 chain data")]
    L1ChainDataRetrievalError(#[source] aggchain_proof_contracts::Error),

    #[error("Prover executor returned an error")]
    ProverExecutorError(#[source] prover_executor::Error),

    #[error("Prover service returned the error: {0}")]
    ProverServiceError(String),

    #[error("Prover failed to prove the transaction")]
    ProverFailedToExecute(#[source] anyhow::Error),

    #[error("Generated proof is not Compressed one (STARK)")]
    GeneratedProofIsNotCompressed,

    #[error("Unable to serialize proof")]
    UnableToSerializeProof(#[source] bincode::Error),

    #[error("Prover witness generation error: {0}")]
    ProverWitnessGenerationError(#[source] WitnessGeneration),
}
