use std::time::Duration;

use alloy_primitives::B256;
use prost::bytes::Bytes;

use crate::{
    rpc::{
        grpc::{AggProofRequest, AggProofResponse},
        AggregationProofProposer, AggregationProofProposerRequest, ProposerRpcClient,
    },
    tests::{mock_server, mock_server::MockProofsService},
    RequestId,
};

fn create_agg_proof_request() -> AggregationProofProposerRequest {
    AggregationProofProposerRequest {
        last_proven_block: 500,
        requested_end_block: 550,
        l1_block_number: 101,
        l1_block_hash: [23; 32].into(),
    }
}

async fn create_mock_grpc_proposer<F>(
    request: AggProofRequest,
    response_handler: F,
) -> anyhow::Result<mock_server::Handle>
where
    F: Fn(
            tonic::Request<AggProofRequest>,
        ) -> Result<tonic::Response<AggProofResponse>, tonic::Status>
        + Send
        + Sync
        + 'static,
{
    let mut server = MockProofsService::new();

    server
        .expect_request_agg_proof()
        .with(mockall::predicate::function(
            move |req: &tonic::Request<AggProofRequest>| req.get_ref() == &request,
        ))
        .returning(response_handler);

    server.run().await
}

#[test_log::test(tokio::test)]
async fn request_an_aggregated_span_proof() {
    let proof_request_id = [1u8; 32];

    let request = create_agg_proof_request();

    let expected_grpc_req = AggProofRequest {
        last_proven_block: request.last_proven_block,
        requested_end_block: request.requested_end_block,
        l1_block_number: request.l1_block_number,
        l1_block_hash: hex::encode(request.l1_block_hash),
    };

    let server = create_mock_grpc_proposer(
        expected_grpc_req,
        move |req: tonic::Request<AggProofRequest>| {
            let req = req.into_inner();
            let response = AggProofResponse {
                last_proven_block: req.last_proven_block,
                end_block: req.requested_end_block,
                proof_request_id: Bytes::from_owner(proof_request_id.to_vec()),
            };
            Ok(tonic::Response::new(response))
        },
    )
    .await
    .expect("valid mock server");

    let service = ProposerRpcClient::new(server.uri(), Duration::from_millis(500))
        .await
        .unwrap();

    let response = service
        .request_agg_proof(request.clone())
        .await
        .expect("successful reply");

    assert_eq!(response.request_id, RequestId(B256::new(proof_request_id)));
    assert_eq!(response.last_proven_block, request.last_proven_block);
    assert_eq!(response.end_block, request.requested_end_block);

    server.stop().await.unwrap();
}

#[test_log::test(tokio::test)]
async fn request_and_receive_an_error() {
    let request = create_agg_proof_request();

    let expected_grpc_req = AggProofRequest {
        last_proven_block: request.last_proven_block,
        requested_end_block: request.requested_end_block,
        l1_block_number: request.l1_block_number,
        l1_block_hash: hex::encode(request.l1_block_hash),
    };

    let server = create_mock_grpc_proposer(
        expected_grpc_req,
        move |_req: tonic::Request<AggProofRequest>| {
            Err(tonic::Status::new(
                tonic::Code::Unknown,
                "Service was not ready",
            ))
        },
    )
    .await
    .expect("valid mock server");

    let service = ProposerRpcClient::new(server.uri(), Duration::from_millis(500))
        .await
        .unwrap();
    let response = service.request_agg_proof(request.clone()).await;

    assert!(response.is_err());
    if let Err(crate::error::Error::Requesting(ref err)) = response {
        if let crate::error::ProofRequestError::Grpc(ref status) = **err {
            assert_eq!(status.code(), tonic::Code::Unknown);
            assert_eq!(status.message(), "Service was not ready");
        } else {
            panic!("Expected a grpcerror");
        }
    } else {
        panic!("Expected an invalid request error");
    }

    server.stop().await.unwrap();
}

#[test_log::test(tokio::test)]
async fn receive_end_block_higher_than_last_chain_block() {
    let request = AggregationProofProposerRequest {
        last_proven_block: 452,
        requested_end_block: 10000,
        l1_block_number: 253,
        l1_block_hash: [23; 32].into(),
    };

    let expected_grpc_req = AggProofRequest {
        last_proven_block: request.last_proven_block,
        requested_end_block: request.requested_end_block,
        l1_block_number: request.l1_block_number,
        l1_block_hash: hex::encode(request.l1_block_hash),
    };

    let server = create_mock_grpc_proposer(
        expected_grpc_req,
        move |_req: tonic::Request<AggProofRequest>| {
            Err(tonic::Status::new(
                tonic::Code::NotFound,
                "No consecutive span proof range found",
            ))
        },
    )
    .await
    .expect("valid mock server");

    let service = ProposerRpcClient::new(server.uri(), Duration::from_millis(500))
        .await
        .unwrap();
    let response = service.request_agg_proof(request.clone()).await;

    assert!(response.is_err());
    if let Err(crate::error::Error::Requesting(ref err)) = response {
        if let crate::error::ProofRequestError::Grpc(ref status) = **err {
            assert_eq!(status.code(), tonic::Code::NotFound);
            assert_eq!(status.message(), "No consecutive span proof range found");
        } else {
            panic!("Expected a grpcerror");
        }
    } else {
        panic!("Expected an invalid request error");
    }

    server.stop().await.unwrap();
}

#[test_log::test(tokio::test)]
async fn receive_an_invalid_start_end_block() {
    let request = AggregationProofProposerRequest {
        last_proven_block: 200,
        requested_end_block: 100,
        l1_block_number: 253,
        l1_block_hash: [23; 32].into(),
    };

    let expected_grpc_req = AggProofRequest {
        last_proven_block: request.last_proven_block,
        requested_end_block: request.requested_end_block,
        l1_block_number: request.l1_block_number,
        l1_block_hash: hex::encode(request.l1_block_hash),
    };

    let server = create_mock_grpc_proposer(
        expected_grpc_req,
        move |_req: tonic::Request<AggProofRequest>| {
            Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "Requested end block must be greater than the last proven block",
            ))
        },
    )
    .await
    .expect("valid mock server");

    let service = ProposerRpcClient::new(server.uri(), Duration::from_millis(500))
        .await
        .unwrap();
    let response = service.request_agg_proof(request.clone()).await;

    assert!(response.is_err());
    if let Err(crate::error::Error::Requesting(ref err)) = response {
        if let crate::error::ProofRequestError::Grpc(ref status) = **err {
            assert_eq!(status.code(), tonic::Code::InvalidArgument);
            assert_eq!(
                status.message(),
                "Requested end block must be greater than the last proven block"
            );
        } else {
            panic!("Expected a grpcerror");
        }
    } else {
        panic!("Expected an invalid request error");
    }

    server.stop().await.unwrap();
}
