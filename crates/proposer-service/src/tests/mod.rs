use aggchain_proof_contracts::contracts::{
    ChainIdProvider, GetTrustedSequencerAddress, L1OpSuccinctConfigFetcher, OpSuccinctConfig,
};
use agglayer_primitives::Address;
use mockall::mock;

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

#[test]
#[ignore = "Requires database connection - to be implemented with integration tests"]
fn test_proposer_service() {}

#[test]
#[ignore = "Requires database connection - to be implemented with integration tests"]
fn unable_to_fetch_block_hash() {}

#[test]
#[ignore = "to be implemented"]
fn test_invalid_proof_vkey_verification_fails() {}
