use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::config::ProposerClientConfig;
use crate::rpc::{ProofStatus, ProposerRpcClient};

mod config;
mod error;
mod rpc;

pub struct ProposerClient {
    rpc: ProposerRpcClient,
}

impl ProposerClient {
    pub fn new(config: ProposerClientConfig) -> Result<Self, error::Error> {
        let url = format!("http://{}:{}", config.host, config.port);
        Ok(ProposerClient {
            rpc: ProposerRpcClient::new(&url)?,
        })
    }

    pub async fn request_proofs(&mut self, request: Request) -> Result<ProofId, error::Error> {
        self.rpc
            .request_span_proof(request.into())
            .await
            .map(Into::into)
    }

    pub async fn check_status(&mut self, proof_id: &ProofId) -> Result<ProofStatus, error::Error> {
        self.rpc.check_status(proof_id).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    start_block: u64,
    end_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    proofs: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProofId(Vec<u8>);

impl Display for ProofId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}
