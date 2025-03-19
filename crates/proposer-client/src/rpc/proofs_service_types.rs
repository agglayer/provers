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

impl TryFrom<AggSpanProofProposerRequest> for grpc::AggProofRequest {
    type Error = GrpcConversionError;

    fn try_from(request: AggSpanProofProposerRequest) -> Result<Self, GrpcConversionError> {
        let request = grpc::AggProofRequest {
            start: convert_field("start", request.start_block)?,
            end: convert_field("end", request.max_block)?,
            l1_block_number: convert_field("l1_block_number", request.l1_block_number)?,
            l1_block_hash: request.l1_block_hash.to_vec(),
        };
        Ok(request)
    }
}

impl TryFrom<grpc::AggProofResponse> for AggSpanProofProposerResponse {
    type Error = ProofRequestError;

    fn try_from(response: grpc::AggProofResponse) -> Result<Self, Self::Error> {
        if response.success {
            let (proof_id, start_block, end_block) = Default::default(); // TODO
            Ok(AggSpanProofProposerResponse {
                proof_id,
                start_block,
                end_block,
            })
        } else {
            Err(ProofRequestError::Failed(response.error))
        }
    }
}
