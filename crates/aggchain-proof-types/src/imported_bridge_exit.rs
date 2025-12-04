use std::cmp::Ordering;

use agglayer_interop::types::GlobalIndex;
use serde::{Deserialize, Serialize};

// TODO: move this to interop repository
// https://github.com/agglayer/provers/issues/141
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BridgeExitHash(pub agglayer_interop::types::Digest);

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct ImportedBridgeExitWithBlockNumber {
    pub block_number: u64,
    pub bridge_exit_hash: BridgeExitHash,
    pub global_index: GlobalIndex,
    pub log_index: u64,
}

impl PartialOrd for ImportedBridgeExitWithBlockNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ImportedBridgeExitWithBlockNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        self.block_number
            // First compare by block_number
            .cmp(&other.block_number)
            // If equal, compare by log_index
            .then_with(|| self.log_index.cmp(&other.log_index))
            // If still equal, compare by bridge_exit_hash
            .then_with(|| self.bridge_exit_hash.cmp(&other.bridge_exit_hash))
    }
}
