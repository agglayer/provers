use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};
use unified_bridge::{L1InfoTreeLeaf, MerkleProof};

/// Errors that can occur during GER verification
#[derive(Debug, thiserror::Error)]
pub enum GerVerificationError {
    /// The provided L1 info root does not match the proof root
    #[error("L1 info root mismatch: expected {expected:?}, got {got:?}")]
    L1InfoRootMismatch { expected: Digest, got: Digest },
    
    /// The Merkle proof verification failed
    #[error("Merkle proof verification failed for leaf hash {leaf_hash:?} at index {index}")]
    MerkleProofVerificationFailed { leaf_hash: Digest, index: u32 },
}

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
    pub block_index: u64,
}

impl InsertedGER {
    /// Verify the inclusion proof against one L1 info root.
    pub fn verify(&self, l1_info_root: Digest) -> Result<(), GerVerificationError> {
        if l1_info_root != self.proof.root {
            return Err(GerVerificationError::L1InfoRootMismatch {
                expected: self.proof.root,
                got: l1_info_root,
            });
        }

        let leaf_hash = self.l1_info_tree_leaf.hash();
        let index = self.l1_info_tree_leaf.l1_info_tree_index;
        
        if !self.proof.verify(leaf_hash, index) {
            return Err(GerVerificationError::MerkleProofVerificationFailed {
                leaf_hash,
                index,
            });
        }

        Ok(())
    }

    /// Returns the inserted GER.
    pub fn ger(&self) -> Digest {
        self.l1_info_tree_leaf.ger()
    }
}
