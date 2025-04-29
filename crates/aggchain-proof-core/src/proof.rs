use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};
use unified_bridge::{aggchain_proof::AggchainProofPublicValues, CommitmentVersion};

use crate::{
    bridge::{BridgeConstraintsInput, BridgeWitness, L2_GER_ADDR},
    error::ProofError,
    full_execution_proof::FepInputs,
};

/// Version of the commitment on the imported bridge exits.
pub const IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION: CommitmentVersion = CommitmentVersion::V3;

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
    pub fep: FepInputs,
    /// Commitment on the imported bridge exits minus the unset ones.
    pub commit_imported_bridge_exits: Digest,
    /// Bridge witness related data.
    pub bridge_witness: BridgeWitness,
}

impl AggchainProofWitness {
    pub fn verify_aggchain_inputs(&self) -> Result<AggchainProofPublicValues, ProofError> {
        // Verify the FEP proof or ECDSA signature.
        self.fep.verify(
            self.l1_info_root,
            self.new_local_exit_root,
            self.commit_imported_bridge_exits,
        )?;

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
            commit_imported_bridge_exits: self.commit_imported_bridge_exits,
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
            commit_imported_bridge_exits: self.commit_imported_bridge_exits,
            bridge_witness: self.bridge_witness.clone(),
        }
    }
}
