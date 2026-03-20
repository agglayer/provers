use alloy_primitives::B256;
use sp1_sdk::{blocking::Prover as _, HashableKey, ProvingKey as _};

use super::*;

#[rstest::rstest]
#[case::agg(aggregation::ELF, &aggregation::VKEY, aggregation::VKEY_HASH)]
#[case::range(range::ELF, &range::VKEY, range::VKEY_HASH)]
fn consistency(#[case] elf: &[u8], #[case] vkey: &LazyVerifyingKey, #[case] vkey_hash: VKeyHash) {
    let prover = sp1_sdk::blocking::CpuProver::new();
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
