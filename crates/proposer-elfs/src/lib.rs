pub use aggkit_prover_types::{
    vkey::LazyVerifyingKey,
    vkey_hash::{HashU32, VKeyHash, Sp1VKeyHash},
};

mod vkeys_raw {
    include!(concat!(env!("OUT_DIR"), "/vkeys_raw.rs"));
}

pub mod aggregation {
    pub use op_succinct_elfs::AGGREGATION_ELF as ELF;

    use crate::{vkeys_raw, HashU32, LazyVerifyingKey, VKeyHash};

    pub static VKEY: LazyVerifyingKey =
        LazyVerifyingKey::from_unparsed_bytes(vkeys_raw::aggregation::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = vkeys_raw::aggregation::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);
}

pub mod range {
    pub use op_succinct_elfs::RANGE_ELF_EMBEDDED as ELF;
    pub use vkeys_raw::range::VKEY_COMMITMENT;

    use crate::{vkeys_raw, HashU32, LazyVerifyingKey, VKeyHash};

    pub static VKEY: LazyVerifyingKey =
        LazyVerifyingKey::from_unparsed_bytes(vkeys_raw::range::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = vkeys_raw::range::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);
}

#[cfg(test)]
mod test;
