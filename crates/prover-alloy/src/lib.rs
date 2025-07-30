use std::{str::FromStr, time::Duration};

use agglayer_evm_client::AlloyRpc;
use alloy::{
    network::Ethereum,
    providers::{
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
        Identity, ProviderBuilder, RootProvider,
    },
    rpc::client::ClientBuilder,
    transports::{http::reqwest, layers::RetryBackoffLayer},
};
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

/// Creates a client builder with common configuration for HTTP RPC clients
fn create_client_builder(
    rpc_url: &url::Url,
    backoff: u64,
    max_retries: u32,
) -> Result<ClientBuilder, anyhow::Error> {
    let retry_policy = RetryBackoffLayer::new(max_retries, backoff, 5);
    let reqwest_client = reqwest::ClientBuilder::new()
        .pool_max_idle_per_host(HTTP_CLIENT_MAX_IDLE_CONNECTIONS_PER_HOST)
        .pool_idle_timeout(Duration::from_secs(
            HTTP_CLIENT_CONNECTION_POOL_IDLE_TIMEOUT,
        ))
        .build()?;

    let http = alloy::transports::http::Http::with_client(reqwest_client, rpc_url.clone());
    let is_local = http.guess_local();
    Ok(ClientBuilder::default()
        .layer(retry_policy)
        .transport(http, is_local))
}

pub fn build_alloy_fill_provider(
    rpc_url: &url::Url,
    backoff: u64,
    max_retries: u32,
) -> Result<AlloyFillProvider, anyhow::Error> {
    let client = create_client_builder(rpc_url, backoff, max_retries)?;
    Ok(ProviderBuilder::new().on_client(client))
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
        let client = create_client_builder(rpc_url, backoff, max_retries)?;
        Ok(AlloyProvider {
            client: ProviderBuilder::new().on_client(client),
        })
    }
}

impl AlloyRpc for AlloyProvider {
    fn alloy_rpc(&self) -> &AlloyFillProvider {
        &self.client
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
