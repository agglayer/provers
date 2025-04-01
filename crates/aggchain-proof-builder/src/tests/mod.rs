use crate::AggchainProverInputs;

#[allow(unused)]
pub fn dump_aggchain_prover_inputs_json(
    aggchain_prover_inputs: &AggchainProverInputs,
) -> Result<(), anyhow::Error> {
    use std::io::Write;
    let file_name = format!(
        "aggchain_prover_inputs_001_lpb_{}_eb_{}.json",
        aggchain_prover_inputs.last_proven_block, aggchain_prover_inputs.end_block
    );
    let mut file = std::fs::File::create(file_name)?;
    let data = serde_json::to_string(&aggchain_prover_inputs)?;
    write!(file, "{}", data)?;
    Ok(())
}

pub fn load_aggchain_prover_inputs_json(
    file_name: &str,
) -> Result<AggchainProverInputs, anyhow::Error> {
    let data: String = std::fs::read_to_string(file_name)?;
    let aggchain_prover_inputs: AggchainProverInputs = serde_json::from_str(&data)?;
    Ok(aggchain_prover_inputs)
}

mod aggchain_proof_builder {
    use std::time::Duration;

    use prover_config::{NetworkProverConfig, ProverType};
    use prover_executor::Executor;
    use tower::buffer::Buffer;
    use tower::{Service, ServiceExt};

    use crate::tests::load_aggchain_prover_inputs_json;
    use crate::{AggchainProverInputs, Error, ProverService};

    fn init_test_prover() -> Result<ProverService, anyhow::Error> {
        let executor = Executor::new(
            &ProverType::NetworkProver(NetworkProverConfig {
                proving_timeout: Duration::from_secs(3600),
                proving_request_timeout: Some(Duration::from_secs(600)),
            }),
            &None,
            crate::AGGCHAIN_PROOF_ELF,
        );
        let executor = tower::ServiceBuilder::new().service(executor).boxed();
        let prover = Buffer::new(executor, 10);
        Ok(prover)
    }

    #[tokio::test]
    async fn execute_aggchain_program_test() -> Result<(), Box<dyn std::error::Error>> {
        let mut prover = init_test_prover()?;

        let aggchain_prover_inputs: AggchainProverInputs = load_aggchain_prover_inputs_json(
            "src/tests/data/aggchain_prover_inputs_001_lpb_18_eb_21.json",
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
            .map_err(|error| Error::ProverFailedToExecute(anyhow::Error::from_boxed(error)))?;

        println!(
            "Prover executor successfully returned response: {:?} ",
            proof
        );

        Ok(())
    }
}
