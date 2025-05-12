use std::str::FromStr;
use std::time::Duration;

use prover_utils::from_env_or_default;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, DurationSeconds};
use url::Url;

use crate::GrpcUri;

/// The default proposer service endpoint
const DEFAULT_PROPOSER_SERVICE_ENDPOINT: &str = "http://proposer-mock-rpc:3000";

/// The default url endpoint for the grpc cluster service
const DEFAULT_SP1_CLUSTER_ENDPOINT: &str = "https://rpc.production.succinct.xyz/";

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerClientConfig {
    /// The proposer service http endpoint.
    #[serde(default = "default_proposer_service_endpoint")]
    #[serde_as(as = "DisplayFromStr")]
    pub proposer_endpoint: GrpcUri,

    /// The sp1 proving cluster endpoint.
    #[serde(default = "default_sp1_cluster_endpoint")]
    pub sp1_cluster_endpoint: Url,

    /// Proposer request timeout in seconds.
    #[serde(default = "default_request_timeout")]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub request_timeout: Duration,

    /// Proving timeout in seconds.
    #[serde(default = "default_proving_timeout")]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub proving_timeout: Duration,
}

impl Default for ProposerClientConfig {
    fn default() -> Self {
        Self {
            proposer_endpoint: default_proposer_service_endpoint(),
            sp1_cluster_endpoint: default_sp1_cluster_endpoint(),
            request_timeout: default_request_timeout(),
            proving_timeout: default_proving_timeout(),
        }
    }
}

fn default_proposer_service_endpoint() -> GrpcUri {
    from_env_or_default(
        "PROPOSER_SERVICE_ENDPOINT",
        GrpcUri::from_str(DEFAULT_PROPOSER_SERVICE_ENDPOINT).unwrap(),
    )
}

fn default_sp1_cluster_endpoint() -> Url {
    from_env_or_default(
        "SP1_CLUSTER_ENDPOINT",
        Url::from_str(DEFAULT_SP1_CLUSTER_ENDPOINT).unwrap(),
    )
}

pub fn default_request_timeout() -> Duration {
    Duration::from_secs(600)
}

pub fn default_proving_timeout() -> Duration {
    Duration::from_secs(3600)
}
