use std::collections::HashMap;

use aggchain_proof_core::Digest;
use agglayer_interop::types::{L1InfoTreeLeaf, MerkleProof};
use serde::{Deserialize, Serialize};

use crate::{
    imported_bridge_exit::ImportedBridgeExitWithBlockNumber,
    inserted_ger::InsertedGerWithBlockNumber,
};

/// Data needed as the input for the aggchain proof generation.
#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub l1_info_tree_merkle_proof: MerkleProof,
    /// Map of the Global Exit Roots with their inclusion proof.
    /// Note: the GER (string) is a base64 encoded string of the GER digest.
    pub ger_leaves: HashMap<String, InsertedGerWithBlockNumber>,
    /// Imporeted bridge exits.
    pub imported_bridge_exits: Vec<ImportedBridgeExitWithBlockNumber>,
}
