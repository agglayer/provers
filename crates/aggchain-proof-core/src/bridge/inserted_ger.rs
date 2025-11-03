use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};
use unified_bridge::{L1InfoTreeLeaf, MerkleProof};

/// Data to verify the legitimacy of one inserted GER.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertedGER {
    /// Merkle proof against one L1 info root.
    pub proof: MerkleProof,
    /// L1 info tree leaf to reconstruct the leaf hash.
    pub l1_info_tree_leaf: L1InfoTreeLeaf,
    /// Block number in which the GER got inserted.
    pub block_number: u64,
    /// Index within the block.
    pub log_index: u64,
}

impl InsertedGER {
    /// Verify the inclusion proof against one L1 info root.
    pub fn verify(&self, l1_info_root: Digest) -> bool {
        // TODO: return differentiated errors
        if l1_info_root != self.proof.root {
            return false;
        }

        self.proof.verify(
            self.l1_info_tree_leaf.hash(),
            self.l1_info_tree_leaf.l1_info_tree_index,
        )
    }

    /// Returns the inserted GER.
    pub fn ger(&self) -> Digest {
        self.l1_info_tree_leaf.ger()
    }
}
