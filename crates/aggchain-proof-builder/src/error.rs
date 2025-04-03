use aggkit_prover_types::vkey_hash::VKeyHash;

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

    #[error("Unable to serialize vkey")]
    UnableToSerializeVkey(#[source] bincode::Error),

    #[error("Prover witness generation error: {0}")]
    ProverWitnessGenerationError(#[source] WitnessGeneration),

    #[error("Prover service is not ready")]
    ProverServiceReadyError(#[source] tower::BoxError),

    #[error("Mismatch on the aggregation vkey. got: {got:?}, expected: {expected:?}")]
    MismatchAggregationVkeyHash { got: VKeyHash, expected: VKeyHash },
}
