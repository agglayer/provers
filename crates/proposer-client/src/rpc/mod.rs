use std::fmt::Display;
use std::time::Duration;

use alloy_primitives::B256;
pub use op_succinct_grpc::proofs as grpc;
use tracing::{error, info};

use crate::{
    error::{self, Error, ProofRequestError},
    GrpcUri, MockProofId, RequestId,
};

mod proofs_service_types;

use grpc::proofs_client::ProofsClient;

/// Proposer client that requests the generation
/// of the aggregation proof from the proposer and gets
/// request_id in response.
#[tonic::async_trait]
pub trait AggregationProofProposer {
    async fn request_agg_proof(
        &self,
        request: AggregationProofProposerRequest,
    ) -> Result<AggregationProofProposerResponse, Error>;

    async fn get_mock_proof(
        &self,
        request: MockProofProposerRequest,
    ) -> Result<MockProofProposerResponse, Error>;
}

/// Request format for the proposer `proofs_requestAggProof`
#[derive(Debug)]
pub struct AggregationProofProposerRequest {
    /// Last block that has already been proven before this request.
    pub last_proven_block: u64,

    /// Maximum block number for the aggregation proof.
    pub requested_end_block: u64,

    /// L1 block number corresponding to requested_end_block.
    pub l1_block_number: u64,

    /// L1 block hash.
    pub l1_block_hash: B256,
}

/// Response for the external proposer `request_span_proof` call
#[derive(Debug)]
pub struct AggregationProofProposerResponse {
    /// Proof request_id, used to fetch the proof from the cluster.
    pub request_id: RequestId,

    /// Last block already proven before this aggregation proof.
    pub last_proven_block: u64,

    /// End block for the aggregation proof.
    pub end_block: u64,
}

impl Display for AggregationProofProposerResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "last proven block: {}, end block: {}, request_id: {}",
            self.last_proven_block, self.end_block, self.request_id
        )
    }
}

/// Request format for the proposer `proofs_getMockProof`
#[derive(Debug)]
pub struct MockProofProposerRequest {
    /// The ID of the mock proof to retrieve
    pub proof_id: MockProofId,
}

/// Response for the proposer `proofs_getMockProof` call
#[derive(Debug)]
pub struct MockProofProposerResponse {
    /// Generated aggregated mock proof
    pub proof: Vec<u8>,
}

pub struct ProposerRpcClient {
    client: ProofsClient<tonic::transport::Channel>,
}

impl ProposerRpcClient {
    pub async fn new(rpc_endpoint: GrpcUri, timeout: Duration) -> Result<Self, Error> {
        // TODO: Configure various other limits besides timeout on the channel.
        let channel = tonic::transport::Channel::builder(rpc_endpoint)
            .timeout(timeout)
            .connect()
            .await
            .inspect_err(|e| error!("Error connecting to proposer gRPC: {e}"))
            .map_err(Error::Connect)?;

        let client = ProofsClient::new(channel);
        Ok(ProposerRpcClient { client })
    }
}

#[tonic::async_trait]
impl AggregationProofProposer for ProposerRpcClient {
    async fn request_agg_proof(
        &self,
        request: AggregationProofProposerRequest,
    ) -> Result<AggregationProofProposerResponse, Error> {
        let request = grpc::AggProofRequest::from(request);

        let mut client = self.client.clone();
        let response: AggregationProofProposerResponse = client
            .request_agg_proof(request)
            .await
            .map_err(ProofRequestError::Grpc)
            .and_then(|resp| resp.into_inner().try_into())
            .inspect_err(|e| error!("Aggregation proof request failed: {e:?}"))
            .map_err(|e| Error::Requesting(Box::new(e)))?;

        info!(
            request_id = response.to_string(),
            "agg proof request submitted"
        );

        Ok(response)
    }

    async fn get_mock_proof(
        &self,
        request: MockProofProposerRequest,
    ) -> Result<MockProofProposerResponse, Error> {
        let request = grpc::GetMockProofRequest::from(request);

        let mut client = self.client.clone();
        let response: MockProofProposerResponse = client
            .get_mock_proof(request)
            .await
            .map_err(ProofRequestError::Grpc)
            .and_then(|resp| resp.into_inner().try_into())
            .inspect_err(|e| error!("Get mock proof request failed: {e:?}"))
            .map_err(|e| Error::Requesting(Box::new(e)))?;

        info!(proof_id = request.proof_id, "mock proof request fullfilled");

        Ok(response)
    }
}
