use std::fmt::Debug;

use aggchain_proof_builder::config::AggchainProofBuilderConfig;
use proposer_service::config::ProposerServiceConfig;
use serde::{Deserialize, Serialize};

/// The Aggchain proof service configuration
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofServiceConfig {
    pub aggchain_proof_builder: AggchainProofBuilderConfig,
    pub proposer_service: ProposerServiceConfig,
}
