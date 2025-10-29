use std::cmp::Ordering;

use alloy_primitives::U256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UnclaimWithBlockNumber {
    /// Global index that got unclaimed.
    pub global_index: U256,
    /// The block number of this unclaim.
    pub block_number: u64,
    /// Index within that block in which a claim got unclaimed.
    pub log_index: u64,
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
            // If equal, compare by log_index
            .then_with(|| self.log_index.cmp(&other.log_index))
            // If still equal, compare by global_index
            .then_with(|| self.global_index.cmp(&other.global_index))
    }
}
