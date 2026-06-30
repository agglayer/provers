use std::{
    sync::Arc,
    task::{Context, Poll},
};

use aggchain_proof_core::full_execution_proof::AggregationProofPublicValues;
use agglayer_evm_client::GetBlockNumber;
use alloy_sol_types::SolType;
use educe::Educe;
pub use error::Error;
use futures::{future::BoxFuture, FutureExt};
use proposer_client::{
    aggregation_prover::AggregationProver,
    mock_grpc_prover::MockGrpcProver,
    network_prover::new_network_prover,
    rpc::{AggregationProofProposerRequest, ProposerRpcClient},
    FepProposerRequest,
};
use sp1_sdk::{NetworkProver, SP1ProofWithPublicValues, SP1VerifyingKey};
use tracing::{debug, info};

use crate::config::ProposerServiceConfig;

#[derive(Debug)]
pub struct ProposerResponse {
    pub aggregation_proof: SP1ProofWithPublicValues,
    pub last_proven_block: u64,
    pub end_block: u64,
    pub public_values: AggregationProofPublicValues,
}

pub mod config;
pub mod error;

#[cfg(test)]
mod tests;

#[derive(Educe)]
#[educe(Clone(bound()))]
pub struct ProposerService<L1Rpc, ProposerClient> {
    pub client: Arc<ProposerClient>,

    pub l1_rpc: Arc<L1Rpc>,

    /// Aggregated span proof verification key.
    aggregation_vkey: SP1VerifyingKey,
}

impl<L1Rpc, Prover>
    ProposerService<L1Rpc, proposer_client::client::Client<ProposerRpcClient, Prover>>
where
    Prover: AggregationProver,
{
    pub async fn new(
        prover: Prover,
        config: &ProposerServiceConfig,
        l1_rpc: Arc<L1Rpc>,
    ) -> Result<Self, Error> {
        let proposer_rpc_client = Arc::new(
            ProposerRpcClient::new(
                config.client.proposer_endpoint.clone(),
                config.client.request_timeout,
            )
            .await?,
        );

        // Use the op-succinct aggregation vkey in effect: the configured override
        // when installed at startup (see `proposer_elfs::install_overrides`),
        // otherwise the value embedded from op-succinct-elfs.
        let aggregation_vkey = proposer_elfs::aggregation::vkey().clone();

        Ok(Self {
            l1_rpc,
            client: Arc::new(proposer_client::client::Client::new(
                proposer_rpc_client,
                prover,
                Some(config.client.proving_timeout),
            )?),
            aggregation_vkey,
        })
    }
}

impl<L1Rpc>
    ProposerService<L1Rpc, proposer_client::client::Client<ProposerRpcClient, NetworkProver>>
{
    pub async fn new_network(
        config: &ProposerServiceConfig,
        l1_rpc: Arc<L1Rpc>,
    ) -> Result<Self, Error> {
        assert!(
            !config.mock,
            "Building a network proposer service with a mock config"
        );
        Self::new(
            new_network_prover(&config.client.sp1_cluster_endpoint)
                .await
                .map_err(Error::Other)?,
            config,
            l1_rpc,
        )
        .await
    }
}

impl<L1Rpc>
    ProposerService<
        L1Rpc,
        proposer_client::client::Client<ProposerRpcClient, MockGrpcProver<ProposerRpcClient>>,
    >
{
    pub async fn new_mock(
        config: &ProposerServiceConfig,
        l1_rpc: Arc<L1Rpc>,
    ) -> Result<Self, Error> {
        assert!(
            config.mock,
            "Building a mock proposer service with a non-mock config"
        );
        let proposer_rpc_client = Arc::new(
            ProposerRpcClient::new(
                config.client.proposer_endpoint.clone(),
                config.client.request_timeout,
            )
            .await?,
        );

        Self::new(
            MockGrpcProver::new(proposer_rpc_client).await,
            config,
            l1_rpc,
        )
        .await
    }
}

impl<L1Rpc, ProposerClient> tower::Service<FepProposerRequest>
    for ProposerService<L1Rpc, ProposerClient>
where
    L1Rpc: GetBlockNumber<Error: Into<eyre::Error>> + Send + Sync + 'static,
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

        async move {
            info!(%last_proven_block, %requested_end_block, "Requesting fep aggregation proof");
            let l1_block_number = l1_rpc
                .get_block_number(l1_block_hash.into())
                .await
                .map_err(|e| {
                    Error::AlloyProviderError(
                        e.into()
                            .wrap_err(format!("Getting the block number for hash {l1_block_hash}")),
                    )
                })?;

            // Request the AggregationProof generation from the proposer.
            let response = client
                .request_agg_proof(AggregationProofProposerRequest {
                    last_proven_block,
                    requested_end_block,
                    l1_block_number,
                    l1_block_hash,
                })
                .await?;
            let request_id = response.request_id;
            let end_block = response.end_block;
            let last_proven_block = response.last_proven_block;
            debug!(%last_proven_block, %end_block, %request_id, "Aggregation proof request submitted");

            // Wait for the prover to finish aggregating span proofs
            let proof_with_pv = client.wait_for_proof(request_id.clone()).await?;

            let public_values =
                AggregationProofPublicValues::abi_decode(proof_with_pv.public_values.as_slice())
                    .map_err(Error::FepPublicValuesDeserializeFailure)?;

            debug!(%last_proven_block, %end_block, %request_id, "Aggregation proof received from the proposer");

            // Verify received proof
            client.verify_agg_proof(request_id.clone(), &proof_with_pv, &aggregation_vkey)?;

            debug!(%last_proven_block, %end_block, %request_id, "Aggregation proof verified successfully");

            info!(%last_proven_block, %end_block, %request_id, "Aggregation proof successfully acquired");

            Ok(ProposerResponse {
                aggregation_proof: proof_with_pv,
                last_proven_block: response.last_proven_block,
                end_block: response.end_block,
                public_values,
            })
        }
        .boxed()
    }
}
