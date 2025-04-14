use aggchain_proof_types::inserted_ger::{InsertedGer, InsertedGerWithBlockNumber};

use crate::error::AggchainProofRequestError as Error;
use crate::v1;

impl TryFrom<v1::ProvenInsertedGerWithBlockNumber> for InsertedGerWithBlockNumber {
    type Error = Error;

    fn try_from(value: v1::ProvenInsertedGerWithBlockNumber) -> Result<Self, Self::Error> {
        Ok(Self {
            block_number: value.block_number,
            block_index: value.block_index,
            inserted_ger: value
                .proven_inserted_ger
                .ok_or_else(|| Error::MissingInsertedGer {
                    field_path: "proven_inserted_ger".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidInsertedGer {
                    field_path: "proven_inserted_ger".to_string(),
                    source: anyhow::Error::from(error),
                })?,
        })
    }
}

impl TryFrom<v1::ProvenInsertedGer> for InsertedGer {
    type Error = Error;

    fn try_from(value: v1::ProvenInsertedGer) -> Result<Self, Self::Error> {
        Ok(Self {
            proof_ger_l1root: value
                .proof_ger_l1root
                .ok_or_else(|| Error::MissingInsertedGer {
                    field_path: "proof_ger_l1root".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidInsertedGer {
                    field_path: "proof_ger_l1root".to_string(),
                    source: anyhow::Error::from(error),
                })?,
            l1_leaf: value
                .l1_leaf
                .ok_or_else(|| Error::MissingL1InfoTreeLeaf {
                    field_path: "l1_leaf".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidL1InfoTreeLeaf {
                    field_path: "l1_leaf".to_string(),
                    source: anyhow::Error::from(error),
                })?,
        })
    }
}
