use std::str::FromStr;
use std::time::Duration;

use alloy::network::Ethereum;
use alloy::providers::fillers::{
    BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
};
use alloy::providers::Identity;
use alloy::providers::{Provider as _, ProviderBuilder};
use alloy::transports::http::reqwest;
use alloy::transports::layers::RetryBackoffLayer;
use alloy::{providers::RootProvider, rpc::client::ClientBuilder};
pub use async_trait::async_trait;
use derive_more::{From, FromStr};
use educe::Educe;
use serde::{Deserialize, Serialize};
use url::Url;

const HTTP_CLIENT_CONNECTION_POOL_IDLE_TIMEOUT: u64 = 90;
const HTTP_CLIENT_MAX_IDLE_CONNECTIONS_PER_HOST: usize = 64;
pub const DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS: u64 = 5000;
pub const DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES: u32 = 64;

pub type AlloyFillProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
    Ethereum,
>;

pub fn build_alloy_fill_provider(
    rpc_url: &url::Url,
    backoff: u64,
    max_retries: u32,
) -> Result<AlloyFillProvider, anyhow::Error> {
    let retry_policy = RetryBackoffLayer::new(max_retries, backoff, 5);
    let reqwest_client = reqwest::ClientBuilder::new()
        .pool_max_idle_per_host(HTTP_CLIENT_MAX_IDLE_CONNECTIONS_PER_HOST)
        .pool_idle_timeout(Duration::from_secs(
            HTTP_CLIENT_CONNECTION_POOL_IDLE_TIMEOUT,
        ))
        .build()?;

    let http = alloy::transports::http::Http::with_client(reqwest_client, rpc_url.clone());
    let is_local = http.guess_local();
    let client = ClientBuilder::default()
        .layer(retry_policy)
        .transport(http, is_local);

    Ok(ProviderBuilder::new().on_client(client))
}

#[async_trait]
#[cfg_attr(feature = "testutils", mockall::automock)]
pub trait Provider {
    async fn get_block_hash(&self, block_number: u64) -> anyhow::Result<alloy::primitives::B256>;

    async fn get_block_number(&self, block_hash: alloy::primitives::B256) -> anyhow::Result<u64>;
}

/// Wrapper around alloy `Provider` client.
/// Performs ETH node response data processing where needed but
/// allows direct use of the provider if necessary.
pub struct AlloyProvider {
    client: AlloyFillProvider,
}

impl AlloyProvider {
    pub fn new(
        rpc_url: &url::Url,
        backoff: u64,
        max_retries: u32,
    ) -> Result<AlloyProvider, anyhow::Error> {
        let retry_policy = RetryBackoffLayer::new(max_retries, backoff, 5);
        let reqwest_client = reqwest::ClientBuilder::new()
            .pool_max_idle_per_host(HTTP_CLIENT_MAX_IDLE_CONNECTIONS_PER_HOST)
            .pool_idle_timeout(Duration::from_secs(
                HTTP_CLIENT_CONNECTION_POOL_IDLE_TIMEOUT,
            ))
            .build()?;

        let http = alloy::transports::http::Http::with_client(reqwest_client, rpc_url.clone());
        let is_local = http.guess_local();
        let client = ClientBuilder::default()
            .layer(retry_policy)
            .transport(http, is_local);

        Ok(AlloyProvider {
            client: ProviderBuilder::new().on_client(client),
        })
    }

    pub fn provider(&self) -> &AlloyFillProvider {
        &self.client
    }
}

#[async_trait]
impl Provider for AlloyProvider {
    async fn get_block_hash(
        &self,
        block_number: u64,
    ) -> Result<alloy::primitives::B256, anyhow::Error> {
        let hash = self
            .client
            .get_block_by_number(block_number.into())
            .await
            .map_err(|error| anyhow::anyhow!("Failed to get L1 block hash: {:?}", error))?
            .ok_or(anyhow::anyhow!(
                "target block {block_number} does not exist"
            ))?
            .header
            .hash;
        Ok(hash)
    }

    async fn get_block_number(
        &self,
        block_hash: alloy::primitives::B256,
    ) -> Result<u64, anyhow::Error> {
        let number = self
            .client
            .get_block_by_hash(block_hash)
            .await
            .map_err(|error| anyhow::anyhow!("Failed to get L1 block number: {:?}", error))?
            .ok_or(anyhow::anyhow!("target block {block_hash} does not exist"))?
            .header
            .number;
        Ok(number)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, From, FromStr, Educe)]
#[serde(transparent)]
#[educe(Default)]
pub struct L1RpcEndpoint {
    #[educe(Default = Url::from_str("http://anvil-mock-l1-rpc:8545").unwrap())]
    pub url: Url,
}

pub fn default_l2_execution_layer_url() -> Url {
    Url::from_str("http://anvil-mock-l2-rpc:8545").unwrap()
}

pub fn default_l2_consensus_layer_url() -> Url {
    Url::from_str("http://rollup-node-mock-l2-rpc:8545").unwrap()
}
