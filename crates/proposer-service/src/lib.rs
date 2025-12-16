use std::{
    sync::Arc,
    task::{Context, Poll},
};

use aggchain_proof_core::full_execution_proof::AggregationProofPublicValues;
use agglayer_evm_client::GetBlockNumber;
use alloy_sol_types::SolType;
use educe::Educe;
pub use error::Error;
use eyre::Context as _;
use futures::{future::BoxFuture, FutureExt};
use proposer_client::{
    aggregation_prover::AggregationProver,
    mock_grpc_prover::MockGrpcProver,
    network_prover::new_network_prover,
    rpc::{AggregationProofProposerRequest, ProposerRpcClient},
    FepProposerRequest,
};
use prover_executor::sp1_fast;
use sp1_prover::SP1VerifyingKey;
use sp1_sdk::NetworkProver;
use tracing::{debug, info};

use crate::config::ProposerServiceConfig;

type AggregationProof = Box<sp1_core_executor::SP1ReduceProof<sp1_prover::InnerSC>>;

#[derive(Debug)]
pub struct ProposerResponse {
    pub aggregation_proof: AggregationProof,
    pub last_proven_block: u64,
    pub end_block: u64,
    pub public_values: AggregationProofPublicValues,
}

pub mod config;
pub mod error;

#[cfg(test)]
mod tests;

pub const AGGREGATION_ELF: &[u8] = proposer_elfs::aggregation::ELF;

#[derive(Educe)]
#[educe(Clone(bound()))]
pub struct ProposerService<L1Rpc, ProposerClient> {
    pub client: Arc<ProposerClient>,

    pub l1_rpc: Arc<L1Rpc>,

    /// Optional database client for persisting proof requests.
    pub db_client: Option<Arc<proposer_db_client::ProposerDBClient>>,

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

        let aggregation_vkey = Self::extract_aggregation_vkey(&prover, AGGREGATION_ELF)
            .await
            .context("Retrieving aggregation vkey")
            .map_err(Error::Other)?;

        let db_client = if let Some(db_config) = &config.database {
            let client = proposer_db_client::ProposerDBClient::new(db_config.database_url.as_str())
                .await?;
            Some(Arc::new(client))
        } else {
            None
        };

        Ok(Self {
            l1_rpc,
            client: Arc::new(proposer_client::client::Client::new(
                proposer_rpc_client,
                prover,
                Some(config.client.proving_timeout),
            )?),
            db_client,
            aggregation_vkey,
        })
    }

    async fn extract_aggregation_vkey(
        prover: &Prover,
        elf: &[u8],
    ) -> eyre::Result<SP1VerifyingKey> {
        let (_pkey, vkey) = prover.compute_pkey_vkey(elf).await?;
        Ok(vkey)
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
            new_network_prover(&config.client.sp1_cluster_endpoint).map_err(Error::Other)?,
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

        Self::new(MockGrpcProver::new(proposer_rpc_client), config, l1_rpc).await
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
        let db_client = self.db_client.clone();

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

            let proof_mode: sp1_sdk::SP1ProofMode = sp1_fast(|| (&proof_with_pv.proof).into()).map_err(Error::Other)?;
            let aggregation_proof = sp1_fast(|| proof_with_pv.proof.clone().try_as_compressed())
                .map_err(Error::Other)?
                .ok_or_else(|| Error::UnsupportedAggregationProofMode(proof_mode))?;

            info!(%last_proven_block, %end_block, %request_id, "Aggregation proof successfully acquired");

            if let Some(db) = db_client {
                use chrono::Utc;
                use proposer_db_client::{OPSuccinctRequest, RequestStatus, RequestType, RequestMode};
                use serde_json::json;
                use sqlx::types::BigDecimal;

                let request = OPSuccinctRequest {
                    id: 0,
                    status: RequestStatus::Complete,
                    req_type: RequestType::Aggregation,
                    mode: RequestMode::Real,
                    start_block: last_proven_block as i64,
                    end_block: end_block as i64,
                    created_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                    proof_request_id: Some(request_id.0.to_vec()),
                    proof_request_time: Some(Utc::now().naive_utc()),
                    checkpointed_l1_block_number: Some(l1_block_number as i64),
                    checkpointed_l1_block_hash: Some(l1_block_hash.0.to_vec()),
                    execution_statistics: json!({}),
                    witnessgen_duration: None,
                    execution_duration: None,
                    prove_duration: None,
                    range_vkey_commitment: vec![],
                    aggregation_vkey_hash: None,
                    rollup_config_hash: vec![],
                    relay_tx_hash: None,
                    proof: Some(bincode::serialize(&aggregation_proof).map_err(|e| Error::Other(e.into()))?),
                    total_nb_transactions: 0,
                    total_eth_gas_used: 0,
                    total_l1_fees: BigDecimal::from(0),
                    total_tx_fees: BigDecimal::from(0),
                    l1_chain_id: 0,
                    l2_chain_id: 0,
                    contract_address: None,
                    prover_address: None,
                    l1_head_block_number: Some(l1_block_number as i64),
                };

                db.insert_request(&request).await?;
                debug!(%request_id, "Inserted aggregation proof request into database");
            }

            Ok(ProposerResponse {
                aggregation_proof,
                last_proven_block: response.last_proven_block,
                end_block: response.end_block,
                public_values,
            })
        }
        .boxed()
    }
}
