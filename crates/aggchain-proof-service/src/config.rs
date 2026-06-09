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

    /// Optional overrides for the op-succinct verification key material.
    ///
    /// When unset, the values embedded from `op-succinct-elfs` at build time
    /// are used. Supplying them here lets an op-succinct upgrade be rolled out
    /// without rebuilding the aggkit-prover image.
    #[serde(default, skip_serializing_if = "OpSuccinctVkeyConfig::is_empty")]
    pub op_succinct: OpSuccinctVkeyConfig,
}

/// Optional overrides of the op-succinct verification key material derived from
/// a given op-succinct release. Each field independently falls back to the
/// value embedded from `op-succinct-elfs` when absent.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct OpSuccinctVkeyConfig {
    /// Bincode-serialized aggregation `SP1VerifyingKey`, hex-encoded (`0x`
    /// prefix optional). Must be produced with the same codec the prover uses
    /// to decode it (see
    /// `aggkit_prover_types::vkey::decode_verifying_key`); the
    /// `op-succinct-vkey` CLI subcommand emits a value in this format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggregation_vkey: Option<alloy_primitives::Bytes>,

    /// Range vkey commitment, hex-encoded 32 bytes (`0x` prefix optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_vkey_commitment: Option<agglayer_interop::types::Digest>,
}

impl OpSuccinctVkeyConfig {
    /// Returns `true` when no override is set, so the section can be omitted
    /// from serialized configuration.
    fn is_empty(&self) -> bool {
        self.aggregation_vkey.is_none() && self.range_vkey_commitment.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_succinct_overrides_parse_from_hex() {
        let config: OpSuccinctVkeyConfig = serde_json::from_str(
            r#"{
                "aggregation-vkey": "0xdeadbeef",
                "range-vkey-commitment":
                    "0x0000000000000000000000000000000000000000000000000000000000000001"
            }"#,
        )
        .expect("parsing op-succinct overrides");

        assert_eq!(
            config.aggregation_vkey,
            Some(alloy_primitives::Bytes::from_static(&[
                0xde, 0xad, 0xbe, 0xef
            ]))
        );
        assert_eq!(config.range_vkey_commitment.expect("commitment").0[31], 1);
    }

    #[test]
    fn op_succinct_overrides_default_to_none() {
        let config: OpSuccinctVkeyConfig =
            serde_json::from_str("{}").expect("parsing empty overrides");
        assert!(config.is_empty());
    }
}
