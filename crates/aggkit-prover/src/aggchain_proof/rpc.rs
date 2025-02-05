use aggkit_prover_types::v1::{
    aggchain_proof_service_server::AggchainProofService as AggchainProofGrpcService,
    GenerateAggchainProofRequest, GenerateAggchainProofResponse,
};
use tonic::{Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt};
use tower::{Service, ServiceExt};
use tracing::instrument;

use super::service::{AggchainProofService, ProofRequest};

#[derive(Default, Clone)]
pub struct GrpcService {
    service: AggchainProofService,
}

#[tonic::async_trait]
impl AggchainProofGrpcService for GrpcService {
    #[instrument(skip(self))]
    async fn generate_aggchain_proof(
        &self,
        request: Request<GenerateAggchainProofRequest>,
    ) -> Result<Response<GenerateAggchainProofResponse>, Status> {
        let request = request.into_inner();
        if request.max_end_block < request.start_block {
            let mut error = ErrorDetails::new();
            error.add_bad_request_violation(
                "max_end_block",
                "max_end_block must be greater than start_block",
            );

            return Err(Status::with_error_details(
                tonic::Code::InvalidArgument,
                "Invalid request argument(s)",
                error,
            ));
        }
        let proof_request = ProofRequest {
            start_block: request.start_block,
            max_block: request.max_end_block,
        };

        let mut service = self.service.clone();

        let service = service
            .ready()
            .await
            .map_err(|_| Status::internal("Unable to get the service"))?;

        match service.call(proof_request).await {
            Ok(_response) => Ok(Response::new(GenerateAggchainProofResponse {
                aggchain_proof: Vec::new(),
                start_block: 0,
                end_block: 0,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
