use agglayer_primitives::Digest;
use alloy_primitives::Address;

use crate::bridge::BridgeConstraintsError;

/// Represents all the aggchain proof errors.
#[derive(thiserror::Error, Debug)]
pub enum ProofError {
    /// Error on the bridge constraints.
    #[error(transparent)]
    BridgeConstraintsError(#[from] BridgeConstraintsError),

    /// The L1 Head provided as public input of the FEP do not match the block
    /// hash contained in the L1 info tree leaf used to verify the inclusion
    /// proof to L1 info root.
    #[error(
        "Mismatch on the L1 head. from_l1_info_tree_leaf: {from_l1_info_tree_leaf}. \
         from_fep_public_values: {from_fep_public_values}."
    )]
    MismatchL1Head {
        from_l1_info_tree_leaf: Digest,
        from_fep_public_values: Digest,
    },

    /// The inclusion proof of the L1 info tree leaf containing the L1 Head is
    /// invalid.
    #[error(
        "Invalid inclusion proof for the L1 info tree leaf containing the L1 head. index: \
         {index}, l1_leaf_hash: {l1_leaf_hash}, l1_info_root: {l1_info_root}."
    )]
    InvalidInclusionProofL1Head {
        index: u32,
        l1_leaf_hash: Digest,
        l1_info_root: Digest,
    },

    /// The signature on the fep public values is invalid.
    #[error("Invalid signature.")]
    InvalidSignature,

    /// The signer recovered from the signature differs from the one declared as
    /// witness.
    #[error("Invalid signer. declared: {declared}, recovered: {recovered}")]
    InvalidSigner {
        declared: Address,
        recovered: Address,
    },
}
