use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::aggchain_proof_builder::AggchainProofBuilderConfig;
use crate::proposer_service::ProposerServiceConfig;

pub const HTTP_RPC_NODE_INITIAL_BACKOFF_MS: u64 = 5000;

pub const HTTP_RPC_NODE_BACKOFF_MAX_RETRIES: u32 = 64;

/// The Aggchain proof service configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofServiceConfig {
    pub aggchain_proof_builder: AggchainProofBuilderConfig,
    pub proposer_service: ProposerServiceConfig,
}

impl AggchainProofServiceConfig {
    pub fn default_for_test() -> Self {
        AggchainProofServiceConfig {
            aggchain_proof_builder: AggchainProofBuilderConfig::default_for_test(),
            proposer_service: ProposerServiceConfig::default_for_test(),
        }
    }
}
