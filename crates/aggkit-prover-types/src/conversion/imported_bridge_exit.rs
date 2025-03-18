use aggchain_proof_types::imported_bridge_exit::ImportedBridgeExitWithBlockNumber;

use crate::error::AggchainProofRequestError as Error;
use crate::v1;

impl TryFrom<v1::ImportedBridgeExitWithBlockNumber> for ImportedBridgeExitWithBlockNumber {
    type Error = Error;

    fn try_from(value: v1::ImportedBridgeExitWithBlockNumber) -> Result<Self, Self::Error> {
        Ok(Self {
            block_number: value.block_number,
            imported_bridge_exit: value
                .imported_bridge_exit
                .ok_or_else(|| Error::MissingImportedBridgeExit {
                    field_path: "imported_bridge_exit".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidImportedBridgeExit {
                    field_path: "imported_bridge_exit".to_string(),
                    source: anyhow::Error::from(error),
                })?,
        })
    }
}
