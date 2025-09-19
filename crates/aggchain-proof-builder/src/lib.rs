pub mod config;
mod error;

#[cfg(test)]
mod tests;

use std::{
    panic::AssertUnwindSafe,
    sync::Arc,
    task::{Context, Poll},
};

use aggchain_proof_contracts::{
    contracts::{
        GetTrustedSequencerAddress, L1OpSuccinctConfigFetcher, L2EvmStateSketchFetcher,
        L2LocalExitRootFetcher, L2OutputAtBlockFetcher,
    },
    AggchainContractsClient,
};
use aggchain_proof_core::{
    bridge::{inserted_ger::InsertedGER, BridgeWitness},
    full_execution_proof::{
        AggchainParamsValues, AggregationProofPublicValues, BabyBearDigest, ClaimRoot, FepInputs,
    },
    proof::{AggchainProofWitness, IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION},
};
use aggchain_proof_types::AggchainProofInputs;
use aggkit_prover_types::vkey_hash::{VKeyHash, Sp1VKeyHash};
use agglayer_interop::types::{
    bincode, GlobalIndexWithLeafHash, ImportedBridgeExitCommitmentValues,
};
use agglayer_primitives::{Address, Digest};
use alloy::eips::BlockNumberOrTag;
pub use error::Error;
use eyre::Context as _;
use futures::{future::BoxFuture, FutureExt, TryFutureExt as _};
use prover_executor::{sp1_async, sp1_fast, Executor, ProofType};
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, SP1Stdin, SP1VerifyingKey};
use tower::{buffer::Buffer, util::BoxService, ServiceExt as _};
use tracing::{debug, error, info};
use unified_bridge::AggchainProofPublicValues;

use crate::config::AggchainProofBuilderConfig;

const MAX_CONCURRENT_REQUESTS: usize = 100;

pub const AGGCHAIN_PROOF_ELF: &[u8] = agglayer_elf_build::elf_bytes!();

/// Hardcoded hash of the "aggregation vkey".
/// NOTE: Format being `hash_u32()` of the `SP1StarkVerifyingKey`.
pub const AGGREGATION_VKEY_HASH: VKeyHash = proposer_elfs::aggregation::VKEY_HASH;

/// Specific commitment for the range proofs.
pub const RANGE_VKEY_COMMITMENT: [u8; 32] = proposer_elfs::range::VKEY_COMMITMENT;

pub(crate) type ProverService = Buffer<
    BoxService<prover_executor::Request, prover_executor::Response, prover_executor::Error>,
    prover_executor::Request,
>;

/// All the data `aggchain-proof-builder` needs for the agghchain
/// proof generation. Collected from various sources.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggchainProverInputs {
    pub output_root: ClaimRoot,
    pub stdin: SP1Stdin,
}

pub enum FepVerification {
    Proof {
        /// Aggregated full execution proof for the number of aggregated block
        /// spans.
        aggregation_proof: Box<sp1_core_executor::SP1ReduceProof<sp1_prover::InnerSC>>,

        /// Aggregation proof's public values produced by the prover. Used to
        /// verify the proof.
        aggregation_proof_public_values: AggregationProofPublicValues,
    },

    Optimistic {
        signature: agglayer_primitives::Signature,
    },
}

pub struct AggchainProofBuilderRequest {
    pub fep_verification: FepVerification,

    /// Last block in the agg_span_proof provided by the proposer.
    /// Could be different from the requested_end_block requested by the
    /// agg-sender.
    pub end_block: u64,

    /// Aggchain proof partial prover inputs coming from the aggsender request.
    pub aggchain_proof_inputs: AggchainProofInputs,
}

pub struct AggchainProofBuilderResponse {
    /// Generated aggchain proof for the block range.
    pub proof: Vec<u8>,

    /// Verification key for the aggchain proof.
    pub vkey: Vec<u8>,

    /// Aggchain params.
    pub aggchain_params: Digest,

    /// Last block proven, before this aggchain proof.
    pub last_proven_block: u64,

    /// Last block included in the aggchain proof.
    pub end_block: u64,

    /// Output root.
    pub output_root: ClaimRoot,

    /// New Local exit root.
    pub new_local_exit_root: Digest,

    /// The public inputs that were provided to the proof
    pub public_values: AggchainProofPublicValues,
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

    /// Verification key for the aggregated fep proof.
    aggregation_vkey: Arc<SP1VerifyingKey>,

    /// Verification key for the aggchain proof.
    aggchain_vkey: Arc<SP1VerifyingKey>,

    /// Static call caller address.
    static_call_caller_address: Address,
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
    ) -> eyre::Result<Self> {
        let executor = Executor::new(
            config.primary_prover.clone(),
            config.fallback_prover.clone(),
            AGGCHAIN_PROOF_ELF,
        )
        .await
        .context("Failed creating executor for AggchainProofBuilder")?;

        let aggchain_vkey = executor.get_vkey().clone();
        let executor = tower::ServiceBuilder::new().service(executor).boxed();

        let prover = Buffer::new(executor, MAX_CONCURRENT_REQUESTS);

        // Retrieve the entire aggregation vkey from the ELF
        let aggregation_vkey = proposer_elfs::aggregation::VKEY.vkey().clone();

        // Check mismatch on aggregation vkey
        {
            let retrieved = sp1_fast(|| VKeyHash::from_vkey(&aggregation_vkey))
                .context("Computing VKey hash")?;
            let expected = AGGREGATION_VKEY_HASH;

            if retrieved != expected {
                return Err(eyre::Report::from(Error::MismatchAggregationVkeyHash {
                    got: retrieved,
                    expected,
                }));
            }
        }

        Ok(AggchainProofBuilder {
            aggchain_vkey,
            contracts_client,
            prover,
            network_id: config.network_id,
            aggregation_vkey: Arc::new(aggregation_vkey),
            static_call_caller_address: config.contracts.static_call_caller_address,
        })
    }

    /// Retrieve l1 and l2 public data needed for aggchain proof generation.
    /// Combine with the rest of the inputs to form an `AggchainProverInputs`.
    pub(crate) async fn retrieve_chain_data(
        contracts_client: Arc<ContractsClient>,
        request: AggchainProofBuilderRequest,
        network_id: u32,
        aggregation_vkey: Arc<SP1VerifyingKey>,
        static_call_caller_address: Address,
    ) -> Result<AggchainProverInputs, Error>
    where
        ContractsClient: L2LocalExitRootFetcher
            + L2OutputAtBlockFetcher
            + L2EvmStateSketchFetcher
            + GetTrustedSequencerAddress
            + L1OpSuccinctConfigFetcher,
    {
        info!(last_proven_block=%request.aggchain_proof_inputs.last_proven_block,
            end_block=%request.end_block,
            "Retrieving chain data for aggchain proof generation");
        let new_blocks_range =
            (request.aggchain_proof_inputs.last_proven_block + 1)..=request.end_block;

        // Fetch from RPCs
        let prev_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.aggchain_proof_inputs.last_proven_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let new_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let l2_pre_root_output_at_block = contracts_client
            .get_l2_output_at_block(request.aggchain_proof_inputs.last_proven_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let claim_root_output_at_block = contracts_client
            .get_l2_output_at_block(request.end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let op_succinct_config = contracts_client
            .get_op_succinct_config()
            .await
            .map_err(Error::L1ChainDataRetrievalError)?;

        let prev_l2_block_sketch = contracts_client
            .get_prev_l2_block_sketch(BlockNumberOrTag::Number(
                request.aggchain_proof_inputs.last_proven_block,
            ))
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let new_l2_block_sketch = contracts_client
            .get_new_l2_block_sketch(BlockNumberOrTag::Number(request.end_block))
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let trusted_sequencer = contracts_client
            .get_trusted_sequencer_address()
            .await
            .map_err(Error::UnableToFetchTrustedSequencerAddress)?;

        // From the request
        let inserted_gers: Vec<InsertedGER> = request
            .aggchain_proof_inputs
            .sorted_inserted_gers(&new_blocks_range);

        // NOTE: Corresponds to all of them because we do not have removed GERs yet.
        let inserted_gers_hash_chain = inserted_gers
            .iter()
            .map(|inserted_ger| inserted_ger.ger())
            .collect();

        // NOTE: Corresponds to all of them because we do not have unset claims yet.
        let bridge_exits_claimed: Vec<GlobalIndexWithLeafHash> = request
            .aggchain_proof_inputs
            .imported_bridge_exits
            .iter()
            .filter(|ib| new_blocks_range.contains(&ib.block_number))
            .map(|ib| GlobalIndexWithLeafHash {
                global_index: ib.global_index.into(),
                bridge_exit_hash: ib.bridge_exit_hash.0,
            })
            .collect();

        let l1_info_tree_leaf = request.aggchain_proof_inputs.l1_info_tree_leaf;
        let mut fep_inputs = FepInputs {
            l1_head: l1_info_tree_leaf.inner.block_hash,
            claim_block_num: request.end_block as u32,
            rollup_config_hash: op_succinct_config.rollup_config_hash,
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
            aggregation_vkey_hash: BabyBearDigest(aggregation_vkey.hash_babybear()),
            range_vkey_commitment: RANGE_VKEY_COMMITMENT,
        };

        {
            match request.fep_verification {
                FepVerification::Proof {
                    ref aggregation_proof_public_values,
                    ..
                } => {
                    let retrieved_from_contracts = AggregationProofPublicValues::from(&fep_inputs);

                    if aggregation_proof_public_values != &retrieved_from_contracts {
                        error!(
                            "Mismatch between the aggregation proof public values - retrieved \
                             from the contracts: {retrieved_from_contracts:?}, received with the \
                             proof: {:?}",
                            aggregation_proof_public_values
                        );
                        return Err(Error::MismatchAggregationProofPublicValues {
                            expected_by_contract: Box::new(retrieved_from_contracts),
                            expected_by_verifier: Box::new(aggregation_proof_public_values.clone()),
                        });
                    }
                }
                FepVerification::Optimistic { signature } => {
                    fep_inputs.signature_optimistic_mode = Some(signature);
                }
            }

            info!(
                "Aggchain-params unrolled values: {:?}; Aggchain-params keccak-hashed: {}",
                AggchainParamsValues::from(&fep_inputs),
                fep_inputs.aggchain_params()
            );

            let prover_witness = AggchainProofWitness {
                prev_local_exit_root,
                new_local_exit_root,
                l1_info_root: request.aggchain_proof_inputs.l1_info_tree_root_hash,
                origin_network: network_id,
                fep: fep_inputs,
                commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues {
                    claims: bridge_exits_claimed.clone(),
                }
                .commitment(IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION),
                bridge_witness: BridgeWitness {
                    inserted_gers,
                    bridge_exits_claimed,
                    global_indices_unset: vec![], // NOTE: no unset yet.
                    raw_inserted_gers: inserted_gers_hash_chain,
                    removed_gers: vec![], // NOTE: no removed GERs yet.
                    prev_l2_block_sketch,
                    new_l2_block_sketch,
                    caller_address: static_call_caller_address,
                },
            };

            let output_root = prover_witness.fep.compute_claim_root();

            let sp1_stdin = sp1_fast(|| {
                let mut stdin = SP1Stdin::new();
                stdin.write(&prover_witness);

                if let FepVerification::Proof {
                    aggregation_proof, ..
                } = request.fep_verification
                {
                    stdin.write_proof(*aggregation_proof, aggregation_vkey.vk.clone());
                }
                stdin
            })
            .context("Failed to build SP1 stdin")
            .map_err(Error::Other)?;

            info!(last_proven_block=%request.aggchain_proof_inputs.last_proven_block,
                end_block=%request.end_block,
                "Chain data for aggchain proof generation successfully retrieved");

            Ok(AggchainProverInputs {
                output_root,
                stdin: sp1_stdin,
            })
        }
    }
}

impl<ContractsClient> tower::Service<AggchainProofBuilderRequest>
    for AggchainProofBuilder<ContractsClient>
where
    ContractsClient: AggchainContractsClient + GetTrustedSequencerAddress + Send + Sync + 'static,
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
        let aggregation_vkey = self.aggregation_vkey.clone();
        let aggchain_vkey = self.aggchain_vkey.clone();
        let static_call_caller_address = self.static_call_caller_address;

        // TODO: figure out a way to stop only this service upon an sp1 panic, and not
        // the entire system. For now, just ignore the panic, even though some
        // internal mutability inside sp1 might end up unhappy.
        sp1_async(AssertUnwindSafe(async move {
            let last_proven_block = req.aggchain_proof_inputs.last_proven_block;
            let end_block = req.end_block;
            info!(%last_proven_block, %end_block, "Starting generation of the aggchain proof");
            // Retrieve all the necessary public inputs. Combine with
            // the data provided by the agg-sender in the request.
            let aggchain_prover_inputs = Self::retrieve_chain_data(
                contracts_client,
                req,
                network_id,
                aggregation_vkey,
                static_call_caller_address,
            )
            .await?;

            let output_root = aggchain_prover_inputs.output_root;
            let prover_executor::Response { proof } = prover
                .ready()
                .await
                .map_err(Error::ProverServiceReadyError)?
                .call(prover_executor::Request {
                    stdin: aggchain_prover_inputs.stdin,
                    proof_type: ProofType::Stark,
                })
                .await
                .map_err(Error::ProverFailedToExecute)?;

            let public_input: AggchainProofPublicValues = bincode::sp1v4()
                .deserialize(proof.public_values.as_slice())
                .unwrap();

            let stark = proof
                .proof
                .try_as_compressed()
                .ok_or(Error::GeneratedProofIsNotCompressed)?;

            debug!(
                "AP public values: prev_local_exit_root: {:?}, new_local_exit_root: {:?}, \
                 l1_info_root: {:?}, origin_network: {:?}, aggchain_params: {:?}, \
                 commit_imported_bridge_exits: {:?}",
                public_input.prev_local_exit_root,
                public_input.new_local_exit_root,
                public_input.l1_info_root,
                public_input.origin_network,
                public_input.aggchain_params,
                public_input.commit_imported_bridge_exits
            );

            info!(%last_proven_block, %end_block, "Aggchain proof generated");

            Ok(AggchainProofBuilderResponse {
                vkey: bincode::default()
                    .serialize(&aggchain_vkey)
                    .map_err(Error::UnableToSerializeVkey)?,
                proof: bincode::default()
                    .serialize(&stark)
                    .map_err(Error::UnableToSerializeProof)?,
                aggchain_params: public_input.aggchain_params,
                last_proven_block,
                end_block,
                output_root,
                new_local_exit_root: public_input.new_local_exit_root,
                public_values: public_input,
            })
        }))
        .map_err(Error::Other)
        .and_then(|res| async { res })
        .boxed()
    }
}
