pub mod grpc {
    tonic::include_proto!("proofs");
}

pub use grpc::proofs_client::ProofsClient;

use super::{
    error::{GrpcConversionError, ProofRequestError},
    AggSpanProofProposerRequest, AggSpanProofProposerResponse,
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

impl From<AggSpanProofProposerRequest> for grpc::AggProofRequest {
    fn from(request: AggSpanProofProposerRequest) -> Self {
        grpc::AggProofRequest {
            start: request.start_block,
            end: request.max_block,
            l1_block_number: request.l1_block_number,
            l1_block_hash: request.l1_block_hash.to_vec(),
        }
    }
}

impl TryFrom<grpc::AggProofResponse> for AggSpanProofProposerResponse {
    type Error = ProofRequestError;

    fn try_from(response: grpc::AggProofResponse) -> Result<Self, Self::Error> {
        if response.success {
            let proof_id = convert_field("request_id", response.request_id.as_slice())
                .map_err(ProofRequestError::ParsingResponse)?;
            Ok(AggSpanProofProposerResponse {
                proof_id,
                start_block: response.start,
                end_block: response.end,
            })
        } else {
            Err(ProofRequestError::Failed(response.error))
        }
    }
}
