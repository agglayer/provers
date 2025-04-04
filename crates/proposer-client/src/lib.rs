use std::{fmt::Display, str::FromStr as _};

use alloy_primitives::B256;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use sp1_sdk::{SP1ProofWithPublicValues, SP1VerifyingKey};

pub use crate::error::Error;
use crate::rpc::{AggregationProofProposerRequest, AggregationProofProposerResponse};

pub mod aggregation_prover;
pub mod client;
pub mod config;
pub mod error;
pub mod mock_prover;
pub mod network_prover;
pub mod rpc;

#[cfg(test)]
mod tests;

#[async_trait::async_trait]
#[cfg_attr(feature = "testutils", mockall::automock)]
pub trait ProposerClient {
    async fn request_agg_proof(
        &self,
        request: AggregationProofProposerRequest,
    ) -> Result<AggregationProofProposerResponse, Error>;

    async fn wait_for_proof(
        &self,
        request_id: RequestId,
    ) -> Result<SP1ProofWithPublicValues, Error>;

    fn verify_agg_proof(
        &self,
        request_id: RequestId,
        proof: &SP1ProofWithPublicValues,
        vkey: &SP1VerifyingKey,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FepProposerRequest {
    pub last_proven_block: u64,
    pub requested_end_block: u64,
    pub l1_block_hash: B256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FepProposerResponse {
    pub aggregation_proof: SP1ProofWithPublicValues,
    pub last_proven_block: u64,
    pub end_block: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RequestId(pub B256);

impl Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if let Ok(id) = B256::from_str(&value) {
            Ok(RequestId(id))
        } else if let Ok(id) = u64::from_str(&value) {
            let mut encoding = [0; 32];
            encoding[..8].copy_from_slice(&id.to_be_bytes());
            Ok(RequestId(B256::from(encoding)))
        } else {
            Err(D::Error::custom(format!("invalid request id: {value:?}")))
        }
    }
}
