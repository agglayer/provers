use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sp1_sdk::network::proto::network::FulfillmentStatus;
use sp1_sdk::network::B256;
use sp1_sdk::{NetworkProver, ProverClient, SP1ProofWithPublicValues};

use crate::config::ProposerClientConfig;
use crate::error::Error;
use crate::rpc::{ProposerProofStatus, ProposerRpcClient};

const CLUSTER_POLL_DURATION_STEP: Duration = Duration::from_secs(10);

mod config;
mod error;
mod rpc;

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

    pub async fn request_span_proof(&mut self, request: Request) -> Result<ProofId, Error> {
        self.rpc
            .request_span_proof(request.into())
            .await
            .map(Into::into)
    }

    pub async fn check_status(&mut self, proof_id: &ProofId) -> Result<ProposerProofStatus, Error> {
        self.rpc.check_status(proof_id).await
    }

    pub async fn wait_for_proof(
        &mut self,
        proof_id: ProofId,
    ) -> Result<Option<SP1ProofWithPublicValues>, Error> {
        if proof_id.0.len() != 32 {
            return Err(Error::InvalidProofId(proof_id));
        }
        let request_id = B256::from_slice(proof_id.0.as_slice());
        let mut remaining_timeout = self.proving_timeout;

        while remaining_timeout > Duration::ZERO {
            let (proof, status) = self
                .prover_client
                .process_proof_status(request_id, Some(remaining_timeout))
                .await
                .map_err(|e| error::Error::Proving(proof_id.clone(), format!("{e:?}")))?;
            match status {
                FulfillmentStatus::Fulfilled => {
                    return Ok(proof);
                }
                FulfillmentStatus::Unfulfillable => {
                    return Err(Error::ProofRequestUnfullfilable(proof_id));
                }
                _ => {
                    tokio::time::sleep(CLUSTER_POLL_DURATION_STEP).await;
                    remaining_timeout = remaining_timeout
                        .checked_sub(CLUSTER_POLL_DURATION_STEP)
                        .unwrap_or_default();
                }
            }
        }
        Err(Error::Timeout(proof_id))
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
