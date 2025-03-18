pub mod bridge;
pub mod error;
pub mod full_execution_proof;
pub mod keccak;
pub mod local_exit_tree;
pub mod proof;
pub mod vkey_hash;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub use bridge::{
    inserted_ger::{InsertedGER, L1InfoTreeLeaf},
    BridgeWitness,
};
pub use keccak::digest::Digest;

pub const AGGCHAIN_TYPE: u16 = 0x0001;
