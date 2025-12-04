use std::time::Duration;

use aggchain_proof_contracts::config::AggchainProofContractsConfig;
use prover_config::ProverType;
use serde::{Deserialize, Serialize};

/// The Aggchain proof builder configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofBuilderConfig {
    /// ID of the network for which the proof is generated (rollup id).
    pub network_id: u32,

    /// Aggchain prover configuration
    pub primary_prover: ProverType,

    /// Fallback prover configuration
    pub fallback_prover: Option<ProverType>,

    /// Aggchain proof generation timeout in seconds.
    #[serde(default = "default_aggchain_prover_timeout")]
    #[serde(with = "prover_utils::with::HumanDuration")]
    pub proving_timeout: Duration,

    /// Contract configuration
    #[serde(default)]
    pub contracts: AggchainProofContractsConfig,
}

impl Default for AggchainProofBuilderConfig {
    fn default() -> Self {
        AggchainProofBuilderConfig {
            network_id: 0,
            proving_timeout: default_aggchain_prover_timeout(),
            primary_prover: ProverType::NetworkProver(prover_config::NetworkProverConfig::default()),
            fallback_prover: None,
            contracts: AggchainProofContractsConfig::default(),
        }
    }
}

fn default_aggchain_prover_timeout() -> Duration {
    Duration::from_secs(3600)
}
