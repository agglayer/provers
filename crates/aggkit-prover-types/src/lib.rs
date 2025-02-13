use bincode::{
    config::{BigEndian, FixintEncoding, WithOtherEndian, WithOtherIntEncoding},
    DefaultOptions, Options,
};

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/aggkit.prover.bin");

#[path = "generated/aggkit.prover.v1.rs"]
#[rustfmt::skip]
#[allow(warnings)]
pub mod v1;

pub fn default_bincode_options(
) -> WithOtherIntEncoding<WithOtherEndian<DefaultOptions, BigEndian>, FixintEncoding> {
    DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

pub type Hash = [u8; 32];
