use alloy_primitives::B256;

#[test]
fn aggregation_vkey_hash_snapshot() {
    insta::assert_debug_snapshot!(crate::aggregation::VKEY_HASH);
}

#[test]
fn range_vkey_commitment_snapshot() {
    insta::assert_debug_snapshot!(B256::new(crate::range::VKEY_COMMITMENT));
}
