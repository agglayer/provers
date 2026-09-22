pub mod config;
mod error;

#[cfg(test)]
mod tests;

use std::{
    hash::Hash,
    panic::AssertUnwindSafe,
    sync::Arc,
    task::{Context, Poll},
};

use aggchain_proof_contracts::{
    contracts::{
        GetTrustedSequencerAddress, L1LatestL2OutputFetcher, L1OpSuccinctConfigFetcher,
        L2EvmStateSketchFetcher, L2LocalExitRootFetcher, L2OutputAtBlockFetcher, OpSuccinctConfig,
    },
    AggchainContractsClient,
};
use aggchain_proof_core::{
    bridge::{inserted_ger::InsertedGER, BridgeWitness},
    full_execution_proof::{
        AggchainParamsValues, AggregationProofPublicValues, ClaimRoot, FepInputs, KoalaBearDigest,
    },
    proof::{AggchainProofWitness, IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION},
};
use aggchain_proof_types::AggchainProofInputs;
use aggkit_prover_types::vkey_hash::{Sp1VKeyHash, VKeyHash};
use agglayer_interop::types::{
    bincode, GlobalIndexWithLeafHash, ImportedBridgeExitCommitmentValues,
};
use agglayer_primitives::{keccak::keccak256, Address, Digest, U256};
use alloy::{eips::BlockNumberOrTag, sol_types::SolValue as _};
pub use error::Error;
use eyre::Context as _;
use futures::{future::BoxFuture, FutureExt, TryFutureExt as _};
use prover_config::ProverType;
use prover_executor::{sp1_async, sp1_fast, Executor, ProofType};
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, SP1Stdin, SP1VerifyingKey};
use tower::{buffer::Buffer, util::BoxService, ServiceExt as _};
use tracing::{debug, error, info, warn};
use unified_bridge::AggchainProofPublicValues;

use crate::config::{AggchainProgram, AggchainProofBuilderConfig};

const MAX_CONCURRENT_REQUESTS: usize = 100;

pub const AGGCHAIN_PROOF_ELF: &[u8] = include_bytes!(env!("AGGLAYER_ELF_PATH"));

/// Aggchain proof program that commits the public values it is given without
/// verifying anything, see [`AggchainProgram::Noop`]. Proven like the regular
/// program: an SP1 mock proof of it is rejected by the pessimistic proof.
pub const AGGCHAIN_PROOF_NOOP_ELF: &[u8] = include_bytes!(env!("AGGLAYER_NOOP_ELF_PATH"));

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
        aggregation_proof: Box<sp1_sdk::SP1ProofWithPublicValues>,

        /// Aggregation proof's public values produced by the prover. Used to
        /// verify the proof.
        aggregation_proof_public_values: AggregationProofPublicValues,
    },

    Optimistic {
        signature: agglayer_primitives::Signature,
    },

    /// Noop program, normal (non-optimistic) request: nothing is verified, so
    /// no aggregation proof is requested from the proposer. An optimistic
    /// request keeps its `Optimistic` variant, the noop program ignores the
    /// signature.
    Noop,
}

impl FepVerification {
    /// Returns the optimistic mode signature if any.
    pub fn optimistic_mode_signature(&self) -> Option<agglayer_primitives::Signature> {
        match &self {
            FepVerification::Proof { .. } | FepVerification::Noop => None,
            FepVerification::Optimistic { signature } => Some(*signature),
        }
    }

    /// Whether the certificate is an optimistic one: the flag the contract
    /// hashes into the aggchain params.
    fn is_optimistic(&self) -> bool {
        matches!(self, FepVerification::Optimistic { .. })
    }
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

/// Filters out values from a list based on a set of keys to remove, using a key
/// extraction function.
///
/// This function iterates over `values`, removing up to N occurrences of each
/// value whose key, as determined by `key_fn`, matches a key in
/// `keys_to_remove`, where N is the number of times the key appears in
/// `keys_to_remove`. The removal is performed in order, and only the first N
/// matching values are removed for each key. Remaining values are preserved in
/// their original order.
///
/// # Arguments
///
/// * `keys_to_remove` - A slice of keys indicating which values to remove. Each
///   occurrence of a key in this slice will remove one matching value from
///   `values`.
/// * `values` - The slice of values to filter.
/// * `key_fn` - A function that extracts a key from a value for comparison.
///
/// # Returns
///
/// Returns a `Result` containing a `Vec<V>` of the filtered values, or an error
/// if an overflow occurs while counting removals.
///
/// # Example
///
/// ```
/// use aggchain_proof_builder::filter_values;
///
/// let keys_to_remove = [1, 2, 2];
/// let values = [1, 2, 2, 3, 4];
/// let filtered = filter_values(&keys_to_remove, &values, |v| *v).unwrap();
/// assert_eq!(filtered, vec![3, 4]);
/// ```
///
/// # Errors
///
/// Returns `Error::FilteringValuesOverflow` if the removal count for any key
/// would overflow `usize`.
pub fn filter_values<K, V, KF>(
    keys_to_remove: &[K],
    values: &[V],
    mut key_fn: KF,
) -> Result<Vec<V>, Error>
where
    K: Eq + Hash + Copy,
    V: Clone,
    KF: FnMut(&V) -> K,
{
    use std::collections::HashMap;

    // Count how many times each key should be removed
    let mut removal_map: HashMap<K, usize> = HashMap::new();
    for &key in keys_to_remove {
        let count = removal_map.entry(key).or_insert(0);
        *count = count
            .checked_add(1)
            .ok_or(Error::FilteringValuesOverflow(*count))?;
    }

    // For each value, if its key is in removal_map and count > 0, skip it and
    // decrement count
    let mut result = Vec::new();
    for value in values {
        let key = key_fn(value);
        if let Some(count) = removal_map.get_mut(&key) {
            if *count > 0 {
                *count -= 1;
                continue;
            }
        }
        result.push(value.clone());
    }

    Ok(result)
}

/// Filters, sorts, and maps items from an iterator based on a block number
/// range.
///
/// This function takes an iterator of items, filters them to include only those
/// whose block number (as determined by `block_number_fn`) falls within the
/// specified `range`, sorts the filtered items using their `Ord`
/// implementation, and then maps each item to a new type using the provided
/// `map_fn`.
///
/// # Type Parameters
/// - `T`: The type of the input items. Must implement `Ord`.
/// - `F`: The mapping function type. Must be a function or closure that takes
///   `T` and returns `U`.
/// - `U`: The type of the output items.
///
/// # Arguments
/// - `items`: An iterator of items to process.
/// - `range`: The inclusive range of block numbers to filter by.
/// - `block_number_fn`: A function that extracts the block number from an item.
/// - `map_fn`: A function that maps each filtered and sorted item to the
///   desired output type.
///
/// # Returns
/// An iterator over the mapped items, filtered and sorted as described.
///
/// # Example
/// ```no_run
/// # use aggchain_proof_builder::filter_sort_map;
/// # struct Item { block_number: u64 }
/// # impl Item {
/// #     fn to_output_type(self) -> u64 { self.block_number }
/// # }
/// # impl Ord for Item {
/// #     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
/// #         self.block_number.cmp(&other.block_number)
/// #     }
/// # }
/// # impl PartialOrd for Item {
/// #     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
/// #         Some(self.cmp(other))
/// #     }
/// # }
/// # impl Eq for Item {}
/// # impl PartialEq for Item {
/// #     fn eq(&self, other: &Self) -> bool {
/// #         self.block_number == other.block_number
/// #     }
/// # }
/// let item1 = Item { block_number: 100 };
/// let item2 = Item { block_number: 150 };
/// let item3 = Item { block_number: 250 };
/// let items = vec![item1, item2, item3];
/// let range = 100..=200;
/// let result: Vec<_> = filter_sort_map(
///     items,
///     &range,
///     |item| item.block_number,
///     |item| item.to_output_type(),
/// )
/// .collect();
/// assert_eq!(result, vec![100, 150]);
/// ```
pub fn filter_sort_map<T, F, U>(
    items: impl IntoIterator<Item = T>,
    range: &std::ops::RangeInclusive<u64>,
    block_number_fn: fn(&T) -> u64,
    map_fn: F,
) -> impl Iterator<Item = U>
where
    F: Fn(T) -> U,
    T: Ord,
{
    let mut filtered_items: Vec<_> = items
        .into_iter()
        .filter(|item| range.contains(&block_number_fn(item)))
        .collect();
    filtered_items.sort();
    filtered_items.into_iter().map(map_fn)
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

    /// Range vkey commitment of the proposer range proofs program.
    range_vkey_commitment: Digest,

    /// Static call caller address.
    static_call_caller_address: Address,

    /// Aggchain proof program in use.
    program: AggchainProgram,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum WitnessGeneration {
    #[error("Invalid inserted GER.")]
    InvalidInsertedGer,
}

/// Aggchain params exactly as `AggchainFEP.getVKeyAndAggchainParams` packs
/// them, built from the values the contract itself uses rather than from a
/// witness. Used by the noop program.
#[derive(Clone, Debug)]
pub struct NoopAggchainParams {
    /// `l2Outputs[latestOutputIndex()].outputRoot` on L1.
    pub l2_pre_root: Digest,
    /// `optimism_outputAtBlock(claim_block_num).outputRoot` on L2.
    pub claim_root: Digest,
    pub claim_block_num: u64,
    pub rollup_config_hash: Digest,
    pub optimistic_mode: bool,
    pub trusted_sequencer: Address,
    pub range_vkey_commitment: Digest,
    /// `aggregationVkey` of the op-succinct config, already in its on-chain
    /// form.
    pub aggregation_vkey_hash: Digest,
}

impl NoopAggchainParams {
    pub fn hash(&self) -> Digest {
        let values = AggchainParamsValues {
            l2_pre_root: self.l2_pre_root.0.into(),
            claim_root: self.claim_root.0.into(),
            claim_block_num: U256::from(self.claim_block_num),
            rollup_config_hash: self.rollup_config_hash.0.into(),
            optimistic_mode: self.optimistic_mode,
            trusted_sequencer: self.trusted_sequencer.into(),
            range_vkey_commitment: self.range_vkey_commitment.0.into(),
            aggregation_vkey_hash: self.aggregation_vkey_hash.0.into(),
        };

        keccak256(values.abi_encode_packed().as_slice())
    }
}

/// Imported bridge exits of the new blocks range, as the aggchain proof sees
/// them.
struct ImportedBridgeExits {
    /// All of them, also the unclaimed ones.
    all: Vec<GlobalIndexWithLeafHash>,

    /// Global indexes of the claims unset in the range.
    unset_claims: Vec<U256>,

    /// Commitment on the ones still claimed, as committed by the proof.
    commitment: Digest,
}

fn collect_imported_bridge_exits(
    inputs: &AggchainProofInputs,
    new_blocks_range: &std::ops::RangeInclusive<u64>,
) -> Result<ImportedBridgeExits, Error> {
    let all: Vec<GlobalIndexWithLeafHash> = filter_sort_map(
        inputs.imported_bridge_exits.clone(),
        new_blocks_range,
        |ib| ib.block_number,
        |ib| GlobalIndexWithLeafHash {
            global_index: ib.global_index.into(),
            bridge_exit_hash: ib.bridge_exit_hash.0,
        },
    )
    .collect();

    let unset_claims: Vec<U256> = filter_sort_map(
        inputs.unclaims.clone(),
        new_blocks_range,
        |unclaim| unclaim.block_number,
        |unclaim| unclaim.global_index,
    )
    .collect();

    // Filter out the unset claims from the all imported bridge exits list.
    let claimed = filter_values(&unset_claims, &all, |value| value.global_index)?;

    Ok(ImportedBridgeExits {
        all,
        unset_claims,
        commitment: ImportedBridgeExitCommitmentValues { claims: claimed }
            .commitment(IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION),
    })
}

/// Root of the latest L1 output, the pre-root the contract hashes for the next
/// one. The request must be anchored at that output.
async fn latest_l1_pre_root<ContractsClient>(
    contracts_client: &ContractsClient,
    last_proven_block: u64,
) -> Result<Digest, Error>
where
    ContractsClient: L1LatestL2OutputFetcher + Sync,
{
    let l1_output = contracts_client
        .get_latest_l2_output()
        .await
        .map_err(Error::L1ChainDataRetrievalError)?;

    match l1_output {
        Some(output) if output.l2_block_number == last_proven_block => Ok(output.output_root),
        _ => Err(Error::NoopAnchorMismatch {
            last_proven_block,
            l1_latest_output_block: l1_output.map(|output| output.l2_block_number),
        }),
    }
}

impl<ContractsClient> AggchainProofBuilder<ContractsClient> {
    pub async fn new(
        config: &AggchainProofBuilderConfig,
        contracts_client: Arc<ContractsClient>,
    ) -> eyre::Result<Self>
    where
        ContractsClient: L1OpSuccinctConfigFetcher,
    {
        let program = match config.program {
            AggchainProgram::Standard => AGGCHAIN_PROOF_ELF,
            AggchainProgram::Noop => AGGCHAIN_PROOF_NOOP_ELF,
        };

        let executor = Executor::new(
            config.primary_prover.clone(),
            config.fallback_prover.clone(),
            program,
        )
        .await
        .context("Failed creating executor for AggchainProofBuilder")?;

        let aggchain_vkey = executor.get_vkey().clone();

        if config.program == AggchainProgram::Noop {
            warn!(
                noop_vkey = %Digest(aggchain_vkey.hash_bytes()),
                "Noop aggchain proof program selected: nothing is verified, the pre-root is taken \
                 from the latest L1 output"
            );

            let mock_prover = [
                Some(&config.primary_prover),
                config.fallback_prover.as_ref(),
            ]
            .into_iter()
            .flatten()
            .any(|prover| matches!(prover, ProverType::MockProver(_)));
            if mock_prover {
                warn!(
                    "The noop program is proven by the SP1 mock prover: only a mock verifier \
                     accepts that proof, a real agglayer rejects it"
                );
            }
        }
        let executor = tower::ServiceBuilder::new().service(executor).boxed();

        let prover = Buffer::new(executor, MAX_CONCURRENT_REQUESTS);

        // Resolve the aggregation vkey and range vkey commitment. These use the
        // configured op-succinct override when one was installed at startup
        // (see `proposer_elfs::install_overrides`), otherwise the
        // values embedded from op-succinct-elfs at build time.
        let aggregation_vkey = Arc::new(proposer_elfs::aggregation::vkey().clone());
        let range_vkey_commitment = Digest(proposer_elfs::range::commitment());

        // Sanity-check that the embedded op-succinct-elfs vkey constants are
        // internally consistent. The resolved (possibly overridden) key is
        // validated against the on-chain op-succinct config below instead.
        {
            let retrieved =
                sp1_fast(|| VKeyHash::from_vkey(proposer_elfs::aggregation::VKEY.vkey()))
                    .context("Computing VKey hash")?;

            if retrieved != AGGREGATION_VKEY_HASH {
                return Err(eyre::Report::from(Error::MismatchAggregationElfVkeyHash {
                    got: retrieved,
                    expected: AGGREGATION_VKEY_HASH,
                }));
            }
        }

        // Check the mismatch of the keys from the op-succinct configuration in
        // the contract
        let op_succinct_config = contracts_client
            .get_op_succinct_config()
            .await
            .map_err(Error::L1ChainDataRetrievalError)?;

        // Validate that the OpSuccinct config keys match expected values
        validate_op_succinct_config_keys(
            &op_succinct_config,
            aggregation_vkey.as_ref(),
            &range_vkey_commitment,
        )?;

        Ok(AggchainProofBuilder {
            aggchain_vkey,
            contracts_client,
            prover,
            network_id: config.network_id,
            aggregation_vkey,
            range_vkey_commitment,
            static_call_caller_address: config.contracts.static_call_caller_address,
            program: config.program,
        })
    }

    /// Inputs of the noop program: the public values assembled from the L1
    /// contract values, the L2 bridge root at both ends of the range, the L2
    /// output at the end block and the aggsender request. No proposer call, no
    /// witness, no state sketch: the pre-block sketch at the reorged anchor is
    /// the call that fails during the reorg being recovered from.
    pub(crate) async fn retrieve_noop_data(
        contracts_client: Arc<ContractsClient>,
        request: AggchainProofBuilderRequest,
        network_id: u32,
        l1_pre_root: Digest,
    ) -> Result<AggchainProverInputs, Error>
    where
        ContractsClient: L2LocalExitRootFetcher
            + L2OutputAtBlockFetcher
            + GetTrustedSequencerAddress
            + L1OpSuccinctConfigFetcher,
    {
        let last_proven_block = request.aggchain_proof_inputs.last_proven_block;
        let end_block = request.end_block;
        info!(%last_proven_block, %end_block, %l1_pre_root,
            "Retrieving chain data for the noop aggchain proof");

        let new_blocks_range = (last_proven_block + 1)..=end_block;

        let prev_local_exit_root = contracts_client
            .get_l2_local_exit_root(last_proven_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let new_local_exit_root = contracts_client
            .get_l2_local_exit_root(end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let claim_output = contracts_client
            .get_l2_output_at_block(end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        // Taken from L1 as-is: these are the values the contract hashes.
        let op_succinct_config = contracts_client
            .get_op_succinct_config()
            .await
            .map_err(Error::L1ChainDataRetrievalError)?;

        let trusted_sequencer = contracts_client
            .get_trusted_sequencer_address()
            .await
            .map_err(Error::UnableToFetchTrustedSequencerAddress)?;

        let imported_bridge_exits =
            collect_imported_bridge_exits(&request.aggchain_proof_inputs, &new_blocks_range)?;

        let aggchain_params = NoopAggchainParams {
            l2_pre_root: l1_pre_root,
            claim_root: claim_output.output_root,
            claim_block_num: end_block,
            rollup_config_hash: op_succinct_config.rollup_config_hash,
            optimistic_mode: request.fep_verification.is_optimistic(),
            trusted_sequencer,
            range_vkey_commitment: op_succinct_config.range_vkey_commitment,
            aggregation_vkey_hash: op_succinct_config.aggregation_vkey_hash,
        };

        let public_values = AggchainProofPublicValues {
            prev_local_exit_root,
            new_local_exit_root,
            l1_info_root: request.aggchain_proof_inputs.l1_info_tree_root_hash,
            origin_network: network_id.into(),
            commit_imported_bridge_exits: imported_bridge_exits.commitment,
            aggchain_params: aggchain_params.hash(),
        };

        info!(
            "Noop aggchain-params unrolled values: {aggchain_params:?}; keccak-hashed: {}",
            public_values.aggchain_params
        );

        let stdin = sp1_fast(|| {
            let mut stdin = SP1Stdin::new();
            stdin.write(&public_values);
            stdin
        })
        .context("Failed to build SP1 stdin")
        .map_err(Error::Other)?;

        Ok(AggchainProverInputs {
            output_root: ClaimRoot(claim_output.output_root),
            stdin,
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
        range_vkey_commitment: Digest,
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

        // Validate that the OpSuccinct config keys match expected values
        validate_op_succinct_config_keys(
            &op_succinct_config,
            &aggregation_vkey,
            &range_vkey_commitment,
        )?;

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

        // Retrieve all the raw GERs from the aggsender input.
        // Removed GERs from this list have invalid merkle proofs.
        let raw_inserted_gers: Vec<InsertedGER> = request
            .aggchain_proof_inputs
            .sorted_inserted_gers(&new_blocks_range);

        let imported_bridge_exits =
            collect_imported_bridge_exits(&request.aggchain_proof_inputs, &new_blocks_range)?;

        // Prepare removed GERS for the proof.
        let removed_gers: Vec<Digest> = filter_sort_map(
            request.aggchain_proof_inputs.removed_gers,
            &new_blocks_range,
            |removed_ger| removed_ger.block_number,
            |removed_ger| removed_ger.global_exit_root,
        )
        .collect();

        // Prepare inserted GERS for the proof, filtering out the removed ones.
        let inserted_gers = filter_values(&removed_gers, &raw_inserted_gers, |value| {
            value.l1_info_tree_leaf.inner.global_exit_root
        })?;

        // Prepare the hash chain of all the GERs (inserted and removed) for the
        // proof.
        let raw_inserted_gers = raw_inserted_gers
            .into_iter()
            .map(|inserted_ger| inserted_ger.l1_info_tree_leaf.inner.global_exit_root)
            .collect();

        let l1_info_tree_leaf = request.aggchain_proof_inputs.l1_info_tree_leaf;
        let fep_inputs = FepInputs {
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
            signature_optimistic_mode: request.fep_verification.optimistic_mode_signature(),
            l1_info_tree_leaf,
            l1_head_inclusion_proof: request.aggchain_proof_inputs.l1_info_tree_merkle_proof,
            aggregation_vkey_hash: KoalaBearDigest(aggregation_vkey.hash_u32()),
            range_vkey_commitment: range_vkey_commitment.0,
        };

        {
            if let FepVerification::Proof {
                ref aggregation_proof_public_values,
                ..
            } = request.fep_verification
            {
                let retrieved_from_contracts = AggregationProofPublicValues::from(&fep_inputs);

                if aggregation_proof_public_values != &retrieved_from_contracts {
                    error!(
                        "Mismatch between the aggregation proof public values - retrieved from \
                         the contracts: {retrieved_from_contracts:?}, received with the proof: \
                         {:?}",
                        aggregation_proof_public_values
                    );
                    return Err(Error::MismatchAggregationProofPublicValues {
                        expected_by_contract: Box::new(retrieved_from_contracts),
                        expected_by_verifier: Box::new(aggregation_proof_public_values.clone()),
                    });
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
                commit_imported_bridge_exits: imported_bridge_exits.commitment,
                bridge_witness: BridgeWitness {
                    inserted_gers,
                    imported_bridge_exits: imported_bridge_exits.all,
                    removed_gers,
                    raw_inserted_gers,
                    unset_claims: imported_bridge_exits.unset_claims,
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
                    let aggregation_proof = aggregation_proof
                        .proof
                        .clone()
                        .try_as_compressed()
                        .ok_or(Error::GeneratedProofIsNotCompressed)?;
                    stdin.write_proof(*aggregation_proof, aggregation_vkey.vk.clone());
                }
                Ok::<_, Error>(stdin)
            })
            .context("Failed to build SP1 stdin")
            .map_err(Error::Other)??;

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

/// Validates that the OpSuccinct config keys match the expected values.
/// This ensures that the same proposer aggregation program is being used.
fn validate_op_succinct_config_keys(
    op_succinct_config: &OpSuccinctConfig,
    aggregation_vkey: &SP1VerifyingKey,
    expected_range_vkey_commitment: &Digest,
) -> Result<(), Error> {
    // Check if retrieved op-succinct config aggregation vkey hash matches
    let expected_aggregation_vkey_hash = Digest(aggregation_vkey.bytes32_raw());
    if op_succinct_config.aggregation_vkey_hash != expected_aggregation_vkey_hash {
        error!(
            "Mismatch on the aggregation vkey hash - got from op succinct contract config: {}, \
             expected from elf config: {}",
            op_succinct_config.aggregation_vkey_hash, expected_aggregation_vkey_hash
        );
        return Err(Error::MismatchAggregationVkeyHash {
            got: op_succinct_config.aggregation_vkey_hash,
            expected: expected_aggregation_vkey_hash,
        });
    }

    // Check if retrieved op-succinct config range_vkey_commitment matches
    if op_succinct_config.range_vkey_commitment != *expected_range_vkey_commitment {
        error!(
            "Mismatch on the range vkey commitment - got from op succinct config: {}, expected: {}",
            op_succinct_config.range_vkey_commitment, expected_range_vkey_commitment
        );
        return Err(Error::MismatchRangeVkeyCommitment {
            got: op_succinct_config.range_vkey_commitment,
            expected: *expected_range_vkey_commitment,
        });
    }

    Ok(())
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
        let range_vkey_commitment = self.range_vkey_commitment;
        let program = self.program;

        // TODO: figure out a way to stop only this service upon an sp1 panic,
        // and not the entire system. For now, just ignore the panic,
        // even though some internal mutability inside sp1 might end up
        // unhappy.
        sp1_async(AssertUnwindSafe(async move {
            let last_proven_block = req.aggchain_proof_inputs.last_proven_block;
            let end_block = req.end_block;
            info!(%last_proven_block, %end_block, "Starting generation of the aggchain proof");
            // Retrieve all the necessary public inputs. Combine with
            // the data provided by the agg-sender in the request.
            let aggchain_prover_inputs = match program {
                AggchainProgram::Standard => {
                    if matches!(req.fep_verification, FepVerification::Noop) {
                        return Err(Error::NoopVerificationRequiresNoopProgram);
                    }
                    Self::retrieve_chain_data(
                        contracts_client,
                        req,
                        network_id,
                        aggregation_vkey,
                        static_call_caller_address,
                        range_vkey_commitment,
                    )
                    .await?
                }
                AggchainProgram::Noop => {
                    let l1_pre_root =
                        latest_l1_pre_root(contracts_client.as_ref(), last_proven_block).await?;
                    Self::retrieve_noop_data(contracts_client, req, network_id, l1_pre_root).await?
                }
            };

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

            let public_input: AggchainProofPublicValues = bincode::sp1_compatible()
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
