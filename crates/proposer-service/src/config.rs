use proposer_client::config::ProposerClientConfig;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerServiceConfig {
    pub client: ProposerClientConfig,

    /// JSON-RPC endpoint of the l1 node.
    #[serde(default = "prover_alloy::default_l1_node_url")]
    pub l1_rpc_endpoint: Url,
}

impl Default for ProposerServiceConfig {
    fn default() -> Self {
        Self {
            client: ProposerClientConfig::default(),
            l1_rpc_endpoint: prover_alloy::default_l1_node_url(),
        }
    }
}
