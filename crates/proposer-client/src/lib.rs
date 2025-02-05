use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sp1_sdk::network::B256;
use sp1_sdk::{NetworkProver, ProverClient, SP1ProofWithPublicValues};

pub use crate::config::ProposerClientConfig;
pub use crate::error::Error;
use crate::rpc::ProposerRpcClient;

pub mod config;
pub mod error;
mod rpc;

#[derive(Clone)]
pub struct ProposerClient {
    rpc: Arc<ProposerRpcClient>,
    prover_client: Arc<NetworkProver>,
    proving_timeout: Duration,
}

impl ProposerClient {
    pub fn new(config: ProposerClientConfig) -> Result<Self, error::Error> {
        let proposer_url = format!("http://{}:{}", config.proposer_host, config.proposer_port);
        let cluster_url = format!(
            "http://{}:{}",
            config.sp1_cluster_host, config.sp1_cluster_port
        );

        let network_prover = ProverClient::builder()
            .network()
            .rpc_url(&cluster_url)
            .build();

        Ok(ProposerClient {
            rpc: Arc::new(ProposerRpcClient::new(&proposer_url)?),
            prover_client: Arc::new(network_prover),
            proving_timeout: config.proving_timeout,
        })
    }

    pub async fn request_agg_proof(&mut self, request: Request) -> Result<ProofId, Error> {
        self.rpc.request_agg_proof(request.into()).await?.try_into()
    }

    pub async fn wait_for_proof(
        &mut self,
        proof_id: ProofId,
    ) -> Result<SP1ProofWithPublicValues, Error> {
        let request_id = B256::from_slice(proof_id.0.as_slice());

        self.prover_client
            .wait_proof(request_id, Some(self.proving_timeout))
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
pub struct ProofId([u8; 32]);

impl Display for ProofId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}
