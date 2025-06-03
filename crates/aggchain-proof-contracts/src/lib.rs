pub mod config;
pub mod contracts;
mod error;

#[cfg(test)]
mod tests;

use std::{str::FromStr, sync::Arc};

use aggchain_proof_core::bridge::{
    static_call::{HashChainType, StaticCallStage},
    BridgeL2SovereignChain,
};
use agglayer_interop::types::Digest;
use alloy::{
    eips::BlockNumberOrTag,
    primitives::{Address, B256},
};
use contracts::{
    GetTrustedSequencerAddress, GlobalExitRootManagerL2SovereignChainRpcClient,
    L2EvmStateSketchFetcher,
};
use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use prover_alloy::{build_alloy_fill_provider, AlloyFillProvider};
use sp1_cc_client_executor::io::EvmSketchInput;
use sp1_cc_host_executor::EvmSketch;
use tracing::info;
use url::Url;

pub use crate::error::Error;
use crate::{
    config::AggchainProofContractsConfig,
    contracts::{
        AggchainFep, AggchainFepRpcClient, GlobalExitRootManagerL2SovereignChain,
        L1RollupConfigHashFetcher, L2LocalExitRootFetcher, L2OutputAtBlock, L2OutputAtBlockFetcher,
        PolygonRollupManagerRpcClient, PolygonZkevmBridgeV2, ZkevmBridgeRpcClient,
    },
};

/// `AggchainContractsClient` is a trait for interacting with the smart
/// contracts relevant for the aggchain prover.
pub trait AggchainContractsClient:
    L2LocalExitRootFetcher
    + L2OutputAtBlockFetcher
    + L1RollupConfigHashFetcher
    + L2EvmStateSketchFetcher
{
}

/// `AggchainProofContractsRpcClient` is a client for interacting with the
/// smart contracts relevant for the aggchain prover.
#[derive(Clone)]
pub struct AggchainContractsRpcClient<RpcProvider> {
    /// Url for the evm state sketch builder.
    l2_root_provider_endpoint: Url,

    /// L2 rpc consensus layer client (rollup node).
    l2_cl_client: Arc<HttpClient>,

    /// Polygon zkevm bridge contract on the l2 network.
    polygon_zkevm_bridge_v2: ZkevmBridgeRpcClient<RpcProvider>,

    /// GER contract on the l2 network.
    global_exit_root_manager_l2: GlobalExitRootManagerL2SovereignChainRpcClient<RpcProvider>,

    /// Aggchain FEP contract on the l1 network.
    aggchain_fep: AggchainFepRpcClient<RpcProvider>,

    /// Trusted sequencer address.
    trusted_sequencer_addr: agglayer_primitives::Address,
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

        Ok((response.0).into())
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
            .rollupConfigHash()
            .call()
            .await
            .map_err(Error::RollupConfigHashError)?;

        Ok((response.0).into())
    }
}

#[async_trait::async_trait]
impl<RpcProvider> GetTrustedSequencerAddress for AggchainContractsRpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Send + Sync,
{
    async fn get_trusted_sequencer_address(&self) -> Result<Address, Error> {
        Ok(self.trusted_sequencer_addr)
    }
}

#[async_trait::async_trait]
impl<RpcProvider> L2EvmStateSketchFetcher for AggchainContractsRpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Send + Sync,
{
    async fn get_prev_l2_block_sketch(
        &self,
        prev_l2_block: BlockNumberOrTag,
    ) -> Result<EvmSketchInput, Error> {
        let l2_root_provider_endpoint = self.l2_root_provider_endpoint.clone();
        let ger_address = *self.global_exit_root_manager_l2.address();
        let bridge_address = *self.polygon_zkevm_bridge_v2.address();

        // Use spawn_local to handle the non-Sync EvmSketch
        let prev_l2_block_sketch = tokio::task::spawn_local(async move {
            let sketch = EvmSketch::builder()
                .at_block(prev_l2_block)
                .el_rpc_url(l2_root_provider_endpoint)
                .build()
                .await
                .map_err(Error::HostExecutorPreBlockInitialization)?;

            // Execute all static calls sequentially
            let _result1 = sketch
                .call(
                    ger_address,
                    Address::default(),
                    GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::PrevHashChain(HashChainType::InsertedGER),
                })?;

            let _result2 = sketch
                .call(
                    ger_address,
                    Address::default(),
                    GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::PrevHashChain(HashChainType::RemovedGER),
                })?;

            let _result3 = sketch
                .call(
                    bridge_address,
                    Address::default(),
                    BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::PrevHashChain(HashChainType::ClaimedGlobalIndex),
                })?;

            let _result4 = sketch
                .call(
                    bridge_address,
                    Address::default(),
                    BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::PrevHashChain(HashChainType::UnsetGlobalIndex),
                })?;

            // Finalize to retrieve the EVMStateSketch
            let prev_l2_block_sketch = sketch
                .finalize()
                .await
                .map_err(Error::InvalidPreBlockSketchFinalization)?;

            Ok::<_, Error>(prev_l2_block_sketch)
        })
        .await
        .map_err(|e| {
            // Join errors are very rare and indicate a critical system issue
            // For now, we'll just panic since this is an unexpected error case
            panic!("Task join error in get_prev_l2_block_sketch: {e}");
        })?;

        prev_l2_block_sketch
    }

    async fn get_new_l2_block_sketch(
        &self,
        new_l2_block: BlockNumberOrTag,
    ) -> Result<EvmSketchInput, Error> {
        let l2_root_provider_endpoint = self.l2_root_provider_endpoint.clone();
        let ger_address = *self.global_exit_root_manager_l2.address();
        let bridge_address = *self.polygon_zkevm_bridge_v2.address();

        // Use spawn_blocking to handle the non-Sync EvmSketch
        let new_l2_block_sketch = tokio::task::spawn_local(async move {
            let sketch = EvmSketch::builder()
                .at_block(new_l2_block)
                .el_rpc_url(l2_root_provider_endpoint)
                .build()
                .await
                .map_err(Error::HostExecutorNewBlockInitialization)?;

            // Execute static call on the bridge address
            let _result1 = sketch
                .call(
                    ger_address,
                    Address::default(),
                    GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::BridgeAddress,
                })?;

            // Execute static call on the new LER
            let _result2 = sketch
                .call(
                    bridge_address,
                    Address::default(),
                    BridgeL2SovereignChain::getRootCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::NewHashChain(HashChainType::InsertedGER),
                })?;

            // Execute all remaining static calls
            let _result3 = sketch
                .call(
                    ger_address,
                    Address::default(),
                    GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::NewHashChain(HashChainType::InsertedGER),
                })?;

            let _result4 = sketch
                .call(
                    ger_address,
                    Address::default(),
                    GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::NewHashChain(HashChainType::RemovedGER),
                })?;

            let _result5 = sketch
                .call(
                    bridge_address,
                    Address::default(),
                    BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::NewHashChain(HashChainType::ClaimedGlobalIndex),
                })?;

            let _result6 = sketch
                .call(
                    bridge_address,
                    Address::default(),
                    BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
                )
                .await
                .map_err(|source| Error::InvalidHostStaticCall {
                    source,
                    stage: StaticCallStage::NewHashChain(HashChainType::UnsetGlobalIndex),
                })?;

            // Finalize to retrieve the EVMStateSketch
            let new_l2_block_sketch = sketch
                .finalize()
                .await
                .map_err(Error::InvalidNewBlockSketchFinalization)?;

            Ok::<_, Error>(new_l2_block_sketch)
        })
        .await
        .map_err(|e| {
            // Join errors are very rare and indicate a critical system issue
            // For now, we'll just panic since this is an unexpected error case
            panic!("Task join error in get_new_l2_block_sketch: {e}");
        })?;

        new_l2_block_sketch
    }
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
            &config.l1_rpc_endpoint.url,
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
            PolygonZkevmBridgeV2::new(polygon_zkevm_bridge_address, l2_el_client.clone());

        // Create client for Polygon rollup manager contract.
        let polygon_rollup_manager =
            PolygonRollupManagerRpcClient::new(config.polygon_rollup_manager, l1_client.clone());

        // Retrieve AggchainFep address from the Polygon rollup manager contract.
        let aggchain_fep_address = polygon_rollup_manager
            .rollupIDToRollupData(network_id)
            .call()
            .await
            .map_err(Error::AggchainFepAddressError)?
            .rollupContract;

        // Create client for AggchainFep smart contract.
        let aggchain_fep = AggchainFep::new(aggchain_fep_address, l1_client.clone());

        let trusted_sequencer_addr = aggchain_fep
            .trustedSequencer()
            .call()
            .await
            .map_err(Error::UnableToRetrieveTrustedSequencerAddress)?;

        info!(global_exit_root_manager_l2=%config.global_exit_root_manager_v2_sovereign_chain,
            polygon_zkevm_bridge_v2=%polygon_zkevm_bridge_v2.address(),
            polygon_rollup_manager=%config.polygon_rollup_manager,
            aggchain_fep=%aggchain_fep.address(),
            "Aggchain proof contracts client created successfully");

        Ok(Self {
            l2_cl_client,
            polygon_zkevm_bridge_v2,
            aggchain_fep,
            l2_root_provider_endpoint: config.l2_execution_layer_rpc_endpoint.clone(),
            global_exit_root_manager_l2,
            trusted_sequencer_addr,
        })
    }
}
