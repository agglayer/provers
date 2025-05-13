use aggchain_proof_types::imported_bridge_exit::{
    BridgeExitHash, ImportedBridgeExitWithBlockNumber,
};

use crate::error::AggchainProofRequestError as Error;
use crate::v1;

impl TryFrom<v1::ImportedBridgeExitWithBlockNumber> for ImportedBridgeExitWithBlockNumber {
    type Error = Error;

    fn try_from(value: v1::ImportedBridgeExitWithBlockNumber) -> Result<Self, Self::Error> {
        Ok(Self {
            block_number: value.block_number,
            global_index: value
                .global_index
                .ok_or_else(|| Error::MissingGlobalIndex {
                    field_path: "global_index".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidDigest {
                    field_path: "global_index".to_string(),
                    source: anyhow::Error::from(error),
                })?,
            bridge_exit_hash: BridgeExitHash(
                value
                    .bridge_exit_hash
                    .ok_or_else(|| Error::MissingGlobalIndex {
                        field_path: "global_index".to_string(),
                    })?
                    .try_into()
                    .map_err(|error| Error::InvalidDigest {
                        field_path: "global_index".to_string(),
                        source: anyhow::Error::from(error),
                    })?,
            ),
        })
    }
}
