mod bridge;
pub mod error;
mod full_execution_proof;
mod keccak;
mod local_exit_tree;
pub mod proof;
pub mod vkey_hash;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub use keccak::digest::Digest;

pub const AGGCHAIN_TYPE: u16 = 0x0001;
