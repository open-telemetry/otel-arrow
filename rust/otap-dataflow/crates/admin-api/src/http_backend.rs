// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP-backed semantic admin backend.

use crate::client::{AdminBackend, HttpAdminClientSettings};
use crate::endpoint::{AdminAuth, AdminEndpoint, AdminScheme};
use crate::{Error, engine, operations, pipeline_groups, pipelines, telemetry};
use async_trait::async_trait;
use reqwest::{Certificate, ClientBuilder, Identity, Method, Url};
use serde::de::DeserializeOwned;
use std::fs;
use std::io;
use std::sync::OnceLock;

struct RawRequest {
    method: Method,
    url: Url,
}

struct RawResponse {
    status: u16,
    url: String,
    body: Vec<u8>,
}

pub(crate) struct HttpBackend {
    client: reqwest::Client,
    auth: AdminAuth,
    endpoint: AdminEndpoint,
}

impl HttpBackend {
    pub(crate) fn from_settings(settings: HttpAdminClientSettings) -> Result<Self, Error> {
        settings.endpoint.validate()?;
        let client = build_http_client(&settings)?;

        Ok(Self {
            client,
            auth: settings.auth,
            endpoint: settings.endpoint,
        })
    }

    async fn request_json<T: DeserializeOwned>(
        &self,
        method: Method,
        segments: &[&str],
        query: &[(&str, String)],
        expected_statuses: &[u16],
    ) -> Result<(u16, T), Error> {
        let (status, body) = self
            .request_raw(method, segments, query, expected_statuses)
            .await?;
        Ok((status, self.decode_json(&body)?))
    }

    async fn request_probe(
        &self,
        method: Method,
        segments: &[&str],
        query: &[(&str, String)],
        expected_statuses: &[u16],
    ) -> Result<pipelines::ProbeResult, Error> {
        let (status_code, body) = self
            .request_raw(method, segments, query, expected_statuses)
            .await?;
        let status = match status_code {
            200 => pipelines::ProbeStatus::Ok,
            500 | 503 => pipelines::ProbeStatus::Failed,
            _ => unreachable!("request_raw should have filtered unexpected probe statuses"),
        };
        Ok(pipelines::ProbeResult::new(
            status,
            Some(self.decode_text(&body)?),
        ))
    }

    async fn request_raw(
        &self,
        method: Method,
        segments: &[&str],
        query: &[(&str, String)],
        expected_statuses: &[u16],
    ) -> Result<(u16, Vec<u8>), Error> {
        let mut url = self.endpoint.url_for_segments(segments.iter().copied())?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                _ = pairs.append_pair(key, value);
            }
        }

        let response = self
            .send(RawRequest {
                method: method.clone(),
                url,
            })
            .await?;

        if expected_statuses.contains(&response.status) {
            Ok((response.status, response.body))
        } else {
            Err(Error::RemoteStatus {
                method: method.as_str().to_string(),
                url: response.url,
                status: response.status,
                body: String::from_utf8_lossy(&response.body).into_owned(),
            })
        }
    }

    async fn send(&self, request: RawRequest) -> Result<RawResponse, Error> {
        let RawRequest { method, url } = request;
        let builder = self.client.request(method, url.clone());

        match self.auth {
            AdminAuth::None => {}
        }

        let response = builder.send().await.map_err(|err| Error::Transport {
            details: err.to_string(),
        })?;
        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let body = response.bytes().await.map_err(|err| Error::Transport {
            details: err.to_string(),
        })?;

        Ok(RawResponse {
            status,
            url: final_url,
            body: body.to_vec(),
        })
    }

    fn decode_json<T: DeserializeOwned>(&self, body: &[u8]) -> Result<T, Error> {
        serde_json::from_slice(body).map_err(|err| Error::Decode {
            details: err.to_string(),
        })
    }

    fn decode_text(&self, body: &[u8]) -> Result<String, Error> {
        String::from_utf8(body.to_vec()).map_err(|err| Error::Decode {
            details: err.to_string(),
        })
    }
}

#[async_trait]
impl AdminBackend for HttpBackend {
    async fn engine_status(&self) -> Result<engine::Status, Error> {
        self.request_json(Method::GET, &["api", "v1", "status"], &[], &[200])
            .await
            .map(|(_, body)| body)
    }

    async fn engine_livez(&self) -> Result<engine::ProbeResponse, Error> {
        self.request_json(Method::GET, &["api", "v1", "livez"], &[], &[200, 500])
            .await
            .map(|(_, body)| body)
    }

    async fn engine_readyz(&self) -> Result<engine::ProbeResponse, Error> {
        self.request_json(Method::GET, &["api", "v1", "readyz"], &[], &[200, 503])
            .await
            .map(|(_, body)| body)
    }

    async fn pipeline_groups_status(&self) -> Result<pipeline_groups::Status, Error> {
        self.request_json(
            Method::GET,
            &["api", "v1", "pipeline-groups", "status"],
            &[],
            &[200],
        )
        .await
        .map(|(_, body)| body)
    }

    async fn pipeline_groups_shutdown(
        &self,
        options: &operations::OperationOptions,
    ) -> Result<pipeline_groups::ShutdownResponse, Error> {
        let query = options.to_query_pairs();
        self.request_json(
            Method::POST,
            &["api", "v1", "pipeline-groups", "shutdown"],
            &query,
            &[200, 202, 500, 504],
        )
        .await
        .map(|(_, body)| body)
    }

    async fn pipeline_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<pipelines::Status>, Error> {
        self.request_json(
            Method::GET,
            &[
                "api",
                "v1",
                "pipeline-groups",
                pipeline_group_id,
                "pipelines",
                pipeline_id,
                "status",
            ],
            &[],
            &[200],
        )
        .await
        .map(|(_, body)| body)
    }

    async fn pipeline_livez(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<pipelines::ProbeResult, Error> {
        self.request_probe(
            Method::GET,
            &[
                "api",
                "v1",
                "pipeline-groups",
                pipeline_group_id,
                "pipelines",
                pipeline_id,
                "livez",
            ],
            &[],
            &[200, 500],
        )
        .await
    }

    async fn pipeline_readyz(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<pipelines::ProbeResult, Error> {
        self.request_probe(
            Method::GET,
            &[
                "api",
                "v1",
                "pipeline-groups",
                pipeline_group_id,
                "pipelines",
                pipeline_id,
                "readyz",
            ],
            &[],
            &[200, 503],
        )
        .await
    }

    async fn telemetry_logs(
        &self,
        query: &telemetry::LogsQuery,
    ) -> Result<Option<telemetry::LogsResponse>, Error> {
        let query_pairs = query.to_query_pairs();
        let (status, body) = self
            .request_raw(
                Method::GET,
                &["api", "v1", "telemetry", "logs"],
                &query_pairs,
                &[200, 404],
            )
            .await?;
        if status == 404 {
            return Ok(None);
        }
        self.decode_json(&body).map(Some)
    }

    async fn telemetry_metrics(
        &self,
        options: &telemetry::MetricsOptions,
    ) -> Result<telemetry::MetricsResponse, Error> {
        let mut query_pairs = options.to_query_pairs();
        query_pairs.push(("format", "json".to_string()));
        self.request_json(
            Method::GET,
            &["api", "v1", "telemetry", "metrics"],
            &query_pairs,
            &[200],
        )
        .await
        .map(|(_, body)| body)
    }

    async fn telemetry_metrics_compact(
        &self,
        options: &telemetry::MetricsOptions,
    ) -> Result<telemetry::CompactMetricsResponse, Error> {
        let mut query_pairs = options.to_query_pairs();
        query_pairs.push(("format", "json_compact".to_string()));
        self.request_json(
            Method::GET,
            &["api", "v1", "telemetry", "metrics"],
            &query_pairs,
            &[200],
        )
        .await
        .map(|(_, body)| body)
    }
}

fn build_http_client(settings: &HttpAdminClientSettings) -> Result<reqwest::Client, Error> {
    validate_tls_settings(settings)?;

    // reqwest with rustls-no-provider requires a process-wide provider even
    // for plain HTTP clients because the TLS backend is selected at client
    // construction time.
    ensure_crypto_provider()?;

    let mut client_builder = ClientBuilder::new()
        .use_rustls_tls()
        .connect_timeout(settings.connect_timeout)
        .tcp_nodelay(settings.tcp_nodelay);

    if let Some(tcp_keepalive) = settings.tcp_keepalive {
        client_builder = client_builder.tcp_keepalive(tcp_keepalive);
    }

    if let Some(tcp_keepalive_interval) = settings.tcp_keepalive_interval {
        client_builder = client_builder.tcp_keepalive_interval(tcp_keepalive_interval);
    }

    if let Some(timeout) = settings.timeout {
        client_builder = client_builder.timeout(timeout);
    }

    if let Some(tls) = &settings.tls {
        let mut certs = Vec::new();

        if let Some(ca_pem) = &tls.ca_pem {
            let cert = Certificate::from_pem(ca_pem.as_bytes()).map_err(client_config_error)?;
            certs.push(cert);
        }

        if let Some(ca_file) = &tls.ca_file {
            let ca_pem = fs::read(ca_file).map_err(client_config_io_error)?;
            let cert = Certificate::from_pem(&ca_pem).map_err(client_config_error)?;
            certs.push(cert);
        }

        if tls.include_system_ca_certs_pool.unwrap_or(true) {
            client_builder = client_builder.tls_certs_merge(certs);
        } else {
            client_builder = client_builder.tls_certs_only(certs);
        }

        if let Some(true) = tls.insecure_skip_verify {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let client_cert_configured = tls.config.cert_file.is_some()
            || tls
                .config
                .cert_pem
                .as_ref()
                .is_some_and(|pem| !pem.trim().is_empty());
        let client_key_configured = tls.config.key_file.is_some()
            || tls
                .config
                .key_pem
                .as_ref()
                .is_some_and(|pem| !pem.trim().is_empty());

        if client_cert_configured || client_key_configured {
            if !(client_cert_configured && client_key_configured) {
                return Err(Error::ClientConfig {
                    details: "both client certificate and key must be provided for mTLS"
                        .to_string(),
                });
            }

            let cert_pem = if let Some(cert_file) = &tls.config.cert_file {
                fs::read(cert_file).map_err(client_config_io_error)?
            } else if let Some(cert_pem) = &tls.config.cert_pem {
                cert_pem.as_bytes().to_vec()
            } else {
                unreachable!();
            };

            let key_pem = if let Some(key_file) = &tls.config.key_file {
                fs::read(key_file).map_err(client_config_io_error)?
            } else if let Some(key_pem) = &tls.config.key_pem {
                key_pem.as_bytes().to_vec()
            } else {
                unreachable!();
            };

            let mut identity_pem = cert_pem;
            identity_pem.extend_from_slice(&key_pem);

            let identity = Identity::from_pem(&identity_pem).map_err(client_config_error)?;
            client_builder = client_builder.identity(identity);
        }
    }

    client_builder.build().map_err(client_config_error)
}

fn client_config_error(error: impl std::fmt::Display) -> Error {
    Error::ClientConfig {
        details: error.to_string(),
    }
}

fn client_config_io_error(error: io::Error) -> Error {
    Error::ClientConfig {
        details: error.to_string(),
    }
}

fn validate_tls_settings(settings: &HttpAdminClientSettings) -> Result<(), Error> {
    let Some(tls) = &settings.tls else {
        return Ok(());
    };

    if settings.endpoint.scheme == AdminScheme::Http {
        let tls_enabled = tls.ca_file.is_some()
            || tls
                .ca_pem
                .as_ref()
                .is_some_and(|pem| !pem.trim().is_empty())
            || tls.config.cert_file.is_some()
            || tls
                .config
                .cert_pem
                .as_ref()
                .is_some_and(|pem| !pem.trim().is_empty())
            || tls.config.key_file.is_some()
            || tls
                .config
                .key_pem
                .as_ref()
                .is_some_and(|pem| !pem.trim().is_empty())
            || tls.server_name.is_some()
            || tls.insecure == Some(true)
            || tls.insecure_skip_verify == Some(true);
        if tls_enabled {
            return Err(Error::ClientConfig {
                details: "TLS settings require an https admin endpoint".to_string(),
            });
        }
    }

    if let Some(server_name) = &tls.server_name {
        return Err(Error::ClientConfig {
            details: format!(
                "server_name_override is not supported by the current admin HTTP client implementation: {server_name}"
            ),
        });
    }

    if let Some(true) = tls.insecure {
        return Err(Error::ClientConfig {
            details: "tls.insecure is not supported by the admin SDK; use AdminEndpoint::http(...) for plaintext connections".to_string(),
        });
    }

    Ok(())
}

fn ensure_crypto_provider() -> Result<(), Error> {
    static INIT: OnceLock<Result<(), String>> = OnceLock::new();

    INIT.get_or_init(|| {
        // Feature unification can enable multiple crypto backends at once, for
        // example during `--all-features` CI builds. Prefer the most explicit
        // override in that case rather than failing the entire build.
        #[cfg(feature = "crypto-openssl")]
        {
            let _ = rustls_openssl::default_provider().install_default();
            Ok(())
        }

        #[cfg(all(feature = "crypto-aws-lc", not(feature = "crypto-openssl")))]
        {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            Ok(())
        }

        #[cfg(all(
            feature = "crypto-ring",
            not(feature = "crypto-openssl"),
            not(feature = "crypto-aws-lc")
        ))]
        {
            let _ = rustls::crypto::ring::default_provider().install_default();
            Ok(())
        }

        #[cfg(all(
            feature = "crypto-symcrypt",
            not(feature = "crypto-openssl"),
            not(feature = "crypto-aws-lc"),
            not(feature = "crypto-ring")
        ))]
        {
            let _ = rustls_symcrypt::default_symcrypt_provider().install_default();
            Ok(())
        }

        #[cfg(not(any(
            feature = "crypto-ring",
            feature = "crypto-aws-lc",
            feature = "crypto-openssl",
            feature = "crypto-symcrypt"
        )))]
        {
            Err(
                "Admin client construction requires one of the admin SDK crypto features: \
                 crypto-ring, crypto-aws-lc, crypto-openssl, or crypto-symcrypt"
                    .to_string(),
            )
        }
    })
    .clone()
    .map_err(client_config_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tls::{TlsClientConfig, TlsConfig};
    use crate::{AdminClient, engine, operations, pipeline_groups, pipelines, telemetry};
    use otap_test_tls_certs::{ExtendedKeyUsage, generate_ca};
    use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(server: &MockServer) -> AdminClient {
        let endpoint = AdminEndpoint::http("127.0.0.1", server.address().port());
        AdminClient::builder()
            .http(HttpAdminClientSettings::new(endpoint))
            .build()
            .expect("client should build")
    }

    async fn start_https_json_server(
        server_cert_pem: &str,
        server_key_pem: &str,
        body: String,
    ) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
        ensure_crypto_provider().expect("install rustls crypto provider for test TLS server");

        let cert_chain: Vec<_> = CertificateDer::pem_slice_iter(server_cert_pem.as_bytes())
            .collect::<Result<_, _>>()
            .expect("parse cert chain");
        let private_key =
            PrivateKeyDer::from_pem_slice(server_key_pem.as_bytes()).expect("parse private key");

        let tls_server_cfg = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .expect("build tls server config");
        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_server_cfg));

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind tls server");
        let addr = listener.local_addr().expect("tls server local addr");

        let task = tokio::spawn(async move {
            let (tcp_stream, _) = listener.accept().await.expect("accept tls connection");
            let mut stream = tls_acceptor
                .accept(tcp_stream)
                .await
                .expect("complete tls handshake");

            let mut request = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                let bytes_read = stream.read(&mut buf).await.expect("read request");
                if bytes_read == 0 {
                    return;
                }
                request.extend_from_slice(&buf[..bytes_read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write response");
            stream.flush().await.expect("flush response");
        });

        (addr, task)
    }

    #[tokio::test]
    async fn builder_builds_http_client() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(engine::Status {
                generated_at: "2026-01-01T00:00:00Z".to_string(),
                pipelines: Default::default(),
            }))
            .mount(&server)
            .await;

        let endpoint = AdminEndpoint::http("127.0.0.1", server.address().port());
        let client = AdminClient::builder()
            .http(HttpAdminClientSettings::new(endpoint))
            .build()
            .expect("client should build");

        let response = client
            .engine()
            .status()
            .await
            .expect("status should decode");
        assert_eq!(response.generated_at, "2026-01-01T00:00:00Z");
    }

    #[tokio::test]
    async fn engine_status_decodes_json() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(engine::Status {
                generated_at: "2026-01-01T00:00:00Z".to_string(),
                pipelines: Default::default(),
            }))
            .mount(&server)
            .await;

        let response = client(&server)
            .engine()
            .status()
            .await
            .expect("status should decode");
        assert_eq!(response.generated_at, "2026-01-01T00:00:00Z");
    }

    #[tokio::test]
    async fn engine_livez_decodes_failure_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/livez"))
            .respond_with(
                ResponseTemplate::new(500).set_body_json(engine::ProbeResponse {
                    probe: engine::ProbeKind::Livez,
                    status: engine::ProbeStatus::Failed,
                    generated_at: "2026-01-01T00:00:00Z".to_string(),
                    message: None,
                    failing: vec![],
                }),
            )
            .mount(&server)
            .await;

        let response = client(&server)
            .engine()
            .livez()
            .await
            .expect("livez should decode");
        assert_eq!(response.status, engine::ProbeStatus::Failed);
    }

    #[tokio::test]
    async fn pipeline_groups_shutdown_accepts_query_and_non_200_success_shapes() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/pipeline-groups/shutdown"))
            .and(query_param("wait", "true"))
            .and(query_param("timeout_secs", "30"))
            .respond_with(ResponseTemplate::new(202).set_body_json(
                pipeline_groups::ShutdownResponse {
                    status: pipeline_groups::ShutdownStatus::Accepted,
                    errors: None,
                    duration_ms: None,
                },
            ))
            .mount(&server)
            .await;

        let response = client(&server)
            .pipeline_groups()
            .shutdown(&operations::OperationOptions {
                wait: true,
                timeout_secs: 30,
            })
            .await
            .expect("shutdown should decode");

        assert_eq!(response.status, pipeline_groups::ShutdownStatus::Accepted);
    }

    #[tokio::test]
    async fn pipeline_status_decodes_optional_payload() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/api/v1/pipeline-groups/default/pipelines/main/status",
            ))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(Some(pipelines::Status {
                    conditions: vec![],
                    total_cores: 1,
                    running_cores: 1,
                    cores: Default::default(),
                })),
            )
            .mount(&server)
            .await;

        let response = client(&server)
            .pipelines()
            .status("default", "main")
            .await
            .expect("pipeline status should decode");

        assert!(response.is_some());
    }

    #[tokio::test]
    async fn pipeline_livez_maps_failed_probe_and_message() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/pipeline-groups/default/pipelines/main/livez"))
            .respond_with(ResponseTemplate::new(500).set_body_string("NOT OK"))
            .mount(&server)
            .await;

        let response = client(&server)
            .pipelines()
            .livez("default", "main")
            .await
            .expect("pipeline livez should decode");
        assert_eq!(response.status, pipelines::ProbeStatus::Failed);
        assert_eq!(response.message.as_deref(), Some("NOT OK"));
    }

    #[tokio::test]
    async fn pipeline_livez_maps_ok_probe_without_message() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/pipeline-groups/default/pipelines/main/livez"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;

        let response = client(&server)
            .pipelines()
            .livez("default", "main")
            .await
            .expect("pipeline livez should decode");

        assert_eq!(response.status, pipelines::ProbeStatus::Ok);
        assert_eq!(response.message, None);
    }

    #[tokio::test]
    async fn pipeline_readyz_maps_service_unavailable_to_failed_probe() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/api/v1/pipeline-groups/default/pipelines/main/readyz",
            ))
            .respond_with(ResponseTemplate::new(503).set_body_string("NOT OK"))
            .mount(&server)
            .await;

        let response = client(&server)
            .pipelines()
            .readyz("default", "main")
            .await
            .expect("pipeline readyz should decode");

        assert_eq!(response.status, pipelines::ProbeStatus::Failed);
        assert_eq!(response.message.as_deref(), Some("NOT OK"));
    }

    #[tokio::test]
    async fn pipeline_probe_unexpected_status_is_remote_status() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/pipeline-groups/default/pipelines/main/livez"))
            .respond_with(ResponseTemplate::new(418).set_body_string("teapot"))
            .mount(&server)
            .await;

        let err = client(&server)
            .pipelines()
            .livez("default", "main")
            .await
            .expect_err("pipeline probe should fail");

        match err {
            Error::RemoteStatus { status, body, .. } => {
                assert_eq!(status, 418);
                assert_eq!(body, "teapot");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn telemetry_logs_returns_none_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/logs"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let response = client(&server)
            .telemetry()
            .logs(&telemetry::LogsQuery::default())
            .await
            .expect("logs should return none");
        assert!(response.is_none());
    }

    #[tokio::test]
    async fn telemetry_metrics_decodes_verbose_json() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/metrics"))
            .and(query_param("reset", "false"))
            .and(query_param("keep_all_zeroes", "false"))
            .and(query_param("format", "json"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(telemetry::MetricsResponse {
                    timestamp: "2026-01-01T00:00:00Z".to_string(),
                    metric_sets: vec![],
                }),
            )
            .mount(&server)
            .await;

        let response = client(&server)
            .telemetry()
            .metrics(&telemetry::MetricsOptions::default())
            .await
            .expect("metrics should decode");

        assert_eq!(response.timestamp, "2026-01-01T00:00:00Z");
    }

    #[tokio::test]
    async fn telemetry_metrics_decodes_compact_json() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/metrics"))
            .and(query_param("reset", "false"))
            .and(query_param("keep_all_zeroes", "false"))
            .and(query_param("format", "json_compact"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                telemetry::CompactMetricsResponse {
                    timestamp: "2026-01-01T00:00:00Z".to_string(),
                    metric_sets: vec![],
                },
            ))
            .mount(&server)
            .await;

        let response = client(&server)
            .telemetry()
            .metrics_compact(&telemetry::MetricsOptions::default())
            .await
            .expect("metrics should decode");

        assert_eq!(response.timestamp, "2026-01-01T00:00:00Z");
    }

    #[tokio::test]
    async fn unexpected_status_is_remote_status() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/status"))
            .respond_with(ResponseTemplate::new(418).set_body_string("teapot"))
            .mount(&server)
            .await;

        let err = client(&server)
            .engine()
            .status()
            .await
            .expect_err("status should fail");

        match err {
            Error::RemoteStatus { status, body, .. } => {
                assert_eq!(status, 418);
                assert_eq!(body, "teapot");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn https_status_decodes_json_with_ca_pem() {
        let ca = generate_ca("admin-ca");
        let server_cert = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );

        let body = serde_json::to_string(&engine::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: Default::default(),
        })
        .expect("serialize status");
        let (addr, server_task) =
            start_https_json_server(&server_cert.cert_pem, &server_cert.key_pem, body).await;

        let client = AdminClient::builder()
            .http(
                HttpAdminClientSettings::new(AdminEndpoint::https("localhost", addr.port()))
                    .with_tls(TlsClientConfig {
                        ca_pem: Some(ca.cert_pem.clone()),
                        ..TlsClientConfig::default()
                    }),
            )
            .build()
            .expect("https client should build");

        let response = client
            .engine()
            .status()
            .await
            .expect("https status should decode");
        assert_eq!(response.generated_at, "2026-01-01T00:00:00Z");

        server_task.await.expect("https server task should finish");
    }

    #[tokio::test]
    async fn https_status_decodes_json_with_insecure_skip_verify() {
        let ca = generate_ca("admin-ca");
        let server_cert = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );

        let body = serde_json::to_string(&engine::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: Default::default(),
        })
        .expect("serialize status");
        let (addr, server_task) =
            start_https_json_server(&server_cert.cert_pem, &server_cert.key_pem, body).await;

        let client = AdminClient::builder()
            .http(
                HttpAdminClientSettings::new(AdminEndpoint::https("localhost", addr.port()))
                    .with_tls(TlsClientConfig {
                        insecure_skip_verify: Some(true),
                        ..TlsClientConfig::default()
                    }),
            )
            .build()
            .expect("https client should build");

        let response = client
            .engine()
            .status()
            .await
            .expect("https status should decode");
        assert_eq!(response.generated_at, "2026-01-01T00:00:00Z");

        server_task.await.expect("https server task should finish");
    }

    #[tokio::test]
    async fn client_build_reads_ca_file() {
        let ca = generate_ca("admin-ca");
        let dir = tempdir().expect("create tempdir");
        let ca_path = dir.path().join("admin-ca.pem");
        fs::write(&ca_path, ca.cert_pem.as_bytes()).expect("write CA file");

        let client = AdminClient::builder()
            .http(
                HttpAdminClientSettings::new(AdminEndpoint::https("localhost", 8443)).with_tls(
                    TlsClientConfig {
                        ca_file: Some(ca_path),
                        ..TlsClientConfig::default()
                    },
                ),
            )
            .build();

        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn http_endpoint_rejects_tls_settings() {
        let result = AdminClient::builder()
            .http(
                HttpAdminClientSettings::new(AdminEndpoint::http("localhost", 8080)).with_tls(
                    TlsClientConfig {
                        insecure_skip_verify: Some(true),
                        ..TlsClientConfig::default()
                    },
                ),
            )
            .build();

        assert!(matches!(result, Err(Error::ClientConfig { .. })));
    }

    #[tokio::test]
    async fn client_build_rejects_server_name_override() {
        let result = AdminClient::builder()
            .http(
                HttpAdminClientSettings::new(AdminEndpoint::https("localhost", 8443)).with_tls(
                    TlsClientConfig {
                        server_name: Some("admin.example.com".to_string()),
                        ..TlsClientConfig::default()
                    },
                ),
            )
            .build();

        assert!(matches!(result, Err(Error::ClientConfig { .. })));
    }

    #[tokio::test]
    async fn client_build_rejects_partial_mtls_identity() {
        let result = AdminClient::builder()
            .http(
                HttpAdminClientSettings::new(AdminEndpoint::https("localhost", 8443)).with_tls(
                    TlsClientConfig {
                        config: TlsConfig {
                            cert_pem: Some("cert".to_string()),
                            ..TlsConfig::default()
                        },
                        ..TlsClientConfig::default()
                    },
                ),
            )
            .build();

        assert!(matches!(result, Err(Error::ClientConfig { .. })));
    }

    #[tokio::test]
    async fn https_client_build_accepts_inline_mtls_identity() {
        let ca = generate_ca("admin-ca");
        let client_cert =
            ca.issue_leaf("client", Some("client"), Some(ExtendedKeyUsage::ClientAuth));

        let client = AdminClient::builder()
            .http(
                HttpAdminClientSettings::new(AdminEndpoint::https("localhost", 8443)).with_tls(
                    TlsClientConfig {
                        ca_pem: Some(ca.cert_pem.clone()),
                        config: TlsConfig {
                            cert_pem: Some(client_cert.cert_pem),
                            key_pem: Some(client_cert.key_pem),
                            ..TlsConfig::default()
                        },
                        ..TlsClientConfig::default()
                    },
                ),
            )
            .build();

        assert!(client.is_ok());
    }
}
