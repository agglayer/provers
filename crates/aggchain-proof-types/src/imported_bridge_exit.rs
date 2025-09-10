use std::cmp::Ordering;

use agglayer_interop::types::GlobalIndex;
use serde::{Deserialize, Serialize};

// TODO: move this to interop repository
// https://github.com/agglayer/provers/issues/141
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeExitHash(pub agglayer_interop::types::Digest);

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct ImportedBridgeExitWithBlockNumber {
    pub block_number: u64,
    pub bridge_exit_hash: BridgeExitHash,
    pub global_index: GlobalIndex,
}

impl PartialOrd for ImportedBridgeExitWithBlockNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ImportedBridgeExitWithBlockNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.block_number.cmp(&other.block_number).then_with(|| {
            let ordering = self.global_index.cmp(&other.global_index);
            // Debug assert that if block_number and global_index are equal,
            // then bridge_exit_hash should also be equal to maintain Ord guarantees.
            assert!(
                ordering != Ordering::Equal || self.bridge_exit_hash == other.bridge_exit_hash,
                "Items with same block_number and global_index must have the same bridge_exit_hash"
            );
            ordering
        })
    }
}
