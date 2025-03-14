pub mod config;
mod error;

use std::sync::Arc;
use std::task::{Context, Poll};

use aggchain_proof_contracts::contracts::{
    L1RollupConfigHashFetcher, L2LocalExitRootFetcher, L2OutputAtBlockFetcher,
};
use aggchain_proof_contracts::AggchainContractsClient;
use aggchain_proof_core::proof::{AggchainProofPublicValues, AggchainProofWitness};
use aggchain_proof_types::{AggchainProofInputs, Digest};
use bincode::Options;
pub use error::Error;
use futures::{future::BoxFuture, FutureExt};
use prover_executor::{Executor, ProofType};
use sp1_sdk::{SP1Stdin, SP1VerifyingKey};
use tower::buffer::Buffer;
use tower::util::BoxService;
use tower::ServiceExt as _;

use crate::config::AggchainProofBuilderConfig;

const MAX_CONCURRENT_REQUESTS: usize = 100;
pub const ELF: &[u8] =
    include_bytes!("../../../crates/aggchain-proof-program/elf/riscv32im-succinct-zkvm-elf");

pub(crate) type ProverService = Buffer<
    BoxService<prover_executor::Request, prover_executor::Response, prover_executor::Error>,
    prover_executor::Request,
>;

/// All the data `aggchain-proof-builder` needs for the agghchain
/// proof generation. Collected from various sources.
#[derive()]
pub struct AggchainProverInputs {
    pub proof_witness: AggchainProofWitness,
    pub start_block: u64,
    pub end_block: u64,
}

pub struct AggchainProofBuilderRequest {
    /// Aggregated full execution proof for the number of aggregated block
    /// spans.
    pub aggregation_proof: sp1_core_executor::SP1ReduceProof<sp1_prover::InnerSC>,
    /// Last block in the agg_span_proof provided by the proposer.
    /// Could be different from the max_end_block requested by the agg-sender.
    pub end_block: u64,
    /// Aggchain proof request information, public inputs, bridge data,...
    pub aggchain_proof_inputs: AggchainProofInputs,
}

#[derive(Clone, Debug)]
pub struct AggchainProofBuilderResponse {
    /// Generated aggchain proof for the block range.
    pub proof: Vec<u8>,
    /// Aggchain params
    pub aggchain_params: Vec<u8>,
    /// First block included in the aggchain proof.
    pub start_block: u64,
    /// Last block included in the aggchain proof.
    pub end_block: u64,
    /// Output root
    pub output_root: Digest,
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

impl<ContractsClient> AggchainProofBuilder<ContractsClient> {
    pub async fn new(
        config: &AggchainProofBuilderConfig,
        contracts_client: Arc<ContractsClient>,
    ) -> Result<Self, Error> {
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
            contracts_client,
            prover,
            network_id: config.network_id,
            aggchain_proof_vkey,
        })
    }

    /// Retrieve l1 and l2 public data needed for aggchain proof generation.
    /// Combine with the rest of the inputs to form an `AggchainProverInputs`.
    pub(crate) async fn retrieve_chain_data(
        contracts_client: Arc<ContractsClient>,
        request: AggchainProofBuilderRequest,
        _network_id: u32,
    ) -> Result<
        (
            AggchainProofWitness,
            sp1_core_executor::SP1ReduceProof<sp1_prover::InnerSC>,
        ),
        Error,
    >
    where
        ContractsClient:
            L2LocalExitRootFetcher + L2OutputAtBlockFetcher + L1RollupConfigHashFetcher,
    {
        let _prev_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.aggchain_proof_inputs.start_block - 1)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let _new_local_exit_root = contracts_client
            .get_l2_local_exit_root(request.end_block)
            .await
            .map_err(Error::L2ChainDataRetrievalError)?;

        let _l2_pre_root_output_at_block = contracts_client
            .get_l2_output_at_block(request.aggchain_proof_inputs.start_block - 1)
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

        todo!("Fill the proof witness struct with the retrieved data");
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
        let mut prover = self.prover.clone();
        let network_id = self.network_id;
        async move {
            let start_block = req.aggchain_proof_inputs.start_block;
            let end_block = req.end_block;
            // Retrieve all the necessary public inputs. Combine with
            // the data provided by the agg-sender in the request.
            let (aggchain_proof_witness, proof) =
                Self::retrieve_chain_data(contracts_client, req, network_id).await?;

            let mut stdin = SP1Stdin::new();

            let vkey = proof.vk.clone();

            stdin.write(&aggchain_proof_witness);
            stdin.write_proof(proof, vkey);

            let result = prover
                .call(prover_executor::Request {
                    stdin,
                    proof_type: ProofType::Stark,
                })
                .await
                .map_err(|error| Error::ProverFailedToExecute(anyhow::Error::from_boxed(error)));

            match result {
                Ok(prover_executor::Response { proof }) => {
                    let public_input: AggchainProofPublicValues =
                        bincode::deserialize(proof.public_values.as_slice()).unwrap();

                    let stark = proof
                        .proof
                        .try_as_compressed()
                        .ok_or(Error::GeneratedProofIsNotCompressed)?;

                    Ok(AggchainProofBuilderResponse {
                        proof: bincode::DefaultOptions::new()
                            .with_big_endian()
                            .with_fixint_encoding()
                            .serialize(&stark)
                            .map_err(Error::UnableToSerializeProof)?,
                        aggchain_params: public_input.aggchain_params.to_vec(),
                        start_block,
                        end_block,
                        // TODO: Define the output root with the witness data
                        output_root: Default::default(),
                    })
                }
                Err(_) => todo!(),
            }
        }
        .boxed()
    }
}
