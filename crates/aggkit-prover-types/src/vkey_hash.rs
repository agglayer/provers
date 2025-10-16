pub use aggchain_proof_types::HashU32;
pub use agglayer_primitives::vkey_hash::VKeyHash;

pub trait Sp1VKeyHash {
    fn from_vkey<K: sp1_sdk::HashableKey>(vkey: &K) -> Self;
}

impl Sp1VKeyHash for VKeyHash {
    fn from_vkey<K: sp1_sdk::HashableKey>(vkey: &K) -> Self {
        Self::from_hash_u32(vkey.hash_u32())
    }
}
