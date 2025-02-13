use std::sync::Arc;
use std::task::{Context, Poll};

use futures::{future::BoxFuture, FutureExt};
use proposer_client::network_prover::new_network_prover;
use proposer_client::rpc::{AggSpanProofProposerRequest, ProposerRpcClient};
use proposer_client::{ProofId, ProposerClient};
pub use proposer_client::{ProposerRequest, ProposerResponse};
use prover_alloy::providers::Provider;
use prover_alloy::rpc::types::BlockTransactionsKind;
use prover_alloy::AlloyProvider;
use sp1_sdk::NetworkProver;

pub mod error;

pub mod config;

pub use error::Error;

use crate::config::ProposerServiceConfig;

#[derive(Clone)]
pub struct ProposerService {
    pub client: Arc<ProposerClient<ProposerRpcClient, NetworkProver>>,
    pub l1_rpc: Arc<AlloyProvider>,
}

impl ProposerService {
    pub fn new(config: &ProposerServiceConfig, l1_rpc: Arc<AlloyProvider>) -> Result<Self, Error> {
        let proposer_rpc_client = ProposerRpcClient::new(config.client.proposer_endpoint.as_str())?;
        let network_prover = new_network_prover(config.client.sp1_cluster_endpoint.as_str());
        Ok(Self {
            l1_rpc,
            client: Arc::new(ProposerClient::new(
                proposer_rpc_client,
                network_prover,
                Some(config.client.proving_timeout),
            )?),
        })
    }
}

impl tower::Service<ProposerRequest> for ProposerService {
    type Response = ProposerResponse;

    type Error = Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(
        &mut self,
        ProposerRequest {
            start_block,
            max_block,
            l1_block_number,
        }: ProposerRequest,
    ) -> Self::Future {
        let client = self.client.clone();
        let l1_rpc = self.l1_rpc.clone();

        async move {
            let l1_block_hash = l1_rpc
                .get_block_by_number(l1_block_number.into(), BlockTransactionsKind::Hashes)
                .await
                .map_err(|error| {
                    Error::AlloyProviderError(anyhow::anyhow!(
                        "Failed to get L1 block hash: {:?}",
                        error
                    ))
                })?
                .ok_or(Error::AlloyProviderError(anyhow::anyhow!(
                    "target block {l1_block_number} does not exist"
                )))?
                .header
                .hash;

            // Request the AggSpanProof generation from the proposer
            let response = client
                .request_agg_proof(AggSpanProofProposerRequest {
                    start: start_block,
                    end: max_block,
                    l1_block_number,
                    l1_block_hash,
                })
                .await?;
            let proof_id = ProofId(response.proof_id);

            // Wait for the prover to finish aggregating span proofs
            let proofs = client.wait_for_proof(proof_id).await?;

            Ok(ProposerResponse {
                agg_span_proof: proofs,
                start_block: response.start_block,
                end_block: response.end_block,
            })
        }
        .boxed()
    }
}
