use std::{str::FromStr, time::Duration};

use prover_utils::{from_env_or_default, with};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

/// The default url endpoint for the grpc cluster service
const DEFAULT_SP1_CLUSTER_ENDPOINT: &str = "https://rpc.production.succinct.xyz/";

/// Type of the prover to be used for generation of the pessimistic proof
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ProverType {
    NetworkProver(NetworkProverConfig),
    CpuProver(CpuProverConfig),
    MockProver(MockProverConfig),
    SindriProver(SindriProverConfig),
}

impl Default for ProverType {
    fn default() -> Self {
        ProverType::NetworkProver(NetworkProverConfig::default())
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CpuProverConfig {
    #[serde(default = "default_max_concurrency_limit")]
    pub max_concurrency_limit: usize,

    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub proving_request_timeout: Option<Duration>,

    #[serde(default = "default_local_proving_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub proving_timeout: Duration,
}

impl CpuProverConfig {
    // This constant represents the number of second added to the proving_timeout
    pub const DEFAULT_PROVING_TIMEOUT_PADDING: Duration = Duration::from_secs(1);

    pub fn get_proving_request_timeout(&self) -> Duration {
        self.proving_request_timeout
            .unwrap_or_else(|| self.proving_timeout + Self::DEFAULT_PROVING_TIMEOUT_PADDING)
    }
}

impl Default for CpuProverConfig {
    fn default() -> Self {
        Self {
            max_concurrency_limit: default_max_concurrency_limit(),
            proving_request_timeout: None,
            proving_timeout: default_local_proving_timeout(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct NetworkProverConfig {
    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub proving_request_timeout: Option<Duration>,

    #[serde(default = "default_network_proving_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub proving_timeout: Duration,

    /// The sp1 proving cluster endpoint.
    #[serde(default = "default_sp1_cluster_endpoint")]
    pub sp1_cluster_endpoint: url::Url,
}

impl NetworkProverConfig {
    // This constant represents the number of second added to the proving_timeout
    pub const DEFAULT_PROVING_TIMEOUT_PADDING: Duration = Duration::from_secs(1);

    pub fn get_proving_request_timeout(&self) -> Duration {
        self.proving_request_timeout
            .unwrap_or_else(|| self.proving_timeout + Self::DEFAULT_PROVING_TIMEOUT_PADDING)
    }
}

impl Default for NetworkProverConfig {
    fn default() -> Self {
        Self {
            proving_request_timeout: None,
            proving_timeout: default_network_proving_timeout(),
            sp1_cluster_endpoint: default_sp1_cluster_endpoint(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct MockProverConfig {
    #[serde(default = "default_max_concurrency_limit")]
    pub max_concurrency_limit: usize,

    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub proving_request_timeout: Option<Duration>,

    #[serde(default = "default_local_proving_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub proving_timeout: Duration,
}

impl MockProverConfig {
    // This constant represents the number of second added to the proving_timeout
    pub const DEFAULT_PROVING_TIMEOUT_PADDING: Duration = Duration::from_secs(1);

    pub fn get_proving_request_timeout(&self) -> Duration {
        self.proving_request_timeout
            .unwrap_or_else(|| self.proving_timeout + Self::DEFAULT_PROVING_TIMEOUT_PADDING)
    }
}

impl Default for MockProverConfig {
    fn default() -> Self {
        Self {
            max_concurrency_limit: default_max_concurrency_limit(),
            proving_request_timeout: None,
            proving_timeout: default_local_proving_timeout(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SindriProverConfig {
    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub proving_request_timeout: Option<Duration>,

    #[serde(default = "default_local_proving_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub proving_timeout: Duration,

    #[serde(default = "default_sindri_project_name")]
    pub project_name: String,

    #[serde(default = "default_sindri_project_tag")]
    pub project_tag: String,
}

impl SindriProverConfig {
    // This constant represents the number of second added to the proving_timeout
    pub const DEFAULT_PROVING_TIMEOUT_PADDING: Duration = Duration::from_secs(1);

    pub fn get_proving_request_timeout(&self) -> Duration {
        self.proving_request_timeout
            .unwrap_or_else(|| self.proving_timeout + Self::DEFAULT_PROVING_TIMEOUT_PADDING)
    }
}

impl Default for SindriProverConfig {
    fn default() -> Self {
        Self {
            proving_request_timeout: None,
            proving_timeout: default_network_proving_timeout(),
            project_name: default_sindri_project_name(),
            project_tag: default_sindri_project_tag(),
        }
    }
}

pub const fn default_max_concurrency_limit() -> usize {
    100
}

const fn default_local_proving_timeout() -> Duration {
    Duration::from_secs(60 * 5)
}

const fn default_network_proving_timeout() -> Duration {
    Duration::from_secs(60 * 5)
}

fn default_sp1_cluster_endpoint() -> Url {
    from_env_or_default(
        "SP1_CLUSTER_ENDPOINT",
        Url::from_str(DEFAULT_SP1_CLUSTER_ENDPOINT).unwrap(),
    )
}

fn default_sindri_project_name() -> String {
    "pessimistic-proof".to_string()
}

fn default_sindri_project_tag() -> String {
    "latest".to_string()
}
