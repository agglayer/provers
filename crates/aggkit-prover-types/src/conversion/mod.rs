use aggchain_proof_types::AggchainProofInputs;

use crate::error::AggchainProofRequestError as Error;
use crate::v1;

mod imported_bridge_exit;
mod inserted_ger;

impl TryFrom<v1::GenerateAggchainProofRequest> for AggchainProofInputs {
    type Error = Error;

    fn try_from(value: v1::GenerateAggchainProofRequest) -> Result<Self, Self::Error> {
        // Parse l1 info tree proof, of type  [Digest; 32].
        let l1_info_tree_merkle_proof: agglayer_interop::types::MerkleProof = value
            .l1_info_tree_merkle_proof
            .ok_or_else(|| Error::MissingL1InfoTreeMerkleProof {
                field_path: "l1_info_tree_merkle_proof".to_string(),
            })?
            .try_into()
            .map_err(|source| Error::InvalidL1InfoTreeMerkleProof {
                field_path: "l1_info_tree_merkle_proof".to_string(),
                source: anyhow::Error::from(source),
            })?;

        Ok(AggchainProofInputs {
            start_block: value.start_block,
            max_end_block: value.max_end_block,
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
            l1_info_tree_merkle_proof,
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
