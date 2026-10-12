use alloy_primitives::B256;
use sp1_sdk::{blocking::Prover as _, HashableKey, ProvingKey as _};

use super::*;

#[rstest::rstest]
#[case::agg(aggregation::ELF, &aggregation::VKEY, aggregation::VKEY_HASH)]
#[case::range(range::ELF, &range::VKEY, range::VKEY_HASH)]
fn consistency(#[case] elf: &[u8], #[case] vkey: &LazyVerifyingKey, #[case] vkey_hash: VKeyHash) {
    let prover = sp1_sdk::blocking::MockProver::new();
    let proving_key = prover.setup(elf.into()).expect("setting up proving key");
    let computed_vkey = proving_key.verifying_key().clone();

    assert_eq!(VKeyHash::from_vkey(&computed_vkey), vkey_hash);
    assert_eq!(VKeyHash::from_vkey(vkey.vkey()), vkey_hash);
}

#[test]
fn range_commitment_consistency() {
    assert_eq!(range::VKEY.hash_bytes(), range::VKEY_COMMITMENT);
}

#[rstest::rstest]
#[case::agg("aggregation", &aggregation::VKEY)]
#[case::range("range", &range::VKEY)]
fn snap_vkey_hash(#[case] name: &'static str, #[case] vkey: &LazyVerifyingKey) {
    let hu32 = vkey.hash_u32();
    let bytes32 = B256::new(vkey.bytes32_raw());
    let bytes = B256::new(vkey.hash_bytes());

    let snap = format!("{name} vkey\nhash_u32 {hu32:?}\nbytes32  {bytes32}\nbytes    {bytes}\n");

    insta::assert_snapshot!(format!("{name}_vkey"), snap);
}

/// The config-override codec must round-trip with the embedded representation:
/// the bytes emitted by `encode_verifying_key` (used by the `op-succinct-vkey`
/// CLI) must decode back into the same key the runtime falls back to.
#[rstest::rstest]
#[case::agg(&aggregation::VKEY)]
#[case::range(&range::VKEY)]
fn vkey_codec_round_trip(#[case] vkey: &LazyVerifyingKey) {
    use aggkit_prover_types::vkey::{decode_verifying_key, encode_verifying_key};

    let decoded = decode_verifying_key(&vkey.as_bytes()).expect("decoding embedded vkey");
    assert_eq!(
        VKeyHash::from_vkey(&decoded),
        VKeyHash::from_vkey(vkey.vkey())
    );

    let encoded = encode_verifying_key(&decoded).expect("encoding vkey");
    assert_eq!(encoded.as_slice(), vkey.as_bytes().as_ref());
}

/// Decoding incomplete vkey bytes must fail rather than silently producing a
/// bogus key, so a malformed config override is rejected at startup.
#[test]
fn decode_rejects_truncated_vkey() {
    let bytes = aggregation::VKEY.as_bytes();
    let full: &[u8] = bytes.as_ref();
    let truncated = &full[..full.len() / 2];

    assert!(aggkit_prover_types::vkey::decode_verifying_key(truncated).is_err());
}
