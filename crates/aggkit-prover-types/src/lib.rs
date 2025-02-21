use bincode::{
    config::{BigEndian, FixintEncoding, WithOtherEndian, WithOtherIntEncoding},
    DefaultOptions, Options,
};

pub mod compat;

#[allow(clippy::needless_lifetimes)]
mod generated;
pub use generated::aggkit::prover::v1;
pub use generated::agglayer;

pub fn default_bincode_options(
) -> WithOtherIntEncoding<WithOtherEndian<DefaultOptions, BigEndian>, FixintEncoding> {
    DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

pub type Hash = [u8; 32];
