//! A program that verifies the bridge integrity
use alloy_primitives::{address, Address, B256};
use alloy_sol_macro::sol;
use inserted_ger::InsertedGER;
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::io::EVMStateSketch;
use static_call::{execute_static_call, StaticCallError, StaticCallStage};
use std::collections::HashMap;

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

    /// The provided hash chain on inserted GER does not correspond with the computed
    /// one.
    #[error("Mismatch on the hash chain of inserted GERs. computed: {computed}, input: {input}")]
    MismatchInsertedGERHashChain { computed: Digest, input: Digest },

    /// The provided hash chain on inserted GER does not correspond with the computed
    /// one.
    #[error("Mismatch on the hash chain of removed GERs. computed: {computed}, input: {input}")]
    MismatchRemovedGERHashChain { computed: Digest, input: Digest },

    /// The provided hash chain on claimed global indices does not correspond with the
    /// computed one.
    #[error("Mismatch on the hash chain of claimed global indices. computed: {computed}, input: {input}")]
    MismatchClaimedGlobalIndexHashChain { computed: Digest, input: Digest },

    /// The provided hash chain on unset global indices does not correspond with the
    /// computed one.
    #[error(
        "Mismatch on the hash chain of unset global indices. computed: {computed}, input: {input}"
    )]
    MismatchUnsetGlobalIndexHashChain { computed: Digest, input: Digest },

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
    #[error(
        "Mismatch on the constrained global indices. computed: {computed:?}, input: {input:?}"
    )]
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
    /// State sketch to retrieve the previous inserted GER hash chain.
    pub prev_inserted_ger_hash_chain_sketch: EVMStateSketch,
    /// State sketch to retrieve the new inserted GER hash chain.
    pub new_inserted_ger_hash_chain_sketch: EVMStateSketch,
    /// State sketch to retrieve the previous removed GER hash chain.
    pub prev_removed_ger_hash_chain_sketch: EVMStateSketch,
    /// State sketch to retrieve the new removed GER hash chain.
    pub new_removed_ger_hash_chain_sketch: EVMStateSketch,
    /// State sketch to retrieve the previous claimed hash chain.
    pub prev_claimed_hash_chain_global_index_sketch: EVMStateSketch,
    /// State sketch to retrieve the new claimed hash chain.
    pub new_claimed_hash_chain_global_index_sketch: EVMStateSketch,
    /// State sketch to retrieve the previous unset hash chain.
    pub prev_unset_hash_chain_global_index_sketch: EVMStateSketch,
    /// State sketch to retrieve the new unset hash chain.
    pub new_unset_hash_chain_global_index_sketch: EVMStateSketch,
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
    /// Verify the previous and new inserted GER hash chai and its reconstruction.
    fn verify_inserted_ger_hash_chain(&self) -> Result<(), BridgeConstraintsError> {
        // 1.1 Get the state of the inserted GER hash chain of the previous block on L2
        let prev_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self.bridge_witness.prev_inserted_ger_hash_chain_sketch,
                self.ger_addr,
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
            )
            .map_err(|e| {
                BridgeConstraintsError::static_call_error(
                    e,
                    StaticCallStage::PrevInsertedGERHashChain,
                )
            })?;

            // check on block hash
            if retrieved_block_hash != self.prev_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.prev_l2_block_hash,
                    stage: StaticCallStage::PrevInsertedGERHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 1.2 Get the state of the inserted GER hash chain of the new block on L2
        let new_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self.bridge_witness.new_inserted_ger_hash_chain_sketch,
                self.ger_addr,
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
            )
            .map_err(|e| {
                BridgeConstraintsError::static_call_error(
                    e,
                    StaticCallStage::NewInsertedGERHashChain,
                )
            })?;

            // check on block hash
            if retrieved_block_hash != self.new_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.new_l2_block_hash,
                    stage: StaticCallStage::NewInsertedGERHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 1.3 Check that the rebuilt hash chain is equal to the new hash chain
        let rebuilt_hash_chain = self
            .bridge_witness
            .inserted_gers_hash_chain
            .iter()
            .fold(prev_hash_chain, |acc, ger| keccak256_combine([acc, *ger]));

        if rebuilt_hash_chain != new_hash_chain {
            return Err(BridgeConstraintsError::MismatchInsertedGERHashChain {
                computed: rebuilt_hash_chain,
                input: new_hash_chain,
            });
        }

        Ok(())
    }

    /// Verify the previous and new removed GER hash chain and its reconstruction.
    fn verify_removed_ger_hash_chain(&self) -> Result<(), BridgeConstraintsError> {
        // 1.1 Get the state of the removed GER hash chain of the previous block on L2
        let prev_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self.bridge_witness.prev_removed_ger_hash_chain_sketch,
                self.ger_addr,
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
            )
            .map_err(|e| {
                BridgeConstraintsError::static_call_error(
                    e,
                    StaticCallStage::PrevRemovedGERHashChain,
                )
            })?;

            // check on block hash
            if retrieved_block_hash != self.prev_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.prev_l2_block_hash,
                    stage: StaticCallStage::PrevRemovedGERHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 1.2 Get the state of the removed GER hash chain of the new block on L2
        let new_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self.bridge_witness.new_removed_ger_hash_chain_sketch,
                self.ger_addr,
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
            )
            .map_err(|e| {
                BridgeConstraintsError::static_call_error(
                    e,
                    StaticCallStage::NewRemovedGERHashChain,
                )
            })?;

            // check on block hash
            if retrieved_block_hash != self.new_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.new_l2_block_hash,
                    stage: StaticCallStage::NewRemovedGERHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 1.3 Check that the rebuilt hash chain is equal to the new hash chain
        let rebuilt_hash_chain: Digest = self
            .bridge_witness
            .removed_gers_hash_chain
            .iter()
            .fold(prev_hash_chain, |acc, ger| keccak256_combine([acc, *ger]));

        if rebuilt_hash_chain != new_hash_chain {
            return Err(BridgeConstraintsError::MismatchRemovedGERHashChain {
                computed: rebuilt_hash_chain,
                input: new_hash_chain,
            });
        }

        Ok(())
    }

    /// Fetch the bridge address through a static call.
    fn fetch_bridge_address(&self) -> Result<Address, BridgeConstraintsError> {
        let (decoded_return, retrieved_block_hash) = execute_static_call(
            &self.bridge_witness.bridge_address_sketch,
            self.ger_addr,
            GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
        )
        .map_err(|e| {
            BridgeConstraintsError::static_call_error(e, StaticCallStage::BridgeAddress)
        })?;

        // Check on block hash.
        if retrieved_block_hash != self.new_l2_block_hash {
            return Err(BridgeConstraintsError::MismatchBlockHash {
                retrieved: retrieved_block_hash,
                input: self.new_l2_block_hash,
                stage: StaticCallStage::BridgeAddress,
            });
        }

        Ok(decoded_return.bridgeAddress)
    }

    /// Verify the previous and new claimed global index hash chain and its
    /// reconstruction.
    #[allow(unused)]
    fn verify_claimed_global_index_hash_chain(
        &self,
        bridge_address: Address,
    ) -> Result<(), BridgeConstraintsError> {
        // 1.1 Get the state of the claimed global index hash chain of the previous block on L2
        let prev_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self
                    .bridge_witness
                    .prev_claimed_hash_chain_global_index_sketch,
                bridge_address,
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
            )
            .map_err(|e| {
                BridgeConstraintsError::static_call_error(e, StaticCallStage::PrevClaimedHashChain)
            })?;

            // check on block hash
            if retrieved_block_hash != self.prev_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.prev_l2_block_hash,
                    stage: StaticCallStage::PrevClaimedHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 1.2 Get the state of the unset global index hash chain of the new block on L2
        let new_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self
                    .bridge_witness
                    .new_claimed_hash_chain_global_index_sketch,
                self.ger_addr,
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
            )
            .map_err(|e| {
                BridgeConstraintsError::static_call_error(e, StaticCallStage::NewClaimedHashChain)
            })?;

            // check on block hash
            if retrieved_block_hash != self.new_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.prev_l2_block_hash,
                    stage: StaticCallStage::NewClaimedHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 1.3 Check that the rebuilt hash chain is equal to the new hash chain
        let rebuilt_hash_chain_global_index = self
            .bridge_witness
            .global_indices_claimed
            .iter()
            .fold(prev_hash_chain, |acc, &global_index_hashed| {
                keccak256_combine([acc, global_index_hashed.0.into()])
            });

        if rebuilt_hash_chain_global_index != new_hash_chain {
            return Err(
                BridgeConstraintsError::MismatchClaimedGlobalIndexHashChain {
                    computed: rebuilt_hash_chain_global_index,
                    input: new_hash_chain,
                },
            );
        }

        Ok(())
    }

    /// Verify the previous and new unset global index hash chain and its
    /// reconstruction.
    #[allow(unused)]
    fn verify_unset_global_index_hash_chain(
        &self,
        bridge_address: Address,
    ) -> Result<(), BridgeConstraintsError> {
        // 1.1 Get the state of the unset global index hash chain of the previous block on L2
        let prev_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self
                    .bridge_witness
                    .prev_unset_hash_chain_global_index_sketch,
                bridge_address,
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
            )
            .map_err(|e| {
                BridgeConstraintsError::static_call_error(e, StaticCallStage::PrevUnsetHashChain)
            })?;

            // check on block hash
            if retrieved_block_hash != self.prev_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.prev_l2_block_hash,
                    stage: StaticCallStage::PrevUnsetHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 1.2 Get the state of the unset global index hash chain of the new block on L2
        let new_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self.bridge_witness.new_unset_hash_chain_global_index_sketch,
                self.ger_addr,
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
            )
            .map_err(|e| {
                BridgeConstraintsError::static_call_error(e, StaticCallStage::NewUnsetHashChain)
            })?;

            // check on block hash
            if retrieved_block_hash != self.new_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.prev_l2_block_hash,
                    stage: StaticCallStage::NewUnsetHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 1.3 Check that the rebuilt hash chain is equal to the new hash chain
        let rebuilt_hash_chain_global_index = self
            .bridge_witness
            .global_indices_unset
            .iter()
            .fold(prev_hash_chain, |acc, &global_index_hashed| {
                keccak256_combine([acc, global_index_hashed.0.into()])
            });

        if rebuilt_hash_chain_global_index != new_hash_chain {
            return Err(BridgeConstraintsError::MismatchUnsetGlobalIndexHashChain {
                computed: rebuilt_hash_chain_global_index,
                input: new_hash_chain,
            });
        }

        Ok(())
    }

    /// Verify the claimed global indexes extracting the unset global indexes are equal to the
    /// Constrained global indexes.
    fn verify_constrained_global_indices(&self) -> Result<(), BridgeConstraintsError> {
        // Remove the unset indices from the claimed ones.

        // Create a map that counts how many removals are needed for each value in global_indices_unset.
        let mut removal_map: HashMap<B256, usize> = HashMap::new();
        for value in &self.bridge_witness.global_indices_unset {
            *removal_map.entry(value.clone()).or_insert(0) += 1;
        }

        // Iterate over claimed indices and remove (skip) one occurrence for each value in removal_map.
        let filtered_claimed: Vec<B256> = self
            .bridge_witness
            .global_indices_claimed
            .iter()
            .cloned()
            .filter(|value| {
                // If the value needs to be removed...
                if let Some(count) = removal_map.get_mut(value) {
                    if (*count) > 0 {
                        *count -= 1; // Remove one occurrence.
                        return false; // Skip including this occurrence.
                    }
                }
                true // Otherwise, keep the value.
            })
            .collect();

        // Check if the filtered claimed global indices are equal to the constrained global indices.
        if filtered_claimed != self.global_indices {
            return Err(BridgeConstraintsError::MismatchConstrainedGlobalIndices {
                computed: filtered_claimed,
                input: self.global_indices.clone(),
            });
        }

        Ok(())
    }

    /// Verify the new local exit root using the bridge address.
    fn verify_new_ler(&self, bridge_address: Address) -> Result<(), BridgeConstraintsError> {
        let (decoded_return, retrieved_block_hash) = execute_static_call(
            &self.bridge_witness.new_ler_sketch,
            bridge_address,
            BridgeL2SovereignChain::getRootCall {},
        )
        .map_err(|e| BridgeConstraintsError::static_call_error(e, StaticCallStage::NewLer))?;

        // Check on block hash.
        if retrieved_block_hash != self.new_l2_block_hash {
            return Err(BridgeConstraintsError::MismatchBlockHash {
                retrieved: retrieved_block_hash,
                input: self.new_l2_block_hash,
                stage: StaticCallStage::NewLer,
            });
        }

        // Decode new local exit root from the result.
        let new_ler = decoded_return.lastRollupExitRoot.0.into();
        if new_ler != self.new_local_exit_root {
            return Err(BridgeConstraintsError::MismatchNewLocalExitRoot {
                retrieved: new_ler,
                input: self.new_local_exit_root,
            });
        }

        Ok(())
    }

    /// Verify the inclusion proofs of the inserted GERs up to the L1InfoRoot.
    fn verify_inserted_gers(&self) -> Result<(), BridgeConstraintsError> {
        // Create a map that counts how many removals are needed for each value in global_indices_unset.
        let mut removal_map: HashMap<Digest, usize> = HashMap::new();
        for value in &self.bridge_witness.removed_gers_hash_chain {
            *removal_map.entry(value.clone()).or_insert(0) += 1;
        }

        // Iterate over claimed indices and remove (skip) one occurrence for each value in removal_map.
        let filtered_hash_chain_gers: Vec<Digest> = self
            .bridge_witness
            .inserted_gers_hash_chain
            .iter()
            .cloned()
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
            .collect();

        // Check that the filtered_hash_chain_gers are equal to the inserted
        let inserted_gers_compare: Vec<Digest> = self
            .bridge_witness
            .inserted_gers
            .iter()
            .map(|ger| ger.l1_info_tree_leaf.global_exit_root)
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
        self.verify_inserted_ger_hash_chain()?;
        self.verify_removed_ger_hash_chain()?;
        let bridge_address = self.fetch_bridge_address()?;
        self.verify_claimed_global_index_hash_chain(bridge_address)?;
        self.verify_unset_global_index_hash_chain(bridge_address)?;
        self.verify_constrained_global_indices()?;
        self.verify_new_ler(bridge_address)?;
        self.verify_inserted_gers()
    }
}

#[cfg(test)]
mod tests {
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
    //#[ignore = "e2e test, sepolia provider needed"]
    async fn test_bridge_contraints() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the environment variables.
        dotenvy::dotenv().ok();

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
        use std::collections::HashMap;
        let mut removal_map: HashMap<B256, usize> = HashMap::new();
        for idx in &unclaimed_global_indexes {
            *removal_map.entry(idx.clone()).or_insert(0) += 1;
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
            .map(|(index, ger)| InsertedGER {
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
                l1_info_tree_leaf: {
                    L1InfoTreeLeaf {
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
                    }
                },
                l1_info_tree_index: index as u32,
            })
            .collect();

        let inserted_gers: Vec<Digest> = imported_l1_info_tree_leafs
            .iter()
            .map(|ger| ger.l1_info_tree_leaf.global_exit_root)
            .collect();

        // remove the removed gers from the inserted GERS
        let mut removal_map: HashMap<Digest, usize> = HashMap::new();
        for value in &removed_gers {
            *removal_map.entry(value.clone()).or_insert(0) += 1;
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
        let _decoded_hash_chain =
            GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall::abi_decode_returns(
                &_hash_chain,
                true,
            )?
            .hashChain;
        let executor_prev_hash_chain_sketch = executor_prev_hash_chain.finalize().await?;

        // 2. Get the new inserted GER hash chain (new block on L2)
        let mut executor_new_hash_chain =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let _new_hash_chain = executor_new_hash_chain
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
            ))
            .await?;
        let executor_new_hash_chain = executor_new_hash_chain.finalize().await?;

        // 3. Get the bridge address.
        let mut executor_get_bridge_address =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let bridge_address_bytes = executor_get_bridge_address
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
            ))
            .await?;
        let bridge_address =
            GlobalExitRootManagerL2SovereignChain::bridgeAddressCall::abi_decode_returns(
                &bridge_address_bytes,
                true,
            )?
            .bridgeAddress;
        let executor_get_bridge_address_sketch = executor_get_bridge_address.finalize().await?;

        // 4. Get the new local exit root from the bridge on the new L2 block.
        let mut executor_get_ler =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let new_ler_bytes = executor_get_ler
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::getRootCall {},
            ))
            .await?;
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
        let mut executor_prev_removed =
            HostExecutor::new(provider_l2.clone(), block_number_initial).await?;
        let _prev_removed = executor_prev_removed
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
            ))
            .await?;
        let executor_prev_removed_sketch = executor_prev_removed.finalize().await?;

        // 6. Get the removed GER hash chain for the new block.
        let mut executor_new_removed =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let _new_removed = executor_new_removed
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
            ))
            .await?;
        let executor_new_removed_sketch = executor_new_removed.finalize().await?;

        // 7. Get the claimed global index hash chain for the previous block.
        let mut executor_prev_claimed =
            HostExecutor::new(provider_l2.clone(), block_number_initial).await?;
        let _prev_claimed = executor_prev_claimed
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
            ))
            .await?;
        let executor_prev_claimed_sketch = executor_prev_claimed.finalize().await?;

        // 8. Get the claimed global index hash chain for the new block.
        let mut executor_new_claimed =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let _new_claimed = executor_new_claimed
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
            ))
            .await?;
        let executor_new_claimed_sketch = executor_new_claimed.finalize().await?;

        // 9. Get the unset global index hash chain for the previous block.
        let mut executor_prev_unset =
            HostExecutor::new(provider_l2.clone(), block_number_initial).await?;
        let _prev_unset = executor_prev_unset
            .execute(ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
            ))
            .await?;
        let executor_prev_unset_sketch = executor_prev_unset.finalize().await?;

        // 10. Get the unset global index hash chain for the new block.
        let mut executor_new_unset =
            HostExecutor::new(provider_l2.clone(), block_number_final).await?;
        let _new_unset = executor_new_unset
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
            ))
            .await?;
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
                global_indices_claimed: claimed_global_indexes,
                global_indices_unset: unclaimed_global_indexes,
                prev_inserted_ger_hash_chain_sketch: executor_prev_hash_chain_sketch.clone(),
                new_inserted_ger_hash_chain_sketch: executor_new_hash_chain.clone(),
                prev_removed_ger_hash_chain_sketch: executor_prev_removed_sketch,
                new_removed_ger_hash_chain_sketch: executor_new_removed_sketch,
                prev_claimed_hash_chain_global_index_sketch: executor_prev_claimed_sketch,
                new_claimed_hash_chain_global_index_sketch: executor_new_claimed_sketch,
                prev_unset_hash_chain_global_index_sketch: executor_prev_unset_sketch,
                new_unset_hash_chain_global_index_sketch: executor_new_unset_sketch,
                bridge_address_sketch: executor_get_bridge_address_sketch,
                new_ler_sketch: executor_get_ler_sketch,
            },
        };

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
                    inserted_gers: bridge_data_input
                        .bridge_witness
                        .inserted_gers
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
                Err(BridgeConstraintsError::MismatchInsertedGERHashChain { .. })
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
