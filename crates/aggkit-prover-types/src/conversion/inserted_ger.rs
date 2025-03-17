use aggchain_proof_types::inserted_ger::{InsertedGer, InsertedGerWithBlockNumber};

use crate::error::AggchainProofRequestError as Error;
use crate::v1;

impl TryFrom<v1::InsertedGerWithBlockNumber> for InsertedGerWithBlockNumber {
    type Error = Error;

    fn try_from(value: v1::InsertedGerWithBlockNumber) -> Result<Self, Self::Error> {
        Ok(Self {
            block_number: value.block_number,
            inserted_ger_leaf: value
                .inserted_ger_leaf
                .ok_or_else(|| Error::MissingImportedBridgeExit {
                    field_path: "inserted_ger_leaf".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidImportedBridgeExit {
                    field_path: "inserted_ger_leaf".to_string(),
                    source: anyhow::Error::from(error),
                })?,
        })
    }
}

impl TryFrom<v1::InsertedGer> for InsertedGer {
    type Error = Error;

    fn try_from(value: v1::InsertedGer) -> Result<Self, Self::Error> {
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
