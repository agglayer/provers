use std::fmt::Display;
use std::time::Duration;

use alloy_primitives::B256;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use tracing::{error, info};

use crate::error::Error;

/// Proposer client that requests the generation
/// of the aggregation proof from the proposer and gets
/// request_id in response.
#[tonic::async_trait]
pub trait AggregationProofProposer {
    async fn request_agg_proof(
        &self,
        request: AggregationProofProposerRequest,
    ) -> Result<AggregationProofProposerResponse, Error>;
}

/// Request format for the proposer `proofs_requestAggProof`
#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AggregationProofProposerRequest {
    /// Starting block number to request proof from.
    #[serde(rename = "startBlock")]
    pub start_block: u64,
    /// Maximum block number for the aggregation proof.
    #[serde(rename = "maxBlock")]
    pub max_block: u64,
    /// L1 block number corresponding to max_block.
    pub l1_block_number: u64,
    /// L1 block hash.
    #[serde_as(as = "DisplayFromStr")]
    pub l1_block_hash: B256,
}

/// Response for the external proposer `request_span_proof` call
#[derive(Serialize, Deserialize, Debug)]
pub struct AggregationProofProposerResponse {
    /// Proof request_id, used to fetch the proof from the cluster.
    #[serde(rename = "proof_request_id")]
    pub request_id: B256,
    /// Start block for the aggregation proof.
    pub start_block: u64,
    /// End block for the aggregation proof.
    pub end_block: u64,
}

impl Display for AggregationProofProposerResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "start block: {}, end block: {}, request_id: {}",
            self.start_block, self.end_block, self.request_id
        )
    }
}

pub struct ProposerRpcClient {
    client: HttpClient,
}

impl ProposerRpcClient {
    pub fn new(rpc_endpoint: &str, timeout: Duration) -> Result<Self, Error> {
        let client = HttpClient::builder()
            .request_timeout(timeout)
            .build(rpc_endpoint)
            .map_err(Error::UnableToCreateRPCClient)?;

        Ok(ProposerRpcClient { client })
    }
}

#[tonic::async_trait]
impl AggregationProofProposer for ProposerRpcClient {
    async fn request_agg_proof(
        &self,
        request: AggregationProofProposerRequest,
    ) -> Result<AggregationProofProposerResponse, Error> {
        let params = rpc_params![
            request.start_block,
            request.max_block,
            request.l1_block_number,
            request.l1_block_hash
        ];

        let proof_response: AggregationProofProposerResponse = self
            .client
            .request("proofs_requestAggProof", params)
            .await
            .map_err(Error::AggProofRequestFailed)
            .inspect_err(|e| error!("proofs_requestAggProof failed, details: {e:?}"))?;

        info!(
            request_id = proof_response.to_string(),
            "agg proof request submitted"
        );

        Ok(proof_response)
    }
}
