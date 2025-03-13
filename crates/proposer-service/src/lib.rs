use std::sync::Arc;
use std::task::{Context, Poll};

use aggkit_prover_types::vkey_hash::VKeyHash;
pub use error::Error;
use futures::{future::BoxFuture, FutureExt};
use proposer_client::network_prover::new_network_prover;
use proposer_client::rpc::{AggSpanProofProposerRequest, ProposerRpcClient};
use proposer_client::ProofId;
pub use proposer_client::{ProposerRequest, ProposerResponse};
use prover_alloy::Provider;
use sp1_sdk::{NetworkProver, SP1ProofWithPublicValues};

use crate::config::ProposerServiceConfig;

pub mod config;
pub mod error;
#[cfg(test)]
mod tests;

const AGGREGATION_VKEY_HASH: VKeyHash =
    VKeyHash::from_hash_u32(aggchain_proof_core::AGGREGATION_VKEY_HASH);

pub struct ProposerService<L1Rpc, ProposerClient> {
    pub client: Arc<ProposerClient>,

    pub l1_rpc: Arc<L1Rpc>,

    /// Expected aggregated span proof verification key.
    aggregation_vkey_hash: VKeyHash,
}

impl<L1Rpc, ProposerClient> Clone for ProposerService<L1Rpc, ProposerClient> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            l1_rpc: self.l1_rpc.clone(),
            aggregation_vkey_hash: self.aggregation_vkey_hash,
        }
    }
}

impl<L1Rpc> ProposerService<L1Rpc, proposer_client::Client<ProposerRpcClient, NetworkProver>> {
    pub fn new(config: &ProposerServiceConfig, l1_rpc: Arc<L1Rpc>) -> Result<Self, Error> {
        let proposer_rpc_client = ProposerRpcClient::new(config.client.proposer_endpoint.as_str())?;
        let network_prover = new_network_prover(config.client.sp1_cluster_endpoint.as_str())
            .map_err(Error::UnableToCreateNetworkProver)?;

        Ok(Self {
            l1_rpc,
            client: Arc::new(proposer_client::Client::new(
                proposer_rpc_client,
                network_prover,
                Some(config.client.proving_timeout),
            )?),
            aggregation_vkey_hash: AGGREGATION_VKEY_HASH,
        })
    }
}

fn check_aggregation_vkey(
    sp1_proof: &SP1ProofWithPublicValues,
    expected_vkey_hash: VKeyHash,
) -> Result<(), Error> {
    let sp1_proof = &sp1_proof.proof;
    let proof = &**sp1_proof
        .try_as_compressed_ref()
        .ok_or_else(|| Error::UnsupportedAggregationProofMode(sp1_proof.into()))?;

    let vkey_hash = VKeyHash::from_vkey(&proof.vk);
    if vkey_hash != expected_vkey_hash {
        return Err(Error::AggregationVKeyMismatch {
            got: vkey_hash,
            expected: expected_vkey_hash,
        });
    }

    Ok(())
}

impl<L1Rpc, ProposerClient> tower::Service<ProposerRequest>
    for ProposerService<L1Rpc, ProposerClient>
where
    L1Rpc: Provider + Send + Sync + 'static,
    ProposerClient: proposer_client::ProposerClient + Send + Sync + 'static,
{
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
        let expected_vkey_hash = self.aggregation_vkey_hash;

        async move {
            let l1_block_hash = l1_rpc
                .get_block_hash(l1_block_number)
                .await
                .map_err(Error::AlloyProviderError)?;

            // Request the AggSpanProof generation from the proposer.
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
            check_aggregation_vkey(&proofs, expected_vkey_hash)?;

            Ok(ProposerResponse {
                aggregation_proof: proofs,
                start_block: response.start_block,
                end_block: response.end_block,
            })
        }
        .boxed()
    }
}
