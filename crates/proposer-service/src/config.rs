use std::str::FromStr;
use std::time::Duration;

use prover_utils::from_env_or_default;
use serde::{Deserialize, Serialize};
use url::Url;

/// The default proposer service endpoint
const DEFAULT_PROPOSER_SERVICE_ENDPOINT: &str = "http://proposer-mock-rpc:3000";

/// The default url endpoint for the grpc cluster service
const DEFAULT_SP1_CLUSTER_ENDPOINT: &str = "https://rpc.production.succinct.xyz/";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerServiceConfig {
    pub client: ProposerClientConfig,

    /// JSON-RPC endpoint of the l1 node.
    #[serde(default = "prover_alloy::default_l1_url")]
    pub l1_rpc_endpoint: Url,
}

impl Default for ProposerServiceConfig {
    fn default() -> Self {
        Self {
            client: ProposerClientConfig::default(),
            l1_rpc_endpoint: prover_alloy::default_l1_url(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerClientConfig {
    /// The proposer service http endpoint
    #[serde(default = "default_proposer_service_endpoint")]
    pub proposer_endpoint: Url,
    /// The sp1 proving cluster endpoint
    #[serde(default = "default_sp1_cluster_endpoint")]
    pub sp1_cluster_endpoint: Url,
    /// Proving timeout in seconds
    #[serde(default = "default_timeout")]
    pub proving_timeout: Duration,
}

impl Default for ProposerClientConfig {
    fn default() -> Self {
        Self {
            proposer_endpoint: default_proposer_service_endpoint(),
            sp1_cluster_endpoint: default_sp1_cluster_endpoint(),
            proving_timeout: default_timeout(),
        }
    }
}

fn default_proposer_service_endpoint() -> Url {
    from_env_or_default(
        "PROPOSER_SERVICE_ENDPOINT",
        Url::from_str(DEFAULT_PROPOSER_SERVICE_ENDPOINT).unwrap(),
    )
}

fn default_sp1_cluster_endpoint() -> Url {
    from_env_or_default(
        "SP1_CLUSTER_ENDPOINT",
        Url::from_str(DEFAULT_SP1_CLUSTER_ENDPOINT).unwrap(),
    )
}

fn default_timeout() -> Duration {
    Duration::from_secs(3600)
}
