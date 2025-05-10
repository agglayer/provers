use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use aggchain_proof_contracts::contracts::L2OutputAtBlock;
use aggchain_proof_contracts::MockContractsClient;
use aggchain_proof_core::full_execution_proof::AggregationProofPublicValues;
use aggchain_proof_service::config::AggchainProofServiceConfig;
use aggchain_proof_service::service::{AggchainProofService, AggchainProofServiceRequest};
use aggchain_proof_types::AggchainProofInputs;
use aggkit_prover_types::v1::{
    aggchain_proof_service_client::AggchainProofServiceClient,
    aggchain_proof_service_server::AggchainProofServiceServer, GenerateAggchainProofRequest,
};
use aggkit_prover_types::Digest;
use agglayer_evm_client::MockRpc;
use agglayer_interop::types::{L1InfoTreeLeaf, L1InfoTreeLeafInner, MerkleProof};
use alloy::eips::BlockNumberOrTag;
use alloy::hex;
use alloy::primitives::{Address, FixedBytes};
use alloy::sol_types::SolValue;
use http::Uri;
use hyper_util::rt::TokioIo;
use mockall::predicate::{always, eq};
use proposer_client::rpc::AggregationProofProposerResponse;
use proposer_client::{MockAggregationProofProposer, MockAggregationProver, RequestId};
use prover_config::{MockProverConfig, ProverType};
use rsp_mpt::EthereumState;
use sp1_cc_client_executor::io::EVMStateSketch;
use sp1_sdk::{CpuProver, Prover as _, SP1ProofMode, SP1PublicValues, SP1_CIRCUIT_VERSION};
use tonic::transport::{Endpoint, Server};
use tonic_types::StatusExt;
use tower::{service_fn, Service};

use crate::rpc::GrpcService;

#[tokio::test]
async fn service_can_be_called() {
    let proving_key = Arc::new(OnceLock::new());
    let mut service_config = AggchainProofServiceConfig::default();
    service_config.aggchain_proof_builder.primary_prover =
        ProverType::MockProver(MockProverConfig::default());
    let mut proposer_l1_rpc = MockRpc::new();
    let mut aggregation_prover = MockAggregationProver::new();
    let mut aggregation_proof_proposer = MockAggregationProofProposer::new();
    let mut contracts_rpc = MockContractsClient::new();
    aggregation_prover
        .expect_compute_pkey_vkey()
        .once()
        .returning({
            let proving_key = proving_key.clone();
            move |elf| {
                let (pk, vk) = CpuProver::mock().setup(elf);
                let _ = proving_key.set(pk.clone());
                (pk, vk)
            }
        });
    proposer_l1_rpc
        .expect_get_block_number()
        .once()
        .with(eq(Digest([1; 32])))
        .return_once(|_| Ok(0));
    aggregation_proof_proposer
        .expect_request_agg_proof()
        .once()
        .return_once(|_| {
            Box::pin(async {
                Ok(AggregationProofProposerResponse {
                    last_proven_block: 0,
                    end_block: 100,
                    request_id: RequestId([2; 32].into()),
                })
            })
        });
    aggregation_prover
        .expect_wait_for_proof()
        .once()
        .with(eq(alloy::primitives::FixedBytes([2; 32].into())), always())
        .return_once(move |_, _| {
            Box::pin(async move {
                // TODO: figure out where these values come from instead of hardcoding them?
                // For now this is probably fine, we can try to figure that out next time we need to update the test
                let empty_root = FixedBytes(hex!(
                    "012893657d8eb2efad4de0a91bcd0e39ad9837745dec3ea923737ea803fc8e3d"
                ));
                let vkey = FixedBytes(hex!(
                    "0367776036b0d8b12720eab775b651c7251e63a249cb84f63eb1c20418b24e9c"
                ));
                Ok(sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
                    proving_key.get().unwrap(),
                    SP1PublicValues::from(
                        &AggregationProofPublicValues {
                            l1Head: FixedBytes([1; 32]),
                            l2PreRoot: empty_root,
                            l2PostRoot: empty_root,
                            l2BlockNumber: 100,
                            rollupConfigHash: Default::default(),
                            multiBlockVKey: vkey,
                        }
                        .abi_encode(),
                    ),
                    SP1ProofMode::Compressed,
                    SP1_CIRCUIT_VERSION,
                ))
            })
        });
    aggregation_prover
        .expect_verify_aggregated_proof()
        .once()
        .return_once(|_, _| Ok(()));
    contracts_rpc
        .expect_get_l2_local_exit_root()
        .once()
        .with(eq(0))
        .return_once(|_| Ok(Digest::ZERO));
    contracts_rpc
        .expect_get_l2_local_exit_root()
        .once()
        .with(eq(100))
        .return_once(|_| Ok(Digest::ZERO));
    contracts_rpc
        .expect_get_l2_output_at_block()
        .once()
        .with(eq(0))
        .return_once(|_| {
            Ok(L2OutputAtBlock {
                version: Default::default(),
                state_root: Default::default(),
                withdrawal_storage_root: Default::default(),
                latest_block_hash: Default::default(),
                output_root: Default::default(),
            })
        });
    contracts_rpc
        .expect_get_l2_output_at_block()
        .once()
        .with(eq(100))
        .return_once(|_| {
            Ok(L2OutputAtBlock {
                version: Default::default(),
                state_root: Default::default(),
                withdrawal_storage_root: Default::default(),
                latest_block_hash: Default::default(),
                output_root: Default::default(),
            })
        });
    contracts_rpc
        .expect_get_rollup_config_hash()
        .once()
        .return_once(|| Ok(Digest::ZERO));
    contracts_rpc
        .expect_get_prev_l2_block_sketch()
        .once()
        .with(eq(BlockNumberOrTag::Number(0)))
        .return_once(|_| {
            Ok(EVMStateSketch {
                header: Default::default(),
                ancestor_headers: Default::default(),
                state: EthereumState {
                    state_trie: Default::default(),
                    storage_tries: Default::default(),
                },
                state_requests: Default::default(),
                bytecodes: Default::default(),
            })
        });
    contracts_rpc
        .expect_get_new_l2_block_sketch()
        .once()
        .with(eq(BlockNumberOrTag::Number(100)))
        .return_once(|_| {
            Ok(EVMStateSketch {
                header: Default::default(),
                ancestor_headers: Default::default(),
                state: EthereumState {
                    state_trie: Default::default(),
                    storage_tries: Default::default(),
                },
                state_requests: Default::default(),
                bytecodes: Default::default(),
            })
        });
    contracts_rpc
        .expect_get_trusted_sequencer_address()
        .once()
        .return_once(|| Ok(Address::ZERO));
    let mut service = AggchainProofService::mocked(
        &service_config,
        Arc::new(proposer_l1_rpc),
        aggregation_prover,
        aggregation_proof_proposer,
        Arc::new(contracts_rpc),
    )
    .await
    .expect("create aggchain proof service");
    // This was found out by patching interop to display the hashes in verify(),
    // and then running the aggchain-proof. We should probably streamline this
    // process if we ever need to modify this value. This being said, we probably
    // will never need to actually modify this value anyway.
    let resulting_l1_root = Digest::from(hex!(
        "7ce16c3d770b018d0e6d387ff74132c3c7cf00e3b3d4eedf87a90052b5ae9099"
    ));
    let request = AggchainProofServiceRequest {
        aggchain_proof_inputs: AggchainProofInputs {
            last_proven_block: 0,
            requested_end_block: 100,
            l1_info_tree_root_hash: resulting_l1_root,
            l1_info_tree_leaf: L1InfoTreeLeaf {
                l1_info_tree_index: 1,
                rer: Default::default(),
                mer: Default::default(),
                inner: L1InfoTreeLeafInner {
                    global_exit_root: Default::default(),
                    block_hash: Digest([1; 32]),
                    timestamp: 0u64,
                },
            },
            l1_info_tree_merkle_proof: MerkleProof::new(resulting_l1_root, [Digest::default(); 32]),
            ger_leaves: Default::default(),
            imported_bridge_exits: Default::default(),
        },
    };
    service.call(request).await.unwrap();
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
        last_proven_block: 1000,
        requested_end_block: 999,
        l1_info_tree_root_hash: None,
        l1_info_tree_leaf: None,
        l1_info_tree_merkle_proof: None,
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
        assert_eq!(violation.field, "requested_end_block");
        assert_eq!(
            violation.description,
            "requested_end_block must be greater than last_proven_block"
        );
    });
}
