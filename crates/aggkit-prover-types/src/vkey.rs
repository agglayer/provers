use std::{
    fmt::{LowerHex, UpperHex},
    sync::OnceLock,
};

pub use sp1_sdk::SP1VerifyingKey;

pub struct LazyVerifyingKey {
    bytes: &'static [u8],
    vkey: OnceLock<SP1VerifyingKey>,
}

impl LazyVerifyingKey {
    /// New verifying key loaded from given byte string.
    ///
    /// If the byte string is malformed, the methods will panic.
    /// Use with tested static data only.
    pub const fn from_unparsed_bytes(bytes: &'static [u8]) -> Self {
        let vkey = OnceLock::new();
        Self { bytes, vkey }
    }

    /// Get the associated vkey.
    pub fn vkey(&self) -> &SP1VerifyingKey {
        self.vkey.get_or_init(|| {
            prover_elf_utils::elf_info::bincode_codec()
                .deserialize(self.bytes)
                .expect("vkey not encoded correctly")
        })
    }

    /// Get the bincode-encoded representation of the vkey.
    pub fn as_bytes(&self) -> alloy_primitives::Bytes {
        alloy_primitives::Bytes::from_static(self.bytes)
    }
}

impl AsRef<SP1VerifyingKey> for LazyVerifyingKey {
    fn as_ref(&self) -> &SP1VerifyingKey {
        self.vkey()
    }
}

impl std::ops::Deref for LazyVerifyingKey {
    type Target = SP1VerifyingKey;

    fn deref(&self) -> &Self::Target {
        self.vkey()
    }
}

impl LowerHex for LazyVerifyingKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.as_bytes(), f)
    }
}

impl UpperHex for LazyVerifyingKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.as_bytes(), f)
    }
}

/// Error returned when a configured verifying key cannot be decoded.
#[derive(Debug, thiserror::Error)]
#[error("failed to decode SP1 verifying key from the configured bytes: {0}")]
pub struct VKeyDecodeError(String);

/// Error returned when a verifying key cannot be encoded.
#[derive(Debug, thiserror::Error)]
#[error("failed to encode SP1 verifying key: {0}")]
pub struct VKeyEncodeError(String);

/// Decode a bincode-encoded [`SP1VerifyingKey`], as produced by the build-time
/// `prover_elf_utils::ElfInfo::emit_vkey_bytes` /
/// [`LazyVerifyingKey::as_bytes`].
///
/// This must use the exact same codec as [`LazyVerifyingKey::vkey`] so that a
/// configured override and the embedded fallback decode identically.
pub fn decode_verifying_key(bytes: &[u8]) -> Result<SP1VerifyingKey, VKeyDecodeError> {
    prover_elf_utils::elf_info::bincode_codec()
        .deserialize(bytes)
        .map_err(|error| VKeyDecodeError(error.to_string()))
}

/// Encode an [`SP1VerifyingKey`] into the bincode representation accepted by
/// [`decode_verifying_key`]. Uses the same codec as [`decode_verifying_key`],
/// so the two are guaranteed to round-trip.
pub fn encode_verifying_key(vkey: &SP1VerifyingKey) -> Result<Vec<u8>, VKeyEncodeError> {
    prover_elf_utils::elf_info::bincode_codec()
        .serialize(vkey)
        .map_err(|error| VKeyEncodeError(error.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hexdump() {
        let vk = LazyVerifyingKey::from_unparsed_bytes(&[0xab, 0xcd]);
        assert_eq!(format!("{vk:x}"), "0xabcd");
        assert_eq!(format!("{vk:X}"), "0xABCD");
    }
}
