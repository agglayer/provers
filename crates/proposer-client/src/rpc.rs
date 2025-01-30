use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{error::Error, ProofId, Request};

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

    pub async fn request_span_proof(
        &self,
        request: SpanProofRequest,
    ) -> Result<ProofResponse, Error> {
        let proof_response = self
            .client
            .post(format!("{}/request_span_proof", self.url.as_str()))
            .json(&request)
            .send()
            .await?
            .json::<ProofResponse>()
            .await?;

        info!(
            proof_id = proof_response.to_string(),
            "span proof request submitted"
        );

        Ok(proof_response)
    }

    pub async fn check_status(&self, proof_id: &ProofId) -> Result<ProofStatus, Error> {
        let proof_id_str = hex::encode(&proof_id.0);
        let proof_status = self
            .client
            .get(format!("{}/status/{}", self.url.as_str(), &proof_id_str,))
            .send()
            .await?
            .json::<ProofStatus>()
            .await?;

        debug!(proof_id = proof_id_str, "status: {:?}", proof_status);

        Ok(proof_status)
    }
}

// Proposer structures

#[derive(Deserialize, Serialize, Debug)]
pub struct SpanProofRequest {
    pub start: u64,
    pub end: u64,
}

impl From<SpanProofRequest> for Request {
    fn from(request: SpanProofRequest) -> Self {
        Request {
            start_block: request.start,
            end_block: request.end,
        }
    }
}

impl From<Request> for SpanProofRequest {
    fn from(request: Request) -> Self {
        SpanProofRequest {
            start: request.start_block,
            end: request.end_block,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProofResponse {
    pub proof_id: Vec<u8>,
}

impl Display for ProofResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.proof_id))
    }
}

impl From<ProofResponse> for ProofId {
    fn from(proof_response: ProofResponse) -> Self {
        ProofId(proof_response.proof_id)
    }
}

impl From<ProofId> for ProofResponse {
    fn from(proof_id: ProofId) -> Self {
        ProofResponse {
            proof_id: proof_id.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
/// The status of a proof request.
pub struct ProofStatus {
    pub fulfillment_status: i32,
    pub execution_status: i32,
    pub proof: Vec<u8>,
}
