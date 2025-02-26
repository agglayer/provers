//! A program that verifies the bridge integrity
use alloy_primitives::{address, Address};
use alloy_sol_macro::sol;
use alloy_sol_types::SolCall;
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::{io::EVMStateSketch, ClientExecutor, ContractInput};

use crate::inserted_ger::InsertedGER;
use crate::keccak::digest::Digest;
use crate::keccak::keccak256_combine;

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
    #[error("Invalid merkle path from the GER to the L1 Info Root.")]
    InvalidMerklePathGERToL1Root,

    /// The block hashes used on the sketches do not match
    #[error("Block hash does not match: {left} != {right}")]
    MismatchBlockHash { left: Digest, right: Digest },

    #[error("Hash chain does not match: computed {computed} vs input {input}")]
    MismatchHashChain { computed: Digest, input: Digest },

    #[error("Local exit root does not match: retrieved {retrieved} vs input {input}")]
    MismatchLocalExitRoot { retrieved: Digest, input: Digest },
}

// All the bridge data required to verify the BridgeConstraintsInput integrity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeWitness {
    pub injected_gers: Vec<InsertedGER>,
    pub prev_hash_chain_sketch: EVMStateSketch,
    pub new_hash_chain_sketch: EVMStateSketch,
    pub get_bridge_address_sketch: EVMStateSketch,
    pub new_ler_sketch: EVMStateSketch,
}

// All the bridge data required to verify the bridge smart contract integrity
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
    /// Returns the hash chain and its block hash for the given block sketch.
    fn hash_chain_state(&self, sketch: &EVMStateSketch) -> (Digest, Digest) {
        // Load executor with the L2 block sketch
        let executor_hash_chain: ClientExecutor = ClientExecutor::new(&sketch).unwrap();

        let get_hash_chain_input = ContractInput::new_call(
            self.ger_addr,
            Address::default(),
            GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
        );

        // Execute the static call
        let hash_chain_call_output = executor_hash_chain.execute(get_hash_chain_input).unwrap();

        // Decode hash chain from the result
        (
            GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall::abi_decode_returns(
                &hash_chain_call_output.contractOutput,
                true,
            )
            .unwrap()
            .hashChain
            .0
            .into(),
            hash_chain_call_output.blockHash.0.into(),
        )
    }

    pub fn verify(&self) -> Result<(), BridgeConstraintsError> {
        // Verify bridge state:

        // 1. Get the state of the hash chain of the previous block on L2
        let (prev_hash_chain, prev_hash_chain_block_hash) =
            self.hash_chain_state(&self.bridge_witness.prev_hash_chain_sketch);

        // 2. Get the state of the hash chain of the new block on L2
        let (new_hash_chain, new_hash_chain_block_hash) =
            self.hash_chain_state(&self.bridge_witness.new_hash_chain_sketch);

        // 3.1 Get the bridge address from the GER smart contract.
        // Since the bridge address is not constant but the l2 ger address is
        // We can retrieve the bridge address saving some public inputs and possible
        // errors
        let (bridge_address, bridge_address_retrieved_block_hash) = {
            let executor_get_bridge_address: ClientExecutor =
                ClientExecutor::new(&self.bridge_witness.get_bridge_address_sketch).unwrap();

            let get_bridge_address_contract_input: ContractInput = ContractInput::new_call(
                self.ger_addr,
                Address::default(),
                GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
            );

            // Execute the static call
            let get_bridge_address_call_output = executor_get_bridge_address
                .execute(get_bridge_address_contract_input)
                .unwrap();

            // Decode new bridge address from the result
            (
                GlobalExitRootManagerL2SovereignChain::bridgeAddressCall::abi_decode_returns(
                    &get_bridge_address_call_output.contractOutput,
                    true,
                )
                .unwrap()
                .bridgeAddress,
                get_bridge_address_call_output.blockHash.0.into(),
            )
        };

        // 3.2 Get the new local exit root
        let (new_ler, new_ler_retrieved_block_hash) = {
            let executor_new_ler: ClientExecutor =
                ClientExecutor::new(&self.bridge_witness.new_ler_sketch).unwrap();

            let get_new_ler_contract_input: ContractInput = ContractInput::new_call(
                bridge_address,
                Address::default(),
                BridgeL2SovereignChain::getRootCall {},
            );

            // Execute the static call
            let new_ler_call_output = executor_new_ler
                .execute(get_new_ler_contract_input)
                .unwrap();

            // Decode new local exit root from the result
            (
                BridgeL2SovereignChain::getRootCall::abi_decode_returns(
                    &new_ler_call_output.contractOutput,
                    true,
                )
                .unwrap()
                .lastRollupExitRoot
                .0
                .into(),
                new_ler_call_output.blockHash.0.into(),
            )
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
            return Err(BridgeConstraintsError::MismatchLocalExitRoot {
                retrieved: self.new_local_exit_root,
                input: new_ler,
            });
        }

        // 4.3 Check Gers are inside of L1InfoRoot
        self.verify_inserted_gers()?;

        // 4.4. Check the block hashes of sketches matches the inputs
        // So we that all the static calls are consistent with the blockhashes
        let check_block_hash = |left, right| {
            if left != right {
                Err(BridgeConstraintsError::MismatchBlockHash { left, right })
            } else {
                Ok(())
            }
        };

        // assert blockhashes
        check_block_hash(self.prev_l2_block_hash, prev_hash_chain_block_hash)?;

        check_block_hash(self.new_l2_block_hash, new_hash_chain_block_hash)?;

        check_block_hash(self.new_l2_block_hash, bridge_address_retrieved_block_hash)?;

        check_block_hash(self.new_l2_block_hash, new_ler_retrieved_block_hash)?;

        Ok(())
    }

    // Verify all insterted gers are inside of the L1InfoRoot using merkle proofs
    pub fn verify_inserted_gers(&self) -> Result<(), BridgeConstraintsError> {
        self.bridge_witness
            .injected_gers
            .iter()
            .all(|ger| ger.verify(self.l1_info_root.0.into()))
            .then_some(())
            .ok_or(BridgeConstraintsError::InvalidMerklePathGERToL1Root)
    }
}

impl BridgeWitness {
    /// Computes the GER hash chain from the initial hash.
    fn ger_hash_chain(&self, initial_hash_chain: Digest) -> Digest {
        self.injected_gers
            .iter()
            .map(|ger| ger.inserted_ger())
            .fold(initial_hash_chain, |acc, ger| keccak256_combine([acc, ger]))
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
    use serde_json::Value;
    use sp1_cc_host_executor::HostExecutor;
    use url::Url;

    use super::*;
    use crate::inserted_ger::L1InfoTreeLeaf;
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
