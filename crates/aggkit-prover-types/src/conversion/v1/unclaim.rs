use aggchain_proof_types::unclaim::UnclaimWithBlockNumber;
use base64::{prelude::BASE64_STANDARD, Engine};

use crate::{error::AggchainProofRequestError as Error, v1, Digest};

impl TryFrom<v1::Unclaim> for UnclaimWithBlockNumber {
    type Error = Error;

    fn try_from(value: v1::Unclaim) -> Result<Self, Self::Error> {
        let bytes = BASE64_STANDARD
            .decode(&value.unclaim_hash)
            .map_err(|error| Error::InvalidUnclaim {
                field_path: "unclaim_hash".to_string(),
                source: anyhow::Error::from(error),
            })?;

        let array: [u8; 32] = bytes
            .try_into()
            .map_err(|error: Vec<u8>| Error::InvalidUnclaim {
                field_path: "unclaim_hash".to_string(),
                source: anyhow::Error::msg(format!(
                    "Expected unclaim of 32 bytes length, got {}",
                    error.len()
                )),
            })?;

        Ok(Self {
            unclaim_hash: Digest::from(array),
            block_number: value.block_number,
            block_index: value.block_index,
        })
    }
}
