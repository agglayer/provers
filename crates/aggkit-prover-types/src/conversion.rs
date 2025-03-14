use std::collections::HashMap;

use alloy_primitives::Address;

use crate::error::AggchainProofRequestError as Error;
use crate::v1;

impl TryFrom<v1::TokenInfo> for aggchain_proof_types::TokenInfo {
    type Error = Error;

    fn try_from(value: v1::TokenInfo) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::TokenInfo {
            origin_network: value.origin_network,
            origin_token_address: Address::from_slice(&value.origin_token_address),
        })
    }
}

impl From<v1::GlobalIndex> for aggchain_proof_types::GlobalIndex {
    fn from(value: v1::GlobalIndex) -> Self {
        aggchain_proof_types::GlobalIndex {
            mainnet_flag: value.mainnet_flag,
            rollup_index: value.rollup_index,
            leaf_index: value.leaf_index,
        }
    }
}

impl TryFrom<v1::ImportedBridgeExit> for aggchain_proof_types::ImportedBridgeExit {
    type Error = Error;

    fn try_from(value: v1::ImportedBridgeExit) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::ImportedBridgeExit {
            global_index: value
                .global_index
                .ok_or(Error::MissingGlobalIndex {
                    field_path: "imported_bridge_exits.global_index".to_string(),
                })?
                .into(),
        })
    }
}

impl TryFrom<v1::L1InfoTreeLeafInner> for aggchain_proof_types::L1InfoTreeLeafInner {
    type Error = Error;

    fn try_from(value: v1::L1InfoTreeLeafInner) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::L1InfoTreeLeafInner {
            global_exit_root: value.global_exit_root.try_into().map_err(|error| {
                Error::InvalidDigest {
                    field_path: "l1_info_tree_leaf.inner.global_exit_root".to_string(),
                    source: error,
                }
            })?,
            block_hash: value
                .block_hash
                .try_into()
                .map_err(|error| Error::InvalidDigest {
                    field_path: "l1_info_tree_leaf.inner.block_hash".to_string(),
                    source: error,
                })?,
            timestamp: value.timestamp,
        })
    }
}

impl TryFrom<v1::L1InfoTreeLeaf> for aggchain_proof_types::L1InfoTreeLeaf {
    type Error = Error;

    fn try_from(value: v1::L1InfoTreeLeaf) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::L1InfoTreeLeaf {
            l1_info_tree_index: value.l1_info_tree_index,
            rollup_exit_root: value.rer.try_into().map_err(|error| Error::InvalidDigest {
                field_path: "l1_info_tree_leaf.rer".to_string(),
                source: error,
            })?,
            mainnet_exit_root: value.mer.try_into().map_err(|error| Error::InvalidDigest {
                field_path: "l1_info_tree_leaf.mer".to_string(),
                source: error,
            })?,
            inner_leaf: value
                .inner
                .ok_or(Error::MissingL1InfoTreeLeafInner {
                    field_path: "l1_info_tree_leaf.inner".to_string(),
                })?
                .try_into()?,
        })
    }
}

impl TryFrom<v1::InclusionProof> for aggchain_proof_types::InclusionProof {
    type Error = Error;

    fn try_from(value: v1::InclusionProof) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::InclusionProof {
            siblings: value
                .siblings
                .into_iter()
                .map(|x| {
                    x.try_into().map_err(|error| Error::InvalidDigest {
                        field_path: "ger_leaves.claim_from_mainnet.inclusion_proof".to_string(),
                        source: error,
                    })
                })
                .collect::<Result<Vec<aggchain_proof_types::Digest>, Error>>()?,
        })
    }
}

impl TryFrom<v1::ClaimFromMainnet> for aggchain_proof_types::ClaimFromMainnet {
    type Error = Error;

    fn try_from(value: v1::ClaimFromMainnet) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::ClaimFromMainnet {
            inclusion_proof: value
                .inclusion_proof
                .ok_or(Error::MissingInclusionProof {
                    field_path: "ger_leaves.claim_from_mainnet.inclusion_proof".to_string(),
                })?
                .try_into()?,
            l1_leaf: value
                .l1_leaf
                .ok_or(Error::MissingL1InfoTreeLeaf {
                    field_path: "ger_leaves.claim_from_mainnet.l1_leaf".to_string(),
                })?
                .try_into()?,
        })
    }
}

impl TryFrom<v1::GenerateAggchainProofRequest> for aggchain_proof_types::AggchainProofInputs {
    type Error = Error;

    fn try_from(value: v1::GenerateAggchainProofRequest) -> Result<Self, Self::Error> {
        // Parse l1 info tree proof, of type  [Digest; 32].
        let l1_info_tree_merkle_proof: [aggchain_proof_types::Digest; 32] = value
            .l1_info_tree_merkle_proof
            .into_iter()
            .map(|x| {
                x.try_into().map_err(|error| Error::InvalidDigest {
                    field_path: "l1_info_tree_merkle_proof".to_string(),
                    source: error,
                })
            })
            .collect::<Result<Vec<aggchain_proof_types::Digest>, Error>>()?
            .try_into()
            .map_err(|_| Error::MissingL1InfoTreeMerkleProof {
                field_path: "l1_info_tree_merkle_proof".to_string(),
            })?;

        Ok(aggchain_proof_types::AggchainProofInputs {
            start_block: value.start_block,
            max_end_block: value.max_end_block,
            l1_info_tree_root_hash: value.l1_info_tree_root_hash.try_into().map_err(|error| {
                Error::InvalidDigest {
                    field_path: "l1_info_tree_root_hash".to_string(),
                    source: error,
                }
            })?,
            l1_info_tree_leaf: value
                .l1_info_tree_leaf
                .ok_or(Error::MissingL1InfoTreeLeaf {
                    field_path: "l1_info_tree_leaf".to_string(),
                })?
                .try_into()?,
            l1_info_tree_merkle_proof,
            ger_leaves: value
                .ger_leaves
                .into_iter()
                .map(|(k, v)| {
                    Ok((
                        k,
                        v.try_into()
                            .map_err(|_| Error::InvalidClaimFromMainnetConversion {
                                field_path: "ger_leaves".to_string(),
                            })?,
                    ))
                })
                .collect::<Result<HashMap<String, aggchain_proof_types::ClaimFromMainnet>, Error>>(
                )?,
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<Vec<aggchain_proof_types::ImportedBridgeExit>, Error>>()?,
        })
    }
}
