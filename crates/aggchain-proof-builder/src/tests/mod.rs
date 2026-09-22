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
    use aggchain_proof_core::full_execution_proof::{FepInputs, KoalaBearDigest};
    use agglayer_interop::types::{bincode, L1InfoTreeLeaf, L1InfoTreeLeafInner, MerkleProof};
    use agglayer_primitives::{address, keccak::keccak256, Address, Digest, Signature, U256};
    use sp1_sdk::{HashableKey as _, LightProver, Prover as _, SP1Stdin};
    use unified_bridge::AggchainProofPublicValues;

    use crate::NoopAggchainParams;

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
            aggregation_vkey_hash: KoalaBearDigest(proposer_elfs::aggregation::vkey().hash_u32()),
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

    /// The noop params built from the same values as the program's inputs,
    /// with the given pre-root.
    fn noop_params(fep_inputs: &FepInputs, l2_pre_root: Digest) -> NoopAggchainParams {
        NoopAggchainParams {
            l2_pre_root,
            claim_root: fep_inputs.compute_claim_root().0,
            claim_block_num: fep_inputs.claim_block_num.into(),
            rollup_config_hash: fep_inputs.rollup_config_hash,
            optimistic_mode: fep_inputs.signature_optimistic_mode.is_some(),
            trusted_sequencer: fep_inputs.trusted_sequencer,
            range_vkey_commitment: Digest(fep_inputs.range_vkey_commitment),
            aggregation_vkey_hash: Digest(fep_inputs.aggregation_vkey_hash.to_hash_bn254()),
        }
    }

    /// The noop params pack like the program's own encoding, with the L1
    /// pre-root in the first field and nothing else changed.
    #[test]
    fn params_with_the_l1_pre_root_replace_only_the_pre_root() {
        let fep_inputs = fep_inputs();
        let params = noop_params(&fep_inputs, L1_PRE_ROOT);

        let mut packed = fep_inputs.encoded_aggchain_params();
        packed[..32].copy_from_slice(&L1_PRE_ROOT.0);
        let expected = keccak256(&packed);

        assert_eq!(params.hash(), expected);
        assert_ne!(params.hash(), fep_inputs.aggchain_params());
    }

    /// The committed noop ELF commits exactly the public values it reads, so a
    /// stale ELF (program changed without `AGGLAYER_ELF_BUILD=update`) fails
    /// here rather than at recovery time.
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
        let mut stdin = SP1Stdin::new();
        stdin.write(&public_values);

        let prover = LightProver::new().await;
        let (committed, _) = prover
            .execute(crate::AGGCHAIN_PROOF_NOOP_ELF.into(), stdin)
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
