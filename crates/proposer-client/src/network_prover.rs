use std::time::Duration;

use alloy_primitives::B256;
use anyhow::Context;
use sp1_sdk::{
    NetworkProver, Prover, SP1ProofWithPublicValues, SP1ProvingKey, SP1VerificationError,
    SP1VerifyingKey,
};

use crate::aggregation_prover::AggregationProver;

#[tonic::async_trait]
impl AggregationProver for NetworkProver {
    fn compute_pkey_vkey(&self, program: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey) {
        self.setup(program)
    }

    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> anyhow::Result<SP1ProofWithPublicValues> {
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

pub fn new_network_prover(endpoint: &url::Url) -> anyhow::Result<NetworkProver> {
    Ok(sp1_sdk::ProverClient::builder()
        .network()
        .rpc_url(endpoint.as_str())
        .private_key(&std::env::var("NETWORK_PRIVATE_KEY").context(
            "Failed to get NETWORK_PRIVATE_KEY, when building NetworkProver for proposer-client",
        )?)
        .build())
}
