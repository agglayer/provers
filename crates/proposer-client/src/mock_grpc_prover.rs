use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};

use alloy_primitives::B256;
use eyre::{eyre, Context};
use prover_executor::{sp1_async, sp1_fast};
use sp1_sdk::{
    MockProver, Prover as _, ProvingKey, SP1ProofWithPublicValues, SP1ProvingKey, SP1VerifyingKey,
};

use crate::{aggregation_prover::AggregationProver, rpc::MockProofProposerRequest};

pub struct MockGrpcProver<Proposer> {
    proposer_rpc: Arc<Proposer>,
    sp1_prover: MockProver,
}

impl<Proposer> MockGrpcProver<Proposer>
where
    Proposer: crate::rpc::AggregationProofProposer + Sync + Send,
{
    pub async fn new(proposer: Arc<Proposer>) -> MockGrpcProver<Proposer> {
        MockGrpcProver {
            proposer_rpc: proposer,
            sp1_prover: sp1_sdk::ProverClient::builder().mock().build().await,
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
        let proving_key = sp1_async(AssertUnwindSafe(async {
            self.sp1_prover.setup(program.into()).await
        }))
        .await?
        .map_err(|error| eyre!(error.to_string()))?;
        let verifying_key = proving_key.verifying_key().clone();
        Ok((proving_key, verifying_key))
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
        sp1_fast(AssertUnwindSafe(|| {
            self.sp1_prover.verify(proof, vkey, None)
        }))
        .context("Verifying aggregated proof")?
        .context("Verifying aggregated proof")
    }
}
