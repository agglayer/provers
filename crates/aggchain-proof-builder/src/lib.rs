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
use prover_executor::{Executor, NetworkExecutor, Request, Response};
use serde::{Deserialize, Serialize};
use sp1_sdk::{Prover, ProverClient, SP1ProofWithPublicValues, SP1VerifyingKey};
use tower::util::BoxCloneService;

use crate::config::AggchainProofBuilderConfig;
use crate::provider::json_rpc::{build_http_retry_provider, AlloyProvider};

const ELF: &[u8] =
    include_bytes!("../../../crates/aggchain-proof-program/elf/riscv32im-succinct-zkvm-elf");

/// Aggchain proof is generated from FEP proof and additional
/// bridge inputs.
/// Resulting work of the aggchain proof builder.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AggchainProof {
    //pub proof: SP1ProofWithPublicValues,
    //TODO add all necessary fields
}

pub struct AggchainProofBuilderRequest {
    /// Aggregated full execution proof for the number of aggregated block spans
    pub agg_span_proof: SP1ProofWithPublicValues,
    /// First block in the aggregated span
    pub start_block: u64,
    /// Last block in the aggregated span (inclusive)
    pub end_block: u64,
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
#[derive(Clone)]
#[allow(unused)]
pub struct AggchainProofBuilder {
    /// Mainnet node rpc client
    l1_client: Arc<AlloyProvider>,

    /// L2 node rpc client
    l2_client: Arc<AlloyProvider>,

    /// Network id of the l2 chain for which the proof is generated
    network_id: u32,

    /// Prover client executor
    prover: prover_executor::NetworkExecutor,

    /// Verification key for the aggchain proof
    aggchain_proof_vkey: SP1VerifyingKey,
}

impl AggchainProofBuilder {
    pub fn new(config: &AggchainProofBuilderConfig) -> Result<Self, Error> {
        let network_prover = ProverClient::builder().network().build();
        let (proving_key, verification_key) = network_prover.setup(ELF);
        let network_executor =
            NetworkExecutor {
                prover: Arc::new(network_prover),
                proving_key,
                verification_key,
                timeout: config.proving_timeout,
            };
        // let prover = Executor::create_prover(&config.primary_prover, ELF);
        let aggchain_proof_vkey = Executor::get_vkey(ELF);
        Ok(AggchainProofBuilder {
            l1_client: Arc::new(
                build_http_retry_provider(
                    &config.l1_rpc_endpoint,
                    config::HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                    config::HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
                )
                .map_err(Error::AlloyProviderError)?,
            ),
            l2_client: Arc::new(
                build_http_retry_provider(
                    &config.l2_rpc_endpoint,
                    config::HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                    config::HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
                )
                .map_err(Error::AlloyProviderError)?,
            ),
            prover: network_executor,
            network_id: config.network_id,
            aggchain_proof_vkey,
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
