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

mod noop {
    use std::sync::Arc;

    use aggchain_proof_contracts::contracts::{
        GetTrustedSequencerAddress, L1L2Output, L1LatestL2OutputFetcher, L1LocalExitRootFetcher,
        L1OpSuccinctConfigFetcher, L1OptimisticModeFetcher, L2LocalExitRootFetcher,
        L2OutputAtBlock, L2OutputAtBlockFetcher, OpSuccinctConfig,
    };
    use aggchain_proof_core::full_execution_proof::{FepInputs, KoalaBearDigest};
    use aggchain_proof_types::{
        imported_bridge_exit::{BridgeExitHash, ImportedBridgeExitWithBlockNumber},
        unclaim::UnclaimWithBlockNumber,
        AggchainProofInputs,
    };
    use agglayer_interop::types::{bincode, L1InfoTreeLeaf, L1InfoTreeLeafInner, MerkleProof};
    use agglayer_primitives::{address, keccak::keccak256, Address, Digest, Signature, U256};
    use unified_bridge::{
        AggchainProofPublicValues, GlobalIndex, GlobalIndexWithLeafHash,
        ImportedBridgeExitCommitmentValues,
    };

    use crate::{
        latest_l1_pre_root, AggchainProofBuilder, AggchainProofBuilderRequest, Error,
        FepVerification, IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION,
    };

    const TRUSTED_SEQUENCER: Address = address!("0x1111111111111111111111111111111111111111");
    const L1_PRE_ROOT: Digest = Digest([0xAAu8; 32]);

    fn fep_inputs() -> FepInputs {
        FepInputs {
            l1_head: Digest([1u8; 32]),
            claim_block_num: 42,
            rollup_config_hash: Digest([2u8; 32]),
            prev_state_root: Digest([3u8; 32]),
            prev_withdrawal_storage_root: Digest([4u8; 32]),
            prev_block_hash: Digest([5u8; 32]),
            new_state_root: Digest([6u8; 32]),
            new_withdrawal_storage_root: Digest([7u8; 32]),
            new_block_hash: Digest([8u8; 32]),
            aggregation_vkey_hash: KoalaBearDigest([1, 2, 3, 4, 5, 6, 7, 8]),
            range_vkey_commitment: [10u8; 32],
            trusted_sequencer: TRUSTED_SEQUENCER,
            signature_optimistic_mode: Some(Signature::new(U256::ZERO, U256::ZERO, false)),
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
            l1_head_inclusion_proof: MerkleProof::new(Digest::ZERO, [Digest::ZERO; 32]),
        }
    }

    struct RecoveryContracts {
        latest_l2_output: Option<L1L2Output>,
        optimistic_mode: bool,
    }

    #[async_trait::async_trait]
    impl L1LatestL2OutputFetcher for RecoveryContracts {
        async fn get_latest_l2_output(
            &self,
        ) -> Result<Option<L1L2Output>, aggchain_proof_contracts::Error> {
            Ok(self.latest_l2_output)
        }
    }

    #[async_trait::async_trait]
    impl L2LocalExitRootFetcher for RecoveryContracts {
        async fn get_l2_local_exit_root(
            &self,
            block_number: u64,
        ) -> Result<Digest, aggchain_proof_contracts::Error> {
            assert_eq!(
                block_number, 42,
                "recovery must not read the L2 state at the anchor"
            );
            Ok(Digest([2; 32]))
        }
    }

    #[async_trait::async_trait]
    impl L1LocalExitRootFetcher for RecoveryContracts {
        async fn get_l1_last_local_exit_root(
            &self,
        ) -> Result<Digest, aggchain_proof_contracts::Error> {
            Ok(Digest([1; 32]))
        }
    }

    #[async_trait::async_trait]
    impl L1OptimisticModeFetcher for RecoveryContracts {
        async fn get_optimistic_mode(&self) -> Result<bool, aggchain_proof_contracts::Error> {
            Ok(self.optimistic_mode)
        }
    }

    #[async_trait::async_trait]
    impl L2OutputAtBlockFetcher for RecoveryContracts {
        async fn get_l2_output_at_block(
            &self,
            block_number: u64,
        ) -> Result<L2OutputAtBlock, aggchain_proof_contracts::Error> {
            assert_eq!(
                block_number, 42,
                "recovery must not fetch the old L2 output"
            );
            Ok(L2OutputAtBlock {
                output_root: fep_inputs().compute_claim_root().0,
                ..Default::default()
            })
        }
    }

    #[async_trait::async_trait]
    impl L1OpSuccinctConfigFetcher for RecoveryContracts {
        async fn get_op_succinct_config(
            &self,
        ) -> Result<OpSuccinctConfig, aggchain_proof_contracts::Error> {
            let inputs = fep_inputs();
            Ok(OpSuccinctConfig {
                rollup_config_hash: inputs.rollup_config_hash,
                range_vkey_commitment: Digest(inputs.range_vkey_commitment),
                aggregation_vkey_hash: Digest(inputs.aggregation_vkey_hash.to_hash_bn254()),
            })
        }
    }

    #[async_trait::async_trait]
    impl GetTrustedSequencerAddress for RecoveryContracts {
        async fn get_trusted_sequencer_address(
            &self,
        ) -> Result<Address, aggchain_proof_contracts::Error> {
            Ok(TRUSTED_SEQUENCER)
        }
    }

    #[tokio::test]
    async fn recovery_public_values_use_l1_anchor_and_filter_claims() -> eyre::Result<()> {
        let latest_l2_output = Some(L1L2Output {
            output_root: L1_PRE_ROOT,
            l2_block_number: 41,
        });

        for (l1_optimistic_mode, optimistic_request) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            let contracts = Arc::new(RecoveryContracts {
                latest_l2_output,
                optimistic_mode: l1_optimistic_mode,
            });
            let pre_root = latest_l1_pre_root(contracts.as_ref(), 41).await?;
            assert_eq!(pre_root, L1_PRE_ROOT);

            // The params hash follows the L1 flag, whatever the request type.
            let mut inputs = fep_inputs();
            if !l1_optimistic_mode {
                inputs.signature_optimistic_mode = None;
            }
            let fep_verification = if optimistic_request {
                FepVerification::Optimistic {
                    signature: Signature::new(U256::ZERO, U256::ZERO, false),
                }
            } else {
                FepVerification::Recovery
            };
            let request = AggchainProofBuilderRequest {
                fep_verification,
                end_block: 42,
                aggchain_proof_inputs: AggchainProofInputs {
                    last_proven_block: 41,
                    requested_end_block: 42,
                    l1_info_tree_root_hash: Digest([3; 32]),
                    l1_info_tree_leaf: inputs.l1_info_tree_leaf.clone(),
                    l1_info_tree_merkle_proof: inputs.l1_head_inclusion_proof.clone(),
                    ger_leaves: Default::default(),
                    imported_bridge_exits: [(43, 3u8), (42, 2), (41, 4), (42, 1)]
                        .into_iter()
                        .map(
                            |(block_number, leaf_index)| ImportedBridgeExitWithBlockNumber {
                                block_number,
                                bridge_exit_hash: BridgeExitHash(Digest([leaf_index; 32])),
                                global_index: GlobalIndex::new(1.into(), leaf_index.into()),
                                log_index: leaf_index.into(),
                            },
                        )
                        .collect(),
                    removed_gers: vec![],
                    unclaims: [(42, 2), (43, 1)]
                        .into_iter()
                        .map(|(block_number, leaf_index)| UnclaimWithBlockNumber {
                            block_number,
                            global_index: GlobalIndex::new(1.into(), leaf_index).into(),
                            log_index: 0,
                        })
                        .collect(),
                },
            };
            let mut result = AggchainProofBuilder::retrieve_recovery_data(
                contracts.clone(),
                request,
                7,
                pre_root,
            )
            .await?;
            let public_values: AggchainProofPublicValues = result.stdin.read();
            let mut packed = inputs.encoded_aggchain_params();
            packed[..32].copy_from_slice(&L1_PRE_ROOT.0);
            let commitment = ImportedBridgeExitCommitmentValues {
                claims: vec![GlobalIndexWithLeafHash {
                    global_index: GlobalIndex::new(1.into(), 1).into(),
                    bridge_exit_hash: Digest([1; 32]),
                }],
            }
            .commitment(IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION);

            assert_eq!(result.output_root.0, inputs.compute_claim_root().0);
            assert_eq!(
                public_values,
                AggchainProofPublicValues {
                    prev_local_exit_root: Digest([1; 32]),
                    new_local_exit_root: Digest([2; 32]),
                    l1_info_root: Digest([3; 32]),
                    origin_network: 7.into(),
                    commit_imported_bridge_exits: commitment,
                    aggchain_params: keccak256(&packed),
                }
            );
            assert_ne!(public_values.aggchain_params, inputs.aggchain_params());
        }
        Ok(())
    }

    #[tokio::test]
    async fn recovery_rejects_missing_or_mismatched_anchor() {
        for latest in [
            None,
            Some(L1L2Output {
                output_root: L1_PRE_ROOT,
                l2_block_number: 40,
            }),
        ] {
            assert!(matches!(
                latest_l1_pre_root(
                    &RecoveryContracts {
                        latest_l2_output: latest,
                        optimistic_mode: false,
                    },
                    41
                )
                .await,
                Err(Error::RecoveryAnchorMismatch { last_proven_block: 41, l1_latest_output_block })
                    if l1_latest_output_block == latest.map(|output| output.l2_block_number)
            ));
        }
    }

    #[tokio::test]
    async fn recovery_rejects_an_empty_range() {
        let inputs = fep_inputs();
        for end_block in [40, 41] {
            let request = AggchainProofBuilderRequest {
                fep_verification: FepVerification::Recovery,
                end_block,
                aggchain_proof_inputs: AggchainProofInputs {
                    last_proven_block: 41,
                    requested_end_block: end_block,
                    l1_info_tree_root_hash: Digest::ZERO,
                    l1_info_tree_leaf: inputs.l1_info_tree_leaf.clone(),
                    l1_info_tree_merkle_proof: inputs.l1_head_inclusion_proof.clone(),
                    ger_leaves: Default::default(),
                    imported_bridge_exits: vec![],
                    removed_gers: vec![],
                    unclaims: vec![],
                },
            };
            let contracts = Arc::new(RecoveryContracts {
                latest_l2_output: None,
                optimistic_mode: false,
            });
            assert!(matches!(
                AggchainProofBuilder::retrieve_recovery_data(contracts, request, 7, L1_PRE_ROOT)
                    .await,
                Err(Error::RecoveryEmptyRange {
                    last_proven_block: 41,
                    end_block: got,
                }) if got == end_block
            ));
        }
    }

    /// The committed noop ELF commits exactly the public values it reads, so a
    /// stale ELF (program changed without `AGGLAYER_ELF_BUILD=update`) fails
    /// here rather than in the skip-proof-verification or recovery mode.
    #[tokio::test]
    async fn noop_elf_commits_the_given_public_values() {
        let public_values = AggchainProofPublicValues {
            prev_local_exit_root: Digest([1u8; 32]),
            new_local_exit_root: Digest([2u8; 32]),
            l1_info_root: Digest([3u8; 32]),
            origin_network: 7.into(),
            commit_imported_bridge_exits: Digest([4u8; 32]),
            aggchain_params: Digest([5u8; 32]),
        };
        let stdin = crate::noop_stdin(&public_values).unwrap();

        let committed =
            prover_executor::ExecutionLimiter::new(1, std::time::Duration::from_secs(30))
                .execute(crate::AGGCHAIN_PROOF_NOOP_ELF, stdin)
                .await
                .unwrap();
        let committed: AggchainProofPublicValues = bincode::sp1_compatible()
            .deserialize(committed.as_slice())
            .unwrap();

        assert_eq!(committed, public_values);
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
                private_stdin: false,
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
