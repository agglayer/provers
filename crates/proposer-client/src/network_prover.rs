use std::{panic::AssertUnwindSafe, time::Duration};

use alloy_primitives::B256;
use eyre::{eyre, Context};
use prover_executor::{sp1_async, sp1_block_in_place, sp1_fast};
use sp1_sdk::{NetworkProver, Prover, SP1ProofWithPublicValues, SP1ProvingKey, SP1VerifyingKey};

use crate::aggregation_prover::AggregationProver;

#[tonic::async_trait]
impl AggregationProver for NetworkProver {
    async fn compute_pkey_vkey(
        &self,
        program: &[u8],
    ) -> eyre::Result<(SP1ProvingKey, SP1VerifyingKey)> {
        // TODO: Figure out a way to kill this struct if there's an unwind, and start
        // again with a fresh Prover
        sp1_block_in_place(AssertUnwindSafe(|| self.setup(program)))
    }

    async fn wait_for_proof(
        &self,
        request_id: B256,
        timeout: Option<Duration>,
    ) -> eyre::Result<SP1ProofWithPublicValues> {
        // TODO: Figure out a way to kill this struct if there's an unwind, and start
        // again with a fresh Prover
        sp1_async(AssertUnwindSafe(self.wait_proof(request_id, timeout)))
            .await?
            .map_err(|e| eyre!(e))
            .context("Failed waiting for proof")
    }

    fn verify_aggregated_proof(
        &self,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> eyre::Result<()> {
        // TODO: Figure out a way to kill this struct if there's an unwind, and start
        // again with a fresh Prover
        sp1_fast(AssertUnwindSafe(|| self.verify(proof, vkey)))?.map_err(eyre::Report::from)
    }
}

pub fn new_network_prover<T: AsRef<str>>(endpoint: T) -> eyre::Result<NetworkProver> {
    let endpoint = endpoint.as_ref();
    sp1_fast(move || {
        Ok(sp1_sdk::ProverClient::builder()
            .network()
            .rpc_url(endpoint)
            .private_key(&std::env::var("NETWORK_PRIVATE_KEY").context(
                "Failed to get NETWORK_PRIVATE_KEY, when building NetworkProver for \
                 proposer-client",
            )?)
            .build())
    })?
}
