use bincode::Options as _;
pub use prover_executor::Error;
use tonic::Status;

use crate::{
    default_bincode_options,
    v1::{ErrorKind, GenerateProofError},
};
pub struct ErrorWrapper;

impl ErrorWrapper {
    pub fn try_into_status(value: &Error) -> Result<Status, bincode::Error> {
        let (code, message, details) = match value {
            Error::UnableToExecuteProver => {
                let details = default_bincode_options().serialize(&GenerateProofError {
                    error: vec![].into(),
                    error_type: ErrorKind::UnableToExecuteProver.into(),
                })?;

                (
                    tonic::Code::Internal,
                    "Unable to execute prover".to_string(),
                    details,
                )
            }
            Error::ProverFailed(_) => {
                let details = default_bincode_options().serialize(&GenerateProofError {
                    error: vec![].into(),
                    error_type: ErrorKind::ProverFailed.into(),
                })?;
                (tonic::Code::Internal, value.to_string(), details)
            }
            Error::ProofVerificationFailed(ref proof_verification_error) => {
                let details = default_bincode_options().serialize(&GenerateProofError {
                    error: default_bincode_options()
                        .serialize(&proof_verification_error)?
                        .into(),
                    error_type: ErrorKind::ProofVerificationFailed.into(),
                })?;

                (tonic::Code::InvalidArgument, value.to_string(), details)
            }
            Error::ExecutorFailed(ref proof_error) => {
                let details = default_bincode_options().serialize(&GenerateProofError {
                    error: default_bincode_options().serialize(&proof_error)?.into(),
                    error_type: ErrorKind::ExecutorFailed.into(),
                })?;

                (tonic::Code::InvalidArgument, value.to_string(), details)
            }
        };

        Ok(Status::with_details(code, message, details.into()))
    }
}
