use std::time::Duration;

use aggchain_proof_contracts::config::AggchainProofContractsConfig;
use prover_config::ProverType;
use serde::{Deserialize, Serialize};

/// Aggchain proof program to run.
///
/// Orthogonal to the prover type: the program decides *what* is proven, the
/// prover decides *how* the proof is produced (`network-prover` / `cpu-prover`
/// give a real SP1 proof, `mock-prover` gives an SP1 mock proof that only a
/// mock verifier accepts).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AggchainProgram {
    /// The regular program: verifies the FEP (or the optimistic signature) and
    /// the bridge constraints.
    #[default]
    Standard,

    /// Recovery only: commits the public values without verifying anything,
    /// with the pre-root taken from the latest L1 output so that the aggchain
    /// params match the ones the contract computes after an L2 reorg. Accepted
    /// by the aggchain contract only under selector `0xFFFF0001`, with its vkey
    /// registered in the rollup's `ownedAggchainVKeys`.
    Noop,
}

/// The Aggchain proof builder configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofBuilderConfig {
    /// ID of the network for which the proof is generated (rollup id).
    pub network_id: u32,

    /// Aggchain proof program to run, see [`AggchainProgram`].
    #[serde(default)]
    pub program: AggchainProgram,

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
            program: AggchainProgram::default(),
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
