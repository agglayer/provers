use serde::{Deserialize, Serialize};

use crate::config::ProposerClientConfig;
use crate::proposer_v1::GetProofsRequest;

mod error;
mod rpc;

#[path = "generated/aggkit.proposer.v1.rs"]
#[rustfmt::skip]
#[allow(warnings)]
pub mod proposer_v1;
mod config;

pub struct ProposerClient {
    grpc: rpc::ProposerGrpcClient,
}

impl ProposerClient {
    pub fn new(config: ProposerClientConfig) -> Result<Self, error::Error> {
        Ok(ProposerClient {
            grpc: rpc::connect_proposer_service(&config)?,
        })
    }

    pub async fn get_proofs(
        &mut self,
        request: Request,
    ) -> Result<Response, error::Error> {
        match self.grpc.get_proofs(GetProofsRequest::from(request)).await {
            Ok(response) => Ok(response.into_inner().into()),
            Err(status) => Err(status.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    start_block: u64,
    end_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    proofs: Vec<u8>,
}
