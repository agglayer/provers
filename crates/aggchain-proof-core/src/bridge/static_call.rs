use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use sp1_cc_client_executor::{io::EVMStateSketch, ClientExecutor, ContractInput};

use crate::keccak::digest::Digest;

/// Context giver about the stage of the error.
#[derive(Clone, Debug)]
pub enum StaticCallStage {
    /// Related to the fetch of the hash chain on GER in the previous L2 block.
    PrevHashChainGER,
    /// Related to the fetch of the hash chain on GER in the new L2 block.
    NewHashChainGER,
    /// Related to the fetch of the hash chain on global indices in the previous
    /// L2 block.
    PrevHashChainGlobalIndex,
    /// Related to the fetch of the hash chain on global indices in the new L2
    /// block.
    NewHashChainGlobalIndex,
    /// Related to the fetch of the bridge address from the GER smart contract.
    BridgeAddress,
    /// Related to the fetch of the new local exit root.
    NewLer,
}

/// Represents all the static call errors.
#[derive(thiserror::Error, Debug)]
pub enum StaticCallError {
    #[error("Failure on the initialization of the ClientExecutor.")]
    ClientInitialization(#[source] eyre::Report),
    #[error("Failure on the execution of the ClientExecutor.")]
    ClientExecution(#[source] eyre::Report),
    #[error("Failure on the decoding of the contractOutput.")]
    DecodeContractOutput(#[source] alloy_sol_types::Error),
}

/// Returns the decoded output values and the block hash of a static call.
///
/// WARN: The static call must not use the `chainID` opcode, as it will return 1
/// (mainnet). The EVM version used by the Solidity compiler must be compatible
/// with the version used in the static call. No special precompiled contracts
/// are supported.
/// Even though the current example satisfies these constraints, it's important
/// to keep them in mind when updating the code.
pub fn execute_static_call<C: SolCall>(
    state_sketch: &EVMStateSketch,
    contract_address: Address,
    calldata: C,
) -> Result<(C::Return, Digest), StaticCallError> {
    let cc_public_values = ClientExecutor::new(state_sketch)
        .map_err(StaticCallError::ClientInitialization)?
        .execute(ContractInput::new_call(
            contract_address,
            Address::default(),
            calldata,
        ))
        .map_err(StaticCallError::ClientExecution)?;

    let decoded_contract_output = C::abi_decode_returns(&cc_public_values.contractOutput, true)
        .map_err(StaticCallError::DecodeContractOutput)?;

    Ok((decoded_contract_output, cc_public_values.blockHash.0.into()))
}
