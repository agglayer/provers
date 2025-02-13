use std::str::FromStr as _;
use std::time::Duration;

use alloy::network::Ethereum;
use alloy::providers::fillers::{
    BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
};
use alloy::providers::Identity;
use alloy::transports::http::reqwest;
use alloy::transports::layers::RetryBackoffLayer;
pub use alloy::*;
use alloy::{
    providers::{ProviderBuilder, RootProvider},
    rpc::client::ClientBuilder,
};
use url::Url;

const HTTP_CLIENT_CONNECTION_POOL_IDLE_TIMEOUT: u64 = 90;
const HTTP_CLIENT_MAX_IDLE_CONNECTIONS_PER_HOST: usize = 64;
pub const DEFAULT_HTTP_RPC_NODE_INITIAL_BACKOFF_MS: u64 = 5000;
pub const DEFAULT_HTTP_RPC_NODE_BACKOFF_MAX_RETRIES: u32 = 64;

pub type AlloyProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
    Ethereum,
>;

pub fn build_http_retry_provider(
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
    Ok(ProviderBuilder::new().on_client(client))
}

pub fn default_l1_url() -> Url {
    Url::from_str("http://anvil-mock-l1-rpc:8545").unwrap()
}

pub fn default_l2_url() -> Url {
    Url::from_str("http://anvil-mock-l2-rpc:8545").unwrap()
}
