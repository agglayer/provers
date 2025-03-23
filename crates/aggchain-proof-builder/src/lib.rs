pub mod config;
mod error;

use std::sync::Arc;
use std::task::{Context, Poll};

use aggchain_proof_contracts::contracts::{
    L1RollupConfigHashFetcher, L2EVMStateSketchesFetched, L2LocalExitRootFetcher,
    L2OutputAtBlockFetcher,
};
use aggchain_proof_contracts::AggchainContractsClient;
use aggchain_proof_core::bridge::inserted_ger::InsertedGER;
use aggchain_proof_core::bridge::BridgeWitness;
use aggchain_proof_core::full_execution_proof::FepPublicValues;
use aggchain_proof_core::proof::{AggchainProofPublicValues, AggchainProofWitness};
use aggchain_proof_core::Digest;
use aggchain_proof_types::AggchainProofInputs;
use agglayer_interop::types::GlobalIndex;
use alloy::eips::BlockNumberOrTag;
use alloy_primitives::{Address, FixedBytes};
use bincode::Options;
pub use error::Error;
use futures::{future::BoxFuture, FutureExt};
use prover_executor::{Executor, ProofType};
use sp1_sdk::network::B256;
use sp1_sdk::{SP1Stdin, SP1VerifyingKey};
use tower::buffer::Buffer;
use tower::util::BoxService;
use tower::ServiceExt as _;

use crate::config::AggchainProofBuilderConfig;

const MAX_CONCURRENT_REQUESTS: usize = 100;
pub const ELF: &[u8] =
    include_bytes!("../../../crates/aggchain-proof-program/elf/riscv32im-succinct-zkvm-elf");

pub(crate) type ProverService = Buffer<
    BoxService<prover_executor::Request, prover_executor::Response, prover_executor::Error>,
    prover_executor::Request,
>;

/// All the data `aggchain-proof-builder` needs for the agghchain
/// proof generation. Collected from various sources.
pub struct AggchainProverInputs {
    pub proof_witness: AggchainProofWitness,
    pub stdin: SP1Stdin,
    pub start_block: u64,
    pub end_block: u64,
}

pub struct AggchainProofBuilderRequest {
    /// Aggregated full execution proof for the number of aggregated block
    /// spans.
    pub aggregation_proof: Box<sp1_core_executor::SP1ReduceProof<sp1_prover::InnerSC>>,
    /// Last block in the agg_span_proof provided by the proposer.
    /// Could be different from the max_end_block requested by the agg-sender.
    pub end_block: u64,
    /// Aggchain proof request information, public inputs, bridge data,...
    pub aggchain_proof_inputs: AggchainProofInputs,
}

#[derive(Clone, Debug)]
pub struct AggchainProofBuilderResponse {
    /// Generated aggchain proof for the block range.
    pub proof: Vec<u8>,
    /// Aggchain params
    pub aggchain_params: Vec<u8>,
    /// First block included in the aggchain proof.
    pub start_block: u64,
    /// Last block included in the aggchain proof.
    pub end_block: u64,
    /// Output root
    pub output_root: Digest,
}

/// This service is responsible for building an Aggchain proof.
#[derive(Clone)]
#[allow(unused)]
pub struct AggchainProofBuilder<ContractsClient> {
    /// Client for interacting with the smart contracts relevant for the
    /// aggchain prover.
    contracts_client: Arc<ContractsClient>,

    /// Network id of the l2 chain for which the proof is generated.
    network_id: u32,

    /// Prover client service.
    prover: ProverService,

    /// Verification key for the aggchain proof.
    aggchain_proof_vkey: SP1VerifyingKey,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum WitnessGeneration {
    #[error("Invalid inserted GER.")]
    InvalidInsertedGer,
    #[error("Cannot interpret the aggregation proof as 'compressed' type.")]
    WrongAggregationProofType,
}

pub fn encoded_global_index(value: &GlobalIndex) -> FixedBytes<32> {
    let mut bytes = [0u8; 32];

    let leaf_bytes = value.leaf_index.to_le_bytes();
    bytes[0..4].copy_from_slice(&leaf_bytes);

    let rollup_bytes = value.rollup_index.to_le_bytes();
    bytes[4..8].copy_from_slice(&rollup_bytes);

    if value.mainnet_flag {
        bytes[8] |= 0x01;
    }

    bytes.into()
}

impl<ContractsClient> AggchainProofBuilder<ContractsClient> {
    pub async fn new(
        config: &AggchainProofBuilderConfig,
        contracts_client: Arc<ContractsClient>,
    ) -> Result<Self, Error> {
        let executor = tower::ServiceBuilder::new()
            .service(Executor::new(
                &config.primary_prover,
                &config.fallback_prover,
                ELF,
            ))
            .boxed();

        let prover = Buffer::new(executor, MAX_CONCURRENT_REQUESTS);
        let aggchain_proof_vkey = Executor::get_vkey(ELF);

        Ok(AggchainProofBuilder {
            contracts_client,
            prover,
            network_id: config.network_id,
            aggchain_proof_vkey,
        })
    }

    /// Retrieve l1 and l2 public data needed for aggchain proof generation.
    /// Combine with the rest of the inputs to form an `AggchainProverInputs`.
    pub(crate) async fn retrieve_chain_data(
        contracts_client: Arc<ContractsClient>,
        request: AggchainProofBuilderRequest,
        network_id: u32,
    ) -> Result<AggchainProverInputs, Error>
    where
        ContractsClient: L2LocalExitRootFetcher
            + L2OutputAtBlockFetcher
            + L2EVMStateSketchesFetched
            + L1RollupConfigHashFetcher,
    {
        let block_range = request.aggchain_proof_inputs.start_block..request.end_block; // Handle +-1

        // Fetch from RPCs
        let prev_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.aggchain_proof_inputs.start_block - 1)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let new_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let l2_pre_root_output_at_block = contracts_client
            .get_l2_output_at_block(request.aggchain_proof_inputs.start_block - 1)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let claim_root_output_at_block = contracts_client
            .get_l2_output_at_block(request.end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let rollup_config_hash = contracts_client
            .get_rollup_config_hash()
            .await
            .map_err(Error::L1ChainDataRetrievalError)?;

        let prev_l2_block_sketch = contracts_client
            .get_prev_l2_block_sketch(BlockNumberOrTag::Number(
                request.aggchain_proof_inputs.start_block,
            ))
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let new_l2_block_sketch = contracts_client
            .get_new_l2_block_sketch(BlockNumberOrTag::Number(request.end_block))
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let trusted_sequencer = Address::default(); // TODO: from config or l1

        // From the request
        let inserted_gers: Vec<InsertedGER> = request
            .aggchain_proof_inputs
            .ger_leaves
            .values()
            .filter(|inserted_ger| block_range.contains(&inserted_ger.block_number))
            .cloned()
            .map(|elmt| {
                Ok(InsertedGER {
                    proof: elmt.inserted_ger.proof_ger_l1root,
                    l1_info_tree_leaf: elmt.inserted_ger.l1_leaf,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // NOTE: Corresponds to all of them because we do not have removed GERs yet.
        let inserted_gers_hash_chain = inserted_gers
            .iter()
            .map(|inserted_ger| inserted_ger.ger())
            .collect();

        // NOTE: Corresponds to all of them because we do not have unset claims yet.
        let global_indices: Vec<B256> = request
            .aggchain_proof_inputs
            .imported_bridge_exits
            .iter()
            .filter(|ib| block_range.contains(&ib.block_number))
            .map(|ib| encoded_global_index(&ib.imported_bridge_exit.global_index))
            .collect();

        let l1_info_tree_leaf = request.aggchain_proof_inputs.l1_info_tree_leaf;
        let fep = FepPublicValues {
            l1_head: l1_info_tree_leaf.inner.block_hash,
            claim_block_num: request.end_block as u32,
            rollup_config_hash,
            prev_state_root: l2_pre_root_output_at_block.state_root,
            prev_withdrawal_storage_root: l2_pre_root_output_at_block.withdrawal_storage_root,
            prev_block_hash: l2_pre_root_output_at_block.latest_block_hash,
            new_state_root: claim_root_output_at_block.state_root,
            new_withdrawal_storage_root: claim_root_output_at_block.withdrawal_storage_root,
            new_block_hash: claim_root_output_at_block.latest_block_hash,
            trusted_sequencer,
            signature_optimistic_mode: None, // unsupported for now
        };

        let prover_witness = AggchainProofWitness {
            prev_local_exit_root,
            new_local_exit_root,
            l1_info_root: request.aggchain_proof_inputs.l1_info_tree_root_hash,
            origin_network: network_id,
            fep,
            l1_info_tree_leaf,
            l1_head_inclusion_proof: request.aggchain_proof_inputs.l1_info_tree_merkle_proof,
            global_indices: global_indices.clone(),
            bridge_witness: BridgeWitness {
                inserted_gers,
                global_indices_claimed: global_indices,
                global_indices_unset: vec![], // NOTE: no unset yet.
                raw_inserted_gers: inserted_gers_hash_chain,
                removed_gers: vec![], // NOTE: no removed GERs yet.
                prev_l2_block_sketch,
                new_l2_block_sketch,
            },
        };

        let aggregation_proof = request.aggregation_proof;
        let aggregation_vkey = aggregation_proof.vk.clone();
        let witness = prover_witness.clone();
        let sp1_stdin = {
            let mut stdin = SP1Stdin::new();
            stdin.write(&prover_witness);
            stdin.write_proof(*aggregation_proof, aggregation_vkey);
            stdin
        };

        Ok(AggchainProverInputs {
            start_block: request.aggchain_proof_inputs.start_block,
            end_block: request.end_block,
            stdin: sp1_stdin,
            proof_witness: witness,
        })
    }

    /// Generate aggchain proof
    #[allow(unused)]
    pub(crate) async fn generate_aggchain_proof(
        mut _prover: ProverService,
        _inputs: AggchainProverInputs,
    ) -> Result<AggchainProofBuilderResponse, Error> {
        todo!()
    }
}

impl<ContractsClient> tower::Service<AggchainProofBuilderRequest>
    for AggchainProofBuilder<ContractsClient>
where
    ContractsClient: AggchainContractsClient + Send + Sync + 'static,
{
    type Response = AggchainProofBuilderResponse;

    type Error = Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.prover.poll_ready(cx).map_err(|e| {
            if let Some(error) = e.downcast_ref::<prover_executor::Error>() {
                Error::ProverExecutorError(error.clone())
            } else {
                Error::ProverServiceError(e.to_string())
            }
        })
    }

    fn call(&mut self, req: AggchainProofBuilderRequest) -> Self::Future {
        let contracts_client = self.contracts_client.clone();
        let mut prover = self.prover.clone();
        let network_id = self.network_id;
        async move {
            let start_block = req.aggchain_proof_inputs.start_block;
            let end_block = req.end_block;
            // Retrieve all the necessary public inputs. Combine with
            // the data provided by the agg-sender in the request.
            let aggchain_prover_inputs =
                Self::retrieve_chain_data(contracts_client, req, network_id).await?;

            let result = prover
                .call(prover_executor::Request {
                    stdin: aggchain_prover_inputs.stdin,
                    proof_type: ProofType::Stark,
                })
                .await
                .map_err(|error| Error::ProverFailedToExecute(anyhow::Error::from_boxed(error)));

            match result {
                Ok(prover_executor::Response { proof }) => {
                    let public_input: AggchainProofPublicValues =
                        bincode::deserialize(proof.public_values.as_slice()).unwrap();

                    let stark = proof
                        .proof
                        .try_as_compressed()
                        .ok_or(Error::GeneratedProofIsNotCompressed)?;

                    Ok(AggchainProofBuilderResponse {
                        proof: bincode::DefaultOptions::new()
                            .with_big_endian()
                            .with_fixint_encoding()
                            .serialize(&stark)
                            .map_err(Error::UnableToSerializeProof)?,
                        aggchain_params: public_input.aggchain_params.to_vec(),
                        start_block,
                        end_block,
                        // TODO: Define the output root with the witness data
                        output_root: Default::default(),
                    })
                }
                Err(_) => todo!(),
            }
        }
        .boxed()
    }
}
