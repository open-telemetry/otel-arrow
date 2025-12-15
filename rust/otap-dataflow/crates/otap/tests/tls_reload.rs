// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]
#![allow(unused_results)]

#[cfg(feature = "experimental-tls")]
mod tests {
    use otap_df_config::tls::{TlsConfig, TlsServerConfig};
    use otap_df_otap::tls_utils::build_reloadable_server_config;
    use rustls_pki_types::CertificateDer;
    use rustls_pki_types::pem::PemObject;
    use std::fs;
    use std::io::BufReader;
    use std::net::SocketAddr;
    use std::process::Command;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio_rustls::TlsConnector;

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

    async fn start_server(
        config: TlsServerConfig,
        listener: tokio::net::TcpListener,
    ) -> tokio::task::JoinHandle<()> {
        let server_config = build_reloadable_server_config(&config)
            .await
            .expect("Failed to build server config");
        let acceptor = tokio_rustls::TlsAcceptor::from(server_config);

        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.expect("Accept failed");
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    if let Ok(mut stream) = acceptor.accept(stream).await {
                        let mut buf = [0; 1024];
                        if let Ok(n) = stream.read(&mut buf).await {
                            // Ignore write errors in test server
                            let _ = stream.write_all(&buf[..n]).await;
                        }
                    }
                });
            }
        })
    }

    #[tokio::test]
    async fn test_tls_reload_integration() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        let cert_path = path.join("server.crt");
        let key_path = path.join("server.key");

        // 1. Generate CA1 and Server1
        generate_ca(path, "ca1", "Test CA 1");
        generate_server_cert(path, "server1", "ca1", "localhost");

        // 2. Start Server with Server1
        fs::copy(path.join("server1.crt"), &cert_path).expect("copy cert failed");
        fs::copy(path.join("server1.key"), &key_path).expect("copy key failed");

        let config = TlsServerConfig {
            config: TlsConfig {
                cert_file: Some(cert_path.clone()),
                key_file: Some(key_path.clone()),
                reload_interval: Some(Duration::from_secs(1)),
                cert_pem: None,
                key_pem: None,
            },
            client_ca_file: None,
            client_ca_pem: None,
            include_system_ca_certs_pool: None,
            handshake_timeout: None,
            watch_client_ca: false,
        };

        let addr: SocketAddr = "127.0.0.1:0".parse().expect("Invalid address");
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Bind failed");
        let local_addr = listener.local_addr().expect("Failed to get local addr");

        let server_handle = start_server(config, listener).await;

        // 3. Connect with Client trusting CA1 (Should Succeed)
        let mut root_store1 = rustls::RootCertStore::empty();
        let ca1_pem = fs::read(path.join("ca1.crt")).unwrap();
        for cert in CertificateDer::pem_reader_iter(&mut BufReader::new(&ca1_pem[..])) {
            root_store1.add(cert.unwrap()).unwrap();
        }

        let client_config1 = rustls::ClientConfig::builder()
            .with_root_certificates(root_store1)
            .with_no_client_auth();
        let connector1 = TlsConnector::from(Arc::new(client_config1));

        let stream = TcpStream::connect(local_addr).await.unwrap();
        let domain = rustls::pki_types::ServerName::try_from("localhost").unwrap();
        let _stream = connector1
            .connect(domain.clone(), stream)
            .await
            .expect("Handshake with CA1 failed");

        // 4. Rotate to Server2 (signed by CA2)
        tokio::time::sleep(Duration::from_secs(3)).await; // Wait for reload interval

        // Generate CA2 and Server2 now to ensure mtime is different
        generate_ca(path, "ca2", "Test CA 2");
        generate_server_cert(path, "server2", "ca2", "localhost");

        fs::copy(path.join("server2.crt"), &cert_path).expect("copy cert failed");
        fs::copy(path.join("server2.key"), &key_path).expect("copy key failed");

        // 5. Trigger Reload by making a connection (async reload happens in background)
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Make a dummy connection to trigger the reload check
        // This connection will use the old cert but spawn async reload
        let stream = TcpStream::connect(local_addr).await.unwrap();
        let _ = connector1.connect(domain.clone(), stream).await; // May succeed or fail, doesn't matter

        // Wait for async reload to complete
        tokio::time::sleep(Duration::from_millis(200)).await;

        // 6. Connect with Client trusting CA2 (Should Succeed)
        let mut root_store2 = rustls::RootCertStore::empty();
        let ca2_pem = fs::read(path.join("ca2.crt")).unwrap();
        for cert in CertificateDer::pem_reader_iter(&mut BufReader::new(&ca2_pem[..])) {
            root_store2.add(cert.unwrap()).unwrap();
        }

        let client_config2 = rustls::ClientConfig::builder()
            .with_root_certificates(root_store2)
            .with_no_client_auth();
        let connector2 = TlsConnector::from(Arc::new(client_config2));

        let stream = TcpStream::connect(local_addr).await.unwrap();
        let _stream = connector2
            .connect(domain.clone(), stream)
            .await
            .expect("Handshake with CA2 failed (Reload didn't happen?)");

        // 7. Verify Client trusting CA1 now fails
        let stream = TcpStream::connect(local_addr).await.unwrap();
        let result = connector1.connect(domain, stream).await;
        assert!(
            result.is_err(),
            "Handshake with CA1 should fail after reload"
        );

        // Cleanup
        server_handle.abort();
    }
}
