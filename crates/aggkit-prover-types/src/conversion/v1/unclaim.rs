use aggchain_proof_types::unclaim::UnclaimWithBlockNumber;

use crate::{error::AggchainProofRequestError as Error, v1};

impl TryFrom<v1::Unclaim> for UnclaimWithBlockNumber {
    type Error = Error;

    fn try_from(value: v1::Unclaim) -> Result<Self, Self::Error> {
        let global_index = value
            .global_index
            .ok_or_else(|| Error::MissingUnclaimGlobalIndex {
                field_path: "global_index".to_string(),
            })?
            .try_into()
            .map_err(|error| Error::InvalidDigest {
                field_path: "global_index".to_string(),
                source: eyre::Error::from(error),
            })?;

        Ok(Self {
            global_index,
            block_number: value.block_number,
            block_index: value.block_index,
        })
    }
}
