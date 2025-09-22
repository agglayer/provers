pub mod config;
pub mod contracts;
mod error;

#[cfg(test)]
mod tests;

use std::{panic::AssertUnwindSafe, str::FromStr, sync::Arc};

use aggchain_proof_core::bridge::{
    static_call::{HashChainType, StaticCallStage},
    BridgeL2SovereignChain,
};
use agglayer_interop::types::Digest;
use agglayer_primitives::Address;
use alloy::{
    eips::BlockNumberOrTag, hex, network::AnyNetwork, primitives::B256, providers::Provider,
    sol_types::SolCall,
};
use contracts::{
    GetTrustedSequencerAddress, GlobalExitRootManagerL2SovereignChainRpcClient,
    L2EvmStateSketchFetcher,
};
use eyre::Context as _;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use prover_alloy::{build_alloy_fill_provider, AlloyFillProvider};
use prover_executor::sp1_async;
use sp1_cc_client_executor::{
    io::{EvmSketchInput, Primitives},
    ContractInput, Genesis,
};
use sp1_cc_host_executor::EvmSketch;
use tracing::{debug, info};
use url::Url;

pub use crate::error::Error;
use crate::{
    config::AggchainProofContractsConfig,
    contracts::{
        AggchainFep, AggchainFepRpcClient, GlobalExitRootManagerL2SovereignChain,
        L1OpSuccinctConfigFetcher, L2LocalExitRootFetcher, L2OutputAtBlock, L2OutputAtBlockFetcher,
        OpSuccinctConfig, PolygonRollupManagerRpcClient, PolygonZkevmBridgeV2,
        ZkevmBridgeRpcClient,
    },
};

/// `AggchainContractsClient` is a trait for interacting with the smart
/// contracts relevant for the aggchain prover.
pub trait AggchainContractsClient:
    L2LocalExitRootFetcher
    + L2OutputAtBlockFetcher
    + L1OpSuccinctConfigFetcher
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

    /// Caller address.
    static_call_caller_address: agglayer_primitives::Address,

    /// Evm sketch genesis configuration.
    evm_sketch_genesis: Genesis,

    /// Aggchain FEP opSuccinctConfig name.
    op_succinct_config_name: agglayer_primitives::alloy_primitives::FixedBytes<32>,
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
impl<RpcProvider> L1OpSuccinctConfigFetcher for AggchainContractsRpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Send + Sync,
{
    async fn get_op_succinct_config(&self) -> Result<OpSuccinctConfig, Error> {
        let op_succinct_config = self
            .aggchain_fep
            .opSuccinctConfigs(self.op_succinct_config_name)
            .call()
            .await
            .map_err(Error::OpSuccinctConfigRetrievalError)?;

        Ok(OpSuccinctConfig {
            range_vkey_commitment: (op_succinct_config.rangeVkeyCommitment.0).into(),
            aggregation_vkey_hash: (op_succinct_config.aggregationVkey.0).into(),
            rollup_config_hash: (op_succinct_config.rollupConfigHash.0).into(),
        })
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
        // TODO: Figure out how to deal with interior mutability here — AssertUnwindSafe
        // sounds suboptimal
        sp1_async(AssertUnwindSafe(async move {
            let sketch = EvmSketch::builder()
                .optimism()
                .at_block(prev_l2_block)
                .with_genesis(self.evm_sketch_genesis.clone())
                .el_rpc_url(self.l2_root_provider_endpoint.clone())
                .build()
                .await
                .map_err(Error::HostExecutorPreBlockInitialization)?;

            let caller_address = self.static_call_caller_address;
            let ger_address = *self.global_exit_root_manager_l2.address();
            let bridge_address = *self.polygon_zkevm_bridge_v2.address();

            // Static calls on the hash chains
            {
                host_execute(
                    caller_address,
                    ger_address,
                    &sketch,
                    GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
                    StaticCallStage::PrevHashChain(HashChainType::InsertedGER),
                )
                .await?;

                host_execute(
                    caller_address,
                    ger_address,
                    &sketch,
                    GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
                    StaticCallStage::PrevHashChain(HashChainType::RemovedGER),
                )
                .await?;

                host_execute(
                    caller_address,
                    bridge_address,
                    &sketch,
                    BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
                    StaticCallStage::PrevHashChain(HashChainType::ClaimedGlobalIndex),
                )
                .await?;

                host_execute(
                    caller_address,
                    bridge_address,
                    &sketch,
                    BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
                    StaticCallStage::PrevHashChain(HashChainType::UnsetGlobalIndex),
                )
                .await?;
            }

            // Finalize to retrieve the EVMStateSketch
            let prev_l2_block_sketch = sketch
                .finalize()
                .await
                .map_err(Error::InvalidPreBlockSketchFinalization)?;

            Ok(prev_l2_block_sketch)
        }))
        .await
        .context("Failed getting previous L2 block sketch")
        .map_err(Error::Other)?
    }

    async fn get_new_l2_block_sketch(
        &self,
        new_l2_block: BlockNumberOrTag,
    ) -> Result<EvmSketchInput, Error> {
        // TODO: Figure out how to deal with interior mutability here — AssertUnwindSafe
        // sounds suboptimal
        sp1_async(AssertUnwindSafe(async move {
            let sketch = EvmSketch::builder()
                .optimism()
                .at_block(new_l2_block)
                .with_genesis(self.evm_sketch_genesis.clone())
                .el_rpc_url(self.l2_root_provider_endpoint.clone())
                .build()
                .await
                .map_err(Error::HostExecutorNewBlockInitialization)?;

            let caller_address = self.static_call_caller_address;
            let ger_address = *self.global_exit_root_manager_l2.address();
            let bridge_address = *self.polygon_zkevm_bridge_v2.address();

            // Static call on the bridge address
            host_execute(
                caller_address,
                ger_address,
                &sketch,
                GlobalExitRootManagerL2SovereignChain::bridgeAddressCall {},
                StaticCallStage::BridgeAddress,
            )
            .await?;

            // Static call on the new LER
            host_execute(
                caller_address,
                bridge_address,
                &sketch,
                BridgeL2SovereignChain::getRootCall {},
                StaticCallStage::NewLer,
            )
            .await?;

            // Static calls on the hash chains
            {
                host_execute(
                    caller_address,
                    ger_address,
                    &sketch,
                    GlobalExitRootManagerL2SovereignChain::insertedGERHashChainCall {},
                    StaticCallStage::NewHashChain(HashChainType::InsertedGER),
                )
                .await?;

                host_execute(
                    caller_address,
                    ger_address,
                    &sketch,
                    GlobalExitRootManagerL2SovereignChain::removedGERHashChainCall {},
                    StaticCallStage::NewHashChain(HashChainType::RemovedGER),
                )
                .await?;

                host_execute(
                    caller_address,
                    bridge_address,
                    &sketch,
                    BridgeL2SovereignChain::claimedGlobalIndexHashChainCall {},
                    StaticCallStage::NewHashChain(HashChainType::ClaimedGlobalIndex),
                )
                .await?;

                host_execute(
                    caller_address,
                    bridge_address,
                    &sketch,
                    BridgeL2SovereignChain::unsetGlobalIndexHashChainCall {},
                    StaticCallStage::NewHashChain(HashChainType::UnsetGlobalIndex),
                )
                .await?;
            }

            // Finalize to retrieve the EVMStateSketch
            let new_l2_block_sketch = sketch
                .finalize()
                .await
                .map_err(Error::InvalidNewBlockSketchFinalization)?;

            Ok(new_l2_block_sketch)
        }))
        .await
        .context("Failed getting new L2 block sketch")
        .map_err(Error::Other)?
    }
}

async fn host_execute<C: SolCall, P: Provider<AnyNetwork> + Clone, PT: Primitives>(
    caller_address: Address,
    contract_address: alloy::primitives::Address,
    sketch: &EvmSketch<P, PT>,
    calldata: C,
    stage: StaticCallStage,
) -> Result<(), Error> {
    let output_bytes = sketch
        .call_raw(&ContractInput::new_call(
            contract_address,
            caller_address.into(),
            calldata,
        ))
        .await
        .map_err(|source| Error::InvalidHostStaticCall { source, stage })?;

    debug!("output bytes for static call at stage {stage:?}: {output_bytes:?}");

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
            config.global_exit_root_manager_v2_sovereign_chain.into(),
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
        let polygon_rollup_manager = PolygonRollupManagerRpcClient::new(
            config.polygon_rollup_manager.into(),
            l1_client.clone(),
        );

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
            .map_err(Error::UnableToRetrieveTrustedSequencerAddress)?
            .into();

        let op_succinct_config_name: [u8; 32] = hex::decode(&config.op_succinct_config_name)
            .map_err(|_| {
                Error::InvalidOpSuccinctConfigName(config.op_succinct_config_name.clone())
            })?
            .try_into()
            .map_err(|_| {
                Error::InvalidOpSuccinctConfigName(config.op_succinct_config_name.clone())
            })?;

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
            static_call_caller_address: config.static_call_caller_address,
            evm_sketch_genesis: config::parse_evm_sketch_genesis(&config.evm_sketch_genesis)?,
            op_succinct_config_name: Digest(op_succinct_config_name).into(),
        })
    }
}
