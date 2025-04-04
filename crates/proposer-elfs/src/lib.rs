pub use aggkit_prover_types::{
    vkey::LazyVerifyingKey,
    vkey_hash::{HashU32, VKeyHash},
};

pub mod aggregation {
    pub use op_succinct_elfs::AGG_ELF as ELF;

    use crate::{HashU32, LazyVerifyingKey, VKeyHash};

    pub const VKEY: LazyVerifyingKey =
        LazyVerifyingKey::new_unchecked(proposer_vkeys_raw::aggregation::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = proposer_vkeys_raw::aggregation::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);
}

pub mod range {
    pub use op_succinct_elfs::RANGE_ELF as ELF;

    use crate::{HashU32, LazyVerifyingKey, VKeyHash};

    pub const VKEY: LazyVerifyingKey =
        LazyVerifyingKey::new_unchecked(proposer_vkeys_raw::range::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = proposer_vkeys_raw::range::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);
}
