use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;

use aggchain_proof_service::config::AggchainProofServiceConfig;
use prover_config::{NetworkProverConfig, ProverType};
use prover_logger::log::Log;
use serde::{Deserialize, Serialize};

pub use crate::{shutdown::ShutdownConfig, telemetry::TelemetryConfig};

pub mod shutdown;
pub(crate) mod telemetry;

pub(crate) const DEFAULT_IP: std::net::Ipv4Addr = std::net::Ipv4Addr::new(0, 0, 0, 0);

/// The Aggkit Prover configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProverConfig {
    /// The gRPC endpoint used by the prover.
    #[serde(default = "default_socket_addr")]
    pub grpc_endpoint: SocketAddr,

    #[serde(default, skip_serializing_if = "crate::default")]
    pub grpc: GrpcConfig,

    /// The log configuration.
    #[serde(default)]
    pub log: Log,

    /// Telemetry configuration.
    #[serde(default)]
    pub telemetry: TelemetryConfig,

    /// The list of configuration options used during shutdown.
    #[serde(default)]
    pub shutdown: ShutdownConfig,

    #[serde(default)]
    pub aggchain_proof_service: AggchainProofServiceConfig,

    /// The primary prover to be used for generation of the pessimistic proof
    #[serde(default)]
    pub primary_prover: ProverType,

    /// The fallback prover to be used for generation of the pessimistic proof
    #[serde(default)]
    pub fallback_prover: Option<ProverType>,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            grpc_endpoint: default_socket_addr(),
            log: Log::default(),
            telemetry: TelemetryConfig::default(),
            shutdown: ShutdownConfig::default(),
            aggchain_proof_service: AggchainProofServiceConfig::default(),
            primary_prover: ProverType::NetworkProver(NetworkProverConfig::default()),
            fallback_prover: None,
            grpc: Default::default(),
        }
    }
}

impl ProverConfig {
    pub fn try_load(path: &Path) -> Result<Self, ConfigurationError> {
        let reader = std::fs::read_to_string(path).map_err(|source| {
            ConfigurationError::UnableToReadConfigFile {
                path: path.to_path_buf(),
                source,
            }
        })?;

        let deserializer = toml::de::Deserializer::new(&reader);
        serde::Deserialize::deserialize(deserializer)
            .map_err(ConfigurationError::DeserializationError)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GrpcConfig {
    #[serde(
        skip_serializing_if = "same_as_default_max_decoding_message_size",
        default = "default_max_decoding_message_size"
    )]
    pub max_decoding_message_size: usize,
    #[serde(
        skip_serializing_if = "same_as_default_max_encoding_message_size",
        default = "default_max_encoding_message_size"
    )]
    pub max_encoding_message_size: usize,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            max_decoding_message_size: default_max_decoding_message_size(),
            max_encoding_message_size: default_max_encoding_message_size(),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ClientProverConfig {
    #[serde(default)]
    pub grpc: GrpcConfig,
}

const fn default_max_decoding_message_size() -> usize {
    4 * 1024 * 1024
}

fn same_as_default_max_decoding_message_size(value: &usize) -> bool {
    *value == default_max_decoding_message_size()
}

const fn default_max_encoding_message_size() -> usize {
    4 * 1024 * 1024
}

fn same_as_default_max_encoding_message_size(value: &usize) -> bool {
    *value == default_max_encoding_message_size()
}

const fn default_socket_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081)
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Unable to read the configuration file: {source}")]
    UnableToReadConfigFile {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to deserialize the configuration: {0}")]
    DeserializationError(#[from] toml::de::Error),
}

pub(crate) fn default<T: Default + PartialEq>(t: &T) -> bool {
    *t == Default::default()
}
