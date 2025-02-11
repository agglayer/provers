use std::sync::Arc;
use std::task::{Context, Poll};

use aggkit_prover_config::proposer_service::ProposerServiceConfig;
use futures::{future::BoxFuture, FutureExt};
use proposer_client::agg_span_prover::new_agg_span_prover;
use proposer_client::rpc::ProposerRpcClient;
use proposer_client::{ProofId, ProposerClient};
pub use proposer_client::{ProposerRequest, ProposerResponse};
use sp1_sdk::NetworkProver;

pub mod error;

pub use error::Error;

#[derive(Clone)]
pub struct ProposerService {
    pub client: Arc<ProposerClient<ProposerRpcClient, NetworkProver>>,
}

impl ProposerService {
    pub fn new(config: &ProposerServiceConfig) -> Result<Self, crate::error::Error> {
        let proposer_rpc_client = ProposerRpcClient::new(config.client.proposer_endpoint.as_str())?;
        let network_prover = new_agg_span_prover(config.client.sp1_cluster_endpoint.as_str());
        Ok(Self {
            client: Arc::new(ProposerClient::new(
                proposer_rpc_client,
                network_prover,
                Some(config.client.proving_timeout),
            )?),
        })
    }
}

impl tower::Service<ProposerRequest> for ProposerService {
    type Response = ProposerResponse;

    type Error = Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ProposerRequest) -> Self::Future {
        let client = self.client.clone();

        async move {
            // Request the AggSpanProof generation from the proposer
            let response = client.request_agg_proof(req.into()).await?;
            let proof_id = ProofId(response.proof_id);

            // Wait for the prover to finish aggregating span proofs
            let proofs = client.wait_for_proof(proof_id).await?;

            Ok(ProposerResponse {
                agg_span_proof: proofs,
                start_block: response.start_block,
                end_block: response.end_block,
            })
        }
        .boxed()
    }
}
