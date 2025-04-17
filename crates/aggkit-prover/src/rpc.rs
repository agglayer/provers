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
use agglayer_interop::types::U256;
use prost::bytes::Bytes;
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

        let mut context = HashMap::new();
        macro_rules! context_field {
            ($name:ident: $($data:tt)*) => {
                context.insert(stringify!($name).to_owned(), Bytes::from(aggchain_proof_inputs.$($data)*.to_vec()));
            };
        }
        context_field!(last_proven_block: last_proven_block.to_be_bytes());
        context_field!(requested_end_block: requested_end_block.to_be_bytes());
        context_field!(l1_info_tree_root_hash: l1_info_tree_root_hash.as_bytes());
        context_field!(l1_info_tree_index: l1_info_tree_leaf.l1_info_tree_index.to_be_bytes());
        context_field!(l1_info_tree_rer: l1_info_tree_leaf.rer.as_bytes());
        context_field!(l1_info_tree_mer: l1_info_tree_leaf.mer.as_bytes());
        context_field!(l1_info_tree_ger: l1_info_tree_leaf.inner.global_exit_root.as_bytes());
        context_field!(l1_info_tree_block_hash: l1_info_tree_leaf.inner.block_hash.as_bytes());
        context_field!(l1_info_tree_timestamp: l1_info_tree_leaf.inner.timestamp.to_be_bytes());
        macro_rules! int_to_bytes {
            ($val:expr) => {
                Bytes::from($val.to_be_bytes().to_vec())
            };
        }
        for (name, ger) in aggchain_proof_inputs.ger_leaves.iter() {
            context.insert(
                format!("ger/{name}/block_number"),
                int_to_bytes!(ger.block_number),
            );
            context.insert(
                format!("ger/{name}/block_index"),
                int_to_bytes!(ger.block_index),
            );
            context.insert(
                format!("ger/{name}/l1_leaf_index"),
                int_to_bytes!(ger.inserted_ger.l1_leaf.l1_info_tree_index),
            );
        }
        for (i, ibe) in aggchain_proof_inputs
            .imported_bridge_exits
            .iter()
            .enumerate()
        {
            context.insert(
                format!("ibe/{i}/block_number"),
                int_to_bytes!(ibe.block_number),
            );
            context.insert(
                format!("ibe/{i}/bridge_exit_hash"),
                Bytes::from(ibe.bridge_exit_hash.0.as_bytes().to_vec()),
            );
            let global_index: U256 = ibe.global_index.into();
            context.insert(
                format!("ibe/{i}/global_index"),
                Bytes::from(
                    global_index
                        .as_le_bytes()
                        .iter()
                        .rev()
                        .copied()
                        .collect::<Vec<_>>(),
                ),
            );
        }

        let proof_request = AggchainProofServiceRequest {
            aggchain_proof_inputs,
        };

        let mut service = self.service.clone();

        let service = service
            .ready()
            .await
            .map_err(|_| Status::internal("Unable to get the service"))?;

        match service.call(proof_request).await {
            Ok(response) => {
                context.insert(
                    "public_values".to_owned(),
                    Bytes::from(
                        bincode::serialize(&response.public_values)
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
                Ok(Response::new(GenerateAggchainProofResponse {
                    aggchain_proof: Some(AggchainProof {
                        aggchain_params: Some(response.aggchain_params.into()),
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
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
