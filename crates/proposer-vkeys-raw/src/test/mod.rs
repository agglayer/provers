use alloy_primitives::{b256, B256};

#[test]
fn check_expected_values() {
    let agg_vkey_hash: [u32; 8] = [
        1638338388, 1256426946, 526664290, 1122453194, 1403846808, 601906877, 1176579776, 312933952,
    ];

    assert_eq!(
        crate::aggregation::VKEY_HASH,
        agg_vkey_hash,
        "aggregation vkey hash mismatch"
    );

    assert_eq!(
        B256::new(crate::range::VKEY_COMMITMENT),
        b256!("0x35882a76205af8c12eaeea7551ff8dbc392dc2a95b0f7f31660a5468237d4434"),
        "range vkey commitment mismatch"
    )
}
