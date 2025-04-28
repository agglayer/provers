use aggchain_proof_core::{Digest, AGGCHAIN_PROOF_PROGRAM_VERSION, AGGCHAIN_TYPE};
use alloy_primitives::U256;
use alloy_sol_types::{sol, SolValue};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct VKeySelector([u8; 4]);

impl VKeySelector {
    pub const fn new(program: u16, aggchain_type: u16) -> Self {
        VKeySelector((((program as u32) << 16) | aggchain_type as u32).to_be_bytes())
    }

    pub fn to_be_bytes(&self) -> [u8; 4] {
        self.0
    }
}

// TODO: Making this unused as it will be used in another iteration
pub const AGGCHAIN_VKEY_SELECTOR: VKeySelector =
    VKeySelector::new(AGGCHAIN_PROOF_PROGRAM_VERSION, AGGCHAIN_TYPE);

sol! {
    struct CustomChainData {
        bytes4 selector;
        bytes32 output_root;
        bytes32 l2_block_number;
    }
}

pub fn compute_custom_chain_data(output_root: Digest, l2_block_number: u64) -> Vec<u8> {
    CustomChainData {
        selector: AGGCHAIN_VKEY_SELECTOR.to_be_bytes().into(),
        output_root: output_root.0.into(),
        l2_block_number: U256::from(l2_block_number).to_be_bytes().into(),
    }
    .abi_encode()
}
