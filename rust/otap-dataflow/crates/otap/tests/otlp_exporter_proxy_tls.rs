// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests validating exporter-side proxy + TLS behavior for the OTLP exporter.

#![cfg(feature = "experimental-tls")]
#![allow(missing_docs)]

use bytes::Bytes;
use otap_df_config::tls::{TlsClientConfig, TlsConfig};
use otap_df_otap::otap_grpc::client_settings::GrpcClientSettings;
use otap_df_otap::otap_grpc::otlp::client::LogsServiceClient;
use otap_df_otap::otap_grpc::proxy::ProxyConfig;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_server::{
    LogsService, LogsServiceServer,
};
use prost::Message;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose,
};
use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status};

struct LogsServiceMock {
    sender: mpsc::Sender<()>,
}

#[tonic::async_trait]
impl LogsService for LogsServiceMock {
    async fn export(
        &self,
        _request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        self.sender
            .send(())
            .await
            .map_err(|_| Status::internal("send failed"))?;
        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

fn new_ca() -> (Certificate, Issuer<'static, KeyPair>) {
    let mut params = CertificateParams::new(Vec::default()).expect("empty SAN");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params
        .distinguished_name
        .push(DnType::CommonName, "Test CA");
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);
    let key_pair = KeyPair::generate().expect("ca key");
    let ca = params.self_signed(&key_pair).expect("ca cert");
    let issuer = Issuer::new(params, key_pair);
    (ca, issuer)
}

fn new_leaf(
    cn: &str,
    san: &str,
    eku: ExtendedKeyUsagePurpose,
    issuer: &Issuer<'_, KeyPair>,
) -> (String, String) {
    let mut params = CertificateParams::new(vec![san.to_string()]).expect("SAN");
    params.distinguished_name.push(DnType::CommonName, cn);
    params.use_authority_key_identifier_extension = true;
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.extended_key_usages.push(eku);
    let key_pair = KeyPair::generate().expect("leaf key");
    let cert = params.signed_by(&key_pair, issuer).expect("leaf cert");
    (cert.pem(), key_pair.serialize_pem())
}

async fn start_connect_proxy(target_hits: Arc<AtomicUsize>) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind failed");
    let addr = listener.local_addr().expect("local_addr failed");

    let _proxy_task = tokio::spawn(async move {
        loop {
            let (mut downstream, _) = match listener.accept().await {
                Ok(v) => v,
                Err(_) => return,
            };

            let target_hits = target_hits.clone();
            let _conn_task = tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let mut read = 0usize;

                // Read until end of headers.
                loop {
                    let n = downstream.read(&mut buf[read..]).await.unwrap_or(0);
                    if n == 0 {
                        return;
                    }
                    read += n;
                    if read >= 4 && buf[..read].windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                    if read == buf.len() {
                        return;
                    }
                }

                let headers = String::from_utf8_lossy(&buf[..read]);
                let mut lines = headers.lines();
                let request_line = lines.next().unwrap_or("");

                // CONNECT host:port HTTP/1.1
                let mut parts = request_line.split_whitespace();
                let method = parts.next().unwrap_or("");
                let authority = parts.next().unwrap_or("");
                let version = parts.next().unwrap_or("");

                if method != "CONNECT" {
                    let _ = downstream
                        .write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n")
                        .await;
                    return;
                }

                if version != "HTTP/1.1" && version != "HTTP/1.0" {
                    let _ = downstream
                        .write_all(b"HTTP/1.1 505 HTTP Version Not Supported\r\n\r\n")
                        .await;
                    return;
                }

                let (host, port) = match authority.rsplit_once(':') {
                    Some((h, p)) => (h, p),
                    None => {
                        let _ = downstream
                            .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                            .await;
                        return;
                    }
                };

                let port: u16 = match port.parse() {
                    Ok(p) => p,
                    Err(_) => {
                        let _ = downstream
                            .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                            .await;
                        return;
                    }
                };

                let mut upstream = match TcpStream::connect((host, port)).await {
                    Ok(s) => s,
                    Err(_) => {
                        let _ = downstream
                            .write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")
                            .await;
                        return;
                    }
                };

                let _ = downstream
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await;

                let _ = target_hits.fetch_add(1, Ordering::Relaxed);

                let _ = tokio::io::copy_bidirectional(&mut downstream, &mut upstream).await;
            });
        }
    });

    addr
}

async fn start_tls_logs_server() -> (
    SocketAddr,
    String,
    String,
    String,
    tokio::task::JoinHandle<()>,
    mpsc::Receiver<()>,
) {
    if let Err(err) = rustls::crypto::ring::default_provider().install_default() {
        // It's fine if the provider is already installed (e.g. by another test)
        log::debug!("rustls default provider installation failed in test: {err:?}");
    }

    let (ca, ca_issuer) = new_ca();
    let ca_pem = ca.pem();
    let (server_cert_pem, server_key_pem) = new_leaf(
        "localhost",
        "localhost",
        ExtendedKeyUsagePurpose::ServerAuth,
        &ca_issuer,
    );
    let (client_cert_pem, client_key_pem) = new_leaf(
        "client",
        "client",
        ExtendedKeyUsagePurpose::ClientAuth,
        &ca_issuer,
    );

    let (tx, rx) = mpsc::channel::<()>(8);
    let logs_service = LogsServiceServer::new(LogsServiceMock { sender: tx });

    let server_identity = Identity::from_pem(server_cert_pem.as_bytes(), server_key_pem.as_bytes());
    let client_ca_root = tonic::transport::Certificate::from_pem(ca_pem.as_bytes());

    let tls = ServerTlsConfig::new()
        .identity(server_identity)
        .client_ca_root(client_ca_root);

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind failed");
    let addr: SocketAddr = listener.local_addr().expect("local_addr failed");
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

    let handle = tokio::spawn(async move {
        Server::builder()
            .tls_config(tls)
            .expect("tls_config failed")
            .add_service(logs_service)
            .serve_with_incoming(incoming)
            .await
            .expect("serve failed");
    });

    (addr, ca_pem, client_cert_pem, client_key_pem, handle, rx)
}

async fn send_one_request(channel: tonic::transport::Channel) {
    let mut client = LogsServiceClient::new(channel);
    let req = ExportLogsServiceRequest {
        resource_logs: Vec::new(),
    };

    let mut buf = Vec::new();
    req.encode(&mut buf).expect("encode failed");
    let _ = client
        .export(Bytes::from(buf))
        .await
        .expect("export failed");
}

#[tokio::test]
async fn otlp_exporter_connects_through_connect_proxy_eager() {
    let (srv_addr, ca_pem, client_cert_pem, client_key_pem, server_handle, mut rx) =
        start_tls_logs_server().await;

    let proxy_hits = Arc::new(AtomicUsize::new(0));
    let proxy_addr = start_connect_proxy(proxy_hits.clone()).await;

    let settings = GrpcClientSettings {
        grpc_endpoint: format!("https://localhost:{}", srv_addr.port()),
        tls: Some(TlsClientConfig {
            config: TlsConfig {
                cert_file: None,
                cert_pem: Some(client_cert_pem),
                key_file: None,
                key_pem: Some(client_key_pem),
                reload_interval: None,
            },
            ca_file: None,
            ca_pem: Some(ca_pem),
            include_system_ca_certs_pool: Some(false),
            server_name: Some("localhost".to_string()),
            ..TlsClientConfig::default()
        }),
        proxy: Some(ProxyConfig {
            https_proxy: Some(format!("http://{}:{}", proxy_addr.ip(), proxy_addr.port()).into()),
            ..ProxyConfig::default()
        }),
        ..GrpcClientSettings::default()
    };

    let channel = settings.connect_channel(None).await.unwrap();
    send_one_request(channel).await;

    let observed = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
        .await
        .unwrap();
    assert!(observed.is_some());

    assert!(proxy_hits.load(Ordering::Relaxed) >= 1);

    server_handle.abort();
}

#[tokio::test]
async fn otlp_exporter_connects_through_connect_proxy_lazy() {
    let (srv_addr, ca_pem, client_cert_pem, client_key_pem, server_handle, mut rx) =
        start_tls_logs_server().await;

    let proxy_hits = Arc::new(AtomicUsize::new(0));
    let proxy_addr = start_connect_proxy(proxy_hits.clone()).await;

    let settings = GrpcClientSettings {
        grpc_endpoint: format!("https://localhost:{}", srv_addr.port()),
        tls: Some(TlsClientConfig {
            config: TlsConfig {
                cert_file: None,
                cert_pem: Some(client_cert_pem),
                key_file: None,
                key_pem: Some(client_key_pem),
                reload_interval: None,
            },
            ca_file: None,
            ca_pem: Some(ca_pem),
            include_system_ca_certs_pool: Some(false),
            server_name: Some("localhost".to_string()),
            ..TlsClientConfig::default()
        }),
        proxy: Some(ProxyConfig {
            https_proxy: Some(format!("http://{}:{}", proxy_addr.ip(), proxy_addr.port()).into()),
            ..ProxyConfig::default()
        }),
        ..GrpcClientSettings::default()
    };

    let channel = settings.connect_channel_lazy(None).await.unwrap();
    send_one_request(channel).await;

    let observed = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
        .await
        .unwrap();
    assert!(observed.is_some());

    assert!(proxy_hits.load(Ordering::Relaxed) >= 1);

    server_handle.abort();
}

async fn start_plain_logs_server() -> (SocketAddr, tokio::task::JoinHandle<()>, mpsc::Receiver<()>)
{
    let (tx, rx) = mpsc::channel::<()>(8);
    let logs_service = LogsServiceServer::new(LogsServiceMock { sender: tx });

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind failed");
    let addr: SocketAddr = listener.local_addr().expect("local_addr failed");
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(logs_service)
            .serve_with_incoming(incoming)
            .await
            .expect("serve failed");
    });

    (addr, handle, rx)
}

#[tokio::test]
async fn otlp_exporter_connects_through_connect_proxy_http_endpoint() {
    let (srv_addr, server_handle, mut rx) = start_plain_logs_server().await;

    let proxy_hits = Arc::new(AtomicUsize::new(0));
    let proxy_addr = start_connect_proxy(proxy_hits.clone()).await;

    let settings = GrpcClientSettings {
        grpc_endpoint: format!("http://localhost:{}", srv_addr.port()),
        proxy: Some(ProxyConfig {
            http_proxy: Some(format!("http://{}:{}", proxy_addr.ip(), proxy_addr.port()).into()),
            ..ProxyConfig::default()
        }),
        ..GrpcClientSettings::default()
    };

    let channel = settings.connect_channel(None).await.unwrap();
    send_one_request(channel).await;

    let observed = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
        .await
        .unwrap();
    assert!(observed.is_some());

    assert!(
        proxy_hits.load(Ordering::Relaxed) >= 1,
        "Expected proxy to be used for http:// endpoint"
    );

    server_handle.abort();
}

#[tokio::test]
async fn otlp_exporter_respects_no_proxy() {
    let (srv_addr, server_handle, mut rx) = start_plain_logs_server().await;

    let proxy_hits = Arc::new(AtomicUsize::new(0));
    let proxy_addr = start_connect_proxy(proxy_hits.clone()).await;

    let settings = GrpcClientSettings {
        grpc_endpoint: format!("http://localhost:{}", srv_addr.port()),
        proxy: Some(ProxyConfig {
            http_proxy: Some(format!("http://{}:{}", proxy_addr.ip(), proxy_addr.port()).into()),
            // Should bypass proxy for localhost
            no_proxy: Some("localhost".to_string()),
            ..ProxyConfig::default()
        }),
        ..GrpcClientSettings::default()
    };

    let channel = settings.connect_channel(None).await.unwrap();
    send_one_request(channel).await;

    let observed = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
        .await
        .unwrap();
    assert!(observed.is_some());

    assert_eq!(
        proxy_hits.load(Ordering::Relaxed),
        0,
        "Expected proxy to be bypassed"
    );

    server_handle.abort();
}
