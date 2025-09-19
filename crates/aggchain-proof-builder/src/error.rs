use aggchain_proof_core::full_execution_proof::AggregationProofPublicValues;
use agglayer_interop::types::bincode;
use agglayer_primitives::vkey_hash::VKeyHash;

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
    ProverFailedToExecute(#[source] tower::BoxError),

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

    /// Mismatch on the aggregation proof public values between what we got from
    /// the contracts and what we expect from the proof public values.
    #[error(
        "Mismatch on the aggregation proof public values. expected by contract: \
         {expected_by_contract:?}, expected by verifier: {expected_by_verifier:?}"
    )]
    MismatchAggregationProofPublicValues {
        expected_by_contract: Box<AggregationProofPublicValues>,
        expected_by_verifier: Box<AggregationProofPublicValues>,
    },
    #[error("Unable to fetch trusted sequencer address")]
    UnableToFetchTrustedSequencerAddress(#[source] aggchain_proof_contracts::Error),

    #[error(transparent)]
    Other(eyre::Report),
}
