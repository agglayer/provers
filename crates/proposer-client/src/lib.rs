use std::fmt::Display;

use alloy_primitives::B256;
use serde::{Deserialize, Serialize};
use sp1_sdk::{SP1ProofWithPublicValues, SP1VerifyingKey};

pub use crate::error::Error;

pub mod aggregation_prover;
pub mod client;
pub mod config;
pub mod database_prover;
pub mod error;

#[async_trait::async_trait]
#[cfg_attr(feature = "testutils", mockall::automock)]
pub trait ProposerClient {
    async fn wait_for_proof(
        &self,
        request_id: RequestId,
    ) -> Result<SP1ProofWithPublicValues, Error>;

    #[allow(clippy::result_large_err)]
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

impl TryFrom<&[u8]> for RequestId {
    type Error = core::array::TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}
