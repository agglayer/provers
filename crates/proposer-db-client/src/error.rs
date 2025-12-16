use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Proof not found for the given ID")]
    ProofNotFound,

    #[error("No consecutive complete range proofs found")]
    NoRangeProofsFound,
}
