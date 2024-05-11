//! Agglayer configuration.
//!
//! The agglayer is configured via its TOML configuration file, `agglayer.toml`
//! by default, which is deserialized into the [`Config`] struct.

use std::collections::HashMap;

use agglayer_signer::ConfiguredSigner;
use ethers::signers::{LocalWallet, Signer};
use ethers_gcp_kms_signer::{GcpKeyRingRef, GcpKmsProvider, GcpKmsSigner};
use serde::Deserialize;
use tracing::debug;
use url::Url;

use self::{eth_tx_manager::PrivateKey, rpc::deserialize_rpc_map, telemetry::TelemetryConfig};

pub(crate) const DEFAULT_IP: std::net::Ipv4Addr = std::net::Ipv4Addr::new(0, 0, 0, 0);

pub(crate) mod epoch;
pub(crate) mod error;
pub(crate) mod eth_tx_manager;
pub(crate) mod l1;
pub mod log;
pub(crate) mod rpc;
pub(crate) mod telemetry;

pub use epoch::Epoch;
pub use error::ConfigError;
pub use eth_tx_manager::EthTxManager;
pub use l1::L1;
pub use log::Log;
pub use rpc::RpcConfig;

/// The Agglayer configuration.
#[derive(Deserialize, Debug)]
#[cfg_attr(any(test, feature = "testutils"), derive(Default))]
pub struct Config {
    /// A map of Zkevm node RPC endpoints for each rollup.
    ///
    /// The key is the rollup ID, and the value is the URL of the associated RPC
    /// endpoint.
    #[serde(rename = "FullNodeRPCs", deserialize_with = "deserialize_rpc_map")]
    pub full_node_rpcs: HashMap<u32, Url>,
    /// The log configuration.
    #[serde(rename = "Log")]
    pub log: Log,
    /// The local RPC server configuration.
    #[serde(rename = "RPC")]
    pub rpc: RpcConfig,
    /// The L1 configuration.
    #[serde(rename = "L1")]
    pub l1: L1,
    /// The transaction management configuration.
    #[serde(rename = "EthTxManager")]
    pub eth_tx_manager: EthTxManager,

    /// Telemetry configuration.
    #[serde(rename = "Telemetry")]
    pub telemetry: TelemetryConfig,

    /// The Epoch configuration.
    #[serde(rename = "Epoch", default = "Epoch::default")]
    pub epoch: Epoch,
}

impl Config {
    /// Get the target RPC socket address from the configuration.
    pub fn rpc_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from((self.rpc.host, self.rpc.port))
    }

    /// Get the first local private key specified in the configuration.
    fn local_pk(&self) -> Result<&PrivateKey, ConfigError> {
        self.eth_tx_manager
            .private_keys
            .first()
            .ok_or(ConfigError::NoPk)
    }

    /// Decrypt the first local keystore specified in the configuration.
    pub(crate) fn local_wallet(&self) -> Result<LocalWallet, ConfigError> {
        let pk = self.local_pk()?;
        Ok(LocalWallet::decrypt_keystore(&pk.path, &pk.password)?.with_chain_id(self.l1.chain_id))
    }

    /// Create a GCP KMS signer from the configuration.
    /// This will first attempt to use the environment variables, and if they
    /// are not set, it will fall back to the values specified configuration
    /// file.
    ///
    /// The `ethers_gcp_kms_signer` library will attempt to load credentials in
    /// the typical fashion for GCP:
    /// - If the application is running in a k8s cluster, it should
    ///   automatically pick up credentials.
    /// - If the `GOOGLE_APPLICATION_CREDENTIALS` environment is set, attempt to
    ///   load a service account JSON from this path.
    pub(crate) async fn gcp_kms_signer(&self) -> Result<GcpKmsSigner, ConfigError> {
        let project_id = std::env::var("GOOGLE_PROJECT_ID").or_else(|_| {
            self.eth_tx_manager
                .kms_project_id
                .clone()
                .ok_or(ConfigError::KmsConfig("GOOGLE_PROJECT_ID".to_string()))
        })?;
        let location = std::env::var("GOOGLE_LOCATION").or_else(|_| {
            self.eth_tx_manager
                .kms_location
                .clone()
                .ok_or(ConfigError::KmsConfig("GOOGLE_LOCATION".to_string()))
        })?;
        let keyring = std::env::var("GOOGLE_KEYRING").or_else(|_| {
            self.eth_tx_manager
                .kms_keyring
                .clone()
                .ok_or(ConfigError::KmsConfig("GOOGLE_KEYRING".to_string()))
        })?;
        let key_name = std::env::var("GOOGLE_KEY_NAME").or_else(|_| {
            self.eth_tx_manager
                .kms_key_name
                .clone()
                .ok_or(ConfigError::KmsConfig("GOOGLE_KEY_NAME".to_string()))
        })?;

        let keyring = GcpKeyRingRef::new(&project_id, &location, &keyring);
        let provider = GcpKmsProvider::new(keyring).await?;
        Ok(GcpKmsSigner::new(provider, key_name.to_string(), 1, self.l1.chain_id).await?)
    }

    /// Get either a local wallet or GCP KMS signer based on the configuration.
    ///
    /// The logic here that determines which signer to use is as follows:
    /// 1. If a GCP KMS key name is specified, attempt to use the GCP KMS
    ///    signer.
    /// 2. Otherwise, attempt use the local wallet.
    ///
    /// This logic is ported directly from the original agglayer Go codebase.
    pub async fn get_configured_signer(&self) -> Result<ConfiguredSigner, ConfigError> {
        if self.eth_tx_manager.kms_key_name.is_some() {
            debug!("Using GCP KMS signer");
            Ok(ConfiguredSigner::GcpKms(self.gcp_kms_signer().await?))
        } else {
            debug!("Using local wallet signer");
            Ok(ConfiguredSigner::Local(self.local_wallet()?))
        }
    }
}
