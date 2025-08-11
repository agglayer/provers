use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RemovedGerWithBlockNumber {
    /// Global exit root digest value.
    pub global_exit_root: Digest,
    /// The block number of the removed GER.
    pub block_number: u64,
    /// Index within the block where the GER got removed.
    pub block_index: u64,
}
