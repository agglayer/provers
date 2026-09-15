use agglayer_primitives::{
    keccak::{keccak256, keccak256_combine},
    Address, Digest,
};
use alloy_primitives::{FixedBytes, B256, U256};
use alloy_sol_types::{sol, SolValue};
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

/// Bits each digest word occupies in the packed form.
const DIGEST_WORD_BITS: usize = 31;

/// Pack a vkey's KoalaBear digest into the 32 byte value registered on L1 as
/// `aggregationVkey`.
///
/// The encoding is sp1's `HashableKey::bytes32_raw`: the eight digest words
/// laid end to end, most significant first, `DIGEST_WORD_BITS` bits each.
fn hash_bn254_bytes(digest: HashU32) -> [u8; 32] {
    digest
        .into_iter()
        .fold(U256::ZERO, |packed, word| {
            (packed << DIGEST_WORD_BITS) | U256::from(word)
        })
        .to_be_bytes()
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

    /// Aggregation vkey hash, as the KoalaBear digest words.
    pub aggregation_vkey_hash: HashU32,

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
        bytes32 l1_head;
        bytes32 l2_pre_root;
        bytes32 l2_post_root;
        uint64 l2_block_number;
        bytes32 rollup_config_hash;
        bytes32 multi_block_vkey;
        address prover_address;
    }
}

impl From<&FepInputs> for AggregationProofPublicValues {
    fn from(inputs: &FepInputs) -> Self {
        Self {
            l1_head: inputs.l1_head.0.into(),
            l2_pre_root: inputs.compute_l2_pre_root().into(),
            l2_post_root: inputs.compute_claim_root().into(),
            l2_block_number: inputs.claim_block_num.into(),
            rollup_config_hash: inputs.rollup_config_hash.0.into(),
            multi_block_vkey: inputs.range_vkey_commitment.into(),
            prover_address: inputs.trusted_sequencer.into(),
        }
    }
}

sol! {
    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct AggchainParamsValues {
        bytes32 l2_pre_root;
        bytes32 claim_root;
        uint256 claim_block_num;
        bytes32 rollup_config_hash;
        bool optimistic_mode;
        address trusted_sequencer;
        bytes32 range_vkey_commitment;
        bytes32 aggregation_vkey_hash;
    }
}

impl From<&FepInputs> for AggchainParamsValues {
    fn from(inputs: &FepInputs) -> Self {
        Self {
            l2_pre_root: inputs.compute_l2_pre_root().into(),
            claim_root: inputs.compute_claim_root().into(),
            claim_block_num: U256::from(inputs.claim_block_num),
            rollup_config_hash: inputs.rollup_config_hash.0.into(),
            optimistic_mode: inputs.optimistic_mode() == OptimisticMode::Ecdsa,
            trusted_sequencer: inputs.trusted_sequencer.into(),
            range_vkey_commitment: inputs.range_vkey_commitment.into(),
            aggregation_vkey_hash: hash_bn254_bytes(inputs.aggregation_vkey_hash).into(),
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
                    &self.aggregation_vkey_hash,
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
    use slop_algebra::{AbstractField, PrimeField32};
    use sp1_primitives::SP1Field;
    use sp1_sdk::HashableKey;

    use crate::{
        full_execution_proof::{compute_output_root, hash_bn254_bytes},
        vkey_hash::HashU32,
    };

    /// Lets sp1's own encoder run on a digest that no real vkey produced, so
    /// every test below can use sp1 as the reference rather than restating the
    /// encoding.
    struct ArbitraryVkey(HashU32);

    impl HashableKey for ArbitraryVkey {
        fn hash_koalabear(&self) -> [SP1Field; 8] {
            self.0.map(SP1Field::from_canonical_u32)
        }

        fn hash_u32(&self) -> HashU32 {
            self.0
        }
    }

    /// What sp1 encodes a digest to, or `None` where it refuses: its encoder
    /// assumes a packed digest always fills its buffer, and panics otherwise.
    fn sp1_encoding(digest: HashU32) -> Option<[u8; 32]> {
        std::panic::catch_unwind(|| ArbitraryVkey(digest).bytes32_raw()).ok()
    }

    /// A digest with nothing in it.
    const EMPTY: HashU32 = [0; 8];

    /// The largest word the digest field holds, without these tests needing to
    /// know how it is bounded.
    fn largest_word() -> u32 {
        (SP1Field::zero() - SP1Field::one()).as_canonical_u32()
    }

    /// `digest` with one word replaced. The only digest constructor these tests
    /// need: over [`EMPTY`] it isolates a single coordinate, over a saturated
    /// digest it perturbs one.
    fn replacing(digest: HashU32, position: usize, word: u32) -> HashU32 {
        std::array::from_fn(|index| {
            if index == position {
                word
            } else {
                digest[index]
            }
        })
    }

    /// Every coordinate of the encoding, at each value where an implementation
    /// would go wrong: empty, minimal, and the largest the field holds.
    ///
    /// The leading word stays at its largest throughout so sp1's own encoder
    /// accepts all of them -- it refuses short packings, see
    /// `encodes_the_digests_sp1_refuses` -- and the final digest carries the
    /// leading coordinate at full scale on its own.
    fn coordinate_boundaries() -> impl Iterator<Item = HashU32> {
        let saturated = [largest_word(); 8];

        (1..8)
            .flat_map(move |position| {
                [0, 1, largest_word()].map(move |word| replacing(saturated, position, word))
            })
            .chain(std::iter::once(replacing(EMPTY, 0, largest_word())))
    }

    /// The value committed for the real op-succinct keys is byte for byte what
    /// sp1 produces, and therefore what `addOpSuccinctConfig` registers on L1.
    #[test]
    fn matches_sp1_for_the_real_op_succinct_vkeys() {
        for vkey in [
            proposer_elfs::aggregation::VKEY.vkey(),
            proposer_elfs::range::VKEY.vkey(),
        ] {
            assert_eq!(hash_bn254_bytes(vkey.hash_u32()), vkey.bytes32_raw());
        }
    }

    /// Agreement with sp1 on every coordinate of the encoding, at each of its
    /// boundary values.
    ///
    /// Both implementations are the same positional sum -- every digest word
    /// scaled by the weight of its position -- so they can only disagree
    /// through a coordinate's weight or through coordinates interfering. This
    /// pins each weight against sp1, and
    /// `no_word_reaches_into_the_next_positions_range` rules out interference,
    /// which together settle every digest rather than a sample of them.
    #[test]
    fn matches_sp1_on_every_coordinate_boundary() {
        for digest in coordinate_boundaries() {
            let expected =
                sp1_encoding(digest).expect("leading word is maximal, so sp1 encodes this");

            assert_eq!(hash_bn254_bytes(digest), expected, "diverged on {digest:?}");
        }
    }

    /// A word at its largest stays strictly below the smallest contribution of
    /// the position above it, so no word can ever carry into its neighbour.
    ///
    /// This is what makes the encoding injective for *every* digest: distinct
    /// digests cannot share a commitment, so the value registered on L1 pins
    /// exactly one aggregation vkey.
    #[test]
    fn no_word_reaches_into_the_next_positions_range() {
        let commitment = |position, word| hash_bn254_bytes(replacing(EMPTY, position, word));

        for position in 1..8 {
            assert!(
                commitment(position, largest_word()) < commitment(position - 1, 1),
                "word {position} at its largest reaches position {}",
                position - 1,
            );
        }
    }

    /// sp1's own encoder cannot represent every digest: it assumes the packed
    /// value fills its buffer, and gives up when it does not. That is the
    /// reason the encoding is computed here rather than called out to.
    #[test]
    fn encodes_the_digests_sp1_refuses() {
        let digest = replacing(EMPTY, 7, 1);

        assert!(sp1_encoding(digest).is_none(), "sp1 encoded it after all");
        assert_ne!(hash_bn254_bytes(digest), [0u8; 32]);
    }

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
