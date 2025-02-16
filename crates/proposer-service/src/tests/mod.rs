use std::sync::Arc;

use alloy_primitives::{FixedBytes, B256};
use proposer_client::{rpc::AggSpanProofProposerRequest, MockProposerClient, ProposerRequest};
use prover_alloy::MockProvider;
use sp1_sdk::{Prover as _, SP1_CIRCUIT_VERSION};
use tower::Service as _;

use crate::Error;
use crate::ProposerService;

const ELF: &[u8] = include_bytes!("../../../prover-dummy-program/elf/riscv32im-succinct-zkvm-elf");

#[tokio::test]
async fn test_proposer_service() {
    let mut l1_rpc = MockProvider::new();
    l1_rpc
        .expect_get_block_hash()
        .once()
        .returning(|_| Box::pin(async { Ok(B256::ZERO) }));

    let mut client = MockProposerClient::new();
    client
        .expect_request_agg_proof()
        .once()
        .returning(|request: AggSpanProofProposerRequest| {
            Box::pin(async move {
                Ok(proposer_client::rpc::AggSpanProofProposerResponse {
                    proof_id: FixedBytes::new([0; 32]),
                    start_block: request.start,
                    end_block: request.end,
                })
            })
        });

    let stdin = sp1_sdk::SP1Stdin::new();

    let (pk, public_values) = {
        let client = sp1_sdk::ProverClient::builder().cpu().build();

        let (pk, _vk) = client.setup(ELF);
        let (public_values, _) = client.execute(&pk.elf, &stdin).run().unwrap();

        (pk, public_values)
    };

    client.expect_wait_for_proof().once().return_once(move |_| {
        Box::pin(async move {
            Ok(sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
                &pk,
                public_values,
                sp1_sdk::SP1ProofMode::Compressed,
                SP1_CIRCUIT_VERSION,
            ))
        })
    });

    let client = Arc::new(client);
    let l1_rpc = Arc::new(l1_rpc);
    let mut proposer_service = ProposerService { client, l1_rpc };

    let request = ProposerRequest {
        start_block: 0,
        max_block: 10,
        l1_block_number: 0,
    };

    let response = proposer_service.call(request).await.unwrap();
    assert_eq!(response.start_block, 0);
}

#[tokio::test]
async fn unable_to_fetch_block_hash() {
    let mut l1_rpc = MockProvider::new();
    l1_rpc
        .expect_get_block_hash()
        .once()
        .returning(|_| Box::pin(async { anyhow::bail!("Failed to fetch block hash") }));

    let client = MockProposerClient::new();

    let client = Arc::new(client);
    let l1_rpc = Arc::new(l1_rpc);
    let mut proposer_service = ProposerService { client, l1_rpc };

    let request = ProposerRequest {
        start_block: 0,
        max_block: 10,
        l1_block_number: 0,
    };

    let response = proposer_service.call(request).await;
    assert!(response.is_err());
    assert!(matches!(
        response.unwrap_err(),
        Error::AlloyProviderError(_)
    ));
}
