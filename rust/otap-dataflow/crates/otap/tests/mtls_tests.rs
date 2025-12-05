//! Integration tests for mTLS functionality.

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "experimental-tls")]

use otap_df_config::tls::{TlsConfig, TlsServerConfig};
use otap_df_otap::tls_utils::build_reloadable_server_config;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::fs;
use std::io;
use std::process::Command;
use std::sync::Arc;
use tempfile::TempDir;

/// Check if OpenSSL CLI is available on the system.
fn is_openssl_available() -> bool {
    Command::new("openssl")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Skips the test if OpenSSL is not available.
fn skip_if_no_openssl() -> bool {
    if !is_openssl_available() {
        eprintln!("SKIPPED: OpenSSL CLI not found.");
        true
    } else {
        false
    }
}

/// Generate a self-signed certificate using OpenSSL CLI.
fn generate_cert(dir: &std::path::Path, name: &str, cn: &str, is_ca: bool) {
    let key_out = format!("{}.key", name);
    let cert_out = format!("{}.crt", name);
    let subj = format!("/CN={}", cn);

    let mut final_args = vec![
        "req", "-x509", "-newkey", "rsa:2048", "-keyout", &key_out, "-out", &cert_out, "-days",
        "1", "-nodes", "-subj", &subj, "-addext",
    ];

    let san = format!("subjectAltName=DNS:{}", cn);

    if is_ca {
        final_args.push("basicConstraints=critical,CA:TRUE");
    } else {
        final_args.push("basicConstraints=critical,CA:FALSE");
        final_args.push("-addext");
        final_args.push(&san);
    }

    let output = Command::new("openssl")
        .args(&final_args)
        .current_dir(dir)
        .output()
        .expect("Failed to execute openssl");

    if !output.status.success() {
        panic!(
            "Certificate generation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[tokio::test]
async fn test_mtls_client_cert_verification() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    if skip_if_no_openssl() {
        return;
    }
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // 1. Generate self-signed client certificate (used directly for trust verification)
    generate_cert(path, "client", "Test Client", false);
    let client_cert_path = path.join("client.crt");
    let client_key_path = path.join("client.key");

    // Server Cert (self-signed)
    generate_cert(path, "server", "localhost", false);
    let server_cert_path = path.join("server.crt");
    let server_key_path = path.join("server.key");

    // 3. Configure Server to trust Client Cert
    let config = TlsServerConfig {
        config: TlsConfig {
            cert_file: Some(server_cert_path.clone()),
            key_file: Some(server_key_path),
            cert_pem: None,
            key_pem: None,
            reload_interval: None,
        },
        client_ca_file: Some(client_cert_path.clone()),
        client_ca_pem: None,
        include_system_ca_certs_pool: None,
        handshake_timeout: None,
    };

    let server_config = build_reloadable_server_config(&config)
        .await
        .expect("Failed to build server config");
    let tls_acceptor = tokio_rustls::TlsAcceptor::from(server_config);

    // 4. Start Server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get addr");

    let server_task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("Failed to accept");
        match tls_acceptor.accept(stream).await {
            Ok(_tls_stream) => true,
            Err(e) => {
                log::debug!("Server handshake failed: {}", e);
                false
            }
        }
    });

    // 5. Client Connection (Success Case)
    let mut root_store = rustls::RootCertStore::empty();
    let server_cert_pem = fs::read_to_string(&server_cert_path).expect("Read server cert");
    for cert in CertificateDer::pem_reader_iter(&mut io::BufReader::new(server_cert_pem.as_bytes()))
    {
        root_store
            .add(cert.expect("Parse cert"))
            .expect("Add cert to root store");
    }

    let client_cert_pem = fs::read_to_string(&client_cert_path).expect("Read client cert");
    let client_key_pem = fs::read_to_string(&client_key_path).expect("Read client key");

    let client_certs: Vec<_> =
        CertificateDer::pem_reader_iter(&mut io::BufReader::new(client_cert_pem.as_bytes()))
            .collect::<Result<_, _>>()
            .expect("Parse client certs");
    let client_key =
        PrivateKeyDer::from_pem_reader(&mut io::BufReader::new(client_key_pem.as_bytes()))
            .expect("Parse client key");

    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_cert(client_certs, client_key)
        .expect("Failed to build client config");

    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
    let stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("Failed to connect");
    let domain = rustls::pki_types::ServerName::try_from("localhost").expect("Parse domain");

    let _ = connector
        .connect(domain, stream)
        .await
        .expect("Client handshake failed");

    assert!(
        server_task.await.expect("Server task panicked"),
        "Server should accept valid client cert"
    );
}

#[tokio::test]
async fn test_mtls_missing_client_cert() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let _ = rustls::crypto::ring::default_provider().install_default();
    if skip_if_no_openssl() {
        return;
    }
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path();

    generate_cert(path, "client", "Test Client", false);
    let client_cert_path = path.join("client.crt");

    generate_cert(path, "server", "localhost", false);
    let server_cert_path = path.join("server.crt");
    let server_key_path = path.join("server.key");

    let config = TlsServerConfig {
        config: TlsConfig {
            cert_file: Some(server_cert_path.clone()),
            key_file: Some(server_key_path),
            cert_pem: None,
            key_pem: None,
            reload_interval: None,
        },
        client_ca_file: Some(client_cert_path.clone()),
        client_ca_pem: None,
        include_system_ca_certs_pool: None,
        handshake_timeout: None,
    };

    let server_config = build_reloadable_server_config(&config)
        .await
        .expect("Failed to build server config");
    let tls_acceptor = tokio_rustls::TlsAcceptor::from(server_config);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get addr");

    let server_task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("Failed to accept");
        match tls_acceptor.accept(stream).await {
            Ok(_) => true,
            Err(e) => {
                log::info!("Server correctly rejected client: {}", e);
                false
            }
        }
    });

    let mut root_store = rustls::RootCertStore::empty();
    let server_cert_pem = fs::read_to_string(&server_cert_path).expect("Read server cert");
    for cert in CertificateDer::pem_reader_iter(&mut io::BufReader::new(server_cert_pem.as_bytes()))
    {
        root_store
            .add(cert.expect("Parse cert"))
            .expect("Add cert to root store");
    }

    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
    let stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("Failed to connect");
    let domain = rustls::pki_types::ServerName::try_from("localhost").expect("Parse domain");

    // The TLS handshake might appear to succeed from the client side initially,
    // but the server will reject it. We need to try I/O to see the rejection.
    let handshake_or_io_failed = match connector.connect(domain, stream).await {
        Err(_) => true, // Handshake failed immediately
        Ok(mut tls_stream) => {
            // Try to write and read - this will fail if server rejected us
            let write_result = tls_stream.write_all(b"test").await;
            if write_result.is_err() {
                true
            } else {
                let mut buf = [0u8; 1];
                let read_result = tls_stream.read(&mut buf).await;
                // Connection should be closed/reset by server
                matches!(read_result, Err(_) | Ok(0))
            }
        }
    };

    let server_rejected = !server_task.await.expect("Server task panicked");
    assert!(server_rejected, "Server should reject missing client cert");
    assert!(
        handshake_or_io_failed,
        "Client should detect connection failure"
    );
}

#[tokio::test]
async fn test_mtls_wrong_client_cert() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let _ = rustls::crypto::ring::default_provider().install_default();
    if skip_if_no_openssl() {
        return;
    }
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path();

    generate_cert(path, "trusted_client", "Trusted Client", false);
    let trusted_client_cert_path = path.join("trusted_client.crt");

    generate_cert(path, "untrusted_client", "Untrusted Client", false);
    let untrusted_client_cert_path = path.join("untrusted_client.crt");
    let untrusted_client_key_path = path.join("untrusted_client.key");

    generate_cert(path, "server", "localhost", false);
    let server_cert_path = path.join("server.crt");
    let server_key_path = path.join("server.key");

    let config = TlsServerConfig {
        config: TlsConfig {
            cert_file: Some(server_cert_path.clone()),
            key_file: Some(server_key_path),
            cert_pem: None,
            key_pem: None,
            reload_interval: None,
        },
        client_ca_file: Some(trusted_client_cert_path.clone()),
        client_ca_pem: None,
        include_system_ca_certs_pool: None,
        handshake_timeout: None,
    };

    let server_config = build_reloadable_server_config(&config)
        .await
        .expect("Failed to build server config");
    let tls_acceptor = tokio_rustls::TlsAcceptor::from(server_config);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get addr");

    let server_task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("Failed to accept");
        match tls_acceptor.accept(stream).await {
            Ok(_) => true,
            Err(e) => {
                log::info!("Server correctly rejected untrusted client: {}", e);
                false
            }
        }
    });

    let mut root_store = rustls::RootCertStore::empty();
    let server_cert_pem = fs::read_to_string(&server_cert_path).expect("Read server cert");
    for cert in CertificateDer::pem_reader_iter(&mut io::BufReader::new(server_cert_pem.as_bytes()))
    {
        root_store
            .add(cert.expect("Parse cert"))
            .expect("Add cert to root store");
    }

    let client_cert_pem =
        fs::read_to_string(&untrusted_client_cert_path).expect("Read client cert");
    let client_key_pem = fs::read_to_string(&untrusted_client_key_path).expect("Read client key");

    let client_certs: Vec<_> =
        CertificateDer::pem_reader_iter(&mut io::BufReader::new(client_cert_pem.as_bytes()))
            .collect::<Result<_, _>>()
            .expect("Parse client certs");
    let client_key =
        PrivateKeyDer::from_pem_reader(&mut io::BufReader::new(client_key_pem.as_bytes()))
            .expect("Parse client key");

    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_cert(client_certs, client_key)
        .expect("Failed to build client config");

    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
    let stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("Failed to connect");
    let domain = rustls::pki_types::ServerName::try_from("localhost").expect("Parse domain");

    // The TLS handshake might appear to succeed from the client side initially,
    // but the server will reject it. We need to try I/O to see the rejection.
    let handshake_or_io_failed = match connector.connect(domain, stream).await {
        Err(_) => true, // Handshake failed immediately
        Ok(mut tls_stream) => {
            // Try to write and read - this will fail if server rejected us
            let write_result = tls_stream.write_all(b"test").await;
            if write_result.is_err() {
                true
            } else {
                let mut buf = [0u8; 1];
                let read_result = tls_stream.read(&mut buf).await;
                // Connection should be closed/reset by server
                matches!(read_result, Err(_) | Ok(0))
            }
        }
    };

    let server_rejected = !server_task.await.expect("Server task panicked");
    assert!(
        server_rejected,
        "Server should reject untrusted client cert"
    );
    assert!(
        handshake_or_io_failed,
        "Client should detect connection failure"
    );
}

#[tokio::test]
async fn test_build_server_config_corrupted_pem() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    if skip_if_no_openssl() {
        return;
    }
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path();
    generate_cert(path, "server", "localhost", false);

    let cert_path = path.join("server.crt");
    let key_path = path.join("server.key");

    fs::write(&key_path, "NOT A VALID KEY").expect("Write corrupted key");

    let config = TlsServerConfig {
        config: TlsConfig {
            cert_file: Some(cert_path),
            key_file: Some(key_path),
            cert_pem: None,
            key_pem: None,
            reload_interval: None,
        },
        client_ca_file: None,
        client_ca_pem: None,
        include_system_ca_certs_pool: None,
        handshake_timeout: None,
    };

    let result = build_reloadable_server_config(&config).await;
    assert!(result.is_err(), "Should fail with corrupted key");
}
