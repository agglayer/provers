use aggchain_proof_types::unclaim::UnclaimWithBlockNumber;

use crate::{error::AggchainProofRequestError as Error, v1};

impl TryFrom<v1::Unclaim> for UnclaimWithBlockNumber {
    type Error = Error;

    fn try_from(value: v1::Unclaim) -> Result<Self, Self::Error> {
        let unclaim_hash = value
            .unclaim_hash
            .ok_or_else(|| Error::MissingUnclaimHash {
                field_path: "unclaim_hash".to_string(),
            })?
            .try_into()
            .map_err(|error| Error::InvalidDigest {
                field_path: "unclaim_hash".to_string(),
                source: anyhow::Error::from(error),
            })?;

        Ok(Self {
            unclaim_hash,
            block_number: value.block_number,
            block_index: value.block_index,
        })
    }
}
