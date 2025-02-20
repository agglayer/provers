use serde::{Deserialize, Serialize};

use crate::{
    bridge::{BridgeInput, BridgeWitness, L2_GER_ADDR},
    error::ProofError,
    full_execution_proof::FepWithPublicValues,
    keccak::digest::Digest,
};

/// Aggchain proof is generated from the FEP proof and additional
/// bridge information.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AggchainProof {
    //pub proof: SP1ProofWithPublicValues,
    //TODO add all necessary fields
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AggchainProofWitness {
    /// Previous local exit root.
    pub prev_local_exit_root: Digest,
    /// New local exit root.
    pub new_local_exit_root: Digest,
    /// Commitment to the imported bridge exits indexes.
    pub commit_imported_bridge_exits: Digest,
    /// L1 info root used to import bridge exits.
    pub l1_info_root: Digest,
    /// Origin network for which the proof was generated.
    pub origin_network: u32,
    /// Full execution proof with its metadata.
    pub fep: FepWithPublicValues,
    /// Bridge witness related data.
    pub bridge_witness: BridgeWitness,
}

impl AggchainProofWitness {
    pub fn generate_aggchain_proof(&mut self) -> Result<AggchainProofPublicValues, ProofError> {
        let public_values = self.public_values();

        // Verify the FEP exclusively within the SP1 VM
        #[cfg(target_os = "zkvm")]
        self.fep.verify()?;

        // Verify the bridge constraints
        self.generate_bridge_input()?.verify()?;

        Ok(public_values)
    }
}

impl AggchainProofWitness {
    pub fn public_values(&self) -> AggchainProofPublicValues {
        AggchainProofPublicValues {
            prev_local_exit_root: self.prev_local_exit_root,
            new_local_exit_root: self.new_local_exit_root,
            l1_info_root: self.l1_info_root,
            origin_network: self.origin_network,
            commit_imported_bridge_exits: self.commit_imported_bridge_exits,
            aggchain_params: self.fep.aggchain_params().into(),
        }
    }
}

impl AggchainProofWitness {
    pub fn generate_bridge_input(&mut self) -> Result<BridgeInput, ProofError> {
        let (prev_blockhash, new_blockhash) = self.fep.get_block_hashes()?;

        Ok(BridgeInput {
            ger_addr: L2_GER_ADDR, // set as constant for now
            prev_l2_block_hash: alloy_primitives::FixedBytes::from(prev_blockhash),
            new_l2_block_hash: alloy_primitives::FixedBytes::from(new_blockhash),
            new_local_exit_root: alloy_primitives::FixedBytes::from(&self.new_local_exit_root.0),
            l1_info_root: alloy_primitives::FixedBytes::from(&self.l1_info_root.0),
            bridge_witness: self.bridge_witness.clone(),
        })
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

/// Leaf tree inclusion proof.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InclusionProof {
    pub siblings: Vec<Digest>,
}

/// L1 info tree leaf, part of the
/// L1 info tree.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct L1InfoTreeLeaf {
    /// Previous block hash of leaf.
    pub previous_block_hash: Digest,
    /// Block number timestamp.
    pub timestamp: u64,
    /// Mainnet exit root hash.
    pub mainnet_exit_root_hash: Digest,
    /// Rollup exit root hash.
    pub rollup_exit_root_hash: Digest,
    /// Global exit root hash.
    pub global_exit_root_hash: Digest,
    /// Leaf hash.
    pub leaf_hash: Digest,
    /// Leaf index.
    pub l1_info_tree_index: u32,
}
