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
///
/// Both values are hex strings handled by their types' existing serde; the
/// aggregation vkey bytes are turned into a real `SP1VerifyingKey` with the
/// same bincode codec the prover uses elsewhere. Produce them with the
/// `op-succinct-vkey` CLI subcommand.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct OpSuccinctVkeyConfig {
    /// Bincode-serialized aggregation `SP1VerifyingKey`, hex-encoded (`0x`
    /// prefix optional).
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
    use proposer_elfs::{Sp1VKeyHash as _, VKeyHash};

    use super::*;

    #[test]
    fn op_succinct_aggregation_vkey_round_trips_through_config() {
        // `Bytes` carries the hex value through serde, and the existing bincode
        // codec turns it back into the real verifying key. Use a real serialized
        // vkey: the one embedded from op-succinct-elfs.
        let encoded = alloy_primitives::hex::encode(proposer_elfs::aggregation::VKEY.as_bytes());
        let config: OpSuccinctVkeyConfig =
            serde_json::from_str(&format!(r#"{{ "aggregation-vkey": "0x{encoded}" }}"#))
                .expect("parsing aggregation vkey");

        let bytes = config.aggregation_vkey.expect("aggregation vkey present");
        let vkey = proposer_elfs::decode_verifying_key(bytes.as_ref()).expect("decodes");
        assert_eq!(
            VKeyHash::from_vkey(&vkey),
            VKeyHash::from_vkey(proposer_elfs::aggregation::VKEY.vkey()),
        );
    }

    #[test]
    fn op_succinct_overrides_default_to_none() {
        let config: OpSuccinctVkeyConfig =
            serde_json::from_str("{}").expect("parsing empty overrides");
        assert!(config.is_empty());
    }
}
