pub mod config;
mod error;

use std::sync::Arc;
use std::task::{Context, Poll};

use aggchain_proof_contracts::contracts::{
    L1RollupConfigHashFetcher, L2EvmStateSketchFetcher, L2LocalExitRootFetcher,
    L2OutputAtBlockFetcher,
};
use aggchain_proof_contracts::AggchainContractsClient;
use aggchain_proof_core::bridge::inserted_ger::InsertedGER;
use aggchain_proof_core::bridge::{
    compute_imported_bridge_exits_commitment, BridgeWitness, GlobalIndexWithLeafHash,
};
use aggchain_proof_core::full_execution_proof::{
    AggregationOutputs, FepInputs, AGGREGATION_VKEY_HASH,
};
use aggchain_proof_core::proof::{AggchainProofPublicValues, AggchainProofWitness};
use aggchain_proof_core::Digest;
use aggchain_proof_types::AggchainProofInputs;
use agglayer_primitives::utils::Hashable;
use alloy::eips::BlockNumberOrTag;
use alloy_primitives::{b256, Address};
use bincode::Options;
pub use error::Error;
use futures::{future::BoxFuture, FutureExt};
use prover_executor::{Executor, ProofType};
use sp1_sdk::{HashableKey, Prover, ProverClient, SP1Stdin, SP1VerifyingKey};
use tower::buffer::Buffer;
use tower::util::BoxService;
use tower::ServiceExt as _;

use crate::config::AggchainProofBuilderConfig;

const MAX_CONCURRENT_REQUESTS: usize = 100;
pub const AGGCHAIN_PROOF_ELF: &[u8] =
    include_bytes!("../../../crates/aggchain-proof-program/elf/riscv32im-succinct-zkvm-elf");

pub const AGGREGATION_PROOF_ELF: &[u8] =
    include_bytes!("../../../crates/aggchain-proof-program/elf/aggregation-elf");

pub(crate) type ProverService = Buffer<
    BoxService<prover_executor::Request, prover_executor::Response, prover_executor::Error>,
    prover_executor::Request,
>;

/// All the data `aggchain-proof-builder` needs for the agghchain
/// proof generation. Collected from various sources.
pub struct AggchainProverInputs {
    pub proof_witness: AggchainProofWitness,
    pub stdin: SP1Stdin,
    pub last_proven_block: u64,
    pub end_block: u64,
}

pub struct AggchainProofBuilderRequest {
    /// Aggregated full execution proof for the number of aggregated block
    /// spans.
    pub aggregation_proof: Box<sp1_core_executor::SP1ReduceProof<sp1_prover::InnerSC>>,

    /// Last block in the agg_span_proof provided by the proposer.
    /// Could be different from the requested_end_block requested by the
    /// agg-sender.
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

    /// Last block proven, before this aggchain proof.
    pub last_proven_block: u64,

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
                AGGCHAIN_PROOF_ELF,
            ))
            .boxed();

        let prover = Buffer::new(executor, MAX_CONCURRENT_REQUESTS);
        let aggchain_proof_vkey = Executor::get_vkey(AGGCHAIN_PROOF_ELF);

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
            + L2EvmStateSketchFetcher
            + L1RollupConfigHashFetcher,
    {
        let new_blocks_range =
            (request.aggchain_proof_inputs.last_proven_block + 1)..=request.end_block;

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 01");
        // Fetch from RPCs
        let prev_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.aggchain_proof_inputs.last_proven_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 03");

        let new_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 04");

        let l2_pre_root_output_at_block = contracts_client
            .get_l2_output_at_block(request.aggchain_proof_inputs.last_proven_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;
        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 05");

        let claim_root_output_at_block = contracts_client
            .get_l2_output_at_block(request.end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 06");

        let rollup_config_hash = contracts_client
            .get_rollup_config_hash()
            .await
            .map_err(Error::L1ChainDataRetrievalError)?;

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 07");

        let prev_l2_block_sketch = contracts_client
            .get_prev_l2_block_sketch(BlockNumberOrTag::Number(
                request.aggchain_proof_inputs.last_proven_block,
            ))
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 08");

        let new_l2_block_sketch = contracts_client
            .get_new_l2_block_sketch(BlockNumberOrTag::Number(request.end_block))
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 09");

        let trusted_sequencer = Address::default(); // TODO: from config or l1

        // From the request
        let inserted_gers: Vec<InsertedGER> = request
            .aggchain_proof_inputs
            .ger_leaves
            .values()
            .filter(|inserted_ger| new_blocks_range.contains(&inserted_ger.block_number))
            .cloned()
            .map(|e| InsertedGER {
                proof: e.inserted_ger.proof_ger_l1root,
                l1_info_tree_leaf: e.inserted_ger.l1_leaf,
            })
            .collect();

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 10");

        // NOTE: Corresponds to all of them because we do not have removed GERs yet.
        let inserted_gers_hash_chain = inserted_gers
            .iter()
            .map(|inserted_ger| inserted_ger.ger())
            .collect();

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 11");

        // NOTE: Corresponds to all of them because we do not have unset claims yet.
        let bridge_exits_claimed: Vec<GlobalIndexWithLeafHash> = request
            .aggchain_proof_inputs
            .imported_bridge_exits
            .iter()
            .filter(|ib| new_blocks_range.contains(&ib.block_number))
            .map(|ib| GlobalIndexWithLeafHash {
                global_index: ib.imported_bridge_exit.global_index.into(),
                bridge_exit_hash: ib.imported_bridge_exit.bridge_exit.hash(),
            })
            .collect();

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 12");

        let l1_info_tree_leaf = request.aggchain_proof_inputs.l1_info_tree_leaf;
        let mut fep = FepInputs {
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
            signature_optimistic_mode: None, // NOTE: disabled for now
            l1_info_tree_leaf,
            l1_head_inclusion_proof: request.aggchain_proof_inputs.l1_info_tree_merkle_proof,
        };

        let retrieved_pv = AggregationOutputs::from(&fep);
        println!("retrieved pv: {:?}", retrieved_pv);

        // workaround
        fep.rollup_config_hash =
            b256!("8a3f045ea5a3e7dbc2800ec2a0e61b8a31433ca07cadae822d7b35631ca7ce52")
                .0
                .into();

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 13");

        let prover_witness = AggchainProofWitness {
            prev_local_exit_root,
            new_local_exit_root,
            l1_info_root: request.aggchain_proof_inputs.l1_info_tree_root_hash,
            origin_network: network_id,
            fep,
            commit_imported_bridge_exits: compute_imported_bridge_exits_commitment(
                &bridge_exits_claimed,
            ),
            bridge_witness: BridgeWitness {
                inserted_gers,
                bridge_exits_claimed,
                global_indices_unset: vec![], // NOTE: no unset yet.
                raw_inserted_gers: inserted_gers_hash_chain,
                removed_gers: vec![], // NOTE: no removed GERs yet.
                prev_l2_block_sketch,
                new_l2_block_sketch,
            },
        };

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 14");
        let aggregation_proof = request.aggregation_proof.clone();
        let aggregation_vkey = aggregation_proof.vk.clone();
        let witness = prover_witness.clone();

        let prover = ProverClient::builder().cpu().build();
        let (_, agg_vk_from_elf) = prover.setup(AGGREGATION_PROOF_ELF);

        // mismatch verification
        {
            let hardcoded_vkey_hash = AGGREGATION_VKEY_HASH;
            let received_vkey_hash_u32 = aggregation_vkey.hash_u32();
            println!("hardcoded aggregation vkey hash: {hardcoded_vkey_hash:?}");
            println!("received hash u32: {received_vkey_hash_u32:?}");
            println!("from elf hash u32: {:?}", agg_vk_from_elf.hash_u32());
        }

        let sp1_stdin = {
            let mut stdin = SP1Stdin::new();
            stdin.write(&prover_witness);
            stdin.write_proof(*aggregation_proof, agg_vk_from_elf.vk);
            stdin
        };

        println!(">>>>>>>>>> AggchainProofBuilder RetrieveChainData Checkpoint 15");

        Ok(AggchainProverInputs {
            last_proven_block: request.aggchain_proof_inputs.last_proven_block,
            end_block: request.end_block,
            stdin: sp1_stdin,
            proof_witness: witness,
        })
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
        println!(">>>>>>>>>> AggchainProofBuilder Checkpoint 11");
        self.prover.poll_ready(cx).map_err(|e| {
            println!(">>>>>>>>>> AggchainProofBuilder Checkpoint 12");
            if let Some(error) = e.downcast_ref::<prover_executor::Error>() {
                println!(">>>>>>>>>> AggchainProofBuilder Checkpoint 13");
                Error::ProverExecutorError(error.clone())
            } else {
                println!(">>>>>>>>>> AggchainProofBuilder Checkpoint 14");
                Error::ProverServiceError(e.to_string())
            }
        })
    }

    fn call(&mut self, req: AggchainProofBuilderRequest) -> Self::Future {
        println!(">>>>>>>>>> AggchainProofBuilder Checkpoint 01");
        let contracts_client = self.contracts_client.clone();
        let mut prover = self.prover.clone();
        let network_id = self.network_id;

        async move {
            let last_proven_block = req.aggchain_proof_inputs.last_proven_block;
            let end_block = req.end_block;
            // Retrieve all the necessary public inputs. Combine with
            // the data provided by the agg-sender in the request.
            let aggchain_prover_inputs =
                Self::retrieve_chain_data(contracts_client, req, network_id).await?;

            let prover_executor::Response { proof } = prover
                .ready()
                .await
                .map_err(Error::ProverServiceReadyError)?
                .call(prover_executor::Request {
                    stdin: aggchain_prover_inputs.stdin,
                    proof_type: ProofType::Stark,
                })
                .await
                .map_err(|error| Error::ProverFailedToExecute(anyhow::Error::from_boxed(error)))?;

            println!(">>>>>>>>>> AggchainProofBuilder Checkpoint 04");

            let public_input: AggchainProofPublicValues =
                bincode::deserialize(proof.public_values.as_slice()).unwrap();

            let stark = proof
                .proof
                .try_as_compressed()
                .ok_or(Error::GeneratedProofIsNotCompressed)?;

            println!(">>>>>>>>>> AggchainProofBuilder Checkpoint 05");

            Ok(AggchainProofBuilderResponse {
                proof: bincode::DefaultOptions::new()
                    .with_big_endian()
                    .with_fixint_encoding()
                    .serialize(&stark)
                    .map_err(Error::UnableToSerializeProof)?,
                aggchain_params: public_input.aggchain_params.to_vec(),
                last_proven_block,
                end_block,
                // TODO: Define the output root with the witness data
                output_root: Default::default(),
            })
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregration_vkey() {
        let prover = ProverClient::builder().cpu().build();
        let (_, agg_vk_from_elf) = prover.setup(AGGREGATION_PROOF_ELF);

        println!("agg_vk hashu32 {:?}", agg_vk_from_elf.hash_u32());
    }
}
