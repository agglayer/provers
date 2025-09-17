use std::cmp::Ordering;

use agglayer_interop::types::{L1InfoTreeLeaf, MerkleProof};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct InsertedGerWithBlockNumber {
    // The block number of the ger.
    pub block_number: u64,
    // The insert ger.
    pub inserted_ger: InsertedGer,
    // The index of the injected GER event in block.
    pub block_index: u64,
}

impl PartialOrd for InsertedGerWithBlockNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InsertedGerWithBlockNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.block_number.cmp(&other.block_number).then_with(|| {
            let ordering = self.block_index.cmp(&other.block_index);
            // Assert that if block_number and block_index are equal,
            // then inserted_ger should also be equal to maintain Ord guarantees.
            assert!(
                ordering != Ordering::Equal || self.inserted_ger == other.inserted_ger,
                "Items with same block_number and block_index must have same inserted_ger"
            );
            ordering
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct InsertedGer {
    // Proof from GER to L1Root
    pub proof_ger_l1root: MerkleProof,
    // L1InfoTree leaf
    pub l1_leaf: L1InfoTreeLeaf,
}
