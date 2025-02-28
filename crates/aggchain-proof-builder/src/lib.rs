pub mod config;
mod error;

use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};

use aggchain_proof_contracts::contracts::{
    L1RollupConfigHashFetcher, L2LocalExitRootFetcher, L2OutputAtBlockFetcher,
};
use aggchain_proof_contracts::{AggchainContractsClient, AggchainContractsRpcClient};
use aggchain_proof_core::proof::{AggchainProofWitness, InclusionProof, L1InfoTreeLeaf};
use aggkit_prover_types::Hash;
pub use error::Error;
use futures::{future::BoxFuture, FutureExt};
use prover_alloy::AlloyFillProvider;
use prover_executor::Executor;
use sp1_sdk::{SP1Proof, SP1ProofWithPublicValues, SP1VerifyingKey};
use tower::buffer::Buffer;
use tower::util::BoxService;
use tower::ServiceExt as _;

use crate::config::AggchainProofBuilderConfig;

const MAX_CONCURRENT_REQUESTS: usize = 100;
const ELF: &[u8] =
    include_bytes!("../../../crates/aggchain-proof-program/elf/riscv32im-succinct-zkvm-elf");

pub(crate) type ProverService = Buffer<
    BoxService<prover_executor::Request, prover_executor::Response, prover_executor::Error>,
    prover_executor::Request,
>;

/// All the data `aggchain-proof-builder` needs for the agghchain
/// proof generation. Collected from various sources.
pub struct AggchainProverInputs {
    pub proof_witness: AggchainProofWitness,
    pub start_block: u64,
    pub end_block: u64,
}

pub struct AggchainProofBuilderRequest {
    /// Aggregated full execution proof for the number of aggregated block
    /// spans.
    pub agg_span_proof: SP1ProofWithPublicValues,
    /// First block in the aggregated span.
    pub start_block: u64,
    /// Last block in the aggregated span (inclusive).
    pub end_block: u64,
    /// Root hash of the l1 info tree, containing all relevant GER.
    /// Provided by agg-sender.
    pub l1_info_tree_root_hash: Hash,
    /// Particular leaf of the l1 info tree corresponding
    /// to the max_block.
    pub l1_info_tree_leaf: L1InfoTreeLeaf,
    /// Inclusion proof of the l1 info tree leaf to the
    /// l1 info tree root
    pub l1_info_tree_merkle_proof: [Hash; 32],
    /// Map of the Global Exit Roots with their inclusion proof.
    /// Note: the GER (string) is a base64 encoded string of the GER digest.
    pub ger_inclusion_proofs: HashMap<String, InclusionProof>,
}

#[derive(Clone, Debug)]
pub struct AggchainProofBuilderResponse {
    /// Generated aggchain proof for the block range.
    pub proof: SP1Proof,
    /// First block included in the aggchain proof.
    pub start_block: u64,
    /// Last block included in the aggchain proof.
    pub end_block: u64,
}

/// This service is responsible for building an Aggchain proof.
#[derive(Clone)]
#[allow(unused)]
pub struct AggchainProofBuilder<ContractsClient> {
    /// Client for interacting with the smart contracts relevant for the
    /// aggchain prover.
    contracts_client: Arc<ContractsClient>,

    /// Network id of the l2 chain for which the proof is generated.
    network_id: u32,

    /// Prover client service.
    prover: ProverService,

    /// Verification key for the aggchain proof.
    aggchain_proof_vkey: SP1VerifyingKey,
}

impl AggchainProofBuilder<AggchainContractsRpcClient<AlloyFillProvider>> {
    pub async fn new(config: &AggchainProofBuilderConfig) -> Result<Self, Error> {
        let executor = tower::ServiceBuilder::new()
            .service(Executor::new(
                &config.primary_prover,
                &config.fallback_prover,
                ELF,
            ))
            .boxed();

        let prover = Buffer::new(executor, MAX_CONCURRENT_REQUESTS);
        let aggchain_proof_vkey = Executor::get_vkey(ELF);

        Ok(AggchainProofBuilder {
            contracts_client: Arc::new(
                AggchainContractsRpcClient::new(
                    &config.l1_rpc_endpoint,
                    &config.l2_execution_layer_rpc_endpoint,
                    &config.l2_consensus_layer_rpc_endpoint,
                    config.network_id,
                    &config.contracts,
                )
                .await
                .map_err(Error::ContractsClientInitFailed)?,
            ),
            prover,
            network_id: config.network_id,
            aggchain_proof_vkey,
        })
    }
}

impl<ContractsClient> AggchainProofBuilder<ContractsClient> {
    /// Retrieve l1 and l2 public data needed for aggchain proof generation.
    /// Combine with the rest of the inputs to form an `AggchainProverInputs`.
    pub(crate) async fn retrieve_chain_data(
        contracts_client: Arc<ContractsClient>,
        request: AggchainProofBuilderRequest,
    ) -> Result<AggchainProverInputs, Error>
    where
        ContractsClient:
            L2LocalExitRootFetcher + L2OutputAtBlockFetcher + L1RollupConfigHashFetcher,
    {
        let _prev_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.start_block - 1)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let _new_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let _l2_pre_root_output_at_block = contracts_client
            .get_l2_output_at_block(request.start_block - 1)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let _claim_root_output_at_block = contracts_client
            .get_l2_output_at_block(request.end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let _rollup_config_hash = contracts_client
            .get_rollup_config_hash()
            .await
            .map_err(Error::L1ChainDataRetrievalError)?;

        todo!()
    }

    /// Generate aggchain proof
    pub(crate) async fn generate_aggchain_proof(
        mut _prover: ProverService,
        _inputs: AggchainProverInputs,
    ) -> Result<AggchainProofBuilderResponse, Error> {
        todo!()
    }
}

impl<ContractsClient> tower::Service<AggchainProofBuilderRequest>
    for AggchainProofBuilder<ContractsClient>
where
    ContractsClient: AggchainContractsClient + Send + Sync + 'static,
{
    type Response = AggchainProofBuilderResponse;

    type Error = Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.prover.poll_ready(cx).map_err(|e| {
            if let Some(error) = e.downcast_ref::<prover_executor::Error>() {
                Error::ProverExecutorError(error.clone())
            } else {
                Error::ProverServiceError(e.to_string())
            }
        })
    }

    fn call(&mut self, req: AggchainProofBuilderRequest) -> Self::Future {
        let contracts_client = self.contracts_client.clone();
        let prover = self.prover.clone();
        async move {
            // Retrieve all the necessary public inputs. Combine with
            // the data provided by the agg-sender in the request.
            let aggchain_prover_inputs = Self::retrieve_chain_data(contracts_client, req).await?;

            // Generate aggchain proof.
            Self::generate_aggchain_proof(prover, aggchain_prover_inputs).await
        }
        .boxed()
    }
}
