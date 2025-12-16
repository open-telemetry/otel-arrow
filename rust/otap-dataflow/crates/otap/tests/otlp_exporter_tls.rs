// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration test validating exporter-side TLS/mTLS for the OTLP exporter.

#![cfg(feature = "experimental-tls")]
#![allow(missing_docs)]

use otap_df_config::tls::{TlsClientConfig, TlsConfig};
use otap_df_otap::otap_grpc::client_settings::GrpcClientSettings;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyPair, KeyUsagePurpose,
};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tonic::transport::{Identity, Server, ServerTlsConfig};

use bytes::Bytes;
use otap_df_otap::otap_grpc::otlp::client::LogsServiceClient;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_server::{
    LogsService, LogsServiceServer,
};
use prost::Message;
use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Status};

struct LogsServiceMock {
    sender: Sender<()>,
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

fn new_ca() -> (Certificate, KeyPair) {
    let mut params = CertificateParams::new(Vec::default()).expect("empty SAN");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params
        .distinguished_name
        .push(DnType::CommonName, "Test CA");
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);
    let key_pair = KeyPair::generate().expect("ca key");
    (params.self_signed(&key_pair).expect("ca cert"), key_pair)
}

fn new_leaf(
    cn: &str,
    san: &str,
    eku: ExtendedKeyUsagePurpose,
    ca: &Certificate,
    ca_key: &KeyPair,
) -> (String, String) {
    let mut params = CertificateParams::new(vec![san.to_string()]).expect("SAN");
    params.distinguished_name.push(DnType::CommonName, cn);
    params.use_authority_key_identifier_extension = true;
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.extended_key_usages.push(eku);
    let key_pair = KeyPair::generate().expect("leaf key");
    let cert = params.signed_by(&key_pair, ca, ca_key).expect("leaf cert");
    (cert.pem(), key_pair.serialize_pem())
}

#[tokio::test]
async fn otlp_exporter_connects_with_mtls() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Generate CA, server cert, client cert.
    let (ca, ca_key) = new_ca();
    let ca_pem = ca.pem();
    let (server_cert_pem, server_key_pem) = new_leaf(
        "localhost",
        "localhost",
        ExtendedKeyUsagePurpose::ServerAuth,
        &ca,
        &ca_key,
    );
    let (client_cert_pem, client_key_pem) = new_leaf(
        "client",
        "client",
        ExtendedKeyUsagePurpose::ClientAuth,
        &ca,
        &ca_key,
    );

    // gRPC service mock.
    let (tx, mut rx) = mpsc::channel::<()>(8);
    let logs_service = LogsServiceServer::new(LogsServiceMock { sender: tx });

    let server_identity = Identity::from_pem(server_cert_pem.as_bytes(), server_key_pem.as_bytes());
    let client_ca_root = tonic::transport::Certificate::from_pem(ca_pem.as_bytes());

    let tls = ServerTlsConfig::new()
        .identity(server_identity)
        .client_ca_root(client_ca_root);

    // Bind to ephemeral port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

    let server = tokio::spawn(async move {
        Server::builder()
            .tls_config(tls)
            .unwrap()
            .add_service(logs_service)
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    // Build client endpoint with exporter-style TLS config.
    let settings = GrpcClientSettings {
        grpc_endpoint: format!("https://localhost:{}", addr.port()),
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
        ..GrpcClientSettings::default()
    };

    let endpoint = settings.build_endpoint_with_tls().await.unwrap();
    let channel = endpoint.connect().await.unwrap();

    // Send a tiny export request to prove the connection works.
    // We only assert that server received at least one request.
    let mut client = LogsServiceClient::new(channel);
    let req = ExportLogsServiceRequest {
        resource_logs: Vec::new(),
    };

    let mut buf = Vec::new();
    req.encode(&mut buf).unwrap();
    let _ = client.export(Bytes::from(buf)).await.unwrap();

    // Server should observe a message.
    let observed = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
        .await
        .unwrap();
    assert!(observed.is_some());

    server.abort();
}

#[tokio::test]
async fn otlp_exporter_fails_with_invalid_ca_pem() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Generate CA and server cert.
    let (ca, ca_key) = new_ca();
    let (server_cert_pem, server_key_pem) = new_leaf(
        "localhost",
        "localhost",
        ExtendedKeyUsagePurpose::ServerAuth,
        &ca,
        &ca_key,
    );

    // gRPC service mock.
    let (tx, _rx) = mpsc::channel::<()>(8);
    let logs_service = LogsServiceServer::new(LogsServiceMock { sender: tx });

    let server_identity = Identity::from_pem(server_cert_pem.as_bytes(), server_key_pem.as_bytes());
    let tls = ServerTlsConfig::new().identity(server_identity);

    // Bind to ephemeral port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

    let server = tokio::spawn(async move {
        Server::builder()
            .tls_config(tls)
            .unwrap()
            .add_service(logs_service)
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    // Invalid CA PEM should prevent a successful TLS connection.
    let settings = GrpcClientSettings {
        grpc_endpoint: format!("https://localhost:{}", addr.port()),
        tls: Some(TlsClientConfig {
            config: TlsConfig::default(),
            ca_file: None,
            ca_pem: Some("not a pem bundle".to_string()),
            include_system_ca_certs_pool: Some(false),
            server_name: Some("localhost".to_string()),
            ..TlsClientConfig::default()
        }),
        ..GrpcClientSettings::default()
    };

    let endpoint = settings.build_endpoint_with_tls().await.unwrap();
    let connect_res = endpoint.connect().await;
    assert!(connect_res.is_err());

    server.abort();
}

#[tokio::test]
async fn otlp_exporter_allows_http_with_tls_config() {
    let settings = GrpcClientSettings {
        grpc_endpoint: "http://localhost:4317".to_string(),
        tls: Some(TlsClientConfig {
            config: TlsConfig::default(),
            ca_file: None,
            ca_pem: Some("fake pem".to_string()),
            include_system_ca_certs_pool: None,
            server_name: None,
            ..TlsClientConfig::default()
        }),
        ..GrpcClientSettings::default()
    };

    let result = settings.build_endpoint_with_tls().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn otlp_exporter_fails_partial_mtls() {
    let settings = GrpcClientSettings {
        grpc_endpoint: "https://localhost:4317".to_string(),
        tls: Some(TlsClientConfig {
            config: TlsConfig {
                cert_pem: Some("fake cert".to_string()),
                key_pem: None, // Missing key
                ..TlsConfig::default()
            },
            ca_file: None,
            ca_pem: None,
            include_system_ca_certs_pool: None,
            server_name: None,
            ..TlsClientConfig::default()
        }),
        ..GrpcClientSettings::default()
    };

    let result = settings.build_endpoint_with_tls().await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("both client certificate and key must be provided")
    );
}

#[tokio::test]
async fn otlp_exporter_connects_with_tls_only() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Generate CA and server cert (no client cert needed for TLS-only).
    let (ca, ca_key) = new_ca();
    let ca_pem = ca.pem();
    let (server_cert_pem, server_key_pem) = new_leaf(
        "localhost",
        "localhost",
        ExtendedKeyUsagePurpose::ServerAuth,
        &ca,
        &ca_key,
    );

    // gRPC service mock.
    let (tx, mut rx) = mpsc::channel::<()>(8);
    let logs_service = LogsServiceServer::new(LogsServiceMock { sender: tx });

    let server_identity = Identity::from_pem(server_cert_pem.as_bytes(), server_key_pem.as_bytes());
    // No client_ca_root - server doesn't require client certificates (TLS-only, not mTLS).
    let tls = ServerTlsConfig::new().identity(server_identity);

    // Bind to ephemeral port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

    let server = tokio::spawn(async move {
        Server::builder()
            .tls_config(tls)
            .unwrap()
            .add_service(logs_service)
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    // Build client endpoint with TLS but no client identity (TLS-only).
    let settings = GrpcClientSettings {
        grpc_endpoint: format!("https://localhost:{}", addr.port()),
        tls: Some(TlsClientConfig {
            config: TlsConfig::default(), // No client cert/key
            ca_file: None,
            ca_pem: Some(ca_pem),
            include_system_ca_certs_pool: Some(false),
            server_name: Some("localhost".to_string()),
            ..TlsClientConfig::default()
        }),
        ..GrpcClientSettings::default()
    };

    let endpoint = settings.build_endpoint_with_tls().await.unwrap();
    let channel = endpoint.connect().await.unwrap();

    // Send a request to prove the TLS connection works.
    let mut client = LogsServiceClient::new(channel);
    let req = ExportLogsServiceRequest {
        resource_logs: Vec::new(),
    };

    let mut buf = Vec::new();
    req.encode(&mut buf).unwrap();
    let _ = client.export(Bytes::from(buf)).await.unwrap();

    // Server should observe a message.
    let observed = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
        .await
        .unwrap();
    assert!(observed.is_some());

    server.abort();
}
