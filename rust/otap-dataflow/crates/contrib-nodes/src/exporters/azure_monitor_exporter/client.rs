// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;

use otap_df_telemetry::common_attributes::HttpResponse;
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderValue},
};
use tokio::time::{Duration, Instant};

use super::config::ApiConfig;
use super::error::Error;
use super::metrics::AzureMonitorExporterMetricsRc;

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: Duration = Duration::from_secs(3);
const MAX_BACKOFF: Duration = Duration::from_secs(30);
const MAX_IDLE_CONNECTIONS_PER_HOST: usize = 2;

/// HTTP header name for Azure Monitor source resource ID tracking.
pub(super) const AZURE_MONITOR_SOURCE_RESOURCEID_HEADER: &str = "azure-monitor-source-resourceid";

/// URL-encode a value for use in an HTTP header (RFC 3986 percent-encoding).
pub(super) fn url_encode_header_value(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

/// HTTP client for Azure Log Analytics Data Collection Rule (DCR) endpoint.
///
/// Handles authentication, compression, and HTTP communication with the
/// Azure Monitor Logs Ingestion API.
#[derive(Clone)]
pub struct LogsIngestionClient {
    http_client: Client,
    endpoint: String,

    /// Optional ARM resource ID header for Azure Monitor source tracking.
    resource_id_header: Option<HeaderValue>,

    /// Shared metrics tracker for recording HTTP status codes and latency.
    metrics: AzureMonitorExporterMetricsRc,
}

pub struct LogsIngestionClientPool {
    clients: Vec<LogsIngestionClient>,
    metrics: AzureMonitorExporterMetricsRc,
}

impl LogsIngestionClientPool {
    pub fn new(capacity: usize, metrics: AzureMonitorExporterMetricsRc) -> Self {
        Self {
            clients: Vec::with_capacity(capacity),
            metrics,
        }
    }

    fn create_http_clients(
        &self,
        count: usize,
        user_agent: Option<&str>,
    ) -> Result<Vec<Client>, Error> {
        let mut clients = Vec::with_capacity(count);

        for _ in 0..count {
            let mut builder = Client::builder()
                .http1_only()
                .timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(MAX_IDLE_CONNECTIONS_PER_HOST)
                .pool_idle_timeout(Duration::from_secs(90))
                .tcp_nodelay(true);

            if let Some(ua) = user_agent {
                builder = builder.user_agent(ua);
            }

            let http_client = builder.build().map_err(Error::CreateClient)?;

            clients.push(http_client);
        }

        Ok(clients)
    }

    pub async fn initialize(&mut self, config: &ApiConfig) -> Result<(), Error> {
        let http_clients =
            self.create_http_clients(self.clients.capacity(), config.user_agent.as_deref())?;

        for http_client in http_clients {
            let client = LogsIngestionClient::new(config, http_client, self.metrics.clone())?;
            self.clients.push(client);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn take(&mut self) -> LogsIngestionClient {
        self.clients.pop().expect("client pool is empty")
    }

    #[inline(always)]
    pub fn release(&mut self, client: LogsIngestionClient) {
        self.clients.push(client);
    }
}

impl LogsIngestionClient {
    /// Creates a new Azure Monitor logs ingestion client instance from provided components.
    ///
    /// Primarily used for testing.
    ///
    /// # Arguments
    /// * `http_client` - The HTTP client to use for requests
    /// * `endpoint` - The full endpoint URL for the Azure Monitor ingestion API
    ///
    /// # Returns
    /// A configured client instance
    #[must_use]
    pub fn from_parts(
        http_client: Client,
        endpoint: String,
        metrics: AzureMonitorExporterMetricsRc,
    ) -> Self {
        Self {
            http_client,
            endpoint,
            resource_id_header: None,
            metrics,
        }
    }

    /// Creates a new Azure Monitor logs ingestion client instance from the configuration.
    ///
    /// # Arguments
    /// * `config` - The API configuration containing endpoint, DCR, and stream info
    /// * `http_client` - The HTTP client to use for requests
    ///
    /// # Returns
    /// * `Ok(LogsIngestionClient)` - A configured client instance
    /// * `Err(Error)` - If client initialization fails
    pub fn new(
        config: &ApiConfig,
        http_client: Client,
        metrics: AzureMonitorExporterMetricsRc,
    ) -> Result<Self, Error> {
        let endpoint = format!(
            "{}/dataCollectionRules/{}/streams/{}?api-version=2021-11-01-preview",
            config.dcr_endpoint, config.dcr, config.stream_name
        );

        let resource_id_header = config
            .azure_monitor_source_resourceid
            .as_deref()
            .and_then(|v| {
                let encoded = url_encode_header_value(v);
                HeaderValue::from_str(&encoded).ok()
            });

        Ok(Self {
            http_client,
            endpoint,
            resource_id_header,
            metrics,
        })
    }

    /// Export compressed data to Log Analytics ingestion API with automatic retry.
    ///
    /// Retries on:
    /// - Network errors
    /// - 429 (rate limiting) - uses Retry-After header if present
    /// - 5xx (server errors)
    ///
    /// A 401 is not retried here: every attempt would replay `auth_header`, so
    /// recovery belongs to the caller, which invalidates the rejected token and
    /// re-dispatches once a fresh one is cached.
    ///
    /// # Arguments
    /// * `body` - The gzip-compressed JSON data to send
    /// * `auth_header` - The authorization header for this request
    ///
    /// # Returns
    /// * `Ok(Duration)` - Total time spent (including retries) if successful
    /// * `Err(Error)` - Error if all retries are exhausted or a non-retryable error is returned
    pub async fn export(
        &mut self,
        body: Bytes,
        auth_header: &HeaderValue,
    ) -> Result<Duration, Error> {
        let mut attempt = 0u32;
        let mut rng = SmallRng::seed_from_u64(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before UNIX epoch")
                .as_nanos() as u64
                ^ (self as *const _ as u64),
        );

        loop {
            match self.try_export(body.clone(), auth_header).await {
                Ok(duration) => return Ok(duration),
                Err(e) if !e.is_retryable() => {
                    return Err(Error::ExportFailed {
                        attempts: attempt + 1,
                        last_error: Box::new(e),
                    });
                }
                Err(e) => {
                    attempt += 1;

                    // ToDo: Add an upper bound for server-driven retries (429/5xx with
                    // Retry-After). Currently only the non-server-driven path enforces
                    // MAX_RETRIES; a server that perpetually returns 429 with Retry-After
                    // will cause this loop to retry indefinitely.
                    let delay = if let Some(server_delay) = e.retry_after() {
                        let base_delay = server_delay.max(Duration::from_secs(5));
                        let jitter = Duration::from_secs(3)
                            + Duration::from_secs_f64(rng.random::<f64>() * 7.0);
                        base_delay + jitter
                    } else {
                        if attempt >= MAX_RETRIES {
                            return Err(Error::ExportFailed {
                                attempts: attempt,
                                last_error: Box::new(e),
                            });
                        }
                        let backoff = INITIAL_BACKOFF * 2u32.pow(attempt - 1);
                        let base_delay = backoff.min(MAX_BACKOFF);
                        let jitter_factor = 0.85 + rng.random::<f64>() * 0.30;
                        base_delay.mul_f64(jitter_factor)
                    };

                    // TODO: Revisit whether DEBUG or INFO is the right level for retry attempts.
                    otel_debug!("azure_monitor_exporter.export.retrying", attempt = attempt, delay_ms = delay.as_millis() as u64, error = ?e);

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Single export attempt without retry logic.
    async fn try_export(
        &mut self,
        body: Bytes,
        auth_header: &HeaderValue,
    ) -> Result<Duration, Error> {
        let start = Instant::now();

        let mut request = self
            .http_client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_ENCODING, "gzip")
            .header(AUTHORIZATION, auth_header);

        if let Some(ref resource_id) = self.resource_id_header {
            request = request.header(AZURE_MONITOR_SOURCE_RESOURCEID_HEADER, resource_id);
        }

        let response = match request.body(body).send().await {
            Ok(resp) => resp,
            Err(e) => {
                self.metrics.borrow_mut().record_http_attempt(
                    HttpResponse::NetworkError,
                    start.elapsed().as_millis() as f64,
                );
                return Err(Error::network(e));
            }
        };

        let status_code = response.status().as_u16();
        let elapsed = start.elapsed();

        self.metrics.borrow_mut().record_http_attempt(
            http_response_for_status(status_code),
            elapsed.as_millis() as f64,
        );

        if response.status().is_success() {
            return Ok(elapsed);
        }

        // Extract Retry-After header before consuming response
        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs);

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        otel_debug!(
            "azure_monitor_exporter.client.error",
            status = status.as_u16(),
            message = %body
        );

        match status.as_u16() {
            401 => Err(Error::unauthorized(body)),
            403 => Err(Error::forbidden(body)),
            413 => Err(Error::PayloadTooLarge),
            429 => Err(Error::RateLimited { body, retry_after }),
            500..=599 => Err(Error::ServerError {
                status,
                body,
                retry_after,
            }),
            _ => Err(Error::UnexpectedStatus { status, body }),
        }
    }
}

fn http_response_for_status(status: u16) -> HttpResponse {
    match status {
        200..=299 => HttpResponse::Http2xx,
        400 => HttpResponse::Http400,
        401 => HttpResponse::Http401,
        403 => HttpResponse::Http403,
        404 => HttpResponse::Http404,
        413 => HttpResponse::Http413,
        429 => HttpResponse::Http429,
        500..=599 => HttpResponse::Http5xx,
        _ => HttpResponse::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::super::metrics::AzureMonitorExporterMetricsTracker;
    use super::*;
    use otap_df_engine::context::{ControllerContext, PipelineContext};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use reqwest::header::HeaderValue;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // ==================== Test Helpers ====================

    fn create_test_metrics() -> AzureMonitorExporterMetricsRc {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx: PipelineContext =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        Rc::new(RefCell::new(AzureMonitorExporterMetricsTracker::register(
            &pipeline_ctx,
        )))
    }

    fn create_test_api_config() -> ApiConfig {
        ApiConfig {
            dcr_endpoint: "https://test.azure.com".to_string(),
            dcr: "test-dcr".to_string(),
            stream_name: "test-stream".to_string(),
            schema: Default::default(),
            azure_monitor_source_resourceid: None,
            gzip_compression_level: 6,
            user_agent: None,
        }
    }

    fn create_test_http_client() -> Client {
        otap_df_otap::crypto::ensure_crypto_provider();
        Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("failed to create HTTP client")
    }

    /// Scenario: Azure Monitor returns classified client-error and unclassified HTTP statuses.
    /// Guarantees: 400 and 404 have dedicated metric buckets; other statuses use `Other`.
    #[test]
    fn classifies_client_error_and_unexpected_statuses() {
        assert_eq!(http_response_for_status(400), HttpResponse::Http400);
        assert_eq!(http_response_for_status(404), HttpResponse::Http404);
        assert_eq!(http_response_for_status(418), HttpResponse::Other);
    }

    // ==================== Construction Tests ====================

    #[test]
    fn test_new_builds_correct_endpoint() {
        let api_config = ApiConfig {
            dcr_endpoint: "https://test.azure.com".to_string(),
            dcr: "test-dcr-id".to_string(),
            stream_name: "test-stream".to_string(),
            schema: Default::default(),
            azure_monitor_source_resourceid: None,
            gzip_compression_level: 6,
            user_agent: None,
        };

        let http_client = create_test_http_client();

        let client = LogsIngestionClient::new(&api_config, http_client, create_test_metrics())
            .expect("failed to create client");

        assert_eq!(
            client.endpoint,
            "https://test.azure.com/dataCollectionRules/test-dcr-id/streams/test-stream?api-version=2021-11-01-preview"
        );
    }

    #[test]
    fn test_new_with_special_characters_in_config() {
        let api_config = ApiConfig {
            dcr_endpoint: "https://my-endpoint.azure.com".to_string(),
            dcr: "dcr-abc-123-def".to_string(),
            stream_name: "Custom-Stream_Name".to_string(),
            schema: Default::default(),
            azure_monitor_source_resourceid: None,
            gzip_compression_level: 6,
            user_agent: None,
        };

        let http_client = create_test_http_client();

        let client =
            LogsIngestionClient::new(&api_config, http_client, create_test_metrics()).unwrap();

        assert!(client.endpoint.contains("dcr-abc-123-def"));
        assert!(client.endpoint.contains("Custom-Stream_Name"));
    }

    #[test]
    fn test_from_parts_creates_client() {
        let client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com/endpoint".to_string(),
            create_test_metrics(),
        );

        assert_eq!(client.endpoint, "https://example.com/endpoint");
    }

    // ==================== Export Tests ====================

    /// Scenario: an export is dispatched with a caller-supplied bearer header.
    /// Guarantees: that exact header reaches the ingestion endpoint, so the
    /// credential a request carries is the one its caller stamped on it.
    #[tokio::test]
    async fn export_sends_the_supplied_authorization_header() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer supplied-token"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            mock_server.uri(),
            create_test_metrics(),
        );

        let result = client
            .export(
                Bytes::from_static(b"payload"),
                &HeaderValue::from_static("Bearer supplied-token"),
            )
            .await;

        assert!(result.is_ok(), "expected success, got {result:?}");
    }

    /// Scenario: one pooled client dispatches two exports carrying different
    /// bearer headers.
    /// Guarantees: each request sends its own header, so a client cannot leak a
    /// credential from an earlier export into a later one.
    #[tokio::test]
    async fn export_uses_the_header_supplied_for_each_request() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer first"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer second"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            mock_server.uri(),
            create_test_metrics(),
        );

        for token in ["Bearer first", "Bearer second"] {
            let _ = client
                .export(
                    Bytes::from_static(b"payload"),
                    &HeaderValue::from_str(token).unwrap(),
                )
                .await
                .expect("export should succeed");
        }
    }

    /// Scenario: the ingestion endpoint rejects an export with HTTP 401.
    /// Guarantees: the client fails after a single attempt instead of replaying
    /// the same rejected token, and reports the failure as an auth rejection so
    /// the caller can invalidate that token.
    #[tokio::test]
    async fn unauthorized_export_is_not_retried() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401).set_body_string("expired"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            mock_server.uri(),
            create_test_metrics(),
        );

        let error = client
            .export(
                Bytes::from_static(b"payload"),
                &HeaderValue::from_static("Bearer stale"),
            )
            .await
            .expect_err("401 should fail the export");

        assert!(error.is_unauthorized());
        assert!(matches!(error, Error::ExportFailed { attempts: 1, .. }));
        assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
    }

    /// Scenario: the ingestion endpoint rejects an export with HTTP 403.
    /// Guarantees: the client fails after a single attempt and does not report a
    /// permission problem as an auth rejection, since refreshing the token
    /// cannot resolve it.
    #[tokio::test]
    async fn forbidden_export_is_not_retried() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(403).set_body_string("denied"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            mock_server.uri(),
            create_test_metrics(),
        );

        let error = client
            .export(
                Bytes::from_static(b"payload"),
                &HeaderValue::from_static("Bearer scoped-out"),
            )
            .await
            .expect_err("403 should fail the export");

        assert!(!error.is_unauthorized());
        assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
    }

    /// Scenario: a client configured with an Azure Monitor source resource ID
    /// exports a batch.
    /// Guarantees: the resource ID header accompanies the request alongside the
    /// per-request bearer header.
    #[tokio::test]
    async fn export_sends_the_configured_resource_id_header() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header(AZURE_MONITOR_SOURCE_RESOURCEID_HEADER, "sub%2Frg"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut api_config = create_test_api_config();
        api_config.dcr_endpoint = mock_server.uri();
        api_config.azure_monitor_source_resourceid = Some("sub/rg".to_string());
        let mut client = LogsIngestionClient::new(
            &api_config,
            create_test_http_client(),
            create_test_metrics(),
        )
        .unwrap();

        let _ = client
            .export(
                Bytes::from_static(b"payload"),
                &HeaderValue::from_static("Bearer token"),
            )
            .await
            .expect("export should succeed");
    }

    // ==================== LogsIngestionClientPool Tests ====================

    #[test]
    fn test_pool_new_creates_empty_pool() {
        let pool = LogsIngestionClientPool::new(5, create_test_metrics());

        assert_eq!(pool.clients.capacity(), 5);
        assert_eq!(pool.clients.len(), 0);
    }

    #[test]
    fn test_pool_new_various_capacities() {
        for capacity in [1, 4, 8, 32, 100] {
            let pool = LogsIngestionClientPool::new(capacity, create_test_metrics());
            assert_eq!(pool.clients.capacity(), capacity);
        }
    }

    #[test]
    fn test_pool_take_and_release_single() {
        let api_config = create_test_api_config();

        let mut pool = LogsIngestionClientPool::new(1, create_test_metrics());
        let client = LogsIngestionClient::new(
            &api_config,
            create_test_http_client(),
            create_test_metrics(),
        )
        .unwrap();
        pool.clients.push(client);

        assert_eq!(pool.clients.len(), 1);

        let taken = pool.take();
        assert_eq!(pool.clients.len(), 0);

        pool.release(taken);
        assert_eq!(pool.clients.len(), 1);
    }

    #[test]
    fn test_pool_take_and_release_multiple() {
        let api_config = create_test_api_config();

        let mut pool = LogsIngestionClientPool::new(3, create_test_metrics());
        for _ in 0..3 {
            let client = LogsIngestionClient::new(
                &api_config,
                create_test_http_client(),
                create_test_metrics(),
            )
            .unwrap();
            pool.clients.push(client);
        }

        assert_eq!(pool.clients.len(), 3);

        // Take all
        let c1 = pool.take();
        let c2 = pool.take();
        let c3 = pool.take();
        assert_eq!(pool.clients.len(), 0);

        // Release in different order
        pool.release(c2);
        pool.release(c1);
        pool.release(c3);
        assert_eq!(pool.clients.len(), 3);
    }

    #[test]
    fn test_pool_release_beyond_capacity() {
        let api_config = create_test_api_config();

        let mut pool = LogsIngestionClientPool::new(1, create_test_metrics());

        // Release more than capacity (Vec will grow)
        for _ in 0..5 {
            let client = LogsIngestionClient::new(
                &api_config,
                create_test_http_client(),
                create_test_metrics(),
            )
            .unwrap();
            pool.release(client);
        }

        assert_eq!(pool.clients.len(), 5);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_client_clone_has_same_endpoint() {
        let client1 = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com/endpoint".to_string(),
            create_test_metrics(),
        );

        let client2 = client1.clone();

        assert_eq!(client1.endpoint, client2.endpoint);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_client_with_empty_endpoint() {
        let client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "".to_string(),
            create_test_metrics(),
        );

        assert_eq!(client.endpoint, "");
    }

    #[test]
    fn test_pool_create_http_clients() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let pool = LogsIngestionClientPool::new(4, create_test_metrics());

        let result = pool.create_http_clients(4, None);

        assert!(result.is_ok());
        let clients = result.unwrap();
        assert_eq!(clients.len(), 4);
    }

    #[test]
    fn test_pool_create_http_clients_zero() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let pool = LogsIngestionClientPool::new(4, create_test_metrics());

        let result = pool.create_http_clients(0, None);

        assert!(result.is_ok());
        let clients = result.unwrap();
        assert_eq!(clients.len(), 0);
    }

    #[test]
    fn test_pool_create_http_clients_with_user_agent() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let pool = LogsIngestionClientPool::new(4, create_test_metrics());

        let result = pool.create_http_clients(4, Some("my-app/1.0"));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 4);
    }

    // ==================== Resource ID Header Tests ====================

    #[test]
    fn test_resource_id_header_set_when_configured() {
        let input = "/subscriptions/215b5735-fa8b-4dd4-86dc-997320c68c2d/resourceGroups/rg-test/providers/Microsoft.Kubernetes/connectedClusters/test-cluster/providers/microsoft.kubernetesconfiguration/extensions/pipe";
        let encoded = url_encode_header_value(input);
        let header = HeaderValue::from_str(&encoded).expect("valid header value");

        let expected = "%2Fsubscriptions%2F215b5735-fa8b-4dd4-86dc-997320c68c2d%2FresourceGroups%2Frg-test%2Fproviders%2FMicrosoft.Kubernetes%2FconnectedClusters%2Ftest-cluster%2Fproviders%2Fmicrosoft.kubernetesconfiguration%2Fextensions%2Fpipe";
        assert_eq!(header.to_str().unwrap(), expected);
    }

    #[test]
    fn test_resource_id_header_none_when_not_configured() {
        let api_config = create_test_api_config();
        assert!(api_config.azure_monitor_source_resourceid.is_none());
    }
}
