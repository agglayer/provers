use super::{
    error::{GrpcConversionError, ProofRequestError},
    grpc, AggregationProofProposerRequest, AggregationProofProposerResponse,
    MockProofProposerRequest, MockProofProposerResponse,
};

fn convert_field<T, U: TryFrom<T, Error = E>, E: Into<eyre::Error>>(
    field: &'static str,
    value: T,
) -> Result<U, GrpcConversionError> {
    U::try_from(value).map_err(|e| GrpcConversionError {
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
            last_proven_block,
            end_block,
            proof_request_id,
        } = response;

        let request_id = convert_field("proof_request_id", &*proof_request_id)
            .map_err(ProofRequestError::ParsingResponse)?;

        Ok(AggregationProofProposerResponse {
            request_id,
            last_proven_block,
            end_block,
        })
    }
}

impl From<MockProofProposerRequest> for grpc::GetMockProofRequest {
    fn from(request: MockProofProposerRequest) -> Self {
        grpc::GetMockProofRequest {
            proof_id: request.proof_id.0,
        }
    }
}

impl TryFrom<grpc::GetMockProofResponse> for MockProofProposerResponse {
    type Error = ProofRequestError;

    fn try_from(response: grpc::GetMockProofResponse) -> Result<Self, Self::Error> {
        let grpc::GetMockProofResponse { proof } = response;

        Ok(MockProofProposerResponse {
            proof: convert_field("proof", &*proof).map_err(ProofRequestError::ParsingResponse)?,
        })
    }
}
