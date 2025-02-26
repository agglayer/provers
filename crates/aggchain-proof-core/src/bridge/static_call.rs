use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::ContractPublicValues;
use sp1_cc_client_executor::{io::EVMStateSketch, ClientExecutor, ContractInput};

/// Context giver for static call errors.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum StaticCallStage {
    PrevHashChain,
    NewHashChain,
    BridgeAddress,
    NewLER,
}

/// Represents all the static call errors.
#[derive(Clone, thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StaticCallError {
    #[error("Failure on the initialization of the ClientExecutor: {0}")]
    ClientInitializationFailure(String),
    #[error("Failure on the execution of the ClientExecutor: {0}")]
    ClientExecutionFailure(String),
    #[error("Failure on the decoding of the contractOutput: {0}")]
    DecodeContractOutputFailure(String),
}

/// Execute a static call.
/// Returns the public values and the decoded output values.
pub fn execute_static_call<C: SolCall>(
    state_sketch: &EVMStateSketch,
    contract_address: Address,
    calldata: C,
) -> Result<(ContractPublicValues, C::Return), StaticCallError> {
    let cc_public_values = ClientExecutor::new(state_sketch)
        .map_err(|e| StaticCallError::ClientInitializationFailure(e.to_string()))?
        .execute(ContractInput::new_call(
            contract_address,
            Address::default(),
            calldata,
        ))
        .map_err(|e| StaticCallError::ClientExecutionFailure(e.to_string()))?;

    let decoded_contract_output = C::abi_decode_returns(&cc_public_values.contractOutput, true)
        .map_err(|e| StaticCallError::DecodeContractOutputFailure(e.to_string()))?;

    Ok((cc_public_values, decoded_contract_output))
}
