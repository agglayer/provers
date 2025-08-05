use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};

use alloy_primitives::B256;
use eyre::Context;
use prover_executor::{sp1_block_in_place, sp1_fast};
use sp1_sdk::{CpuProver, Prover as _, SP1ProofWithPublicValues, SP1ProvingKey, SP1VerifyingKey};

use crate::{aggregation_prover::AggregationProver, rpc::MockProofProposerRequest};

pub struct MockGrpcProver<Proposer> {
    proposer_rpc: Arc<Proposer>,
    sp1_prover: CpuProver,
}

impl<Proposer> MockGrpcProver<Proposer>
where
    Proposer: crate::rpc::AggregationProofProposer + Sync + Send,
{
    pub fn new(proposer: Arc<Proposer>) -> MockGrpcProver<Proposer> {
        MockGrpcProver {
            proposer_rpc: proposer,
            sp1_prover: sp1_sdk::CpuProver::mock(),
        }
    }
}

#[tonic::async_trait]
impl<Proposer> AggregationProver for MockGrpcProver<Proposer>
where
    Proposer: crate::rpc::AggregationProofProposer + Sync + Send,
{
    async fn compute_pkey_vkey(
        &self,
        program: &[u8],
    ) -> eyre::Result<(SP1ProvingKey, SP1VerifyingKey)> {
        // TODO: Figure out a way to kill this struct if there's an unwind, and start
        // again with a fresh Prover
        sp1_block_in_place(AssertUnwindSafe(|| self.sp1_prover.setup(program)))
    }

    async fn wait_for_proof(
        &self,
        request_id: B256,
        _timeout: Option<Duration>,
    ) -> eyre::Result<SP1ProofWithPublicValues> {
        let proof_id: i64 = i64::from_be_bytes(request_id[24..].try_into()?);
        debug_assert!(request_id[..24].iter().all(|v| *v == 0));

        let response = self
            .proposer_rpc
            .get_mock_proof(MockProofProposerRequest {
                proof_id: crate::MockProofId(proof_id),
            })
            .await?;

        let proof =
            sp1_fast(|| agglayer_interop_types::bincode::default().deserialize(&response.proof))
                .with_context(|| format!("deserializing proof {request_id}"))?
                .with_context(|| format!("deserializing proof {request_id}"))?;

        Ok(proof)
    }

    fn verify_aggregated_proof(
        &self,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> eyre::Result<()> {
        // TODO: kill sp1 prover if there's a panic, to avoid any interior mutability on
        // panic issues?
        sp1_fast(AssertUnwindSafe(|| self.sp1_prover.verify(proof, vkey)))
            .context("Verifying aggregated proof")?
            .context("Verifying aggregated proof")
    }
}
