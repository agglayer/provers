use agglayer_primitives::{
    keccak::{keccak256, keccak256_combine},
    Digest,
};
use alloy_primitives::{Address, FixedBytes, B256, U256};
use alloy_sol_types::{sol, SolValue};
use p3_baby_bear::BabyBear;
use p3_bn254_fr::Bn254Fr;
use p3_field::{AbstractField, PrimeField, PrimeField32};
use serde::{Deserialize, Serialize};
use sha2::{Digest as Sha256Digest, Sha256};
use unified_bridge::{L1InfoTreeLeaf, MerkleProof};

use crate::{error::ProofError, vkey_hash::HashU32};

/// Hardcoded for now, might see if we might need it as input
pub const OUTPUT_ROOT_VERSION: [u8; 32] = [0u8; 32];

/// L2PreRoot is the representation of the previous OutputRoot
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct L2PreRoot(pub Digest);

impl From<L2PreRoot> for FixedBytes<32> {
    fn from(value: L2PreRoot) -> FixedBytes<32> {
        value.0.as_bytes().into()
    }
}

/// ClaimRoot is the hash of the concatenation of the OutputRoot version +
/// payload
///
/// Payload composed of `state_root`, `withdrawal_storage_root`,
/// `latest_block_hash`
///
/// Details: https://specs.optimism.io/protocol/proposals.html#l2-output-commitment-construction
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClaimRoot(pub Digest);

impl From<ClaimRoot> for FixedBytes<32> {
    fn from(value: ClaimRoot) -> FixedBytes<32> {
        value.0.as_bytes().into()
    }
}

impl From<ClaimRoot> for L2PreRoot {
    fn from(value: ClaimRoot) -> L2PreRoot {
        L2PreRoot(value.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BabyBearDigest(pub [BabyBear; 8]);

impl BabyBearDigest {
    pub fn to_hash_u32(&self) -> HashU32 {
        self.0.map(|n| n.as_canonical_u32())
    }

    pub fn to_hash_bn254(&self) -> [u8; 32] {
        let vkey_digest_bn254: Bn254Fr = {
            let mut result = Bn254Fr::zero();
            for word in self.0.iter() {
                // Since BabyBear prime is less than 2^31, we can shift by 31 bits each time and
                // still be within the Bn254Fr field, so we don't have to
                // truncate the top 3 bits.
                result *= Bn254Fr::from_canonical_u64(1 << 31);
                result += Bn254Fr::from_canonical_u32(word.as_canonical_u32());
            }
            result
        };
        let vkey_bytes = vkey_digest_bn254.as_canonical_biguint().to_bytes_be();
        let mut result = [0u8; 32];
        result[1..].copy_from_slice(&vkey_bytes);
        result
    }
}

/// Public values to verify the FEP.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FepInputs {
    /// OP succinct values.
    pub l1_head: Digest,
    pub claim_block_num: u32,
    pub rollup_config_hash: Digest,
    /// Pre root values.
    pub prev_state_root: Digest,
    pub prev_withdrawal_storage_root: Digest,
    pub prev_block_hash: Digest,
    /// Claim root values.
    pub new_state_root: Digest,
    pub new_withdrawal_storage_root: Digest,
    pub new_block_hash: Digest,

    /// Aggregation vkey hash babybear.
    pub aggregation_vkey_hash: BabyBearDigest,

    /// Range vkey commitment.
    pub range_vkey_commitment: [u8; 32],

    /// Trusted sequencer address.
    pub trusted_sequencer: Address,
    /// Signature in the "OptimisticMode" case.
    pub signature_optimistic_mode: Option<agglayer_primitives::Signature>,
    /// L1 info tree leaf containing the `l1Head` as block hash.
    pub l1_info_tree_leaf: L1InfoTreeLeaf,
    /// Inclusion proof of the leaf to the l1 info root.
    pub l1_head_inclusion_proof: MerkleProof,
}

sol! {
    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct AggregationProofPublicValues {
        bytes32 l1Head;
        bytes32 l2PreRoot;
        bytes32 l2PostRoot;
        uint64 l2BlockNumber;
        bytes32 rollupConfigHash;
        bytes32 multiBlockVKey;
        address proverAddress;
    }
}

impl From<&FepInputs> for AggregationProofPublicValues {
    fn from(inputs: &FepInputs) -> Self {
        Self {
            l1Head: inputs.l1_head.0.into(),
            l2PreRoot: inputs.compute_l2_pre_root().into(),
            l2PostRoot: inputs.compute_claim_root().into(),
            l2BlockNumber: inputs.claim_block_num.into(),
            rollupConfigHash: inputs.rollup_config_hash.0.into(),
            multiBlockVKey: inputs.range_vkey_commitment.into(),
            proverAddress: inputs.trusted_sequencer,
        }
    }
}

sol! {
    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct AggchainParamsValues {
        bytes32 l2PreRoot;
        bytes32 claimRoot;
        uint256 claimBlockNum;
        bytes32 rollupConfigHash;
        bool optimisticMode;
        address trustedSequencer;
        bytes32 range_vkey_commitment;
        bytes32 aggregation_vkey_hash;
    }
}

impl From<&FepInputs> for AggchainParamsValues {
    fn from(inputs: &FepInputs) -> Self {
        Self {
            l2PreRoot: inputs.compute_l2_pre_root().into(),
            claimRoot: inputs.compute_claim_root().into(),
            claimBlockNum: U256::from(inputs.claim_block_num),
            rollupConfigHash: inputs.rollup_config_hash.0.into(),
            optimisticMode: inputs.optimistic_mode() == OptimisticMode::Ecdsa,
            trustedSequencer: inputs.trusted_sequencer,
            range_vkey_commitment: inputs.range_vkey_commitment.into(),
            aggregation_vkey_hash: inputs.aggregation_vkey_hash.to_hash_bn254().into(),
        }
    }
}

impl FepInputs {
    pub fn sha256_public_values(&self) -> [u8; 32] {
        let encoded_public_values =
            AggregationProofPublicValues::abi_encode(&AggregationProofPublicValues::from(self));

        Sha256::digest(encoded_public_values.as_slice()).into()
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
enum OptimisticMode {
    Sp1 = 0,
    Ecdsa = 1,
}

impl FepInputs {
    pub fn encoded_aggchain_params(&self) -> Vec<u8> {
        AggchainParamsValues::abi_encode_packed(&AggchainParamsValues::from(self))
    }

    /// Compute the chain-specific commitment forwarded to the PP.
    pub fn aggchain_params(&self) -> Digest {
        keccak256(self.encoded_aggchain_params().as_slice())
    }

    fn optimistic_mode(&self) -> OptimisticMode {
        if self.signature_optimistic_mode.is_some() {
            OptimisticMode::Ecdsa
        } else {
            OptimisticMode::Sp1
        }
    }

    /// Verify one ECDSA or the sp1 proof.
    pub fn verify(
        &self,
        l1_info_root: Digest,
        new_local_exit_root: Digest,
        commit_imported_bridge_exits: Digest,
    ) -> Result<(), ProofError> {
        if let Some(signature) = self.signature_optimistic_mode {
            // Verify only one ECDSA on the public inputs
            let sha256_fep_public_values = self.sha256_public_values();
            let signature_commitment = keccak256_combine([
                sha256_fep_public_values,
                new_local_exit_root.0,
                commit_imported_bridge_exits.0,
            ]);

            let recovered_signer = signature
                .recover_address_from_prehash(&B256::new(signature_commitment.0))
                .map_err(|_| ProofError::InvalidSignature)?;

            if recovered_signer != self.trusted_sequencer {
                eprintln!(
                    "fep public values: {:?}",
                    AggregationProofPublicValues::from(self)
                );
                eprintln!(
                    "signed_commitment: {signature_commitment:?} = keccak(sha256_fep_pv: \
                     {sha256_fep_public_values:?} || new_ler:
                     {new_local_exit_root:?} || commit_imported_bridge_exits: \
                     {commit_imported_bridge_exits:?})"
                );
                return Err(ProofError::InvalidSigner {
                    declared: self.trusted_sequencer,
                    recovered: recovered_signer,
                });
            }

            Ok(())
        } else {
            // Verify l1 head
            self.verify_l1_head(l1_info_root)?;

            // Verify the FEP stark proof.
            #[cfg(not(target_os = "zkvm"))]
            unreachable!("verify_sp1_proof is not callable outside of SP1");

            #[cfg(target_os = "zkvm")]
            {
                sp1_zkvm::lib::verify::verify_sp1_proof(
                    &self.aggregation_vkey_hash.to_hash_u32(),
                    &self.sha256_public_values().into(),
                );

                return Ok(());
            }
        }
    }
}

impl FepInputs {
    /// Verify that the `l1Head` considered by the FEP exists in the L1 Info
    /// Tree
    pub fn verify_l1_head(&self, l1_info_root: Digest) -> Result<(), ProofError> {
        if self.l1_head != self.l1_info_tree_leaf.inner.block_hash {
            return Err(ProofError::MismatchL1Head {
                from_l1_info_tree_leaf: self.l1_info_tree_leaf.inner.block_hash,
                from_fep_public_values: self.l1_head,
            });
        }

        let inclusion_proof_valid = self.l1_head_inclusion_proof.verify(
            self.l1_info_tree_leaf.hash(),
            self.l1_info_tree_leaf.l1_info_tree_index,
        );

        // TODO: proper error
        if !(inclusion_proof_valid && l1_info_root == self.l1_head_inclusion_proof.root) {
            return Err(ProofError::InvalidInclusionProofL1Head {
                index: self.l1_info_tree_leaf.l1_info_tree_index,
                l1_leaf_hash: self.l1_info_tree_leaf.hash(),
                l1_info_root,
            });
        }

        Ok(())
    }

    /// Compute l2 pre root.
    pub fn compute_l2_pre_root(&self) -> L2PreRoot {
        compute_output_root(
            self.prev_state_root.0,
            self.prev_withdrawal_storage_root.0,
            self.prev_block_hash.0,
        )
        .into()
    }

    /// Compute claim root.
    pub fn compute_claim_root(&self) -> ClaimRoot {
        compute_output_root(
            self.new_state_root.0,
            self.new_withdrawal_storage_root.0,
            self.new_block_hash.0,
        )
    }
}

/// Compute output root as defined here:
/// https://specs.optimism.io/protocol/proposals.html#l2-output-commitment-construction
pub(crate) fn compute_output_root(
    state_root: [u8; 32],
    withdrawal_storage_root: [u8; 32],
    block_hash: [u8; 32],
) -> ClaimRoot {
    ClaimRoot(keccak256_combine([
        OUTPUT_ROOT_VERSION,
        state_root,
        withdrawal_storage_root,
        block_hash,
    ]))
}

#[cfg(test)]
mod tests {
    use crate::full_execution_proof::compute_output_root;

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

        let computed_output_root = compute_output_root(state, withdrawal, block_hash).0 .0;
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
