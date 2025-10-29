use aggchain_proof_types::removed_ger::RemovedGerWithBlockNumber;

use crate::{error::AggchainProofRequestError as Error, v1};

impl TryFrom<v1::RemovedGer> for RemovedGerWithBlockNumber {
    type Error = Error;

    fn try_from(value: v1::RemovedGer) -> Result<Self, Self::Error> {
        let global_exit_root = value
            .global_exit_root
            .ok_or_else(|| Error::MissingRemovedGer {
                field_path: "global_exit_root".to_string(),
            })?
            .try_into()
            .map_err(|error| Error::InvalidDigest {
                field_path: "global_exit_root".to_string(),
                source: eyre::Error::from(error),
            })?;

        Ok(Self {
            global_exit_root,
            block_number: value.block_number,
            log_index: value.log_index,
        })
    }
}
