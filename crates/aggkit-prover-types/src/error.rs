/// Represents the errors that could happen with the grpc request
/// to generate the aggchain proof
#[derive(thiserror::Error, Debug)]
pub enum AggchainProofRequestError {
    #[error("Missing bridge exit token info")]
    MissingTokenInfo { field_path: String },

    #[error("Missing request bridge exit")]
    MissingBridgeExit { field_path: String },

    #[error("Missing request global index")]
    MissingGlobalIndex { field_path: String },

    #[error("Missing inner l1 info tree leaf")]
    MissingL1InfoTreeLeafInner { field_path: String },

    #[error("Missing l1 info tree leaf")]
    MissingL1InfoTreeLeaf { field_path: String },

    #[error("Invalid l1 info tree leaf")]
    InvalidL1InfoTreeLeaf {
        field_path: String,
        source: anyhow::Error,
    },

    #[error("Missing l1 info tree root hash")]
    MissingL1InfoTreeRootHash { field_path: String },

    #[error("Missing or invalid l1 info merkle tree proof")]
    MissingL1InfoTreeMerkleProof { field_path: String },

    #[error("Invalid l1 info tree merkle proof")]
    InvalidL1InfoTreeMerkleProof {
        source: anyhow::Error,
        field_path: String,
    },

    #[error("Invalid claim from mainnet value")]
    InvalidClaimFromMainnetConversion { field_path: String },

    #[error("Missing inclusion proof")]
    MissingInclusionProof { field_path: String },

    #[error("Invalid digest")]
    InvalidDigest {
        field_path: String,
        source: anyhow::Error,
    },

    #[error("Invalid imported bridge exit")]
    InvalidImportedBridgeExit {
        field_path: String,
        source: anyhow::Error,
    },

    #[error("Missing imported bridge exit")]
    MissingImportedBridgeExit { field_path: String },
    #[error("Missing inserted ger")]
    MissingInsertedGer { field_path: String },
    #[error("Invalid inserted ger")]
    InvalidInsertedGer {
        field_path: String,
        source: anyhow::Error,
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
            | AggchainProofRequestError::InvalidL1InfoTreeMerkleProof { field_path, .. }
            | AggchainProofRequestError::InvalidClaimFromMainnetConversion { field_path }
            | AggchainProofRequestError::InvalidImportedBridgeExit { field_path, .. }
            | AggchainProofRequestError::InvalidL1InfoTreeLeaf { field_path, .. }
            | AggchainProofRequestError::MissingL1InfoTreeRootHash { field_path }
            | AggchainProofRequestError::MissingInsertedGer { field_path }
            | AggchainProofRequestError::InvalidInsertedGer { field_path, .. }
            | AggchainProofRequestError::MissingImportedBridgeExit { field_path }
            | AggchainProofRequestError::MissingInclusionProof { field_path }
            | AggchainProofRequestError::InvalidDigest { field_path, .. } => field_path,
        }
    }
}
