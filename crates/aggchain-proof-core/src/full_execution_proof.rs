use serde::{Deserialize, Serialize};
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

#[allow(unused)]
impl FepPublicValues {
    pub fn hash(&self) -> [u8; 32] {
        let public_values = [
            self.l1_head.as_slice(),
            self.compute_l2_pre_root_bytes().as_slice(),
            self.computed_claim_root_bytes().as_slice(),
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
            self.public_values.compute_l2_pre_root_bytes().as_slice(),
            self.public_values.computed_claim_root_bytes().as_slice(),
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

impl FepPublicValues {
    // Compute l2 pre root
    pub fn compute_l2_pre_root_bytes(&self) -> [u8; 32] {
        compute_output_root(
            self.new_state_root,
            self.new_withdrawal_storage_root,
            self.new_block_hash,
        )
    }

    // Compute claim root
    pub fn computed_claim_root_bytes(&self) -> [u8; 32] {
        compute_output_root(
            self.new_state_root,
            self.new_withdrawal_storage_root,
            self.new_block_hash,
        )
    }

    // This function should always be implemented regardless the FEP, since it's
    // used by the bridge proof Return the previous and new block hashes
    pub fn get_block_hashes(&self) -> Result<([u8; 32], [u8; 32]), ProofError> {
        Ok((self.prev_block_hash, self.new_block_hash))
    }
}

// Compute output root as defined here:
// https://specs.optimism.io/protocol/proposals.html#l2-output-commitment-construction
pub fn compute_output_root(
    new_state_root: [u8; 32],
    new_withdrawal_storage_root: [u8; 32],
    new_block_hash: [u8; 32],
) -> [u8; 32] {
    keccak256_combine([
        OUTPUT_ROOT_VERSION,
        new_state_root,
        new_withdrawal_storage_root,
        new_block_hash,
    ])
    .0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_output_root_expected_value() {
        // Provided inputs from the rpc endpoint: optimism_outputAtBlock
        let state_hex = "0xc82b7f91a1c9e78463653c6ec44a579062426d71d3404325fa5f129615e0473d";
        let withdrawal_hex = "0x8ed4baae3a927be3dea54996b4d5899f8c01e7594bf50b17dc1e741388ce3d12";
        let block_hash_hex = "0x61438199094c9db8d5c154034de9940712805469459346ed1b4e0fa57da5519b";
        let expected_output_root_hex =
            "0x720311395abb5216bee64000575e07dd3b64847b9f88d4d77b64e6aa28fc93a2";

        let state = hex_str_to_array(state_hex);
        let withdrawal = hex_str_to_array(withdrawal_hex);
        let block_hash = hex_str_to_array(block_hash_hex);
        let expected_output_root = hex_str_to_array(expected_output_root_hex);

        let computed_output_root = compute_output_root(state, withdrawal, block_hash);
        assert_eq!(
            computed_output_root, expected_output_root,
            "compute_output_root should return the expected hash"
        );
    }

    fn hex_str_to_array(s: &str) -> [u8; 32] {
        let s = s.trim_start_matches("0x");
        let bytes = hex::decode(s).expect("Decoding hex string failed");
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        array
    }
}
