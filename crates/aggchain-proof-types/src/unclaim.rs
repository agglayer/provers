use std::cmp::Ordering;

use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UnclaimWithBlockNumber {
    /// Hash of the claim (bridge_exit + global_index) that got unclaimed.
    pub unclaim_hash: Digest,
    /// The block number of this unclaim.
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
            .then_with(|| {
                let ordering = self.block_index.cmp(&other.block_index);
                // Debug assert that if block_number and block_index are equal,
                // then unclaim_hash should also be equal to maintain Ord guarantees.
                debug_assert!(
                    ordering != Ordering::Equal || self.unclaim_hash == other.unclaim_hash,
                    "Items with same block_number and block_index must have same unclaim_hash"
                );
                ordering
            })
    }
}
