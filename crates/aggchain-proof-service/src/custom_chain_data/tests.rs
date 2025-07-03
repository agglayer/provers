use std::{fs::File, path::PathBuf};

use agglayer_interop::types::Digest;
use serde::{Deserialize, Deserializer};

use super::*;

// Constants for default programs for ECDSA
const ECDSA_DEFAULT: u16 = 0x00;
// Constants for default programs for FEP
const FEP_DEFAULT: u16 = 0x00;
// Constant for custom FEP program
const CUSTOM_FEP_PROGRAM: u16 = 0x01;
// Constant for custom FEP program
const CUSTOM_FEP_PROGRAM2: u16 = 0x02;

// Aggchain type for
const AGGCHAIN_TYPE_ECDSA: u16 = 0x00;

const AGGCHAIN_TYPE_FEP: u16 = 0x01;

#[test]
fn aggchain_pattern() {
    // aggchain is using aggchain-type 0 and use the default ECDSA program
    assert_eq!(
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        (((ECDSA_DEFAULT as u32) << 16) | AGGCHAIN_TYPE_ECDSA as u32)
    );
    // aggchain is using aggchain-type 0 and use the default FEP program -> Should
    // fail
    assert_eq!(
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        (((FEP_DEFAULT as u32) << 16) | AGGCHAIN_TYPE_ECDSA as u32)
    );
    // aggchain is using aggchain-type 1 and use the default FEP program
    assert_eq!(
        0b0000_0000_0000_0000_0000_0000_0000_0001,
        (((FEP_DEFAULT as u32) << 16) | AGGCHAIN_TYPE_FEP as u32)
    );
    // aggchain is using aggchain-type 1 and use the default ECDSA program ->
    // Allowed as we'll support ECDSA for type 1
    assert_eq!(
        0b0000_0000_0000_0000_0000_0000_0000_0001,
        (((ECDSA_DEFAULT as u32) << 16) | AGGCHAIN_TYPE_FEP as u32)
    );
    // aggchain is using aggchain-type 1 and use its own FEP program
    assert_eq!(
        0b0000_0000_0000_0001_0000_0000_0000_0001,
        (((CUSTOM_FEP_PROGRAM as u32) << 16) | AGGCHAIN_TYPE_FEP as u32)
    );
    // aggchain is using aggchain-type 1 and use its own FEP program
    assert_eq!(
        0b0000_0000_0000_0010_0000_0000_0000_0001,
        (((CUSTOM_FEP_PROGRAM2 as u32) << 16) | AGGCHAIN_TYPE_FEP as u32)
    );
}

#[tokio::test]
async fn test_custom_chain_data_builder_service() {
    let response = compute_custom_chain_data(ClaimRoot(Digest([1u8; 32])), 10u64);

    let mut expected = [0u8; 96];
    // program selector
    expected[0..4].copy_from_slice(&[0, 5, 0, 1]);

    // output root
    expected[32..64].copy_from_slice(&[1u8; 32]);

    // l2 block number
    expected[64..96].copy_from_slice(&U256::from(10u64).to_be_bytes::<32>());

    assert_eq!(response, expected.to_vec());
}

#[derive(Debug, Deserialize)]
struct TestVectorEntry {
    input: TestVectorEntryInput,
    output: TestVectorEntryOutput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestVectorEntryInput {
    #[serde(deserialize_with = "hex_to_u16")]
    aggchain_type: u16,
    #[serde(deserialize_with = "hex_to_u16")]
    aggchain_v_key_selector: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestVectorEntryOutput {
    #[serde(deserialize_with = "hex_to_u32")]
    final_aggchain_v_key_selector: u32,
}

fn hex_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    u32::from_str_radix(s.trim_start_matches("0x"), 16).map_err(serde::de::Error::custom)
}
fn hex_to_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    u16::from_str_radix(s.trim_start_matches("0x"), 16).map_err(serde::de::Error::custom)
}

#[test]
fn test_aggchain_selector() {
    let path: PathBuf = env!("CARGO_MANIFEST_DIR").parse().unwrap();

    let test_vectors = path.join("test-vectors/aggchain-selector.json");
    let file = File::open(test_vectors).expect("Failed to open file");

    let data: Vec<TestVectorEntry> = serde_json::from_reader(file).expect("Failed to parse JSON");

    for TestVectorEntry { input, output } in data {
        assert_eq!(
            VKeySelector::new(input.aggchain_v_key_selector, input.aggchain_type).to_be_bytes(),
            output.final_aggchain_v_key_selector.to_be_bytes()
        );
    }
}
