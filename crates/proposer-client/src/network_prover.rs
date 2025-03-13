use std::time::Duration;

use alloy_primitives::B256;
use anyhow::{Context, Error};
use sp1_sdk::{NetworkProver, SP1ProofWithPublicValues};

/// This prover waits for the SP1 cluster generated
/// AggSpanProof based on the proof id.
#[tonic::async_trait]
pub trait AggSpanProver {
    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> Result<SP1ProofWithPublicValues, anyhow::Error>;
}

#[tonic::async_trait]
impl AggSpanProver for NetworkProver {
    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> Result<SP1ProofWithPublicValues, Error> {
        self.wait_proof(request_id, timeout).await
    }
}

pub fn new_network_prover(endpoint: &str) -> anyhow::Result<NetworkProver> {
    Ok(sp1_sdk::ProverClient::builder()
        .network()
        .rpc_url(endpoint)
        .private_key(&std::env::var("PROPOSER_NETWORK_PRIVATE_KEY").context(
            "Failed to get PROPOSER_NETWORK_PRIVATE_KEY, when building NetworkProver for \
             proposer-client",
        )?)
        .build())
}
