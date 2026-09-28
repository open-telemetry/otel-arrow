// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Client CA hot-reload when the CA file is a Kubernetes Secret/ConfigMap volume.

#![cfg(target_os = "linux")]
#![allow(missing_docs)]

use otel_arrow_dfe_config::tls::{TlsConfig, TlsServerConfig};
use otel_arrow_dfe_otap::tls_utils::build_reloadable_server_config;
use otel_arrow_dfe_test_tls_certs::{
    ExtendedKeyUsage, GeneratedCa, GeneratedCert, generate_ca, generate_self_signed_cert,
};
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const CA_CN: &str = "Test Client Root CA";
const RELOAD_TIMEOUT: Duration = Duration::from_secs(10);

/// A directory laid out like a kubelet AtomicWriter volume:
/// `ca.crt -> ..data/ca.crt`, `..data -> ..<generation>/`.
struct AtomicWriterVolume {
    dir: TempDir,
    generation: u32,
}

impl AtomicWriterVolume {
    fn new(ca_pem: &str) -> Self {
        let dir = TempDir::new().expect("create temp dir");
        let ts = Self::ts_name(1);
        fs::create_dir(dir.path().join(&ts)).expect("create timestamped dir");
        fs::write(dir.path().join(&ts).join("ca.crt"), ca_pem).expect("write CA");
        symlink(&ts, dir.path().join("..data")).expect("link ..data");
        symlink("..data/ca.crt", dir.path().join("ca.crt")).expect("link ca.crt");
        Self { dir, generation: 1 }
    }

    fn ts_name(generation: u32) -> String {
        format!("..2026_01_01_00_00_00.{generation}")
    }

    fn ca_path(&self) -> PathBuf {
        self.dir.path().join("ca.crt")
    }

    /// Real file behind `ca.crt` for the current generation.
    fn current_file(&self) -> PathBuf {
        self.dir
            .path()
            .join(Self::ts_name(self.generation))
            .join("ca.crt")
    }

    /// Applies a kubelet update: write a new timestamped dir, swap `..data`, delete the old dir.
    fn update(&mut self, ca_pem: &str) {
        self.swap_in(Some(ca_pem));
        assert_eq!(
            fs::read_to_string(self.ca_path()).expect("read mounted CA"),
            ca_pem
        );
    }

    /// Like `update`, but the new generation has no `ca.crt`, so the mounted path dangles.
    fn update_without_ca(&mut self) {
        self.swap_in(None);
        assert!(fs::metadata(self.ca_path()).is_err());
    }

    fn swap_in(&mut self, ca_pem: Option<&str>) {
        let old_ts = Self::ts_name(self.generation);
        self.generation += 1;
        let new_ts = Self::ts_name(self.generation);
        let dir = self.dir.path();
        fs::create_dir(dir.join(&new_ts)).expect("create timestamped dir");
        if let Some(ca_pem) = ca_pem {
            fs::write(dir.join(&new_ts).join("ca.crt"), ca_pem).expect("write CA");
        }
        symlink(&new_ts, dir.join("..data_tmp")).expect("link ..data_tmp");
        fs::rename(dir.join("..data_tmp"), dir.join("..data")).expect("swap ..data");
        fs::remove_dir_all(dir.join(old_ts)).expect("remove old dir");
    }
}

fn client_leaf(ca: &GeneratedCa) -> GeneratedCert {
    ca.issue_leaf("client", None, Some(ExtendedKeyUsage::ClientAuth))
}

fn client_config(server_cert_pem: &str, client: &GeneratedCert) -> Arc<rustls::ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    for cert in CertificateDer::pem_slice_iter(server_cert_pem.as_bytes()) {
        roots
            .add(cert.expect("parse server cert"))
            .expect("add server cert");
    }
    let chain: Vec<_> = CertificateDer::pem_slice_iter(client.cert_pem.as_bytes())
        .map(|c| c.expect("parse client cert"))
        .collect();
    let key = PrivateKeyDer::from_pem_slice(client.key_pem.as_bytes()).expect("parse client key");
    Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_client_auth_cert(chain, key)
            .expect("build client config"),
    )
}

/// Returns the server-side handshake result (error text on failure).
async fn handshake(
    acceptor: &tokio_rustls::TlsAcceptor,
    client: Arc<rustls::ClientConfig>,
) -> Result<(), String> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local addr");
    let acceptor = acceptor.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor
            .accept(stream)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    });
    let stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let connector = tokio_rustls::TlsConnector::from(client);
    let _ = connector
        .connect("localhost".try_into().expect("server name"), stream)
        .await;
    server.await.expect("server task")
}

/// Retries the handshake until it succeeds or `RELOAD_TIMEOUT` elapses.
async fn wait_until_accepted(
    acceptor: &tokio_rustls::TlsAcceptor,
    client: Arc<rustls::ClientConfig>,
) -> Result<(), String> {
    let deadline = Instant::now() + RELOAD_TIMEOUT;
    loop {
        let result = handshake(acceptor, Arc::clone(&client)).await;
        if result.is_ok() || Instant::now() >= deadline {
            return result;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn build_acceptor(
    ca_path: &Path,
    server: &GeneratedCert,
) -> (TempDir, tokio_rustls::TlsAcceptor) {
    let server_dir = TempDir::new().expect("create temp dir");
    let cert_file = server_dir.path().join("server.crt");
    let key_file = server_dir.path().join("server.key");
    fs::write(&cert_file, &server.cert_pem).expect("write server cert");
    fs::write(&key_file, &server.key_pem).expect("write server key");
    let config = TlsServerConfig {
        config: TlsConfig {
            cert_file: Some(cert_file),
            key_file: Some(key_file),
            cert_pem: None,
            key_pem: None,
            reload_interval: None,
        },
        client_ca_file: Some(ca_path.to_path_buf()),
        client_ca_pem: None,
        include_system_ca_certs_pool: None,
        watch_client_ca: true,
        handshake_timeout: None,
    };
    let server_config = build_reloadable_server_config(&config)
        .await
        .expect("build server config");
    let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
    (server_dir, acceptor)
}

/// Scenario: a client CA mounted from a Secret/ConfigMap rotates to a new CA
/// with the same subject name and a different key.
#[tokio::test]
async fn client_ca_reloads_after_atomic_writer_update() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

    let ca1 = generate_ca(CA_CN);
    let ca2 = generate_ca(CA_CN);
    let server = generate_self_signed_cert("localhost", Some("localhost"), false);
    let client1 = client_config(&server.cert_pem, &client_leaf(&ca1));
    let client2 = client_config(&server.cert_pem, &client_leaf(&ca2));

    let mut volume = AtomicWriterVolume::new(&ca1.cert_pem);
    let (_server_dir, acceptor) = build_acceptor(&volume.ca_path(), &server).await;

    assert_eq!(handshake(&acceptor, Arc::clone(&client1)).await, Ok(()));
    assert_eq!(
        handshake(&acceptor, Arc::clone(&client2)).await,
        Err("invalid peer certificate: BadSignature".to_string())
    );

    volume.update(&format!("{}{}", ca1.cert_pem, ca2.cert_pem));

    // Sanity check: a freshly built verifier trusts the rotated bundle.
    let (_fresh_dir, fresh) = build_acceptor(&volume.ca_path(), &server).await;
    assert_eq!(handshake(&fresh, Arc::clone(&client2)).await, Ok(()));

    assert_eq!(wait_until_accepted(&acceptor, client2).await, Ok(()));
    assert_eq!(handshake(&acceptor, client1).await, Ok(()));
}

/// Scenario: a second rotation lands inside the debounce window of the first reload.
#[tokio::test]
async fn client_ca_reloads_rotations_within_debounce_window() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

    let ca1 = generate_ca(CA_CN);
    let ca2 = generate_ca(CA_CN);
    let ca3 = generate_ca(CA_CN);
    let server = generate_self_signed_cert("localhost", Some("localhost"), false);
    let client2 = client_config(&server.cert_pem, &client_leaf(&ca2));
    let client3 = client_config(&server.cert_pem, &client_leaf(&ca3));

    let mut volume = AtomicWriterVolume::new(&ca1.cert_pem);
    let (_server_dir, acceptor) = build_acceptor(&volume.ca_path(), &server).await;

    volume.update(&ca2.cert_pem);
    assert_eq!(
        wait_until_accepted(&acceptor, Arc::clone(&client2)).await,
        Ok(())
    );

    volume.update(&ca3.cert_pem);
    assert_eq!(wait_until_accepted(&acceptor, client3).await, Ok(()));
    assert!(handshake(&acceptor, client2).await.is_err());
}

/// Scenario: a reload fails, and the file is then fixed without any event on the
/// watched directory (an in-place write inside the timestamped directory).
#[tokio::test]
async fn client_ca_retries_failed_reload_without_new_event() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

    let ca1 = generate_ca(CA_CN);
    let ca2 = generate_ca(CA_CN);
    let server = generate_self_signed_cert("localhost", Some("localhost"), false);
    let client1 = client_config(&server.cert_pem, &client_leaf(&ca1));
    let client2 = client_config(&server.cert_pem, &client_leaf(&ca2));

    let mut volume = AtomicWriterVolume::new(&ca1.cert_pem);
    let (_server_dir, acceptor) = build_acceptor(&volume.ca_path(), &server).await;

    volume.update("not a certificate\n");
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(handshake(&acceptor, Arc::clone(&client1)).await, Ok(()));

    fs::write(volume.current_file(), &ca2.cert_pem).expect("fix CA in place");
    assert_eq!(wait_until_accepted(&acceptor, client2).await, Ok(()));
    assert!(handshake(&acceptor, client1).await.is_err());
}

/// Scenario: the mounted path is missing after an update, and the file then appears
/// without any event on the watched directory.
#[tokio::test]
async fn client_ca_retries_missing_path_without_new_event() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

    let ca1 = generate_ca(CA_CN);
    let ca2 = generate_ca(CA_CN);
    let server = generate_self_signed_cert("localhost", Some("localhost"), false);
    let client1 = client_config(&server.cert_pem, &client_leaf(&ca1));
    let client2 = client_config(&server.cert_pem, &client_leaf(&ca2));

    let mut volume = AtomicWriterVolume::new(&ca1.cert_pem);
    let (_server_dir, acceptor) = build_acceptor(&volume.ca_path(), &server).await;

    volume.update_without_ca();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(handshake(&acceptor, Arc::clone(&client1)).await, Ok(()));

    fs::write(volume.current_file(), &ca2.cert_pem).expect("create CA in place");
    assert_eq!(wait_until_accepted(&acceptor, client2).await, Ok(()));
    assert!(handshake(&acceptor, client1).await.is_err());
}
