// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Give an HTTPS-only exporter the same private capture as the HTTP probes.
use anyhow::Result;
use otel_arrow_dfe_test_tls_certs::{ExtendedKeyUsage, generate_ca};
use std::{fs, net::SocketAddr, path::Path, sync::Arc};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
    task::{JoinHandle, JoinSet},
};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    },
};

/// A loopback TLS proxy that forwards decrypted requests to the fixture's HTTP capture server.
pub struct TlsCapture {
    pub url: String,
    task: JoinHandle<()>,
}

impl TlsCapture {
    /// Write a fixture CA for the engine to trust and serve HTTPS until the capture is dropped.
    pub async fn start(dir: &Path, upstream: SocketAddr) -> Result<Self> {
        let ca = generate_ca("DFE offline fixture");
        let leaf = ca.issue_leaf(
            "localhost",
            Some("127.0.0.1"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        fs::write(dir.join("fixture-ca.pem"), ca.cert_pem)?;
        let config = ServerConfig::builder_with_provider(
            otel_arrow_dfe_otap::crypto::selected_crypto_provider(),
        )
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from_pem_slice(leaf.cert_pem.as_bytes())?],
            PrivateKeyDer::from_pem_slice(leaf.key_pem.as_bytes())?,
        )?;
        let acceptor = TlsAcceptor::from(Arc::new(config));
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let url = format!("https://{}", listener.local_addr()?);
        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            while let Ok((socket, _)) = listener.accept().await {
                let acceptor = acceptor.clone();
                let _ = connections.spawn(async move {
                    let mut tls = acceptor.accept(socket).await?;
                    let mut http = TcpStream::connect(upstream).await?;
                    copy_bidirectional(&mut tls, &mut http).await
                });
                while connections.try_join_next().is_some() {}
            }
        });
        Ok(Self { url, task })
    }
}

impl Drop for TlsCapture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
