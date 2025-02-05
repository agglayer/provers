use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::aggchain_proof_builder::AggchainProofBuilderConfig;
use crate::proposer_service::ProposerServiceConfig;

/// The initial blockchain node backoff in milliseconds
pub const HTTP_RPC_NODE_INITIAL_BACKOFF_MS: u64 = 5000;

/// The blockchain node backoff number of retries
pub const HTTP_RPC_NODE_BACKOFF_MAX_RETRIES: u32 = 64;

/// The Aggchain proof service configuration
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofServiceConfig {
    pub aggchain_proof_builder_config: AggchainProofBuilderConfig,
    pub proposer_service_config: ProposerServiceConfig,
}
