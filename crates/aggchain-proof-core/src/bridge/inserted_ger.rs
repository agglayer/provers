use serde::{Deserialize, Serialize};

use crate::Digest;
use crate::{
    keccak::keccak256_combine,
    local_exit_tree::{hasher::Keccak256Hasher, proof::LETMerkleProof},
};

impl L1InfoTreeLeaf {
    /// Hashes the L1 Info Tree leaf.
    pub fn hash(&self) -> Digest {
        keccak256_combine([
            self.global_exit_root.as_slice(),
            self.block_hash.as_slice(),
            &self.timestamp.to_be_bytes(),
        ])
    }
}

/// Contents of one leaf of the L1 Info Tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1InfoTreeLeaf {
    pub(crate) global_exit_root: Digest,
    pub(crate) block_hash: Digest,
    pub(crate) timestamp: u64,
}

/// Data to verify the legitimacy of one inserted GER.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertedGER {
    /// Merkle proof against one L1 info root.
    pub(crate) proof: LETMerkleProof<Keccak256Hasher>,
    /// L1 info tree leaf to reconstruct the leaf hash.
    pub(crate) l1_info_tree_leaf: L1InfoTreeLeaf,
    /// Index of the leaf in the L1 info tree.
    pub(crate) l1_info_tree_index: u32,
}

impl InsertedGER {
    /// Verify the inclusion proof against one L1 info root.
    pub fn verify(&self, l1_info_root: Digest) -> bool {
        self.proof.verify(
            self.l1_info_tree_leaf.hash(),
            self.l1_info_tree_index,
            l1_info_root,
        )
    }

    /// Returns the inserted GER.
    pub fn ger(&self) -> Digest {
        self.l1_info_tree_leaf.global_exit_root
    }
}
