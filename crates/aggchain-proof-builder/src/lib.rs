mod provider;

pub mod aggchain_prover;

use std::sync::Arc;
use std::task::{Context, Poll};

use aggchain_proof_core::proof::AggchainProofWitness;
use aggkit_prover_config::aggchain_proof_service::{
    HTTP_RPC_NODE_BACKOFF_MAX_RETRIES, HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
};
use aggkit_prover_config::AggchainProofBuilderConfig;
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::network::primitives::BlockTransactionsKind;
use alloy::primitives::B256;
use alloy::providers::Provider;
use alloy::transports::{RpcError, TransportErrorKind};
use futures::{future::BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use sp1_sdk::{SP1ProofWithPublicValues, SP1VerifyingKey};

use crate::aggchain_prover::AggChainProver;
use crate::provider::json_rpc::{build_http_retry_provider, AlloyProvider};

const ELF: &[u8] =
    include_bytes!("../../../crates/aggchain-proof-program/elf/riscv32im-succinct-zkvm-elf");

/// Agghchain proof is generated from FEP proof and additional
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
pub struct AgghcainProofBuilderResponse {
    pub proof: AggchainProof,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Alloy error: {0}")]
    AlloyProviderError(anyhow::Error),

    #[error("Alloy transport rpc error: {0:?}")]
    AlloyRpcTransportError(#[from] RpcError<TransportErrorKind>),

    #[error(transparent)]
    ProofGenerationError(#[from] aggchain_proof_core::error::ProofError),

    #[error("Prover program error: {0}")]
    ProverProgramError(#[from] std::io::Error),
}

/// This service is responsible for building an Aggchain proof.
#[derive(Clone)]
#[allow(dead_code)]
pub struct AggchainProofBuilder<Prover> {
    l1_client: Arc<AlloyProvider>,

    l2_client: Arc<AlloyProvider>,

    prover: Arc<Prover>,

    /// Rollup id of the l2 chain for which the proof is generated
    rollup_id: u32,

    /// Verification key for the aggchain proof
    aggchain_proof_vkey: SP1VerifyingKey,
}

impl<Prover: AggChainProver> AggchainProofBuilder<Prover> {
    pub fn new(config: &AggchainProofBuilderConfig, prover: Arc<Prover>) -> Result<Self, Error> {
        let (_aggchain_proof_pkey, aggchain_proof_vkey) = { prover.get_vkey(ELF) };

        Ok(AggchainProofBuilder {
            l1_client: Arc::new(
                build_http_retry_provider(
                    &config.l1_rpc_endpoint,
                    HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                    HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
                )
                .map_err(Error::AlloyProviderError)?,
            ),
            l2_client: Arc::new(
                build_http_retry_provider(
                    &config.l2_rpc_endpoint,
                    HTTP_RPC_NODE_INITIAL_BACKOFF_MS,
                    HTTP_RPC_NODE_BACKOFF_MAX_RETRIES,
                )
                .map_err(Error::AlloyProviderError)?,
            ),
            prover,
            rollup_id: config.rollup_id,
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

impl<Prover: AggChainProver> tower::Service<AggchainProofBuilderRequest>
    for AggchainProofBuilder<Prover>
{
    type Response = AgghcainProofBuilderResponse;

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

            Ok(AgghcainProofBuilderResponse {
                proof: Default::default(),
            })
        }
        .boxed()
    }
}
