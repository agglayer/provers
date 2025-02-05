use std::time::Duration;

use prover_utils::with;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Type of the prover to be used for generation of the pessimistic proof
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ProverType {
    NetworkProver(NetworkProverConfig),
    CpuProver(CpuProverConfig),
    GpuProver(GpuProverConfig),
    MockProver(MockProverConfig),
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
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GpuProverConfig {
    #[serde(default = "default_max_concurrency_limit")]
    pub max_concurrency_limit: usize,

    #[serde_as(as = "Option<crate::with::HumanDuration>")]
    pub proving_request_timeout: Option<Duration>,

    #[serde(default = "default_local_proving_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub proving_timeout: Duration,
}

impl GpuProverConfig {
    // This constant represents the number of second added to the proving_timeout
    pub const DEFAULT_PROVING_TIMEOUT_PADDING: Duration = Duration::from_secs(1);

    pub fn get_proving_request_timeout(&self) -> Duration {
        self.proving_request_timeout
            .unwrap_or_else(|| self.proving_timeout + Self::DEFAULT_PROVING_TIMEOUT_PADDING)
    }
}

impl Default for GpuProverConfig {
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

pub const fn default_max_concurrency_limit() -> usize {
    100
}

const fn default_local_proving_timeout() -> Duration {
    Duration::from_secs(60 * 5)
}

const fn default_network_proving_timeout() -> Duration {
    Duration::from_secs(60 * 5)
}
