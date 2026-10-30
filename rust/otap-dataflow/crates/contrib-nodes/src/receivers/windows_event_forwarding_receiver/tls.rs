// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{config::Config, identity::SourceIdentity};
use otel_arrow_dfe_config::tls::{TlsConfig, TlsServerConfig};
use otel_arrow_dfe_otap::tls_utils::{
    accept_tls_connection, build_reloadable_server_config, read_file_with_limit_async,
};
use rustls::pki_types::{CertificateDer, pem::PemObject};
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{TlsAcceptor, server::TlsStream};

const MAX_CLIENT_CA_BYTES: usize = 4 * 1024 * 1024; // 4 MiB total for all client CA bundles.

/// An acceptor that requires certificates from the configured client CAs.
#[derive(Clone)]
pub struct SourceAcceptor {
    acceptor: TlsAcceptor,
}

/// Build an HTTP/1.1 TLS acceptor without binding a socket.
///
/// Each credential file is bounded by the shared TLS loader's 4 MiB limit.
/// Combined client CA PEM, including one separator byte per file, is limited
/// to 4 MiB. Every CA file must contain at least one certificate.
/// Credentials and client trust are startup snapshots; changes require
/// rebuilding the acceptor.
///
/// The process must install its selected rustls crypto provider first.
///
/// # Errors
///
/// Returns configuration, credential loading, or TLS setup errors. System
/// trust anchors are never added, and client authentication cannot be disabled.
pub async fn build_acceptor(config: &Config) -> Result<SourceAcceptor, io::Error> {
    config.validate().map_err(invalid_input)?;
    let mut client_ca_pem = String::new();
    for path in &config.tls.client_ca_files {
        let bytes = read_file_with_limit_async(path).await?;
        if client_ca_pem
            .len()
            .saturating_add(bytes.len())
            .saturating_add(1)
            > MAX_CLIENT_CA_BYTES
        {
            return Err(invalid_input("combined client CA bundles exceed 4 MiB"));
        }
        let pem = std::str::from_utf8(&bytes).map_err(invalid_input)?;
        let mut certificates = CertificateDer::pem_slice_iter(&bytes).peekable();
        if certificates.peek().is_none() {
            return Err(invalid_input(format!(
                "client CA file {path:?} contains no certificates"
            )));
        }
        for certificate in certificates {
            let _ = certificate.map_err(invalid_input)?;
        }
        client_ca_pem.push_str(pem);
        client_ca_pem.push('\n');
    }
    // Inline PEM makes the shared builder validate the certificate/key pair at
    // startup rather than deferring validation to a lazily resolved handshake.
    let cert_pem = String::from_utf8(read_file_with_limit_async(&config.tls.cert_file).await?)
        .map_err(invalid_input)?;
    let key_pem = String::from_utf8(read_file_with_limit_async(&config.tls.key_file).await?)
        .map_err(invalid_input)?;
    let server_config = TlsServerConfig {
        config: TlsConfig {
            cert_pem: Some(cert_pem),
            key_pem: Some(key_pem),
            ..Default::default()
        },
        client_ca_pem: Some(client_ca_pem),
        include_system_ca_certs_pool: Some(false),
        handshake_timeout: Some(config.limits.handshake_timeout),
        ..Default::default()
    };
    let mut server = build_reloadable_server_config(&server_config).await?;
    Arc::make_mut(&mut server).alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(SourceAcceptor {
        acceptor: TlsAcceptor::from(server),
    })
}

/// Complete a bounded mutual TLS handshake, then authorize the verified leaf.
///
/// # Errors
///
/// Returns `TimedOut` for a stalled handshake, TLS errors for invalid peers,
/// and `PermissionDenied` when the verified peer has no authorized DNS identity.
pub async fn accept_source<Stream>(
    stream: Stream,
    acceptor: &SourceAcceptor,
    config: &Config,
) -> Result<(TlsStream<Stream>, SourceIdentity), io::Error>
where
    Stream: AsyncRead + AsyncWrite + Unpin,
{
    let stream =
        accept_tls_connection(stream, &acceptor.acceptor, config.limits.handshake_timeout).await?;
    let certificate = stream
        .get_ref()
        .1
        .peer_certificates()
        .and_then(|certificates| certificates.first())
        .ok_or_else(|| invalid_input("verified client certificate is required"))?;
    let identity = SourceIdentity::from_verified_certificate(certificate.as_ref(), &config.auth)
        .map_err(|error| io::Error::new(io::ErrorKind::PermissionDenied, error))?;
    Ok((stream, identity))
}

fn invalid_input(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_otap::crypto::ensure_crypto_provider;
    use otel_arrow_dfe_test_tls_certs::{
        ExtendedKeyUsage, GeneratedCa, GeneratedCert, generate_ca,
    };
    use rustls::{
        ClientConfig, RootCertStore,
        pki_types::{PrivateKeyDer, ServerName},
    };
    use std::{fs, time::Duration};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_rustls::TlsConnector;

    fn fixture() -> (tempfile::TempDir, GeneratedCa, GeneratedCa, Config) {
        ensure_crypto_provider();
        let directory = tempfile::tempdir().unwrap();
        let server_ca = generate_ca("WEF server CA");
        server_ca
            .issue_leaf(
                "localhost",
                Some("localhost"),
                Some(ExtendedKeyUsage::ServerAuth),
            )
            .write_to_dir(directory.path(), "server");
        let client_ca = generate_ca("WEF client CA");
        client_ca.write_cert_to_dir(directory.path(), "ca");
        let config = serde_json::from_value(serde_json::json!({
            "endpoint": "127.0.0.1:0",
            "tls": {
                "cert_file": directory.path().join("server.crt"),
                "key_file": directory.path().join("server.key"),
                "client_ca_files": [directory.path().join("ca.crt")]
            },
            "auth": {"allowed_sources": ["host.example.com"]},
            "subscriptions": [{"name": "test", "query": "<QueryList/>"}]
        }))
        .unwrap();
        (directory, server_ca, client_ca, config)
    }

    fn connector(server_ca: &str, client: Option<&GeneratedCert>) -> TlsConnector {
        let mut roots = RootCertStore::empty();
        roots
            .add(CertificateDer::from_pem_slice(server_ca.as_bytes()).unwrap())
            .unwrap();
        let builder = ClientConfig::builder().with_root_certificates(roots);
        let mut config = match client {
            Some(client) => builder
                .with_client_auth_cert(
                    vec![CertificateDer::from_pem_slice(client.cert_pem.as_bytes()).unwrap()],
                    PrivateKeyDer::from_pem_slice(client.key_pem.as_bytes()).unwrap(),
                )
                .unwrap(),
            None => builder.with_no_client_auth(),
        };
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        TlsConnector::from(Arc::new(config))
    }

    /// Scenario: clients present trusted, missing, untrusted, unauthorized, or wrong-EKU certificates.
    /// Guarantees: only verified allowed sources from configured CAs obtain an HTTP/1.1 TLS stream.
    #[tokio::test]
    async fn mutual_tls_authentication() {
        let (directory, server_ca, client_ca, mut config) = fixture();
        let second_ca = generate_ca("second WEF client CA");
        second_ca.write_cert_to_dir(directory.path(), "second");
        config
            .tls
            .client_ca_files
            .push(directory.path().join("second.crt"));
        let acceptor = build_acceptor(&config).await.unwrap();
        let trusted = client_ca.issue_leaf(
            "host",
            Some("HOST.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let second = second_ca.issue_leaf(
            "host",
            Some("host.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let unauthorized = client_ca.issue_leaf(
            "other",
            Some("other.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let missing_san =
            client_ca.issue_leaf("host.example.com", None, Some(ExtendedKeyUsage::ClientAuth));
        let untrusted = server_ca.issue_leaf(
            "host",
            Some("host.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let wrong_usage = client_ca.issue_leaf(
            "host",
            Some("host.example.com"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        for (name, client, accepted, unauthorized) in [
            ("trusted", Some(&trusted), true, false),
            ("second CA", Some(&second), true, false),
            ("missing certificate", None, false, false),
            ("unauthorized", Some(&unauthorized), false, true),
            ("missing SAN", Some(&missing_san), false, true),
            ("untrusted", Some(&untrusted), false, false),
            ("wrong EKU", Some(&wrong_usage), false, false),
        ] {
            let connector = connector(&server_ca.cert_pem, client);
            let (server_io, client_io) = tokio::io::duplex(64 * 1024);
            let (server, _) = tokio::time::timeout(Duration::from_secs(5), async {
                tokio::join!(
                    accept_source(server_io, &acceptor, &config),
                    connector.connect(ServerName::try_from("localhost").unwrap(), client_io),
                )
            })
            .await
            .unwrap();
            assert_eq!(server.is_ok(), accepted, "{name}");
            match server {
                Ok((stream, identity)) => {
                    assert_eq!(identity.as_str(), "host.example.com");
                    assert_eq!(
                        stream.get_ref().1.alpn_protocol(),
                        Some(b"http/1.1".as_slice())
                    );
                }
                Err(error) if unauthorized => {
                    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied)
                }
                Err(_) => {}
            }
        }
    }

    /// Scenario: an accepted transport never sends a TLS handshake.
    /// Guarantees: the configured deadline returns TimedOut and drops the stalled connection.
    #[tokio::test]
    async fn handshake_timeout() {
        let (_directory, _server_ca, _client_ca, mut config) = fixture();
        config.limits.handshake_timeout = Duration::from_millis(25);
        let acceptor = build_acceptor(&config).await.unwrap();
        tokio::time::pause();
        let (server, mut client) = tokio::io::duplex(1024);
        let start = tokio::time::Instant::now();
        let error = accept_source(server, &acceptor, &config).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(start.elapsed() >= config.limits.handshake_timeout);
        assert!(start.elapsed() <= config.limits.handshake_timeout + Duration::from_millis(1));
        assert_eq!(client.read(&mut [0]).await.unwrap(), 0);
    }

    /// Scenario: an established TCP connection presents plaintext instead of TLS.
    /// Guarantees: plaintext is rejected without yielding an authorized source.
    #[tokio::test]
    async fn rejects_plaintext() {
        let (_directory, _server_ca, _client_ca, config) = fixture();
        let acceptor = build_acceptor(&config).await.unwrap();
        let (server, mut client) = tokio::io::duplex(1024);
        client.write_all(b"GET / HTTP/1.1\r\n\r\n").await.unwrap();
        let error = accept_source(server, &acceptor, &config).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    /// Scenario: startup credentials are missing, empty, malformed, mismatched, or not UTF-8.
    /// Guarantees: invalid server credentials and every invalid CA file fail setup explicitly.
    #[tokio::test]
    async fn rejects_invalid_credentials() {
        let (directory, _server_ca, client_ca, config) = fixture();
        for path in [
            &config.tls.cert_file,
            &config.tls.key_file,
            &config.tls.client_ca_files[0],
        ] {
            let original = fs::read(path).unwrap();
            for bytes in [
                b"".as_slice(),
                b"not PEM",
                b"\xff",
                b"-----BEGIN CERTIFICATE-----\nAA==\n-----END CERTIFICATE-----\n",
            ] {
                fs::write(path, bytes).unwrap();
                assert!(build_acceptor(&config).await.is_err(), "{path:?}");
            }
            fs::remove_file(path).unwrap();
            assert!(build_acceptor(&config).await.is_err(), "{path:?}");
            fs::write(path, original).unwrap();
        }
        let original_key = fs::read(&config.tls.key_file).unwrap();
        let other = client_ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        fs::write(&config.tls.key_file, other.key_pem).unwrap();
        assert!(build_acceptor(&config).await.is_err());
        fs::write(&config.tls.key_file, original_key).unwrap();
        let empty_ca = directory.path().join("empty.crt");
        fs::write(&empty_ca, "").unwrap();
        let mut config = config;
        config.tls.client_ca_files.push(empty_ca);
        assert!(build_acceptor(&config).await.is_err());
        config.tls.client_ca_files.clear();
        assert!(build_acceptor(&config).await.is_err());
    }

    /// Scenario: a credential exceeds 4 MiB or combined CA PEM reaches its aggregate limit.
    /// Guarantees: every oversized file fails, and aggregate PEM accepts exactly 4 MiB but not one byte more.
    #[tokio::test]
    async fn bounds_credential_loading() {
        let (directory, _server_ca, client_ca, mut config) = fixture();
        for path in [
            &config.tls.cert_file,
            &config.tls.key_file,
            &config.tls.client_ca_files[0],
        ] {
            let original = fs::read(path).unwrap();
            fs::File::create(path)
                .unwrap()
                .set_len(MAX_CLIENT_CA_BYTES as u64 + 1)
                .unwrap();
            assert!(build_acceptor(&config).await.is_err(), "{path:?}");
            fs::write(path, original).unwrap();
        }
        let second = directory.path().join("second.crt");
        config.tls.client_ca_files.push(second.clone());
        let mut pem = client_ca.cert_pem.into_bytes();
        pem.resize(MAX_CLIENT_CA_BYTES / 2 - 1, b'\n');
        for path in &config.tls.client_ca_files {
            fs::write(path, &pem).unwrap();
        }
        assert!(build_acceptor(&config).await.is_ok());
        pem.push(b'\n');
        fs::write(second, pem).unwrap();
        let error = build_acceptor(&config).await.err().unwrap();
        assert_eq!(error.to_string(), "combined client CA bundles exceed 4 MiB");
    }
}
