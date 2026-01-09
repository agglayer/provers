use proposer_client::config::ProposerClientConfig;
use proposer_db_client::ProposerDBConfig;
use prover_alloy::L1RpcEndpoint;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerServiceConfig {
    #[serde(default)] // bool::default() is false
    pub mock: bool,

    pub client: ProposerClientConfig,

    /// JSON-RPC endpoint of the l1 node.
    #[serde(default)]
    pub l1_rpc_endpoint: L1RpcEndpoint,

    /// JSON-RPC endpoint of the L2 consensus layer (rollup node).
    #[serde(default = "prover_alloy::default_l2_consensus_layer_url")]
    pub l2_consensus_layer_rpc_endpoint: Url,

    /// Optional database configuration for persisting proof requests.
    #[serde(default)]
    pub database: Option<ProposerDBConfig>,
}
