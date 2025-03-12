mod bridge;
pub mod error;
mod full_execution_proof;
mod keccak;
mod local_exit_tree;
pub mod proof;
mod version;
pub mod vkey_hash;

pub use keccak::digest::Digest;
pub use version::AGGCHAIN_PROOF_PROGRAM_VERSION;

pub const AGGCHAIN_TYPE: u16 = 0x0001;
