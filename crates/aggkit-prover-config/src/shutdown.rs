use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ShutdownConfig {
    #[serde(default = "default_runtime_shutdown_timeout")]
    #[serde(with = "prover_utils::with::HumanDuration")]
    pub runtime_timeout: Duration,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            runtime_timeout: default_runtime_shutdown_timeout(),
        }
    }
}

const fn default_runtime_shutdown_timeout() -> Duration {
    Duration::from_secs(30)
}
