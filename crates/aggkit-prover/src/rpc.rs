use aggchain_proof_service::{
    config::AggchainProofServiceConfig,
    service::{AggchainProofService, AggchainProofServiceRequest},
};
use aggchain_proof_types::{AggchainProofInputs, OptimisticAggchainProofInputs};
use aggkit_prover_types::{
    conversion::v1::context::Contextualize as _,
    error::AggchainProofRequestError,
    v1::{
        aggchain_proof_service_server::AggchainProofService as AggchainProofGrpcService,
        GenerateAggchainProofRequest, GenerateAggchainProofResponse,
        GenerateOptimisticAggchainProofRequest, GenerateOptimisticAggchainProofResponse,
    },
};
use agglayer_interop::{
    grpc::v1::{AggchainProof, Sp1StarkProof},
    types::bincode,
};
use prost::bytes::Bytes;
use prover_executor::sp1_fast;
use sp1_sdk::SP1_CIRCUIT_VERSION;
use tonic::{Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt};
use tower::{buffer::Buffer, Service, ServiceExt};
use tracing::{error, info, instrument};

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

        let last_proven_block = request.last_proven_block;
        let requested_end_block = request.requested_end_block;

        info!(
            %last_proven_block,
            %requested_end_block,
            l1_info_tree_root_hash=%hex::encode(
                request
                    .l1_info_tree_root_hash
                    .as_ref()
                    .map_or(&Bytes::default(), |v| &v.value)
            ),
            "Received GenerateAggchainProof request"
        );

        if request.requested_end_block <= request.last_proven_block {
            let mut error = ErrorDetails::new();
            error.add_bad_request_violation(
                "requested_end_block",
                "requested_end_block must be greater than last_proven_block",
            );

            error!(%last_proven_block, %requested_end_block, ?error,
                "Invalid GenerateAggchainProof request argument(s)");

            return Err(Status::with_error_details(
                tonic::Code::InvalidArgument,
                "Invalid GenerateAggchainProof request argument(s)",
                error,
            ));
        }

        let aggchain_proof_inputs: AggchainProofInputs =
            request
                .try_into()
                .map_err(|error: AggchainProofRequestError| {
                    error!(%last_proven_block, %requested_end_block, ?error, "Invalid GenerateAggchainProof request data");
                    let field = error.field_path();
                    let mut error_details = ErrorDetails::new();
                    error_details.add_bad_request_violation(field, error.to_string());
                    Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Invalid GenerateAggchainProof request data",
                        error_details,
                    )
                })?;

        let mut context = aggchain_proof_inputs.context();

        let proof_request = AggchainProofServiceRequest::Normal(aggchain_proof_inputs);

        let mut service = self.service.clone();

        let service = service
            .ready()
            .await
            .inspect_err(|e| error!(%last_proven_block, %requested_end_block, "Unable to use the aggchain proof service: {e:?} "))
            .map_err(|_| Status::internal("Unable to use the aggchain proof service"))?;

        match service.call(proof_request).await {
            Ok(response) => {
                info!(?response.custom_chain_data,
                    "customchaindata: {}",
                    hex::encode(&response.custom_chain_data)
                );
                context.insert(
                    "public_values".to_owned(),
                    Bytes::from(
                        bincode::sp1v4()
                            .serialize(&response.public_values)
                            .unwrap_or_else(|_| b"bincode serialization failed".to_vec()),
                    ),
                );
                context.insert(
                    "local_exit_root_hash".to_owned(),
                    Bytes::from(response.local_exit_root_hash.as_bytes().to_vec()),
                );
                context.insert(
                    "end_block".to_owned(),
                    Bytes::from(response.end_block.to_be_bytes().to_vec()),
                );
                info!(last_proven_block = %response.last_proven_block,
                    end_block = %response.end_block,
                    "GenerateAggchainProof request executed successfully");
                Ok(Response::new(GenerateAggchainProofResponse {
                    aggchain_proof: Some(AggchainProof {
                        aggchain_params: Some(response.aggchain_params.into()),
                        // Signature is handled by the initiator
                        signature: None,
                        context,
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
                }))
            }
            // TODO: Return a different error when the proof is not yet ready.
            // The gRPC API currently does not expose the status.
            Err(error) => {
                error!(%last_proven_block, %requested_end_block, ?error, "Unable to execute GenerateAggchainProof request");
                Err(Status::internal(error.to_string()))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn generate_optimistic_aggchain_proof(
        &self,
        request: Request<GenerateOptimisticAggchainProofRequest>,
    ) -> Result<Response<GenerateOptimisticAggchainProofResponse>, Status> {
        let request = request.into_inner();

        let aggchain_proof_inputs: OptimisticAggchainProofInputs =
            request
                .try_into()
                .map_err(|error: AggchainProofRequestError| {
                    let field = error.field_path();
                    let mut error_details = ErrorDetails::new();
                    error_details.add_bad_request_violation(field, error.to_string());
                    error!(
                        "Invalid GenerateOptimisticAggchainProof request data: {error_details:?}"
                    );
                    Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Invalid GenerateOptimisticAggchainProof request data",
                        error_details,
                    )
                })?;

        let last_proven_block = aggchain_proof_inputs
            .aggchain_proof_inputs
            .last_proven_block;
        let requested_end_block = aggchain_proof_inputs
            .aggchain_proof_inputs
            .requested_end_block;

        info!(
            %last_proven_block, %requested_end_block,
            l1_info_tree_root_hash=%hex::encode(
                aggchain_proof_inputs
                    .aggchain_proof_inputs
                    .l1_info_tree_root_hash
            ),
            "Received GenerateOptimisticAggchainProof request");

        if aggchain_proof_inputs
            .aggchain_proof_inputs
            .requested_end_block
            <= aggchain_proof_inputs
                .aggchain_proof_inputs
                .last_proven_block
        {
            let mut error = ErrorDetails::new();
            error.add_bad_request_violation(
                "requested_end_block",
                "requested_end_block must be greater than last_proven_block",
            );

            error!(%last_proven_block, %requested_end_block,
                "Invalid GenerateOptimisticAggchainProof request argument(s): {error:?}");

            return Err(Status::with_error_details(
                tonic::Code::InvalidArgument,
                "Invalid GenerateOptimisticAggchainProof request argument(s)",
                error,
            ));
        }

        let mut context = aggchain_proof_inputs.context();

        let proof_request = AggchainProofServiceRequest::Optimistic(aggchain_proof_inputs);

        let mut service = self.service.clone();

        let service = service
            .ready()
            .await
            .inspect_err(|e| error!(%last_proven_block, %requested_end_block, "Unable to use the aggchain proof service: {e:?} "))
            .map_err(|_| Status::internal("Unable to use the aggchain proof service"))?;

        match service.call(proof_request).await {
            Ok(response) => {
                context.insert(
                    "public_values".to_owned(),
                    Bytes::from(
                        sp1_fast(|| bincode::sp1v4().serialize(&response.public_values))
                            .unwrap_or_else(|_| Ok(b"bincode serialization failed".to_vec()))
                            .unwrap_or_else(|_| b"bincode serialization failed".to_vec()),
                    ),
                );
                context.insert(
                    "local_exit_root_hash".to_owned(),
                    Bytes::from(response.local_exit_root_hash.as_bytes().to_vec()),
                );
                context.insert(
                    "end_block".to_owned(),
                    Bytes::from(response.end_block.to_be_bytes().to_vec()),
                );
                info!(last_proven_block = %response.last_proven_block,
                    end_block = %response.end_block,
                    "Generate optimistic aggchain proof request executed successfully");
                Ok(Response::new(GenerateOptimisticAggchainProofResponse {
                    aggchain_proof: Some(AggchainProof {
                        aggchain_params: Some(response.aggchain_params.into()),
                        // Signature is handled by the initiator
                        signature: None,
                        context,
                        proof: Some(agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(
                            Sp1StarkProof {
                                version: SP1_CIRCUIT_VERSION.to_string(),
                                proof: response.proof.into(),
                                vkey: response.vkey.into(),
                            },
                        )),
                    }),
                    local_exit_root_hash: Some(response.local_exit_root_hash.into()),
                    custom_chain_data: response.custom_chain_data.into(),
                }))
            }
            // TODO: Return a different error when the proof is not yet ready.
            // The gRPC API currently does not expose the status.
            Err(error) => {
                error!(%last_proven_block, %requested_end_block, ?error, "Unable to execute GenerateOptimisticAggchainProof request");
                Err(Status::internal(error.to_string()))
            }
        }
    }
}
