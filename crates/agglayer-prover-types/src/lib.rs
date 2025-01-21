use bincode::{
    config::{BigEndian, FixintEncoding, WithOtherEndian, WithOtherIntEncoding},
    DefaultOptions, Options,
};
use pessimistic_proof::ProofError;
use serde::{Deserialize, Serialize};
use sp1_sdk::SP1VerificationError;
use tonic::Status;
use v1::{ProofGenerationError, ProofGenerationErrorType};

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/agglayer.prover.bin");

#[path = "generated/agglayer.prover.v1.rs"]
#[rustfmt::skip]
#[allow(warnings)]
pub mod v1;

pub fn default_bincode_options(
) -> WithOtherIntEncoding<WithOtherEndian<DefaultOptions, BigEndian>, FixintEncoding> {
    DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("Unable to execute prover")]
    UnableToExecuteProver,

    #[error("Prover failed: {0}")]
    ProverFailed(String),

    #[error("Prover verification failed: {0}")]
    ProofVerificationFailed(#[from] ProofVerificationError),

    #[error("Prover executor failed: {0}")]
    ExecutorFailed(#[from] ProofError),
}

impl TryFrom<&Error> for Status {
    type Error = bincode::Error;

    fn try_from(value: &Error) -> Result<Self, Self::Error> {
        let (code, message, details) = match value {
            Error::UnableToExecuteProver => {
                let details = default_bincode_options().serialize(&ProofGenerationError {
                    error: vec![],
                    error_type: ProofGenerationErrorType::UnableToExecuteProver.into(),
                })?;

                (
                    tonic::Code::Internal,
                    "Unable to execute prover".to_string(),
                    details,
                )
            }
            Error::ProverFailed(_) => {
                let details = default_bincode_options().serialize(&ProofGenerationError {
                    error: vec![],
                    error_type: ProofGenerationErrorType::ProverFailed.into(),
                })?;
                (tonic::Code::Internal, value.to_string(), details)
            }
            Error::ProofVerificationFailed(ref proof_verification_error) => {
                let details = default_bincode_options().serialize(&ProofGenerationError {
                    error: default_bincode_options().serialize(&proof_verification_error)?,
                    error_type: ProofGenerationErrorType::ProofVerificationFailed.into(),
                })?;

                (tonic::Code::InvalidArgument, value.to_string(), details)
            }
            Error::ExecutorFailed(ref proof_error) => {
                let details = default_bincode_options().serialize(&ProofGenerationError {
                    error: default_bincode_options().serialize(&proof_error)?,
                    error_type: ProofGenerationErrorType::ProverFailed.into(),
                })?;

                (tonic::Code::InvalidArgument, value.to_string(), details)
            }
        };

        Ok(Status::with_details(code, message, details.into()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error, PartialEq, Eq)]
pub enum ProofVerificationError {
    #[error("Version mismatch: {0}")]
    VersionMismatch(String),
    #[error("Core machine verification error: {0}")]
    Core(String),
    #[error("Recursion verification error: {0}")]
    Recursion(String),
    #[error("Plonk verification error: {0}")]
    Plonk(String),
    #[error("Groth16 verification error: {0}")]
    Groth16(String),
    #[error("Invalid public values")]
    InvalidPublicValues,
}

impl From<SP1VerificationError> for ProofVerificationError {
    fn from(err: SP1VerificationError) -> Self {
        match err {
            SP1VerificationError::VersionMismatch(version) => {
                ProofVerificationError::VersionMismatch(version)
            }
            SP1VerificationError::Core(core) => ProofVerificationError::Core(core.to_string()),
            SP1VerificationError::Recursion(recursion) => {
                ProofVerificationError::Recursion(recursion.to_string())
            }
            SP1VerificationError::Plonk(error) => ProofVerificationError::Plonk(error.to_string()),
            SP1VerificationError::Groth16(error) => {
                ProofVerificationError::Groth16(error.to_string())
            }
            SP1VerificationError::InvalidPublicValues => {
                ProofVerificationError::InvalidPublicValues
            }
        }
    }
}
