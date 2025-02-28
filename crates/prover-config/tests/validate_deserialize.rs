use pretty_assertions::assert_eq;
use prover_config::{CpuProverConfig, MockProverConfig, NetworkProverConfig, ProverType, SindriProverConfig};
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
        })
    );
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

#[test]
fn sindri_prover() {
    let input = "./tests/fixtures/validate_config/prover_config_sindri_prover.toml";
    let config: TestConfig = toml::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();

    assert_eq!(
        config.primary_prover,
        ProverType::SindriProver(SindriProverConfig {
            proving_request_timeout: Some(std::time::Duration::from_secs(300)),
            proving_timeout: std::time::Duration::from_secs(600),
        })
    );
}
