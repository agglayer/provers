use aggchain_proof_types::OptimisticAggchainProofInputs;

use crate::{error::AggchainProofRequestError as Error, v1};

impl TryFrom<v1::GenerateOptimisticAggchainProofRequest> for OptimisticAggchainProofInputs {
    type Error = Error;

    fn try_from(value: v1::GenerateOptimisticAggchainProofRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            signature_optimistic_mode: value
                .optimistic_mode_signature
                .ok_or_else(|| Error::MissingOptimisticModeSignature {
                    field_path: "optimistic_mode_signature".to_string(),
                })?
                .value
                .as_ref()
                .try_into()
                .map_err(|error| Error::InvalidOptimisticModeSignature {
                    field_path: "optimistic_mode_signature".to_string(),
                    source: eyre::Error::from(error),
                })?,
            aggchain_proof_inputs: value
                .aggchain_proof_request
                .ok_or_else(|| Error::MissingAggchainProofRequest {
                    field_path: "aggchain_proof_request".to_string(),
                })?
                .try_into()
                .map_err(|error| Error::InvalidAggchainProofRequest {
                    field_path: "aggchain_proof_request".to_string(),
                    source: eyre::Error::from(error),
                })?,
        })
    }
}
