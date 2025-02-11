use alloy_primitives::keccak256;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
#[cfg(target_os = "zkvm")]
use sha2::{Digest as Sha256Digest, Sha256};
use sp1_zkvm::lib::utils::words_to_bytes_le;

use crate::{error::ProofError, keccak::digest::Digest, keccak::keccak256_combine};

type Vkey = [u32; 8];

/// Public values to verify the FEP.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FepPublicValues {
    pub l1_head: Digest,
    pub l2_pre_root: Digest,
    pub claim_root: Digest,
    pub claim_block_num: u32,
    pub rollup_config_hash: Digest,
    pub range_vkey_commitment: Digest,
    #[serde(with = "BigArray")]
    pub l2_pre_root_bytes: [u8; 128],
    #[serde(with = "BigArray")]
    pub claim_root_bytes: [u8; 128],
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
            self.public_values.l2_pre_root.as_slice(),
            self.public_values.claim_root.as_slice(),
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
    // return the blockhashes given the l2_pre_root and claim_root
    // Follow this encoding: https://github.com/op-rs/kona/blob/161547c73aa326a924b79cca5d811a202c5c45a0/crates/proof/executor/src/executor/mod.rs#L448-L453
    pub fn get_block_hashes(&self) -> Result<(Digest, Digest), ProofError> {
        let computed_l2_pre_root: alloy_primitives::FixedBytes<32> =
            keccak256(self.public_values.l2_pre_root_bytes);
        if computed_l2_pre_root != self.public_values.l2_pre_root.0 {
            return Err(ProofError::InvalidBlockHash("l2_pre_root mismatch".into()));
        }
        let computed_claim_root = keccak256(self.public_values.claim_root_bytes);
        if computed_claim_root != self.public_values.claim_root.0 {
            return Err(ProofError::InvalidBlockHash("claim_root mismatch".into()));
        }

        let prev_block_hash: [u8; 32] = self.public_values.l2_pre_root_bytes[96..128]
            .try_into()
            .expect("slice with correct length");
        let new_block_hash: [u8; 32] = self.public_values.claim_root_bytes[96..128]
            .try_into()
            .expect("slice with correct length");

        Ok((prev_block_hash.into(), new_block_hash.into()))
    }
}
