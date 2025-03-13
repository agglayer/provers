use aggchain_proof_core::{Digest, AGGCHAIN_PROOF_PROGRAM_VERSION, AGGCHAIN_TYPE};
use alloy_primitives::U256;
use bincode::Options;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug)]
// TODO: Making this unused as it will be used in another iteration
#[allow(unused)]
pub struct VKeySelector([u8; 4]);

impl VKeySelector {
    pub const fn new(program: u16, aggchain_type: u16) -> Self {
        VKeySelector((((program as u32) << 16) | aggchain_type as u32).to_be_bytes())
    }

    #[cfg(test)]
    pub fn to_be_bytes(&self) -> [u8; 4] {
        self.0
    }
}

// TODO: Making this unused as it will be used in another iteration
#[allow(unused)]
const AGGCHAIN_VKEY_SELECTOR: VKeySelector =
    VKeySelector::new(AGGCHAIN_PROOF_PROGRAM_VERSION, AGGCHAIN_TYPE);

#[derive(Serialize, Deserialize)]
pub(crate) struct CustomChainData {
    selector: u16,
    output_root: Digest,
    l2_block_number: [u8; U256::BYTES],
}

pub fn compute_custom_chain_data(
    output_root: Digest,
    l2_block_number: u64,
) -> Result<Vec<u8>, bincode::Error> {
    bincode::DefaultOptions::default()
        .with_fixint_encoding()
        .with_big_endian()
        .serialize(&CustomChainData {
            selector: AGGCHAIN_PROOF_PROGRAM_VERSION,
            output_root,
            l2_block_number: U256::from(l2_block_number).to_be_bytes(),
        })
}
