use std::{collections::HashMap, ops::RangeInclusive};

use aggchain_proof_core::{bridge::inserted_ger::InsertedGER, Digest};
use agglayer_interop::types::{L1InfoTreeLeaf, MerkleProof};
use serde::{Deserialize, Serialize};

use crate::{
    imported_bridge_exit::ImportedBridgeExitWithBlockNumber,
    inserted_ger::InsertedGerWithBlockNumber,
};

/// Data needed as the input for the aggchain proof generation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AggchainProofInputs {
    ///  The last proven block before the requested aggchain proof.
    pub last_proven_block: u64,

    /// The max end block for which the aggchain proof is requested.
    pub requested_end_block: u64,

    /// Root hash of the L1 info tree.
    pub l1_info_tree_root_hash: Digest,

    /// Particular leaf of the l1 info tree corresponding
    /// to the requested_end_block.
    pub l1_info_tree_leaf: L1InfoTreeLeaf,

    /// Inclusion proof of the l1 info tree leaf to the
    /// l1 info tree root.
    pub l1_info_tree_merkle_proof: MerkleProof,

    /// Map of the Global Exit Roots with their inclusion proof.
    /// Note: the GER (string) is a base64 encoded string of the GER digest.
    pub ger_leaves: HashMap<String, InsertedGerWithBlockNumber>,

    /// Imported bridge exits.
    pub imported_bridge_exits: Vec<ImportedBridgeExitWithBlockNumber>,
}

/// Data needed as the input for the aggchain proof generation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptimisticAggchainProofInputs {
    pub aggchain_proof_inputs: AggchainProofInputs,
    pub signature_optimistic_mode: agglayer_primitives::Signature,
}

impl AggchainProofInputs {
    pub fn sorted_inserted_gers(&self, range: &RangeInclusive<u64>) -> Vec<InsertedGER> {
        let mut values: Vec<InsertedGER> = self
            .ger_leaves
            .values()
            .filter(|inserted_ger| range.contains(&inserted_ger.block_number))
            .cloned()
            .map(|e| InsertedGER {
                proof: e.inserted_ger.proof_ger_l1root,
                l1_info_tree_leaf: e.inserted_ger.l1_leaf,
                block_number: e.block_number,
                block_index: e.block_index,
            })
            .collect();

        values.sort_unstable_by_key(|e| (e.block_number, e.block_index));
        values
    }
}
