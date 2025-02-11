//! A program that verifies the bridge integrity
use alloy_primitives::{address, Address, FixedBytes};
use alloy_sol_macro::sol;
use alloy_sol_types::SolCall;
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::{io::EVMStateSketch, ClientExecutor, ContractInput};

use crate::inserted_ger::InsertedGER;
use crate::keccak::keccak256_combine;

// temporal solution, won't work with Outpost networks
pub const L2_GER_ADDR: Address = address!("a40d5f56745a118d0906a34e69aec8c0db1cb8fa");

sol! (
    interface GlobalExitRootManagerL2SovereignChain {
        function insertedGERHashChain() public view returns (bytes32 hashChain);
        function lastRollupExitRoot() public view returns (bytes32 lastRollupExitRoot);
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
    MismatchBlockHash {
        left: FixedBytes<32>,
        right: FixedBytes<32>,
    },

    #[error("Hash chain does not match: computed {computed} vs input {input}")]
    MismatchHashChain {
        computed: FixedBytes<32>,
        input: FixedBytes<32>,
    },

    #[error("Local exit root does not match: retrieved {retrieved} vs input {input}")]
    MismatchLocalExitRoot {
        retrieved: FixedBytes<32>,
        input: FixedBytes<32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeWitness {
    pub injected_gers: Vec<InsertedGER>,
    pub prev_hash_chain_sketch: EVMStateSketch,
    pub new_hash_chain_sketch: EVMStateSketch,
    pub new_ler_sketch: EVMStateSketch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeInput {
    pub ger_addr: Address,
    pub prev_l2_block_hash: FixedBytes<32>,
    pub new_l2_block_hash: FixedBytes<32>,
    pub new_local_exit_root: FixedBytes<32>,
    pub l1_info_root: FixedBytes<32>,
    pub bridge_witness: BridgeWitness,
}

impl BridgeInput {
    pub fn verify(&self) -> Result<(), BridgeConstraintsError> {
        // TODO: handle failed calls
        // TODO: explore other decodings for optimizing performance
        // let sbridge_input_bytes = sp1_zkvm::io::read::<Vec<u8>>();
        // let input =
        // bincode::deserialize::<BridgeInput>(&sbridge_input_bytes).unwrap();

        // Verify bridge state:

        // 1. Get the state of the hash chain of the previous block on L2

        // Load executor with the previous L2 block sketch
        let executor_prev_hash_chain: ClientExecutor =
            ClientExecutor::new(&self.bridge_witness.prev_hash_chain_sketch).unwrap();

        let hash_chain_calldata =
            GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {};
        let get_prev_hash_chain_input = ContractInput::new_call(
            self.ger_addr,
            Address::default(),
            hash_chain_calldata.clone(),
        );

        // Execute the static call
        let prev_hash_chain_call_output = executor_prev_hash_chain
            .execute(get_prev_hash_chain_input)
            .unwrap();

        // Decode prev hash chain from the result
        let prev_hash_chain =
            GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall::abi_decode_returns(
                &prev_hash_chain_call_output.contractOutput,
                true,
            )
            .unwrap()
            .hashChain;

        // 2. Get the state of the hash chain of the new block on L2
        let executor_new_hash_chain: ClientExecutor =
            ClientExecutor::new(&self.bridge_witness.new_hash_chain_sketch).unwrap();

        let get_new_hash_chain_contract_input: ContractInput =
            ContractInput::new_call(self.ger_addr, Address::default(), hash_chain_calldata);

        // Execute the static call
        let new_hash_chain_call_output = executor_new_hash_chain
            .execute(get_new_hash_chain_contract_input)
            .unwrap();

        // Decode new hash chain from the result
        let new_hash_chain =
            GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall::abi_decode_returns(
                &new_hash_chain_call_output.contractOutput,
                true,
            )
            .unwrap()
            .hashChain;

        // 3. Get the new local exit root
        let executor_new_ler: ClientExecutor =
            ClientExecutor::new(&self.bridge_witness.new_ler_sketch).unwrap();

        let get_new_ler_contract_input: ContractInput = ContractInput::new_call(
            self.ger_addr,
            Address::default(),
            GlobalExitRootManagerL2SovereignChain::lastRollupExitRootCall {},
        );

        // Execute the static call
        let new_ler_call_output = executor_new_ler
            .execute(get_new_ler_contract_input)
            .unwrap();

        // Decode new local exit root from the result
        let new_ler =
            GlobalExitRootManagerL2SovereignChain::lastRollupExitRootCall::abi_decode_returns(
                &new_ler_call_output.contractOutput,
                true,
            )
            .unwrap()
            .lastRollupExitRoot;

        // 4. Check consistency of the calls

        // 4.1 Reconstruct hashChain based on the previous hashChain and the injected
        // Gers
        let reconstructed_hash_chain = compute_ger_hash_chain(
            prev_hash_chain,
            self.bridge_witness
                .injected_gers
                .iter()
                .map(|inserted_ger| inserted_ger.inserted_ger().0.into()),
        );

        // check that the reconstructed hash chain is equal to the new hash chain
        if reconstructed_hash_chain != new_hash_chain {
            return Err(BridgeConstraintsError::MismatchHashChain {
                computed: reconstructed_hash_chain,
                input: new_hash_chain,
            });
        }

        // 4.2 Check that the new local exit root returned from L2 matches the expected
        // value in the input.
        if new_ler != self.new_local_exit_root {
            return Err(BridgeConstraintsError::MismatchLocalExitRoot {
                retrieved: self.new_local_exit_root,
                input: new_ler,
            });
        }

        // 4.3 Check Gers are inside of L1InfoRoot TODO
        self.verify_inserted_gers()?;

        // 4.4. Check the block hashes of sketches matches the inputs
        let check_block_hash = |left, right| {
            if left != right {
                Err(BridgeConstraintsError::MismatchBlockHash { left, right })
            } else {
                Ok(())
            }
        };

        // assert blockhashes
        check_block_hash(
            self.prev_l2_block_hash,
            prev_hash_chain_call_output.blockHash,
        )?;

        check_block_hash(self.new_l2_block_hash, new_hash_chain_call_output.blockHash)?;

        check_block_hash(self.new_l2_block_hash, new_ler_call_output.blockHash)?;

        Ok(())
    }

    pub fn verify_inserted_gers(&self) -> Result<(), BridgeConstraintsError> {
        self.bridge_witness
            .injected_gers
            .iter()
            .find(|ger| !ger.verify(self.l1_info_root.0.into()))
            .map_or(Ok(()), |_| {
                Err(BridgeConstraintsError::InvalidMerklePathGERToL1Root)
            })
    }
}

pub fn compute_ger_hash_chain(
    initial_hash_chain: FixedBytes<32>,
    global_exit_roots: impl Iterator<Item = FixedBytes<32>>,
) -> FixedBytes<32> {
    let mut hash_chain = initial_hash_chain;
    for ger in global_exit_roots {
        hash_chain = FixedBytes::from(keccak256_combine([hash_chain, ger]).0);
    }
    hash_chain
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{address, hex};
    use alloy_provider::RootProvider;
    use alloy_rpc_types::BlockNumberOrTag;
    use sp1_cc_host_executor::HostExecutor;
    use url::Url;

    use super::*;
    use crate::inserted_ger::L1InfoTreeLeaf;

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "Unable to properly test with mock yet"]
    async fn test_bridge_contraints() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the environment variables.
        dotenvy::dotenv().ok();

        let bridge_data_input = {
            const CHAIN_ID_L2: u64 = 11155111;
            const PREV_BLOCK_NUMBER_L2: u64 = 7733215; // example value
            const NEW_BLOCK_NUMBER_L2: u64 = 7733218; // example value
            const GER_ADDR: Address = address!("877f8af6F04658769353ac51120283B0BF4A260D");
            const IMPORTED_GERS: [&str; 2] = [
                "0xa4b867dea490e3735bce8712e7c5071c4dca879b4f8eaff0f973d808a623e425",
                "0x0bf1d0a5a680af4ea266ab7a8052735e1e3faf71fadfe8cfdeae70b1ab8f9d85",
            ];

            // Convert the hex strings to FixedBytes by decoding each hex string.
            let imported_gers: Vec<InsertedGER> = IMPORTED_GERS
                .iter()
                .map(|ger_hex| {
                    let bytes = hex::decode(ger_hex).unwrap();
                    let inserted_ger = alloy_primitives::FixedBytes::from_slice(&bytes);

                    // TODO: proper merkle proof
                    InsertedGER {
                        proof: Default::default(),
                        l1_info_tree_leaf: L1InfoTreeLeaf {
                            global_exit_root: inserted_ger.0.into(),
                            block_hash: Default::default(),
                            timestamp: Default::default(),
                        },
                        l1_info_tree_index: Default::default(),
                    }
                })
                .collect();

            let rpc_url_l2 = std::env::var(format!("RPC_{}", CHAIN_ID_L2))
                .expect("RPC URL must be defined")
                .parse::<Url>()
                .expect("Invalid URL format");

            let block_number_initial = BlockNumberOrTag::Number(PREV_BLOCK_NUMBER_L2);
            let block_number_final = BlockNumberOrTag::Number(NEW_BLOCK_NUMBER_L2);

            // 1. Get the the prev hash chain of the previous block on L2

            // Setup the provider and host executor for initial GER
            let provider_l2: RootProvider<alloy::network::AnyNetwork> =
                RootProvider::new_http(rpc_url_l2);

            let mut executor_prev_hash_chain =
                HostExecutor::new(provider_l2.clone(), block_number_initial).await?;

            let hash_chain_calldata =
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {};

            let _hash_chain = executor_prev_hash_chain
                .execute(ContractInput::new_call(
                    GER_ADDR,
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
                    GER_ADDR,
                    Address::default(),
                    GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
                ))
                .await?;

            let executor_new_hash_chain = executor_new_hash_chain.finalize().await?;

            // 3. Get the new local exit root from the new L2 block
            let mut executor_get_ler =
                HostExecutor::new(provider_l2.clone(), block_number_final).await?;

            let new_ler_bytes = executor_get_ler
                .execute(ContractInput::new_call(
                    GER_ADDR,
                    Address::default(),
                    GlobalExitRootManagerL2SovereignChain::lastRollupExitRootCall {},
                ))
                .await?;

            let new_ler =
                GlobalExitRootManagerL2SovereignChain::lastRollupExitRootCall::abi_decode_returns(
                    &new_ler_bytes,
                    true,
                )?
                .lastRollupExitRoot;

            let executor_get_ler_sketch = executor_get_ler.finalize().await?;

            // Commit the bridge proof.
            BridgeInput {
                ger_addr: GER_ADDR,
                prev_l2_block_hash: executor_prev_hash_chain_sketch.header.hash_slow(),
                new_l2_block_hash: executor_new_hash_chain.header.hash_slow(),
                new_local_exit_root: new_ler,
                l1_info_root: FixedBytes::default(),
                bridge_witness: BridgeWitness {
                    injected_gers: imported_gers,
                    prev_hash_chain_sketch: executor_prev_hash_chain_sketch.clone(),
                    new_hash_chain_sketch: executor_new_hash_chain.clone(),
                    new_ler_sketch: executor_get_ler_sketch,
                },
            }
        };

        assert!(bridge_data_input.verify().is_ok());
        Ok(())
    }
}
