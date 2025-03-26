pub mod bridge;
pub mod error;
pub mod full_execution_proof;
pub mod proof;
pub mod vkey_hash;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub use agglayer_primitives::digest::Digest;

pub const AGGCHAIN_TYPE: u16 = 0x0001;
