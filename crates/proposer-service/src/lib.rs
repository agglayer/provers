use std::{
    sync::Arc,
    task::{Context, Poll},
};

use aggchain_proof_contracts::contracts::{GetTrustedSequencerAddress, L1OpSuccinctConfigFetcher};
use aggchain_proof_core::full_execution_proof::AggregationProofPublicValues;
use agglayer_evm_client::GetBlockNumber;
use alloy_sol_types::SolType;
use educe::Educe;
pub use error::Error;
use eyre::Context as _;
use futures::{future::BoxFuture, FutureExt};
use proposer_client::{
    mock_grpc_prover::MockGrpcProver,
    network_prover::new_network_prover,
    rpc::{AggregationProofProposerRequest, ProposerRpcClient},
    FepProposerRequest,
};
use prover_executor::sp1_fast;
use sp1_prover::SP1VerifyingKey;
use sp1_sdk::{CpuProver, NetworkProver, Prover as _};
use tracing::{debug, error, info};

use crate::{
    config::ProposerServiceConfig,
    l2_rpc::{L2ConsensusLayerClient, L2SafeHeadFetcher},
};

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

/// Backend for proof generation - either gRPC client or database.
#[derive(Educe)]
#[educe(Clone(bound()))]
pub enum ProofBackend<ProposerClient> {
    /// Use gRPC client to request proofs from the proposer service.
    Grpc {
        client: Arc<ProposerClient>,
        poll_interval_ms: u64,
        max_retries: u32,
    },
    /// Use database to insert proof requests and poll for completion.
    Database {
        db_client: Arc<proposer_db_client::ProposerDBClient>,
        poll_interval_ms: u64,
        max_retries: u32,
    },
}

#[derive(Educe)]
#[educe(Clone(bound()))]
pub struct ProposerService<L1Rpc, L2Rpc, ProposerClient, ContractsClient> {
    /// Backend for proof generation (gRPC or database).
    pub backend: ProofBackend<ProposerClient>,

    pub l1_rpc: Arc<L1Rpc>,

    /// Client for fetching L2 safe head information from the rollup node.
    pub l2_rpc: Arc<L2Rpc>,

    /// Contracts client for fetching L1 contract configuration.
    pub contracts_client: Arc<ContractsClient>,

    /// Aggregated span proof verification key.
    aggregation_vkey: SP1VerifyingKey,

    /// L1 chain ID for filtering range proofs
    l1_chain_id: i64,

    /// L2 chain ID for filtering range proofs
    l2_chain_id: i64,

    /// Whether the service is running in mock mode
    mock: bool,
}

impl<L1Rpc, L2Rpc, ProposerClient, ContractsClient>
    ProposerService<L1Rpc, L2Rpc, ProposerClient, ContractsClient>
{
    /// Creates a new ProposerService with the specified backend.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        backend: ProofBackend<ProposerClient>,
        l1_rpc: Arc<L1Rpc>,
        l2_rpc: Arc<L2Rpc>,
        contracts_client: Arc<ContractsClient>,
        aggregation_vkey: SP1VerifyingKey,
        l1_chain_id: i64,
        l2_chain_id: i64,
        mock: bool,
    ) -> Self {
        Self {
            backend,
            l1_rpc,
            l2_rpc,
            contracts_client,
            aggregation_vkey,
            l1_chain_id,
            l2_chain_id,
            mock,
        }
    }
}

impl<L1Rpc, ContractsClient>
    ProposerService<
        L1Rpc,
        L2ConsensusLayerClient,
        proposer_client::client::Client<ProposerRpcClient, NetworkProver>,
        ContractsClient,
    >
where
    ContractsClient: aggchain_proof_contracts::contracts::ChainIdProvider,
{
    pub async fn new_network(
        config: &ProposerServiceConfig,
        l1_rpc: Arc<L1Rpc>,
        contracts_client: Arc<ContractsClient>,
    ) -> Result<Self, Error> {
        assert!(
            !config.mock,
            "Building a network proposer service with a mock config"
        );

        let l2_rpc = Arc::new(L2ConsensusLayerClient::new(
            &config.l2_consensus_layer_rpc_endpoint,
        )?);

        let l1_chain_id = contracts_client.l1_chain_id() as i64;
        let l2_chain_id = contracts_client.l2_chain_id() as i64;

        let prover =
            new_network_prover(&config.client.sp1_cluster_endpoint).map_err(Error::Other)?;

        let aggregation_vkey = Self::extract_aggregation_vkey(&prover, AGGREGATION_ELF)
            .await
            .context("Retrieving aggregation vkey")
            .map_err(Error::Other)?;

        let backend = if let Some(db_config) = &config.database {
            let db_client =
                proposer_db_client::ProposerDBClient::new(&db_config.database_url).await?;

            ProofBackend::Database {
                db_client: Arc::new(db_client),
                poll_interval_ms: db_config.poll_interval_ms,
                max_retries: db_config.max_retries,
            }
        } else {
            let proposer_rpc_client = Arc::new(
                ProposerRpcClient::new(
                    config.client.proposer_endpoint.clone(),
                    config.client.request_timeout,
                )
                .await?,
            );

            let client = Arc::new(proposer_client::client::Client::new(
                proposer_rpc_client,
                prover,
                Some(config.client.proving_timeout),
            )?);

            ProofBackend::Grpc {
                client,
                poll_interval_ms: 5000,
                max_retries: 720,
            }
        };

        Ok(Self::new(
            backend,
            l1_rpc,
            l2_rpc,
            contracts_client,
            aggregation_vkey,
            l1_chain_id,
            l2_chain_id,
            config.mock,
        ))
    }

    async fn extract_aggregation_vkey(
        prover: &NetworkProver,
        elf: &[u8],
    ) -> eyre::Result<SP1VerifyingKey> {
        let (_pkey, vkey) = prover.setup(elf);
        Ok(vkey)
    }
}

impl<L1Rpc, ContractsClient>
    ProposerService<
        L1Rpc,
        L2ConsensusLayerClient,
        proposer_client::client::Client<ProposerRpcClient, MockGrpcProver<ProposerRpcClient>>,
        ContractsClient,
    >
where
    ContractsClient: aggchain_proof_contracts::contracts::ChainIdProvider,
{
    pub async fn new_mock(
        config: &ProposerServiceConfig,
        l1_rpc: Arc<L1Rpc>,
        contracts_client: Arc<ContractsClient>,
    ) -> Result<Self, Error> {
        assert!(
            config.mock,
            "Building a mock proposer service with a non-mock config"
        );

        let l2_rpc = Arc::new(L2ConsensusLayerClient::new(
            &config.l2_consensus_layer_rpc_endpoint,
        )?);

        let l1_chain_id = contracts_client.l1_chain_id() as i64;
        let l2_chain_id = contracts_client.l2_chain_id() as i64;

        let prover = sp1_sdk::ProverClient::builder().mock().build();

        let aggregation_vkey = Self::extract_aggregation_vkey(&prover, AGGREGATION_ELF)
            .await
            .context("Retrieving aggregation vkey")
            .map_err(Error::Other)?;

        let backend = if let Some(db_config) = &config.database {
            let db_client =
                proposer_db_client::ProposerDBClient::new(&db_config.database_url).await?;

            ProofBackend::Database {
                db_client: Arc::new(db_client),
                poll_interval_ms: db_config.poll_interval_ms,
                max_retries: db_config.max_retries,
            }
        } else {
            let proposer_rpc_client = Arc::new(
                ProposerRpcClient::new(
                    config.client.proposer_endpoint.clone(),
                    config.client.request_timeout,
                )
                .await?,
            );

            let mock_grpc_prover = MockGrpcProver::new(proposer_rpc_client.clone());

            let client = Arc::new(proposer_client::client::Client::new(
                proposer_rpc_client,
                mock_grpc_prover,
                Some(config.client.proving_timeout),
            )?);

            ProofBackend::Grpc {
                client,
                poll_interval_ms: 5000,
                max_retries: 720,
            }
        };

        Ok(Self::new(
            backend,
            l1_rpc,
            l2_rpc,
            contracts_client,
            aggregation_vkey,
            l1_chain_id,
            l2_chain_id,
            config.mock,
        ))
    }

    async fn extract_aggregation_vkey(
        prover: &CpuProver,
        elf: &[u8],
    ) -> eyre::Result<SP1VerifyingKey> {
        let (_pkey, vkey) = prover.setup(elf);
        Ok(vkey)
    }
}

impl<L1Rpc, L2Rpc, ProposerClient, ContractsClient> tower::Service<FepProposerRequest>
    for ProposerService<L1Rpc, L2Rpc, ProposerClient, ContractsClient>
where
    L1Rpc: GetBlockNumber<Error: Into<eyre::Error>> + Send + Sync + 'static,
    L2Rpc: L2SafeHeadFetcher + Send + Sync + 'static,
    ProposerClient: proposer_client::ProposerClient + Send + Sync + 'static,
    ContractsClient: L1OpSuccinctConfigFetcher + GetTrustedSequencerAddress + Send + Sync + 'static,
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
        let backend = self.backend.clone();
        let l1_rpc = self.l1_rpc.clone();
        let l2_rpc = self.l2_rpc.clone();
        let aggregation_vkey = self.aggregation_vkey.clone();
        let contracts_client = self.contracts_client.clone();
        let l1_chain_id = self.l1_chain_id;
        let l2_chain_id = self.l2_chain_id;
        let mock = self.mock;

        async move {
            debug!("Starting FEP proposer request");

            let l1_block_number = l1_rpc
                .get_block_number(l1_block_hash.into())
                .await
                .map_err(|e| {
                    let err = Error::AlloyProviderError(
                        e.into()
                            .wrap_err(format!("Getting the block number for hash {l1_block_hash}")),
                    );
                    error!(%l1_block_hash, "Failed to get L1 block number: {}", err);
                    err
                })?;

            let l1_limited_end_block =
                limit_end_block_to_safe_head(&l2_rpc, requested_end_block, l1_block_number).await?;

            // Check if the requested end block is less or equal than the requested start block
            if l1_limited_end_block <= last_proven_block {
                error!(
                    %l1_limited_end_block,
                    %last_proven_block,
                    "Requested end block is less than or equal to the last proven block"
                );
                return Err(Error::Other(
                    eyre::eyre!("Requested end block is less than or equal to the requested start block")
                ));
            }

            // Fetch op_succinct_config from contracts (needed for rollup_config_hash)
            let op_succinct_config = contracts_client
                .get_op_succinct_config()
                .await
                .map_err(|e| {
                    let err = Error::Other(eyre::eyre!("Failed to fetch op_succinct_config from contracts: {}", e));
                    error!("Failed to fetch op_succinct_config from contracts: {}", err);
                    err
                })?;

            let trusted_sequencer = contracts_client
                .get_trusted_sequencer_address()
                .await
                .map_err(|e| {
                    let err = Error::Other(eyre::eyre!("Failed to fetch trusted sequencer address from contracts: {}", e));
                    error!("Failed to fetch trusted sequencer address from contracts: {}", err);
                    err
                })?;

            // Limit according to the existing span proofs range (only when database backend is configured)
            let limited_end_block = if let ProofBackend::Database { db_client, .. } = backend.clone() {
                info!("Fetching range proofs from database: start_block={}, end_block={}, l1_chain_id={}, l2_chain_id={}",
                      last_proven_block, l1_limited_end_block, l1_chain_id, l2_chain_id);

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
                        let err = Error::Other(eyre::eyre!("Failed to fetch range proofs: {}", e));
                        error!(
                            %last_proven_block,
                            %l1_limited_end_block,
                            %l1_chain_id,
                            %l2_chain_id,
                            "Failed to fetch range proofs from database: {}", err
                        );
                        err
                    })?;

                // Limit end block to the last available range proof
                if let Some(last_range_proof) = range_proofs.last() {
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
                    error!(
                        %last_proven_block,
                        %l1_limited_end_block,
                        %l1_chain_id,
                        %l2_chain_id,
                        "No range proofs found for the requested range"
                    );
                    return Err(Error::Other(eyre::eyre!(
                        "No range proofs found for the requested range"
                    )));
                }
            } else {
                l1_limited_end_block
            };

            info!(%last_proven_block, %limited_end_block, "Requesting fep aggregation proof");

            // Get proof based on backend type
            let proof_with_pv: sp1_sdk::SP1ProofWithPublicValues = match backend {
                ProofBackend::Database { db_client, poll_interval_ms, max_retries } => {
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

                    let db_request_id = db_client.insert_request(&request).await.map_err(|e| {
                        error!(%last_proven_block, %limited_end_block, "Failed to insert aggregation proof request into database: {}", e);
                        e
                    })?;
                    info!(%db_request_id, %last_proven_block, %limited_end_block, "Inserted aggregation proof request into database");

                    // Poll database until proof is complete
                    debug!(%db_request_id, "Polling database for proof completion");
                    let proof_bytes = db_client
                        .wait_for_proof_completion(db_request_id, poll_interval_ms, max_retries)
                        .await
                        .map_err(|e| {
                            error!(%db_request_id, "Failed to wait for proof completion: {}", e);
                            e
                        })?;

                    // Deserialize proof using bincode
                    let proof_with_pv: sp1_sdk::SP1ProofWithPublicValues =
                        sp1_fast(|| agglayer_interop_types::bincode::default().deserialize(&proof_bytes))
                            .map_err(|e| {
                                error!(%db_request_id, "Failed during proof deserialization (panic): {}", e);
                                Error::Other(e)
                            })?
                            .map_err(|e| {
                                error!(%db_request_id, "Failed to deserialize proof from database: {}", e);
                                Error::Other(eyre::eyre!("Failed to deserialize proof from database: {}", e))
                            })?;

                    info!(%db_request_id, "Proof retrieved and deserialized from database");

                    // Verify the proof
                    use std::panic::AssertUnwindSafe;
                    let prover = sp1_sdk::ProverClient::builder().mock().build();
                    sp1_fast(AssertUnwindSafe(|| prover.verify(&proof_with_pv, &aggregation_vkey)))
                        .map_err(Error::Other)?
                        .map_err(|e| {
                            let err = Error::Other(eyre::eyre!("Failed to verify proof from database: {}", e));
                            error!(%db_request_id, "Failed to verify proof from database: {}", err);
                            err
                        })?;

                    debug!(%db_request_id, "Aggregation proof verified successfully");
                    proof_with_pv
                }
                ProofBackend::Grpc { client, .. } => {
                    // gRPC workflow: request proof generation from proposer
                    info!("Using gRPC workflow");

                    let response = client
                        .request_agg_proof(AggregationProofProposerRequest {
                            last_proven_block,
                            requested_end_block,
                            l1_block_number,
                            l1_block_hash,
                        })
                        .await
                        .map_err(|e| {
                            error!(%last_proven_block, %requested_end_block, "Failed to request aggregation proof via gRPC: {}", e);
                            e
                        })?;
                    debug!(%last_proven_block, end_block = %response.end_block, request_id = %response.request_id, "Aggregation proof request response received");

                    // Wait for the prover to finish aggregating span proofs
                    let proof_with_pv = client.wait_for_proof(response.request_id.clone()).await.map_err(|e| {
                        error!(request_id = %response.request_id, "Failed to wait for proof via gRPC: {}", e);
                        e
                    })?;
                    debug!(%response.request_id, "Aggregation proof received from the proposer");

                    // Verify received proof
                    client.verify_agg_proof(response.request_id.clone(), &proof_with_pv, &aggregation_vkey).map_err(|e| {
                        error!(request_id = %response.request_id, "Failed to verify aggregation proof via gRPC: {}", e);
                        e
                    })?;
                    debug!(%response.request_id, "Aggregation proof verified successfully");

                    proof_with_pv
                }
            };

            let public_values =
                AggregationProofPublicValues::abi_decode(proof_with_pv.public_values.as_slice())
                    .map_err(|e| {
                        error!("Failed to decode aggregation proof public values: {}", e);
                        Error::FepPublicValuesDeserializeFailure(e)
                    })?;

            debug!("Aggregation proof public values decoded successfully");

            let proof_mode: sp1_sdk::SP1ProofMode =
                sp1_fast(|| (&proof_with_pv.proof).into()).map_err(|e| {
                    error!("Failed to get proof mode: {}", e);
                    Error::Other(e)
                })?;
            let aggregation_proof = sp1_fast(|| proof_with_pv.proof.clone().try_as_compressed())
                .map_err(|e| {
                    error!("Failed to convert proof to compressed format: {}", e);
                    Error::Other(e)
                })?
                .ok_or_else(|| {
                    error!(?proof_mode, "Unsupported aggregation proof mode");
                    Error::UnsupportedAggregationProofMode(proof_mode)
                })?;

            // Get the actual end_block from the proof's public values
            let end_block = public_values.l2_block_number;

            info!(%last_proven_block, %end_block, "Aggregation proof successfully acquired");

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
    let safe_head_response = l2_rpc.get_safe_head_at_l1_block(safe_l1_block).await.map_err(|e| {
        error!(%safe_l1_block, %l1_block_number, "Failed to fetch safe head at L1 block: {}", e);
        e
    })?;
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
