use std::time::Duration;

use alloy_primitives::B256;
use anyhow::Context;
use sp1_sdk::{
    CpuProver, Prover as _, SP1ProofWithPublicValues, SP1ProvingKey, SP1VerificationError,
    SP1VerifyingKey,
};

use crate::aggregation_prover::AggregationProver;

pub struct MockProver {
    rpc_url: String,
    sp1_prover: CpuProver,
}

impl MockProver {
    pub fn new(endpoint: &str) -> anyhow::Result<MockProver> {
        Ok(MockProver {
            rpc_url: endpoint.to_string(),
            sp1_prover: sp1_sdk::CpuProver::new(),
        })
    }
}

#[tonic::async_trait]
impl AggregationProver for MockProver {
    fn compute_pkey_vkey(&self, program: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey) {
        self.sp1_prover.setup(program)
    }

    async fn wait_for_proof(
        &self,
        _request_id: B256,
        _timeout: Option<Duration>,
    ) -> anyhow::Result<SP1ProofWithPublicValues> {
        todo!() // self.wait_proof(request_id, timeout).await
    }

    fn verify_aggregated_proof(
        &self,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), SP1VerificationError> {
        self.sp1_prover.verify(proof, vkey)
    }
}
