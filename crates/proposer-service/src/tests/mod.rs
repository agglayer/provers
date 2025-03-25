use std::sync::Arc;

use alloy_primitives::{FixedBytes};
use proposer_client::{
    rpc::AggregationProofProposerRequest, FepProposerRequest, MockProposerClient,
};
use prover_alloy::MockProvider;
use sp1_sdk::{Prover as _, SP1PublicValues, SP1_CIRCUIT_VERSION};
use tower::Service as _;

use crate::{Error, ProposerService};

const ELF: &[u8] = include_bytes!("../../../prover-dummy-program/elf/riscv32im-succinct-zkvm-elf");

fn generate_keys() -> (
    sp1_sdk::SP1ProvingKey,
    sp1_sdk::SP1VerifyingKey,
    SP1PublicValues,
) {
    let stdin = sp1_sdk::SP1Stdin::new();
    let client = sp1_sdk::ProverClient::builder().mock().build();
    let (pk, vk) = client.setup(ELF);
    let (public_values, _) = client.execute(&pk.elf, &stdin).run().unwrap();
    (pk, vk, public_values)
}

#[tokio::test]
async fn test_proposer_service() {
    let mut l1_rpc = MockProvider::new();

    l1_rpc
        .expect_get_block_number()
        .once()
        .returning(|_| Box::pin(async { Ok(10) }));

    let mut client = MockProposerClient::new();
    client.expect_request_agg_proof().once().returning(
        |request: AggregationProofProposerRequest| {
            Box::pin(async move {
                Ok(proposer_client::rpc::AggregationProofProposerResponse {
                    request_id: FixedBytes::new([0; 32]),
                    start_block: request.start_block,
                    end_block: request.max_block,
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
        aggregation_vkey_hash: vkey,
    };

    let request = FepProposerRequest {
        start_block: 0,
        max_block: 10,
        l1_block_hash: Default::default(),
    };

    let response = proposer_service.call(request).await.unwrap();
    assert_eq!(response.start_block, 0);
}

#[tokio::test]
async fn unable_to_fetch_block_hash() {
    let mut l1_rpc = MockProvider::new();
    l1_rpc
        .expect_get_block_number()
        .once()
        .returning(|_| Box::pin(async { anyhow::bail!("Failed to fetch block number") }));

    let client = MockProposerClient::new();

    let (_pkey, vkey, _public_values) = generate_keys();

    let client = Arc::new(client);
    let l1_rpc = Arc::new(l1_rpc);
    let mut proposer_service = ProposerService {
        client,
        l1_rpc,
        aggregation_vkey_hash: vkey,
    };

    let request = FepProposerRequest {
        start_block: 0,
        max_block: 10,
        l1_block_hash: Default::default(),
    };

    let response = proposer_service.call(request).await;
    assert!(response.is_err());
    assert!(matches!(
        response.unwrap_err(),
        Error::AlloyProviderError(_)
    ));
}
