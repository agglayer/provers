use aggchain_proof_core::Digest;
use alloy::network::Ethereum;
use alloy::primitives::B256;
use alloy::sol;

use crate::Error;

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    GlobalExitRootManagerL2SovereignChain,
    "contracts/global-exit-root-manager-l2-sovereign-chain.json"
);

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    PolygonZkevmBridgeV2,
    "contracts/polygon-zkevm-bridge-v2.json"
);

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    PolygonRollupManager,
    "contracts/polygon-rollup-manager.json"
);

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    AggchainFep,
    "contracts/aggchain-fep.json"
);

pub(crate) type ZkevmBridgeRpcClient<RpcProvider> =
    PolygonZkevmBridgeV2::PolygonZkevmBridgeV2Instance<(), RpcProvider, Ethereum>;

pub(crate) type PolygonRollupManagerRpcClient<RpcProvider> =
    PolygonRollupManager::PolygonRollupManagerInstance<(), RpcProvider, Ethereum>;

pub(crate) type AggchainFepRpcClient<RpcProvider> =
    AggchainFep::AggchainFepInstance<(), RpcProvider, Ethereum>;

#[async_trait::async_trait]
pub trait L2LocalExitRootFetcher {
    async fn get_l2_local_exit_root(&self, block_number: u64) -> Result<Digest, Error>;
}

#[async_trait::async_trait]
pub trait L2OutputAtBlockFetcher {
    fn parse_l2_output_root(json: serde_json::Value) -> Result<L2OutputAtBlock, Error>;
    async fn get_l2_output_at_block(&self, block_number: u64) -> Result<L2OutputAtBlock, Error>;
}

#[async_trait::async_trait]
pub trait L1RollupConfigHashFetcher {
    async fn get_rollup_config_hash(&self) -> Result<Digest, Error>;
}

/// L2 output at block data structure.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct L2OutputAtBlock {
    pub version: B256,
    pub state_root: Digest,
    pub withdrawal_storage_root: Digest,
    pub latest_block_hash: Digest,
    pub output_root: Digest,
}
