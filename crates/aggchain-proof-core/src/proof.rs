use agglayer_primitives::Digest;
use serde::{Deserialize, Serialize};
use unified_bridge::{AggchainProofPublicValues, ImportedBridgeExitCommitmentVersion};

use crate::{
    bridge::{BridgeConstraintsInput, BridgeWitness, L2_GER_ADDR},
    error::ProofError,
    full_execution_proof::FepInputs,
};

/// Version of the commitment on the imported bridge exits.
pub const IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION: ImportedBridgeExitCommitmentVersion =
    ImportedBridgeExitCommitmentVersion::V3;

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
            origin_network: self.origin_network.into(),
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

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader, path::PathBuf};

    use agglayer_primitives::{Address, Digest};

    use super::AggchainProofWitness;
    use crate::{
        bridge::BridgeConstraintsInput,
        full_execution_proof::{FepInputs, KoalaBearDigest},
    };

    fn test_input(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/test_input")
            .join(name)
    }

    /// Builds a witness out of the committed bridge fixture and fixed FEP
    /// values. Everything but the bridge witness is synthetic and
    /// deterministic.
    fn fixture_witness() -> AggchainProofWitness {
        let reader = BufReader::new(File::open(test_input("bridge_constraints_input.json")).unwrap());
        let bridge: BridgeConstraintsInput = serde_json::from_reader(reader).unwrap();
        let first_ger = bridge.bridge_witness.inserted_gers[0].clone();
        let d = |b: u8| Digest([b; 32]);

        AggchainProofWitness {
            prev_local_exit_root: d(1),
            new_local_exit_root: bridge.new_local_exit_root,
            l1_info_root: bridge.l1_info_root,
            origin_network: 7,
            fep: FepInputs {
                l1_head: d(2),
                claim_block_num: 1234,
                rollup_config_hash: d(3),
                prev_state_root: d(4),
                prev_withdrawal_storage_root: d(5),
                prev_block_hash: bridge.prev_l2_block_hash,
                new_state_root: d(6),
                new_withdrawal_storage_root: d(7),
                new_block_hash: bridge.new_l2_block_hash,
                aggregation_vkey_hash: KoalaBearDigest([
                    2_000_000_001,
                    2_000_000_002,
                    2_000_000_003,
                    2_000_000_004,
                    2_000_000_005,
                    2_000_000_006,
                    2_000_000_007,
                    2_000_000_008,
                ]),
                range_vkey_commitment: [9u8; 32],
                trusted_sequencer: Address::new([0xab; 20]),
                signature_optimistic_mode: None,
                l1_info_tree_leaf: first_ger.l1_info_tree_leaf,
                l1_head_inclusion_proof: first_ger.proof,
            },
            commit_imported_bridge_exits: bridge.commit_imported_bridge_exits,
            bridge_witness: bridge.bridge_witness,
        }
    }

    /// Pins the public values the FEP program commits for the fixture witness
    /// (JSON of `AggchainProofPublicValues`, which is what the program
    /// bincode-serializes). Set `UPDATE_PUBLIC_VALUES_FIXTURE=1` to regenerate
    /// `src/test_input/fep_public_values.json`.
    #[test]
    fn fep_public_values_are_byte_stable() {
        let witness = fixture_witness();
        let bytes = serde_json::to_vec_pretty(&witness.public_values()).unwrap();

        let path = test_input("fep_public_values.json");
        if std::env::var_os("UPDATE_PUBLIC_VALUES_FIXTURE").is_some() {
            std::fs::write(&path, &bytes).unwrap();
        }
        assert_eq!(
            bytes,
            std::fs::read(&path).unwrap(),
            "FEP public values changed"
        );
    }
}
