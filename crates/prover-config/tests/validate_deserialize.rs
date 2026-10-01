use pretty_assertions::assert_eq;
use prover_config::{CpuProverConfig, MockProverConfig, NetworkProverConfig, ProverType};
use rstest::rstest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct TestConfig {
    primary_prover: ProverType,
    fallback_prover: Option<ProverType>,
}

#[test]
fn network_prover() {
    let input = "./tests/fixtures/validate_config/prover_config_network_prover.toml";
    let config: TestConfig = toml::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();

    assert_eq!(
        config.primary_prover,
        ProverType::NetworkProver(NetworkProverConfig {
            proving_request_timeout: Some(std::time::Duration::from_secs(300)),
            proving_timeout: std::time::Duration::from_secs(600),
            sp1_cluster_endpoint: url::Url::parse("https://rpc.production.succinct.xyz/").unwrap(),
            private_stdin: false,
        })
    );
}

#[test]
fn network_prover_defaults() -> Result<(), toml::de::Error> {
    let config = NetworkProverConfig::default();
    assert!(!config.private_stdin);
    assert_eq!(toml::from_str::<NetworkProverConfig>("")?, config);
    Ok(())
}

#[rstest]
#[case("private-stdin = true", true)]
#[case("private-stdin = false", false)]
fn network_prover_private_stdin(
    #[case] input: &str,
    #[case] expected: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config: NetworkProverConfig = toml::from_str(input)?;
    assert_eq!(config.private_stdin, expected);
    assert_eq!(
        toml::from_str::<NetworkProverConfig>(&toml::to_string(&config)?)?,
        config,
    );
    Ok(())
}

#[test]
fn cpu_prover() {
    let input = "./tests/fixtures/validate_config/prover_config_cpu_prover.toml";
    let config: TestConfig = toml::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();

    assert_eq!(
        config.primary_prover,
        ProverType::CpuProver(CpuProverConfig {
            max_concurrency_limit: 10,
            proving_request_timeout: Some(std::time::Duration::from_secs(300)),
            proving_timeout: std::time::Duration::from_secs(600),
        })
    );
}

#[test]
fn network_and_cpu_prover() {
    let input = "./tests/fixtures/validate_config/prover_config_primary_fallback_prover.toml";
    let config: TestConfig = toml::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();

    assert_eq!(
        config.primary_prover,
        ProverType::NetworkProver(NetworkProverConfig {
            proving_request_timeout: Some(std::time::Duration::from_secs(300)),
            proving_timeout: std::time::Duration::from_secs(600),
            sp1_cluster_endpoint: url::Url::parse("https://rpc.production.succinct.xyz/").unwrap(),
            private_stdin: false,
        })
    );

    assert_eq!(
        config.fallback_prover,
        Some(ProverType::CpuProver(CpuProverConfig {
            max_concurrency_limit: 10,
            proving_request_timeout: Some(std::time::Duration::from_secs(300)),
            proving_timeout: std::time::Duration::from_secs(600),
        }))
    );
}

#[test]
fn mock_prover() {
    let input = "./tests/fixtures/validate_config/prover_config_mock_prover.toml";
    let config: TestConfig = toml::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();

    assert_eq!(
        config.primary_prover,
        ProverType::MockProver(MockProverConfig {
            max_concurrency_limit: 10,
            proving_request_timeout: Some(std::time::Duration::from_secs(300)),
            proving_timeout: std::time::Duration::from_secs(600),
        })
    );
}
