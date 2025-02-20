use serde::{Deserialize, Serialize};
#[cfg(target_os = "zkvm")]
use sha2::{Digest as Sha256Digest, Sha256};
use sp1_zkvm::lib::utils::words_to_bytes_le;

use crate::{error::ProofError, keccak::digest::Digest, keccak::keccak256_combine};

type Vkey = [u32; 8];

// Hardcoded for now, might see if we might need it as input
pub const OUTPUT_ROOT_VERSION: [u8; 32] = [0; 32];

/// Public values to verify the FEP.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FepPublicValues {
    pub l1_head: Digest,
    pub claim_block_num: u32,
    pub rollup_config_hash: Digest,
    pub range_vkey_commitment: Digest,
    pub prev_state_root: [u8; 32],
    pub prev_withdrawal_storage_root: [u8; 32],
    pub prev_block_hash: [u8; 32],
    pub new_state_root: [u8; 32],
    pub new_withdrawal_storage_root: [u8; 32],
    pub new_block_hash: [u8; 32],
}

#[cfg(target_os = "zkvm")]
impl FepPublicValues {
    pub fn hash(&self) -> [u8; 32] {
        let public_values = [
            self.l1_head.as_slice(),
            self.l2_pre_root.as_slice(),
            self.claim_root.as_slice(),
            &self.claim_block_num.to_be_bytes(),
            self.rollup_config_hash.as_slice(),
            self.range_vkey_commitment.as_slice(),
        ]
        .concat();

        Sha256::digest(&public_values).into()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FepWithPublicValues {
    pub(crate) public_values: FepPublicValues,
    pub(crate) aggregation_vkey: Vkey,
}

impl FepWithPublicValues {
    /// Compute the chain-specific commitment forwarded to the PP.
    pub fn aggchain_params(&self) -> [u8; 32] {
        keccak256_combine([
            self.public_values.l1_head.as_slice(),
            self.compute_l2_pre_root_bytes().as_slice(),
            self.computed_claim_root_bytes().as_slice(),
            &self.public_values.claim_block_num.to_be_bytes(),
            self.public_values.rollup_config_hash.as_slice(),
            self.public_values.range_vkey_commitment.as_slice(),
            words_to_bytes_le(&self.aggregation_vkey).as_slice(),
        ])
        .0
    }

    /// Verify the SP1 proof
    pub fn verify(&self) -> Result<(), ProofError> {
        #[cfg(not(target_os = "zkvm"))]
        unreachable!("verify_sp1_proof is not callable outside of SP1");

        #[cfg(target_os = "zkvm")]
        {
            sp1_zkvm::lib::verify::verify_sp1_proof(
                &self.aggregation_vkey,
                &self.public_values.hash().into(),
            );

            return Ok(());
        }
    }
}

impl FepWithPublicValues {
    // Follow this encoding: https://github.com/op-rs/kona/blob/161547c73aa326a924b79cca5d811a202c5c45a0/crates/proof/executor/src/executor/mod.rs#L448-L453
    pub fn compute_l2_pre_root_bytes(&self) -> [u8; 32] {
        keccak256_combine([
            OUTPUT_ROOT_VERSION,
            self.public_values.prev_state_root,
            self.public_values.prev_withdrawal_storage_root,
            self.public_values.prev_block_hash,
        ])
        .0
    }

    pub fn computed_claim_root_bytes(&self) -> [u8; 32] {
        keccak256_combine([
            OUTPUT_ROOT_VERSION,
            self.public_values.new_state_root,
            self.public_values.new_withdrawal_storage_root,
            self.public_values.new_block_hash,
        ])
        .0
    }

    pub fn get_block_hashes(&self) -> Result<([u8; 32], [u8; 32]), ProofError> {
        Ok((self.public_values.prev_block_hash, self.public_values.new_block_hash))
    }
}
