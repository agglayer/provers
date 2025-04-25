use std::sync::Arc;
use std::time::Duration;

use alloy_primitives::B256;
use anyhow::Context;
use bincode::Options;
use sp1_sdk::{
    CpuProver, Prover as _, SP1ProofWithPublicValues, SP1ProvingKey, SP1VerificationError,
    SP1VerifyingKey,
};

use crate::aggregation_prover::AggregationProver;
use crate::rpc::MockProofProposerRequest;

pub struct MockGrpcProver<Proposer> {
    proposer_rpc: Arc<Proposer>,
    sp1_prover: CpuProver,
}

impl<Proposer> MockGrpcProver<Proposer>
where
    Proposer: crate::rpc::AggregationProofProposer + Sync + Send,
{
    pub fn new(proposer: Arc<Proposer>) -> anyhow::Result<MockGrpcProver<Proposer>> {
        Ok(MockGrpcProver {
            proposer_rpc: proposer,
            sp1_prover: sp1_sdk::CpuProver::mock(),
        })
    }
}

#[tonic::async_trait]
impl<Proposer> AggregationProver for MockGrpcProver<Proposer>
where
    Proposer: crate::rpc::AggregationProofProposer + Sync + Send,
{
    fn compute_pkey_vkey(&self, program: &[u8]) -> (SP1ProvingKey, SP1VerifyingKey) {
        self.sp1_prover.setup(program)
    }

    async fn wait_for_proof(
        &self,
        request_id: B256,
        _timeout: Option<Duration>,
    ) -> anyhow::Result<SP1ProofWithPublicValues> {
        let proof_id: i64 = i64::from_be_bytes(request_id[24..].try_into()?);
        debug_assert!(request_id[..24].iter().all(|v| *v == 0));

        let response = self
            .proposer_rpc
            .get_mock_proof(MockProofProposerRequest {
                proof_id: crate::ProofId(proof_id),
            })
            .await?;

        let proof = bincode::DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
            .deserialize(&response.proof)
            .with_context(|| format!("deserializing proof {request_id}"))?;

        Ok(proof)
    }

    fn verify_aggregated_proof(
        &self,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), SP1VerificationError> {
        self.sp1_prover.verify(proof, vkey)
    }
}
