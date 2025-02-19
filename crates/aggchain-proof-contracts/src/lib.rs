mod error;

use std::sync::Arc;

use alloy::network::Ethereum;
use alloy::primitives::Address;
use alloy::primitives::{address, B256};
use alloy::sol;
use prover_alloy::{AlloyFillProvider, AlloyProvider};
use url::Url;

pub use crate::error::Error;

/// Address of the `GlobalExitRootManagerL2SovereignChain.sol` contract on the
/// L2 chain is always fixed.
const GLOBAL_EXIT_ROOT_MANAGER_L2_SOVEREIGN_CHAIN_ADDRESS: Address =
    address!("0xa40D5f56745a118D0906a34E69aeC8C0Db1cB8fA");

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

type GlobalExitRootManager =
    GlobalExitRootManagerL2SovereignChain::GlobalExitRootManagerL2SovereignChainInstance<
        (),
        AlloyFillProvider,
        Ethereum,
    >;
type ZkevmBridge =
    PolygonZkevmBridgeV2::PolygonZkevmBridgeV2Instance<(), AlloyFillProvider, Ethereum>;

/// Wrapper around alloy `Provider` client.
/// Performs ETH node response data processing where needed but
/// allows direct use of the provider if necessary.
#[derive(Clone)]
pub struct AggchainProofContractsClient {
    /// Mainnet node rpc client.
    _l1_client: Arc<AlloyProvider>,

    /// L2 node rpc client.
    _l2_client: Arc<AlloyProvider>,

    /// Global exit root manager for smart sovereign chain
    /// contract on the l2 network.
    _global_exit_root_manager_l2: GlobalExitRootManager,

    /// Polygon zkevm bridge contract on the l2 network.
    _polygon_zkevm_bridge_v2: ZkevmBridge,
}

impl AggchainProofContractsClient {
    pub fn new(l1_rpc_endpoint: &Url, l2_rpc_endpoint: &Url) -> Result<Self, crate::Error> {
        let l1_client = Arc::new(
            AlloyProvider::new(
                l1_rpc_endpoint,
                prover_alloy::DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                prover_alloy::DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
            )
            .map_err(Error::ProviderInitializationError)?,
        );

        let l2_client = Arc::new(
            AlloyProvider::new(
                l2_rpc_endpoint,
                prover_alloy::DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                prover_alloy::DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
            )
            .map_err(Error::ProviderInitializationError)?,
        );

        // Create client for global exit root manager smart contract
        let global_exit_root_manager_l2 = GlobalExitRootManagerL2SovereignChain::new(
            GLOBAL_EXIT_ROOT_MANAGER_L2_SOVEREIGN_CHAIN_ADDRESS,
            l2_client.provider().clone(),
        );

        // Retrieve PolygonZkEVMBridgeV2 contract address from the global exit root
        // manager contract
        let polygon_zkevm_bridge_address = {
            let global_exit_root_manager_l2 = global_exit_root_manager_l2.clone();
            std::thread::spawn(move || {
                futures::executor::block_on(async move {
                    // We need to retrieve PolygonZkEVMBridgeV2 contract address from the
                    // global exit root manager contract
                    global_exit_root_manager_l2.bridgeAddress().call().await
                })
            })
        }
        .join()
        .expect("Couldn't join on the associated thread")
        .map_err(Error::BridgeAddressError)?;

        // Create client for Polygon zkevm bridge v2 smart contract
        let polygon_zkevm_bridge_v2 = PolygonZkevmBridgeV2::new(
            polygon_zkevm_bridge_address._0,
            l2_client.provider().clone(),
        );

        Ok(Self {
            _l1_client: l1_client,
            _l2_client: l2_client,
            _global_exit_root_manager_l2: global_exit_root_manager_l2,
            _polygon_zkevm_bridge_v2: polygon_zkevm_bridge_v2,
        })
    }
    
}


