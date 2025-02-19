use alloy_primitives::{Address, FixedBytes};
use serde::{Deserialize, Serialize};
use sp1_cc_client_executor::io::EVMStateSketch;

use crate::inserted_ger::InsertedGER;

/// Represents all the bridge constraints errors.
#[derive(Clone, thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BridgeConstraintsError {
    /// The inclusion proof from the GER to the L1 info Root is invalid.
    #[error("Invalid merkle path from the GER to the L1 Info Root.")]
    InvalidMerklePathGERToL1Root,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeWitness {
    pub injected_gers: Vec<InsertedGER>,
    pub prev_hash_chain_sketch: EVMStateSketch,
    pub new_hash_chain_sketch: EVMStateSketch,
    pub new_ler_sketch: EVMStateSketch,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct BridgeInput {
    pub ger_addr: Address,
    pub prev_l2_block_hash: FixedBytes<32>,
    pub new_l2_block_hash: FixedBytes<32>,
    pub new_local_exit_root: FixedBytes<32>,
    pub l1_info_root: FixedBytes<32>,
    pub bridge_witness: BridgeWitness,
}

impl BridgeInput {
    pub fn verify(&mut self) -> Result<(), BridgeConstraintsError> {
        self.verify_inserted_gers()?;
        todo!()
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
