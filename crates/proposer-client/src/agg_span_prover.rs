use std::time::Duration;

use alloy_primitives::B256;
use anyhow::Error;
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

pub fn new_agg_span_prover(endpoint: &str) -> NetworkProver {
    sp1_sdk::ProverClient::builder()
        .network()
        .rpc_url(endpoint)
        .build()
}
