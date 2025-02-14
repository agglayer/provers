use serde::{Deserialize, Serialize};

use crate::{digest::Digest, error::ProofError, full_execution_proof::FepWithPublicValues};

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
    /// Bridge constraints related data
    pub bridge: BridgeData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BridgeData;

impl BridgeData {
    pub fn verify(&mut self) -> Result<(), ProofError> {
        todo!()
    }
}

impl AggchainProofWitness {
    pub fn generate_aggchain_proof(&mut self) -> Result<AggchainProofPublicValues, ProofError> {
        // Verify the FEP exclusively within the SP1 VM
        #[cfg(target_os = "zkvm")]
        self.fep.verify()?;

        // Verify the bridge constraints
        self.bridge.verify()?;

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
            commit_imported_bridge_exits: self.commit_imported_bridge_exits,
            aggchain_params: self.fep.aggchain_params(),
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
