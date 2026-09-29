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

mod aggregation_vkey {
    use aggchain_proof_core::{
        full_execution_proof::{hash_bn254_bytes, AggregationProofPublicValues},
        vkey_hash::HashU32,
    };
    use agglayer_primitives::{Digest, Signature, U256};
    use prover_executor::Executor;
    use sp1_sdk::{HashableKey as _, SP1ProofMode, SP1ProofWithPublicValues, SP1_CIRCUIT_VERSION};

    use crate::{Error, FepVerification, AGGCHAIN_PROOF_ELF};

    async fn proof_verification() -> eyre::Result<(FepVerification, HashU32)> {
        let vkey = Executor::compute_program_vkey(AGGCHAIN_PROOF_ELF).await?;
        let digest = vkey.hash_u32();
        let proof = SP1ProofWithPublicValues::create_mock_proof(
            &vkey,
            Default::default(),
            SP1ProofMode::Compressed,
            SP1_CIRCUIT_VERSION,
        );
        Ok((
            FepVerification::Proof {
                aggregation_proof: Box::new(proof),
                aggregation_vkey: Box::new(vkey),
                aggregation_proof_public_values: AggregationProofPublicValues {
                    l1_head: Default::default(),
                    l2_pre_root: Default::default(),
                    l2_post_root: Default::default(),
                    l2_block_number: 0,
                    rollup_config_hash: Default::default(),
                    multi_block_vkey: Default::default(),
                    prover_address: Default::default(),
                },
            },
            digest,
        ))
    }

    fn optimistic_verification() -> FepVerification {
        FepVerification::Optimistic {
            signature: Signature::new(U256::ZERO, U256::ZERO, false),
        }
    }

    #[tokio::test]
    async fn accepts_aggregation_key_matching_l1() -> eyre::Result<()> {
        let (verification, digest) = proof_verification().await?;
        let configured_hash = Digest(hash_bn254_bytes(digest));

        assert_eq!(
            verification.validate_aggregation_vkey_hash(configured_hash)?,
            digest,
        );
        Ok(())
    }

    #[tokio::test]
    async fn rejects_aggregation_key_mismatching_l1() -> eyre::Result<()> {
        let (verification, digest) = proof_verification().await?;
        let proven_hash = Digest(hash_bn254_bytes(digest));
        let error = verification
            .validate_aggregation_vkey_hash(Digest::ZERO)
            .expect_err("a key different from L1 must fail");

        assert!(matches!(
            error,
            Error::MismatchAggregationVkeyHash { got, expected }
                if got == Digest::ZERO && expected == proven_hash
        ));
        Ok(())
    }

    #[tokio::test]
    async fn rejects_invalid_l1_key_in_both_modes() -> eyre::Result<()> {
        let (proof, _) = proof_verification().await?;
        let invalid_hash = Digest([0xff; 32]);
        for verification in [proof, optimistic_verification()] {
            let error = verification
                .validate_aggregation_vkey_hash(invalid_hash)
                .expect_err("an invalid packed key must fail");

            assert!(matches!(
                error,
                Error::InvalidAggregationVkeyHash(hash) if hash == invalid_hash
            ));
        }
        Ok(())
    }

    #[test]
    fn optimistic_mode_uses_l1_aggregation_key() -> eyre::Result<()> {
        let digest = [1, 2, 3, 4, 5, 6, 7, 8];
        let configured_hash = Digest(hash_bn254_bytes(digest));

        assert_eq!(
            optimistic_verification().validate_aggregation_vkey_hash(configured_hash)?,
            digest,
        );
        Ok(())
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
