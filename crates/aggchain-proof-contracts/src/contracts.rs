use agglayer_interop::types::Digest;
use alloy::network::Ethereum;
use alloy::sol;

use crate::Error;

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    GlobalExitRootManagerL2SovereignChain,
    "contracts/GlobalExitRootManagerL2SovereignChain.json"
);

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    PolygonZkevmBridgeV2,
    "contracts/PolygonZkEVMBridgeV2.json"
);

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    PolygonRollupManager,
    "contracts/PolygonRollupManager.json"
);

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    AggchainFep,
    "contracts/AggchainFEP.json"
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
    async fn get_l2_output_at_block(&self, block_number: u64) -> Result<L2OutputAtBlock, Error>;
}

#[async_trait::async_trait]
pub trait L1RollupConfigHashFetcher {
    async fn get_rollup_config_hash(&self) -> Result<Digest, Error>;
}

/// L2 output at block data structure.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct L2OutputAtBlock {
    pub version: Digest,
    pub state_root: Digest,
    pub withdrawal_storage_root: Digest,
    pub latest_block_hash: Digest,
    pub output_root: Digest,
}
