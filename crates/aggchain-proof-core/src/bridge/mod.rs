//! A program that verifies the bridge integrity
use std::collections::HashMap;
use std::hash::Hash;

use alloy_primitives::{address, Address, B256};
use alloy_sol_macro::sol;
use alloy_sol_types::SolCall;
use inserted_ger::InsertedGER;
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::io::EVMStateSketch;
use static_call::{HashChainType, StaticCallError, StaticCallStage, StaticCallWithContext};

use crate::{keccak::keccak256_combine, Digest};

pub(crate) mod inserted_ger;
mod static_call;

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
        "Mismatch on the hash chain {hash_chain_type:?}. computed: {computed}, input: {input}"
    )]
    MismatchHashChain {
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
    MismatchConstrainedGlobalIndices {
        computed: Vec<B256>,
        input: Vec<B256>,
    },

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
}

impl BridgeConstraintsError {
    fn static_call_error(
        source: StaticCallError,
        stage: StaticCallStage,
    ) -> BridgeConstraintsError {
        BridgeConstraintsError::StaticCallError { source, stage }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashChainSketches {
    prev_sketch: EVMStateSketch,
    new_sketch: EVMStateSketch,
}

/// Bridge data required to verify the BridgeConstraintsInput integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeWitness {
    /// List of inserted GER.
    pub inserted_gers: Vec<InsertedGER>,
    /// List of inserted GER.
    pub inserted_gers_hash_chain: Vec<Digest>,
    /// List of removed GER.
    pub removed_gers_hash_chain: Vec<Digest>,
    /// List of the global index of each imported bridge exit.
    pub global_indices_claimed: Vec<B256>,
    /// List of the global index of each unset bridge exit.
    pub global_indices_unset: Vec<B256>,
    /// State sketches to retrieve the inserted GER hash chain.
    pub inserted_ger_sketches: HashChainSketches,
    /// State sketches to retrieve the removed GER hash chain.
    pub removed_ger_sketches: HashChainSketches,
    /// State sketches to retrieve the claims hash chain.
    pub claimed_global_index_sketches: HashChainSketches,
    /// State sketches to retrieve the unset hash chain.
    pub unset_global_index_sketches: HashChainSketches,
    /// State sketch to retrieve the bridge address.
    pub bridge_address_sketch: EVMStateSketch,
    /// State sketch to retrieve the new LER.
    pub new_ler_sketch: EVMStateSketch,
}

/// Bridge data required to verify the bridge smart contract integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConstraintsInput {
    pub ger_addr: Address,
    pub prev_l2_block_hash: Digest,
    pub new_l2_block_hash: Digest,
    pub new_local_exit_root: Digest,
    pub l1_info_root: Digest,
    pub global_indices: Vec<B256>,
    pub bridge_witness: BridgeWitness,
}

impl BridgeConstraintsInput {
    fn fetch_hash_chains<C: SolCall + Clone>(
        &self,
        hash_chain_sketch: &HashChainSketches,
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
        .execute(&hash_chain_sketch.prev_sketch, calldata.clone())?;

        // Get the state of the hash chain of the new block on L2
        let new_hash_chain = StaticCallWithContext {
            address,
            stage: StaticCallStage::NewHashChain(hash_chain),
            block_hash: self.new_l2_block_hash,
        }
        .execute(&hash_chain_sketch.new_sketch, calldata)?;

        Ok((prev_hash_chain, new_hash_chain))
    }

    /// Verify the previous and new hash chains and their reconstructions.
    fn verify_ger_hash_chains(&self) -> Result<(), BridgeConstraintsError> {
        // Verify the hash chain on inserted GER
        {
            let hash_chain_type = HashChainType::InsertedGER;
            let (prev_hash_chain, new_hash_chain): (Digest, Digest) = self
                .fetch_hash_chains(
                    &self.bridge_witness.inserted_ger_sketches,
                    hash_chain_type,
                    self.ger_addr,
                    GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
                )
                .map(|(prev, new)| (prev.hashChain.0.into(), new.hashChain.0.into()))?;

            validate_hash_chain(
                self.bridge_witness.inserted_gers_hash_chain.iter().cloned(),
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
                    &self.bridge_witness.removed_ger_sketches,
                    hash_chain_type,
                    self.ger_addr,
                    GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
                )
                .map(|(prev, new)| (prev.hashChain.0.into(), new.hashChain.0.into()))?;

            validate_hash_chain(
                self.bridge_witness.removed_gers_hash_chain.iter().cloned(),
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
                    &self.bridge_witness.claimed_global_index_sketches,
                    hash_chain_type,
                    bridge_address,
                    BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
                )
                .map(|(prev, new)| (prev.hashChain.0.into(), new.hashChain.0.into()))?;

            validate_hash_chain(
                self.bridge_witness
                    .global_indices_claimed
                    .iter()
                    .map(|idx| idx.0.into()),
                prev_hash_chain,
                new_hash_chain,
                hash_chain_type,
            )?;
        }

        // Verify the hash chain on unset global index
        {
            let hash_chain_type = HashChainType::UnsetGlobalIndex;
            let (prev_hash_chain, new_hash_chain): (Digest, Digest) = self
                .fetch_hash_chains(
                    &self.bridge_witness.unset_global_index_sketches,
                    hash_chain_type,
                    bridge_address,
                    BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
                )
                .map(|(prev, new)| (prev.hashChain.0.into(), new.hashChain.0.into()))?;

            validate_hash_chain(
                self.bridge_witness
                    .global_indices_unset
                    .iter()
                    .map(|idx| idx.0.into()),
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
            &self.bridge_witness.new_ler_sketch,
            BridgeL2SovereignChain::getRootCall {},
        )?
        .lastRollupExitRoot
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
            &self.bridge_witness.bridge_address_sketch,
            GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
        )?
        .bridgeAddress;

        Ok(bridge_address)
    }

    /// Verify that the claimed global indexes minus the unset global indexes
    /// are equal to the Constrained global indexes.
    fn verify_constrained_global_indices(&self) -> Result<(), BridgeConstraintsError> {
        let filtered_claimed = filter_values(
            &self.bridge_witness.global_indices_unset,
            &self.bridge_witness.global_indices_claimed,
        );

        // Check if the filtered claimed global indices are equal to the constrained
        // global indices.
        if filtered_claimed != self.global_indices {
            return Err(BridgeConstraintsError::MismatchConstrainedGlobalIndices {
                computed: filtered_claimed,
                input: self.global_indices.clone(),
            });
        }

        Ok(())
    }

    /// Verify the inclusion proofs of the inserted GERs up to the L1InfoRoot.
    fn verify_inserted_gers(&self) -> Result<(), BridgeConstraintsError> {
        // Iterate over claimed indices and remove (skip) one occurrence for each value
        // in removal_map.
        let filtered_hash_chain_gers = filter_values(
            &self.bridge_witness.removed_gers_hash_chain,
            &self.bridge_witness.inserted_gers_hash_chain,
        );

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
                l1_info_leaf_index: wrong_ger.l1_info_tree_index,
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
}

/// Validate that the rebuilt hash chain is equal to the new hash chain.
fn validate_hash_chain(
    hashes: impl Iterator<Item = Digest>,
    prev_hash_chain: Digest,
    new_hash_chain: Digest,
    hash_chain_type: HashChainType,
) -> Result<(), BridgeConstraintsError> {
    let rebuilt_hash_chain =
        hashes.fold(prev_hash_chain, |acc, hash| keccak256_combine([acc, hash]));

    if rebuilt_hash_chain != new_hash_chain {
        return Err(BridgeConstraintsError::MismatchHashChain {
            computed: rebuilt_hash_chain,
            input: new_hash_chain,
            hash_chain_type,
        });
    }

    Ok(())
}

fn filter_values<I: Eq + Hash + Copy>(removed: &[I], values: &[I]) -> Vec<I> {
    // Create a map that counts how many removals are needed for each removed value
    let mut removal_map: HashMap<I, usize> = HashMap::new();
    removed.iter().for_each(|&value| {
        *removal_map.entry(value).or_insert(0) += 1;
    });

    // Iterate over values and remove (skip) one occurrence for each removed value
    values
        .iter()
        .filter(|value| {
            // If the value needs to be removed...
            if let Some(count) = removal_map.get_mut(value) {
                if *count > 0 {
                    *count -= 1; // Remove one occurrence.
                    return false; // Skip including this occurrence.
                }
            }
            true // Otherwise, keep the value.
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::BufReader;
    use std::str::FromStr;

    use alloy::providers::RootProvider;
    use alloy::rpc::types::BlockNumberOrTag;
    use alloy_primitives::hex;
    use alloy_sol_types::SolCall;
    use serde_json::Value;
    use sp1_cc_client_executor::ContractInput;
    use sp1_cc_host_executor::HostExecutor;
    use url::Url;

    use super::*;
    use crate::bridge::inserted_ger::L1InfoTreeLeaf;
    use crate::local_exit_tree::proof::LETMerkleProof;

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

        let claimed_global_indexes: Vec<B256> = json_data["claimedGlobalIndexes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                let s_str = s.as_str().unwrap();
                // Pad left with zeros to 64 hex characters if needed
                let padded = format!("{:0>64}", s_str);
                hex::decode(padded).unwrap().as_slice().try_into().unwrap()
            })
            .collect();

        let unclaimed_global_indexes: Vec<B256> = json_data["unclaimedGlobalIndexes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                let s_str = s.as_str().unwrap();
                let padded = format!("{:0>64}", s_str);
                hex::decode(padded).unwrap().as_slice().try_into().unwrap()
            })
            .collect();

        // Compute constrained global indices.
        let mut removal_map: HashMap<B256, usize> = HashMap::new();
        for idx in &unclaimed_global_indexes {
            *removal_map.entry(*idx).or_insert(0) += 1;
        }
        let constrained_global_indices: Vec<B256> = claimed_global_indexes
            .iter()
            .cloned()
            .filter(|v| {
                if let Some(count) = removal_map.get_mut(v) {
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
                        proof: LETMerkleProof {
                            siblings: ger["proof"]
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
                        },
                        l1_info_tree_leaf: L1InfoTreeLeaf {
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
                        l1_info_tree_index: index as u32,
                    })
                }
            })
            .collect();

        let inserted_gers: Vec<Digest> = global_exit_roots
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
                let digest = ger.l1_info_tree_leaf.global_exit_root;
                if let Some(count) = removal_map.get_mut(&digest) {
                    if *count > 0 {
                        *count -= 1;
                        return false;
                    }
                }
                true
            })
            .collect();

        let rpc_url_l2 = std::env::var(format!("RPC_{}", chain_id_l2))
            .expect("RPC URL must be defined")
            .parse::<Url>()
            .expect("Invalid URL format");

        let block_number_initial = BlockNumberOrTag::Number(initial_block_number);
        let block_number_final = BlockNumberOrTag::Number(final_block_number);

        // 1. Get the prev inserted GER hash chain (previous block on L2)
        println!("Step 1: Fetching previous inserted GER hash chain...");
        let provider_l2: RootProvider<alloy::network::AnyNetwork> =
            RootProvider::new_http(rpc_url_l2.clone());
        let mut executor_prev_hash_chain =
            HostExecutor::new(provider_l2.clone(), block_number_initial).await?;
        let _hash_chain = executor_prev_hash_chain
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
            ))
            .await?;
        println!(
            "Step 1: Received prev inserted GER hash chain: {:?}",
            _hash_chain
        );

        let _decoded_hash_chain =
            GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall::abi_decode_returns(
                &_hash_chain,
                true,
            )?
            .hashChain;
        let executor_prev_hash_chain_sketch = executor_prev_hash_chain.finalize().await?;

        // 2. Get the new inserted GER hash chain (new block on L2)
        println!("Step 2: Fetching new inserted GER hash chain...");
        let mut executor_new_hash_chain =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let _new_hash_chain = executor_new_hash_chain
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
            ))
            .await?;
        println!(
            "Step 2: Received new inserted GER hash chain: {:?}",
            _new_hash_chain
        );
        let executor_new_hash_chain = executor_new_hash_chain.finalize().await?;

        // 3. Get the bridge address.
        println!("Step 3: Fetching bridge address...");
        let mut executor_get_bridge_address =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let bridge_address_bytes = executor_get_bridge_address
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
            GlobalExitRootManagerL2SovereignChain::bridgeAddressCall::abi_decode_returns(
                &bridge_address_bytes,
                true,
            )?
            .bridgeAddress;
        let executor_get_bridge_address_sketch = executor_get_bridge_address.finalize().await?;

        // 4. Get the new local exit root from the bridge on the new L2 block.
        println!("Step 4: Fetching new local exit root from bridge...");
        let mut executor_get_ler =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let new_ler_bytes = executor_get_ler
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
            BridgeL2SovereignChain::getRootCall::abi_decode_returns(&new_ler_bytes, true)?
                .lastRollupExitRoot
                .0
                .into();
        let expected_new_ler: Digest = {
            let bytes = hex::decode(local_exit_root.trim_start_matches("0x")).unwrap();
            let arr: [u8; 32] = bytes.try_into().unwrap();
            arr.into()
        };
        assert_eq!(new_ler, expected_new_ler);
        let executor_get_ler_sketch = executor_get_ler.finalize().await?;

        // 5. Get the removed GER hash chain for the previous block.
        println!("Step 5: Fetching previous removed GER hash chain...");
        let mut executor_prev_removed =
            HostExecutor::new(provider_l2.clone(), block_number_initial).await?;
        let _prev_removed = executor_prev_removed
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
            ))
            .await?;
        println!(
            "Step 5: Received previous removed GER hash chain: {:?}",
            _prev_removed
        );
        let executor_prev_removed_sketch = executor_prev_removed.finalize().await?;

        // 6. Get the removed GER hash chain for the new block.
        println!("Step 6: Fetching new removed GER hash chain...");
        let mut executor_new_removed =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let _new_removed = executor_new_removed
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
            ))
            .await?;
        println!(
            "Step 6: Received new removed GER hash chain: {:?}",
            _new_removed
        );
        let executor_new_removed_sketch = executor_new_removed.finalize().await?;

        // 7. Get the claimed global index hash chain for the previous block.
        println!("Step 7: Fetching previous claimed global index hash chain...");
        let mut executor_prev_claimed =
            HostExecutor::new(provider_l2.clone(), block_number_initial).await?;
        let _prev_claimed = executor_prev_claimed
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
            ))
            .await?;
        println!(
            "Step 7: Received previous claimed global index hash chain: {:?}",
            _prev_claimed
        );
        let executor_prev_claimed_sketch = executor_prev_claimed.finalize().await?;

        // 8. Get the claimed global index hash chain for the new block.
        println!("Step 8: Fetching new claimed global index hash chain...");
        let mut executor_new_claimed =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let _new_claimed = executor_new_claimed
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
            ))
            .await?;
        println!(
            "Step 8: Received new claimed global index hash chain: {:?}",
            _new_claimed
        );
        let executor_new_claimed_sketch = executor_new_claimed.finalize().await?;

        // 9. Get the unset global index hash chain for the previous block.
        println!("Step 9: Fetching previous unset global index hash chain...");
        let mut executor_prev_unset =
            HostExecutor::new(provider_l2.clone(), block_number_initial).await?;
        let _prev_unset = executor_prev_unset
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
            ))
            .await?;
        println!(
            "Step 9: Received previous unset global index hash chain: {:?}",
            _prev_unset
        );
        let executor_prev_unset_sketch = executor_prev_unset.finalize().await?;

        // 10. Get the unset global index hash chain for the new block.
        println!("Step 10: Fetching new unset global index hash chain...");
        let mut executor_new_unset =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let _new_unset = executor_new_unset
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
            ))
            .await?;
        println!(
            "Step 10: Received new unset global index hash chain: {:?}",
            _new_unset
        );
        let executor_new_unset_sketch = executor_new_unset.finalize().await?;

        // Commit the bridge proof.
        let bridge_data_input = BridgeConstraintsInput {
            ger_addr: ger_address,
            prev_l2_block_hash: executor_prev_hash_chain_sketch.header.hash_slow().0.into(),
            new_l2_block_hash: executor_new_hash_chain.header.hash_slow().0.into(),
            new_local_exit_root: expected_new_ler,
            l1_info_root: {
                let bytes = hex::decode(l1_info_root.trim_start_matches("0x")).unwrap();
                let arr: [u8; 32] = bytes.try_into().unwrap();
                arr.into()
            },
            global_indices: constrained_global_indices, // Constrained indices
            bridge_witness: BridgeWitness {
                inserted_gers: final_imported_l1_info_tree_leafs,
                inserted_gers_hash_chain: inserted_gers,
                removed_gers_hash_chain: removed_gers,
                bridge_address_sketch: executor_get_bridge_address_sketch,
                new_ler_sketch: executor_get_ler_sketch,
                global_indices_claimed: claimed_global_indexes,
                global_indices_unset: unclaimed_global_indexes,
                inserted_ger_sketches: HashChainSketches {
                    prev_sketch: executor_prev_hash_chain_sketch,
                    new_sketch: executor_new_hash_chain,
                },
                claimed_global_index_sketches: HashChainSketches {
                    prev_sketch: executor_prev_claimed_sketch,
                    new_sketch: executor_new_claimed_sketch,
                },
                removed_ger_sketches: HashChainSketches {
                    prev_sketch: executor_prev_removed_sketch,
                    new_sketch: executor_new_removed_sketch,
                },
                unset_global_index_sketches: HashChainSketches {
                    prev_sketch: executor_prev_unset_sketch,
                    new_sketch: executor_new_unset_sketch,
                },
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

        assert!(bridge_data_input.verify().is_ok());

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
                    inserted_gers_hash_chain: bridge_data_input
                        .bridge_witness
                        .inserted_gers_hash_chain
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

        Ok(())
    }

    #[test]
    fn test_bridge_constraints_from_file() {
        // Read and parse the JSON file
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/test_input/bridge_constraints_input.json");
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let bridge_data_input: BridgeConstraintsInput = serde_json::from_reader(reader).unwrap();

        // Verify the bridge data input
        assert!(bridge_data_input.verify().is_ok());
    }
}
