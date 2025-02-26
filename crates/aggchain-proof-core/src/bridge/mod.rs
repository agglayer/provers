//! A program that verifies the bridge integrity
use alloy_primitives::{address, Address};
use alloy_sol_macro::sol;
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::io::EVMStateSketch;
use static_call::{execute_static_call, StaticCallError, StaticCallStage};

use crate::bridge::inserted_ger::InsertedGER;
use crate::keccak::digest::Digest;
use crate::keccak::keccak256_combine;

mod inserted_ger;
mod static_call;

// This solution won't work with Outpost networks as this address won't be
// constant GlobalExitRootManagerL2SovereignChain smart contract address
pub(crate) const L2_GER_ADDR: Address = address!("a40d5f56745a118d0906a34e69aec8c0db1cb8fa");

// Contract interfaces of the pre-deployed contracts on sovereign chains
sol! (
    interface GlobalExitRootManagerL2SovereignChain {
        function insertedGERHashChain() public view returns (bytes32 hashChain);
        function bridgeAddress() public view returns (address bridgeAddress);
    }
);

sol! (
    interface BridgeL2SovereignChain {
        function getRoot() public view returns (bytes32 lastRollupExitRoot);
    }
);

/// Represents all the bridge constraints errors.
#[derive(Clone, thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BridgeConstraintsError {
    /// The inclusion proof from the GER to the L1 info Root is invalid.
    #[error(
        "Invalid inclusion proof for inserted GER. l1_info_leaf_index: {l1_info_index}, \
         l1_info_root: {l1_info_root}, inserted_ger: {inserted_ger}"
    )]
    InvalidMerklePathGERToL1Root {
        inserted_ger: Digest,
        l1_info_index: u32,
        l1_info_root: Digest,
    },

    /// The block hashes used on the sketches do not match.
    #[error("Mismatch on the block hash at {stage:?}. retrieved: {retrieved}, input: {input}")]
    MismatchBlockHash {
        retrieved: Digest,
        input: Digest,
        stage: StaticCallStage,
    },

    /// The provided hash chain does not correspond with the computed one.
    #[error("Mismatch on the hash chain. computed: {computed}, input: {input}")]
    MismatchHashChain { computed: Digest, input: Digest },

    /// The provided new LER does not correspond with the one retrieved from
    /// contracts.
    #[error("Mismatch on the new LER. retrieved: {retrieved}, input: {input}")]
    MismatchNewLocalExitRoot { retrieved: Digest, input: Digest },

    /// The static call failed at the given stage.
    #[error("Failure upon static call at {stage:?}: {error}")]
    StaticCallError {
        stage: StaticCallStage,
        error: StaticCallError,
    },
}

/// Bridge data required to verify the BridgeConstraintsInput integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeWitness {
    pub injected_gers: Vec<InsertedGER>,
    pub prev_hash_chain_sketch: EVMStateSketch,
    pub new_hash_chain_sketch: EVMStateSketch,
    pub get_bridge_address_sketch: EVMStateSketch,
    pub new_ler_sketch: EVMStateSketch,
}

impl BridgeWitness {
    /// Computes the GER hash chain starting from the initial hash.
    fn ger_hash_chain(&self, initial_hash_chain: Digest) -> Digest {
        self.injected_gers
            .iter()
            .map(|inserted_ger| inserted_ger.ger())
            .fold(initial_hash_chain, |acc, ger| keccak256_combine([acc, ger]))
    }
}

/// Bridge data required to verify the bridge smart contract integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConstraintsInput {
    pub ger_addr: Address,
    pub prev_l2_block_hash: Digest,
    pub new_l2_block_hash: Digest,
    pub new_local_exit_root: Digest,
    pub l1_info_root: Digest,
    pub bridge_witness: BridgeWitness,
}

// Warning using static calls:
// The static call must not use the chainID opcode, since will return 1
// (mainnet). Evm version used by the solidity compiler must be compatible with
// the version used on the static call. No special precompileds are supported
// Even though the current example satisfies these constraints, it's important
// to keep them in mind when updating the code.
impl BridgeConstraintsInput {
    /// Verify the bridge state.
    pub fn verify(&self) -> Result<(), BridgeConstraintsError> {
        let static_call_error =
            |error, stage| BridgeConstraintsError::StaticCallError { stage, error };

        // 1. Get the state of the hash chain of the previous block on L2
        let prev_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self.bridge_witness.prev_hash_chain_sketch,
                self.ger_addr,
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
            )
            .map_err(|e| static_call_error(e, StaticCallStage::PrevHashChain))?;

            // check on block hash
            if retrieved_block_hash != self.prev_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.prev_l2_block_hash,
                    stage: StaticCallStage::PrevHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 2. Get the state of the hash chain of the new block on L2
        let new_hash_chain: Digest = {
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self.bridge_witness.new_hash_chain_sketch,
                self.ger_addr,
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
            )
            .map_err(|e| static_call_error(e, StaticCallStage::NewHashChain))?;

            // check on block hash
            if retrieved_block_hash != self.new_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.new_l2_block_hash,
                    stage: StaticCallStage::NewHashChain,
                });
            }

            decoded_return.hashChain.0.into()
        };

        // 3.1 Get the bridge address from the GER smart contract.
        // Since the bridge address is not constant but the l2 ger address is
        // We can retrieve the bridge address saving some public inputs and possible
        // errors
        let bridge_address = {
            // Execute the static call
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self.bridge_witness.get_bridge_address_sketch,
                self.ger_addr,
                GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
            )
            .map_err(|e| static_call_error(e, StaticCallStage::BridgeAddress))?;

            // check on block hash
            if retrieved_block_hash != self.new_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.new_l2_block_hash,
                    stage: StaticCallStage::BridgeAddress,
                });
            }

            // Decode new bridge address from the result
            decoded_return.bridgeAddress
        };

        // 3.2 Get the new local exit root
        let new_ler = {
            // Execute the static call
            let (decoded_return, retrieved_block_hash) = execute_static_call(
                &self.bridge_witness.new_ler_sketch,
                bridge_address,
                BridgeL2SovereignChain::getRootCall {},
            )
            .map_err(|e| static_call_error(e, StaticCallStage::NewLER))?;

            // check on block hash
            if retrieved_block_hash != self.new_l2_block_hash {
                return Err(BridgeConstraintsError::MismatchBlockHash {
                    retrieved: retrieved_block_hash,
                    input: self.new_l2_block_hash,
                    stage: StaticCallStage::NewLER,
                });
            }

            // Decode new local exit root from the result
            decoded_return.lastRollupExitRoot.0.into()
        };

        // 4. Check consistency of the calls

        // 4.1 Reconstruct hashChain based on the previous hashChain and the injected
        // GERs
        let reconstructed_hash_chain = self.bridge_witness.ger_hash_chain(prev_hash_chain);

        // Check that the reconstructed hash chain is equal to the new hash chain
        if reconstructed_hash_chain != new_hash_chain {
            return Err(BridgeConstraintsError::MismatchHashChain {
                computed: reconstructed_hash_chain,
                input: new_hash_chain,
            });
        }

        // 4.2 Check that the new local exit root returned from L2 matches the expected
        if new_ler != self.new_local_exit_root {
            return Err(BridgeConstraintsError::MismatchNewLocalExitRoot {
                retrieved: new_ler,
                input: self.new_local_exit_root,
            });
        }

        // 4.3 Check Gers are inside of L1InfoRoot
        self.verify_inserted_gers()
    }

    /// Verify the inclusion proofs of the inserted GERs up to the L1InfoRoot.
    fn verify_inserted_gers(&self) -> Result<(), BridgeConstraintsError> {
        self.bridge_witness
            .injected_gers
            .iter()
            .find(|ger| !ger.verify(self.l1_info_root))
            .map_or(Ok(()), |inserted_ger| {
                Err(BridgeConstraintsError::InvalidMerklePathGERToL1Root {
                    inserted_ger: inserted_ger.ger(),
                    l1_info_index: inserted_ger.l1_info_tree_index,
                    l1_info_root: self.l1_info_root,
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::str::FromStr;

    use alloy_primitives::hex;
    use alloy_provider::RootProvider;
    use alloy_rpc_types::BlockNumberOrTag;
    use alloy_sol_types::SolCall;
    use serde_json::Value;
    use sp1_cc_client_executor::ContractInput;
    use sp1_cc_host_executor::HostExecutor;
    use url::Url;

    use super::*;
    use crate::bridge::inserted_ger::L1InfoTreeLeaf;
    use crate::local_exit_tree::proof::LETMerkleProof;

    #[tokio::test(flavor = "multi_thread")]
    //#[ignore = "Unable to properly test with mock yet"]
    async fn test_bridge_contraints() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the environment variables.
        dotenvy::dotenv().ok();

        // Read and parse the JSON file
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/test_input/bridge_test.json");
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
            .collect();

        let rpc_url_l2 = std::env::var(format!("RPC_{}", chain_id_l2))
            .expect("RPC URL must be defined")
            .parse::<Url>()
            .expect("Invalid URL format");

        let block_number_initial = BlockNumberOrTag::Number(initial_block_number);
        let block_number_final = BlockNumberOrTag::Number(final_block_number);

        // 1. Get the the prev hash chain of the previous block on L2

        // Setup the provider and host executor for initial hash chain
        let provider_l2: RootProvider<alloy::network::AnyNetwork> =
            RootProvider::new_http(rpc_url_l2);

        let mut executor_prev_hash_chain =
            HostExecutor::new(provider_l2.clone(), block_number_initial).await?;

        let hash_chain_calldata =
            GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {};

        let _hash_chain = executor_prev_hash_chain
            .execute(ContractInput::new_call(
                ger_address,
                Address::default(),
                hash_chain_calldata.clone(),
            ))
            .await?;

        let _decoded_hash_chain =
            GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall::abi_decode_returns(
                &_hash_chain,
                true,
            )?
            .hashChain;
        let executor_prev_hash_chain_sketch = executor_prev_hash_chain.finalize().await?;

        // 2. Get the new hash chain of the new block on L2
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

        // 3. Get the bridge address
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

        // 4. Get the new local exit root from the bridge on the new L2 block
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
            bridge_witness: BridgeWitness {
                injected_gers: imported_l1_info_tree_leafs,
                prev_hash_chain_sketch: executor_prev_hash_chain_sketch.clone(),
                new_hash_chain_sketch: executor_new_hash_chain.clone(),
                get_bridge_address_sketch: executor_get_bridge_address_sketch,
                new_ler_sketch: executor_get_ler_sketch,
            },
        };

        assert!(bridge_data_input.verify().is_ok());
        Ok(())
    }
}
