use proposer_client::config::ProposerClientConfig;
use proposer_db_client::ProposerDBConfig;
use prover_alloy::L1RpcEndpoint;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProposerServiceConfig {
    #[serde(default)] // bool::default() is false
    pub mock: bool,

    pub client: ProposerClientConfig,

    /// JSON-RPC endpoint of the l1 node.
    pub l1_rpc_endpoint: L1RpcEndpoint,

    /// Optional database configuration for persisting proof requests.
    #[serde(default)]
    pub database: Option<ProposerDBConfig>,
}
