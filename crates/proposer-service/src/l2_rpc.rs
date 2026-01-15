use std::sync::Arc;

use alloy_primitives::{B256, U64};
use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::Error;

/// A block identifier containing hash and number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockId {
    pub hash: B256,
    pub number: U64,
}

/// Response from the `optimism_safeHeadAtL1Block` RPC method.
///
/// This method returns the safe L2 block that was derived from data up to and
/// including the specified L1 block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeHeadAtL1Block {
    /// The L1 block that was queried.
    pub l1_block: BlockId,
    /// The safe L2 head at this L1 block.
    pub safe_head: BlockId,
}

/// Trait for fetching the safe L2 head at a given L1 block.
#[async_trait::async_trait]
pub trait L2SafeHeadFetcher: Send + Sync {
    /// Returns the safe L2 block that was derived from data up to and including
    /// the specified L1 block number.
    async fn get_safe_head_at_l1_block(
        &self,
        l1_block_number: u64,
    ) -> Result<SafeHeadAtL1Block, Error>;
}

/// Client for interacting with the L2 consensus layer (rollup node) RPC.
pub struct L2ConsensusLayerClient {
    client: HttpClient,
}

impl L2ConsensusLayerClient {
    /// Creates a new L2 consensus layer client.
    pub fn new(endpoint: &Url) -> Result<Self, Error> {
        let client = HttpClient::builder()
            .build(endpoint.as_str())
            .map_err(Error::L2ConsensusLayerClientInit)?;

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl L2SafeHeadFetcher for L2ConsensusLayerClient {
    async fn get_safe_head_at_l1_block(
        &self,
        l1_block_number: u64,
    ) -> Result<SafeHeadAtL1Block, Error> {
        let params = rpc_params![format!("0x{l1_block_number:x}")];
        let response: SafeHeadAtL1Block = self
            .client
            .request("optimism_safeHeadAtL1Block", params)
            .await
            .map_err(Error::L2SafeHeadFetch)?;

        Ok(response)
    }
}

#[async_trait::async_trait]
impl<T: L2SafeHeadFetcher> L2SafeHeadFetcher for Arc<T> {
    async fn get_safe_head_at_l1_block(
        &self,
        l1_block_number: u64,
    ) -> Result<SafeHeadAtL1Block, Error> {
        (**self).get_safe_head_at_l1_block(l1_block_number).await
    }
}

#[cfg(any(test, feature = "testutils"))]
mockall::mock! {
    /// Mock implementation of [`L2SafeHeadFetcher`] for testing.
    pub L2SafeHeadFetcher {}

    #[async_trait::async_trait]
    impl L2SafeHeadFetcher for L2SafeHeadFetcher {
        async fn get_safe_head_at_l1_block(
            &self,
            l1_block_number: u64,
        ) -> Result<SafeHeadAtL1Block, Error>;
    }
}
