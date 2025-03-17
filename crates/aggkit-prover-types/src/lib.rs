use bincode::{
    config::{BigEndian, FixintEncoding, WithOtherEndian, WithOtherIntEncoding},
    DefaultOptions, Options,
};

#[path = "generated/aggkit.prover.v1.rs"]
#[rustfmt::skip]
#[allow(warnings)]
pub mod v1;
pub mod conversion;
pub mod error;
pub mod vkey_hash;

pub fn default_bincode_options(
) -> WithOtherIntEncoding<WithOtherEndian<DefaultOptions, BigEndian>, FixintEncoding> {
    DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

pub use agglayer_interop::types::Digest;
