use std::collections::HashMap;

use aggchain_proof_service::config::AggchainProofServiceConfig;
use aggchain_proof_service::service::{AggchainProofService, AggchainProofServiceRequest};
use aggkit_prover_types::v1::{
    aggchain_proof_service_client::AggchainProofServiceClient,
    aggchain_proof_service_server::AggchainProofServiceServer, GenerateAggchainProofRequest,
};
use http::Uri;
use hyper_util::rt::TokioIo;
use tonic::transport::{Endpoint, Server};
use tonic_types::StatusExt;
use tower::{service_fn, Service};

use crate::rpc::GrpcService;

#[tokio::test]
#[ignore]
async fn service_can_be_called() {
    std::env::set_var(
        "NETWORK_PRIVATE_KEY",
        "0xaabbccddff000000000000000000000000000000000000000000000000000000",
    );
    let mut service = AggchainProofService::new(&AggchainProofServiceConfig::default())
        .await
        .expect("create aggchain proof service");
    let request = AggchainProofServiceRequest::default();
    let response = service.call(request).await;
    assert!(response.is_ok());
}

#[tokio::test]
#[ignore]
async fn testing_rpc_failure() {
    std::env::set_var(
        "NETWORK_PRIVATE_KEY",
        "0xaabbccddff000000000000000000000000000000000000000000000000000000",
    );

    let (client, server) = tokio::io::duplex(1024);

    let service = GrpcService::new(&AggchainProofServiceConfig::default())
        .await
        .expect("create grpc service");

    tokio::spawn(async move {
        Server::builder()
            .add_service(AggchainProofServiceServer::new(service))
            .serve_with_incoming(tokio_stream::once(Ok::<_, std::io::Error>(server)))
            .await
    });

    // Move client to an option so we can _move_ the inner value
    // on the first attempt to connect. All other attempts will fail.
    let mut client = Some(client);
    let channel = Endpoint::try_from("http://[::]:50051")
        .expect("valid endpoint")
        .connect_with_connector(service_fn(move |_: Uri| {
            let client = client.take();

            async move {
                if let Some(client) = client {
                    Ok(TokioIo::new(client))
                } else {
                    Err(std::io::Error::other("Client already taken"))
                }
            }
        }))
        .await
        .unwrap();

    let mut client = AggchainProofServiceClient::new(channel);

    let request = tonic::Request::new(GenerateAggchainProofRequest {
        start_block: 1000,
        max_end_block: 999,
        l1_info_tree_root_hash: vec![],
        l1_info_tree_leaf: None,
        l1_info_tree_merkle_proof: vec![],
        ger_leaves: HashMap::new(),
        imported_bridge_exits: vec![],
    });

    let response = client.generate_aggchain_proof(request).await;

    assert!(response.is_err());

    let error = response.unwrap_err();
    assert_eq!(error.code(), tonic::Code::InvalidArgument);
    let details = error.get_error_details();

    assert!(details.has_bad_request_violations());
    let violations = &details.bad_request().unwrap().field_violations;

    assert_eq!(violations.len(), 1);

    violations.iter().for_each(|violation| {
        assert_eq!(violation.field, "max_end_block");
        assert_eq!(
            violation.description,
            "max_end_block must be greater than start_block"
        );
    });
}
