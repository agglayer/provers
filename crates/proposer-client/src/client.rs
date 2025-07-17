use std::{sync::Arc, time::Duration};

use educe::Educe;
use sp1_sdk::{SP1ProofWithPublicValues, SP1VerifyingKey};

use crate::{
    aggregation_prover::AggregationProver,
    error,
    rpc::{
        AggregationProofProposer, AggregationProofProposerRequest, AggregationProofProposerResponse,
    },
    Error, ProposerClient, RequestId,
};

/// Implementation of the proposer client.
/// The Proposer client is responsible for retrieval of the AggregationProof.
/// AggregationProof is the aggregated proof of the multiple
/// block span full execution proofs.
///
/// The proposer client communicates with the proposer API to
/// request creation of the AggregationProof (getting the proof ID in return),
/// and directly communicates with the SP1 cluster using NetworkProver
/// to retrieve the generated proof.
#[derive(Educe)]
#[educe(Clone(bound()))]
pub struct Client<Proposer, Prover> {
    proposer_rpc: Arc<Proposer>,
    prover_rpc: Arc<Prover>,
    proving_timeout: Option<Duration>,
}

impl<Proposer, Prover> Client<Proposer, Prover> {
    #[allow(clippy::result_large_err)]
    pub fn new(
        proposer: Arc<Proposer>,
        prover: Prover,
        proving_timeout: Option<Duration>,
    ) -> Result<Self, error::Error> {
        Ok(Self {
            proposer_rpc: proposer,
            prover_rpc: Arc::new(prover),
            proving_timeout,
        })
    }
}

#[async_trait::async_trait]
impl<Proposer, Prover> ProposerClient for Client<Proposer, Prover>
where
    Proposer: AggregationProofProposer + Sync + Send,
    Prover: AggregationProver + Sync + Send,
{
    async fn request_agg_proof(
        &self,
        request: AggregationProofProposerRequest,
    ) -> Result<AggregationProofProposerResponse, Error> {
        self.proposer_rpc.request_agg_proof(request).await
    }

    async fn wait_for_proof(
        &self,
        request_id: RequestId,
    ) -> Result<SP1ProofWithPublicValues, Error> {
        self.prover_rpc
            .wait_for_proof(request_id.0, self.proving_timeout)
            .await
            .map_err(|e| Error::Proving(request_id, e.to_string()))
    }

    fn verify_agg_proof(
        &self,
        request_id: RequestId,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), Error> {
        self.prover_rpc
            .verify_aggregated_proof(proof, vkey)
            .map_err(|source| Error::Verification { request_id, source })
    }
}
