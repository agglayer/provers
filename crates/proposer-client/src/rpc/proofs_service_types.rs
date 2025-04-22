use super::{
    error::{GrpcConversionError, ProofRequestError},
    grpc, AggregationProofProposerRequest, AggregationProofProposerResponse,
};

fn convert_field<T, U: TryFrom<T, Error = E>, E: Into<anyhow::Error>>(
    field: &'static str,
    value: T,
) -> Result<U, GrpcConversionError> {
    U::try_from(value).map_err(|e| GrpcConversionError::Conversion {
        field,
        source: e.into(),
    })
}

impl From<AggregationProofProposerRequest> for grpc::AggProofRequest {
    fn from(request: AggregationProofProposerRequest) -> Self {
        grpc::AggProofRequest {
            last_proven_block: request.last_proven_block,
            requested_end_block: request.requested_end_block,
            l1_block_number: request.l1_block_number,
            l1_block_hash: hex::encode(request.l1_block_hash),
        }
    }
}

impl TryFrom<grpc::AggProofResponse> for AggregationProofProposerResponse {
    type Error = ProofRequestError;

    fn try_from(response: grpc::AggProofResponse) -> Result<Self, Self::Error> {
        let grpc::AggProofResponse {
            success,
            error,
            last_proven_block,
            end_block,
            proof_request_id,
        } = response;

        if !success {
            return Err(ProofRequestError::Failed(error));
        }

        let request_id = convert_field("proof_request_id", &*proof_request_id)
            .map_err(ProofRequestError::ParsingResponse)?;

        Ok(AggregationProofProposerResponse {
            request_id,
            last_proven_block,
            end_block,
        })
    }
}
