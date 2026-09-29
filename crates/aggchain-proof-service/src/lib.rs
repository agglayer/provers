pub mod config;

mod custom_chain_data;
mod error;
pub mod service;

pub use aggchain_proof_builder::{AGGCHAIN_PROOF_ELF, AGGCHAIN_PROOF_NOOP_ELF};
pub use custom_chain_data::{VKeySelector, AGGCHAIN_VKEY_SELECTOR, NOOP_SELECTOR};
pub use error::Error;
