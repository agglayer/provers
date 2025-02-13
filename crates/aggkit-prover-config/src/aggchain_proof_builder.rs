use std::str::FromStr;

use jsonrpsee::core::Serialize;
use serde::Deserialize;
use url::Url;

/// The Aggchain proof builder configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofBuilderConfig {
    /// JSON-RPC endpoint of the l1 node.
    #[serde(default = "default_l1_url")]
    pub l1_rpc_endpoint: Url,

    /// Json rpc endpoint of the l2 rollup node.
    #[serde(default = "default_l2_url")]
    pub l2_rpc_endpoint: Url,

    /// Id of the rollup chain
    pub network_id: u32,
}

impl Default for AggchainProofBuilderConfig {
    fn default() -> Self {
        AggchainProofBuilderConfig {
            l1_rpc_endpoint: default_l1_url(),
            l2_rpc_endpoint: default_l2_url(),
            network_id: 0,
        }
    }
}

fn default_l1_url() -> Url {
    Url::from_str("http://anvil-mock-l1-rpc:8545").unwrap()
}

fn default_l2_url() -> Url {
    Url::from_str("http://anvil-mock-l2-rpc:8545").unwrap()
}
