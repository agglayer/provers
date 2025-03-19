use std::fmt::Display;

use alloy_primitives::B256;
use tracing::info;

use crate::error::{self, Error, ProofRequestError};

mod proofs_service_types;

use proofs_service_types::ProofsClient;

/// Proposer client that requests the generation
/// of the AggSpanProof from the proposer and gets
/// proof_id in response.
#[tonic::async_trait]
pub trait AggSpanProofProposer {
    async fn request_agg_proof(
        &self,
        request: AggSpanProofProposerRequest,
    ) -> Result<AggSpanProofProposerResponse, Error>;
}

pub struct ProposerRpcClient {
    client: ProofsClient<tonic::transport::Channel>,
}

impl ProposerRpcClient {
    pub async fn new(rpc_endpoint: &str) -> Result<Self, Error> {
        let client = ProofsClient::connect(rpc_endpoint.to_string())
            .await
            .map_err(Error::Connect)?;

        Ok(ProposerRpcClient { client })
    }
}

#[tonic::async_trait]
impl AggSpanProofProposer for ProposerRpcClient {
    async fn request_agg_proof(
        &self,
        request: AggSpanProofProposerRequest,
    ) -> Result<AggSpanProofProposerResponse, Error> {
        let request = proofs_service_types::grpc::AggProofRequest::try_from(request)
            .map_err(|e| Error::Requesting(ProofRequestError::ComposingRequest(e)))?;

        let mut client = self.client.clone();
        let response: AggSpanProofProposerResponse = client
            .request_agg_proof(request)
            .await
            .map_err(|e| Error::Requesting(ProofRequestError::Grpc(e)))?
            .into_inner()
            .try_into()
            .map_err(Error::Requesting)?;

        info!(
            proof_id = response.to_string(),
            "agg proof request submitted"
        );

        Ok(response)
    }
}

/// Request format for the proposer `request_agg_proof`
#[derive(Debug)]
pub struct AggSpanProofProposerRequest {
    // Starting block number to request proof from
    pub start_block: u64,
    // Maximum block number on which the proof needs to be aggregated
    pub max_block: u64,
    pub l1_block_number: u64,
    pub l1_block_hash: B256,
}

/// Response for the external proposer `request_span_proof` call
#[derive(Debug)]
pub struct AggSpanProofProposerResponse {
    pub proof_id: B256,
    pub start_block: u64,
    pub end_block: u64,
}

impl Display for AggSpanProofProposerResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.proof_id.fmt(f)
    }
}
