use aggchain_proof_types::removed_ger::RemovedGerWithBlockNumber;
use base64::{prelude::BASE64_STANDARD, Engine};

use crate::{error::AggchainProofRequestError as Error, v1, Digest};

impl TryFrom<v1::RemovedGer> for RemovedGerWithBlockNumber {
    type Error = Error;

    fn try_from(value: v1::RemovedGer) -> Result<Self, Self::Error> {
        let bytes = BASE64_STANDARD
            .decode(&value.global_exit_root)
            .map_err(|error| Error::InvalidRemovedGer {
                field_path: "global_exit_root".to_string(),
                source: anyhow::Error::from(error),
            })?;

        let array: [u8; 32] =
            bytes
                .try_into()
                .map_err(|error: Vec<u8>| Error::InvalidRemovedGer {
                    field_path: "global_exit_root".to_string(),
                    source: anyhow::Error::msg(format!(
                        "Expected GER value of 32 bytes length, got {}",
                        error.len()
                    )),
                })?;

        Ok(Self {
            global_exit_root: Digest::from(array),
            block_number: value.block_number,
            block_index: value.block_index,
        })
    }
}
