use std::net::Ipv4Addr;
use std::time::Duration;

use prover_utils::from_env_or_default;
use serde::{Deserialize, Serialize};

/// The default port for the proposer json rpc service
const DEFAULT_PROPOSER_CLIENT_PORT: u16 = 3000;
/// The default host for the proposer json rpc service
const DEFAULT_PROPOSER_CLIENT_HOST: &str = "127.0.0.1";

/// The default host and port for the grpc cluster service
const DEFAULT_PROPOSER_CLUSTER_PORT: u16 = 5432;
const DEFAULT_PROPOSER_CLUSTER_HOST: &str = "127.0.0.1";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerClientConfig {
    /// The proposer http endpoint host
    #[serde(default = "default_proposer_host")]
    pub proposer_host: Ipv4Addr,
    /// The proposer http endpoint port
    #[serde(default = "default_proposer_port")]
    pub proposer_port: u16,
    /// The proving cluster grpc host
    #[serde(default = "default_cluster_host")]
    pub sp1_cluster_host: Ipv4Addr,
    /// The proving cluster grpc port
    #[serde(default = "default_cluster_port")]
    pub sp1_cluster_port: u16,
    /// Network prover program
    pub prover_program: Vec<u8>,
    /// Proving timeout in seconds
    #[serde(default = "default_timeout")]
    pub proving_timeout: Duration,
}

impl Default for ProposerClientConfig {
    fn default() -> Self {
        Self {
            proposer_host: default_proposer_host(),
            proposer_port: default_proposer_port(),
            sp1_cluster_host: default_cluster_host(),
            sp1_cluster_port: default_cluster_port(),
            prover_program: vec![],
            proving_timeout: default_timeout(),
        }
    }
}

/// The default port for the proposer http service
fn default_proposer_port() -> u16 {
    from_env_or_default("PROPOSER_CLIENT_PORT", DEFAULT_PROPOSER_CLIENT_PORT)
}

/// The default host for the proposer http service
fn default_proposer_host() -> Ipv4Addr {
    from_env_or_default(
        "PROPOSER_CLIENT_HOST",
        DEFAULT_PROPOSER_CLIENT_HOST.to_string(),
    )
    .parse::<Ipv4Addr>()
    .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
}

/// The default port for the cluster service
fn default_cluster_port() -> u16 {
    from_env_or_default("PROPOSER_CLUSTER_PORT", DEFAULT_PROPOSER_CLUSTER_PORT)
}

/// The default host for the cluster service
fn default_cluster_host() -> Ipv4Addr {
    from_env_or_default(
        "PROPOSER_CLUSTER_HOST",
        DEFAULT_PROPOSER_CLUSTER_HOST.to_string(),
    )
    .parse::<Ipv4Addr>()
    .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
}

fn default_timeout() -> Duration {
    Duration::from_secs(3600)
}
