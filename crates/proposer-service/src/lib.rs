use std::sync::Arc;
use std::task::{Context, Poll};

use aggchain_proof_core::full_execution_proof::AggregationOutputs;
use bincode::config::Options;
pub use error::Error;
use futures::{future::BoxFuture, FutureExt};
use proposer_client::network_prover::new_network_prover;
use proposer_client::rpc::{AggregationProofProposerRequest, ProposerRpcClient};
use proposer_client::FepProposerRequest;
use proposer_client::RequestId;
use prover_alloy::Provider;
use sp1_prover::SP1VerifyingKey;
use sp1_sdk::{NetworkProver, Prover};
use tracing::info;

use crate::config::ProposerServiceConfig;

type AggregationProof = Box<sp1_core_executor::SP1ReduceProof<sp1_prover::InnerSC>>;

#[derive(Debug)]
pub struct ProposerResponse {
    pub aggregation_proof: AggregationProof,
    pub last_proven_block: u64,
    pub end_block: u64,
}

pub mod config;
pub mod error;

#[cfg(test)]
mod tests;

pub const AGGREGATION_ELF: &[u8] =
    include_bytes!("../../aggchain-proof-program/elf/aggregation-elf");

pub struct ProposerService<L1Rpc, ProposerClient> {
    pub client: Arc<ProposerClient>,

    pub l1_rpc: Arc<L1Rpc>,

    /// Aggregated span proof verification key.
    aggregation_vkey: SP1VerifyingKey,
}

impl<L1Rpc, ProposerClient> Clone for ProposerService<L1Rpc, ProposerClient> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            l1_rpc: self.l1_rpc.clone(),
            aggregation_vkey: self.aggregation_vkey.clone(),
        }
    }
}

impl<L1Rpc>
    ProposerService<L1Rpc, proposer_client::client::Client<ProposerRpcClient, NetworkProver>>
{
    pub fn new(config: &ProposerServiceConfig, l1_rpc: Arc<L1Rpc>) -> Result<Self, Error> {
        let proposer_rpc_client = ProposerRpcClient::new(
            config.client.proposer_endpoint.as_str(),
            config.client.request_timeout,
        )?;
        let network_prover = new_network_prover(config.client.sp1_cluster_endpoint.as_str())
            .map_err(Error::UnableToCreateNetworkProver)?;

        let aggregation_vkey = Self::extract_aggregation_vkey(AGGREGATION_ELF);

        Ok(Self {
            l1_rpc,
            client: Arc::new(proposer_client::client::Client::new(
                proposer_rpc_client,
                network_prover,
                Some(config.client.proving_timeout),
            )?),
            aggregation_vkey,
        })
    }

    fn extract_aggregation_vkey(elf: &[u8]) -> SP1VerifyingKey {
        let client = sp1_sdk::ProverClient::builder().network().build();
        let (_pkey, vkey) = client.setup(elf);
        vkey
    }
}

impl<L1Rpc, ProposerClient> tower::Service<FepProposerRequest>
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
        FepProposerRequest {
            last_proven_block,
            requested_end_block,
            l1_block_hash,
        }: FepProposerRequest,
    ) -> Self::Future {
        let client = self.client.clone();
        let l1_rpc = self.l1_rpc.clone();
        let aggregation_vkey = self.aggregation_vkey.clone();

        println!(
            ">>>>>>>>>> Proposer service request received, start_block: {last_proven_block:} \
             max_block: {requested_end_block:} l1_block_hash: {l1_block_hash:}"
        );

        async move {
            println!(">>>>>>>>>> Checkpoint 11");
            let l1_block_number = l1_rpc
                .get_block_number(l1_block_hash)
                .await
                .map_err(Error::AlloyProviderError)?;

            println!(
                ">>>>>>>>>> Checkpoint 12 op succinct proposer request: {:#?}",
                AggregationProofProposerRequest {
                    last_proven_block,
                    requested_end_block,
                    l1_block_number,
                    l1_block_hash,
                }
            );

            // Request the AggregationProof generation from the proposer.
            let response = client
                .request_agg_proof(AggregationProofProposerRequest {
                    last_proven_block,
                    requested_end_block,
                    l1_block_number,
                    l1_block_hash,
                })
                .await
                .inspect_err(|e| println!(">>>>>> ERROR: {e:#?}"))?;
            let request_id = RequestId(response.request_id);
            info!("Aggregation proof request submitted: {}", request_id);

            println!(">>>>>>>>>> Checkpoint 13, request_id: {request_id:}");

            // Wait for the prover to finish aggregating span proofs
            let proofs = client.wait_for_proof(request_id.clone()).await?;

            let deser_pv = AggregationOutputs::bincode_options()
                .deserialize(proofs.public_values.as_slice())?;

            println!("fep public values: {:?}", deser_pv);
            println!(">>>>>>>>>> Checkpoint 14");

            // Verify received proof
            client.verify_agg_proof(request_id, &proofs, &aggregation_vkey)?;

            let proof_mode: sp1_sdk::SP1ProofMode = (&proofs.proof).into();
            let aggregation_proof: AggregationProof = proofs
                .proof
                .clone()
                .try_as_compressed()
                .ok_or_else(|| Error::UnsupportedAggregationProofMode(proof_mode))?;

            println!(">>>>>>>>>> Checkpoint 15");

            Ok(ProposerResponse {
                aggregation_proof,
                last_proven_block: response.last_proven_block,
                end_block: response.end_block,
            })
        }
        .boxed()
    }
}
