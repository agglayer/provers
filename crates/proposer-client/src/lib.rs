use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sp1_sdk::SP1ProofWithPublicValues;

pub use crate::config::ProposerClientConfig;
pub use crate::error::Error;
use crate::network_prover::AggSpanProofProver;
use crate::rpc::AggSpanProofProposer;

pub mod config;
pub mod error;
pub mod network_prover;
pub mod rpc;

/// The Proposer client is responsible for retrieval of the AggProof.
/// AggProof is the aggregated proof of the multiple
/// block span full execution proofs.
///
/// The proposer client communicates with the proposer API to
/// request creation of the AggProof (getting the proof ID in return),
/// and directly communicates with the SP1 cluster using NetworkProver
/// to retrieve the generated proof.
#[derive(Clone)]
pub struct ProposerClient<Proposer, Prover> {
    proposer: Arc<Proposer>,
    prover: Arc<Prover>,
    proving_timeout: Option<Duration>,
}

impl<Proposer, Prover> ProposerClient<Proposer, Prover>
where
    Proposer: AggSpanProofProposer,
    Prover: AggSpanProofProver,
{
    pub fn new(
        proposer: Proposer,
        prover: Prover,
        timeout: Option<Duration>,
    ) -> Result<Self, error::Error> {
        Ok(ProposerClient {
            proposer: Arc::new(proposer),
            prover: Arc::new(prover),
            proving_timeout: timeout,
        })
    }

    pub async fn request_agg_proof(&mut self, request: Request) -> Result<ProofId, Error> {
        self.proposer
            .request_agg_proof(request.into())
            .await?
            .try_into()
    }

    pub async fn wait_for_proof(
        &mut self,
        proof_id: ProofId,
    ) -> Result<SP1ProofWithPublicValues, Error> {
        let request_id = proof_id.0;

        self.prover
            .wait_for_proof(request_id, self.proving_timeout)
            .await
            .map_err(|e| Error::Proving(proof_id, e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub start_block: u64,
    pub max_block: u64,
    pub l1_block_number: u64,
    pub l1_block_hash: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    proofs: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProofId(alloy_primitives::B256);

impl Display for ProofId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}
