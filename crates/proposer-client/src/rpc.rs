use std::fmt::Display;

use alloy_primitives::B256;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use tracing::info;

use crate::{error::Error, ProposerRequest};

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
    client: HttpClient,
}

impl ProposerRpcClient {
    pub fn new(rpc_endpoint: &str) -> Result<Self, Error> {
        let client = HttpClient::builder()
            .build(rpc_endpoint)
            .map_err(Error::UnableToCreateRPCClient)?;

        Ok(ProposerRpcClient { client })
    }
}

#[tonic::async_trait]
impl AggSpanProofProposer for ProposerRpcClient {
    async fn request_agg_proof(
        &self,
        request: AggSpanProofProposerRequest,
    ) -> Result<AggSpanProofProposerResponse, Error> {
        let params = rpc_params![
            request.start,
            request.end,
            request.l1_block_number,
            request.l1_block_hash
        ];
        let proof_response: AggSpanProofProposerResponse = self
            .client
            .request("proofs_requestAggProof", params)
            .await
            .map_err(Error::AggProofRequestFailed)?;

        info!(
            proof_id = proof_response.to_string(),
            "agg proof request submitted"
        );

        Ok(proof_response)
    }
}

/// Request format for the proposer `request_agg_proof`
#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AggSpanProofProposerRequest {
    // Starting block number to request proof from
    #[serde(rename = "startBlock")]
    pub start: u64,
    // Maximum block number on which the proof needs to be aggregated
    #[serde(rename = "maxBlock")]
    pub end: u64,
    pub l1_block_number: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub l1_block_hash: B256,
}

impl From<AggSpanProofProposerRequest> for ProposerRequest {
    fn from(request: AggSpanProofProposerRequest) -> Self {
        ProposerRequest {
            start_block: request.start,
            max_block: request.end,
            l1_block_number: request.l1_block_number,
        }
    }
}

/// Response for the external proposer `request_span_proof` call
#[derive(Serialize, Deserialize, Debug)]
pub struct AggSpanProofProposerResponse {
    #[serde(rename = "proof_request_id")]
    pub proof_id: B256,
    pub start_block: u64,
    pub end_block: u64,
}

impl Display for AggSpanProofProposerResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.proof_id)
    }
}
