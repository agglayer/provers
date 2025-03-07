mod keccak;
mod proof;

pub use keccak::{digest::Digest, keccak256_combine};
pub use proof::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum NetworkIndex {
    #[default]
    Mainnet,
    Rollup(u32),
}
