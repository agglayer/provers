use agglayer_interop::types::{L1InfoTreeLeaf, MerkleProof};
use agglayer_primitives::digest::Digest;
use alloy_primitives::{keccak256, B256};
use serde::{Deserialize, Serialize};

use crate::{
    bridge::{BridgeConstraintsInput, BridgeWitness, L2_GER_ADDR},
    error::ProofError,
    full_execution_proof::FepPublicValues,
    keccak256_combine,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AggchainProofWitness {
    /// Previous local exit root.
    pub prev_local_exit_root: Digest,
    /// New local exit root.
    pub new_local_exit_root: Digest,
    /// L1 info root used to import bridge exits.
    pub l1_info_root: Digest,
    /// Origin network for which the proof was generated.
    pub origin_network: u32,
    /// Full execution proof with its metadata.
    pub fep: FepPublicValues,
    /// L1 info tree leaf containing the `l1Head` as block hash.
    pub l1_info_tree_leaf: L1InfoTreeLeaf,
    /// Inclusion proof of the leaf to the l1 info root.
    pub l1_head_inclusion_proof: MerkleProof,
    /// List of the global index of each imported bridge exit.
    pub global_indices: Vec<B256>,
    /// Bridge witness related data.
    pub bridge_witness: BridgeWitness,
}

impl AggchainProofWitness {
    pub fn verify_aggchain_inputs(&self) -> Result<AggchainProofPublicValues, ProofError> {
        // Verify the FEP proof or ECDSA signature.
        self.fep.verify()?;

        // Verify that the `l1Head` considered by the FEP exists in the L1 Info Tree
        {
            if self.fep.l1_head != self.l1_info_tree_leaf.inner.block_hash {
                return Err(ProofError::MismatchL1Head {
                    from_l1_info_tree_leaf: self.l1_info_tree_leaf.inner.block_hash,
                    from_fep_public_values: self.fep.l1_head,
                });
            }

            let inclusion_proof_valid = self.l1_head_inclusion_proof.verify(
                self.l1_info_tree_leaf.hash(),
                self.l1_info_tree_leaf.l1_info_tree_index,
            );

            // TODO: proper error
            if !(inclusion_proof_valid && self.l1_info_root == self.l1_head_inclusion_proof.root) {
                return Err(ProofError::InvalidInclusionProofL1Head {
                    index: self.l1_info_tree_leaf.l1_info_tree_index,
                    l1_leaf_hash: self.l1_info_tree_leaf.hash(),
                    l1_info_root: self.l1_info_root,
                });
            }
        }

        // Verify the bridge constraints
        self.bridge_constraints_input().verify()?;

        Ok(self.public_values())
    }
}

impl AggchainProofWitness {
    pub fn public_values(&self) -> AggchainProofPublicValues {
        AggchainProofPublicValues {
            prev_local_exit_root: self.prev_local_exit_root,
            new_local_exit_root: self.new_local_exit_root,
            l1_info_root: self.l1_info_root,
            origin_network: self.origin_network,
            commit_imported_bridge_exits: keccak256_combine(
                self.global_indices
                    .iter()
                    .map(|idx| keccak256(idx.as_slice())),
            ),
            aggchain_params: self.fep.aggchain_params(),
        }
    }
}

impl AggchainProofWitness {
    pub fn bridge_constraints_input(&self) -> BridgeConstraintsInput {
        BridgeConstraintsInput {
            ger_addr: L2_GER_ADDR, // set as constant for now
            prev_l2_block_hash: self.fep.prev_block_hash,
            new_l2_block_hash: self.fep.new_block_hash,
            new_local_exit_root: self.new_local_exit_root,
            l1_info_root: self.l1_info_root,
            global_indices: self.global_indices.clone(),
            bridge_witness: self.bridge_witness.clone(),
        }
    }
}

/// Public values of the SP1 aggchain proof.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AggchainProofPublicValues {
    /// Previous local exit root.
    pub prev_local_exit_root: Digest,
    /// New local exit root.
    pub new_local_exit_root: Digest,
    /// L1 info root used to import bridge exits.
    pub l1_info_root: Digest,
    /// Origin network for which the proof was generated.
    pub origin_network: u32,
    /// Commitment to the imported bridge exits indexes.
    pub commit_imported_bridge_exits: Digest,
    /// Chain-specific commitment forwarded by the PP.
    pub aggchain_params: Digest,
}
