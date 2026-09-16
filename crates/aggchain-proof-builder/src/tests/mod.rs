use crate::AggchainProverInputs;

#[allow(unused)]
pub fn dump_aggchain_prover_inputs_json(
    aggchain_prover_inputs: &AggchainProverInputs,
    last_proven_block: u64,
    end_block: u64,
) -> eyre::Result<()> {
    use std::io::Write;
    let file_name =
        format!("aggchain_prover_inputs_001_lpb_{last_proven_block}_eb_{end_block}.json",);
    let mut file = std::fs::File::create(file_name)?;
    let data = serde_json::to_string(&aggchain_prover_inputs)?;
    write!(file, "{data}")?;
    Ok(())
}

pub fn load_aggchain_prover_inputs_json(file_name: &str) -> eyre::Result<AggchainProverInputs> {
    let data: String = std::fs::read_to_string(file_name)?;
    let aggchain_prover_inputs: AggchainProverInputs = serde_json::from_str(&data)?;
    Ok(aggchain_prover_inputs)
}

#[tokio::test]
async fn mock_vkey_matches_mock_elf() -> eyre::Result<()> {
    use sp1_sdk::HashableKey as _;

    let vkey =
        prover_executor::Executor::compute_program_vkey(crate::AGGCHAIN_PROOF_MOCK_ELF).await?;
    let derived = vkey.hash_bytes();
    assert_eq!(
        derived,
        crate::MOCK_VKEY,
        "MOCK_VKEY is stale, mock elf vkey is 0x{}",
        alloy_primitives::hex::encode(derived)
    );
    Ok(())
}

mod optimistic_safe_block {
    use std::{collections::HashMap, sync::Arc};

    use aggchain_proof_contracts::{
        contracts::{
            GetTrustedSequencerAddress, L1OpSuccinctConfigFetcher, L2EvmStateSketchFetcher,
            L2LocalExitRootFetcher, L2OutputAtBlock, L2OutputAtBlockFetcher, L2SafeBlockFetcher,
            OpSuccinctConfig,
        },
        Error as ContractsError,
    };
    use aggchain_proof_types::AggchainProofInputs;
    use agglayer_interop::types::{L1InfoTreeLeaf, L1InfoTreeLeafInner, MerkleProof};
    use agglayer_primitives::{Address, Digest, Signature, U256};
    use alloy::eips::BlockNumberOrTag;
    use sp1_cc_client_executor::io::EvmSketchInput;

    use crate::{AggchainProofBuilder, AggchainProofBuilderRequest, Error, FepVerification};

    /// Contracts client that only answers the safe block query. The first call
    /// after the safe block check fails with a recognizable error so the test
    /// can tell the check passed without going through the rest of the flow.
    struct StubContractsClient {
        safe_block_number: u64,
    }

    #[async_trait::async_trait]
    impl L2SafeBlockFetcher for StubContractsClient {
        async fn get_l2_safe_block_number(&self) -> Result<u64, ContractsError> {
            Ok(self.safe_block_number)
        }
    }

    #[async_trait::async_trait]
    impl L2LocalExitRootFetcher for StubContractsClient {
        async fn get_l2_local_exit_root(&self, _: u64) -> Result<Digest, ContractsError> {
            Err(ContractsError::Other(eyre::eyre!(
                "stop after the safe block check"
            )))
        }
    }

    #[async_trait::async_trait]
    impl L2OutputAtBlockFetcher for StubContractsClient {
        async fn get_l2_output_at_block(&self, _: u64) -> Result<L2OutputAtBlock, ContractsError> {
            unreachable!("not reached after the local exit root failure")
        }
    }

    #[async_trait::async_trait]
    impl L1OpSuccinctConfigFetcher for StubContractsClient {
        async fn get_op_succinct_config(&self) -> Result<OpSuccinctConfig, ContractsError> {
            unreachable!("not reached after the local exit root failure")
        }
    }

    #[async_trait::async_trait]
    impl GetTrustedSequencerAddress for StubContractsClient {
        async fn get_trusted_sequencer_address(&self) -> Result<Address, ContractsError> {
            unreachable!("not reached after the local exit root failure")
        }
    }

    #[async_trait::async_trait]
    impl L2EvmStateSketchFetcher for StubContractsClient {
        async fn get_prev_l2_block_sketch(
            &self,
            _: BlockNumberOrTag,
        ) -> Result<EvmSketchInput, ContractsError> {
            unreachable!("not reached after the local exit root failure")
        }

        async fn get_new_l2_block_sketch(
            &self,
            _: BlockNumberOrTag,
        ) -> Result<EvmSketchInput, ContractsError> {
            unreachable!("not reached after the local exit root failure")
        }
    }

    fn optimistic_request(end_block: u64) -> AggchainProofBuilderRequest {
        AggchainProofBuilderRequest {
            fep_verification: FepVerification::Optimistic {
                signature: Signature::new(U256::ZERO, U256::ZERO, false),
            },
            end_block,
            aggchain_proof_inputs: AggchainProofInputs {
                last_proven_block: 0,
                requested_end_block: end_block,
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
                removed_gers: vec![],
                unclaims: vec![],
            },
        }
    }

    async fn retrieve_chain_data(
        safe_block_number: u64,
        end_block: u64,
    ) -> Result<crate::AggchainProverInputs, Error> {
        AggchainProofBuilder::<StubContractsClient>::retrieve_chain_data(
            Arc::new(StubContractsClient { safe_block_number }),
            optimistic_request(end_block),
            1,
            Arc::new(proposer_elfs::aggregation::vkey().clone()),
            Address::ZERO,
            Digest::ZERO,
        )
        .await
    }

    #[tokio::test]
    async fn rejects_end_block_beyond_safe_head() {
        let result = retrieve_chain_data(100, 101).await;

        assert!(matches!(
            result,
            Err(Error::OptimisticEndBlockNotSafe {
                end_block: 101,
                safe_block_number: 100,
            })
        ));
    }

    #[tokio::test]
    async fn accepts_end_block_at_safe_head() {
        let result = retrieve_chain_data(100, 100).await;

        assert!(matches!(result, Err(Error::L2ChainDataRetrievalError(_))));
    }
}

mod aggchain_proof_builder {
    use std::time::Duration;

    use eyre::Context as _;
    use prover_config::{NetworkProverConfig, ProverType};
    use prover_executor::Executor;
    use tower::{buffer::Buffer, Service, ServiceExt};

    use crate::{
        tests::load_aggchain_prover_inputs_json, AggchainProverInputs, Error, ProverService,
    };

    async fn init_network_prover() -> eyre::Result<ProverService> {
        let executor = Executor::new(
            ProverType::NetworkProver(NetworkProverConfig {
                proving_timeout: Duration::from_secs(3600),
                proving_request_timeout: Some(Duration::from_secs(600)),
                sp1_cluster_endpoint: "https://rpc.production.succinct.xyz/".parse()?,
            }),
            None,
            crate::AGGCHAIN_PROOF_ELF,
        )
        .await
        .context("Failed initializing network prover for AggchainProofBuilder")?;
        let executor = tower::ServiceBuilder::new().service(executor).boxed();
        let prover = Buffer::new(executor, 10);
        Ok(prover)
    }

    #[tokio::test]
    #[ignore = "requires network key, run manually"]
    async fn execute_aggchain_program_test() -> eyre::Result<()> {
        let mut prover = init_network_prover().await?;

        let aggchain_prover_inputs: AggchainProverInputs = load_aggchain_prover_inputs_json(
            "src/tests/data/aggchain_prover_inputs_001_lpb_1_eb_4.json",
        )?;

        let prover_executor::Response { proof } = prover
            .ready()
            .await
            .map_err(Error::ProverServiceReadyError)?
            .call(prover_executor::Request {
                stdin: aggchain_prover_inputs.stdin,
                proof_type: prover_executor::ProofType::Stark,
            })
            .await
            .map_err(Error::ProverFailedToExecute)?;

        println!("Prover executor successfully returned response: {proof:?}");

        Ok(())
    }
}
