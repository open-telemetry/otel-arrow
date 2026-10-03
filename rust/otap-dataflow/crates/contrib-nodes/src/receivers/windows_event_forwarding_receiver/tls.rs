// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{config::Config, identity::SourceIdentity};
use otel_arrow_dfe_config::tls::{TlsConfig, TlsServerConfig};
use otel_arrow_dfe_otap::tls_utils::{
    accept_tls_connection, build_reloadable_server_config, read_file_with_limit_async,
};
use rustls::pki_types::{CertificateDer, pem::PemObject};
use sha1::{Digest, Sha1};
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use x509_parser::prelude::{FromDer, X509Certificate};

/// TLS acceptor and its startup CA snapshot for delivery certificate selection.
#[derive(Clone)]
pub struct SourceAcceptor {
    acceptor: TlsAcceptor,
    client_cas: Arc<[CertificateDer<'static>]>,
}

impl SourceAcceptor {
    /// Find the issuing CA of an already verified peer leaf, including supplied intermediates.
    pub(super) fn issuer_thumbprint(&self, peer: &[CertificateDer<'_>]) -> io::Result<[u8; 20]> {
        issuer_thumbprint(peer, &self.client_cas)
    }
}

fn issuer_thumbprint(
    peer: &[CertificateDer<'_>],
    client_cas: &[CertificateDer<'_>],
) -> io::Result<[u8; 20]> {
    let leaf = peer
        .first()
        .ok_or_else(|| invalid_input("missing verified peer certificate"))?;
    let (_, leaf) = X509Certificate::from_der(leaf.as_ref()).map_err(invalid_input)?;
    let mut selected = None;
    for candidate in peer.iter().skip(1).chain(client_cas) {
        let (remaining, issuer) =
            X509Certificate::from_der(candidate.as_ref()).map_err(invalid_input)?;
        if !remaining.is_empty()
            || !issuer.is_ca()
            || leaf.issuer() != issuer.subject()
            || leaf.verify_signature(Some(issuer.public_key())).is_err()
        {
            continue;
        }
        let thumbprint: [u8; 20] = Sha1::digest(candidate.as_ref()).into();
        if selected.is_some_and(|previous| previous != thumbprint) {
            return Err(invalid_input(
                "ambiguous client certificate issuer for delivery",
            ));
        }
        selected = Some(thumbprint);
    }
    selected.ok_or_else(|| invalid_input("client certificate issuer unavailable for delivery"))
}

/// Build an HTTP/1.1 acceptor requiring certificates from configured client CAs.
///
/// Client CA bundles are loaded at startup; trust changes require a restart.
/// The process must install its selected rustls crypto provider first.
pub async fn build_acceptor(config: &Config) -> Result<SourceAcceptor, io::Error> {
    config.validate().map_err(invalid_input)?;
    let mut client_ca_pem = String::new();
    for path in &config.tls.client_ca_files {
        let bytes = read_file_with_limit_async(path).await?;
        let pem = std::str::from_utf8(&bytes).map_err(invalid_input)?;
        if client_ca_pem
            .len()
            .saturating_add(pem.len())
            .saturating_add(1)
            > 4 * 1024 * 1024
        {
            return Err(invalid_input("combined client CA bundles exceed 4 MiB"));
        }
        client_ca_pem.push_str(pem);
        client_ca_pem.push('\n');
    }
    let client_cas = CertificateDer::pem_slice_iter(client_ca_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(invalid_input)?;
    let server_config = TlsServerConfig {
        config: TlsConfig {
            cert_file: Some(config.tls.cert_file.clone()),
            key_file: Some(config.tls.key_file.clone()),
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
        client_cas: client_cas.into(),
    })
}

/// Complete the bounded TLS handshake before authorizing source identity.
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
    use otel_arrow_dfe_test_tls_certs::{ExtendedKeyUsage, GeneratedCert, generate_ca};
    use rustls::{
        ClientConfig, RootCertStore,
        pki_types::{CertificateDer, PrivateKeyDer, ServerName, pem::PemObject},
    };
    use tokio_rustls::TlsConnector;

    /// Scenario: a verified leaf's issuer is in a CA bundle or peer chain beside a same-name wrong key.
    /// Guarantees: selection checks signatures, survives leaf renewal, and never substitutes a leaf hash.
    #[test]
    fn selects_issuer_thumbprint() {
        let ca = generate_ca("issuer");
        let other = generate_ca("issuer");
        let issuer = CertificateDer::from_pem_slice(ca.cert_pem.as_bytes()).unwrap();
        let wrong_issuer = CertificateDer::from_pem_slice(other.cert_pem.as_bytes()).unwrap();
        let expected: [u8; 20] = Sha1::digest(issuer.as_ref()).into();
        for _ in 0..2 {
            let leaf = ca.issue_leaf(
                "host",
                Some("host.example.com"),
                Some(ExtendedKeyUsage::ClientAuth),
            );
            let leaf = CertificateDer::from_pem_slice(leaf.cert_pem.as_bytes()).unwrap();
            assert!(
                issuer_thumbprint(
                    std::slice::from_ref(&leaf),
                    std::slice::from_ref(&wrong_issuer)
                )
                .is_err()
            );
            assert!(issuer_thumbprint(std::slice::from_ref(&leaf), &[]).is_err());
            assert_eq!(
                issuer_thumbprint(
                    std::slice::from_ref(&leaf),
                    &[wrong_issuer.clone(), issuer.clone()]
                )
                .unwrap(),
                expected
            );
            assert_eq!(
                issuer_thumbprint(&[leaf, issuer.clone()], std::slice::from_ref(&issuer)).unwrap(),
                expected
            );
        }
    }

    fn connector(server_ca: &str, client: Option<&GeneratedCert>) -> TlsConnector {
        let mut roots = RootCertStore::empty();
        roots
            .add(CertificateDer::from_pem_slice(server_ca.as_bytes()).unwrap())
            .unwrap();
        let builder = ClientConfig::builder().with_root_certificates(roots);
        let mut client_config = match client {
            Some(client) => builder
                .with_client_auth_cert(
                    vec![CertificateDer::from_pem_slice(client.cert_pem.as_bytes()).unwrap()],
                    PrivateKeyDer::from_pem_slice(client.key_pem.as_bytes()).unwrap(),
                )
                .unwrap(),
            None => builder.with_no_client_auth(),
        };
        client_config.alpn_protocols = vec![b"http/1.1".to_vec()];
        TlsConnector::from(Arc::new(client_config))
    }

    /// Scenario: clients present trusted, missing, untrusted, or unauthorized certificates.
    /// Guarantees: only a verified, allowed DNS identity reaches the HTTP/1.1 transport.
    #[tokio::test]
    async fn mutual_tls_authentication() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let directory = tempfile::tempdir().unwrap();
        let ca = generate_ca("WEF test CA");
        ca.write_cert_to_dir(directory.path(), "ca");
        ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        )
        .write_to_dir(directory.path(), "server");
        let config: Config = serde_json::from_value(serde_json::json!({
            "endpoint": "127.0.0.1:0",
            "tls": {"cert_file": directory.path().join("server.crt"),
                "key_file": directory.path().join("server.key"), "client_ca_files": [directory.path().join("ca.crt")]},
            "auth": {"allowed_sources": ["host.example.com"]},
            "subscriptions": [{"name": "test", "query": "<QueryList/>"}]
        })).unwrap();
        let acceptor = build_acceptor(&config).await.unwrap();
        let trusted = ca.issue_leaf(
            "host",
            Some("HOST.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let unauthorized = ca.issue_leaf(
            "other",
            Some("other.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let wrong_ca = generate_ca("untrusted");
        let untrusted = wrong_ca.issue_leaf(
            "host",
            Some("host.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let wrong_usage = ca.issue_leaf(
            "host",
            Some("host.example.com"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        for (client, accepted) in [
            (Some(&trusted), true),
            (None, false),
            (Some(&unauthorized), false),
            (Some(&untrusted), false),
            (Some(&wrong_usage), false),
        ] {
            let connector = connector(&ca.cert_pem, client);
            let (server_io, client_io) = tokio::io::duplex(64 * 1024);
            let (server, _) = tokio::join!(
                accept_source(server_io, &acceptor, &config),
                connector.connect(ServerName::try_from("localhost").unwrap(), client_io),
            );
            assert_eq!(server.is_ok(), accepted);
            if let Ok((stream, identity)) = server {
                assert_eq!(identity.as_str(), "host.example.com");
                assert_eq!(
                    stream.get_ref().1.alpn_protocol(),
                    Some(b"http/1.1".as_slice())
                );
            }
        }
    }
}
