use super::*;

#[rstest::rstest]
#[case::agg(aggregation::ELF, &aggregation::VKEY, aggregation::VKEY_HASH)]
#[case::range(range::ELF, &range::VKEY, range::VKEY_HASH)]
fn consistency(#[case] elf: &[u8], #[case] vkey: &LazyVerifyingKey, #[case] vkey_hash: VKeyHash) {
    let prover = sp1_sdk::CpuProver::new();
    let (_proving_key, computed_vkey) = sp1_sdk::Prover::setup(&prover, elf);

    assert_eq!(VKeyHash::from_vkey(&computed_vkey), vkey_hash);
    assert_eq!(VKeyHash::from_vkey(vkey.vkey()), vkey_hash);
}

#[test]
fn snap_agg_vkey_hash() {
    insta::assert_debug_snapshot!(aggregation::VKEY_HASH)
}

#[test]
fn snap_range_vkey_commitment() {
    insta::assert_debug_snapshot!(range::VKEY_COMMITMENT);
}
