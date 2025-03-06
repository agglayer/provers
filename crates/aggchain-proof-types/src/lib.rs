mod proof;

pub use aggchain_proof_core::keccak::{digest::Digest, keccak256_combine};
pub use aggchain_proof_core::vkey_hash::HashU32;
pub use proof::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum NetworkIndex {
    #[default]
    Mainnet,
    Rollup(u32),
}
