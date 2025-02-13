pub mod config;
mod provider;

use std::sync::Arc;
use std::task::{Context, Poll};

use aggchain_proof_core::proof::AggchainProofWitness;
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::network::primitives::BlockTransactionsKind;
use alloy::primitives::B256;
use alloy::providers::Provider;
use alloy::transports::{RpcError, TransportErrorKind};
use futures::{future::BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use sp1_sdk::SP1ProofWithPublicValues;

use crate::config::AggchainProofBuilderConfig;
use crate::provider::json_rpc::{build_http_retry_provider, AlloyProvider};

/// Aggchain proof is generated from FEP proof and additional
/// bridge inputs.
/// Resulting work of the aggchain proof builder.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AggchainProof {
    //pub proof: SP1ProofWithPublicValues,
    //TODO add all necessary fields
}

pub struct AggchainProofBuilderRequest {
    pub agg_span_proof: SP1ProofWithPublicValues,
    // TODO add rest of the fields
}

#[derive(Clone, Debug)]
pub struct AggchainProofBuilderResponse {
    pub proof: AggchainProof,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AlloyProviderError(anyhow::Error),

    #[error(transparent)]
    AlloyRpcTransportError(#[from] RpcError<TransportErrorKind>),

    #[error(transparent)]
    ProofGenerationError(#[from] aggchain_proof_core::error::ProofError),
}

/// This service is responsible for building an Aggchain proof.
#[derive(Debug, Clone)]
pub struct AggchainProofBuilder {
    l1_client: Arc<AlloyProvider>,

    _l2_client: Arc<AlloyProvider>,
}

impl AggchainProofBuilder {
    pub fn new(config: &AggchainProofBuilderConfig) -> Result<Self, Error> {
        Ok(AggchainProofBuilder {
            l1_client: Arc::new(
                build_http_retry_provider(
                    &config.l1_rpc_endpoint,
                    config::HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                    config::HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
                )
                .map_err(Error::AlloyProviderError)?,
            ),
            _l2_client: Arc::new(
                build_http_retry_provider(
                    &config.l2_rpc_endpoint,
                    config::HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                    config::HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
                )
                .map_err(Error::AlloyProviderError)?,
            ),
        })
    }

    pub async fn get_l1_block_hash(&self, block_num: u64) -> Result<B256, Error> {
        let block = self
            .l1_client
            .get_block(
                BlockId::Number(BlockNumberOrTag::Number(block_num)),
                BlockTransactionsKind::Hashes,
            )
            .await
            .map_err(Error::AlloyRpcTransportError)?
            .ok_or(Error::AlloyProviderError(anyhow::anyhow!(
                "target block {block_num} does not exist"
            )))?;

        Ok(block.header.hash)
    }

    // Retrieve l1 and l2 public data needed for aggchain proof generation
    pub async fn retrieve_chain_data(&self) -> Result<(), Error> {
        //TODO decide output structure
        todo!()
    }

    // Generate aggchain proof
    pub async fn generate_aggchain_proof(
        &self,
        mut _aggchain_proof_witness: AggchainProofWitness,
    ) -> Result<AggchainProof, Error> {
        //TODO implement
        todo!()
    }
}

impl tower::Service<AggchainProofBuilderRequest> for AggchainProofBuilder {
    type Response = AggchainProofBuilderResponse;

    type Error = Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, _req: AggchainProofBuilderRequest) -> Self::Future {
        async move {
            //TODO implement

            // Call all necessary data retrieval
            //self.retrieve_chain_data().await?;

            // Generate proof
            //self.generate_aggchain_proof().await?;

            Ok(AggchainProofBuilderResponse {
                proof: Default::default(),
            })
        }
        .boxed()
    }
}
