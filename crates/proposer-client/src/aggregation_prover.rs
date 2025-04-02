use std::time::Duration;

use alloy_primitives::B256;
use sp1_sdk::{SP1ProofWithPublicValues, SP1VerificationError, SP1VerifyingKey};

/// This prover waits for the SP1 cluster generated
/// AggregationProof based on the proof id.
#[tonic::async_trait]
pub trait AggregationProver {
    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> anyhow::Result<SP1ProofWithPublicValues>;

    fn verify_aggregated_proof(
        &self,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), SP1VerificationError>;
}
