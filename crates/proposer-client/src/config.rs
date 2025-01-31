use std::net::Ipv4Addr;
use std::time::Duration;

use prover_utils::from_env_or_default;
use serde::{Deserialize, Serialize};

/// The default port for the proposer GRPC service
const DEFAULT_PROPOSER_CLIENT_PORT: u16 = 5432;
const DEFAULT_PROPOSER_CLIENT_HOST: &str = "127.0.0.1";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerClientConfig {
    /// The proposer http endpoint host
    #[serde(default = "default_host")]
    pub proposer_host: Ipv4Addr,
    /// The proposer http endpoint port
    #[serde(default = "default_port")]
    pub proposer_port: u16,
    /// The proposer http endpoint host
    #[serde(default = "default_host")]
    pub sp1_cluster_host: Ipv4Addr,
    /// The proposer http endpoint port
    #[serde(default = "default_port")]
    pub sp1_cluster_port: u16,
    /// Network prover program
    pub prover_program: Vec<u8>,
    /// Proving timeout in seconds
    #[serde(default = "default_duration")]
    pub proving_timeout: Duration,
}

/// The default port for the proposer GRPC service
/// If the `GRPC_PORT` environment variable is set, it will take precedence over
fn default_port() -> u16 {
    from_env_or_default("PROPOSER_CLIENT_PORT", DEFAULT_PROPOSER_CLIENT_PORT)
}

/// The default host for the proposer GRPC service
fn default_host() -> Ipv4Addr {
    from_env_or_default(
        "PROPOSER_CLIENT_HOST",
        DEFAULT_PROPOSER_CLIENT_HOST.to_string(),
    )
    .parse::<Ipv4Addr>()
    .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
}

fn default_duration() -> Duration {
    Duration::from_secs(3600)
}
