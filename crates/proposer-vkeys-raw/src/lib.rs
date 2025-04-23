include!(concat!(env!("OUT_DIR"), "/vkeys.rs"));

#[cfg(test)]
mod test {
    use alloy_primitives::hex;

    #[test]
    fn check_expected_values() {
        let agg_vkey_hash: [u32; 8] = [
            1949122874, 766403294, 593485289, 430966933, 1657646871, 73535799, 883940176, 31174925,
        ];
        assert_eq!(
            crate::aggregation::VKEY_HASH,
            agg_vkey_hash,
            "aggregation vkey hash mismatch"
        );

        assert_eq!(
            crate::range::VKEY_COMMITMENT,
            hex!("0367776036b0d8b12720eab775b651c7251e63a249cb84f63eb1c20418b24e9c"),
            "range vkey commitment mismatch"
        )
    }
}
