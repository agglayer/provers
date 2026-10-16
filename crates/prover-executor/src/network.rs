use std::{env, ffi::OsString, fs, path::PathBuf, time::Duration};

use eyre::{bail, Context as _};
use sp1_sdk::{network::builder::NetworkProverBuilder, NetworkProver};
use tracing::info;

const NETWORK_MTLS_CERT_PATH: &str = "NETWORK_MTLS_CERT_PATH";
const NETWORK_MTLS_KEY_PATH: &str = "NETWORK_MTLS_KEY_PATH";
const NETWORK_MTLS_PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(20);

pub async fn build_network_prover(
    mut builder: NetworkProverBuilder,
) -> eyre::Result<NetworkProver> {
    let client_identity = load_network_client_identity()?;
    let uses_client_identity = client_identity.is_some();

    if let Some((certificate, private_key)) = client_identity {
        builder = builder.client_identity(certificate, private_key);
    }

    let prover = builder.build().await;
    if uses_client_identity {
        tokio::time::timeout(NETWORK_MTLS_PREFLIGHT_TIMEOUT, prover.get_balance())
            .await
            .context("Network mTLS preflight timed out after 20 seconds")?
            .map_err(|error| eyre::eyre!("{error:#}"))
            .context("Network mTLS preflight failed")?;
        info!("SP1 prover network mTLS enabled");
    }

    Ok(prover)
}

fn load_network_client_identity() -> eyre::Result<Option<(Vec<u8>, Vec<u8>)>> {
    load_network_client_identity_from_paths(
        env::var_os(NETWORK_MTLS_CERT_PATH),
        env::var_os(NETWORK_MTLS_KEY_PATH),
    )
}

fn load_network_client_identity_from_paths(
    certificate_path: Option<OsString>,
    private_key_path: Option<OsString>,
) -> eyre::Result<Option<(Vec<u8>, Vec<u8>)>> {
    let (certificate_path, private_key_path) = match (certificate_path, private_key_path) {
        (None, None) => return Ok(None),
        (Some(certificate_path), Some(private_key_path)) => (
            PathBuf::from(certificate_path),
            PathBuf::from(private_key_path),
        ),
        _ => {
            return Err(eyre::eyre!(
                "{NETWORK_MTLS_CERT_PATH} and {NETWORK_MTLS_KEY_PATH} must be set together"
            ))
        }
    };

    if certificate_path.as_os_str().is_empty() {
        bail!("{NETWORK_MTLS_CERT_PATH} must not be empty");
    }
    if private_key_path.as_os_str().is_empty() {
        bail!("{NETWORK_MTLS_KEY_PATH} must not be empty");
    }

    let certificate = fs::read(&certificate_path).with_context(|| {
        format!(
            "Failed to read {NETWORK_MTLS_CERT_PATH} file at {}",
            certificate_path.display()
        )
    })?;
    let private_key = fs::read(&private_key_path).with_context(|| {
        format!(
            "Failed to read {NETWORK_MTLS_KEY_PATH} file at {}",
            private_key_path.display()
        )
    })?;

    Ok(Some((certificate, private_key)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_identity_is_optional() {
        let identity = load_network_client_identity_from_paths(None, None).unwrap();

        assert!(identity.is_none());
    }

    #[test]
    fn client_identity_paths_must_be_set_together() {
        let error =
            load_network_client_identity_from_paths(Some(OsString::from("certificate.pem")), None)
                .unwrap_err();

        assert!(error.to_string().contains("must be set together"));
    }

    #[test]
    fn client_identity_paths_must_not_be_empty() {
        let error = load_network_client_identity_from_paths(
            Some(OsString::new()),
            Some(OsString::from("private-key.pem")),
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("NETWORK_MTLS_CERT_PATH must not be empty"));
    }

    #[test]
    fn loads_client_identity_files() {
        let directory = tempfile::tempdir().unwrap();
        let certificate_path = directory.path().join("certificate.pem");
        let private_key_path = directory.path().join("private-key.pem");
        fs::write(&certificate_path, b"certificate").unwrap();
        fs::write(&private_key_path, b"private key").unwrap();

        let identity = load_network_client_identity_from_paths(
            Some(certificate_path.into_os_string()),
            Some(private_key_path.into_os_string()),
        )
        .unwrap()
        .unwrap();

        assert_eq!(identity.0, b"certificate");
        assert_eq!(identity.1, b"private key");
    }

    #[test]
    fn reports_unreadable_client_identity_file() {
        let directory = tempfile::tempdir().unwrap();
        let certificate_path = directory.path().join("missing-certificate.pem");
        let private_key_path = directory.path().join("private-key.pem");
        fs::write(&private_key_path, b"private key").unwrap();

        let error = load_network_client_identity_from_paths(
            Some(certificate_path.into_os_string()),
            Some(private_key_path.into_os_string()),
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("Failed to read NETWORK_MTLS_CERT_PATH file"));
    }
}
