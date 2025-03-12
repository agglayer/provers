use std::collections::HashMap;

use aggchain_proof_core::keccak::keccak256_combine;
use aggchain_proof_core::local_exit_tree::hasher::Keccak256Hasher;
use aggchain_proof_core::local_exit_tree::proof::LETMerkleProof;
use alloy_primitives::Address;
use alloy_primitives::FixedBytes;
use serde::{Deserialize, Serialize};

use crate::Digest;

/// Inclusion proof for the L1 info tree.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct InclusionProof {
    pub siblings: Vec<Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid inclusion proof length. got: {got}, expected: {expected}")]
pub struct InvalidInclusionProofLength {
    got: usize,
    expected: usize,
}

impl TryFrom<InclusionProof> for LETMerkleProof<Keccak256Hasher> {
    type Error = InvalidInclusionProofLength;

    fn try_from(value: InclusionProof) -> Result<Self, Self::Error> {
        Ok(Self {
            siblings: *value.siblings.first_chunk::<{ Self::TREE_DEPTH }>().ok_or(
                InvalidInclusionProofLength {
                    got: value.siblings.len(),
                    expected: Self::TREE_DEPTH,
                },
            )?,
        })
    }
}

/// Claim from L1, used to prove the inclusion of the L1 info tree leaf.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ClaimFromMainnet {
    /// Proof from GER to Root
    pub inclusion_proof: InclusionProof,
    /// Related L1InfoTree leaf
    pub l1_leaf: L1InfoTreeLeaf,
}

/// Structure that represents a L1 info tree leaf, part of the
/// L1 info tree.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct L1InfoTreeLeaf {
    pub l1_info_tree_index: u32,
    pub rollup_exit_root: Digest,
    pub mainnet_exit_root: Digest,
    pub inner_leaf: L1InfoTreeLeafInner,
}

impl From<L1InfoTreeLeaf> for aggchain_proof_core::L1InfoTreeLeaf {
    fn from(value: L1InfoTreeLeaf) -> Self {
        Self {
            global_exit_root: keccak256_combine([value.mainnet_exit_root, value.rollup_exit_root]),
            block_hash: value.inner_leaf.block_hash,
            timestamp: value.inner_leaf.timestamp,
        }
    }
}

/// Represents the inner part of the leaf in the L1InfoTree.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct L1InfoTreeLeafInner {
    pub global_exit_root: Digest,
    pub block_hash: Digest,
    pub timestamp: u64,
}

/// Represents a token bridge exit originating on another network but claimed on
/// the current network.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ImportedBridgeExit {
    /// The global index of the imported bridge exit.
    pub global_index: GlobalIndex,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GlobalIndex {
    pub mainnet_flag: bool,
    pub rollup_index: u32,
    pub leaf_index: u32,
}

impl From<&GlobalIndex> for FixedBytes<32> {
    fn from(value: &GlobalIndex) -> Self {
        let mut bytes = [0u8; 32];

        let leaf_bytes = value.leaf_index.to_le_bytes();
        bytes[0..4].copy_from_slice(&leaf_bytes);

        let rollup_bytes = value.rollup_index.to_le_bytes();
        bytes[4..8].copy_from_slice(&rollup_bytes);

        if value.mainnet_flag {
            bytes[8] |= 0x01;
        }

        bytes.into()
    }
}

/// Encapsulates the information to uniquely identify a token on the origin
/// network.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TokenInfo {
    /// Network which the token originates from.
    pub origin_network: u32,
    /// The address of the token on the origin network.
    pub origin_token_address: Address,
}

/// Data needed as the input for the aggchain proof generation.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AggchainProofInputs {
    ///  The start block for which the aggchain proof is requested.
    pub start_block: u64,
    /// The max end block for which the aggchain proof is requested.
    pub max_end_block: u64,
    /// Root hash of the L1 info tree.
    pub l1_info_tree_root_hash: Digest,
    /// Particular leaf of the l1 info tree corresponding
    /// to the max_block.
    pub l1_info_tree_leaf: L1InfoTreeLeaf,
    /// Inclusion proof of the l1 info tree leaf to the
    /// l1 info tree root.
    pub l1_info_tree_merkle_proof: [Digest; 32],
    /// Map of the Global Exit Roots with their inclusion proof.
    /// Note: the GER (string) is a base64 encoded string of the GER digest.
    pub ger_leaves: HashMap<String, ClaimFromMainnet>,
    /// Imporeted bridge exits.
    pub imported_bridge_exits: Vec<ImportedBridgeExit>,
}
