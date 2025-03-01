pub mod config;
pub mod contracts;
mod error;

use std::str::FromStr;
use std::sync::Arc;

use aggchain_proof_core::Digest;
use alloy::primitives::B256;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;
use prover_alloy::{build_alloy_fill_provider, AlloyFillProvider};
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
    L2LocalExitRootFetcher + L2OutputAtBlockFetcher + L1RollupConfigHashFetcher
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

    /// L2 rpc consensus layer client (rollup node).
    l2_cl_client: Arc<HttpClient>,

    /// Polygon zkevm bridge contract on the l2 network.
    polygon_zkevm_bridge_v2: ZkevmBridgeRpcClient<RpcProvider>,

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
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::primitives::B256;
    use prover_alloy::AlloyFillProvider;

    use crate::AggchainContractsRpcClient;

    #[test]
    fn parsing_l2_output_root() -> Result<(), Box<dyn std::error::Error>> {
        let json_str = r#"{"version":"0x0000000000000000000000000000000000000000000000000000000000000000",
        "outputRoot":"0xf9758545eb67c1a90276b44bb80047fa72148f88c69a8653f36cd157f537bde4",
        "blockRef":{"hash":"0x2d0d159b47e89cd85b82c18d217fa47f5901e81e71ae80356854849656b43354","number":16, "parentHash":"0xa64a3a659538ce9fa247ddcd83a6ce51442de0047e5a01aa6c11833493819831","timestamp":1740413570,
        "l1origin":{"hash":"0xa3a945dcb7f80f558631917e37e7b633fb430347a303a025b944bdfdc5f4c5a3","number":13},"sequenceNumber":8},
        "withdrawalStorageRoot":"0x8ed4baae3a927be3dea54996b4d5899f8c01e7594bf50b17dc1e741388ce3d12",
        "stateRoot":"0x965938f5738ae8c501d7de25f61597ea2a76ea2802fc535233eb427376b43328",
        "syncStatus":{"current_l1":{"hash":"0x62735836100c2257429e9cfb80250b5d0df29d0ea9c6bcf0f6da8c2a355115d8","number":29,"parentHash":"0xb036ae96aabc83a1b43ced9de73f4e8b8288cdffcbfbcc29a62ae015ec2d696c","timestamp":1740413646},"current_l1_finalized":{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","number":0,"parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":0},"head_l1":{"hash":"0x907f61e597d733f603350f7a9eae4a2b444ebda317460f3d67e8d2e3eb53466c","number":30,"parentHash":"0x62735836100c2257429e9cfb80250b5d0df29d0ea9c6bcf0f6da8c2a355115d8","timestamp":1740413652},"safe_l1":{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","number":0,"parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":0},"finalized_l1":{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","number":0,"parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":0},"unsafe_l2":{"hash":"0x04cc3d231feefa1685bf2ddf192085b72caf80901b2b96416af107347d66c493","number":59,"parentHash":"0x6521911591fd0cada5e864361a84b6bdd6619b4103011c1cab34b07488041d05","timestamp":1740413656,"l1origin":{"hash":"0xe35ebcb9669083f8b1c54be0b4b073ea620b5419d9cbcdbce808b21ba1f1a577","number":27},"sequenceNumber":0},"safe_l2":{"hash":"0x049bbccee5391aee272fe5a90e8243b0af5cef59bbf0fb971cb62314d9149780","number":41,"parentHash":"0xd47dff00fd82e599635eac5139ddea0070d57cd1111be64c604db8cc64f2396b","timestamp":1740413620,"l1origin":{"hash":"0xdb22ab9cb1619cd14f3e9092c7729faf0efa711afcc7f5440b44d5ea6263239e","number":21},"sequenceNumber":0},"finalized_l2":{"hash":"0x0dc6c55aa95cce979b62983736b1ca066560db65bac8efbb4050ddb412b4ad8c","number":0,"parentHash":"0x0000000000000000000000000000000000000000000000000000000000000000","timestamp":1740413538,"l1origin":{"hash":"0x12ec696bd1cb110f89513b620071935f09460ee6a81fea25fc2f400ea816bb7b","number":11},"sequenceNumber":0},"pending_safe_l2":{"hash":"0x049bbccee5391aee272fe5a90e8243b0af5cef59bbf0fb971cb62314d9149780","number":41,"parentHash":"0xd47dff00fd82e599635eac5139ddea0070d57cd1111be64c604db8cc64f2396b","timestamp":1740413620,"l1origin":{"hash":"0xdb22ab9cb1619cd14f3e9092c7729faf0efa711afcc7f5440b44d5ea6263239e","number":21},"sequenceNumber":0},"cross_unsafe_l2":{"hash":"0x04cc3d231feefa1685bf2ddf192085b72caf80901b2b96416af107347d66c493","number":59,"parentHash":"0x6521911591fd0cada5e864361a84b6bdd6619b4103011c1cab34b07488041d05","timestamp":1740413656,"l1origin":{"hash":"0xe35ebcb9669083f8b1c54be0b4b073ea620b5419d9cbcdbce808b21ba1f1a577","number":27},"sequenceNumber":0},"local_safe_l2":{"hash":"0x049bbccee5391aee272fe5a90e8243b0af5cef59bbf0fb971cb62314d9149780","number":41,"parentHash":"0xd47dff00fd82e599635eac5139ddea0070d57cd1111be64c604db8cc64f2396b","timestamp":1740413620,
        "l1origin":{"hash":"0xdb22ab9cb1619cd14f3e9092c7729faf0efa711afcc7f5440b44d5ea6263239e","number":21},"sequenceNumber":0}}}"#;
        let result = AggchainContractsRpcClient::<AlloyFillProvider>::parse_l2_output_root(
            serde_json::from_str(json_str).unwrap(),
        )?;

        assert_eq!(B256::from(result.version.0), B256::default());
        assert_eq!(
            B256::from(result.latest_block_hash.0),
            B256::from_str("0x2d0d159b47e89cd85b82c18d217fa47f5901e81e71ae80356854849656b43354")
                .unwrap()
        );
        assert_eq!(
            B256::from(result.output_root.0),
            B256::from_str("0xf9758545eb67c1a90276b44bb80047fa72148f88c69a8653f36cd157f537bde4")
                .unwrap()
        );

        Ok(())
    }
}
