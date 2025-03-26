use std::time::Duration;

use alloy_primitives::B256;
use anyhow::{Context, Error};
use sp1_sdk::{
    NetworkProver, Prover, SP1ProofWithPublicValues, SP1VerificationError, SP1VerifyingKey,
};

/// This prover waits for the SP1 cluster generated
/// AggregationProof based on the proof id.
#[tonic::async_trait]
pub trait AggregationProver {
    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> Result<SP1ProofWithPublicValues, anyhow::Error>;

    fn verify_aggregated_proof(
        &self,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), SP1VerificationError>;
}

#[tonic::async_trait]
impl AggregationProver for NetworkProver {
    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> Result<SP1ProofWithPublicValues, Error> {
        self.wait_proof(request_id, timeout).await
    }

    fn verify_aggregated_proof(
        &self,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), SP1VerificationError> {
        self.verify(proof, vkey)
    }
}

pub fn new_network_prover(endpoint: &str) -> anyhow::Result<NetworkProver> {
    Ok(sp1_sdk::ProverClient::builder()
        .network()
        .rpc_url(endpoint)
        .private_key(&std::env::var("NETWORK_PRIVATE_KEY").context(
            "Failed to get NETWORK_PRIVATE_KEY, when building NetworkProver for proposer-client",
        )?)
        .build())
}
