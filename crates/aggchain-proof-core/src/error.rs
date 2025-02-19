use serde::{Deserialize, Serialize};

use crate::bridge::BridgeConstraintsError;

/// Represents all the aggchain proof errors.
#[derive(Clone, thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofError {
    /// Error on the bridge constraints.
    #[error("Failure on the bridge constraints.")]
    BridgeConstraintsFailure(#[from] BridgeConstraintsError),
}
