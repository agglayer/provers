use std::path::Path;

use aggkit_prover_config::ProverConfig as Config;
use insta::assert_snapshot;
use pretty_assertions::assert_eq;

fn toml_snapshot_string<T: serde::Serialize>(value: &T) -> String {
    let cur_dir = Path::new("./").canonicalize().unwrap();
    toml::to_string_pretty(value)
        .unwrap()
        .replace(cur_dir.to_str().unwrap(), "/tmp/agglayer")
}

#[test]
fn empty_rpcs() {
    let input = "./tests/fixtures/validate_config/empty_rpcs.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_snapshot!(toml_snapshot_string(&config));
}

#[test]
fn prover_grpc_max_decoding_message_size() {
    let input = "./tests/fixtures/validate_config/prover_grpc_max_decoding_message_size.toml";

    let config: Config = toml::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();

    assert_snapshot!(toml_snapshot_string(&config));

    assert_eq!(config.grpc.max_decoding_message_size, 100 * 1024 * 1024);
}
