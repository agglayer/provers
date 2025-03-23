pub mod config;
pub mod contracts;
mod error;

#[cfg(test)]
mod tests;

use std::str::FromStr;
use std::sync::Arc;

use aggchain_proof_core::bridge::static_call::{HashChainType, StaticCallStage};
use aggchain_proof_core::bridge::BridgeL2SovereignChain;
use agglayer_interop::types::Digest;
use alloy::eips::BlockNumberOrTag;
use alloy::network::AnyNetwork;
use alloy::primitives::{Address, B256};
use alloy::providers::{Provider, RootProvider};
use alloy::sol_types::SolCall;
use contracts::{GlobalExitRootManagerL2SovereignChainRpcClient, L2EVMStateSketchesFetched};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;
use prover_alloy::{build_alloy_fill_provider, AlloyFillProvider};
use sp1_cc_client_executor::io::EVMStateSketch;
use sp1_cc_client_executor::ContractInput;
use sp1_cc_host_executor::HostExecutor;
use tracing::info;

use crate::config::AggchainProofContractsConfig;
use crate::contracts::{
    AggchainFep, AggchainFepRpcClient, GlobalExitRootManagerL2SovereignChain,
    L1RollupConfigHashFetcher, L2LocalExitRootFetcher, L2OutputAtBlock, L2OutputAtBlockFetcher,
    PolygonRollupManagerRpcClient, PolygonZkevmBridgeV2, ZkevmBridgeRpcClient,
};
pub use crate::error::Error;

/// `AggchainContractsClient` is a trait for interacting with the smart
/// contracts relevant for the aggchain prover.
pub trait AggchainContractsClient:
    L2LocalExitRootFetcher
    + L2OutputAtBlockFetcher
    + L1RollupConfigHashFetcher
    + L2EVMStateSketchesFetched
{
}

/// `AggchainProofContractsRpcClient` is a client for interacting with the
/// smart contracts relevant for the aggchain prover.
#[derive(Clone)]
pub struct AggchainContractsRpcClient<RpcProvider> {
    /// Mainnet node rpc client.
    _l1_client: Arc<RpcProvider>,

    /// L2 rpc execution layer client.
    _l2_el_client: Arc<RpcProvider>,

    /// L2 rpc execution layer client.
    l2_root_provider: RootProvider<AnyNetwork>,

    /// L2 rpc consensus layer client (rollup node).
    l2_cl_client: Arc<HttpClient>,

    /// Polygon zkevm bridge contract on the l2 network.
    polygon_zkevm_bridge_v2: ZkevmBridgeRpcClient<RpcProvider>,

    /// GER contract on the l2 network.
    global_exit_root_manager_l2: GlobalExitRootManagerL2SovereignChainRpcClient<RpcProvider>,

    /// Aggchain FEP contract on the l1 network.
    aggchain_fep: AggchainFepRpcClient<RpcProvider>,
}

impl<T: alloy::providers::Provider> AggchainContractsClient for AggchainContractsRpcClient<T> {}

#[async_trait::async_trait]
impl<RpcProvider> L2LocalExitRootFetcher for AggchainContractsRpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Send + Sync,
{
    async fn get_l2_local_exit_root(&self, block_number: u64) -> Result<Digest, Error> {
        let response = self
            .polygon_zkevm_bridge_v2
            .getRoot()
            .call()
            .block(block_number.into())
            .await
            .map_err(Error::LocalExitRootError)?;

        Ok((*response._0).into())
    }
}

#[async_trait::async_trait]
impl<RpcProvider> L2OutputAtBlockFetcher for AggchainContractsRpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Send + Sync,
{
    async fn get_l2_output_at_block(&self, block_number: u64) -> Result<L2OutputAtBlock, Error> {
        let params = rpc_params![format!("0x{block_number:x}")];
        let json: serde_json::Value = self
            .l2_cl_client
            .request(&crate::config::default_output_at_block_endpoint(), params)
            .await
            .map_err(Error::L2OutputAtBlockRetrievalError)?;

        Self::parse_l2_output_root(json)
    }
}

#[async_trait::async_trait]
impl<RpcProvider> L1RollupConfigHashFetcher for AggchainContractsRpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Send + Sync,
{
    async fn get_rollup_config_hash(&self) -> Result<Digest, Error> {
        let response = self
            .aggchain_fep
            .chainConfigHash()
            .call()
            .await
            .map_err(Error::RollupConfigHashError)?;

        Ok((*response._0).into())
    }
}

#[async_trait::async_trait]
impl<RpcProvider> L2EVMStateSketchesFetched for AggchainContractsRpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Send + Sync,
{
    async fn get_prev_l2_block_sketch(
        &self,
        prev_l2_block: BlockNumberOrTag,
    ) -> Result<EVMStateSketch, Error> {
        let mut executor: HostExecutor<RootProvider<AnyNetwork>> =
            HostExecutor::new(self.l2_root_provider.clone(), prev_l2_block)
                .await
                .map_err(Error::HostExecutorPreBlockInitialization)?;

        let ger_address = *self.global_exit_root_manager_l2.address();
        let bridge_address = *self.polygon_zkevm_bridge_v2.address();

        // Static calls on the hash chains
        {
            _ = host_execute(
                ger_address,
                &mut executor,
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
                StaticCallStage::PrevHashChain(HashChainType::InsertedGER),
            )
            .await?;

            _ = host_execute(
                ger_address,
                &mut executor,
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
                StaticCallStage::PrevHashChain(HashChainType::RemovedGER),
            )
            .await?;

            _ = host_execute(
                bridge_address,
                &mut executor,
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
                StaticCallStage::PrevHashChain(HashChainType::ClaimedGlobalIndex),
            )
            .await?;

            _ = host_execute(
                bridge_address,
                &mut executor,
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
                StaticCallStage::PrevHashChain(HashChainType::UnsetGlobalIndex),
            )
            .await?;
        }

        // Finalize to retrieve the EVMStateSketch
        let prev_l2_block_sketch = executor
            .finalize()
            .await
            .map_err(Error::InvalidPreBlockSketchFinalization)?;

        Ok(prev_l2_block_sketch)
    }

    async fn get_new_l2_block_sketch(
        &self,
        new_l2_block: BlockNumberOrTag,
    ) -> Result<EVMStateSketch, Error> {
        let mut executor: HostExecutor<RootProvider<AnyNetwork>> =
            HostExecutor::new(self.l2_root_provider.clone(), new_l2_block)
                .await
                .map_err(Error::HostExecutorNewBlockInitialization)?;

        let ger_address = *self.global_exit_root_manager_l2.address();
        let bridge_address = *self.polygon_zkevm_bridge_v2.address();

        // Static call on the bridge address
        _ = host_execute(
            ger_address,
            &mut executor,
            GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
            StaticCallStage::BridgeAddress,
        )
        .await?;

        // Static call on the new LER
        _ = host_execute(
            bridge_address,
            &mut executor,
            BridgeL2SovereignChain::getRootCall {},
            StaticCallStage::NewHashChain(HashChainType::InsertedGER),
        )
        .await?;

        // Static calls on the hash chains
        {
            _ = host_execute(
                ger_address,
                &mut executor,
                GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
                StaticCallStage::NewHashChain(HashChainType::InsertedGER),
            )
            .await?;

            _ = host_execute(
                ger_address,
                &mut executor,
                GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
                StaticCallStage::NewHashChain(HashChainType::RemovedGER),
            )
            .await?;

            _ = host_execute(
                bridge_address,
                &mut executor,
                BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
                StaticCallStage::NewHashChain(HashChainType::ClaimedGlobalIndex),
            )
            .await?;

            _ = host_execute(
                bridge_address,
                &mut executor,
                BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
                StaticCallStage::NewHashChain(HashChainType::UnsetGlobalIndex),
            )
            .await?;
        }

        // Finalize to retrieve the EVMStateSketch
        let new_l2_block_sketch = executor
            .finalize()
            .await
            .map_err(Error::InvalidNewBlockSketchFinalization)?;

        Ok(new_l2_block_sketch)
    }
}

async fn host_execute<C: SolCall, P: Provider<AnyNetwork> + Clone>(
    contract_address: Address,
    host_executor: &mut HostExecutor<P>,
    calldata: C,
    stage: StaticCallStage,
) -> Result<(), Error> {
    let caller_address = Address::default(); // irrelevant caller address

    let _ = host_executor
        .execute(ContractInput::new_call(
            contract_address,
            caller_address,
            calldata,
        ))
        .await
        .map_err(|source| Error::InvalidHostStaticCall { source, stage })?;

    Ok(())
}

impl<RpcProvider> AggchainContractsRpcClient<RpcProvider> {
    fn parse_l2_output_root(json: serde_json::Value) -> Result<L2OutputAtBlock, Error> {
        fn parse_hash(json: &serde_json::Value, field: &str) -> Result<Digest, Error> {
            let value_str = json
                .get(field)
                .ok_or(Error::L2OutputAtBlockValueMissing(field.to_string()))?
                .as_str()
                .ok_or(Error::L2OutputAtBlockValueMissing(field.to_string()))?;

            B256::from_str(value_str)
                .map(|bytes| bytes.0.into())
                .map_err(|e| Error::L2OutputAtBlockInvalidValue(field.to_string(), e))
        }

        let block_ref = json
            .get("blockRef")
            .ok_or(Error::L2OutputAtBlockValueMissing("blockRef".to_string()))?;

        Ok(L2OutputAtBlock {
            version: parse_hash(&json, "version")?,
            state_root: parse_hash(&json, "stateRoot")?,
            withdrawal_storage_root: parse_hash(&json, "withdrawalStorageRoot")?,
            latest_block_hash: parse_hash(block_ref, "hash")?,
            output_root: parse_hash(&json, "outputRoot")?,
        })
    }
}

impl AggchainContractsRpcClient<AlloyFillProvider> {
    pub async fn new(
        network_id: u32,
        config: &AggchainProofContractsConfig,
    ) -> Result<Self, crate::Error> {
        let l1_client = build_alloy_fill_provider(
            &config.l1_rpc_endpoint,
            prover_alloy::DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
            prover_alloy::DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
        )
        .map_err(Error::ProviderInitializationError)?;

        let l2_el_client = build_alloy_fill_provider(
            &config.l2_execution_layer_rpc_endpoint,
            prover_alloy::DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
            prover_alloy::DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
        )
        .map_err(Error::ProviderInitializationError)?;

        let l2_cl_client = Arc::new(
            HttpClient::builder()
                .build(&config.l2_consensus_layer_rpc_endpoint)
                .map_err(Error::RollupNodeInitError)?,
        );

        // Create client for global exit root manager smart contract.
        let global_exit_root_manager_l2 = GlobalExitRootManagerL2SovereignChain::new(
            config.global_exit_root_manager_v2_sovereign_chain,
            l2_el_client.clone(),
        );

        // Retrieve PolygonZkEVMBridgeV2 contract address from the global exit root
        // manager contract.
        let polygon_zkevm_bridge_address = global_exit_root_manager_l2
            .bridgeAddress()
            .call()
            .await
            .map_err(Error::BridgeAddressError)?;

        // Create client for Polygon zkevm bridge v2 smart contract.
        let polygon_zkevm_bridge_v2 =
            PolygonZkevmBridgeV2::new(polygon_zkevm_bridge_address._0, l2_el_client.clone());

        // Create client for Polygon rollup manager contract.
        let polygon_rollup_manager =
            PolygonRollupManagerRpcClient::new(config.polygon_rollup_manager, l1_client.clone());

        // Retrieve AggchainFep address from the Polygon rollup manager contract.
        let aggchain_fep_address = polygon_rollup_manager
            .rollupIDToRollupData(network_id)
            .call()
            .await
            .map_err(Error::AggchainFepAddressError)?
            .rollupData
            .rollupContract;

        // Create client for AggchainFep smart contract.
        let aggchain_fep = AggchainFep::new(aggchain_fep_address, l1_client.clone());

        let l2_root_provider =
            RootProvider::<AnyNetwork>::new_http(config.l2_execution_layer_rpc_endpoint.clone());

        info!(global_exit_root_manager_l2=%config.global_exit_root_manager_v2_sovereign_chain,
            polygon_zkevm_bridge_v2=%polygon_zkevm_bridge_v2.address(),
            polygon_rollup_manager=%config.polygon_rollup_manager,
            aggchain_fep=%aggchain_fep.address(),
            "Aggchain proof contracts client created successfully");

        Ok(Self {
            _l1_client: Arc::new(l1_client),
            _l2_el_client: Arc::new(l2_el_client),
            l2_cl_client,
            polygon_zkevm_bridge_v2,
            aggchain_fep,
            l2_root_provider,
            global_exit_root_manager_l2,
        })
    }
}
