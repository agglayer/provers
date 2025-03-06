use std::collections::HashMap;

use aggchain_proof_types::AggchainProofRequestError as Error;
use alloy_primitives::Address;

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

impl TryFrom<v1::BridgeExit> for aggchain_proof_types::BridgeExit {
    type Error = Error;

    fn try_from(value: v1::BridgeExit) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::BridgeExit {
            leaf_type: value.leaf_type,
            token_info: value
                .token_info
                .ok_or(Error::MissingTokenInfo)?
                .try_into()?,
            destination_network: value.destination_network,
            destination_address: Address::from_slice(&value.destination_address),
            amount: value.amount,
            is_metadata_hashed: value.is_metadata_hashed,
            metadata: value.metadata,
        })
    }
}

impl TryFrom<v1::ImportedBridgeExit> for aggchain_proof_types::ImportedBridgeExit {
    type Error = Error;

    fn try_from(value: v1::ImportedBridgeExit) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::ImportedBridgeExit {
            bridge_exit: value
                .bridge_exit
                .ok_or(Error::MissingBridgeExit)?
                .try_into()?,
            global_index: value.global_index.ok_or(Error::MissingGlobalIndex)?.into(),
        })
    }
}

impl TryFrom<v1::L1InfoTreeLeafInner> for aggchain_proof_types::L1InfoTreeLeafInner {
    type Error = Error;

    fn try_from(value: v1::L1InfoTreeLeafInner) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::L1InfoTreeLeafInner {
            global_exit_root: value
                .global_exit_root
                .try_into()
                .map_err(Error::InvalidHexConversion)?,
            block_hash: value
                .block_hash
                .try_into()
                .map_err(Error::InvalidHexConversion)?,
            timestamp: value.timestamp,
        })
    }
}

impl TryFrom<v1::L1InfoTreeLeaf> for aggchain_proof_types::L1InfoTreeLeaf {
    type Error = Error;

    fn try_from(value: v1::L1InfoTreeLeaf) -> Result<Self, Self::Error> {
        Ok(aggchain_proof_types::L1InfoTreeLeaf {
            l1_info_tree_index: value.l1_info_tree_index,
            rollup_exit_root: value.rer.try_into().map_err(Error::InvalidHexConversion)?,
            mainnet_exit_root: value.mer.try_into().map_err(Error::InvalidHexConversion)?,
            inner_leaf: value
                .inner
                .ok_or(Error::MissingL1InfoTreeLeafInner)?
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
                .map(|x| x.try_into().map_err(Error::InvalidHexConversion))
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
                .ok_or(Error::MissingInclusionProof)?
                .try_into()?,
            l1_leaf: value
                .l1_leaf
                .ok_or(Error::MissingL1InfoTreeLeaf)?
                .try_into()?,
        })
    }
}

impl TryFrom<v1::GenerateAggchainProofRequest> for aggchain_proof_types::AggchainProofRequest {
    type Error = Error;

    fn try_from(value: v1::GenerateAggchainProofRequest) -> Result<Self, Self::Error> {
        // Parse l1 info tree proof, of type  [Digest; 32].
        let l1_info_tree_merkle_proof: [aggchain_proof_types::Digest; 32] = value
            .l1_info_tree_merkle_proof
            .into_iter()
            .map(|x| x.try_into().map_err(Error::InvalidHexConversion))
            .collect::<Result<Vec<aggchain_proof_types::Digest>, Error>>()?
            .try_into()
            .map_err(|_| Error::MissingL1InfoTreeMerkleProof)?;

        Ok(aggchain_proof_types::AggchainProofRequest {
            start_block: value.start_block,
            max_end_block: value.max_end_block,
            l1_info_tree_root_hash: value
                .l1_info_tree_root_hash
                .try_into()
                .map_err(Error::InvalidHexConversion)?,
            l1_info_tree_leaf: value
                .l1_info_tree_leaf
                .ok_or(Error::MissingL1InfoTreeLeaf)?
                .try_into()?,
            l1_info_tree_merkle_proof,
            ger_leaves: value
                .ger_leaves
                .into_iter()
                .map(|(k, v)| {
                    Ok((
                        k,
                        v.try_into()
                            .map_err(|_| Error::InvalidClaimFromMainnetConversion)?,
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
