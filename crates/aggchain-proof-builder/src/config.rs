use std::time::Duration;

use aggchain_proof_contracts::config::AggchainProofContractsConfig;
use prover_config::ProverType;
use serde::{Deserialize, Serialize};

/// What the prover does with a request. Orthogonal to the prover type, which
/// decides how the resulting program is proven (`network-prover` /
/// `cpu-prover` give a real SP1 proof, `mock-prover` an SP1 mock proof that
/// only a mock verifier accepts).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AggchainProofMode {
    /// Proves the standard program, which verifies the FEP (or the optimistic
    /// signature) and the bridge constraints.
    #[default]
    Standard,

    /// After an L2 reorg past a settled output: proves the noop program over
    /// public values built from L1, with the pre-root of the latest L1 output.
    /// op-succinct-proposer is not used.
    Recovery,
}

/// The Aggchain proof builder configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofBuilderConfig {
    /// ID of the network for which the proof is generated (rollup id).
    pub network_id: u32,

    /// See [`AggchainProofMode`].
    #[serde(default)]
    pub mode: AggchainProofMode,

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
            mode: AggchainProofMode::default(),
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
