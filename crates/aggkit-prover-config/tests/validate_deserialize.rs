use std::path::Path;

use aggkit_prover_config::ProverConfig as Config;
use insta::assert_toml_snapshot;
use pretty_assertions::assert_eq;
use prover_config::{CpuProverConfig, MockProverConfig, NetworkProverConfig, ProverType};

#[test]
fn empty_rpcs() {
    let input = "./tests/fixtures/validate_config/empty_rpcs.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_toml_snapshot!(config, {
        ".storage.*" => insta::dynamic_redaction(|value, path| {
            if path.to_string() != "storage.db-path" {
                if let insta::internals::Content::String(path) = value {
                    return insta::internals::Content::String(path.replace(Path::new("./").canonicalize().unwrap().to_str().unwrap(), "/tmp/agglayer"));
                }
            }

            value
        }),
    });
}

#[test]
fn prover_grpc_max_decoding_message_size() {
    let input = "./tests/fixtures/validate_config/prover_grpc_max_decoding_message_size.toml";

    let config: Config = toml::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();

    assert_toml_snapshot!(config, {
        ".storage.*" => insta::dynamic_redaction(|value, path| {
            if path.to_string() != "storage.db-path" {
                if let insta::internals::Content::String(path) = value {
                    return insta::internals::Content::String(path.replace(Path::new("./").canonicalize().unwrap().to_str().unwrap(), "/tmp/agglayer"));
                }
            }

            value
        }),
    });

    assert_eq!(config.grpc.max_decoding_message_size, 100 * 1024 * 1024);
}

#[test]
fn network_prover() {
    let input = "./tests/fixtures/validate_config/prover_config_network_prover.toml";
    let config = Config::try_load(Path::new(input)).unwrap();

    assert_eq!(
        config.primary_prover,
        ProverType::NetworkProver(NetworkProverConfig {
            proving_request_timeout: Some(std::time::Duration::from_secs(300)),
            proving_timeout: std::time::Duration::from_secs(600),
        })
    );

    assert_eq!(config.fallback_prover, None);
}

#[test]
fn cpu_prover() {
    let input = "./tests/fixtures/validate_config/prover_config_cpu_prover.toml";
    let config = Config::try_load(Path::new(input)).unwrap();

    assert_eq!(
        config.primary_prover,
        ProverType::CpuProver(CpuProverConfig {
            max_concurrency_limit: 10,
            proving_request_timeout: Some(std::time::Duration::from_secs(300)),
            proving_timeout: std::time::Duration::from_secs(600),
        })
    );

    assert_eq!(config.fallback_prover, None);
}

#[test]
fn network_and_cpu_prover() {
    let input = "./tests/fixtures/validate_config/prover_config_primary_fallback_prover.toml";
    let config = Config::try_load(Path::new(input)).unwrap();

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
    let config = Config::try_load(Path::new(input)).unwrap();

    assert_eq!(
        config.primary_prover,
        ProverType::MockProver(MockProverConfig {
            max_concurrency_limit: 10,
            proving_request_timeout: Some(std::time::Duration::from_secs(300)),
            proving_timeout: std::time::Duration::from_secs(600),
        })
    );

    assert_eq!(config.fallback_prover, None);
}
