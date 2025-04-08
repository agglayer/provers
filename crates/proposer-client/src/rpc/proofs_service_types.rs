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

impl From<AggregationProofProposerRequest> for grpc::AggregationProofRequest {
    fn from(request: AggregationProofProposerRequest) -> Self {
        grpc::AggregationProofRequest {
            last_proven_block: request.last_proven_block,
            requested_end_block: request.requested_end_block,
            l1_block_number: request.l1_block_number,
            l1_block_hash: request.l1_block_hash.to_vec().into(),
        }
    }
}

impl TryFrom<grpc::AggregationProofResponse> for AggregationProofProposerResponse {
    type Error = ProofRequestError;

    fn try_from(response: grpc::AggregationProofResponse) -> Result<Self, Self::Error> {
        let request_id = convert_field("request_id", response.request_id.to_vec().as_slice())
            .map_err(ProofRequestError::ParsingResponse)?;
        Ok(AggregationProofProposerResponse {
            request_id,
            last_proven_block: response.last_proven_block,
            end_block: response.end_block,
        })
    }
}
