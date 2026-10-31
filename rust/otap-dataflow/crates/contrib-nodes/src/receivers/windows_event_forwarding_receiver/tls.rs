// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{config::Config, identity::SourceIdentity};
use otel_arrow_dfe_config::tls::{TlsConfig, TlsServerConfig};
use otel_arrow_dfe_otap::tls_utils::{
    accept_tls_connection, build_reloadable_server_config, read_file_with_limit_async,
};
use rustls::{
    crypto::WebPkiSupportedAlgorithms,
    pki_types::{CertificateDer, UnixTime, pem::PemObject},
};
use sha1::{Digest, Sha1};
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use x509_parser::prelude::{FromDer, X509Certificate};

// Bound combined CA PEM at startup and, separately, peer DER during issuer selection to 4 MiB.
const MAX_CLIENT_CA_BYTES: usize = 4 * 1024 * 1024;
// Bound issuer-search work across peer intermediates and configured CA certificates, including duplicates.
const MAX_ISSUER_CANDIDATES: usize = 256;

/// An acceptor that requires certificates from the configured client CAs.
#[derive(Clone)]
pub struct SourceAcceptor {
    acceptor: TlsAcceptor,
    client_cas: Arc<[CertificateDer<'static>]>,
    signature_algorithms: WebPkiSupportedAlgorithms,
}

impl SourceAcceptor {
    /// Select the direct issuer of an already TLS-verified and authorized client.
    ///
    /// Candidates come from the supplied peer intermediates and startup CA
    /// snapshot, not from disk or the network. The selected TLS provider verifies
    /// the direct leaf-to-issuer path. SHA-1 is only the protocol's certificate
    /// identifier, not a signature algorithm or an authorization decision.
    /// Call once per authenticated connection, not per event or response.
    ///
    /// # Errors
    ///
    /// Fails for missing, malformed, or ambiguous issuers. Selection is bounded
    /// to 256 candidates and 4 MiB of peer DER, independently of TLS verification.
    pub fn issuer_thumbprint(&self, verified_peer: &[CertificateDer<'_>]) -> io::Result<[u8; 20]> {
        issuer_thumbprint(verified_peer, &self.client_cas, self.signature_algorithms)
    }
}

fn issuer_thumbprint(
    peer: &[CertificateDer<'_>],
    client_cas: &[CertificateDer<'_>],
    algorithms: WebPkiSupportedAlgorithms,
) -> io::Result<[u8; 20]> {
    if peer
        .len()
        .saturating_sub(1)
        .saturating_add(client_cas.len())
        > MAX_ISSUER_CANDIDATES
        || peer
            .iter()
            .try_fold(0usize, |size, cert| size.checked_add(cert.len()))
            .is_none_or(|size| size > MAX_CLIENT_CA_BYTES)
    {
        return Err(invalid_input("issuer selection exceeds certificate limits"));
    }
    let leaf_der = peer
        .first()
        .ok_or_else(|| invalid_input("missing verified client certificate"))?;
    let (remaining, leaf) = X509Certificate::from_der(leaf_der.as_ref()).map_err(invalid_input)?;
    if !remaining.is_empty() {
        return Err(invalid_input("trailing data in client certificate"));
    }
    let verified_leaf = webpki::EndEntityCert::try_from(leaf_der).map_err(invalid_input)?;
    let mut selected: Option<&CertificateDer<'_>> = None;
    for candidate in peer.iter().skip(1).chain(client_cas) {
        let (remaining, issuer) =
            X509Certificate::from_der(candidate.as_ref()).map_err(invalid_input)?;
        if !remaining.is_empty() {
            return Err(invalid_input("trailing data in issuer certificate"));
        }
        if !issuer.is_ca() || leaf.issuer() != issuer.subject() {
            continue;
        }
        let anchor = webpki::anchor_from_trusted_cert(candidate).map_err(invalid_input)?;
        // No intermediates: this candidate must directly sign the verified leaf.
        // In particular, a same-name CA with a different key is not an issuer.
        if verified_leaf
            .verify_for_usage(
                algorithms.all,
                &[anchor],
                &[],
                UnixTime::now(),
                webpki::KeyUsage::client_auth(),
                None,
                None,
            )
            .is_err()
        {
            continue;
        }
        if selected.is_some_and(|previous| previous.as_ref() != candidate.as_ref()) {
            return Err(invalid_input("ambiguous client certificate issuer"));
        }
        selected = Some(candidate);
    }
    let issuer = selected.ok_or_else(|| invalid_input("client certificate issuer unavailable"))?;
    Ok(Sha1::digest(issuer.as_ref()).into())
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
    let mut client_cas = Vec::new();
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
            client_cas.push(certificate.map_err(invalid_input)?.into_owned());
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
    let signature_algorithms = server.crypto_provider().signature_verification_algorithms;
    Ok(SourceAcceptor {
        acceptor: TlsAcceptor::from(server),
        client_cas: client_cas.into(),
        signature_algorithms,
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
                    let issuer_pem = if name == "second CA" {
                        &second_ca.cert_pem
                    } else {
                        &client_ca.cert_pem
                    };
                    let issuer = CertificateDer::from_pem_slice(issuer_pem.as_bytes()).unwrap();
                    let expected: [u8; 20] = Sha1::digest(issuer.as_ref()).into();
                    assert_eq!(
                        acceptor
                            .issuer_thumbprint(stream.get_ref().1.peer_certificates().unwrap())
                            .unwrap(),
                        expected
                    );
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

    /// Scenario: issuer candidates include same-name wrong keys, duplicate DER, and renewed leaves.
    /// Guarantees: only the direct signer is selected, regardless of chain/bundle order or leaf renewal.
    #[test]
    fn selects_direct_issuer() {
        ensure_crypto_provider();
        let algorithms = rustls::crypto::CryptoProvider::get_default()
            .unwrap()
            .signature_verification_algorithms;
        let ca = generate_ca("issuer");
        let wrong = generate_ca("issuer");
        let issuer = CertificateDer::from_pem_slice(ca.cert_pem.as_bytes()).unwrap();
        let wrong = CertificateDer::from_pem_slice(wrong.cert_pem.as_bytes()).unwrap();
        let expected: [u8; 20] = Sha1::digest(issuer.as_ref()).into();
        for _ in 0..2 {
            let client = ca.issue_leaf(
                "source",
                Some("host.example.com"),
                Some(ExtendedKeyUsage::ClientAuth),
            );
            let leaf = CertificateDer::from_pem_slice(client.cert_pem.as_bytes()).unwrap();
            for candidates in [
                vec![wrong.clone(), issuer.clone()],
                vec![issuer.clone(), wrong.clone(), issuer.clone()],
            ] {
                assert_eq!(
                    issuer_thumbprint(std::slice::from_ref(&leaf), &candidates, algorithms)
                        .unwrap(),
                    expected
                );
                let mut peer = vec![leaf.clone()];
                peer.extend(candidates);
                assert_eq!(issuer_thumbprint(&peer, &[], algorithms).unwrap(), expected);
            }
            assert!(
                issuer_thumbprint(
                    std::slice::from_ref(&leaf),
                    std::slice::from_ref(&wrong),
                    algorithms
                )
                .is_err()
            );
            assert!(issuer_thumbprint(std::slice::from_ref(&leaf), &[], algorithms).is_err());
        }
    }

    /// Scenario: a peer sends an intermediate issuer and its root, or two distinct same-key issuer certificates.
    /// Guarantees: the intermediate rather than root is advertised, and ambiguous issuer DER fails closed.
    #[test]
    fn rejects_ambiguous_issuers() {
        use rcgen::{
            BasicConstraints, CertificateParams, DnType, IsCa, Issuer, KeyPair, KeyUsagePurpose,
        };
        ensure_crypto_provider();
        let algorithms = rustls::crypto::CryptoProvider::get_default()
            .unwrap()
            .signature_verification_algorithms;
        let mut params = CertificateParams::new(Vec::new()).unwrap();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages.push(KeyUsagePurpose::KeyCertSign);
        params.distinguished_name.push(DnType::CommonName, "root");
        let root_key = KeyPair::generate().unwrap();
        let root = params.self_signed(&root_key).unwrap();
        let root_issuer = Issuer::new(params.clone(), root_key);
        params
            .distinguished_name
            .push(DnType::CommonName, "intermediate");
        let key = KeyPair::generate().unwrap();
        params.serial_number = Some(1u64.into());
        let intermediate = params.signed_by(&key, &root_issuer).unwrap();
        params.serial_number = Some(2u64.into());
        let renewed = params.signed_by(&key, &root_issuer).unwrap();
        let issuer = Issuer::new(params, key);
        let mut leaf_params = CertificateParams::new(vec!["host.example.com".into()]).unwrap();
        leaf_params
            .extended_key_usages
            .push(ExtendedKeyUsage::ClientAuth);
        let leaf = leaf_params
            .signed_by(&KeyPair::generate().unwrap(), &issuer)
            .unwrap();
        let expected: [u8; 20] = Sha1::digest(intermediate.der().as_ref()).into();
        assert_eq!(
            issuer_thumbprint(
                &[
                    leaf.der().clone(),
                    intermediate.der().clone(),
                    root.der().clone()
                ],
                &[],
                algorithms
            )
            .unwrap(),
            expected
        );
        assert!(
            issuer_thumbprint(&[leaf.der().clone()], &[root.der().clone()], algorithms).is_err()
        );
        assert!(
            issuer_thumbprint(
                &[leaf.der().clone(), intermediate.der().clone()],
                &[renewed.der().clone()],
                algorithms
            )
            .unwrap_err()
            .to_string()
            .contains("ambiguous")
        );
    }

    /// Scenario: selection receives absent, malformed, trailing-data, or excessive certificate inputs.
    /// Guarantees: invalid input and both count/byte limits produce explicit errors without a thumbprint.
    #[test]
    fn bounds_issuer_selection() {
        ensure_crypto_provider();
        let algorithms = rustls::crypto::CryptoProvider::get_default()
            .unwrap()
            .signature_verification_algorithms;
        let ca = generate_ca("issuer");
        let client = ca.issue_leaf(
            "source",
            Some("host.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let leaf = CertificateDer::from_pem_slice(client.cert_pem.as_bytes()).unwrap();
        let issuer = CertificateDer::from_pem_slice(ca.cert_pem.as_bytes()).unwrap();
        let malformed = CertificateDer::from(vec![0u8]);
        let mut trailing = issuer.as_ref().to_vec();
        trailing.push(0);
        for candidates in [
            vec![malformed.clone()],
            vec![CertificateDer::from(trailing)],
        ] {
            assert!(
                issuer_thumbprint(std::slice::from_ref(&leaf), &candidates, algorithms).is_err()
            );
        }
        assert!(issuer_thumbprint(&[], &[], algorithms).is_err());
        assert!(
            issuer_thumbprint(&[malformed], std::slice::from_ref(&issuer), algorithms).is_err()
        );
        let candidates = vec![issuer.clone(); MAX_ISSUER_CANDIDATES];
        assert!(issuer_thumbprint(std::slice::from_ref(&leaf), &candidates, algorithms).is_ok());
        assert!(
            issuer_thumbprint(&[leaf.clone(), issuer], &candidates, algorithms)
                .unwrap_err()
                .to_string()
                .contains("limits")
        );
        assert!(
            issuer_thumbprint(
                &[CertificateDer::from(vec![0; MAX_CLIENT_CA_BYTES + 1])],
                &[],
                algorithms
            )
            .unwrap_err()
            .to_string()
            .contains("limits")
        );
    }

    /// Scenario: the CA file changes after the TLS acceptor is built.
    /// Guarantees: issuer hints use the same startup trust snapshot as handshakes, without rereading disk.
    #[tokio::test]
    async fn issuer_selection_uses_startup_snapshot() {
        let (_directory, _server_ca, ca, config) = fixture();
        let acceptor = build_acceptor(&config).await.unwrap();
        let client = ca.issue_leaf(
            "source",
            Some("host.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let leaf = CertificateDer::from_pem_slice(client.cert_pem.as_bytes()).unwrap();
        let issuer = CertificateDer::from_pem_slice(ca.cert_pem.as_bytes()).unwrap();
        let expected: [u8; 20] = Sha1::digest(issuer.as_ref()).into();
        fs::write(
            &config.tls.client_ca_files[0],
            generate_ca("replacement").cert_pem,
        )
        .unwrap();
        assert_eq!(acceptor.issuer_thumbprint(&[leaf]).unwrap(), expected);
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
