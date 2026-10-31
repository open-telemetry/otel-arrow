// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_engine::error::ReceiverErrorKind;
use otel_arrow_dfe_scraper::database::DriverAdapter;
use otel_arrow_dfe_test_tls_certs::{ExtendedKeyUsage, generate_ca};
use rustls_pki_types::PrivateKeyDer;
use serde_json::json;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::TlsAcceptor;

const SSL_REQUEST: [u8; 8] = [0, 0, 0, 8, 4, 210, 22, 47];
const CANCEL_REQUEST: [u8; 16] = [0, 0, 0, 16, 4, 210, 22, 46, 0, 0, 0, 42, 0, 0, 0, 99];

struct Fixture {
    _directory: tempfile::TempDir,
    ca_file: std::path::PathBuf,
    acceptor: TlsAcceptor,
}

impl Fixture {
    fn new(server_name: &str) -> Self {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let directory = tempfile::tempdir().expect("CA directory");
        let ca = generate_ca("PostgreSQL transport CA");
        let leaf = ca.issue_leaf(
            server_name,
            Some(server_name),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        let ca_file = directory.path().join("ca.pem");
        std::fs::write(&ca_file, ca.cert_pem).expect("CA file");
        let certificates = CertificateDer::pem_slice_iter(leaf.cert_pem.as_bytes())
            .collect::<std::result::Result<Vec<_>, _>>()
            .expect("server certificate");
        let key = PrivateKeyDer::from_pem_slice(leaf.key_pem.as_bytes()).expect("server key");
        let server = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certificates, key)
            .expect("server TLS");
        Self {
            _directory: directory,
            ca_file,
            // Rustls requires shared ownership of immutable configuration; all
            // network futures in these fixtures run on the same local runtime.
            acceptor: TlsAcceptor::from(Arc::new(server)),
        }
    }

    fn adapter(&self, address: SocketAddr, enabled: bool) -> PostgreSqlAdapter {
        let tls = if enabled {
            json!({"ca_file": self.ca_file})
        } else {
            json!({"enabled": false})
        };
        let validated = super::super::tests::test_validated_with_connection(json!({
            "host": address.ip().to_string(),
            "port": address.port(),
            "database": "fixture",
            "tls": tls,
        }));
        PostgreSqlAdapter::new(validated, super::super::tests::test_provider()).expect("adapter")
    }
}

async fn bounded_test(future: impl Future<Output = ()>) {
    tokio::task::LocalSet::new()
        .run_until(async {
            tokio::time::timeout(Duration::from_secs(30), future)
                .await
                .expect("transport scenario completed");
        })
        .await;
}

async fn listen() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("loopback listener");
    let address = listener.local_addr().expect("listener address");
    (listener, address)
}

async fn ssl_request(listener: &TcpListener) -> TcpStream {
    let (mut socket, _) = listener.accept().await.expect("TCP connection");
    let mut request = [0; 8];
    let _ = socket.read_exact(&mut request).await.expect("SSLRequest");
    assert_eq!(request, SSL_REQUEST);
    socket.write_all(b"S").await.expect("TLS supported");
    socket
}

fn frame(tag: u8, body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![tag];
    bytes.extend_from_slice(
        &u32::try_from(body.len() + 4)
            .expect("frame length")
            .to_be_bytes(),
    );
    bytes.extend_from_slice(body);
    bytes
}

async fn read_startup<S: AsyncRead + Unpin>(stream: &mut S) {
    let length = stream.read_u32().await.expect("startup length");
    assert!((8..4096).contains(&length));
    let mut body = vec![0; length as usize - 4];
    let _ = stream.read_exact(&mut body).await.expect("startup body");
    assert_eq!(&body[..4], &196608u32.to_be_bytes());
    assert!(
        body.windows(b"reader".len())
            .any(|bytes| bytes == b"reader")
    );
}

async fn startup<S: AsyncRead + AsyncWrite + Unpin>(stream: &mut S) {
    read_startup(stream).await;
    let mut response = frame(b'R', &0u32.to_be_bytes());
    response.extend(frame(b'K', &CANCEL_REQUEST[8..]));
    response.extend(frame(b'Z', b"I"));
    stream.write_all(&response).await.expect("startup response");
}

async fn terminated<S: AsyncRead + Unpin>(stream: &mut S) {
    let mut message = [0; 5];
    let _ = stream.read_exact(&mut message).await.expect("Terminate");
    assert_eq!(message, [b'X', 0, 0, 0, 4]);
}

/// Scenario: A private-CA PostgreSQL session connects, cancels work over a second verified TLS connection, and closes.
/// Guarantees: Startup and cancellation use TLS, cancellation preserves the backend key, and the driver is joined on shutdown.
#[tokio::test]
async fn verified_tls_startup_cancellation_and_shutdown() {
    bounded_test(async {
        let fixture = Fixture::new("localhost");
        let (listener, address) = listen().await;
        let validated = super::super::tests::test_validated_with_connection(json!({
            "host": "localhost",
            "port": address.port(),
            "database": "fixture",
            "tls": {"ca_file": fixture.ca_file},
        }));
        let mut adapter = PostgreSqlAdapter::new(validated, super::super::tests::test_provider())
            .expect("adapter");
        let server = async {
            let socket = ssl_request(&listener).await;
            let mut session = fixture.acceptor.accept(socket).await.expect("session TLS");
            startup(&mut session).await;
            let socket = ssl_request(&listener).await;
            let mut cancellation = fixture.acceptor.accept(socket).await.expect("cancel TLS");
            let mut request = [0; 16];
            let _ = cancellation
                .read_exact(&mut request)
                .await
                .expect("CancelRequest");
            assert_eq!(request, CANCEL_REQUEST);
            terminated(&mut session).await;
        };
        let client = async {
            adapter
                .connect(
                    &Operation::default(),
                    &super::super::tests::fixed_credential(),
                )
                .await
                .expect("verified connection");
            assert!(
                adapter
                    .cancel
                    .borrow()
                    .as_ref()
                    .expect("cancel token")
                    .tls
                    .is_some()
            );
            cancel_native(&adapter.cancel).await;
            assert_eq!(
                adapter.close_session(CloseReason::Idle).await,
                Ok(SessionClose::Graceful)
            );
            assert!(adapter.session.is_none());
            assert!(adapter.cancel.borrow().is_none());
        };
        tokio::join!(server, client);
    })
    .await;
}

/// Scenario: The server certificate has either an untrusted issuer or a hostname different from the configured endpoint.
/// Guarantees: Both failures are terminal TLS configuration errors and no authenticated session or cancel token is published.
#[tokio::test]
async fn untrusted_and_mismatched_certificates_fail_closed() {
    bounded_test(async {
        for mismatch in [false, true] {
            let trusted = Fixture::new("127.0.0.1");
            let rejected = Fixture::new(if mismatch {
                "wrong.example"
            } else {
                "127.0.0.1"
            });
            let (listener, address) = listen().await;
            let mut adapter = if mismatch {
                rejected.adapter(address, true)
            } else {
                trusted.adapter(address, true)
            };
            let server = async {
                let socket = ssl_request(&listener).await;
                assert!(rejected.acceptor.accept(socket).await.is_err());
            };
            let client = async {
                let error = adapter
                    .connect(
                        &Operation::default(),
                        &super::super::tests::fixed_credential(),
                    )
                    .await
                    .expect_err("certificate rejected");
                assert_eq!(error, Error::TlsCertificate);
                assert!(!PostgreSqlAdapter::is_retryable(&error));
                assert_eq!(
                    PostgreSqlAdapter::classify_error(&error),
                    ReceiverErrorKind::Configuration
                );
                assert!(adapter.session.is_none());
                assert!(adapter.cancel.borrow().is_none());
            };
            tokio::join!(server, client);
        }
    })
    .await;
}

/// Scenario: A PostgreSQL server refuses SSLRequest while TLS is required.
/// Guarantees: The client emits a TLS-specific terminal error and closes without sending plaintext startup credentials.
#[tokio::test]
async fn tls_refusal_does_not_downgrade() {
    bounded_test(async {
        let fixture = Fixture::new("127.0.0.1");
        let (listener, address) = listen().await;
        let mut adapter = fixture.adapter(address, true);
        let server = async {
            let (mut socket, _) = listener.accept().await.expect("connection");
            let mut request = [0; 8];
            let _ = socket.read_exact(&mut request).await.expect("SSLRequest");
            assert_eq!(request, SSL_REQUEST);
            socket.write_all(b"N").await.expect("refuse TLS");
            assert_eq!(socket.read(&mut request).await.expect("client closes"), 0);
        };
        let client = async {
            assert_eq!(
                adapter
                    .connect(
                        &Operation::default(),
                        &super::super::tests::fixed_credential()
                    )
                    .await,
                Err(Error::TlsHandshake)
            );
            assert!(adapter.session.is_none());
        };
        tokio::join!(server, client);
    })
    .await;
}

/// Scenario: TLS is explicitly disabled for a plaintext PostgreSQL endpoint.
/// Guarantees: Startup and cancellation skip SSLRequest and normal local-driver shutdown still completes.
#[tokio::test]
async fn explicit_plaintext_connects_without_ssl_request() {
    bounded_test(async {
        let fixture = Fixture::new("127.0.0.1");
        let (listener, address) = listen().await;
        let mut adapter = fixture.adapter(address, false);
        assert!(
            tls_connector(&adapter.validated.config.connection)
                .await
                .expect("plaintext")
                .is_none()
        );
        let server = async {
            let (mut socket, _) = listener.accept().await.expect("connection");
            startup(&mut socket).await;
            let (mut cancellation, _) = listener.accept().await.expect("cancel connection");
            let mut request = [0; 16];
            let _ = cancellation
                .read_exact(&mut request)
                .await
                .expect("CancelRequest");
            assert_eq!(request, CANCEL_REQUEST);
            terminated(&mut socket).await;
        };
        let client = async {
            adapter
                .connect(
                    &Operation::default(),
                    &super::super::tests::fixed_credential(),
                )
                .await
                .expect("plaintext");
            assert!(
                adapter
                    .cancel
                    .borrow()
                    .as_ref()
                    .expect("cancel token")
                    .tls
                    .is_none()
            );
            cancel_native(&adapter.cancel).await;
            assert_eq!(
                adapter.close_session(CloseReason::Idle).await,
                Ok(SessionClose::Graceful)
            );
        };
        tokio::join!(server, client);
    })
    .await;
}

/// Scenario: A TCP peer accepts SSLRequest and closes before finishing a real Rustls handshake.
/// Guarantees: Interrupted TLS startup is retryable and does not leave a session or cancellation token behind.
#[tokio::test]
async fn interrupted_rustls_handshake_is_retryable() {
    bounded_test(async {
        let fixture = Fixture::new("127.0.0.1");
        let (listener, address) = listen().await;
        let mut adapter = fixture.adapter(address, true);
        let server = async {
            let mut socket = ssl_request(&listener).await;
            let mut hello = [0; 1024];
            assert!(socket.read(&mut hello).await.expect("ClientHello") > 0);
            socket.shutdown().await.expect("interrupt handshake");
        };
        let client = async {
            assert_eq!(
                adapter
                    .connect(
                        &Operation::default(),
                        &super::super::tests::fixed_credential()
                    )
                    .await,
                Err(Error::Unavailable)
            );
            assert!(adapter.session.is_none());
            assert!(adapter.cancel.borrow().is_none());
        };
        tokio::join!(server, client);
    })
    .await;
}

/// Scenario: Multiple backend frames and an oversized DataRow header share an encrypted TLS write.
/// Guarantees: Decrypted reads never cross PostgreSQL frame boundaries and an oversized header is rejected before reaching the driver.
#[tokio::test]
async fn tls_plaintext_framing_remains_bounded() {
    bounded_test(async {
        let fixture = Fixture::new("127.0.0.1");
        let (listener, address) = listen().await;
        let validated = super::super::tests::test_validated_with_ca(&fixture.ca_file);
        let mut tls = tls_connector(&validated.config.connection)
            .await
            .expect("TLS")
            .expect("enabled");
        let limits = FrameLimits::new(1024).expect("limits");
        let first = frame(b'D', b"first row");
        let second = frame(b'D', b"second row");
        let expected = [first.as_slice(), second.as_slice()].concat();
        let mut wire = expected.clone();
        wire.push(b'D');
        wire.extend_from_slice(
            &i32::try_from(limits.data_row)
                .expect("frame ceiling")
                .to_be_bytes(),
        );
        let server = async {
            let (socket, _) = listener.accept().await.expect("connection");
            let mut stream = fixture.acceptor.accept(socket).await.expect("TLS");
            stream.write_all(&wire).await.expect("encrypted frames");
            stream.shutdown().await.expect("server close");
        };
        let client = async {
            let socket = TcpStream::connect(address).await.expect("connection");
            let inner = <MakeRustlsConnect as MakeTlsConnect<TcpStream>>::make_tls_connect(
                &mut tls,
                "127.0.0.1",
            )
            .expect("connector");
            let failure = ReceiveFailure::default();
            let connector = BoundedTlsConnect {
                inner,
                failure: failure.clone(),
                limits,
                handshake_completed: Rc::new(Cell::new(false)),
            };
            let mut stream = connector.connect(socket).await.expect("verified TLS");
            let mut offset = 0;
            let mut buffer = [0; 4096];
            while offset < expected.len() {
                let count = stream.read(&mut buffer).await.expect("frame");
                assert!(count > 0);
                let boundary = if offset < first.len() {
                    first.len()
                } else {
                    expected.len()
                };
                assert!(offset + count <= boundary);
                assert_eq!(&buffer[..count], &expected[offset..offset + count]);
                offset += count;
            }
            buffer.fill(0);
            assert_eq!(
                stream.read(&mut buffer).await.expect_err("oversize").kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(failure.get(), Some(Error::Limit));
            assert_eq!(buffer, [0; 4096]);
        };
        tokio::join!(server, client);
    })
    .await;
}

/// Scenario: The configured CA bundle is replaced between two connections to the same PostgreSQL endpoint.
/// Guarantees: Reconnection reads the new trust bundle and joins both driver tasks without changing the source fingerprint.
#[tokio::test]
async fn reconnect_reloads_custom_ca_file() {
    bounded_test(async {
        let first = Fixture::new("127.0.0.1");
        let second = Fixture::new("127.0.0.1");
        let (listener, address) = listen().await;
        let mut adapter = first.adapter(address, true);
        let fingerprint = adapter.validated.fingerprint.clone();
        let server = async {
            for fixture in [&first, &second] {
                let socket = ssl_request(&listener).await;
                let mut stream = fixture.acceptor.accept(socket).await.expect("TLS");
                startup(&mut stream).await;
                terminated(&mut stream).await;
            }
        };
        let client = async {
            adapter
                .connect(
                    &Operation::default(),
                    &super::super::tests::fixed_credential(),
                )
                .await
                .expect("first CA");
            assert_eq!(
                adapter.close_session(CloseReason::Idle).await,
                Ok(SessionClose::Graceful)
            );
            let _ = std::fs::copy(&second.ca_file, &first.ca_file).expect("rotate CA");
            adapter
                .connect(
                    &Operation::default(),
                    &super::super::tests::fixed_credential(),
                )
                .await
                .expect("rotated CA");
            assert_eq!(adapter.validated.fingerprint, fingerprint);
            assert_eq!(
                adapter.close_session(CloseReason::Idle).await,
                Ok(SessionClose::Graceful)
            );
        };
        tokio::join!(server, client);
    })
    .await;
}
