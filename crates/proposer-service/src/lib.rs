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
use proposer_client::{aggregation_prover::AggregationProver, database_prover::DatabaseProver, FepProposerRequest};
use prover_executor::sp1_fast;
use sp1_prover::SP1VerifyingKey;
use tracing::{debug, info};
use aggchain_proof_contracts::contracts::{L1OpSuccinctConfigFetcher, GetTrustedSequencerAddress};

use crate::config::ProposerServiceConfig;
use crate::l2_rpc::{L2ConsensusLayerClient, L2SafeHeadFetcher};

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
pub mod l2_rpc;

#[cfg(test)]
mod tests;

pub const AGGREGATION_ELF: &[u8] = proposer_elfs::aggregation::ELF;

/// Number of L1 blocks to look back when querying for the safe L2 head.
const L1_SAFE_HEAD_LOOKBACK: u64 = 20;

#[derive(Educe)]
#[educe(Clone(bound()))]
pub struct ProposerService<L1Rpc, L2Rpc, ProposerClient, ContractsClient> {
    pub client: Arc<ProposerClient>,

    pub l1_rpc: Arc<L1Rpc>,

    /// Client for fetching L2 safe head information from the rollup node.
    pub l2_rpc: Arc<L2Rpc>,

    /// Database client for persisting proof requests.
    pub db_client: Arc<proposer_db_client::ProposerDBClient>,

    /// Contracts client for fetching L1 contract configuration.
    pub contracts_client: Arc<ContractsClient>,

    /// Aggregated span proof verification key.
    aggregation_vkey: SP1VerifyingKey,

    /// Database polling interval in milliseconds
    poll_interval_ms: u64,

    /// Maximum polling retries before timeout
    max_retries: u32,

    /// L1 chain ID for filtering range proofs
    l1_chain_id: i64,

    /// L2 chain ID for filtering range proofs
    l2_chain_id: i64,

    /// Whether the service is running in mock mode
    mock: bool,
}

impl<L1Rpc, Prover, ContractsClient>
    ProposerService<
        L1Rpc,
        L2ConsensusLayerClient,
        proposer_client::client::Client<Prover>,
        ContractsClient,
    >
where
    Prover: AggregationProver,
    ContractsClient: aggchain_proof_contracts::contracts::ChainIdProvider,
{
    pub async fn new(
        prover: Prover,
        config: &ProposerServiceConfig,
        l1_rpc: Arc<L1Rpc>,
        db_client: Arc<proposer_db_client::ProposerDBClient>,
        contracts_client: Arc<ContractsClient>,
    ) -> Result<Self, Error> {
        let aggregation_vkey = Self::extract_aggregation_vkey(&prover, AGGREGATION_ELF)
            .await
            .context("Retrieving aggregation vkey")
            .map_err(Error::Other)?;

        let l2_rpc = Arc::new(L2ConsensusLayerClient::new(
            &config.l2_consensus_layer_rpc_endpoint,
        )?);

        let db_config = config
            .database
            .as_ref()
            .ok_or_else(|| Error::Other(eyre::eyre!("Database configuration is required")))?;
        let poll_interval_ms = db_config.poll_interval_ms;
        let max_retries = db_config.max_retries;

        // Fetch chain IDs from the contracts client (which obtained them from RPC providers)
        let l1_chain_id = contracts_client.l1_chain_id() as i64;
        let l2_chain_id = contracts_client.l2_chain_id() as i64;

        Ok(Self {
            l1_rpc,
            l2_rpc,
            client: Arc::new(proposer_client::client::Client::new(
                prover,
                Some(config.client.proving_timeout),
            )?),
            db_client,
            contracts_client,
            aggregation_vkey,
            poll_interval_ms,
            max_retries,
            l1_chain_id,
            l2_chain_id,
            mock: config.mock,
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

impl<L1Rpc, ContractsClient>
    ProposerService<
        L1Rpc,
        L2ConsensusLayerClient,
        proposer_client::client::Client<DatabaseProver>,
        ContractsClient,
    >
where
    ContractsClient: aggchain_proof_contracts::contracts::ChainIdProvider,
{
    /// Create a new ProposerService with real SP1 verification.
    ///
    /// Use this for production where proofs are real proofs generated
    /// by the SP1 proving network and stored in the database.
    pub async fn new_real(
        config: &ProposerServiceConfig,
        l1_rpc: Arc<L1Rpc>,
        contracts_client: Arc<ContractsClient>,
    ) -> Result<Self, Error> {
        assert!(
            !config.mock,
            "Building a real proposer service with a mock config"
        );

        let db_config = config
            .database
            .as_ref()
            .ok_or_else(|| Error::Other(eyre::eyre!("Database configuration is required")))?;
        let db_client = Arc::new(
            proposer_db_client::ProposerDBClient::new(&db_config.database_url).await?,
        );

        Self::new(
            DatabaseProver::new(db_client.clone()),
            config,
            l1_rpc,
            db_client,
            contracts_client,
        )
        .await
    }

    /// Create a new ProposerService with mock SP1 verification.
    ///
    /// Use this for testing where proofs are mock proofs generated
    /// by a mock prover and stored in the database.
    pub async fn new_mock(
        config: &ProposerServiceConfig,
        l1_rpc: Arc<L1Rpc>,
        contracts_client: Arc<ContractsClient>,
    ) -> Result<Self, Error> {
        assert!(
            config.mock,
            "Building a mock proposer service with a non-mock config"
        );

        let db_config = config
            .database
            .as_ref()
            .ok_or_else(|| Error::Other(eyre::eyre!("Database configuration is required")))?;
        let db_client = Arc::new(
            proposer_db_client::ProposerDBClient::new(&db_config.database_url).await?,
        );

        Self::new(
            DatabaseProver::new_mock(db_client.clone()),
            config,
            l1_rpc,
            db_client,
            contracts_client,
        )
        .await
    }
}

impl<L1Rpc, L2Rpc, ProposerClient, ContractsClient> tower::Service<FepProposerRequest>
    for ProposerService<L1Rpc, L2Rpc, ProposerClient, ContractsClient>
where
    L1Rpc: GetBlockNumber<Error: Into<eyre::Error>> + Send + Sync + 'static,
    L2Rpc: L2SafeHeadFetcher + Send + Sync + 'static,
    ProposerClient: proposer_client::ProposerClient + Send + Sync + 'static,
    ContractsClient:
        L1OpSuccinctConfigFetcher + GetTrustedSequencerAddress + Send + Sync + 'static,
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
        let l2_rpc = self.l2_rpc.clone();
        let aggregation_vkey = self.aggregation_vkey.clone();
        let db_client = self.db_client.clone();
        let contracts_client = self.contracts_client.clone();
        let poll_interval_ms = self.poll_interval_ms;
        let max_retries = self.max_retries;
        let l1_chain_id = self.l1_chain_id;
        let l2_chain_id = self.l2_chain_id;
        let mock = self.mock;

        async move {
            let l1_block_number = l1_rpc
                .get_block_number(l1_block_hash.into())
                .await
                .map_err(|e| {
                    Error::AlloyProviderError(
                        e.into()
                            .wrap_err(format!("Getting the block number for hash {l1_block_hash}")),
                    )
                })?;

            let l1_limited_end_block =
                limit_end_block_to_safe_head(&l2_rpc, requested_end_block, l1_block_number).await?;

            // Check if the requested end block is less or equal than the requested start block
            if l1_limited_end_block <= last_proven_block {
                return Err(Error::Other(
                    eyre::eyre!("Requested end block is less than or equal to the requested start block")
                ));
            }

            // Fetch op_succinct_config from contracts (needed for rollup_config_hash)
            let op_succinct_config = contracts_client
                .get_op_succinct_config()
                .await
                .map_err(|e| {
                Error::Other(eyre::eyre!("Failed to fetch op_succinct_config from contracts: {}", e))
            })?;

            let trusted_sequencer = contracts_client
                .get_trusted_sequencer_address()
                .await
                .map_err(|e| {
                    Error::Other(eyre::eyre!("Failed to fetch trusted sequencer address from contracts: {}", e))
                })?;

            // Limit according to the existing span proofs range
            let range_proofs = db_client
                .get_consecutive_complete_range_proofs(
                    last_proven_block as i64,
                    l1_limited_end_block as i64,
                    &proposer_elfs::range::VKEY_COMMITMENT,
                    &op_succinct_config.rollup_config_hash.0,
                    l1_chain_id,
                    l2_chain_id,
                )
                .await
                .map_err(|e| {
                    Error::Other(eyre::eyre!("Failed to fetch range proofs: {}", e))
                })?;

            // Limit end block to the last available range proof
            let limited_end_block = if let Some(last_range_proof) = range_proofs.last() {
                let range_limited_end_block = last_range_proof.end_block as u64;
                if range_limited_end_block < l1_limited_end_block {
                    debug!(
                        %l1_limited_end_block,
                        %range_limited_end_block,
                        "Limiting end block to last available range proof"
                    );
                    range_limited_end_block
                } else {
                    l1_limited_end_block
                }
            } else {
                return Err(Error::Other(eyre::eyre!(
                    "No range proofs found for the requested range"
                )));
            };

            info!(%last_proven_block, %limited_end_block, "Requesting fep aggregation proof");

            // Database workflow: insert request and poll for proof_request_id
            use chrono::Utc;
            use proposer_db_client::{OPSuccinctRequest, RequestStatus, RequestType, RequestMode};
            use serde_json::json;
            use sqlx::types::BigDecimal;

            // Insert request with Unrequested status
            let request = OPSuccinctRequest {
                id: 0, // Will be set by database
                status: RequestStatus::Unrequested,
                req_type: RequestType::Aggregation,
                mode: if mock { RequestMode::Mock } else { RequestMode::Real },
                start_block: last_proven_block as i64,
                end_block: limited_end_block as i64,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                proof_request_id: None,
                proof_request_time: None,
                checkpointed_l1_block_number: Some(l1_block_number as i64),
                checkpointed_l1_block_hash: Some(l1_block_hash.0.to_vec()),
                execution_statistics: json!({}),
                witnessgen_duration: None,
                execution_duration: None,
                prove_duration: None,
                range_vkey_commitment: op_succinct_config.range_vkey_commitment.0.to_vec(),
                aggregation_vkey_hash: Some(op_succinct_config.aggregation_vkey_hash.0.to_vec()),
                rollup_config_hash: op_succinct_config.rollup_config_hash.0.to_vec(),
                relay_tx_hash: None,
                proof: None,
                total_nb_transactions: 0,
                total_eth_gas_used: 0,
                total_l1_fees: BigDecimal::from(0),
                total_tx_fees: BigDecimal::from(0),
                l1_chain_id,
                l2_chain_id,
                contract_address: None,
                prover_address: Some(trusted_sequencer.as_slice().to_vec()),
                l1_head_block_number: Some(l1_block_number as i64),
            };

            let db_request_id = db_client.insert_request(&request).await?;
            info!(%db_request_id, %last_proven_block, %l1_limited_end_block, "Inserted aggregation proof request into database");

            // Poll database until proof_request_id is available (status reaches Execution)
            debug!(%db_request_id, "Polling database for proof_request_id");
            let proof_request_id_bytes = db_client
                .wait_for_proof_request_id(db_request_id, poll_interval_ms, max_retries)
                .await?;

            // Convert proof_request_id bytes to the expected type
            let proof_request_id_array: [u8; 32] = proof_request_id_bytes
                .try_into()
                .map_err(|v: Vec<u8>| {
                    Error::Other(eyre::eyre!(
                        "Invalid proof_request_id length: expected 32, got {}",
                        v.len()
                    ))
                })?;
            let request_id = proposer_client::RequestId(proof_request_id_array.into());
            info!(%db_request_id, %request_id, "Got proof_request_id from database, waiting for proof");

            // Wait for the proof (reads from database)
            let proof_with_pv = client.wait_for_proof(request_id.clone()).await?;

            let public_values =
                AggregationProofPublicValues::abi_decode(proof_with_pv.public_values.as_slice())
                    .map_err(Error::FepPublicValuesDeserializeFailure)?;

            debug!(%request_id, "Aggregation proof received");

            // Verify received proof
            client.verify_agg_proof(request_id.clone(), &proof_with_pv, &aggregation_vkey)?;

            debug!(%request_id, "Aggregation proof verified successfully");

            let proof_mode: sp1_sdk::SP1ProofMode =
                sp1_fast(|| (&proof_with_pv.proof).into()).map_err(Error::Other)?;
            let aggregation_proof = sp1_fast(|| proof_with_pv.proof.clone().try_as_compressed())
                .map_err(Error::Other)?
                .ok_or_else(|| Error::UnsupportedAggregationProofMode(proof_mode))?;

            // Get the actual end_block from the proof's public values
            let end_block = public_values.l2_block_number;

            info!(%request_id, %last_proven_block, %end_block, "Aggregation proof successfully acquired");

            Ok(ProposerResponse {
                aggregation_proof,
                last_proven_block,
                end_block,
                public_values,
            })
        }
        .boxed()
    }
}

/// Limits the requested end block to the safe L2 head derived from L1 data.
///
/// This ensures we don't request proofs for L2 blocks that haven't been safely
/// derived from L1 yet. We look back `L1_SAFE_HEAD_LOOKBACK` blocks from the
/// current L1 block to account for reorg safety.
async fn limit_end_block_to_safe_head<L2Rpc: L2SafeHeadFetcher>(
    l2_rpc: &L2Rpc,
    requested_end_block: u64,
    l1_block_number: u64,
) -> Result<u64, Error> {
    let safe_l1_block = l1_block_number.saturating_sub(L1_SAFE_HEAD_LOOKBACK);
    let safe_head_response = l2_rpc.get_safe_head_at_l1_block(safe_l1_block).await?;
    let safe_head: u64 = safe_head_response.safe_head.number.to();

    if safe_head < requested_end_block {
        debug!(
            %requested_end_block,
            %safe_head,
            %l1_block_number,
            "Limiting requested end block to safe head"
        );
        return Ok(safe_head);
    }

    Ok(requested_end_block)
}
