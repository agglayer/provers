use alloy_primitives::B256;
use sp1_sdk::HashableKey;

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

#[rstest::rstest]
#[case::agg("aggregation", &aggregation::VKEY)]
#[case::range("range", &range::VKEY)]
fn snap_vkey_hash(#[case] name: &'static str, #[case] vkey: &LazyVerifyingKey) {
    let hash_u32 = vkey.hash_u32();
    let hash_bytes32 = B256::new(vkey.bytes32_raw());
    let hash_bytes = B256::new(vkey.hash_bytes());

    let snap = format!(
        "{name} vkey\n\
        hash_u32 {hash_u32:?}\n\
        bytes32  {hash_bytes32}\n\
        bytes    {hash_bytes}\n"
    );

    insta::assert_snapshot!(format!("{name}_vkey"), snap);
}
