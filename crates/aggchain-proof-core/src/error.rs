use serde::{Deserialize, Serialize};

/// Represents all the aggchain proof errors.
#[derive(Clone, thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofError {}
