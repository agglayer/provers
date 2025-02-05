use serde::{Deserialize, Serialize};

use crate::{digest::Digest, error::ProofError, full_execution_proof::FepWithPublicValues};

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
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
