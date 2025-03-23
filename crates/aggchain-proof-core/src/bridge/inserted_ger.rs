use agglayer_interop::types::{L1InfoTreeLeaf, MerkleProof};
use agglayer_primitives::digest::Digest;
use serde::{Deserialize, Serialize};

/// Data to verify the legitimacy of one inserted GER.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertedGER {
    /// Merkle proof against one L1 info root.
    pub proof: MerkleProof,
    /// L1 info tree leaf to reconstruct the leaf hash.
    pub l1_info_tree_leaf: L1InfoTreeLeaf,
}

impl InsertedGER {
    /// Verify the inclusion proof against one L1 info root.
    pub fn verify(&self, l1_info_root: Digest) -> bool {
        let valid_merkle_proof = self.proof.verify(
            self.l1_info_tree_leaf.hash(),
            self.l1_info_tree_leaf.l1_info_tree_index,
        );

        // TODO: return differentiated errors
        (l1_info_root == self.proof.root) && valid_merkle_proof
    }

    /// Returns the inserted GER.
    pub fn ger(&self) -> Digest {
        self.l1_info_tree_leaf.inner.global_exit_root
    }
}
