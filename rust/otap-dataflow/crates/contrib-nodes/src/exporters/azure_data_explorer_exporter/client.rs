// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP client for ADX Kusto streaming ingestion.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the transport is wired into the exporter runtime in a follow-up phase"
    )
)]

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_otap::metrics::ExporterAttempt;
use otel_arrow_dfe_telemetry::common_attributes::HttpResponse;
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderName, HeaderValue, RETRY_AFTER},
};
use std::time::SystemTime;
use tokio::time::{Duration, Instant};

use super::error::Error;
use super::metrics::AzureDataExplorerExporterMetricsRc;

/// Initial backoff delay between retries.
const INITIAL_BACKOFF: Duration = Duration::from_secs(3);
/// Maximum backoff delay cap.
const MAX_BACKOFF: Duration = Duration::from_secs(30);
const RESPONSE_DETAIL_CHUNK_LENGTH: usize = 512;
const MAX_ERROR_RESPONSE_BYTES: usize = 64 * 1024;
const MAX_RESPONSE_DETAIL_CHUNKS: usize =
    MAX_ERROR_RESPONSE_BYTES.div_ceil(RESPONSE_DETAIL_CHUNK_LENGTH);

/// Values available for every physical submission of one ADX batch.
#[derive(Clone, Copy)]
pub(super) struct ExportAttemptMetadata {
    pub(super) signal: SignalType,
    pub(super) items: u64,
    pub(super) payload_size: usize,
}

/// HTTP client for ADX Kusto streaming ingestion endpoint.
///
/// Not bound to a single destination table: the target table (and optional
/// JSON mapping) are supplied per `send` call so one pool of clients can
/// serve logs, metrics, and traces batches alike.
#[derive(Clone)]
pub struct AzureDataExplorerClient {
    http_client: Client,
    cluster_uri: String,
    database: String,
    metrics: AzureDataExplorerExporterMetricsRc,
    max_retries: u32,
    timeout: Duration,
    log_response_body: bool,
}

impl AzureDataExplorerClient {
    /// Emit a bounded ADX response prefix in a fixed maximum number of chunks.
    /// The compact failure event is useful for structured telemetry, but its
    /// single-line summary cannot preserve multiline response detail.
    fn log_response_detail(status: reqwest::StatusCode, endpoint: &str, body: &str) {
        let chunks = Self::response_detail_chunks(body);
        let chunk_count = chunks.len();
        for (index, chunk) in chunks.into_iter().enumerate() {
            otel_debug!(
                "azure_data_explorer_exporter.client.response_detail",
                message = chunk,
                status = status.as_u16(),
                endpoint = endpoint,
                chunk_index = index as u64,
                chunk_count = chunk_count as u64
            );
        }
    }

    fn response_detail_chunks(body: &str) -> Vec<&str> {
        if body.is_empty() {
            return vec!["<empty response body>"];
        }

        let mut chunks = Vec::with_capacity(
            body.len()
                .div_ceil(RESPONSE_DETAIL_CHUNK_LENGTH)
                .min(MAX_RESPONSE_DETAIL_CHUNKS),
        );
        let mut remaining = body;
        while !remaining.is_empty() && chunks.len() < MAX_RESPONSE_DETAIL_CHUNKS {
            let mut end = remaining.len().min(RESPONSE_DETAIL_CHUNK_LENGTH);
            while !remaining.is_char_boundary(end) {
                end -= 1;
            }
            chunks.push(&remaining[..end]);
            remaining = &remaining[end..];
        }
        chunks
    }

    /// Build the streaming-ingestion endpoint URL for a given table.
    fn build_endpoint(
        cluster_uri: &str,
        database: &str,
        table: &str,
        json_mapping: Option<&str>,
    ) -> String {
        let base = cluster_uri.trim_end_matches('/');
        let database = urlencoding::encode(database);
        let table = urlencoding::encode(table);
        let mut endpoint = format!("{base}/v1/rest/ingest/{database}/{table}?streamFormat=JSON",);
        if let Some(mapping) = json_mapping.filter(|mapping| !mapping.is_empty()) {
            endpoint.push_str("&mappingName=");
            endpoint.push_str(&urlencoding::encode(mapping));
        }
        endpoint
    }

    /// Create a new client bound to a cluster and database, but no specific table.
    pub fn new(
        cluster_uri: &str,
        database: &str,
        http_client: Client,
        metrics: AzureDataExplorerExporterMetricsRc,
        max_retries: u32,
        timeout: Duration,
        log_response_body: bool,
    ) -> Self {
        Self {
            http_client,
            cluster_uri: cluster_uri.to_owned(),
            database: database.to_owned(),
            metrics,
            max_retries,
            timeout,
            log_response_body,
        }
    }

    /// Send a gzip-compressed batch of JSON records to the given table.
    pub(super) async fn send(
        &self,
        table: &str,
        json_mapping: Option<&str>,
        auth_header: HeaderValue,
        compressed_data: Bytes,
        first_attempt: ExporterAttempt,
        metadata: ExportAttemptMetadata,
    ) -> Result<Duration, Error> {
        let endpoint = Self::build_endpoint(&self.cluster_uri, &self.database, table, json_mapping);
        let mut rng = SmallRng::seed_from_u64(
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos() as u64)
                ^ (self as *const _ as u64),
        );
        let mut attempt = 0;
        let start = Instant::now();
        let deadline = start + self.timeout;
        let operation_id = uuid::Uuid::new_v4();
        let encoded_table = urlencoding::encode(table);
        let mut first_attempt = Some(first_attempt);

        loop {
            attempt += 1;
            let export_attempt = first_attempt
                .take()
                .unwrap_or_else(|| self.new_attempt(metadata));
            let result = self
                .try_send(
                    table,
                    json_mapping,
                    &endpoint,
                    auth_header.clone(),
                    &encoded_table,
                    operation_id,
                    attempt,
                    deadline,
                    start,
                    compressed_data.clone(),
                    export_attempt,
                    metadata,
                )
                .await;

            match result {
                Ok(duration) => return Ok(duration),
                Err(error) if !error.is_retryable() => return Err(error),
                Err(error) if attempt > self.max_retries => {
                    return Err(Error::ExportFailed {
                        attempts: attempt,
                        last_error: Box::new(error),
                    });
                }
                Err(error) => {
                    match &error {
                        Error::Network { .. } | Error::ResponseBody { .. } => {
                            otel_warn!(
                                "azure_data_explorer_exporter.client.network_error",
                                attempt = attempt,
                                error = %error
                            );
                        }
                        Error::RateLimited { .. } => {
                            otel_warn!(
                                "azure_data_explorer_exporter.client.rate_limited",
                                attempt = attempt
                            );
                        }
                        Error::ServerError { status, .. } => {
                            otel_warn!(
                                "azure_data_explorer_exporter.client.server_error",
                                attempt = attempt,
                                status = status.as_u16()
                            );
                        }
                        _ => {}
                    }
                    let retry_after = error.retry_after();
                    Self::backoff(&mut rng, attempt, deadline, self.timeout, retry_after).await?;
                }
            }
        }
    }

    fn new_attempt(&self, metadata: ExportAttemptMetadata) -> ExporterAttempt {
        let mut attempt = self.metrics.borrow().boundary.attempt(metadata.signal);
        attempt.set_item_count_with(|| metadata.items);
        attempt.set_payload_size_with(|| metadata.payload_size);
        attempt
    }

    #[allow(clippy::too_many_arguments)]
    async fn try_send(
        &self,
        table: &str,
        json_mapping: Option<&str>,
        endpoint: &str,
        auth_header: HeaderValue,
        encoded_table: &str,
        operation_id: uuid::Uuid,
        attempt: u32,
        deadline: Instant,
        operation_start: Instant,
        compressed_data: Bytes,
        export_attempt: ExporterAttempt,
        metadata: ExportAttemptMetadata,
    ) -> Result<Duration, Error> {
        let completed = export_attempt
            .run(async |attempt_metrics| {
                attempt_metrics.set_item_count_with(|| metadata.items);
                attempt_metrics.set_payload_size_with(|| metadata.payload_size);
                let result = tokio::time::timeout_at(
                    deadline,
                    self.try_send_request(
                        table,
                        json_mapping,
                        endpoint,
                        auth_header,
                        encoded_table,
                        operation_id,
                        attempt,
                        operation_start,
                        compressed_data,
                    ),
                )
                .await
                .map_err(|_| {
                    self.metrics
                        .borrow_mut()
                        .record_http_response(HttpResponse::NetworkError);
                    Error::OperationTimeout {
                        timeout: self.timeout,
                    }
                })?;
                match result {
                    Ok(duration) => Ok(duration),
                    Err(error) if error.is_refusal() => Err(attempt_metrics.refused(error)),
                    Err(error) => Err(attempt_metrics.failed(error)),
                }
            })
            .await;
        self.metrics.borrow_mut().boundary.record(completed)
    }

    #[allow(clippy::too_many_arguments)]
    async fn try_send_request(
        &self,
        table: &str,
        json_mapping: Option<&str>,
        endpoint: &str,
        auth_header: HeaderValue,
        encoded_table: &str,
        operation_id: uuid::Uuid,
        attempt: u32,
        operation_start: Instant,
        compressed_data: Bytes,
    ) -> Result<Duration, Error> {
        let response = self
            .http_client
            .post(endpoint)
            .header(AUTHORIZATION, auth_header)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_ENCODING, "gzip")
            .header(
                HeaderName::from_static("x-ms-client-request-id"),
                format!("otap-dfe-azure-data-explorer;table={encoded_table};attempt={attempt};{operation_id};"),
            )
            .body(compressed_data.clone())
            .send()
            .await
            .map_err(|error| {
                self.metrics
                    .borrow_mut()
                    .record_http_response(HttpResponse::NetworkError);
                Error::from_reqwest(error)
            })?;
        let status = response.status();

        if status.is_success() {
            self.metrics
                .borrow_mut()
                .record_http_response(Self::classify_http_response(status));
            let duration = operation_start.elapsed();
            return Ok(duration);
        }

        let retry_after = Self::parse_retry_after(&response);
        let (body, response_bytes, response_truncated) =
            match Self::read_error_response(response, status, retry_after).await {
                Ok(response) => response,
                Err(error) => {
                    self.metrics
                        .borrow_mut()
                        .record_http_response(HttpResponse::NetworkError);
                    return Err(error);
                }
            };
        self.metrics
            .borrow_mut()
            .record_http_response(Self::classify_http_response(status));
        otel_warn!(
            "azure_data_explorer_exporter.client.http_failure",
            status = status.as_u16(),
            endpoint = endpoint,
            table = table,
            json_mapping = json_mapping.unwrap_or(""),
            attempt = attempt,
            response_bytes = response_bytes,
            response_truncated = response_truncated,
            response_body_logging_enabled = self.log_response_body,
            message = "ADX rejected the ingestion request"
        );

        if self.log_response_body {
            Self::log_response_detail(status, endpoint, &body);
        }

        match status.as_u16() {
            401 => Err(Error::UnexpectedStatus { status, body }),
            403 => Err(Error::UnexpectedStatus { status, body }),
            409 => {
                otel_warn!(
                    "azure_data_explorer_exporter.client.conflict",
                    hint = "409 Conflict usually means streaming ingestion is not enabled. \
                            Run '.alter table <TABLE> policy streamingingestion enable' in ADX \
                            and ensure streaming ingestion is enabled at the cluster level."
                );
                Err(Error::UnexpectedStatus { status, body })
            }
            429 => Err(Error::RateLimited { body, retry_after }),
            500..=599 => Err(Error::ServerError {
                status,
                body,
                retry_after,
            }),
            _ => Err(Error::UnexpectedStatus { status, body }),
        }
    }

    fn classify_http_response(status: reqwest::StatusCode) -> HttpResponse {
        match status.as_u16() {
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

    fn parse_retry_after(response: &reqwest::Response) -> Option<Duration> {
        let value = response.headers().get(RETRY_AFTER)?.to_str().ok()?;
        if let Ok(seconds) = value.parse::<u64>() {
            return Some(Duration::from_secs(seconds));
        }
        let retry_at = httpdate::parse_http_date(value).ok()?;
        retry_at.duration_since(SystemTime::now()).ok()
    }

    async fn read_error_response(
        mut response: reqwest::Response,
        status: reqwest::StatusCode,
        retry_after: Option<Duration>,
    ) -> Result<(String, u64, bool), Error> {
        let advertised_size = response.content_length();
        let mut bytes = Vec::with_capacity(
            advertised_size
                .unwrap_or(0)
                .min(MAX_ERROR_RESPONSE_BYTES as u64) as usize,
        );
        let mut truncated =
            advertised_size.is_some_and(|size| size > MAX_ERROR_RESPONSE_BYTES as u64);
        while bytes.len() < MAX_ERROR_RESPONSE_BYTES {
            let Some(chunk) = response
                .chunk()
                .await
                .map_err(|source| Error::ResponseBody {
                    status,
                    retry_after,
                    source,
                })?
            else {
                break;
            };
            let remaining = MAX_ERROR_RESPONSE_BYTES - bytes.len();
            if chunk.len() > remaining {
                bytes.extend_from_slice(&chunk[..remaining]);
                truncated = true;
                break;
            }
            bytes.extend_from_slice(&chunk);
        }
        if bytes.len() == MAX_ERROR_RESPONSE_BYTES && !truncated {
            truncated = response
                .chunk()
                .await
                .map_err(|source| Error::ResponseBody {
                    status,
                    retry_after,
                    source,
                })?
                .is_some();
        }
        let retained_size = bytes.len() as u64;
        let response_size = advertised_size.unwrap_or(retained_size);
        let mut body = String::from_utf8_lossy(&bytes).into_owned();
        if truncated {
            body.push_str("\n[response body truncated]");
        }
        Ok((body, response_size, truncated))
    }

    async fn backoff(
        rng: &mut SmallRng,
        attempt: u32,
        deadline: Instant,
        timeout: Duration,
        retry_after: Option<Duration>,
    ) -> Result<(), Error> {
        let jitter_factor = 0.7 + rng.random_range(0.0..0.6_f64);
        let requested_delay = Self::retry_delay(attempt, jitter_factor, retry_after);
        let delay = requested_delay.min(deadline.saturating_duration_since(Instant::now()));
        otel_debug!(
            "azure_data_explorer_exporter.client.backoff",
            attempt = attempt,
            delay_ms = delay.as_millis() as u64
        );
        tokio::time::timeout_at(deadline, tokio::time::sleep(delay))
            .await
            .map_err(|_| Error::OperationTimeout { timeout })?;
        if Instant::now() >= deadline {
            return Err(Error::OperationTimeout { timeout });
        }
        Ok(())
    }

    fn retry_delay(attempt: u32, jitter_factor: f64, retry_after: Option<Duration>) -> Duration {
        let base = INITIAL_BACKOFF * 2u32.saturating_pow(attempt.saturating_sub(1));
        let capped = base.min(MAX_BACKOFF);
        let client_delay = capped.mul_f64(jitter_factor).min(MAX_BACKOFF);
        retry_after.map_or(client_delay, |server_delay| server_delay.max(client_delay))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AzureDataExplorerClient, ExportAttemptMetadata, MAX_BACKOFF, MAX_ERROR_RESPONSE_BYTES,
        MAX_RESPONSE_DETAIL_CHUNKS, RESPONSE_DETAIL_CHUNK_LENGTH,
    };
    use crate::exporters::azure_data_explorer_exporter::error::Error;
    use crate::exporters::azure_data_explorer_exporter::metrics::{
        AzureDataExplorerExporterMetricsRc, AzureDataExplorerExporterMetricsTracker,
    };
    use bytes::Bytes;
    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
    use otel_arrow_dfe_otap::metrics::ExporterAttempt;
    use otel_arrow_dfe_telemetry::common_attributes::Outcome;
    use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
    use reqwest::header::HeaderValue;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use wiremock::matchers::{body_bytes, header, header_regex, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Scenario: ADX returns a numeric Retry-After header.
    /// Guarantees: the client preserves the server's minimum retry delay.
    #[tokio::test]
    async fn retry_after_seconds_are_parsed() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "17"))
            .mount(&server)
            .await;

        let response = reqwest::Client::new()
            .get(server.uri())
            .send()
            .await
            .expect("mock response");

        assert_eq!(
            AzureDataExplorerClient::parse_retry_after(&response),
            Some(Duration::from_secs(17))
        );
    }

    /// Scenario: ADX or an intermediary returns an oversized error body.
    /// Guarantees: the client retains only the bounded diagnostic prefix and marks truncation.
    #[tokio::test]
    async fn error_response_body_is_bounded() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500).set_body_bytes(vec![
                b'x';
                MAX_ERROR_RESPONSE_BYTES
                    + 1024
            ]))
            .mount(&server)
            .await;
        let response = reqwest::Client::new()
            .get(server.uri())
            .send()
            .await
            .expect("mock response");
        let status = response.status();

        let (body, response_bytes, truncated) =
            AzureDataExplorerClient::read_error_response(response, status, None)
                .await
                .expect("bounded body");

        assert!(truncated);
        assert_eq!(response_bytes, (MAX_ERROR_RESPONSE_BYTES + 1024) as u64);
        assert!(body.ends_with("[response body truncated]"));
        assert!(body.len() < MAX_ERROR_RESPONSE_BYTES + 64);
    }

    /// Scenario: a retained ADX response body contains one newline per byte.
    /// Guarantees: debug response logging emits a fixed maximum number of byte-bounded chunks.
    #[test]
    fn response_detail_chunks_bound_newline_amplification() {
        let body = "\n".repeat(MAX_ERROR_RESPONSE_BYTES);

        let chunks = AzureDataExplorerClient::response_detail_chunks(&body);

        assert_eq!(chunks.len(), MAX_RESPONSE_DETAIL_CHUNKS);
        assert!(
            chunks
                .iter()
                .all(|chunk| chunk.len() <= RESPONSE_DETAIL_CHUNK_LENGTH)
        );
        assert_eq!(chunks.concat(), body);
    }

    async fn truncated_error_response(status_line: &str, retry_after: Option<u64>) -> Error {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let retry_after_header =
            retry_after.map_or_else(String::new, |seconds| format!("Retry-After: {seconds}\r\n"));
        let response = format!(
            "HTTP/1.1 {status_line}\r\n\
             Content-Length: 7\r\n\
             {retry_after_header}\
             Connection: close\r\n\r\n\
             x"
        );
        let server = async move {
            let (mut stream, _) = listener.accept().await.expect("client connection");
            let mut request = vec![0_u8; 4096];
            let _ = stream.read(&mut request).await.expect("request bytes");
            stream
                .write_all(response.as_bytes())
                .await
                .expect("truncated response");
        };
        let request = reqwest::Client::new()
            .get(format!("http://{address}"))
            .send();

        let (_, response) = tokio::join!(server, request);
        let response = response.expect("response headers");
        let status = response.status();
        let retry_after = AzureDataExplorerClient::parse_retry_after(&response);
        AzureDataExplorerClient::read_error_response(response, status, retry_after)
            .await
            .expect_err("truncated body")
    }

    /// Scenario: retryable HTTP statuses close a declared response body before it is complete.
    /// Guarantees: 429/503 preserve `Retry-After`, and a truncated 408 remains retryable.
    #[tokio::test]
    async fn truncated_retryable_response_preserves_retry_policy() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        for status_line in ["429 Too Many Requests", "503 Service Unavailable"] {
            let error = truncated_error_response(status_line, Some(60)).await;
            assert!(error.is_retryable(), "{status_line}");
            assert_eq!(
                error.retry_after(),
                Some(Duration::from_secs(60)),
                "{status_line}"
            );
        }

        let error = truncated_error_response("408 Request Timeout", None).await;
        assert!(error.is_retryable());
        assert_eq!(error.retry_after(), None);
    }

    /// Scenario: the local exponential delay reaches its cap before positive jitter is applied.
    /// Guarantees: jitter cannot exceed 30 seconds, while `Retry-After` may require a longer delay.
    #[test]
    fn retry_delay_caps_jitter_before_server_minimum() {
        assert_eq!(
            AzureDataExplorerClient::retry_delay(5, 1.2, None),
            MAX_BACKOFF
        );
        assert_eq!(
            AzureDataExplorerClient::retry_delay(5, 1.2, Some(Duration::from_secs(60))),
            Duration::from_secs(60)
        );
    }

    fn test_metrics() -> AzureDataExplorerExporterMetricsRc {
        let interests = Interests::NODE_INPUT_METRICS
            | Interests::NODE_LOCAL_DURATION
            | Interests::NODE_ITEM_COUNTS
            | Interests::NODE_SIZE;
        let (pipeline_ctx, _registry) = test_pipeline_ctx_with_interests(interests);
        Rc::new(RefCell::new(
            AzureDataExplorerExporterMetricsTracker::register(&pipeline_ctx),
        ))
    }

    fn export_attempt(
        metrics: &AzureDataExplorerExporterMetricsRc,
        items: u64,
        payload_size: usize,
    ) -> (ExporterAttempt, ExportAttemptMetadata) {
        let metadata = ExportAttemptMetadata {
            signal: SignalType::Logs,
            items,
            payload_size,
        };
        let mut attempt = metrics.borrow().boundary.attempt(metadata.signal);
        attempt.set_item_count_with(|| metadata.items);
        attempt.set_payload_size_with(|| metadata.payload_size);
        (attempt, metadata)
    }

    fn attempted_metric(snapshots: &[MetricSetSnapshot], outcome: Outcome, metric: &str) -> u64 {
        let outcome = match outcome {
            Outcome::Success => "success",
            Outcome::Failure => "failure",
            Outcome::Refused => "refused",
        };
        let snapshot = snapshots
            .iter()
            .find(|snapshot| {
                snapshot.descriptor().name == "exporter.attempted"
                    && snapshot.measurement_attribute_value("signal") == Some("logs")
                    && snapshot.measurement_attribute_value("outcome") == Some(outcome)
                    && snapshot
                        .descriptor()
                        .metrics
                        .iter()
                        .any(|candidate| candidate.name == metric)
            })
            .unwrap_or_else(|| panic!("exporter attempt snapshot for {metric}"));
        let index = snapshot
            .descriptor()
            .metrics
            .iter()
            .position(|candidate| candidate.name == metric)
            .unwrap_or_else(|| panic!("{metric} metric"));
        snapshot.get_metrics()[index].to_u64_lossy()
    }

    fn attempted_recorded(snapshots: &[MetricSetSnapshot], outcome: Outcome, metric: &str) -> bool {
        let outcome = match outcome {
            Outcome::Success => "success",
            Outcome::Failure => "failure",
            Outcome::Refused => "refused",
        };
        snapshots.iter().any(|snapshot| {
            if snapshot.descriptor().name != "exporter.attempted"
                || snapshot.measurement_attribute_value("signal") != Some("logs")
                || snapshot.measurement_attribute_value("outcome") != Some(outcome)
            {
                return false;
            }
            snapshot
                .descriptor()
                .metrics
                .iter()
                .position(|candidate| candidate.name == metric)
                .is_some_and(|index| !snapshot.get_metrics()[index].is_zero())
        })
    }

    fn http_response_metric(snapshots: &[MetricSetSnapshot], response: &str) -> u64 {
        let snapshot = snapshots
            .iter()
            .find(|snapshot| {
                snapshot.descriptor().name == "exporter.azure_data_explorer.http"
                    && snapshot.measurement_attribute_value("response") == Some(response)
            })
            .expect("HTTP response snapshot");
        let index = snapshot
            .descriptor()
            .metrics
            .iter()
            .position(|metric| metric.name == "responses")
            .expect("responses metric");
        snapshot.get_metrics()[index].to_u64_lossy()
    }

    /// Scenario: database, table, and mapping names contain URL-reserved characters.
    /// Guarantees: the streaming ingestion endpoint encodes every user-controlled component.
    #[test]
    fn endpoint_encodes_path_and_mapping_components() {
        let endpoint = AzureDataExplorerClient::build_endpoint(
            "https://example.kusto.windows.net/",
            "db/name",
            "table name",
            Some("mapping+name"),
        );

        assert_eq!(
            endpoint,
            "https://example.kusto.windows.net/v1/rest/ingest/db%2Fname/table%20name?streamFormat=JSON&mappingName=mapping%2Bname"
        );
    }

    /// Scenario: no JSON mapping is configured for a streaming ingestion request.
    /// Guarantees: the endpoint omits the mapping query parameter.
    #[test]
    fn endpoint_omits_absent_mapping() {
        let endpoint = AzureDataExplorerClient::build_endpoint(
            "https://example.kusto.windows.net",
            "db",
            "table",
            None,
        );

        assert_eq!(
            endpoint,
            "https://example.kusto.windows.net/v1/rest/ingest/db/table?streamFormat=JSON"
        );
    }

    /// Scenario: an environment-substituted mapping name is an empty string.
    /// Guarantees: the endpoint treats the empty value as an omitted mapping.
    #[test]
    fn endpoint_omits_empty_mapping() {
        let endpoint = AzureDataExplorerClient::build_endpoint(
            "https://example.kusto.windows.net",
            "db",
            "table",
            Some(""),
        );

        assert_eq!(
            endpoint,
            "https://example.kusto.windows.net/v1/rest/ingest/db/table?streamFormat=JSON"
        );
    }

    /// Scenario: ADX accepts a managed streaming request with a named JSON mapping.
    /// Guarantees: the client sends authorization, request identity, gzip metadata, stable endpoint fields, and the exact payload.
    #[tokio::test]
    async fn sends_managed_streaming_request() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let server = MockServer::start().await;
        let payload = Bytes::from_static(b"compressed-payload");
        Mock::given(method("POST"))
            .and(path("/v1/rest/ingest/db/table"))
            .and(query_param("streamFormat", "JSON"))
            .and(query_param("mappingName", "mapping"))
            .and(header("authorization", "Bearer test"))
            .and(header("content-encoding", "gzip"))
            .and(header("content-type", "application/json"))
            .and(header_regex(
                "x-ms-client-request-id",
                r"^otap-dfe-azure-data-explorer;table=table;attempt=1;[0-9a-f-]{36};$",
            ))
            .and(body_bytes(payload.clone()))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let metrics = test_metrics();
        let client = AzureDataExplorerClient::new(
            &server.uri(),
            "db",
            reqwest::Client::new(),
            metrics.clone(),
            1,
            Duration::from_secs(5),
            false,
        );

        let (attempt, metadata) = export_attempt(&metrics, 3, payload.len());
        let _duration = client
            .send(
                "table",
                Some("mapping"),
                HeaderValue::from_static("Bearer test"),
                payload,
                attempt,
                metadata,
            )
            .await
            .expect("managed streaming request should succeed");

        let snapshots = metrics.borrow_mut().boundary.terminal_snapshots();
        assert_eq!(
            attempted_metric(&snapshots, Outcome::Success, "messages"),
            1
        );
        assert_eq!(attempted_metric(&snapshots, Outcome::Success, "items"), 3);
        assert_eq!(
            attempted_metric(&snapshots, Outcome::Success, "payload.size"),
            b"compressed-payload".len() as u64
        );
        assert!(attempted_recorded(&snapshots, Outcome::Success, "duration"));
    }

    /// Scenario: an ADX streaming-ingestion endpoint is built for JSON Lines records.
    /// Guarantees: the request declares ADX's `JSON` stream format rather than the broader `MultiJSON` format.
    #[test]
    fn endpoint_declares_json_lines_stream_format() {
        let endpoint = AzureDataExplorerClient::build_endpoint(
            "https://example.kusto.windows.net",
            "database",
            "table",
            Some("mapping"),
        );

        assert_eq!(
            endpoint,
            "https://example.kusto.windows.net/v1/rest/ingest/database/table?streamFormat=JSON&mappingName=mapping"
        );
    }

    /// Scenario: ADX endpoint components contain reserved path and query characters.
    /// Guarantees: database, table, and mapping values are percent-encoded independently.
    #[test]
    fn endpoint_encodes_dynamic_components() {
        let endpoint = AzureDataExplorerClient::build_endpoint(
            "https://example.kusto.windows.net/",
            "database/name",
            "table name",
            Some("mapping&version=1"),
        );

        assert_eq!(
            endpoint,
            "https://example.kusto.windows.net/v1/rest/ingest/database%2Fname/table%20name?streamFormat=JSON&mappingName=mapping%26version%3D1"
        );
    }

    /// Scenario: ADX returns 429 or 500 with no retries configured.
    /// Guarantees: exhaustion reports one attempt and preserves refusal/failure classification in errors and metrics.
    #[tokio::test]
    async fn classifies_shared_attempt_outcomes() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let metrics = test_metrics();
        let payload = Bytes::from_static(b"payload");
        let max_retries = 0;

        for status in [429, 500] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status))
                .expect(1)
                .mount(&server)
                .await;
            let client = AzureDataExplorerClient::new(
                &server.uri(),
                "db",
                reqwest::Client::new(),
                metrics.clone(),
                max_retries,
                Duration::from_secs(5),
                false,
            );
            let (attempt, metadata) = export_attempt(&metrics, 2, payload.len());

            let error = client
                .send(
                    "table",
                    Some("mapping"),
                    HeaderValue::from_static("Bearer test"),
                    payload.clone(),
                    attempt,
                    metadata,
                )
                .await
                .expect_err("retry budget should be exhausted");
            assert!(matches!(&error, Error::ExportFailed { attempts: 1, .. }));
            assert!(
                error
                    .to_string()
                    .starts_with("Export failed after 1 attempts: ")
            );
            assert_eq!(error.is_refusal(), status == 429);
        }

        let snapshots = metrics.borrow_mut().boundary.terminal_snapshots();
        for outcome in [Outcome::Refused, Outcome::Failure] {
            assert_eq!(attempted_metric(&snapshots, outcome, "messages"), 1);
            assert_eq!(attempted_metric(&snapshots, outcome, "items"), 2);
            assert_eq!(
                attempted_metric(&snapshots, outcome, "payload.size"),
                payload.len() as u64
            );
            assert!(attempted_recorded(&snapshots, outcome, "duration"));
        }
    }

    /// Scenario: ADX accepts a request carrying explicit export-attempt metadata.
    /// Guarantees: shared exporter metrics record the message, item, and payload-size values.
    #[tokio::test]
    async fn successful_request_records_attempt_metadata() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        let payload = Bytes::from_static(b"compressed");
        let metrics = test_metrics();
        let client = AzureDataExplorerClient::new(
            &server.uri(),
            "database",
            reqwest::Client::new(),
            metrics.clone(),
            1,
            Duration::from_secs(5),
            false,
        );
        let (attempt, metadata) = export_attempt(&metrics, 3, payload.len());

        _ = client
            .send(
                "table",
                Some("mapping"),
                HeaderValue::from_static("Bearer test"),
                payload.clone(),
                attempt,
                metadata,
            )
            .await
            .expect("request should succeed");

        let snapshots = metrics.borrow_mut().terminal_snapshots();
        assert_eq!(
            attempted_metric(&snapshots, Outcome::Success, "messages"),
            1
        );
        assert_eq!(attempted_metric(&snapshots, Outcome::Success, "items"), 3);
        assert_eq!(
            attempted_metric(&snapshots, Outcome::Success, "payload.size"),
            payload.len() as u64
        );
    }

    /// Scenario: ADX returns a Retry-After value too large for the local clock.
    /// Guarantees: the operation deadline bounds the delay without an instant overflow.
    #[tokio::test]
    async fn retry_after_is_bounded_by_operation_deadline() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(429).insert_header("Retry-After", u64::MAX.to_string()),
            )
            .expect(1)
            .mount(&server)
            .await;
        let timeout = Duration::from_millis(50);
        let metrics = test_metrics();
        let client = AzureDataExplorerClient::new(
            &server.uri(),
            "database",
            reqwest::Client::new(),
            metrics.clone(),
            1,
            timeout,
            false,
        );
        let payload = Bytes::from_static(b"compressed");
        let (attempt, metadata) = export_attempt(&metrics, 1, payload.len());

        let error = client
            .send(
                "table",
                Some("mapping"),
                HeaderValue::from_static("******"),
                payload,
                attempt,
                metadata,
            )
            .await
            .expect_err("the operation deadline should expire before Retry-After");

        assert!(matches!(error, Error::OperationTimeout { .. }));
    }

    /// Scenario: ADX returns a retryable response whose backoff would exceed the configured operation timeout.
    /// Guarantees: the passed timeout bounds the complete operation, including retry backoff.
    #[tokio::test]
    async fn operation_timeout_includes_retry_backoff() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;
        let timeout = Duration::from_millis(500);
        let metrics = test_metrics();
        let client = AzureDataExplorerClient::new(
            &server.uri(),
            "database",
            reqwest::Client::new(),
            metrics.clone(),
            2,
            timeout,
            false,
        );

        let payload = Bytes::from_static(b"compressed");
        let (attempt, metadata) = export_attempt(&metrics, 1, payload.len());
        let error = client
            .send(
                "table",
                Some("mapping"),
                HeaderValue::from_static("Bearer test"),
                payload,
                attempt,
                metadata,
            )
            .await
            .expect_err("retry backoff should exhaust operation deadline");

        assert!(matches!(
            error,
            Error::OperationTimeout {
                timeout: actual_timeout
            } if actual_timeout == timeout
        ));
        let snapshots = metrics.borrow_mut().terminal_snapshots();
        assert_eq!(
            attempted_metric(&snapshots, Outcome::Failure, "messages"),
            1
        );
        assert_eq!(http_response_metric(&snapshots, "http_5xx"), 1);
        assert!(!snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.azure_data_explorer.http"
                && snapshot.measurement_attribute_value("response") == Some("network_error")
        }));
    }

    /// Scenario: an ADX request remains in flight until the operation deadline.
    /// Guarantees: the timed-out physical attempt records one failed attempt and one network-error response.
    #[tokio::test]
    async fn request_deadline_records_http_response_outcome() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204).set_delay(Duration::from_secs(2)))
            .expect(1)
            .mount(&server)
            .await;
        let timeout = Duration::from_millis(50);
        let metrics = test_metrics();
        let client = AzureDataExplorerClient::new(
            &server.uri(),
            "database",
            reqwest::Client::new(),
            metrics.clone(),
            0,
            timeout,
            false,
        );
        let payload = Bytes::from_static(b"compressed");
        let (attempt, metadata) = export_attempt(&metrics, 1, payload.len());

        let error = client
            .send(
                "table",
                Some("mapping"),
                HeaderValue::from_static("******"),
                payload,
                attempt,
                metadata,
            )
            .await
            .expect_err("request should reach the operation deadline");

        assert!(matches!(error, Error::OperationTimeout { .. }));
        let snapshots = metrics.borrow_mut().terminal_snapshots();
        assert_eq!(
            attempted_metric(&snapshots, Outcome::Failure, "messages"),
            1
        );
        assert_eq!(http_response_metric(&snapshots, "network_error"), 1);
    }

    /// Scenario: ADX sends error headers but stalls the response body until the operation deadline.
    /// Guarantees: the physical attempt records only `network_error`, not both that and the HTTP status.
    #[tokio::test]
    async fn response_body_deadline_records_one_http_outcome() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let server = async {
            let (mut stream, _) = listener.accept().await.expect("client connection");
            let mut request = vec![0_u8; 4096];
            let _ = stream.read(&mut request).await.expect("request bytes");
            stream
                .write_all(
                    b"HTTP/1.1 500 Internal Server Error\r\n\
                      Content-Length: 7\r\n\
                      Content-Type: text/plain\r\n\
                      Connection: close\r\n\r\n",
                )
                .await
                .expect("response headers");
            stream.flush().await.expect("flush response headers");
            tokio::time::sleep(Duration::from_millis(100)).await;
        };

        let timeout = Duration::from_millis(20);
        let metrics = test_metrics();
        let client = AzureDataExplorerClient::new(
            &format!("http://{address}"),
            "database",
            reqwest::Client::new(),
            metrics.clone(),
            0,
            timeout,
            false,
        );
        let payload = Bytes::from_static(b"compressed");
        let (attempt, metadata) = export_attempt(&metrics, 1, payload.len());
        let request = client.send(
            "table",
            Some("mapping"),
            HeaderValue::from_static("******"),
            payload,
            attempt,
            metadata,
        );

        let (_, error) = tokio::join!(server, request);
        assert!(matches!(error, Err(Error::OperationTimeout { .. })));

        let snapshots = metrics.borrow_mut().terminal_snapshots();
        assert_eq!(http_response_metric(&snapshots, "network_error"), 1);
        assert!(!snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.azure_data_explorer.http"
                && snapshot.measurement_attribute_value("response") == Some("http_5xx")
        }));
    }

    /// Scenario: ADX keeps returning 429 or 503 to a batch configured with one retry.
    /// Guarantees: exhaustion reports two attempts, retains the last error, and keeps response bodies out of log-safe summaries.
    #[tokio::test]
    async fn retry_exhaustion_reports_attempt_count() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        for status in [429, 503] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status).set_body_string("service response"))
                .expect(2)
                .mount(&server)
                .await;
            let metrics = test_metrics();
            let client = AzureDataExplorerClient::new(
                &server.uri(),
                "db",
                reqwest::Client::new(),
                metrics.clone(),
                1,
                Duration::from_secs(60),
                false,
            );
            let payload = Bytes::from_static(b"payload");
            let (attempt, metadata) = export_attempt(&metrics, 2, payload.len());

            let error = client
                .send(
                    "table",
                    None,
                    HeaderValue::from_static("Bearer test-token"),
                    payload,
                    attempt,
                    metadata,
                )
                .await
                .expect_err("retry budget should be exhausted");

            let Error::ExportFailed {
                attempts,
                ref last_error,
            } = error
            else {
                panic!("expected retry exhaustion, got {error:?}");
            };
            assert_eq!(attempts, 2);
            match last_error.as_ref() {
                Error::RateLimited { body, .. } => {
                    assert_eq!(status, 429);
                    assert_eq!(body, "service response");
                }
                Error::ServerError {
                    status: actual_status,
                    body,
                    ..
                } => {
                    assert_eq!(actual_status.as_u16(), status);
                    assert_eq!(body, "service response");
                }
                other => panic!("unexpected last error: {other:?}"),
            }
            assert_eq!(
                error.to_string(),
                format!("Export failed after 2 attempts: {last_error}")
            );
            assert_eq!(
                error.log_safe_summary(),
                format!(
                    "Export failed after 2 attempts: {}",
                    last_error.log_safe_summary()
                )
            );
            assert!(!error.log_safe_summary().contains("service response"));
            assert_eq!(error.is_refusal(), status == 429);
            assert!(!error.is_permanent_refusal());
        }
    }

    /// Scenario: an ADX batch configured with one retry receives a 500 followed by success.
    /// Guarantees: `max_retries` counts retries after the initial submission, and both attempts are measured.
    #[tokio::test]
    async fn retry_records_each_shared_attempt() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let metrics = test_metrics();
        let payload = Bytes::from_static(b"payload");
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let client = AzureDataExplorerClient::new(
            &server.uri(),
            "db",
            reqwest::Client::new(),
            metrics.clone(),
            1,
            Duration::from_secs(10),
            false,
        );
        let (attempt, metadata) = export_attempt(&metrics, 2, payload.len());

        let _duration = client
            .send(
                "table",
                Some("mapping"),
                HeaderValue::from_static("Bearer test"),
                payload,
                attempt,
                metadata,
            )
            .await
            .expect("retry should succeed");

        let snapshots = metrics.borrow_mut().boundary.terminal_snapshots();
        assert_eq!(
            attempted_metric(&snapshots, Outcome::Failure, "messages"),
            1
        );
        assert_eq!(
            attempted_metric(&snapshots, Outcome::Success, "messages"),
            1
        );
    }
}
