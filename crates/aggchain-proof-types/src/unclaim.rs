use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnclaimWithBlockNumber {
    /// Unclaim hash value.
    pub unclaim_hash: Digest,
    /// The block number of an unclaim.
    pub block_number: u64,
    /// Index within that block in which a claim got unclaimed.
    pub block_index: u64,
}
