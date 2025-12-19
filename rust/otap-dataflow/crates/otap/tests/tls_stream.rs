// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

#[cfg(feature = "experimental-tls")]
mod tests {
    use futures::StreamExt;
    use otap_df_otap::tls_utils::create_tls_stream;
    use rustls_pki_types::pem::PemObject;
    use rustls_pki_types::{CertificateDer, PrivateKeyDer};
    use std::fs;
    use std::io;
    use std::process::Command;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio_rustls::TlsAcceptor;
    use tokio_stream::wrappers::TcpListenerStream;

    fn generate_ca(dir: &std::path::Path, name: &str, cn: &str) {
        let status = Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-keyout",
                &format!("{}.key", name),
                "-out",
                &format!("{}.crt", name),
                "-days",
                "1",
                "-nodes",
                "-subj",
                &format!("/CN={}", cn),
                "-addext",
                "basicConstraints=critical,CA:TRUE",
                "-addext",
                "keyUsage=critical,keyCertSign,cRLSign",
            ])
            .current_dir(dir)
            .output()
            .expect("Failed to generate CA");
        if !status.status.success() {
            panic!("CA gen failed: {}", String::from_utf8_lossy(&status.stderr));
        }
    }

    fn generate_server_cert(dir: &std::path::Path, name: &str, ca_name: &str, cn: &str) {
        let status = Command::new("openssl")
            .args([
                "req",
                "-newkey",
                "rsa:2048",
                "-keyout",
                &format!("{}.key", name),
                "-out",
                &format!("{}.csr", name),
                "-nodes",
                "-subj",
                &format!("/CN={}", cn),
            ])
            .current_dir(dir)
            .output()
            .expect("Failed to generate CSR");
        if !status.status.success() {
            panic!(
                "CSR gen failed: {}",
                String::from_utf8_lossy(&status.stderr)
            );
        }

        let ext_file = dir.join(format!("{}.ext", name));
        fs::write(&ext_file, "subjectAltName=DNS:localhost,IP:127.0.0.1")
            .expect("Failed to write extension file");

        let status = Command::new("openssl")
            .args([
                "x509",
                "-req",
                "-in",
                &format!("{}.csr", name),
                "-CA",
                &format!("{}.crt", ca_name),
                "-CAkey",
                &format!("{}.key", ca_name),
                "-CAcreateserial",
                "-out",
                &format!("{}.crt", name),
                "-days",
                "1",
                "-extfile",
                ext_file.to_str().expect("Invalid UTF-8 path"),
            ])
            .current_dir(dir)
            .output()
            .expect("Failed to sign cert");
        if !status.status.success() {
            panic!("Sign failed: {}", String::from_utf8_lossy(&status.stderr));
        }
    }

    fn load_server_config(
        cert_path: &std::path::Path,
        key_path: &std::path::Path,
    ) -> Arc<rustls::ServerConfig> {
        let cert_pem = fs::read(cert_path).expect("Failed to read cert file");
        let key_pem = fs::read(key_path).expect("Failed to read key file");

        let certs = CertificateDer::pem_reader_iter(&mut io::BufReader::new(&cert_pem[..]))
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to parse certs");
        let key = PrivateKeyDer::from_pem_reader(&mut io::BufReader::new(&key_pem[..]))
            .expect("Failed to parse key");

        let mut config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("Failed to build server config");
        config.alpn_protocols = vec![b"h2".to_vec()];
        Arc::new(config)
    }

    #[tokio::test]
    async fn test_tls_stream_success() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        generate_ca(path, "ca", "Test CA");
        generate_server_cert(path, "server", "ca", "localhost");
        let server_config = load_server_config(&path.join("server.crt"), &path.join("server.key"));
        let acceptor = TlsAcceptor::from(server_config);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let listener_stream = TcpListenerStream::new(listener);

        let tls_stream = create_tls_stream(listener_stream, acceptor, None);
        let mut tls_stream = Box::pin(tls_stream);

        // Spawn server loop
        let server_handle = tokio::spawn(async move {
            if let Some(Ok(mut stream)) = tls_stream.next().await {
                let mut buf = [0u8; 5];
                assert_eq!(stream.read_exact(&mut buf).await.expect("read failed"), 5);
                assert_eq!(&buf, b"hello");
                stream.write_all(b"world").await.expect("write failed");
            } else {
                panic!("Expected a stream item");
            }
        });

        // Client connect
        let mut root_store = rustls::RootCertStore::empty();
        let ca_pem = fs::read(path.join("ca.crt")).unwrap();
        for cert in CertificateDer::pem_reader_iter(&mut io::BufReader::new(&ca_pem[..])) {
            root_store.add(cert.unwrap()).unwrap();
        }
        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let stream = TcpStream::connect(addr).await.unwrap();
        let domain = rustls::pki_types::ServerName::try_from("localhost").unwrap();
        let mut stream = connector
            .connect(domain, stream)
            .await
            .expect("connect failed");

        stream.write_all(b"hello").await.expect("write failed");
        let mut buf = [0u8; 5];
        assert_eq!(stream.read_exact(&mut buf).await.expect("read failed"), 5);
        assert_eq!(&buf, b"world");

        server_handle.await.expect("server task failed");
    }

    #[tokio::test]
    async fn test_tls_stream_handshake_failure_filtered() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        generate_ca(path, "ca", "Test CA");
        generate_server_cert(path, "server", "ca", "localhost");
        let server_config = load_server_config(&path.join("server.crt"), &path.join("server.key"));
        let acceptor = TlsAcceptor::from(server_config);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let listener_stream = TcpListenerStream::new(listener);

        let tls_stream = create_tls_stream(listener_stream, acceptor, None);
        let mut tls_stream = Box::pin(tls_stream);

        // Spawn server loop
        let server_handle = tokio::spawn(async move {
            // The first connection (garbage) should be filtered out.
            // The second connection (valid) should be accepted.
            if let Some(Ok(mut stream)) = tls_stream.next().await {
                stream.write_all(b"success").await.expect("write failed");
            } else {
                panic!("Expected a stream item");
            }
        });

        // 1. Connect with garbage (Handshake failure)
        let mut stream = TcpStream::connect(addr).await.expect("connect failed");
        stream.write_all(b"NOT TLS").await.expect("write failed");
        // We expect the server to close this connection or ignore it, but NOT yield it.
        // We can't easily assert "not yielded" without a timeout or sending a second valid request.
        // So we send a second valid request.

        // Give server time to process and reject/filter the first one
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 2. Connect with valid client
        let mut root_store = rustls::RootCertStore::empty();
        let ca_pem = fs::read(path.join("ca.crt")).unwrap();
        for cert in CertificateDer::pem_reader_iter(&mut io::BufReader::new(&ca_pem[..])) {
            root_store.add(cert.unwrap()).unwrap();
        }
        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let stream = TcpStream::connect(addr).await.unwrap();
        let domain = rustls::pki_types::ServerName::try_from("localhost").unwrap();
        let mut stream = connector
            .connect(domain, stream)
            .await
            .expect("connect failed");

        let mut buf = [0u8; 7];
        assert_eq!(stream.read_exact(&mut buf).await.expect("read failed"), 7);
        assert_eq!(&buf, b"success");

        server_handle.await.expect("server task failed");
    }

    #[tokio::test]
    async fn test_tls_stream_transport_error_propagated() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        generate_ca(path, "ca", "Test CA");
        generate_server_cert(path, "server", "ca", "localhost");
        let server_config = load_server_config(&path.join("server.crt"), &path.join("server.key"));
        let acceptor = TlsAcceptor::from(server_config);

        // Create a mock stream that yields an error
        let mock_stream = futures::stream::iter(vec![Err(io::Error::new(
            io::ErrorKind::ConnectionReset,
            "Simulated transport error",
        ))]);

        // We need to hint the type T for the mock stream.
        // Since we are passing Err, T is never instantiated, but the type system needs it.
        // We can use tokio::io::DuplexStream as a dummy T.
        let mock_stream = mock_stream.map(|res| {
            res.map(|_: tokio::io::DuplexStream| -> tokio::io::DuplexStream { unreachable!() })
        });

        let tls_stream = create_tls_stream(mock_stream, acceptor, None);
        let mut tls_stream = Box::pin(tls_stream);

        let item = tls_stream.next().await;
        assert!(item.is_some());
        let res = item.unwrap();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::ConnectionReset);
    }

    /// Verifies that handshake_timeout parameter is enforced.
    /// This tests the DoS protection feature where slow/malicious clients
    /// that don't complete the TLS handshake are timed out.
    #[tokio::test]
    #[cfg_attr(
        target_os = "macos",
        ignore = "Skipping on macOS due to flakiness. See https://github.com/open-telemetry/otel-arrow/issues/1614"
    )]
    async fn test_handshake_respects_timeout() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        generate_ca(path, "ca", "Test CA");
        generate_server_cert(path, "server", "ca", "localhost");
        let server_config = load_server_config(&path.join("server.crt"), &path.join("server.key"));
        let acceptor = TlsAcceptor::from(server_config);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let listener_stream = TcpListenerStream::new(listener);

        // Use a very short timeout (100ms) to make the test fast
        let short_timeout = Some(Duration::from_millis(100));
        let tls_stream = create_tls_stream(listener_stream, acceptor, short_timeout);
        let mut tls_stream = Box::pin(tls_stream);

        // Track when we started
        let start = std::time::Instant::now();

        // Spawn server that waits for connections
        let server_handle = tokio::spawn(async move {
            // The slow client should timeout and be filtered out.
            // The valid client should succeed.
            if let Some(Ok(mut stream)) = tls_stream.next().await {
                stream.write_all(b"success").await.expect("write failed");
            } else {
                panic!("Expected a valid stream after timeout");
            }
        });

        // 1. Connect but DON'T do TLS handshake (simulating slow/malicious client)
        // This client connects at TCP level but never sends TLS ClientHello
        let _slow_client = TcpStream::connect(addr).await.expect("connect failed");
        // Don't send anything - just hold the connection open

        // Wait longer than the timeout to ensure it fires
        tokio::time::sleep(Duration::from_millis(200)).await;

        // 2. Now connect with a valid TLS client
        let mut root_store = rustls::RootCertStore::empty();
        let ca_pem = fs::read(path.join("ca.crt")).unwrap();
        for cert in CertificateDer::pem_reader_iter(&mut io::BufReader::new(&ca_pem[..])) {
            root_store.add(cert.unwrap()).unwrap();
        }
        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let stream = TcpStream::connect(addr).await.unwrap();
        let domain = rustls::pki_types::ServerName::try_from("localhost").unwrap();
        let mut stream = connector
            .connect(domain, stream)
            .await
            .expect("valid client connect failed");

        // Read success message from server
        let mut buf = [0u8; 7];
        assert_eq!(stream.read_exact(&mut buf).await.expect("read failed"), 7);
        assert_eq!(&buf, b"success");

        server_handle.await.expect("server task failed");

        // Verify the test completed in reasonable time (timeout was enforced, not waiting forever)
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(5),
            "Test took too long ({:?}), timeout may not be working",
            elapsed
        );
    }

    /// Verifies that concurrent TLS handshakes don't block each other.
    /// Multiple slow clients should not prevent a fast client from completing.
    /// This tests the buffer_unordered concurrency mechanism.
    #[tokio::test]
    async fn test_concurrent_handshakes_not_blocked() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        generate_ca(path, "ca", "Test CA");
        generate_server_cert(path, "server", "ca", "localhost");
        let server_config = load_server_config(&path.join("server.crt"), &path.join("server.key"));
        let acceptor = TlsAcceptor::from(server_config);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let listener_stream = TcpListenerStream::new(listener);

        // Short timeout so slow clients get cleaned up
        let tls_stream =
            create_tls_stream(listener_stream, acceptor, Some(Duration::from_millis(500)));
        let mut tls_stream = Box::pin(tls_stream);

        // Counter for successful connections
        let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let success_count_clone = success_count.clone();

        // Server accepts multiple connections
        let server_handle = tokio::spawn(async move {
            // We expect at least 1 successful connection (the fast client)
            while let Some(Ok(mut stream)) = tls_stream.next().await {
                let _ = success_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let _ = stream.write_all(b"ok").await;
            }
        });

        // Start multiple "slow" clients that connect but don't complete TLS
        let mut slow_clients = Vec::new();
        for _ in 0..5 {
            let slow = TcpStream::connect(addr).await.expect("slow connect failed");
            slow_clients.push(slow);
        }

        // Give slow clients a moment to be accepted at TCP level
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Now connect with a fast, valid TLS client
        let mut root_store = rustls::RootCertStore::empty();
        let ca_pem = fs::read(path.join("ca.crt")).unwrap();
        for cert in CertificateDer::pem_reader_iter(&mut io::BufReader::new(&ca_pem[..])) {
            root_store.add(cert.unwrap()).unwrap();
        }
        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let start = std::time::Instant::now();
        let stream = TcpStream::connect(addr).await.unwrap();
        let domain = rustls::pki_types::ServerName::try_from("localhost").unwrap();
        let mut stream = connector
            .connect(domain, stream)
            .await
            .expect("fast client should not be blocked by slow clients");

        // Read response
        let mut buf = [0u8; 2];
        let _ = stream.read_exact(&mut buf).await.expect("read failed");
        assert_eq!(&buf, b"ok");

        let elapsed = start.elapsed();

        // Drop slow clients and server
        drop(slow_clients);
        server_handle.abort();

        // The fast client should complete quickly (< 200ms), not waiting for slow client timeouts
        assert!(
            elapsed < Duration::from_millis(200),
            "Fast client took {:?}, should not be blocked by slow clients",
            elapsed
        );

        // At least the fast client succeeded
        assert!(
            success_count.load(std::sync::atomic::Ordering::SeqCst) >= 1,
            "At least one connection should have succeeded"
        );
    }
}
