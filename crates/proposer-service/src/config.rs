use alloy_primitives::{hex, B256};
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

    /// Hash of the aggregated execution proof verification key.
    ///
    /// This is the proof that verifies the proposer provided aggregated span proof (aggregated full
    /// execution proof for the block span).
    #[serde(default = "default_agg_span_proof_vkey_hash")]
    pub agg_span_proof_vkey_hash: B256,
}

impl Default for ProposerServiceConfig {
    fn default() -> Self {
        Self {
            client: ProposerClientConfig::default(),
            l1_rpc_endpoint: prover_alloy::default_l1_node_url(),
            agg_span_proof_vkey_hash: default_agg_span_proof_vkey_hash(),
        }
    }
}

fn default_agg_span_proof_vkey_hash() -> B256 {
    hex!("00441b614a713401ad1090cd7e59813e07352cb247172934f5a05dedb9e671bf").into()
}
