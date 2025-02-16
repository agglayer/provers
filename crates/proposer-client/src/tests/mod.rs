mod proposer_rpc {
    use alloy_primitives::B256;
    use serde_json::json;

    use crate::rpc::{AggSpanProofProposer, AggSpanProofProposerRequest, ProposerRpcClient};

    #[tokio::test]
    async fn request_an_aggregated_span_proof() {
        let mut server = mockito::Server::new_async().await;

        let block_hash: B256 = [23u8; 32].into();
        let proof_request_id = hex::encode([1u8; 32]);

        let mock = server
            .mock("POST", "/request_agg_proof")
            .with_status(201)
            .with_header("content-type", "text/javascript")
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "startBlock": 110,
                "maxBlock": 200,
                "l1BlockNumber": 230203,
                "l1BlockHash": block_hash
            })))
            .with_body(json!({ "start_block": 110, "end_block": 200, "proof_request_id": proof_request_id }).to_string())
            .create();

        let service = ProposerRpcClient::new(&server.url()).unwrap();

        let request = AggSpanProofProposerRequest {
            start: 110,
            end: 200,
            l1_block_number: 230203,
            l1_block_hash: [23; 32].into(),
        };

        let response = service.request_agg_proof(request).await;

        assert!(response.is_ok());
        mock.assert_async().await;
    }

    #[test]
    #[ignore = "to be implemented"]
    fn request_and_receive_an_error() {}

    #[test]
    #[ignore = "to be implemented"]
    fn request_with_a_valid_proof_id() {}

    #[test]
    #[ignore = "to be implemented"]
    fn request_with_an_invalid_proof_id() {}

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
}
