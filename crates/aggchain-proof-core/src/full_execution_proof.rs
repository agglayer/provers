use serde::{Deserialize, Serialize};
#[cfg(target_os = "zkvm")]
use sha2::{Digest as Sha256Digest, Sha256};
use sp1_zkvm::lib::utils::words_to_bytes_le;

use crate::{digest::keccak256_combine, digest::Digest, error::ProofError};

type Vkey = [u32; 8];

/// Public values to verify the FEP.
#[derive(Serialize, Deserialize)]
pub struct FepPublicValues {
    pub l1_head: Digest,
    pub l2_pre_root: Digest,
    pub claim_root: Digest,
    pub claim_block_num: u32,
    pub rollup_config_hash: Digest,
    pub range_vkey_commitment: Digest,
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

#[derive(Serialize, Deserialize)]
pub struct FepWithPublicValues {
    public_values: FepPublicValues,
    aggregation_vkey: Vkey,
}

impl FepWithPublicValues {
    /// Compute the chain-specific commitment forwarded to the PP.
    pub fn aggchain_params(&self) -> [u8; 32] {
        keccak256_combine([
            self.public_values.l1_head.as_slice(),
            self.public_values.l2_pre_root.as_slice(),
            self.public_values.claim_root.as_slice(),
            &self.public_values.claim_block_num.to_be_bytes(),
            self.public_values.rollup_config_hash.as_slice(),
            self.public_values.range_vkey_commitment.as_slice(),
            words_to_bytes_le(&self.aggregation_vkey).as_slice(),
        ])
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
