use agglayer_interop::types::{L1InfoTreeLeaf, MerkleProof};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct InsertedGerWithBlockNumber {
    // The block number of the ger.
    pub block_number: u64,
    // The insert ger.
    pub inserted_ger: InsertedGer,
    // The index of the injected GER event in block.
    pub log_index: u64,
}

impl PartialOrd for InsertedGerWithBlockNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InsertedGerWithBlockNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by block_number
        self.block_number
            .cmp(&other.block_number)
            // If equal, compare by log_index
            .then_with(|| self.log_index.cmp(&other.log_index))
            // If still equal, compare by l1_info_tree_index
            .then_with(|| {
                // Note - `MerkleProof`, `L1InfoTreeLeaf` in `InsertedGer` do not have `Ord`
                // implementation. We use `l1_info_tree_index` to differentiate
                // them.
                self.inserted_ger
                    .l1_leaf
                    .l1_info_tree_index
                    .cmp(&other.inserted_ger.l1_leaf.l1_info_tree_index)
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
