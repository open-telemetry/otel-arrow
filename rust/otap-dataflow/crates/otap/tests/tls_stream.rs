// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

#[cfg(feature = "experimental-tls")]
mod tests {
    use futures::StreamExt;
    use otap_df_otap::tls_utils::create_tls_stream;
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
        fs::write(&ext_file, "subjectAltName=DNS:localhost,IP:127.0.0.1").unwrap();

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
                ext_file.to_str().unwrap(),
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
        let cert_pem = fs::read(cert_path).unwrap();
        let key_pem = fs::read(key_path).unwrap();

        let certs = rustls_pemfile::certs(&mut io::BufReader::new(&cert_pem[..]))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let key = rustls_pemfile::private_key(&mut io::BufReader::new(&key_pem[..]))
            .unwrap()
            .unwrap();

        let mut config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .unwrap();
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

        let tls_stream = create_tls_stream(listener_stream, acceptor);
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
        for cert in rustls_pemfile::certs(&mut io::BufReader::new(&ca_pem[..])) {
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

        let tls_stream = create_tls_stream(listener_stream, acceptor);
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
        for cert in rustls_pemfile::certs(&mut io::BufReader::new(&ca_pem[..])) {
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

        let tls_stream = create_tls_stream(mock_stream, acceptor);
        let mut tls_stream = Box::pin(tls_stream);

        let item = tls_stream.next().await;
        assert!(item.is_some());
        let res = item.unwrap();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::ConnectionReset);
    }
}
