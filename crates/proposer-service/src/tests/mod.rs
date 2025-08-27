use std::sync::Arc;

use agglayer_evm_client::MockRpc;
use alloy_primitives::FixedBytes;
use proposer_client::{
    rpc::AggregationProofProposerRequest, FepProposerRequest, MockProposerClient, RequestId,
};
use sp1_sdk::{Prover as _, SP1PublicValues, SP1_CIRCUIT_VERSION};
use tower::Service as _;

use crate::{Error, ProposerService};

const ELF: &[u8] = include_bytes!("../../../prover-dummy-program/elf/riscv32im-succinct-zkvm-elf");

fn generate_keys() -> (
    sp1_sdk::SP1ProvingKey,
    sp1_sdk::SP1VerifyingKey,
    SP1PublicValues,
) {
    use alloy_primitives::B256;
    use serde::{Deserialize, Serialize};

    let client = sp1_sdk::ProverClient::builder().mock().build();
    let (pk, vk) = client.setup(ELF);

    #[derive(Default, Serialize, Deserialize)]
    struct TestAggregationOutputs {
        l1_head: B256,
        l2_pre_root: B256,
        l2_post_root: B256,
        l2_block_number: u64,
        rollup_config_hash: B256,
        multi_block_vkey: B256,
        prover_address: B256,
    }
    let data = TestAggregationOutputs::default();
    let public_values = SP1PublicValues::from(
        &aggkit_prover_types::bincode::sp1v4()
            .serialize(&data)
            .unwrap(),
    );
    (pk, vk, public_values)
}

#[tokio::test]
async fn test_proposer_service() {
    let mut l1_rpc = MockRpc::new();

    l1_rpc
        .expect_get_block_number()
        .once()
        .returning(|_| Ok(10));

    let mut client = MockProposerClient::new();
    client.expect_request_agg_proof().once().returning(
        |request: AggregationProofProposerRequest| {
            Box::pin(async move {
                Ok(proposer_client::rpc::AggregationProofProposerResponse {
                    request_id: RequestId(FixedBytes::new([0; 32])),
                    last_proven_block: request.last_proven_block,
                    end_block: request.requested_end_block,
                })
            })
        },
    );

    let (pkey, vkey, public_values) = generate_keys();
    {
        let mock_proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
            &pkey,
            public_values,
            sp1_sdk::SP1ProofMode::Compressed,
            SP1_CIRCUIT_VERSION,
        );

        client
            .expect_wait_for_proof()
            .once()
            .return_once(move |_| Box::pin(async move { Ok(mock_proof) }));

        client
            .expect_verify_agg_proof()
            .once()
            .return_once(move |_, _, _| Ok(()));
    };

    let client = Arc::new(client);
    let l1_rpc = Arc::new(l1_rpc);
    let mut proposer_service = ProposerService {
        client,
        l1_rpc,
        aggregation_vkey: vkey,
    };

    let request = FepProposerRequest {
        last_proven_block: 0,
        requested_end_block: 10,
        l1_block_hash: Default::default(),
    };

    let response = proposer_service.call(request).await.unwrap();
    assert_eq!(response.last_proven_block, 0);
}

#[tokio::test]
async fn unable_to_fetch_block_hash() {
    let mut l1_rpc = MockRpc::new();
    l1_rpc
        .expect_get_block_number()
        .once()
        .returning(|_| eyre::bail!("Failed to fetch block number"));

    let client = MockProposerClient::new();

    let (_pkey, vkey, _public_values) = generate_keys();

    let client = Arc::new(client);
    let l1_rpc = Arc::new(l1_rpc);
    let mut proposer_service = ProposerService {
        client,
        l1_rpc,
        aggregation_vkey: vkey,
    };

    let request = FepProposerRequest {
        last_proven_block: 0,
        requested_end_block: 10,
        l1_block_hash: Default::default(),
    };

    let response = proposer_service.call(request).await;
    assert!(response.is_err());
    assert!(matches!(
        response.unwrap_err(),
        Error::AlloyProviderError(_)
    ));
}

#[test]
#[ignore = "to be implemented"]
fn test_invalid_proof_vkey_verificatinon_fails() {}
