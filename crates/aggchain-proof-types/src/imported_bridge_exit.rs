use agglayer_interop::types::ImportedBridgeExit;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ImportedBridgeExitWithBlockNumber {
    pub block_number: u64,
    pub imported_bridge_exit: ImportedBridgeExit,
}
