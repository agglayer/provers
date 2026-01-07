use serde::{Deserialize, Serialize};
use url::Url;

const DEFAULT_DATABASE_URL: &str = "postgresql://localhost:5432/proposer";
const DEFAULT_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_MIN_CONNECTIONS: u32 = 2;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerDBConfig {
    #[serde(default = "default_database_url")]
    pub database_url: Url,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Polling interval in milliseconds for checking proof completion status
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,

    /// Maximum number of polling retries before timing out
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

impl Default for ProposerDBConfig {
    fn default() -> Self {
        Self {
            database_url: default_database_url(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            poll_interval_ms: default_poll_interval_ms(),
            max_retries: default_max_retries(),
        }
    }
}

fn default_database_url() -> Url {
    std::env::var("PROPOSER_DATABASE_URL")
        .ok()
        .and_then(|s| Url::parse(&s).ok())
        .unwrap_or_else(|| Url::parse(DEFAULT_DATABASE_URL).expect("Invalid default database URL"))
}

const fn default_max_connections() -> u32 {
    DEFAULT_MAX_CONNECTIONS
}

const fn default_min_connections() -> u32 {
    DEFAULT_MIN_CONNECTIONS
}

const fn default_poll_interval_ms() -> u64 {
    5000 // 5 seconds
}

const fn default_max_retries() -> u32 {
    720 // 720 * 5s = 1 hour
}
