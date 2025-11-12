use agglayer_interop::types::GlobalIndex;
use serde::{Deserialize, Serialize};

// TODO: move this to interop repository
// https://github.com/agglayer/provers/issues/141
#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct BridgeExitHash(pub agglayer_interop::types::Digest);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ImportedBridgeExitWithBlockNumber {
    pub block_number: u64,
    pub bridge_exit_hash: BridgeExitHash,
    pub global_index: GlobalIndex,
}
