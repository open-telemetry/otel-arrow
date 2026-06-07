// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP-backed semantic admin backend.

use crate::client::{AdminBackend, HttpAdminClientSettings};
use crate::endpoint::{AdminAuth, AdminEndpoint, AdminScheme};
use crate::{Error, engine, groups, operations, pipelines, telemetry};
use async_trait::async_trait;
use reqwest::{Certificate, ClientBuilder, Identity, Method, Url};
use serde::de::DeserializeOwned;
use std::fs;
use std::io;
use std::sync::OnceLock;

struct RawRequest {
    method: Method,
    url: Url,
    body: Option<Vec<u8>>,
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
            .request_raw(method, segments, query, None, expected_statuses)
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
            .request_raw(method, segments, query, None, expected_statuses)
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
        body: Option<Vec<u8>>,
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
                body,
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
        let RawRequest { method, url, body } = request;
        let mut builder = self.client.request(method, url.clone());

        match self.auth {
            AdminAuth::None => {}
        }

        if let Some(body) = body {
            builder = builder
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body);
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

    fn decode_operation_error(&self, status: u16, body: &[u8]) -> Result<Error, Error> {
        let error = self.decode_json::<operations::OperationError>(body)?;
        Ok(Error::AdminOperation { status, error })
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

    async fn groups_status(&self) -> Result<groups::Status, Error> {
        self.request_json(Method::GET, &["api", "v1", "groups", "status"], &[], &[200])
            .await
            .map(|(_, body)| body)
    }

    async fn groups_shutdown(
        &self,
        options: &operations::OperationOptions,
    ) -> Result<groups::ShutdownResponse, Error> {
        let query = options.to_query_pairs();
        self.request_json(
            Method::POST,
            &["api", "v1", "groups", "shutdown"],
            &query,
            &[200, 202, 500, 504],
        )
        .await
        .map(|(_, body)| body)
    }

    async fn pipeline_details(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<pipelines::PipelineDetails>, Error> {
        let (status, body) = self
            .request_raw(
                Method::GET,
                &[
                    "api",
                    "v1",
                    "groups",
                    pipeline_group_id,
                    "pipelines",
                    pipeline_id,
                ],
                &[],
                None,
                &[200, 404],
            )
            .await?;
        if status == 404 {
            return Ok(None);
        }
        self.decode_json(&body).map(Some)
    }

    async fn pipeline_reconfigure(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: &pipelines::ReconfigureRequest,
        options: &operations::OperationOptions,
    ) -> Result<pipelines::ReconfigureOutcome, Error> {
        let query = options.to_query_pairs();
        let (status, body) = self
            .request_raw(
                Method::PUT,
                &[
                    "api",
                    "v1",
                    "groups",
                    pipeline_group_id,
                    "pipelines",
                    pipeline_id,
                ],
                &query,
                Some(
                    serde_json::to_vec(request).map_err(|err| Error::ClientConfig {
                        details: format!("failed to encode reconfigure request: {err}"),
                    })?,
                ),
                &[200, 202, 404, 409, 422, 500, 504],
            )
            .await?;

        match status {
            200 => self
                .decode_json(&body)
                .map(pipelines::ReconfigureOutcome::Completed),
            202 => self
                .decode_json(&body)
                .map(pipelines::ReconfigureOutcome::Accepted),
            409 => match self.decode_json::<pipelines::RolloutStatus>(&body) {
                Ok(status) => Ok(pipelines::ReconfigureOutcome::Failed(status)),
                Err(_) => Err(self.decode_operation_error(status, &body)?),
            },
            504 => self
                .decode_json(&body)
                .map(pipelines::ReconfigureOutcome::TimedOut),
            404 | 422 | 500 => Err(self.decode_operation_error(status, &body)?),
            _ => unreachable!("request_raw should have filtered unexpected statuses"),
        }
    }

    async fn pipeline_rollout_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        rollout_id: &str,
    ) -> Result<Option<pipelines::RolloutStatus>, Error> {
        let (status, body) = self
            .request_raw(
                Method::GET,
                &[
                    "api",
                    "v1",
                    "groups",
                    pipeline_group_id,
                    "pipelines",
                    pipeline_id,
                    "rollouts",
                    rollout_id,
                ],
                &[],
                None,
                &[200, 404],
            )
            .await?;
        if status == 404 {
            return Ok(None);
        }
        self.decode_json(&body).map(Some)
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
                "groups",
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
                "groups",
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
                "groups",
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

    async fn pipeline_shutdown(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        options: &operations::OperationOptions,
    ) -> Result<pipelines::ShutdownOutcome, Error> {
        let query = options.to_query_pairs();
        let (status, body) = self
            .request_raw(
                Method::POST,
                &[
                    "api",
                    "v1",
                    "groups",
                    pipeline_group_id,
                    "pipelines",
                    pipeline_id,
                    "shutdown",
                ],
                &query,
                None,
                &[200, 202, 404, 409, 422, 500, 504],
            )
            .await?;

        match status {
            200 => self
                .decode_json(&body)
                .map(pipelines::ShutdownOutcome::Completed),
            202 => self
                .decode_json(&body)
                .map(pipelines::ShutdownOutcome::Accepted),
            409 => match self.decode_json::<pipelines::ShutdownStatus>(&body) {
                Ok(status) => Ok(pipelines::ShutdownOutcome::Failed(status)),
                Err(_) => Err(self.decode_operation_error(status, &body)?),
            },
            504 => self
                .decode_json(&body)
                .map(pipelines::ShutdownOutcome::TimedOut),
            404 | 422 | 500 => Err(self.decode_operation_error(status, &body)?),
            _ => unreachable!("request_raw should have filtered unexpected statuses"),
        }
    }

    async fn pipeline_shutdown_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        shutdown_id: &str,
    ) -> Result<Option<pipelines::ShutdownStatus>, Error> {
        let (status, body) = self
            .request_raw(
                Method::GET,
                &[
                    "api",
                    "v1",
                    "groups",
                    pipeline_group_id,
                    "pipelines",
                    pipeline_id,
                    "shutdowns",
                    shutdown_id,
                ],
                &[],
                None,
                &[200, 404],
            )
            .await?;
        if status == 404 {
            return Ok(None);
        }
        self.decode_json(&body).map(Some)
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
                None,
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
    use crate::{AdminClient, engine, groups, operations, pipelines, telemetry};
    use otap_test_tls_certs::{ExtendedKeyUsage, generate_ca};
    use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
    use serde_json::json;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(server: &MockServer) -> AdminClient {
        let endpoint = AdminEndpoint::http("127.0.0.1", server.address().port());
        AdminClient::builder()
            .http(HttpAdminClientSettings::new(endpoint))
            .build()
            .expect("client should build")
    }

    fn minimal_pipeline_json() -> serde_json::Value {
        json!({
            "type": "otap",
            "nodes": {
                "recv": {
                    "type": "receiver:traffic_generator",
                    "config": {}
                }
            }
        })
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

    /// Scenario: the SDK calls the group shutdown endpoint with wait/query
    /// options and the server returns a non-200 success body.
    /// Guarantees: the HTTP backend targets `/api/v1/groups/shutdown`,
    /// forwards the query parameters, and still decodes the accepted response.
    #[tokio::test]
    async fn groups_shutdown_accepts_query_and_non_200_success_shapes() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/groups/shutdown"))
            .and(query_param("wait", "true"))
            .and(query_param("timeout_secs", "30"))
            .respond_with(
                ResponseTemplate::new(202).set_body_json(groups::ShutdownResponse {
                    status: groups::ShutdownStatus::Accepted,
                    errors: None,
                    duration_ms: None,
                }),
            )
            .mount(&server)
            .await;

        let response = client(&server)
            .groups()
            .shutdown(&operations::OperationOptions {
                wait: true,
                timeout_secs: 30,
            })
            .await
            .expect("shutdown should decode");

        assert_eq!(response.status, groups::ShutdownStatus::Accepted);
    }

    /// Scenario: a caller requests group status through the public SDK.
    /// Guarantees: the HTTP backend uses the `/api/v1/groups/status` route
    /// instead of the older pipeline-groups path and decodes the payload.
    #[tokio::test]
    async fn groups_status_uses_groups_route() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/groups/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(groups::Status {
                generated_at: "2026-01-01T00:00:00Z".to_string(),
                pipelines: Default::default(),
            }))
            .mount(&server)
            .await;

        let response = client(&server)
            .groups()
            .status()
            .await
            .expect("group status should decode");
        assert_eq!(response.generated_at, "2026-01-01T00:00:00Z");
    }

    #[tokio::test]
    async fn pipeline_status_decodes_optional_payload() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/groups/default/pipelines/main/status"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(Some(pipelines::Status {
                    conditions: vec![],
                    total_cores: 1,
                    running_cores: 1,
                    cores: Default::default(),
                    instances: None,
                    active_generation: None,
                    serving_generations: None,
                    rollout: None,
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

    /// Scenario: the server returns a committed pipeline details payload for an
    /// existing logical pipeline.
    /// Guarantees: the SDK surfaces that payload as `Some(...)` rather than
    /// treating it as an optional or missing resource.
    #[tokio::test]
    async fn pipeline_details_returns_some_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/groups/default/pipelines/main"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "pipelineGroupId": "default",
                "pipelineId": "main",
                "activeGeneration": 3,
                "pipeline": minimal_pipeline_json(),
                "rollout": {
                    "rolloutId": "rollout-3",
                    "state": "running",
                    "targetGeneration": 3,
                    "startedAt": "2026-01-01T00:00:00Z",
                    "updatedAt": "2026-01-01T00:00:01Z"
                }
            })))
            .mount(&server)
            .await;

        let response = client(&server)
            .pipelines()
            .details("default", "main")
            .await
            .expect("pipeline details should decode");

        assert!(response.is_some());
    }

    /// Scenario: a caller submits an asynchronous reconfigure request through
    /// the public SDK.
    /// Guarantees: the backend serializes the request body and query options
    /// correctly and maps an accepted rollout response to `Accepted`.
    #[tokio::test]
    async fn pipeline_reconfigure_encodes_request_and_decodes_accepted() {
        let server = MockServer::start().await;
        let request = pipelines::ReconfigureRequest {
            pipeline: serde_json::from_value(minimal_pipeline_json())
                .expect("fixture pipeline should deserialize"),
            step_timeout_secs: 45,
            drain_timeout_secs: 30,
        };
        Mock::given(method("PUT"))
            .and(path("/api/v1/groups/default/pipelines/main"))
            .and(query_param("wait", "false"))
            .and(query_param("timeout_secs", "120"))
            .and(body_json(
                serde_json::to_value(&request).expect("request should serialize"),
            ))
            .respond_with(ResponseTemplate::new(202).set_body_json(json!({
                "rolloutId": "rollout-3",
                "pipelineGroupId": "default",
                "pipelineId": "main",
                "action": "replace",
                "state": "running",
                "targetGeneration": 3,
                "previousGeneration": 2,
                "startedAt": "2026-01-01T00:00:00Z",
                "updatedAt": "2026-01-01T00:00:01Z",
                "cores": []
            })))
            .mount(&server)
            .await;

        let response = client(&server)
            .pipelines()
            .reconfigure(
                "default",
                "main",
                &request,
                &operations::OperationOptions {
                    wait: false,
                    timeout_secs: 120,
                },
            )
            .await
            .expect("reconfigure should decode");

        match response {
            pipelines::ReconfigureOutcome::Accepted(status) => {
                assert_eq!(status.rollout_id, "rollout-3");
                assert_eq!(status.state, pipelines::PipelineRolloutState::Running);
            }
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    /// Scenario: a waited reconfigure request reaches a terminal failed rollout
    /// and the server reports that state with a 409 status body.
    /// Guarantees: the backend treats this as an operation outcome, not a typed
    /// request rejection, and returns `ReconfigureOutcome::Failed`.
    #[tokio::test]
    async fn pipeline_reconfigure_decodes_failed_outcome_from_409_status_body() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/api/v1/groups/default/pipelines/main"))
            .and(query_param("wait", "true"))
            .and(query_param("timeout_secs", "60"))
            .respond_with(ResponseTemplate::new(409).set_body_json(json!({
                "rolloutId": "rollout-4",
                "pipelineGroupId": "default",
                "pipelineId": "main",
                "action": "replace",
                "state": "failed",
                "targetGeneration": 4,
                "previousGeneration": 3,
                "startedAt": "2026-01-01T00:00:00Z",
                "updatedAt": "2026-01-01T00:00:10Z",
                "failureReason": "candidate failed admission",
                "cores": []
            })))
            .mount(&server)
            .await;

        let request = pipelines::ReconfigureRequest {
            pipeline: serde_json::from_value(minimal_pipeline_json())
                .expect("fixture pipeline should deserialize"),
            step_timeout_secs: 60,
            drain_timeout_secs: 60,
        };

        let response = client(&server)
            .pipelines()
            .reconfigure(
                "default",
                "main",
                &request,
                &operations::OperationOptions {
                    wait: true,
                    timeout_secs: 60,
                },
            )
            .await
            .expect("failed outcome should decode");

        match response {
            pipelines::ReconfigureOutcome::Failed(status) => {
                assert_eq!(status.rollout_id, "rollout-4");
                assert_eq!(status.state, pipelines::PipelineRolloutState::Failed);
            }
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    /// Scenario: the server rejects a reconfigure request before any rollout
    /// work starts and returns a structured operation error body.
    /// Guarantees: the backend preserves that rejection as
    /// `Error::AdminOperation` so callers can distinguish it from transport
    /// failures and terminal rollout outcomes.
    #[tokio::test]
    async fn pipeline_reconfigure_decodes_admin_operation_error() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/api/v1/groups/default/pipelines/main"))
            .respond_with(ResponseTemplate::new(422).set_body_json(json!({
                "kind": "invalid_request",
                "message": "topic runtime mutation is not supported"
            })))
            .mount(&server)
            .await;

        let request = pipelines::ReconfigureRequest {
            pipeline: serde_json::from_value(minimal_pipeline_json())
                .expect("fixture pipeline should deserialize"),
            step_timeout_secs: 60,
            drain_timeout_secs: 60,
        };

        let err = client(&server)
            .pipelines()
            .reconfigure(
                "default",
                "main",
                &request,
                &operations::OperationOptions::default(),
            )
            .await
            .expect_err("request rejection should be typed");

        match err {
            Error::AdminOperation { status, error } => {
                assert_eq!(status, 422);
                assert_eq!(error.kind, operations::OperationErrorKind::InvalidRequest);
                assert_eq!(
                    error.message.as_deref(),
                    Some("topic runtime mutation is not supported")
                );
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    /// Scenario: a caller polls a rollout id that no longer exists or was never
    /// created.
    /// Guarantees: the backend maps HTTP 404 to `Ok(None)` for rollout status
    /// lookups instead of treating it as an SDK error.
    #[tokio::test]
    async fn pipeline_rollout_status_returns_none_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/api/v1/groups/default/pipelines/main/rollouts/rollout-9",
            ))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let response = client(&server)
            .pipelines()
            .rollout_status("default", "main", "rollout-9")
            .await
            .expect("rollout status should decode");

        assert!(response.is_none());
    }

    /// Scenario: a caller waits on pipeline shutdown and the server times out
    /// the wait while returning the latest shutdown snapshot.
    /// Guarantees: the backend decodes that response as
    /// `ShutdownOutcome::TimedOut` and preserves the embedded status.
    #[tokio::test]
    async fn pipeline_shutdown_decodes_timed_out_outcome() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/groups/default/pipelines/main/shutdown"))
            .and(query_param("wait", "true"))
            .and(query_param("timeout_secs", "30"))
            .respond_with(ResponseTemplate::new(504).set_body_json(json!({
                "shutdownId": "shutdown-2",
                "pipelineGroupId": "default",
                "pipelineId": "main",
                "state": "running",
                "startedAt": "2026-01-01T00:00:00Z",
                "updatedAt": "2026-01-01T00:00:30Z",
                "cores": []
            })))
            .mount(&server)
            .await;

        let response = client(&server)
            .pipelines()
            .shutdown(
                "default",
                "main",
                &operations::OperationOptions {
                    wait: true,
                    timeout_secs: 30,
                },
            )
            .await
            .expect("shutdown outcome should decode");

        match response {
            pipelines::ShutdownOutcome::TimedOut(status) => {
                assert_eq!(status.shutdown_id, "shutdown-2");
            }
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    /// Scenario: a caller polls a known pipeline shutdown operation by id.
    /// Guarantees: the backend decodes the returned shutdown snapshot and
    /// surfaces it as `Some(...)`.
    #[tokio::test]
    async fn pipeline_shutdown_status_returns_some_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/api/v1/groups/default/pipelines/main/shutdowns/shutdown-2",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "shutdownId": "shutdown-2",
                "pipelineGroupId": "default",
                "pipelineId": "main",
                "state": "succeeded",
                "startedAt": "2026-01-01T00:00:00Z",
                "updatedAt": "2026-01-01T00:00:05Z",
                "cores": []
            })))
            .mount(&server)
            .await;

        let response = client(&server)
            .pipelines()
            .shutdown_status("default", "main", "shutdown-2")
            .await
            .expect("shutdown status should decode");

        assert!(response.is_some());
    }

    #[tokio::test]
    async fn pipeline_livez_maps_failed_probe_and_message() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/groups/default/pipelines/main/livez"))
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
            .and(path("/api/v1/groups/default/pipelines/main/livez"))
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
            .and(path("/api/v1/groups/default/pipelines/main/readyz"))
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
            .and(path("/api/v1/groups/default/pipelines/main/livez"))
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
