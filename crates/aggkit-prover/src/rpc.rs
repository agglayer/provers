use std::collections::HashMap;

use aggchain_proof_service::config::AggchainProofServiceConfig;
use aggchain_proof_service::service::{AggchainProofService, AggchainProofServiceRequest};
use aggchain_proof_types::AggchainProofInputs;
use aggkit_prover_types::error::AggchainProofRequestError;
use aggkit_prover_types::v1::{
    aggchain_proof_service_server::AggchainProofService as AggchainProofGrpcService,
    GenerateAggchainProofRequest, GenerateAggchainProofResponse,
};
use agglayer_interop::grpc::v1::{AggchainProof, Sp1StarkProof};
use sp1_sdk::SP1_CIRCUIT_VERSION;
use tonic::{Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt};
use tower::buffer::Buffer;
use tower::{Service, ServiceExt};
use tracing::instrument;

const MAX_CONCURRENT_REQUESTS: usize = 100;

#[derive(Clone)]
pub struct GrpcService {
    service: Buffer<AggchainProofService, AggchainProofServiceRequest>,
}

impl GrpcService {
    pub async fn new(
        config: &AggchainProofServiceConfig,
    ) -> Result<Self, aggchain_proof_service::Error> {
        Ok(GrpcService {
            service: tower::ServiceBuilder::new()
                .buffer(MAX_CONCURRENT_REQUESTS)
                .service(AggchainProofService::new(config).await?),
        })
    }
}

#[tonic::async_trait]
impl AggchainProofGrpcService for GrpcService {
    #[instrument(skip(self, request))]
    async fn generate_aggchain_proof(
        &self,
        request: Request<GenerateAggchainProofRequest>,
    ) -> Result<Response<GenerateAggchainProofResponse>, Status> {
        let request = request.into_inner();
        if request.requested_end_block <= request.last_proven_block {
            let mut error = ErrorDetails::new();
            error.add_bad_request_violation(
                "requested_end_block",
                "requested_end_block must be greater than last_proven_block",
            );

            return Err(Status::with_error_details(
                tonic::Code::InvalidArgument,
                "Invalid request argument(s)",
                error,
            ));
        }

        let aggchain_proof_inputs: AggchainProofInputs =
            request
                .try_into()
                .map_err(|error: AggchainProofRequestError| {
                    let field = error.field_path();
                    let mut error_details = ErrorDetails::new();
                    error_details.add_bad_request_violation(field, error.to_string());
                    Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Invalid aggchain proof request data",
                        error_details,
                    )
                })?;

        let proof_request = AggchainProofServiceRequest {
            aggchain_proof_inputs,
        };

        let mut service = self.service.clone();

        let service = service
            .ready()
            .await
            .map_err(|_| Status::internal("Unable to get the service"))?;

        match service.call(proof_request).await {
            Ok(response) => Ok(Response::new(GenerateAggchainProofResponse {
                aggchain_proof: Some(AggchainProof {
                    aggchain_params: Some(response.aggchain_params.into()),
                    context: HashMap::new(),
                    proof: Some(agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(
                        Sp1StarkProof {
                            version: SP1_CIRCUIT_VERSION.to_string(),
                            proof: response.proof.into(),
                            vkey: response.vkey.into(),
                        },
                    )),
                }),
                last_proven_block: response.last_proven_block,
                end_block: response.end_block,
                local_exit_root_hash: Some(response.local_exit_root_hash.into()),
                custom_chain_data: response.custom_chain_data.into(),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
