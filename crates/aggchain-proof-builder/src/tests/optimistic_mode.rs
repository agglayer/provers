use std::{collections::HashMap, sync::Arc};

use aggchain_proof_contracts::{
    contracts::L2OutputAtBlock, sp1_cc_client_executor::io::EVMStateSketch,
    MockAggchainContractsClient,
};
use aggchain_proof_core::Digest;
use aggchain_proof_types::AggchainProofInputs;
use agglayer_interop::types::{L1InfoTreeLeaf, L1InfoTreeLeafInner, MerkleProof};
use agglayer_primitives::Signature;
use alloy::consensus::Header;
use alloy_primitives::U256;
use tower::{buffer::Buffer, service_fn, util::BoxService, Service, ServiceExt as _};

use crate::{
    config::AggchainProofBuilderConfig, AggchainProofBuilder, AggchainProofBuilderRequest,
    AggchainProofMode, MAX_CONCURRENT_REQUESTS,
};

#[tokio::test]
async fn creating_service() {
    let config = AggchainProofBuilderConfig::default();
    let mut contracts_client = MockAggchainContractsClient::new();

    contracts_client
        .expect_get_l2_local_exit_root()
        .returning(|_| Ok(Digest::ZERO));
    contracts_client
        .expect_get_l2_output_at_block()
        .returning(|_| Ok(L2OutputAtBlock::default()));
    contracts_client
        .expect_get_rollup_config_hash()
        .returning(|| Ok(Digest::ZERO));
    contracts_client
        .expect_get_prev_l2_block_sketch()
        .returning(|_| {
            Ok(EVMStateSketch {
                header: Header::default(),
                ancestor_headers: vec![],
                state: Ethe,
                state_requests: HashMap::new(),
                bytecodes: vec![],
            })
        });

    let service: BoxService<
        prover_executor::Request,
        prover_executor::Response,
        prover_executor::Error,
    > = service_fn(|_: prover_executor::Request| async { panic!("Shouldn't be called") }).boxed();

    let prover = Buffer::new(service, MAX_CONCURRENT_REQUESTS);
    let mut service =
        AggchainProofBuilder::new(&config, Arc::new(contracts_client), prover).unwrap();

    _ = service
        .call(AggchainProofBuilderRequest {
            aggchain_proof_mode: AggchainProofMode::Optimistic {
                signature: Signature::new(U256::ZERO, U256::ZERO, false),
            },
            end_block: 10,
            aggchain_proof_inputs: AggchainProofInputs {
                last_proven_block: 0,
                requested_end_block: 10,
                l1_info_tree_root_hash: Digest::ZERO,
                l1_info_tree_leaf: L1InfoTreeLeaf {
                    l1_info_tree_index: 0,
                    rer: Digest::ZERO,
                    mer: Digest::ZERO,
                    inner: L1InfoTreeLeafInner {
                        global_exit_root: Digest::ZERO,
                        block_hash: Digest::ZERO,
                        timestamp: 0,
                    },
                },
                l1_info_tree_merkle_proof: MerkleProof::new(Digest::ZERO, [Digest::ZERO; 32]),
                ger_leaves: HashMap::new(),
                imported_bridge_exits: vec![],
            },
        })
        .await;
}
