use crate::bridge::BridgeConstraintsError;

/// Represents all the aggchain proof errors.
#[derive(thiserror::Error, Debug)]
pub enum ProofError {
    /// Error on the bridge constraints.
    #[error(transparent)]
    BridgeConstraintsError(#[from] BridgeConstraintsError),
}
