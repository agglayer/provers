use alloy::primitives::{address, Address};
use prover_alloy::L1RpcEndpoint;
use prover_utils::from_env_or_default;
use serde::{Deserialize, Serialize};
use url::Url;

/// Address of the `GlobalExitRootManagerL2SovereignChain.sol` contract
/// on the L2 chain is always fixed.
const GLOBAL_EXIT_ROOT_MANAGER_L2_SOVEREIGN_CHAIN_ADDRESS: Address =
    address!("0xa40D5f56745a118D0906a34E69aeC8C0Db1cB8fA");

/// Address of the `PolygonRollupManager.sol` contract
/// on the L1 chain.
const POLYGON_ROLLUP_MANAGER: Address = address!("0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e");

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofContractsConfig {
    /// JSON-RPC endpoint of the l1 node.
    #[serde(default)]
    pub l1_rpc_endpoint: L1RpcEndpoint,

    /// JSON-RPC endpoint of the l2 execution node.
    #[serde(default = "prover_alloy::default_l2_execution_layer_url")]
    pub l2_execution_layer_rpc_endpoint: Url,

    /// JSON-RPC endpoint of the l2 rollup node.
    #[serde(default = "prover_alloy::default_l2_consensus_layer_url")]
    pub l2_consensus_layer_rpc_endpoint: Url,

    /// Address of the L1 PolygonRollupManager.sol contract
    #[serde(default = "default_polygon_rollup_manager")]
    pub polygon_rollup_manager: Address,
    /// Address of the L2 GlobalExitRootManagerL2SovereignChain.sol contract
    #[serde(default = "default_global_exit_root_manager_v2_sovereign_chain")]
    pub global_exit_root_manager_v2_sovereign_chain: Address,
}

impl Default for AggchainProofContractsConfig {
    fn default() -> Self {
        Self {
            l1_rpc_endpoint: L1RpcEndpoint::default(),
            l2_execution_layer_rpc_endpoint: prover_alloy::default_l2_execution_layer_url(),
            l2_consensus_layer_rpc_endpoint: prover_alloy::default_l2_consensus_layer_url(),
            polygon_rollup_manager: default_polygon_rollup_manager(),
            global_exit_root_manager_v2_sovereign_chain:
                default_global_exit_root_manager_v2_sovereign_chain(),
        }
    }
}

pub(crate) fn default_output_at_block_endpoint() -> String {
    from_env_or_default(
        "L2_OUTPUT_AT_BLOCK_ENDPOINT",
        "optimism_outputAtBlock".to_string(),
    )
}

fn default_polygon_rollup_manager() -> Address {
    POLYGON_ROLLUP_MANAGER
}

fn default_global_exit_root_manager_v2_sovereign_chain() -> Address {
    GLOBAL_EXIT_ROOT_MANAGER_L2_SOVEREIGN_CHAIN_ADDRESS
}
