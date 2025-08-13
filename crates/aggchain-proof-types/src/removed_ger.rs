use std::cmp::Ordering;

use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct RemovedGerWithBlockNumber {
    /// Global exit root hash value.
    pub global_exit_root: Digest,
    /// The block number of the removed GER.
    pub block_number: u64,
    /// Index within the block where the GER got removed.
    pub block_index: u64,
}

impl PartialOrd for RemovedGerWithBlockNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RemovedGerWithBlockNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by block_number
        self.block_number
            .cmp(&other.block_number)
            // If equal, compare by block_index
            .then(self.block_index.cmp(&other.block_index))
    }
}
