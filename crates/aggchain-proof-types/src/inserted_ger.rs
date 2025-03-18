use agglayer_interop::types::{L1InfoTreeLeaf, MerkleProof};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InsertedGerWithBlockNumber {
    // The block number of the ger.
    pub block_number: u64,
    // The insert ger.
    pub inserted_ger: InsertedGer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InsertedGer {
    // Proof from GER to L1Root
    pub proof_ger_l1root: MerkleProof,
    // L1InfoTree leaf
    pub l1_leaf: L1InfoTreeLeaf,
}
