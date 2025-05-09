use aggchain_proof_types::{AggchainProofInputs, OptimisticAggchainProofInputs};

use crate::error::AggchainProofRequestError as Error;
use crate::v1;

mod imported_bridge_exit;
mod inserted_ger;

impl TryFrom<v1::GenerateOptimisticAggchainProofRequest> for OptimisticAggchainProofInputs {
    type Error = Error;

    fn try_from(value: v1::GenerateOptimisticAggchainProofRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            signature_optimistic_mode: value
                .optimistic_mode_signature
                .ok_or_else(|| Error::MissingOptimisticModeSignature {
                    field_path: "optimistic_mode_signature".to_string(),
                })?
                .value
                .as_ref()
                .try_into()
                .map_err(|error| Error::InvalidOptimisticModeSignature {
                    field_path: "optimistic_mode_signature".to_string(),
                    source: anyhow::Error::from(error),
                })?,
            aggchain_proof_inputs: value
                .aggchain_proof_request
                .ok_or_else(|| Error::MissingAggchainProofRequest {
                    field_path: "aggchain_proof_request".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidAggchainProofRequest {
                    field_path: "aggchain_proof_request".to_string(),
                    source: anyhow::Error::from(error),
                })?,
        })
    }
}

impl TryFrom<v1::GenerateAggchainProofRequest> for AggchainProofInputs {
    type Error = Error;

    fn try_from(value: v1::GenerateAggchainProofRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            last_proven_block: value.last_proven_block,
            requested_end_block: value.requested_end_block,
            l1_info_tree_root_hash: value
                .l1_info_tree_root_hash
                .ok_or_else(|| Error::MissingL1InfoTreeRootHash {
                    field_path: "l1_info_tree_root_hash".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidDigest {
                    field_path: "l1_info_tree_root_hash".to_string(),
                    source: anyhow::Error::from(error),
                })?,
            l1_info_tree_leaf: value
                .l1_info_tree_leaf
                .ok_or(Error::MissingL1InfoTreeLeaf {
                    field_path: "l1_info_tree_leaf".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidL1InfoTreeLeaf {
                    field_path: "l1_info_tree_leaf".to_string(),
                    source: anyhow::Error::from(error),
                })?,
            l1_info_tree_merkle_proof: value
                .l1_info_tree_merkle_proof
                .ok_or_else(|| Error::MissingL1InfoTreeMerkleProof {
                    field_path: "l1_info_tree_merkle_proof".to_string(),
                })?
                .try_into()
                .map_err(|source| Error::InvalidL1InfoTreeMerkleProof {
                    field_path: "l1_info_tree_merkle_proof".to_string(),
                    source: anyhow::Error::from(source),
                })?,
            ger_leaves: value
                .ger_leaves
                .into_iter()
                .map(|(k, v)| {
                    Ok((
                        k,
                        v.try_into().map_err(|error| {
                            Error::InvalidInsertedGerWithBlockNumberConversion {
                                field_path: "ger_leaves".to_string(),
                                source: anyhow::Error::from(error),
                            }
                        })?,
                    ))
                })
                .collect::<Result<_, _>>()?,
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .map_err(|error| Error::InvalidImportedBridgeExit {
                    field_path: "imported_bridge_exits".to_string(),
                    source: anyhow::Error::from(error),
                })?,
        })
    }
}
