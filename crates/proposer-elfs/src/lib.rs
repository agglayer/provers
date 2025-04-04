pub use aggkit_prover_types::{
    vkey::LazyVerifyingKey,
    vkey_hash::{HashU32, VKeyHash},
};

pub mod aggregation {
    pub use op_succinct_elfs::AGG_ELF as ELF;

    use crate::{HashU32, LazyVerifyingKey, VKeyHash};

    pub static VKEY: LazyVerifyingKey =
        LazyVerifyingKey::new_unchecked(proposer_vkeys_raw::aggregation::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = proposer_vkeys_raw::aggregation::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);
}

pub mod range {
    pub use op_succinct_elfs::RANGE_ELF as ELF;

    use crate::{HashU32, LazyVerifyingKey, VKeyHash};

    pub static VKEY: LazyVerifyingKey =
        LazyVerifyingKey::new_unchecked(proposer_vkeys_raw::range::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = proposer_vkeys_raw::range::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);
}

#[cfg(test)]
mod test {
    use super::*;

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
}
