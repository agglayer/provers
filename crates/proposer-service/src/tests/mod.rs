use std::sync::Arc;

use aggchain_proof_contracts::contracts::{ChainIdProvider, GetTrustedSequencerAddress, L1OpSuccinctConfigFetcher, OpSuccinctConfig};
use agglayer_primitives::Address;
use agglayer_evm_client::MockRpc;
use agglayer_interop::types::Digest;
use alloy_primitives::{FixedBytes, U64};
use mockall::mock;
use proposer_client::{
    rpc::AggregationProofProposerRequest, FepProposerRequest, MockProposerClient, RequestId,
};
use sp1_sdk::{Prover as _, SP1PublicValues, SP1_CIRCUIT_VERSION};
use tower::Service as _;

use crate::l2_rpc::{BlockId, MockL2SafeHeadFetcher, SafeHeadAtL1Block};
use crate::{Error, ProofBackend, ProposerService};

mock! {
    pub ContractsClient {}

    #[async_trait::async_trait]
    impl L1OpSuccinctConfigFetcher for ContractsClient {
        async fn get_op_succinct_config(&self) -> Result<OpSuccinctConfig, aggchain_proof_contracts::Error>;
    }

    #[async_trait::async_trait]
    impl GetTrustedSequencerAddress for ContractsClient {
        async fn get_trusted_sequencer_address(&self) -> Result<Address, aggchain_proof_contracts::Error>;
    }

    impl ChainIdProvider for ContractsClient {
        fn l1_chain_id(&self) -> u64;
        fn l2_chain_id(&self) -> u64;
    }
}

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

    let mut l2_rpc = MockL2SafeHeadFetcher::new();
    l2_rpc.expect_get_safe_head_at_l1_block().returning(|_| {
        Ok(SafeHeadAtL1Block {
            l1_block: BlockId {
                number: U64::from(10),
                hash: Default::default(),
            },
            safe_head: BlockId {
                number: U64::from(100),
                hash: Default::default(),
            },
        })
    });
    let l2_rpc = Arc::new(l2_rpc);

    let mut contracts_client = MockContractsClient::new();
    contracts_client.expect_get_op_succinct_config().returning(|| {
        Ok(OpSuccinctConfig {
            range_vkey_commitment: Digest::default(),
            aggregation_vkey_hash: Digest::default(),
            rollup_config_hash: Digest::default(),
        })
    });
    contracts_client.expect_get_trusted_sequencer_address().returning(|| {
        Ok(Address::new([0u8; 20]))
    });
    let contracts_client = Arc::new(contracts_client);

    let mut proposer_service = ProposerService::new(
        ProofBackend::Grpc {
            client,
            poll_interval_ms: 5000,
            max_retries: 720,
        },
        l1_rpc,
        l2_rpc,
        contracts_client,
        vkey,
        0,
        0,
        false,
    );

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

    let mut l2_rpc = MockL2SafeHeadFetcher::new();
    l2_rpc.expect_get_safe_head_at_l1_block().returning(|_| {
        Ok(SafeHeadAtL1Block {
            l1_block: BlockId {
                number: U64::from(10),
                hash: Default::default(),
            },
            safe_head: BlockId {
                number: U64::from(100),
                hash: Default::default(),
            },
        })
    });
    let l2_rpc = Arc::new(l2_rpc);

    let mut contracts_client = MockContractsClient::new();
    contracts_client.expect_get_op_succinct_config().returning(|| {
        Ok(OpSuccinctConfig {
            range_vkey_commitment: Digest::default(),
            aggregation_vkey_hash: Digest::default(),
            rollup_config_hash: Digest::default(),
        })
    });
    contracts_client.expect_get_trusted_sequencer_address().returning(|| {
        Ok(Address::new([0u8; 20]))
    });
    let contracts_client = Arc::new(contracts_client);

    let mut proposer_service = ProposerService::new(
        ProofBackend::Grpc {
            client,
            poll_interval_ms: 5000,
            max_retries: 720,
        },
        l1_rpc,
        l2_rpc,
        contracts_client,
        vkey,
        0,
        0,
        false,
    );

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
