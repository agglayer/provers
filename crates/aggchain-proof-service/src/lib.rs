pub mod config;

mod custom_chain_data;
mod error;
pub mod service;

pub use aggchain_proof_builder::{AGGCHAIN_PROOF_ELF, AGGCHAIN_PROOF_MOCK_ELF, MOCK_VKEY};
pub use custom_chain_data::{VKeySelector, AGGCHAIN_VKEY_SELECTOR, MOCK_SELECTOR};
pub use error::Error;
