use hex::FromHexError;

/// Represents the errors that could happen with the grpc request
/// to generate the aggchain proof
#[derive(thiserror::Error, Debug)]
pub enum AggchainProofRequestError {
    #[error("Missing bridge exit token info in the aggchain proof request")]
    MissingTokenInfo { field_path: String },

    #[error("Missing request bridge exit in the aggchain proof request")]
    MissingBridgeExit { field_path: String },

    #[error("Missing request global index in the aggchain proof request")]
    MissingGlobalIndex { field_path: String },

    #[error("Missing inner l1 info tree leaf in the aggchain proof request")]
    MissingL1InfoTreeLeafInner { field_path: String },

    #[error("Missing l1 info tree leaf in the aggchain proof request")]
    MissingL1InfoTreeLeaf { field_path: String },

    #[error("Missing or invalid l1 info merkle tree proof in the aggchain proof request")]
    MissingL1InfoTreeMerkleProof { field_path: String },

    #[error("Invalid claim from mainet value in the aggchain proof request")]
    InvalidClaimFromMainnetConversion { field_path: String },

    #[error("Missing inclusion proof in the aggchain proof request")]
    MissingInclusionProof { field_path: String },

    #[error("Invalid hex conversion")]
    InvalidHexConversion {
        field_path: String,
        #[source]
        error: FromHexError,
    },
}

impl AggchainProofRequestError {
    pub fn field_path(&self) -> &str {
        match self {
            AggchainProofRequestError::MissingTokenInfo { field_path }
            | AggchainProofRequestError::MissingBridgeExit { field_path }
            | AggchainProofRequestError::MissingGlobalIndex { field_path }
            | AggchainProofRequestError::MissingL1InfoTreeLeafInner { field_path }
            | AggchainProofRequestError::MissingL1InfoTreeLeaf { field_path }
            | AggchainProofRequestError::MissingL1InfoTreeMerkleProof { field_path }
            | AggchainProofRequestError::InvalidClaimFromMainnetConversion { field_path }
            | AggchainProofRequestError::MissingInclusionProof { field_path }
            | AggchainProofRequestError::InvalidHexConversion { field_path, .. } => field_path,
        }
    }
}
