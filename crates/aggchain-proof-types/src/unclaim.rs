use std::cmp::Ordering;

use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UnclaimWithBlockNumber {
    /// Bridge exit claim that got unclaimed hash value.
    pub unclaim_hash: Digest,
    /// The block number of an unclaim.
    pub block_number: u64,
    /// Index within that block in which a claim got unclaimed.
    pub block_index: u64,
}

impl PartialOrd for UnclaimWithBlockNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UnclaimWithBlockNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by block_number
        self.block_number
            .cmp(&other.block_number)
            // If equal, compare by block_index
            .then(self.block_index.cmp(&other.block_index))
    }
}
