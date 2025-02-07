use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{error::Error, ProofId, Request};

/// Proposer client that requests the generation
/// of the AggProof from the proposer and gets
/// proof_id in response.
#[tonic::async_trait]
pub trait ProposerAggProofClient {
    async fn request_agg_proof(
        &self,
        request: ProposerAggProofRequest,
    ) -> Result<ProposerAggProofResponse, Error>;
}

pub struct ProposerRpcClient {
    client: reqwest::Client,
    url: String,
}

impl ProposerRpcClient {
    pub fn new(rpc_endpoint: &str) -> Result<Self, Error> {
        let headers = reqwest::header::HeaderMap::new();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(ProposerRpcClient {
            client,
            url: rpc_endpoint.to_owned(),
        })
    }
}

#[tonic::async_trait]
impl ProposerAggProofClient for ProposerRpcClient {
    async fn request_agg_proof(
        &self,
        request: ProposerAggProofRequest,
    ) -> Result<ProposerAggProofResponse, Error> {
        let proof_response = self
            .client
            .post(format!("{}/request_agg_proof", self.url.as_str()))
            .json(&request)
            .send()
            .await?
            .json::<ProposerAggProofResponse>()
            .await?;

        info!(
            proof_id = proof_response.to_string(),
            "agg proof request submitted"
        );

        Ok(proof_response)
    }
}

/// Request format for the proposer `request_agg_proof`
#[derive(Deserialize, Serialize, Debug)]
pub struct ProposerAggProofRequest {
    pub start: u64,
    pub end: u64,
    pub l1_block_number: u64,
    pub l1_block_hash: Vec<u8>,
}

impl From<ProposerAggProofRequest> for Request {
    fn from(request: ProposerAggProofRequest) -> Self {
        Request {
            start_block: request.start,
            max_block: request.end,
            l1_block_number: request.l1_block_number,
            l1_block_hash: request.l1_block_hash,
        }
    }
}

impl From<Request> for ProposerAggProofRequest {
    fn from(request: Request) -> Self {
        ProposerAggProofRequest {
            start: request.start_block,
            end: request.max_block,
            l1_block_number: request.l1_block_number,
            l1_block_hash: request.l1_block_hash,
        }
    }
}

/// Response for the proposer `request_span_proof`
#[derive(Serialize, Deserialize, Debug)]
pub struct ProposerAggProofResponse {
    pub proof_id: alloy_primitives::B256,
}

impl Display for ProposerAggProofResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.proof_id)
    }
}

impl TryFrom<ProposerAggProofResponse> for ProofId {
    type Error = crate::Error;

    fn try_from(proof_response: ProposerAggProofResponse) -> Result<Self, Error> {
        let bytes = proof_response
            .proof_id
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidProofId(proof_response.proof_id.to_string()))?;
        Ok(ProofId(bytes))
    }
}

impl From<ProofId> for ProposerAggProofResponse {
    fn from(proof_id: ProofId) -> Self {
        ProposerAggProofResponse {
            proof_id: proof_id.0,
        }
    }
}
