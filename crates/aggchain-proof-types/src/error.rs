use hex::FromHexError;

/// Represents the errors that could happen with the grpc request
/// to generate the aggchain proof
#[derive(thiserror::Error, Debug)]
pub enum AggchainProofRequestError {
    #[error("Missing token info in the aggchain proof request")]
    MissingTokenInfo,

    #[error("Missing request bridge exit in the aggchain proof request")]
    MissingBridgeExit,

    #[error("Missing request global index in the aggchain proof request")]
    MissingGlobalIndex,

    #[error("Missing inner l1 info tree leaf in the aggchain proof request")]
    MissingL1InfoTreeLeafInner,

    #[error("Missing l1 info tree leaf in the aggchain proof request")]
    MissingL1InfoTreeLeaf,

    #[error("Missing or invalid l1 info merkle tree proof in the aggchain proof request")]
    MissingL1InfoTreeMerkleProof,

    #[error("Invalid claim from mainet value in the aggchain proof request")]
    InvalidClaimFromMainnetConversion,

    #[error("Missing inclusion proof in the aggchain proof request")]
    MissingInclusionProof,

    #[error("Invalid hex conversion")]
    InvalidHexConversion(#[source] FromHexError),
}
