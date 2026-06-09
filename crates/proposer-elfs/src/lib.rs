pub use aggkit_prover_types::{
    vkey::{decode_verifying_key, LazyVerifyingKey, SP1VerifyingKey, VKeyDecodeError},
    vkey_hash::{HashU32, Sp1VKeyHash, VKeyHash},
};

mod vkeys_raw {
    include!(concat!(env!("OUT_DIR"), "/vkeys_raw.rs"));
}

pub mod aggregation {
    use std::sync::OnceLock;

    pub use op_succinct_elfs::AGGREGATION_ELF as ELF;

    use crate::{vkeys_raw, HashU32, LazyVerifyingKey, SP1VerifyingKey, VKeyHash};

    pub static VKEY: LazyVerifyingKey =
        LazyVerifyingKey::from_unparsed_bytes(vkeys_raw::aggregation::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = vkeys_raw::aggregation::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);

    /// Configured override, installed once at startup via
    /// [`crate::install_overrides`].
    pub(crate) static VKEY_OVERRIDE: OnceLock<SP1VerifyingKey> = OnceLock::new();

    /// The aggregation verifying key in effect: the configured override when
    /// set, otherwise the key embedded from op-succinct-elfs at build time.
    pub fn vkey() -> &'static SP1VerifyingKey {
        VKEY_OVERRIDE.get().unwrap_or_else(|| VKEY.vkey())
    }
}

pub mod range {
    use std::sync::OnceLock;

    pub use op_succinct_elfs::RANGE_ELF_EMBEDDED as ELF;
    pub use vkeys_raw::range::VKEY_COMMITMENT;

    use crate::{vkeys_raw, HashU32, LazyVerifyingKey, VKeyHash};

    pub static VKEY: LazyVerifyingKey =
        LazyVerifyingKey::from_unparsed_bytes(vkeys_raw::range::VKEY_BYTES);

    pub const VKEY_HASH_U32: HashU32 = vkeys_raw::range::VKEY_HASH;

    pub const VKEY_HASH: VKeyHash = VKeyHash::from_hash_u32(VKEY_HASH_U32);

    /// Configured override, installed once at startup via
    /// [`crate::install_overrides`].
    pub(crate) static VKEY_COMMITMENT_OVERRIDE: OnceLock<[u8; 32]> = OnceLock::new();

    /// The range vkey commitment in effect: the configured override when set,
    /// otherwise the value embedded from op-succinct-elfs at build time.
    pub fn commitment() -> [u8; 32] {
        VKEY_COMMITMENT_OVERRIDE
            .get()
            .copied()
            .unwrap_or(VKEY_COMMITMENT)
    }
}

/// Install optional op-succinct vkey overrides parsed from configuration.
///
/// The values are process-global and intended to be installed once at startup,
/// before the proof services are constructed; a repeated install keeps the
/// first value. When an override is absent, the value embedded from
/// `op-succinct-elfs` at build time is used instead. See [`aggregation::vkey`]
/// and [`range::commitment`] for the resolved accessors the services read.
pub fn install_overrides(
    aggregation_vkey: Option<&[u8]>,
    range_vkey_commitment: Option<[u8; 32]>,
) -> Result<(), VKeyDecodeError> {
    // An `Err` from `set` means the value was already installed, which is
    // expected on repeated startup-time calls (e.g. multiple service
    // constructions in tests); the first installed value is kept.
    if let Some(bytes) = aggregation_vkey {
        let _ = aggregation::VKEY_OVERRIDE.set(decode_verifying_key(bytes)?);
    }
    if let Some(commitment) = range_vkey_commitment {
        let _ = range::VKEY_COMMITMENT_OVERRIDE.set(commitment);
    }
    Ok(())
}

#[cfg(test)]
mod test;
