use serde::{Deserialize, Serialize};

use crate::config::ProposerClientConfig;

mod config;
mod error;
mod rpc;

pub struct ProposerClient {
    _client: String,
}

impl ProposerClient {
    pub fn new(_config: ProposerClientConfig) -> Result<Self, error::Error> {
        Ok(ProposerClient {
            _client: String::default(),
        })
    }

    // pub async fn get_proofs(&mut self, request: Request) -> Result<Response,
    // error::Error> {     match self.client.
    // get_proofs(GetProofsRequest::from(request)).await {         Ok(response)
    // => Ok(response.into_inner().into()),         Err(status) =>
    // Err(status.into()),     }
    // }
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
