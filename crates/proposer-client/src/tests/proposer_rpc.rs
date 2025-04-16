use std::time::Duration;

use alloy_primitives::B256;
use prost::bytes::Bytes;

use crate::{
    rpc::{
        grpc::{AggProofRequest, AggProofResponse},
        AggregationProofProposer, AggregationProofProposerRequest, ProposerRpcClient,
    },
    tests::mock_server::MockProofsService,
    RequestId,
};

#[test_log::test(tokio::test)]
async fn request_an_aggregated_span_proof() {
    let proof_request_id = [1u8; 32];

    let request = AggregationProofProposerRequest {
        last_proven_block: 110,
        requested_end_block: 200,
        l1_block_number: 230203,
        l1_block_hash: [23; 32].into(),
    };

    let server = {
        let mut mock_service = MockProofsService::new();

        let expected_req = AggProofRequest {
            last_proven_block: request.last_proven_block,
            requested_end_block: request.requested_end_block,
            l1_block_number: request.l1_block_number,
            l1_block_hash: hex::encode(request.l1_block_hash),
        };

        mock_service
            .expect_request_agg_proof()
            .with(mockall::predicate::function(
                move |req: &tonic::Request<AggProofRequest>| req.get_ref() == &expected_req,
            ))
            .returning(move |_request| {
                let response = AggProofResponse {
                    success: true,
                    error: String::new(),
                    last_proven_block: 110,
                    end_block: 205,
                    proof_request_id: Bytes::from_owner(proof_request_id.to_vec()),
                };
                Ok(tonic::Response::new(response))
            });

        mock_service.run().await.unwrap()
    };

    let service = ProposerRpcClient::new(
        format!("http://{}", server.local_addr()).parse().unwrap(),
        Duration::from_millis(500),
    )
    .await
    .unwrap();

    let response = service
        .request_agg_proof(request)
        .await
        .expect("successful reply");

    assert_eq!(response.request_id, RequestId(B256::new(proof_request_id)));
    assert_eq!(response.last_proven_block, 110);
    assert_eq!(response.end_block, 205);

    server.stop().await.unwrap();
}

// TODO: Implement this test in next iteration
#[test]
#[ignore = "to be implemented"]
fn request_and_receive_an_error() {}

#[test]
#[ignore = "to be implemented"]
fn request_with_a_valid_request_id() {}

#[test]
#[ignore = "to be implemented"]
fn request_with_an_invalid_request_id() {}

#[test]
#[ignore = "to be implemented"]
fn receive_an_invalid_start_block() {}

#[test]
#[ignore = "to be implemented"]
fn receive_an_invalid_end_block() {}

#[test]
#[ignore = "to be implemented"]
fn receive_an_invalid_start_block_and_end_block() {}

#[test]
#[ignore = "to be implemented"]
fn receive_an_end_block_higher_than_max_block() {}
