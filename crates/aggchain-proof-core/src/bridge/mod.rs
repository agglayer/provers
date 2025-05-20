//! A program that verifies the bridge integrity
use std::{collections::HashMap, hash::Hash};

use agglayer_primitives::{keccak::keccak256_combine, Digest};
use alloy_primitives::{address, Address, U256};
use alloy_sol_macro::sol;
use alloy_sol_types::SolCall;
use inserted_ger::InsertedGER;
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::io::EVMStateSketch;
use static_call::{HashChainType, StaticCallError, StaticCallStage, StaticCallWithContext};
use unified_bridge::{GlobalIndexWithLeafHash, ImportedBridgeExitCommitmentValues};

use crate::proof::IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION;

pub mod inserted_ger;
pub mod static_call;

/// NOTE: Won't work with Outpost networks as this address won't be constant.
/// Address of the GlobalExitRootManagerL2SovereignChain smart contract.
pub(crate) const L2_GER_ADDR: Address = address!("a40d5f56745a118d0906a34e69aec8c0db1cb8fa");

// Contract interfaces of the pre-deployed contracts on sovereign chains
sol! (
    interface GlobalExitRootManagerL2SovereignChain {
        function insertedGERHashChain() public view returns (bytes32 hashChain);
        function removedGERHashChain() public view returns (bytes32 hashChain);
        function bridgeAddress() public view returns (address bridgeAddress);
    }
);

sol! (
    interface BridgeL2SovereignChain {
        function getRoot() public view returns (bytes32 lastRollupExitRoot);
        function claimedGlobalIndexHashChain() public view returns (bytes32 hashChain);
        function unsetGlobalIndexHashChain() public view returns (bytes32 hashChain);
    }
);

/// Represents all the bridge constraints errors.
#[derive(thiserror::Error, Debug)]
pub enum BridgeConstraintsError {
    /// The inclusion proof from the GER to the L1 info Root is invalid.
    #[error(
        "Invalid inclusion proof for inserted GER. l1_info_leaf_index: {l1_info_leaf_index}, \
         l1_info_root: {l1_info_root}, inserted_ger: {inserted_ger}"
    )]
    InvalidMerklePathGERToL1Root {
        inserted_ger: Digest,
        l1_info_leaf_index: u32,
        l1_info_root: Digest,
    },

    /// The block hash retrieved from the static call do not match.
    #[error("Mismatch on the block hash at {stage:?}. retrieved: {retrieved}, input: {input}")]
    MismatchBlockHash {
        retrieved: Digest,
        input: Digest,
        stage: StaticCallStage,
    },

    /// The provided hash chain does not correspond with the computed one.
    #[error(
        "Mismatch on the hash chain {hash_chain_type:?}. prev_hash_chain: {prev_hash_chain}, \
         new_hash_chain: (computed: {computed} != expected: {input})"
    )]
    MismatchHashChain {
        prev_hash_chain: Digest,
        computed: Digest,
        input: Digest,
        hash_chain_type: HashChainType,
    },

    /// The provided new LER does not correspond with the one retrieved from
    /// contracts.
    #[error("Mismatch on the new LER. retrieved: {retrieved}, input: {input}")]
    MismatchNewLocalExitRoot { retrieved: Digest, input: Digest },

    /// The provided constrained global indices do not correspond with the
    /// computed ones.
    #[error(
        "Mismatch on the constrained global indices. computed: {computed:?}, input: {input:?}"
    )]
    MismatchConstrainedBridgeExits { computed: Digest, input: Digest },

    /// The provided inserted gers do not correspond with the
    /// computed ones.
    #[error("Mismatch on the constrained inserted GERs. computed: {computed:?}, input: {input:?}")]
    MismatchConstrainedInsertedGers {
        computed: Vec<Digest>,
        input: Vec<Digest>,
    },

    /// The static call failed at the given stage.
    #[error("Failure upon static call at {stage:?}.")]
    StaticCallError {
        stage: StaticCallStage,
        #[source]
        source: StaticCallError,
    },

    /// The given hash chain overflowed.
    #[error("Overflow on the hashchain elements.")]
    HashChainOverflow,
}

impl BridgeConstraintsError {
    fn static_call_error(
        source: StaticCallError,
        stage: StaticCallStage,
    ) -> BridgeConstraintsError {
        BridgeConstraintsError::StaticCallError { source, stage }
    }
}

/// Bridge data required to verify the BridgeConstraintsInput integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeWitness {
    /// List of inserted GER minus the removed ones.
    pub inserted_gers: Vec<InsertedGER>,
    /// Raw list of inserted GERs which includes also the ones which get
    /// removed.
    pub raw_inserted_gers: Vec<Digest>,
    /// List of removed GER.
    pub removed_gers: Vec<Digest>,
    /// List of the each imported bridge exit containing the global index and
    /// the leaf hash.
    pub bridge_exits_claimed: Vec<GlobalIndexWithLeafHash>,
    /// List of the global index of each unset bridge exit.
    pub global_indices_unset: Vec<U256>,
    /// State sketch for the prev L2 block.
    pub prev_l2_block_sketch: EVMStateSketch,
    /// State sketch for the new L2 block.
    pub new_l2_block_sketch: EVMStateSketch,
}

/// Bridge data required to verify the bridge smart contract integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConstraintsInput {
    pub ger_addr: Address,
    pub prev_l2_block_hash: Digest,
    pub new_l2_block_hash: Digest,
    pub new_local_exit_root: Digest,
    pub l1_info_root: Digest,
    pub commit_imported_bridge_exits: Digest,
    pub bridge_witness: BridgeWitness,
}

impl BridgeConstraintsInput {
    fn fetch_hash_chains<C: SolCall + Clone>(
        &self,
        hash_chain: HashChainType,
        address: Address,
        calldata: C,
    ) -> Result<(C::Return, C::Return), BridgeConstraintsError> {
        // Get the state of the hash chain of the previous block on L2
        let prev_hash_chain = StaticCallWithContext {
            address,
            stage: StaticCallStage::PrevHashChain(hash_chain),
            block_hash: self.prev_l2_block_hash,
        }
        .execute(&self.bridge_witness.prev_l2_block_sketch, calldata.clone())?;

        // Get the state of the hash chain of the new block on L2
        let new_hash_chain = StaticCallWithContext {
            address,
            stage: StaticCallStage::NewHashChain(hash_chain),
            block_hash: self.new_l2_block_hash,
        }
        .execute(&self.bridge_witness.new_l2_block_sketch, calldata)?;

        Ok((prev_hash_chain, new_hash_chain))
    }

    /// Verify the previous and new hash chains and their reconstructions.
    fn verify_ger_hash_chains(&self) -> Result<(), BridgeConstraintsError> {
        // Verify the hash chain on inserted GER
        {
            let hash_chain_type = HashChainType::InsertedGER;
            let (prev_hash_chain, new_hash_chain): (Digest, Digest) = self
                .fetch_hash_chains(
                    hash_chain_type,
                    self.ger_addr,
                    GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
                )
                .map(|(prev, new)| (prev.0.into(), new.0.into()))?;

            self.validate_hash_chain(
                &self.bridge_witness.raw_inserted_gers,
                prev_hash_chain,
                new_hash_chain,
                hash_chain_type,
            )?;
        }

        // Verify the hash chain on removed GER
        {
            let hash_chain_type = HashChainType::RemovedGER;
            let (prev_hash_chain, new_hash_chain): (Digest, Digest) = self
                .fetch_hash_chains(
                    hash_chain_type,
                    self.ger_addr,
                    GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
                )
                .map(|(prev, new)| (prev.0.into(), new.0.into()))?;

            self.validate_hash_chain(
                &self.bridge_witness.removed_gers,
                prev_hash_chain,
                new_hash_chain,
                hash_chain_type,
            )?;
        }

        Ok(())
    }

    /// Verify the previous and new hash chains and their reconstructions.
    fn verify_claims_hash_chains(
        &self,
        bridge_address: Address,
    ) -> Result<(), BridgeConstraintsError> {
        // Verify the hash chain on claimed global index
        {
            let hash_chain_type = HashChainType::ClaimedGlobalIndex;
            let (prev_hash_chain, new_hash_chain): (Digest, Digest) = self
                .fetch_hash_chains(
                    hash_chain_type,
                    bridge_address,
                    BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
                )
                .map(|(prev, new)| (prev.0.into(), new.0.into()))?;

            let claims: Vec<Digest> = self
                .bridge_witness
                .bridge_exits_claimed
                .iter()
                .map(|&idx| idx.commitment())
                .collect();

            self.validate_hash_chain(&claims, prev_hash_chain, new_hash_chain, hash_chain_type)?;
        }

        // Verify the hash chain on unset global index
        {
            let hash_chain_type = HashChainType::UnsetGlobalIndex;
            let (prev_hash_chain, new_hash_chain): (Digest, Digest) = self
                .fetch_hash_chains(
                    hash_chain_type,
                    bridge_address,
                    BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
                )
                .map(|(prev, new)| (prev.0.into(), new.0.into()))?;

            let unset_claims: Vec<Digest> = self
                .bridge_witness
                .global_indices_unset
                .iter()
                .map(|idx| idx.to_be_bytes().into())
                .collect();

            self.validate_hash_chain(
                &unset_claims,
                prev_hash_chain,
                new_hash_chain,
                hash_chain_type,
            )?;
        }

        Ok(())
    }

    // Verify the new local exit root.
    fn verify_new_ler(&self, bridge_address: Address) -> Result<(), BridgeConstraintsError> {
        // Get the new local exit root
        let new_ler: Digest = StaticCallWithContext {
            address: bridge_address,
            stage: StaticCallStage::NewLer,
            block_hash: self.new_l2_block_hash,
        }
        .execute(
            &self.bridge_witness.new_l2_block_sketch,
            BridgeL2SovereignChain::getRootCall {},
        )?
        .0
        .into();

        // Check that the new local exit root returned from L2 matches the expected
        if new_ler != self.new_local_exit_root {
            return Err(BridgeConstraintsError::MismatchNewLocalExitRoot {
                retrieved: new_ler,
                input: self.new_local_exit_root,
            });
        }

        Ok(())
    }

    /// Fetch the bridge address through a static call.
    fn fetch_bridge_address(&self) -> Result<Address, BridgeConstraintsError> {
        // Get the bridge address from the GER smart contract.
        // Since the bridge address is not constant but the l2 ger address is
        // We can retrieve the bridge address saving some public inputs and possible
        // errors
        let bridge_address: Address = StaticCallWithContext {
            address: self.ger_addr,
            stage: StaticCallStage::BridgeAddress,
            block_hash: self.new_l2_block_hash,
        }
        .execute(
            &self.bridge_witness.new_l2_block_sketch,
            GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
        )?;

        Ok(bridge_address)
    }

    /// Verify that the claimed global indexes minus the unset global indexes
    /// are equal to the Constrained global indexes.
    fn verify_constrained_global_indices(&self) -> Result<(), BridgeConstraintsError> {
        // Check if the filtered claimed global indices are equal to the constrained
        // global indices.
        let constrained_claims = ImportedBridgeExitCommitmentValues {
            claims: filter_values(
                &self.bridge_witness.global_indices_unset,
                &self.bridge_witness.bridge_exits_claimed,
                |exit: &GlobalIndexWithLeafHash| -> U256 { exit.global_index },
            )?,
        };

        let computed_commitment =
            constrained_claims.commitment(IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION);

        if computed_commitment != self.commit_imported_bridge_exits {
            return Err(BridgeConstraintsError::MismatchConstrainedBridgeExits {
                computed: computed_commitment,
                input: self.commit_imported_bridge_exits,
            });
        }

        Ok(())
    }

    /// Verify the inclusion proofs of the inserted GERs up to the L1InfoRoot.
    fn verify_inserted_gers(&self) -> Result<(), BridgeConstraintsError> {
        // Iterate over claimed indices and remove (skip) one occurrence for each value
        // in removal_map.
        let filtered_hash_chain_gers = filter_values(
            &self.bridge_witness.removed_gers,
            &self.bridge_witness.raw_inserted_gers,
            |&x| x,
        )?;

        // Check that the filtered_hash_chain_gers are equal to the inserted
        let inserted_gers_compare: Vec<Digest> = self
            .bridge_witness
            .inserted_gers
            .iter()
            .map(|inserted_ger| inserted_ger.ger())
            .collect();

        if filtered_hash_chain_gers != inserted_gers_compare {
            return Err(BridgeConstraintsError::MismatchConstrainedInsertedGers {
                computed: filtered_hash_chain_gers,
                input: inserted_gers_compare,
            });
        }

        // Check that the inserted gers are correctly inserted in the L1InfoRoot.
        let maybe_wrong_inserted_ger = self
            .bridge_witness
            .inserted_gers
            .iter()
            .find(|ger| !ger.verify(self.l1_info_root));

        if let Some(wrong_ger) = maybe_wrong_inserted_ger {
            return Err(BridgeConstraintsError::InvalidMerklePathGERToL1Root {
                inserted_ger: wrong_ger.ger(),
                l1_info_leaf_index: wrong_ger.l1_info_tree_leaf.l1_info_tree_index,
                l1_info_root: self.l1_info_root,
            });
        }

        Ok(())
    }

    /// Verify the bridge state.
    pub fn verify(&self) -> Result<(), BridgeConstraintsError> {
        self.verify_ger_hash_chains()?;
        let bridge_address = self.fetch_bridge_address()?;
        self.verify_claims_hash_chains(bridge_address)?;
        self.verify_new_ler(bridge_address)?;
        self.verify_constrained_global_indices()?;
        self.verify_inserted_gers()
    }

    /// Validate that the rebuilt hash chain is equal to the new hash chain.
    fn validate_hash_chain(
        &self,
        hashes: &[Digest],
        prev_hash_chain: Digest,
        new_hash_chain: Digest,
        hash_chain_type: HashChainType,
    ) -> Result<(), BridgeConstraintsError> {
        let rebuilt_hash_chain = hashes
            .iter()
            .fold(prev_hash_chain, |acc, &hash| keccak256_combine([acc, hash]));

        if rebuilt_hash_chain != new_hash_chain {
            eprintln!(
                "block_hash. prev: {:?}, new: {:?}",
                self.prev_l2_block_hash, self.new_l2_block_hash
            );
            hashes.iter().enumerate().for_each(|(idx, hash)| {
                eprintln!("element #{idx} ({hash_chain_type:?}): {hash:?}")
            });

            return Err(BridgeConstraintsError::MismatchHashChain {
                prev_hash_chain,
                computed: rebuilt_hash_chain,
                input: new_hash_chain,
                hash_chain_type,
            });
        }

        Ok(())
    }
}

fn filter_values<K: Eq + Hash + Copy, V: Copy>(
    removed: &[K],
    values: &[V],
    mut key_fn: impl FnMut(&V) -> K,
) -> Result<Vec<V>, BridgeConstraintsError> {
    // Create a map that counts how many removals are needed for each removed value
    let mut removal_map: HashMap<K, usize> = HashMap::new();
    for &value in removed.iter() {
        let count = removal_map.entry(value).or_insert(0);
        *count = count
            .checked_add(1)
            .ok_or(BridgeConstraintsError::HashChainOverflow)?;
    }

    // Iterate over values and remove (skip) one occurrence for each removed value
    let result = values
        .iter()
        .filter(|value| {
            // If the value needs to be removed...
            if let Some(count) = removal_map.get_mut(&key_fn(value)) {
                if *count > 0 {
                    *count -= 1; // Remove one occurrence.
                    return false; // Skip including this occurrence.
                }
            }
            true // Otherwise, keep the value.
        })
        .copied()
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::File, io::BufReader, str::FromStr};

    use alloy::{providers::RootProvider, rpc::types::BlockNumberOrTag};
    use alloy_primitives::hex;
    use alloy_sol_types::SolCall;
    use serde_json::Value;
    use sp1_cc_client_executor::ContractInput;
    use sp1_cc_host_executor::HostExecutor;
    use unified_bridge::{L1InfoTreeLeaf, L1InfoTreeLeafInner, MerkleProof};
    use url::Url;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "e2e test, sepolia provider needed"]
    async fn test_bridge_contraints() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the environment variables.
        dotenvy::dotenv().ok();

        println!("Starting bridge constraints test...");
        // Read and parse the JSON file
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/test_input/bridge_input_e2e_sepolia.json");
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_data: Value = serde_json::from_reader(reader)?;

        // Extract values from JSON
        let initial_block_number = json_data["initialBlockNumber"].as_u64().unwrap();
        let final_block_number = json_data["finalBlockNumber"].as_u64().unwrap();
        let ger_address =
            Address::from_str(json_data["gerSovereignAddress"].as_str().unwrap()).unwrap();
        let global_exit_roots = &json_data["globalExitRoots"];
        let local_exit_root = json_data["localExitRoot"].as_str().unwrap();
        let l1_info_root = json_data["l1InfoRoot"].as_str().unwrap();
        let chain_id_l2: u64 = json_data["chainId"].as_u64().unwrap();
        let claimed_leafs: Vec<Digest> = json_data["claimedLeafs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                let s_str = s.as_str().unwrap();
                let bytes = hex::decode(s_str.trim_start_matches("0x")).unwrap();
                bytes.try_into().unwrap()
            })
            .collect();

        let l1_info_root: Digest = {
            let bytes = hex::decode(l1_info_root.trim_start_matches("0x")).unwrap();
            let arr: [u8; 32] = bytes.try_into().unwrap();
            arr.into()
        };

        // New extractions for the additional fields:
        let removed_gers: Vec<Digest> = json_data["removedGERs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                let s_str = s.as_str().unwrap();
                let bytes = hex::decode(s_str.trim_start_matches("0x")).unwrap();
                bytes.try_into().unwrap()
            })
            .collect();

        let claimed_global_indexes: Vec<U256> = json_data["claimedGlobalIndexes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                let s_str = s.as_str().unwrap();
                // Pad left with zeros to 64 hex characters if needed
                let padded = format!("{:0>64}", s_str);
                U256::from_be_slice(hex::decode(padded).unwrap().as_slice())
            })
            .collect();

        let unclaimed_global_indexes: Vec<U256> = json_data["unclaimedGlobalIndexes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                let s_str = s.as_str().unwrap();
                let padded = format!("{:0>64}", s_str);
                U256::from_be_slice(hex::decode(padded).unwrap().as_slice())
            })
            .collect();

        let imported_l1_info_tree_leafs: Vec<InsertedGER> = global_exit_roots
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .filter_map(|(index, ger)| {
                if ger.get("proof").is_none() {
                    None
                } else {
                    Some(InsertedGER {
                        block_index: 0u64,  // dataset already ordered
                        block_number: 0u64, // dataset already ordered
                        proof: MerkleProof::new(
                            l1_info_root,
                            ger["proof"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|p| {
                                    hex::decode(p.as_str().unwrap().trim_start_matches("0x"))
                                        .unwrap()
                                        .try_into()
                                        .unwrap()
                                })
                                .collect::<Vec<_>>()
                                .try_into()
                                .expect("Expected 32 siblings in proof"),
                        ),
                        l1_info_tree_leaf: L1InfoTreeLeaf {
                            rer: hex::decode(ger["rer"].as_str().unwrap().trim_start_matches("0x"))
                                .unwrap()
                                .try_into()
                                .unwrap(), // TODO: remove from API
                            mer: hex::decode(ger["mer"].as_str().unwrap().trim_start_matches("0x"))
                                .unwrap()
                                .try_into()
                                .unwrap(), // TODO: remove from API
                            l1_info_tree_index: index as u32,
                            inner: L1InfoTreeLeafInner {
                                global_exit_root: hex::decode(
                                    ger["globalExitRoot"]
                                        .as_str()
                                        .unwrap()
                                        .trim_start_matches("0x"),
                                )
                                .unwrap()
                                .try_into()
                                .unwrap(),
                                block_hash: {
                                    let bytes = hex::decode(
                                        ger["blockHash"].as_str().unwrap().trim_start_matches("0x"),
                                    )
                                    .unwrap();
                                    let array: [u8; 32] =
                                        bytes.try_into().expect("Incorrect length for block hash");
                                    array.into()
                                },
                                timestamp: ger["timestamp"].as_u64().unwrap(),
                            },
                        },
                    })
                }
            })
            .collect();

        let raw_inserted_gers: Vec<Digest> = global_exit_roots
            .as_array()
            .unwrap()
            .iter()
            .map(|ger| {
                hex::decode(
                    ger["globalExitRoot"]
                        .as_str()
                        .unwrap()
                        .trim_start_matches("0x"),
                )
                .unwrap()
                .try_into()
                .unwrap()
            })
            .collect();

        // remove the removed gers from the inserted GERS
        let mut removal_map: HashMap<Digest, usize> = HashMap::new();
        for value in &removed_gers {
            *removal_map.entry(*value).or_insert(0) += 1;
        }
        let final_imported_l1_info_tree_leafs: Vec<InsertedGER> = imported_l1_info_tree_leafs
            .into_iter()
            .filter(|ger| {
                let digest = ger.l1_info_tree_leaf.inner.global_exit_root;
                if let Some(count) = removal_map.get_mut(&digest) {
                    if *count > 0 {
                        *count -= 1;
                        return false;
                    }
                }
                true
            })
            .collect();

        // Instantiate the HostExecutor for the prev and new L2 blocks
        let (prev_l2_block_executor, new_l2_block_executor) = {
            let rpc_url_l2 = std::env::var(format!("RPC_{}", chain_id_l2))
                .expect("RPC URL must be defined")
                .parse::<Url>()
                .expect("Invalid URL format");

            let provider_l2: RootProvider<alloy::network::AnyNetwork> =
                RootProvider::new_http(rpc_url_l2.clone());

            let prev = HostExecutor::new(
                provider_l2.clone(),
                BlockNumberOrTag::Number(initial_block_number),
            )
            .await?;

            let new = HostExecutor::new(
                provider_l2.clone(),
                BlockNumberOrTag::Number(final_block_number),
            )
            .await?;

            (prev, new)
        };

        // 1. Get the prev inserted GER hash chain (previous block on L2)
        println!("Step 1: Fetching previous inserted GER hash chain...");
        let hash_chain = prev_l2_block_executor
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
            ))
            .await?;
        println!(
            "Step 1: Received prev inserted GER hash chain: {:?}",
            hash_chain
        );

        // 2. Get the new inserted GER hash chain (new block on L2)
        println!("Step 2: Fetching new inserted GER hash chain...");
        let new_hash_chain = new_l2_block_executor
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
            ))
            .await?;
        println!(
            "Step 2: Received new inserted GER hash chain: {:?}",
            new_hash_chain
        );

        // 3. Get the bridge address.
        println!("Step 3: Fetching bridge address...");
        let bridge_address_bytes = new_l2_block_executor
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
            ))
            .await?;
        println!(
            "Step 3: Received bridge address bytes: {:?}",
            bridge_address_bytes
        );
        let bridge_address =
            GlobalExitRootManagerL2SovereignChain::bridgeAddressCall::abi_decode_returns_validate(
                &bridge_address_bytes,
            )?;

        // 4. Get the new local exit root from the bridge on the new L2 block.
        println!("Step 4: Fetching new local exit root from bridge...");
        let new_ler_bytes = new_l2_block_executor
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::getRootCall {},
            ))
            .await?;
        println!(
            "Step 4: Received new local exit root bytes: {:?}",
            new_ler_bytes
        );
        let new_ler: Digest =
            BridgeL2SovereignChain::getRootCall::abi_decode_returns_validate(&new_ler_bytes)?
                .0
                .into();
        let expected_new_ler: Digest = {
            let bytes = hex::decode(local_exit_root.trim_start_matches("0x")).unwrap();
            let arr: [u8; 32] = bytes.try_into().unwrap();
            arr.into()
        };
        assert_eq!(new_ler, expected_new_ler);

        // 5. Get the removed GER hash chain for the previous block.
        println!("Step 5: Fetching previous removed GER hash chain...");
        let prev_removed = prev_l2_block_executor
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
            ))
            .await?;
        println!(
            "Step 5: Received previous removed GER hash chain: {:?}",
            prev_removed
        );

        // 6. Get the removed GER hash chain for the new block.
        println!("Step 6: Fetching new removed GER hash chain...");
        let new_removed = new_l2_block_executor
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
            ))
            .await?;
        println!(
            "Step 6: Received new removed GER hash chain: {:?}",
            new_removed
        );

        // 7. Get the claimed global index hash chain for the previous block.
        println!("Step 7: Fetching previous claimed global index hash chain...");
        let prev_claimed = prev_l2_block_executor
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
            ))
            .await?;
        println!(
            "Step 7: Received previous claimed global index hash chain: {:?}",
            prev_claimed
        );

        // 8. Get the claimed global index hash chain for the new block.
        println!("Step 8: Fetching new claimed global index hash chain...");
        let new_claimed = new_l2_block_executor
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
            ))
            .await?;
        println!(
            "Step 8: Received new claimed global index hash chain: {:?}",
            new_claimed
        );

        // 9. Get the unset global index hash chain for the previous block.
        println!("Step 9: Fetching previous unset global index hash chain...");
        let prev_unset = prev_l2_block_executor
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
            ))
            .await?;
        println!(
            "Step 9: Received previous unset global index hash chain: {:?}",
            prev_unset
        );

        // 10. Get the unset global index hash chain for the new block.
        println!("Step 10: Fetching new unset global index hash chain...");
        let new_unset = new_l2_block_executor
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
            ))
            .await?;
        println!(
            "Step 10: Received new unset global index hash chain: {:?}",
            new_unset
        );

        let bridge_exits_claimed: Vec<GlobalIndexWithLeafHash> = claimed_global_indexes
            .iter()
            .zip(claimed_leafs.iter())
            .map(|(&global_index, &leaf)| GlobalIndexWithLeafHash {
                global_index,
                bridge_exit_hash: leaf,
            })
            .collect();

        let mut removal_map: HashMap<U256, usize> = HashMap::new();
        for idx in &unclaimed_global_indexes {
            *removal_map.entry(*idx).or_insert(0) += 1;
        }
        let bridge_exits_claimed_filtered: Vec<GlobalIndexWithLeafHash> = bridge_exits_claimed
            .clone()
            .into_iter()
            .filter(|v| {
                if let Some(count) = removal_map.get_mut(&v.global_index) {
                    if *count > 0 {
                        *count -= 1;
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .collect();

        // Finalize the sketches
        let prev_l2_block_sketch = prev_l2_block_executor.finalize().await?;
        let new_l2_block_sketch = new_l2_block_executor.finalize().await?;

        // Commit the bridge proof.
        let bridge_data_input = BridgeConstraintsInput {
            ger_addr: ger_address,
            prev_l2_block_hash: prev_l2_block_sketch.header.hash_slow().0.into(),
            new_l2_block_hash: new_l2_block_sketch.header.hash_slow().0.into(),
            new_local_exit_root: expected_new_ler,
            l1_info_root,
            commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues {
                claims: bridge_exits_claimed_filtered,
            }
            .commitment(IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION),
            bridge_witness: BridgeWitness {
                inserted_gers: final_imported_l1_info_tree_leafs,
                raw_inserted_gers,
                removed_gers,
                bridge_exits_claimed,
                global_indices_unset: unclaimed_global_indexes,
                prev_l2_block_sketch,
                new_l2_block_sketch,
            },
        };

        println!("Bridge constraints test completed successfully.");

        // Serialize bridge_data_input into JSON and write it to a file.
        let file_path = std::path::Path::new("src/test_input/bridge_constraints_input.json");
        let file = std::fs::File::create(file_path)
            .expect("Failed to create the bridge constraints input file");
        serde_json::to_writer_pretty(file, &bridge_data_input)
            .expect("Failed to write bridge_data_input to file");

        println!("Bridge constraints input file created at: {:?}", file_path);

        assert_bridge_data(bridge_data_input);

        Ok(())
    }

    fn assert_bridge_data(bridge_data_input: BridgeConstraintsInput) {
        bridge_data_input.verify().unwrap();

        // Invalid l1 info root
        {
            let bridge_data_invalid = BridgeConstraintsInput {
                l1_info_root: Digest([0u8; 32]),
                ..bridge_data_input.clone()
            };

            assert!(matches!(
                bridge_data_invalid.verify(),
                Err(BridgeConstraintsError::InvalidMerklePathGERToL1Root { .. })
            ));
        }

        // Invalid hash chain
        {
            let bridge_data_invalid = BridgeConstraintsInput {
                bridge_witness: BridgeWitness {
                    raw_inserted_gers: bridge_data_input
                        .bridge_witness
                        .raw_inserted_gers
                        .iter()
                        .take(1)
                        .cloned()
                        .collect::<Vec<_>>(),
                    ..bridge_data_input.bridge_witness
                },
                ..bridge_data_input
            };

            assert!(matches!(
                bridge_data_invalid.verify(),
                Err(BridgeConstraintsError::MismatchHashChain { .. })
            ));
        }
    }

    #[test]
    fn test_bridge_constraints_from_file() {
        // Read and parse the JSON file
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/test_input/bridge_constraints_input.json");
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let bridge_data_input: BridgeConstraintsInput = serde_json::from_reader(reader).unwrap();
        // If the alloy version changes, this can lead to the file no longer parsing correctly,
        // and thus this test failing.
        // In that case, you should update the file.
        // The process is to:
        // 1. Obtain a Sepolia RPC key, and run `export RPC_11155111=https://eth-sepolia.g.alchemy.com/v2/[censored]`
        // 2. Run `cargo test --workspace -- bridge::tests::test_bridge_contraints --exact --show-output --include-ignored`
        //    (Or you can limit to `--package aggchain-proof-core --lib` if your cargo folder is not filled yet)
        // 3. The file should then be ready for committing

        assert_bridge_data(bridge_data_input);
    }
}
