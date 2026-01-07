use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Proof not found for the given ID")]
    ProofNotFound,

    #[error("No consecutive complete range proofs found")]
    NoRangeProofsFound,

    #[error("Proof generation failed for request {0}")]
    ProofGenerationFailed(i64),

    #[error("Proof generation cancelled for request {0}")]
    ProofGenerationCancelled(i64),

    #[error("Proof generation timeout for request {0}")]
    ProofGenerationTimeout(i64),
}
