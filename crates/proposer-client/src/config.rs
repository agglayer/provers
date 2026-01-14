use std::{str::FromStr, time::Duration};

use prover_utils::from_env_or_default;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use url::Url;

/// The default url endpoint for the grpc cluster service
const DEFAULT_SP1_CLUSTER_ENDPOINT: &str = "https://rpc.production.succinct.xyz/";

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerClientConfig {
    /// The sp1 proving cluster endpoint.
    #[serde(default = "default_sp1_cluster_endpoint")]
    pub sp1_cluster_endpoint: Url,

    /// Proving timeout in seconds.
    #[serde(default = "default_proving_timeout")]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub proving_timeout: Duration,
}

impl Default for ProposerClientConfig {
    fn default() -> Self {
        Self {
            sp1_cluster_endpoint: default_sp1_cluster_endpoint(),
            proving_timeout: default_proving_timeout(),
        }
    }
}

fn default_sp1_cluster_endpoint() -> Url {
    from_env_or_default(
        "SP1_CLUSTER_ENDPOINT",
        Url::from_str(DEFAULT_SP1_CLUSTER_ENDPOINT).unwrap(),
    )
}

pub fn default_proving_timeout() -> Duration {
    Duration::from_secs(3600)
}
