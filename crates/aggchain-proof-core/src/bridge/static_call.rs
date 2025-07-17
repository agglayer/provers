use agglayer_primitives::Digest;
use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use sp1_cc_client_executor::{io::EvmSketchInput, ClientExecutor, ContractInput};

use super::BridgeConstraintsError;

/// Context giver about the stage of the error.
#[derive(Clone, Copy, Debug)]
pub enum HashChainType {
    InsertedGER,
    ClaimedGlobalIndex,
    RemovedGER,
    UnsetGlobalIndex,
}

/// Context giver about the stage of the error.
#[derive(Clone, Copy, Debug)]
pub enum StaticCallStage {
    /// Related to the fetch of the hash chain in the previous L2 block.
    PrevHashChain(HashChainType),
    /// Related to the fetch of the hash chain in the new L2 block.
    NewHashChain(HashChainType),
    /// Related to the fetch of the bridge address from the GER smart contract.
    BridgeAddress,
    /// Related to the fetch of the new local exit root.
    NewLer,
}

/// Represents all the static call errors.
#[derive(thiserror::Error, Debug)]
pub enum StaticCallError {
    #[error("Failure on the initialization of the ClientExecutor.")]
    ClientInitialization(#[source] sp1_cc_client_executor::ClientError),
    #[error("Failure on the execution of the ClientExecutor.")]
    ClientExecution(#[source] eyre::Report),
    #[error("Failure on the decoding of the contractOutput.")]
    DecodeContractOutput(#[source] alloy_sol_types::Error),
}

pub struct StaticCallWithContext {
    pub(crate) address: Address,
    pub(crate) stage: StaticCallStage,
    pub(crate) block_hash: Digest,
}

impl StaticCallWithContext {
    /// Returns the decoded output values of a static call.
    pub fn execute<C: SolCall>(
        &self,
        caller_address: Address,
        state_sketch: &EvmSketchInput,
        calldata: C,
    ) -> Result<C::Return, BridgeConstraintsError> {
        let (decoded_return, retrieved_block_hash) = self
            .execute_helper(state_sketch, calldata, caller_address)
            .map_err(|e| BridgeConstraintsError::static_call_error(e, self.stage))?;

        // check on block hash
        if retrieved_block_hash != self.block_hash {
            return Err(BridgeConstraintsError::MismatchBlockHash {
                retrieved: retrieved_block_hash,
                input: self.block_hash,
                stage: self.stage,
            });
        }

        Ok(decoded_return)
    }

    /// Returns the decoded output values and the block hash of a static call.
    ///
    /// WARN: The static call must not use the `chainID` opcode, as it will
    /// return 1 (mainnet). The EVM version used by the Solidity compiler
    /// must be compatible with the version used in the static call. No
    /// special precompiled contracts are supported.
    /// Even though the current example satisfies these constraints, it's
    /// important to keep them in mind when updating the code.
    fn execute_helper<C: SolCall>(
        &self,
        state_sketch: &EvmSketchInput,
        calldata: C,
        caller_address: Address,
    ) -> Result<(C::Return, Digest), StaticCallError> {
        let cc_public_values = ClientExecutor::optimism(state_sketch)
            .map_err(StaticCallError::ClientInitialization)?
            .execute(ContractInput::new_call(
                self.address,
                caller_address,
                calldata,
            ))
            .map_err(StaticCallError::ClientExecution)?;

        let decoded_contract_output =
            C::abi_decode_returns_validate(&cc_public_values.contractOutput)
                .map_err(StaticCallError::DecodeContractOutput)?;

        Ok((
            decoded_contract_output,
            cc_public_values.anchorHash.0.into(),
        ))
    }
}
