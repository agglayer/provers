use aggchain_proof_builder::config::AggchainProofBuilderConfig;
use proposer_service::config::ProposerServiceConfig;
use serde::{Deserialize, Serialize};

/// The Aggchain proof service configuration
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AggchainProofServiceConfig {
    pub aggchain_proof_builder: AggchainProofBuilderConfig,
    pub proposer_service: ProposerServiceConfig,

    /// Ignored: the op-succinct verification keys are read from the
    /// op-succinct config selected on L1 and from the SP1 network. Still
    /// accepted so existing configurations load.
    #[serde(default, skip_serializing_if = "OpSuccinctVkeyConfig::is_empty")]
    pub op_succinct: OpSuccinctVkeyConfig,
}

/// Former overrides of the op-succinct verification key material.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct OpSuccinctVkeyConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggregation_vkey: Option<alloy_primitives::Bytes>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_vkey_commitment: Option<agglayer_interop::types::Digest>,
}

impl OpSuccinctVkeyConfig {
    /// Returns `true` when neither field is set, so the section can be omitted
    /// from serialized configuration.
    pub(crate) fn is_empty(&self) -> bool {
        self.aggregation_vkey.is_none() && self.range_vkey_commitment.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_op_succinct_section_still_parses() {
        let config: OpSuccinctVkeyConfig = serde_json::from_str(
            r#"{
                "aggregation-vkey": "0x1234",
                "range-vkey-commitment": "0x0036090447cf2995a00135ab08c1edf428f3084360cbbe447d96132361de246f"
            }"#,
        )
        .expect("parsing the ignored op-succinct section");
        assert!(!config.is_empty());
    }

    #[test]
    fn op_succinct_overrides_default_to_none() {
        let config: OpSuccinctVkeyConfig =
            serde_json::from_str("{}").expect("parsing empty overrides");
        assert!(config.is_empty());
    }
}
