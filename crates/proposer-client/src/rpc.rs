use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{error::Error, ProofId, Request};

pub(crate) struct ProposerRpcClient {
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

    pub async fn request_agg_proof(
        &self,
        request: ProposerAggProofRequest,
    ) -> Result<ProposerProofResponse, Error> {
        let proof_response = self
            .client
            .post(format!("{}/request_agg_proof", self.url.as_str()))
            .json(&request)
            .send()
            .await?
            .json::<ProposerProofResponse>()
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
pub struct ProposerProofResponse {
    pub proof_id: Vec<u8>,
}

impl Display for ProposerProofResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.proof_id))
    }
}

impl TryFrom<ProposerProofResponse> for ProofId {
    type Error = crate::Error;

    fn try_from(proof_response: ProposerProofResponse) -> Result<Self, Error> {
        let bytes: [u8; 32] = proof_response
            .proof_id
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidProofId(proof_response.proof_id))?;
        Ok(ProofId(bytes))
    }
}

impl From<ProofId> for ProposerProofResponse {
    fn from(proof_id: ProofId) -> Self {
        ProposerProofResponse {
            proof_id: proof_id.0.to_vec(),
        }
    }
}
