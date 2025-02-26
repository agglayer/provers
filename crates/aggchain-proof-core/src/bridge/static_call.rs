use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::{io::EVMStateSketch, ClientExecutor, ContractInput};

use crate::keccak::digest::Digest;

/// Context giver about the stage of the error.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum StaticCallStage {
    /// Related to the hash chain fetch in the previous L2 block.
    PrevHashChain,
    /// Related to the hash chain fetch in the new L2 block.
    NewHashChain,
    /// Related to the fetch of the bridge address from the GER smart contract.
    BridgeAddress,
    /// Related to the fetch of the new local exit root.
    NewLER,
}

/// Represents all the static call errors.
#[derive(Clone, thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StaticCallError {
    #[error("Failure on the initialization of the ClientExecutor: {0}")]
    ClientInitialization(String),
    #[error("Failure on the execution of the ClientExecutor: {0}")]
    ClientExecution(String),
    #[error("Failure on the decoding of the contractOutput: {0}")]
    DecodeContractOutput(String),
}

/// Execute a static call.
/// Returns the decoded output values and the block hash.
pub fn execute_static_call<C: SolCall>(
    state_sketch: &EVMStateSketch,
    contract_address: Address,
    calldata: C,
) -> Result<(C::Return, Digest), StaticCallError> {
    let cc_public_values = ClientExecutor::new(state_sketch)
        .map_err(|e| StaticCallError::ClientInitialization(e.to_string()))?
        .execute(ContractInput::new_call(
            contract_address,
            Address::default(),
            calldata,
        ))
        .map_err(|e| StaticCallError::ClientExecution(e.to_string()))?;

    let decoded_contract_output = C::abi_decode_returns(&cc_public_values.contractOutput, true)
        .map_err(|e| StaticCallError::DecodeContractOutput(e.to_string()))?;

    Ok((decoded_contract_output, cc_public_values.blockHash.0.into()))
}
