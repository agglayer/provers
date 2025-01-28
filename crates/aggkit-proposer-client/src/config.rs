use std::net::Ipv4Addr;

use prover_utils::from_env_or_default;
use serde::{Deserialize, Serialize};

/// The default port for the proposer GRPC service
const DEFAULT_PROPOSER_PORT: u16 = 5432;
const DEFAULT_PROPOSER_HOST: &str = "127.0.0.1";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerClientConfig {
    /// The proposer gRPC endpoint port
    #[serde(default = "default_port")]
    pub port: u16,
    /// The proposer gRPC endpoint host
    #[serde(default = "default_host")]
    pub host: Ipv4Addr,
}

/// The default port for the proposer GRPC service
/// If the `GRPC_PORT` environment variable is set, it will take precedence over
fn default_port() -> u16 {
    from_env_or_default("AGGKIT_PROPOSER_PORT", DEFAULT_PROPOSER_PORT)
}

/// The default host for the proposer GRPC service
fn default_host() -> Ipv4Addr {
    from_env_or_default("AGGKIT_PROPOSER_HOST", DEFAULT_PROPOSER_HOST.to_string())
        .parse::<Ipv4Addr>()
        .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
}
