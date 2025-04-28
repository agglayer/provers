pub use aggkit_prover_types::{
    vkey::LazyVerifyingKey,
    vkey_hash::{HashU32, VKeyHash},
};

mod vkeys_raw {
    include!(concat!(env!("OUT_DIR"), "/vkeys_raw.rs"));
}

pub mod aggregation {
    pub use op_succinct_elfs::AGG_ELF as ELF;

    use crate::{vkeys_raw, HashU32, LazyVerifyingKey, VKeyHash};

    pub static VKEY: LazyVerifyingKey =
        LazyVerifyingKey::from_unparsed_bytes(vkeys_raw::aggregation::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = vkeys_raw::aggregation::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);
}

pub mod range {
    pub use op_succinct_elfs::RANGE_ELF as ELF;

    use crate::{vkeys_raw, HashU32, LazyVerifyingKey, VKeyHash};
    pub use vkeys_raw::range::VKEY_COMMITMENT;

    pub static VKEY: LazyVerifyingKey =
        LazyVerifyingKey::from_unparsed_bytes(vkeys_raw::range::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = vkeys_raw::range::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::hex;

    #[rstest::rstest]
    #[case::agg(aggregation::ELF, &aggregation::VKEY, aggregation::VKEY_HASH)]
    #[case::range(range::ELF, &range::VKEY, range::VKEY_HASH)]
    fn consistency(
        #[case] elf: &[u8],
        #[case] vkey: &LazyVerifyingKey,
        #[case] vkey_hash: VKeyHash,
    ) {
        let prover = sp1_sdk::CpuProver::new();
        let (_proving_key, computed_vkey) = sp1_sdk::Prover::setup(&prover, elf);

        assert_eq!(VKeyHash::from_vkey(&computed_vkey), vkey_hash);
        assert_eq!(VKeyHash::from_vkey(vkey.vkey()), vkey_hash);
    }

    #[test]
    fn check_expected_raw_vkeys() {
        let agg_vkey_hash: VKeyHash = VKeyHash::from_hash_u32([
            1949122874, 766403294, 593485289, 430966933, 1657646871, 73535799, 883940176, 31174925,
        ]);
        assert_eq!(
            aggregation::VKEY_HASH,
            agg_vkey_hash,
            "aggregation vkey hash mismatch"
        );

        assert_eq!(
            range::VKEY_COMMITMENT,
            hex!("0367776036b0d8b12720eab775b651c7251e63a249cb84f63eb1c20418b24e9c"),
            "range vkey commitment mismatch"
        )
    }
}
