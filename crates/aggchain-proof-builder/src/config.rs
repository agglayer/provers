use std::time::Duration;

use aggchain_proof_contracts::config::AggchainProofContractsConfig;
use prover_config::ProverType;
use serde::{Deserialize, Serialize};
use url::Url;

/// The Aggchain proof builder configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofBuilderConfig {
    /// JSON-RPC endpoint of the l1 node.
    #[serde(default = "prover_alloy::default_l1_node_url")]
    pub l1_rpc_endpoint: Url,

    /// JSON-RPC endpoint of the l2 execution node.
    #[serde(default = "prover_alloy::default_l2_execution_layer_url")]
    pub l2_el_rpc_endpoint: Url,

    /// JSON-RPC endpoint of the l2 rollup node.
    #[serde(default = "prover_alloy::default_l2_consensus_layer_url")]
    pub l2_cl_rpc_endpoint: Url,

    /// ID of the network for which the proof is generated (rollup id).
    pub network_id: u32,

    /// Aggchain prover configuration
    pub primary_prover: ProverType,

    /// Fallback prover configuration
    pub fallback_prover: Option<ProverType>,

    /// Aggchain proof generation timeout in seconds.
    #[serde(default = "default_aggchain_prover_timeout")]
    pub proving_timeout: Duration,

    /// Contract configuration
    #[serde(default)]
    pub contracts: AggchainProofContractsConfig,
}

impl Default for AggchainProofBuilderConfig {
    fn default() -> Self {
        AggchainProofBuilderConfig {
            l1_rpc_endpoint: prover_alloy::default_l1_node_url(),
            l2_el_rpc_endpoint: prover_alloy::default_l2_execution_layer_url(),
            l2_cl_rpc_endpoint: prover_alloy::default_l2_consensus_layer_url(),
            network_id: 0,
            proving_timeout: default_aggchain_prover_timeout(),
            primary_prover: ProverType::NetworkProver(prover_config::NetworkProverConfig::default()),
            fallback_prover: None,
            contracts: AggchainProofContractsConfig::default(),
        }
    }
}

fn default_aggchain_prover_timeout() -> Duration {
    Duration::from_secs(3600)
}
