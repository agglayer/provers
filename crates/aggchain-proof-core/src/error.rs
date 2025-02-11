use serde::{Deserialize, Serialize};

use crate::bridge::BridgeConstraintsError;

/// Represents all the aggchain proof errors.
#[derive(Clone, thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofError {
    /// Error on the bridge constraints.
    #[error(transparent)]
    BridgeConstraintsError(#[from] BridgeConstraintsError),
    /// Invalid block hash error.
    #[error("Invalid block hash: {0}")]
    InvalidBlockHash(String),
}
