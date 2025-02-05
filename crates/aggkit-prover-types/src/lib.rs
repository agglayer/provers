use aggchain_proof_core::proof::AggchainProofPublicValues;
use bincode::{
    config::{BigEndian, FixintEncoding, WithOtherEndian, WithOtherIntEncoding},
    DefaultOptions, Options,
};
use serde::{Deserialize, Serialize};

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

/// Agghchain proof is generated from FEP proof and additional
/// bridge inputs
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct AggchainProof {
    proof: AggchainProofPublicValues,
    //TODO add all necessary fields
}
