//! A program that verifies the bridge integrity
use alloy_sol_types::SolCall;
use alloy_sol_macro::sol;
use alloy_primitives::{B256, address, Address, FixedBytes};
use sp1_cc_client_executor::{io::EVMStateSketch, ClientExecutor, ContractInput};

pub const GER_ADDR: Address = address!("0000000000000000000000000000000000000000");

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeInput {
    pub new_local_exit_root: FixedBytes<32>,
    pub injected_gers: Vec<B256>,
    pub prev_hash_chain_sketch: EVMStateSketch,
    pub new_hash_chain_sketch: EVMStateSketch,
    pub new_ler_sketch: EVMStateSketch,
    pub prev_l2_block_hash: FixedBytes<32>,
    pub new_l2_block_hash: FixedBytes<32>,
}


// try what happens if the calls revert?¿
sol! (
    interface GlobalExitRootManagerL2SovereignChain {
        function insertedGERHashChain() public view returns (bytes32 hashChain);
        function lastRollupExitRoot() public view returns (bytes32 lastRollupExitRoot);
    }
);


pub fn verify_bridge_state(bridge_input: BridgeInput) {
    // Read the bridge_input.
    // Todo explore other decodings for optimizing performance
    // let sbridge_input_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    // let input = bincode::deserialize::<BridgeInput>(&sbridge_input_bytes).unwrap();

    // Verify bridge state:

    // 1. Get the state of the hash chain of the previous block on L2

    // Load executor with the previous L2 block sketch
    let executor_prev_hash_chain: ClientExecutor = ClientExecutor::new(
        bridge_input.prev_hash_chain_sketch 
    ).unwrap();

    let hash_chain_calldata = GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {};
    let get_prev_hash_chain_input = ContractInput::new_call(GER_ADDR, Address::default(), hash_chain_calldata.clone());


    // Execute the static call
    let prev_hash_chain_call_output = executor_prev_hash_chain.execute(
        get_prev_hash_chain_input
    ).unwrap();

    // Decode ger count from the result
    let prev_hash_chain = GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall::abi_decode_returns(
        &prev_hash_chain_call_output.contractOutput, true
    ).unwrap().hashChain;

    // 2. Get the state of the hash chain of the new block on L2
    let executor_new_hash_chain: ClientExecutor = ClientExecutor::new(
        bridge_input.new_hash_chain_sketch
    ).unwrap();


    let get_new_hash_chain_contract_input: ContractInput = ContractInput::new_call(GER_ADDR, Address::default(), hash_chain_calldata);

    // Execute the static call
    let new_hash_chain_call_output = executor_new_hash_chain.execute(
        get_new_hash_chain_contract_input
    ).unwrap();


    // Decode ger count from the result
    let new_hash_chain = GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall::abi_decode_returns(
        &new_hash_chain_call_output.contractOutput, true
    ).unwrap().hashChain;

    // 3. Reconstruct hashChain


    // 4. Check Gers are inside of L1InfoRoot

    
    // 5. Get the new local exit root
     let executor_new_ler: ClientExecutor = ClientExecutor::new(
        bridge_input.new_ler_sketch
    ).unwrap();

    let get_new_ler_contract_input: ContractInput = ContractInput::new_call(GER_ADDR, Address::default(), GlobalExitRootManagerL2SovereignChain::lastRollupExitRootCall {});

    // Execute the static call
    let new_ler_call_output = executor_new_ler.execute(
        get_new_ler_contract_input
    ).unwrap();


    // Decode ger count from the result
    let new_ler = GlobalExitRootManagerL2SovereignChain::lastRollupExitRootCall::abi_decode_returns(
        &new_ler_call_output.contractOutput, true
    ).unwrap().lastRollupExitRoot;

    // assert blockhashes   
    assert!(bridge_input.prev_l2_block_hash == prev_hash_chain_call_output.blockHash, "block hash does not match");
    assert!(bridge_input.new_l2_block_hash == new_hash_chain_call_output.blockHash, "block hash does not match");
    assert!(bridge_input.new_l2_block_hash == new_ler_call_output.blockHash, "block hash does not match");
}
