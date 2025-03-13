use std::collections::HashMap;

use alloy_primitives::Address;
use serde::{Deserialize, Serialize};

use crate::Digest;

/// Inclusion proof for the L1 info tree.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct InclusionProof {
    pub siblings: Vec<Digest>,
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
    /// The bridge exit initiated on another network, called the "sending"
    /// network. Need to verify that the destination network matches the
    /// current network, and that the bridge exit is included in an imported
    /// LER.
    pub bridge_exit: BridgeExit,
    /// The global index of the imported bridge exit.
    pub global_index: GlobalIndex,
}

/// Represents a token bridge exit from the network.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BridgeExit {
    /// The type of the leaf.
    pub leaf_type: i32,
    /// Unique ID for the token being transferred.
    pub token_info: TokenInfo,
    /// Network which the token is transferred to.
    pub destination_network: u32,
    /// Address which will own the received token.
    pub destination_address: Address,
    /// Token amount sent.
    pub amount: String,
    /// Is metadata hashed.
    pub is_metadata_hashed: bool,
    /// Metadata for the bridge exit.
    pub metadata: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GlobalIndex {
    pub mainnet_flag: bool,
    pub rollup_index: u32,
    pub leaf_index: u32,
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
pub struct AggchainProofRequest {
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
