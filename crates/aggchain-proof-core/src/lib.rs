mod bridge;
pub mod error;
mod full_execution_proof;
pub mod keccak;
mod local_exit_tree;
pub mod proof;
pub mod vkey_hash;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub use agglayer_primitives::digest::Digest;
pub use bridge::inserted_ger::L1InfoTreeLeaf;
pub use full_execution_proof::AGGREGATION_VKEY_HASH;

pub const AGGCHAIN_TYPE: u16 = 0x0001;
