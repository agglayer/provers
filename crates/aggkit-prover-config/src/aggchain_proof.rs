use std::fmt::Debug;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::Url;

/// The initial blockchain node backoff in milliseconds
pub const HTTP_RPC_NODE_INITIAL_BACKOFF_MS: u64 = 5000;

/// The blockchain node backoff number of retries
pub const HTTP_RPC_NODE_BACKOFF_MAX_RETRIES: u32 = 64;

/// The Aggchain proof service configuration
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofServiceConfig {
    aggchain_proof_builder: AggchainProofBuilderConfig,
}

/// The Aggchain proof builder configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofBuilderConfig {
    /// Json rpc endpoint of the l1 node.
    pub l1_rpc_endpoint: Url,

    /// Json rpc endpoint of the l2 rollup node.
    pub l2_rpc_endpoint: Url,

    /// Id of the rollup chain
    pub rollup_id: u32,
}

impl Default for AggchainProofBuilderConfig {
    fn default() -> Self {
        AggchainProofBuilderConfig {
            l1_rpc_endpoint: default_l1_url(),
            l2_rpc_endpoint: default_l2_url(),
            rollup_id: 0,
        }
    }
}

fn default_l1_url() -> Url {
    Url::from_str("127.0.0.1::8545").unwrap()
}

fn default_l2_url() -> Url {
    Url::from_str("127.0.0.1::8546").unwrap()
}
