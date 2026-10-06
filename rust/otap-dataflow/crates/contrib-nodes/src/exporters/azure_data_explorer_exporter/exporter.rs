// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Main exporter implementation for Azure Data Explorer (ADX).

use async_trait::async_trait;
use bytes::Bytes;
use flate2::Compression;
use flate2::write::GzEncoder;
use futures::future::LocalBoxFuture;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::ConsumerEffectHandlerExtension;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NackCause, NackMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::metrics::ExporterAttempt;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::OtapPayload;
use otel_arrow_dfe_pdata::PayloadData;
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::views::otap::{OtapLogsView, OtapMetricsView, OtapTracesView};
use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogsData;
use otel_arrow_dfe_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otel_arrow_dfe_pdata::views::otlp::bytes::traces::RawTraceData;
use otel_arrow_dfe_telemetry::common_attributes::Outcome;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::Write;
use std::rc::Rc;
use std::time::Duration;
use tokio::time::Instant;

use super::client::{AzureDataExplorerClient, AzureDataExplorerClientPool, ExportAttemptMetadata};
use super::config::Config;
use super::error::Error;
use super::metrics::{AzureDataExplorerExporterMetricsRc, AzureDataExplorerExporterMetricsTracker};
use super::transformer::Transformer;
use otel_arrow_dfe_otap::bearer_auth::{BearerAuth, BearerAuthEvents};

const COMPRESSION_YIELD_BYTES: usize = 64 * 1024;

/// Which ADX table a batch of rows is destined for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Signal {
    Logs,
    Metrics,
    Traces,
}

impl Signal {
    fn name(self) -> &'static str {
        match self {
            Self::Logs => "logs",
            Self::Metrics => "metrics",
            Self::Traces => "traces",
        }
    }

    fn signal_type(self) -> SignalType {
        match self {
            Self::Logs => SignalType::Logs,
            Self::Metrics => SignalType::Metrics,
            Self::Traces => SignalType::Traces,
        }
    }
}

impl Signal {
    fn table_name(self, config: &Config) -> &str {
        match self {
            Signal::Logs => &config.logs_table_name,
            Signal::Metrics => &config.metrics_table_name,
            Signal::Traces => &config.traces_table_name,
        }
    }

    fn json_mapping(self, config: &Config) -> Option<&str> {
        let mapping = match self {
            Signal::Logs => config.logs_table_json_mapping.as_deref(),
            Signal::Metrics => config.metrics_table_json_mapping.as_deref(),
            Signal::Traces => config.traces_table_json_mapping.as_deref(),
        };
        mapping.filter(|name| !name.is_empty())
    }
}

/// Raises shared bearer-auth warnings under the Azure Data Explorer event namespace.
const AZURE_DATA_EXPLORER_BEARER_AUTH_EVENTS: BearerAuthEvents = BearerAuthEvents {
    invalid_token: |error| {
        otel_warn!("azure_data_explorer_exporter.auth.invalid_bearer_token", error = %error);
    },
    token_stream_closed: || {
        otel_warn!(
            "azure_data_explorer_exporter.auth.token_stream_closed",
            message =
                "bearer token provider closed its stream; no further token refreshes will arrive"
        );
    },
};

/// Completed export result returned by an in-flight future.
struct CompletedExport {
    client: AzureDataExplorerClient,
    signal: Signal,
    table: String,
    json_mapping: Option<String>,
    sample_row: Option<String>,
    failed_payload: Option<String>,
    row_count: u64,
    item_count: u64,
    message_count: u64,
    result: Result<Duration, Error>,
    compressed: Bytes,
    pdata_batch: Vec<OtapPdata>,
    token_generation: u64,
    auth_retry_attempts: u32,
}

/// A coalesced request retained after ADX rejects its bearer token.
struct RetainedExport {
    signal: Signal,
    table: String,
    json_mapping: Option<String>,
    sample_row: Option<String>,
    failed_payload: Option<String>,
    row_count: u64,
    item_count: u64,
    message_count: u64,
    compressed: Bytes,
    pdata_batch: Vec<OtapPdata>,
    auth_retry_attempts: u32,
}

/// Recoverable state for one in-flight export.
struct PendingExport {
    future: LocalBoxFuture<'static, (AzureDataExplorerClient, Result<Duration, Error>)>,
    signal: Signal,
    table: String,
    json_mapping: Option<String>,
    sample_row: Option<String>,
    failed_payload: Option<String>,
    row_count: u64,
    item_count: u64,
    message_count: u64,
    compressed: Bytes,
    pdata_batch: Vec<OtapPdata>,
    token_generation: u64,
    auth_retry_attempts: u32,
}

/// Manages in-flight export futures and their recoverable PData ownership.
struct InFlightExports {
    max: usize,
    pending_messages: usize,
    exports: Vec<PendingExport>,
}

impl InFlightExports {
    fn new(max: usize) -> Self {
        Self {
            max,
            pending_messages: 0,
            exports: Vec::with_capacity(max),
        }
    }

    fn len(&self) -> usize {
        self.exports.len()
    }

    fn pending_messages(&self) -> usize {
        self.pending_messages
    }

    /// Push a new export after the caller has verified capacity.
    fn push(
        &mut self,
        client: AzureDataExplorerClient,
        signal: Signal,
        table: String,
        json_mapping: Option<String>,
        sample_row: Option<String>,
        failed_payload: Option<String>,
        auth_header: http::HeaderValue,
        token_generation: u64,
        compressed: Bytes,
        row_count: u64,
        item_count: u64,
        message_count: u64,
        pdata_batch: Vec<OtapPdata>,
        auth_retry_attempts: u32,
        first_attempt: ExporterAttempt,
        metadata: ExportAttemptMetadata,
    ) {
        debug_assert!(self.exports.len() < self.max);
        self.pending_messages = self.pending_messages.saturating_add(message_count as usize);
        let request_body = compressed.clone();
        let request_table = table.clone();
        let request_mapping = json_mapping.clone();
        let future = Box::pin(async move {
            let result = client
                .send(
                    &request_table,
                    request_mapping.as_deref(),
                    auth_header,
                    request_body,
                    first_attempt,
                    metadata,
                )
                .await;
            (client, result)
        });
        self.exports.push(PendingExport {
            future,
            signal,
            table,
            json_mapping,
            sample_row,
            failed_payload,
            row_count,
            item_count,
            message_count,
            compressed,
            pdata_batch,
            token_generation,
            auth_retry_attempts,
        });
    }

    /// Await the next completed export. Stays pending if nothing is in flight.
    async fn next_completion(&mut self) -> Option<CompletedExport> {
        if self.exports.is_empty() {
            return std::future::pending().await;
        }
        Some(self.next_completion_inner().await)
    }

    async fn next_completion_inner(&mut self) -> CompletedExport {
        let (result, idx, remaining) =
            futures::future::select_all(self.exports.iter_mut().map(|export| &mut export.future))
                .await;
        drop(remaining);
        let pending = self.exports.swap_remove(idx);
        self.pending_messages = self
            .pending_messages
            .saturating_sub(pending.message_count as usize);
        let (client, result) = result;
        CompletedExport {
            client,
            signal: pending.signal,
            table: pending.table,
            json_mapping: pending.json_mapping,
            sample_row: pending.sample_row,
            failed_payload: pending.failed_payload,
            row_count: pending.row_count,
            item_count: pending.item_count,
            message_count: pending.message_count,
            result,
            compressed: pending.compressed,
            pdata_batch: pending.pdata_batch,
            token_generation: pending.token_generation,
            auth_retry_attempts: pending.auth_retry_attempts,
        }
    }

    /// Cancel all requests and recover their unresolved source messages.
    fn cancel_all(&mut self) -> Vec<PendingExport> {
        self.pending_messages = 0;
        std::mem::take(&mut self.exports)
    }
}

/// Accumulates transformed JSON rows across multiple PData messages until a
/// flush threshold (row count, byte size, or time) is reached.
struct BatchAccumulator {
    enabled: bool,
    /// JSON rows waiting to be flushed.
    rows: Vec<Bytes>,
    /// Total uncompressed JSON Lines size, including newline separators.
    bytes: usize,
    /// Source PData messages whose rows are in this batch.
    /// All of these will be acked/nacked together once the HTTP request completes.
    sources: Vec<OtapPdata>,
    /// Source signal items represented by the accumulated messages.
    items: u64,

    max_rows: usize,
    max_bytes: usize,
}

impl BatchAccumulator {
    fn new(enabled: bool, max_rows: usize, max_bytes: usize) -> Self {
        let initial_capacity = if enabled { max_rows.min(1_024) } else { 0 };
        Self {
            enabled,
            rows: Vec::with_capacity(initial_capacity),
            bytes: 0,
            sources: Vec::new(),
            items: 0,
            max_rows,
            max_bytes,
        }
    }

    /// Returns `true` when cross-message batching is enabled.
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn added_bytes(&self, records: &[Bytes]) -> usize {
        let separators = records
            .len()
            .saturating_sub(1)
            .saturating_add(usize::from(!self.rows.is_empty() && !records.is_empty()));
        records
            .iter()
            .map(Bytes::len)
            .sum::<usize>()
            .saturating_add(separators)
    }

    /// Add rows from one PData message.  Returns `true` if a threshold
    /// has been reached and the caller should flush.
    fn push(&mut self, records: Vec<Bytes>, pdata: OtapPdata, item_count: u64) -> bool {
        self.bytes = self.bytes.saturating_add(self.added_bytes(&records));
        self.rows.extend(records);
        self.sources.push(pdata);
        self.items = self.items.saturating_add(item_count);

        self.should_flush()
    }

    fn message_count(&self) -> usize {
        self.sources.len()
    }

    fn would_exceed(&self, records: &[Bytes], max_rows: usize, max_bytes: usize) -> bool {
        (max_rows > 0 && self.rows.len().saturating_add(records.len()) > max_rows)
            || (max_bytes > 0 && self.bytes.saturating_add(self.added_bytes(records)) > max_bytes)
    }

    /// Check if the batch has reached a flush threshold.
    fn should_flush(&self) -> bool {
        if self.rows.is_empty() {
            return false;
        }
        if self.max_rows > 0 && self.rows.len() >= self.max_rows {
            return true;
        }
        if self.max_bytes > 0 && self.bytes >= self.max_bytes {
            return true;
        }
        // Safety valve: prevent unbounded row growth when max_rows is
        // disabled (0).  100 000 rows is well beyond any reasonable
        // single batch; flush to avoid OOM from a burst of tiny records.
        if self.max_rows == 0 && self.rows.len() >= 100_000 {
            return true;
        }
        false
    }

    /// Take everything out of the accumulator for flushing.
    /// Returns `(rows, sources, row_count, item_count, message_count)`.
    fn take(&mut self) -> (Vec<Bytes>, Vec<OtapPdata>, u64, u64, u64) {
        let rows = std::mem::take(&mut self.rows);
        let sources = std::mem::take(&mut self.sources);
        let row_count = rows.len() as u64;
        let item_count = self.items;
        let message_count = sources.len() as u64;
        self.bytes = 0;
        self.items = 0;
        self.rows.reserve(self.max_rows.min(1_024));
        (rows, sources, row_count, item_count, message_count)
    }

    /// Whether there are any pending rows.
    fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Azure Data Explorer (ADX) exporter.
///
/// Sends OpenTelemetry log data to Azure Data Explorer via the Kusto streaming
/// ingestion REST API, using gzip-compressed JSON payloads.
pub struct AzureDataExplorerExporter {
    config: Config,
    transformer: Transformer,
    metrics: AzureDataExplorerExporterMetricsRc,
    client_pool: AzureDataExplorerClientPool,
    in_flight_exports: InFlightExports,
    retained_exports: VecDeque<RetainedExport>,
    token_provider: Option<Box<dyn BearerTokenProvider>>,
}

impl AzureDataExplorerExporter {
    /// Build a new exporter from configuration.
    pub fn new(
        pipeline_ctx: PipelineContext,
        config: Config,
        token_provider: Box<dyn BearerTokenProvider>,
    ) -> Result<Self, Error> {
        config
            .validate()
            .map_err(|e| Error::Config(e.to_string()))?;

        let metrics: AzureDataExplorerExporterMetricsRc = Rc::new(RefCell::new(
            AzureDataExplorerExporterMetricsTracker::register(&pipeline_ctx),
        ));

        let max_in_flight = config.max_in_flight;
        let max_retries = config.max_retries;
        let timeout = config.timeout;
        let log_response_body = config.log_response_body;
        let legacy_logs_body_string = config.legacy_logs_body_string;
        let export_event_name = config.export_event_name;
        let add_event_name_to_log_attributes = config.add_event_name_to_log_attributes;

        Ok(Self {
            config,
            transformer: Transformer::new()
                .with_legacy_logs_body_string(legacy_logs_body_string)
                .with_log_event_name_options(export_event_name, add_event_name_to_log_attributes),
            metrics: metrics.clone(),
            client_pool: AzureDataExplorerClientPool::new(
                max_in_flight + 1,
                metrics,
                max_retries,
                timeout,
                log_response_body,
            ),
            in_flight_exports: InFlightExports::new(max_in_flight),
            retained_exports: VecDeque::new(),
            token_provider: Some(token_provider),
        })
    }

    /// Gzip-compress a batch without monopolizing the node-local runtime.
    async fn compress_batch(records: &[Bytes], level: u32) -> Result<Bytes, Error> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        let mut bytes_since_yield = 0usize;
        for (i, record) in records.iter().enumerate() {
            if i > 0 {
                encoder.write_all(b"\n").map_err(Error::Compression)?;
                bytes_since_yield += 1;
                if bytes_since_yield == COMPRESSION_YIELD_BYTES {
                    tokio::task::yield_now().await;
                    bytes_since_yield = 0;
                }
            }
            let mut remaining = record.as_ref();
            while !remaining.is_empty() {
                let work_remaining = COMPRESSION_YIELD_BYTES - bytes_since_yield;
                let write_len = remaining.len().min(work_remaining);
                encoder
                    .write_all(&remaining[..write_len])
                    .map_err(Error::Compression)?;
                remaining = &remaining[write_len..];
                bytes_since_yield += write_len;
                if bytes_since_yield == COMPRESSION_YIELD_BYTES {
                    tokio::task::yield_now().await;
                    bytes_since_yield = 0;
                }
            }
        }
        let compressed = encoder.finish().map_err(Error::Compression)?;
        Ok(Bytes::from(compressed))
    }

    fn validate_row_sizes(records: &[Bytes], limit: usize) -> Result<(), Error> {
        if let Some(actual) = records
            .iter()
            .map(|record| record.len().saturating_add(1))
            .find(|size| *size > limit)
        {
            return Err(Error::RowTooLarge { actual, limit });
        }
        Ok(())
    }

    fn json_lines_size(records: &[Bytes]) -> usize {
        records
            .iter()
            .map(Bytes::len)
            .sum::<usize>()
            .saturating_add(records.len().saturating_sub(1))
    }

    fn start_attempt(&self, signal: Signal, items: u64) -> ExporterAttempt {
        let mut attempt = self.metrics.borrow().boundary.attempt(signal.signal_type());
        attempt.set_item_count_with(|| items);
        attempt
    }

    fn attempt_metadata(signal: Signal, items: u64, payload_size: usize) -> ExportAttemptMetadata {
        ExportAttemptMetadata {
            signal: signal.signal_type(),
            items,
            payload_size,
        }
    }

    async fn record_started_failure(
        &self,
        mut attempt: ExporterAttempt,
        payload_size: Option<usize>,
    ) {
        if let Some(payload_size) = payload_size {
            attempt.set_payload_size_with(|| payload_size);
        }
        let completed = attempt
            .run(async |attempt| Err::<(), _>(attempt.failed(())))
            .await;
        let result = self.metrics.borrow_mut().boundary.record(completed);
        debug_assert!(result.is_err());
    }

    async fn record_preparation_failure(
        &self,
        signal: SignalType,
        item_count: u64,
        payload_size: Option<usize>,
    ) {
        let mut attempt = self.metrics.borrow().boundary.attempt(signal);
        attempt.set_item_count_with(|| item_count);
        if let Some(payload_size) = payload_size {
            attempt.set_payload_size_with(|| payload_size);
        }
        let completed = attempt
            .run(async |attempt| Err::<(), _>(attempt.failed(())))
            .await;
        let result = self.metrics.borrow_mut().boundary.record(completed);
        debug_assert!(result.is_err());
    }

    async fn record_preparation_success(&self, signal: SignalType, item_count: u64) {
        let mut attempt = self.metrics.borrow().boundary.attempt(signal);
        attempt.set_item_count_with(|| item_count);
        let completed = attempt
            .run(async |_| Ok::<(), otel_arrow_dfe_otap::metrics::ErrorWithOutcome<()>>(()))
            .await;
        let result = self.metrics.borrow_mut().boundary.record(completed);
        debug_assert!(result.is_ok());
    }

    async fn record_preparation_refusal(
        &self,
        signal: SignalType,
        item_count: u64,
        payload_size: Option<usize>,
    ) {
        let mut attempt = self.metrics.borrow().boundary.attempt(signal);
        attempt.set_item_count_with(|| item_count);
        if let Some(payload_size) = payload_size {
            attempt.set_payload_size_with(|| payload_size);
        }
        let completed = attempt
            .run(async |attempt| Err::<(), _>(attempt.refused(())))
            .await;
        let result = self.metrics.borrow_mut().boundary.record(completed);
        debug_assert!(result.is_err());
    }

    fn failed_payload(records: &[Bytes], enabled: bool) -> Option<String> {
        enabled.then(|| {
            records
                .iter()
                .map(|record| String::from_utf8_lossy(record).into_owned())
                .collect::<Vec<_>>()
                .join("\n")
        })
    }

    fn failed_sample_row(records: &[Bytes], enabled: bool) -> Option<String> {
        if !enabled {
            return None;
        }
        records.first().map(|record| {
            let text = String::from_utf8_lossy(record);
            text.chars().take(2048).collect()
        })
    }

    /// Process an incoming message, returning which table its rows belong to.
    fn extract_records(
        &self,
        payload: &OtapPayload,
    ) -> Result<Option<(Signal, Vec<Bytes>)>, Error> {
        let max_rows = self.config.network_requests.max_rows;
        let max_row_bytes = self.config.max_row_bytes;
        let max_bytes = self.config.network_requests.max_bytes;
        match payload.data() {
            PayloadData::OtapArrowRecords(otap_records) => match otap_records {
                OtapArrowRecords::Logs(_) => {
                    let logs_view = OtapLogsView::try_from(otap_records)
                        .map_err(|source| Error::LogsViewCreationFailed { source })?;
                    Ok(Some((
                        Signal::Logs,
                        self.transformer.convert_logs_bounded(
                            &logs_view,
                            max_rows,
                            max_row_bytes,
                            max_bytes,
                        )?,
                    )))
                }
                OtapArrowRecords::Metrics(_) => {
                    let metrics_view = OtapMetricsView::try_from(otap_records)
                        .map_err(|source| Error::MetricsViewCreationFailed { source })?;
                    Ok(Some((
                        Signal::Metrics,
                        self.transformer.convert_metrics_bounded(
                            &metrics_view,
                            max_rows,
                            max_row_bytes,
                            max_bytes,
                        )?,
                    )))
                }
                OtapArrowRecords::Traces(_) => {
                    let traces_view = OtapTracesView::try_from(otap_records)
                        .map_err(|source| Error::TracesViewCreationFailed { source })?;
                    Ok(Some((
                        Signal::Traces,
                        self.transformer.convert_traces_bounded(
                            &traces_view,
                            max_rows,
                            max_row_bytes,
                            max_bytes,
                        )?,
                    )))
                }
            },
            PayloadData::OtlpBytes(otlp_bytes) => match otlp_bytes {
                OtlpProtoBytes::ExportLogsRequest(bytes) => {
                    let logs_view = RawLogsData::try_new(bytes.as_ref())
                        .map_err(|source| Error::LogsViewCreationFailed { source })?;
                    Ok(Some((
                        Signal::Logs,
                        self.transformer.convert_logs_bounded(
                            &logs_view,
                            max_rows,
                            max_row_bytes,
                            max_bytes,
                        )?,
                    )))
                }
                OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                    let metrics_view = RawMetricsData::try_new(bytes.as_ref())
                        .map_err(|source| Error::MetricsViewCreationFailed { source })?;
                    Ok(Some((
                        Signal::Metrics,
                        self.transformer.convert_metrics_bounded(
                            &metrics_view,
                            max_rows,
                            max_row_bytes,
                            max_bytes,
                        )?,
                    )))
                }
                OtlpProtoBytes::ExportTracesRequest(bytes) => {
                    let traces_view = RawTraceData::try_new(bytes.as_ref())
                        .map_err(|source| Error::TracesViewCreationFailed { source })?;
                    Ok(Some((
                        Signal::Traces,
                        self.transformer.convert_traces_bounded(
                            &traces_view,
                            max_rows,
                            max_row_bytes,
                            max_bytes,
                        )?,
                    )))
                }
            },
        }
    }

    async fn finalize_export(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        auth: &mut BearerAuth,
        completed: CompletedExport,
    ) -> Result<(), EngineError> {
        let _ = self
            .finalize_export_until(effect_handler, auth, completed, None)
            .await?;
        Ok(())
    }

    async fn notify_ack_until(
        effect_handler: &EffectHandler<OtapPdata>,
        ack: AckMsg<OtapPdata>,
        deadline: Option<Instant>,
    ) -> Result<bool, EngineError> {
        if let Some(deadline) = deadline {
            return match tokio::time::timeout_at(deadline, effect_handler.notify_ack(ack)).await {
                Ok(result) => {
                    result?;
                    Ok(true)
                }
                Err(_) => Ok(false),
            };
        }
        effect_handler.notify_ack(ack).await?;
        Ok(true)
    }

    async fn notify_nack_until(
        effect_handler: &EffectHandler<OtapPdata>,
        nack: NackMsg<OtapPdata>,
        deadline: Option<Instant>,
    ) -> Result<bool, EngineError> {
        if let Some(deadline) = deadline {
            return match tokio::time::timeout_at(deadline, effect_handler.notify_nack(nack)).await {
                Ok(result) => {
                    result?;
                    Ok(true)
                }
                Err(_) => Ok(false),
            };
        }
        effect_handler.notify_nack(nack).await?;
        Ok(true)
    }

    async fn finalize_export_until(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        auth: &mut BearerAuth,
        completed: CompletedExport,
        deadline: Option<Instant>,
    ) -> Result<bool, EngineError> {
        let CompletedExport {
            client,
            signal,
            table,
            json_mapping,
            sample_row,
            failed_payload,
            row_count,
            item_count,
            message_count,
            result,
            compressed,
            pdata_batch,
            token_generation,
            auth_retry_attempts,
        } = completed;

        self.client_pool.release(client);

        match result {
            Ok(_) => {
                self.metrics
                    .borrow_mut()
                    .record_batch(signal.signal_type(), Outcome::Success);
                for pdata in pdata_batch {
                    if !Self::notify_ack_until(effect_handler, AckMsg::new(pdata), deadline).await?
                    {
                        return Ok(false);
                    }
                }
            }
            Err(e) => {
                if e.is_unauthorized() {
                    auth.invalidate(token_generation);
                    if self.config.network_requests.coalesce
                        && auth_retry_attempts < self.config.max_retries
                    {
                        otel_warn!(
                            "azure_data_explorer_exporter.export.retained_after_unauthorized",
                            signal = signal.name(),
                            row_count = row_count,
                            message_count = message_count
                        );
                        self.retained_exports.push_back(RetainedExport {
                            signal,
                            table,
                            json_mapping,
                            sample_row,
                            failed_payload,
                            row_count,
                            item_count,
                            message_count,
                            compressed,
                            pdata_batch,
                            auth_retry_attempts: auth_retry_attempts.saturating_add(1),
                        });
                        return Ok(true);
                    }
                }
                let outcome = if e.is_refusal() {
                    Outcome::Refused
                } else {
                    Outcome::Failure
                };
                self.metrics
                    .borrow_mut()
                    .record_batch(signal.signal_type(), outcome);
                let permanent_refusal = e.is_permanent_refusal();
                let safe_error = e.log_safe_summary();
                if let Some(sample_row) = sample_row.as_deref() {
                    otel_debug!(
                        "azure_data_explorer_exporter.export.sample_row",
                        message = sample_row,
                        signal = signal.name(),
                        table = table.as_str(),
                        json_mapping = json_mapping.as_deref().unwrap_or("")
                    );
                }
                if let Some(payload) = failed_payload.as_deref() {
                    otel_debug!(
                        "azure_data_explorer_exporter.export.failed_payload",
                        message = payload,
                        signal = signal.name(),
                        table = table.as_str(),
                        json_mapping = json_mapping.as_deref().unwrap_or(""),
                        row_count = row_count
                    );
                }
                otel_warn!(
                    "azure_data_explorer_exporter.export.failed",
                    message = safe_error.as_str(),
                    signal = signal.name(),
                    table = table.as_str(),
                    json_mapping = json_mapping.as_deref().unwrap_or(""),
                    row_count = row_count,
                    message_count = message_count,
                    error = safe_error.as_str()
                );
                for pdata in pdata_batch {
                    let nack =
                        Self::nack_for_export_error(permanent_refusal, safe_error.clone(), pdata);
                    if !Self::notify_nack_until(effect_handler, nack, deadline).await? {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    fn nack_for_export_error(
        permanent_refusal: bool,
        reason: String,
        pdata: OtapPdata,
    ) -> NackMsg<OtapPdata> {
        if permanent_refusal {
            NackMsg::new_permanent_with_cause(reason, pdata, NackCause::Refused)
        } else {
            NackMsg::new(reason, pdata)
        }
    }

    fn pending_messages(
        &self,
        logs_batch: &BatchAccumulator,
        metrics_batch: &BatchAccumulator,
        traces_batch: &BatchAccumulator,
    ) -> usize {
        self.in_flight_exports
            .pending_messages()
            .saturating_add(logs_batch.message_count())
            .saturating_add(metrics_batch.message_count())
            .saturating_add(traces_batch.message_count())
            .saturating_add(
                self.retained_exports
                    .iter()
                    .map(|export| export.message_count as usize)
                    .sum::<usize>(),
            )
    }

    async fn dispatch_retained(&mut self, auth: &mut BearerAuth) -> Result<bool, EngineError> {
        if self.in_flight_exports.len() >= self.config.max_in_flight {
            return Ok(false);
        }
        let Some(retained) = self.retained_exports.pop_front() else {
            return Ok(false);
        };
        let Some((auth_header, token_generation)) = auth.header() else {
            self.retained_exports.push_front(retained);
            return Ok(false);
        };
        let Some(client) = self.client_pool.take() else {
            self.retained_exports.push_front(retained);
            return Err(EngineError::InternalError {
                message: "client pool unexpectedly empty while retrying retained ADX request"
                    .to_string(),
            });
        };
        let RetainedExport {
            signal,
            table,
            json_mapping,
            sample_row,
            failed_payload,
            row_count,
            item_count,
            message_count,
            compressed,
            pdata_batch,
            auth_retry_attempts,
        } = retained;
        let mut first_attempt = self.start_attempt(signal, item_count);
        first_attempt.set_payload_size_with(|| compressed.len());
        let metadata = Self::attempt_metadata(signal, item_count, compressed.len());
        self.in_flight_exports.push(
            client,
            signal,
            table,
            json_mapping,
            sample_row,
            failed_payload,
            auth_header,
            token_generation,
            compressed,
            row_count,
            item_count,
            message_count,
            pdata_batch,
            auth_retry_attempts,
            first_attempt,
            metadata,
        );
        Ok(true)
    }

    async fn nack_retained_exports_until(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        reason: &str,
        deadline: Instant,
    ) -> Result<bool, EngineError> {
        while let Some(retained) = self.retained_exports.pop_front() {
            self.metrics
                .borrow_mut()
                .record_batch(retained.signal.signal_type(), Outcome::Failure);
            for pdata in retained.pdata_batch {
                let nack =
                    NackMsg::new_with_cause(reason.to_owned(), pdata, NackCause::NodeShutdown);
                if !Self::notify_nack_until(effect_handler, nack, Some(deadline)).await? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    async fn settle_one_until(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        auth: &mut BearerAuth,
        deadline: Instant,
    ) -> Result<bool, EngineError> {
        let completed =
            match tokio::time::timeout_at(deadline, self.in_flight_exports.next_completion()).await
            {
                Ok(Some(completed)) => completed,
                Ok(None) | Err(_) => return Ok(false),
            };
        self.finalize_export_until(effect_handler, auth, completed, Some(deadline))
            .await
    }

    /// Compress and dispatch the current batch contents as an in-flight
    /// export request.
    async fn flush_batch(
        &mut self,
        signal: Signal,
        batch: &mut BatchAccumulator,
        gzip_level: u32,
        effect_handler: &EffectHandler<OtapPdata>,
        auth: &mut BearerAuth,
    ) -> Result<bool, EngineError> {
        if batch.is_empty() {
            return Ok(false);
        }
        // Defer the flush instead of awaiting capacity here so the main
        // select loop can continue receiving shutdown controls.
        if self.in_flight_exports.len() >= self.config.max_in_flight {
            return Ok(false);
        }

        let (rows, sources, row_count, item_count, message_count) = batch.take();
        let mut first_attempt = self.start_attempt(signal, item_count);

        match Self::compress_batch(&rows, gzip_level).await {
            Ok(compressed) => {
                first_attempt.set_payload_size_with(|| compressed.len());
                let table = signal.table_name(&self.config).to_owned();
                let json_mapping = signal.json_mapping(&self.config).map(str::to_owned);
                let sample_row = Self::failed_sample_row(&rows, self.config.log_failed_payload);
                let failed_payload = Self::failed_payload(&rows, self.config.log_failed_payload);
                let Some((auth_header, token_generation)) = auth.header() else {
                    if self.config.network_requests.coalesce {
                        self.retained_exports.push_back(RetainedExport {
                            signal,
                            table,
                            json_mapping,
                            sample_row,
                            failed_payload,
                            row_count,
                            item_count,
                            message_count,
                            compressed,
                            pdata_batch: sources,
                            auth_retry_attempts: 0,
                        });
                        return Ok(true);
                    }
                    let reason = auth.not_ready_reason().to_owned();
                    self.record_preparation_refusal(
                        signal.signal_type(),
                        item_count,
                        Some(compressed.len()),
                    )
                    .await;
                    self.metrics
                        .borrow_mut()
                        .record_batch(signal.signal_type(), Outcome::Refused);
                    for pdata in sources {
                        effect_handler
                            .notify_nack(NackMsg::new(reason.clone(), pdata))
                            .await?;
                    }
                    return Ok(true);
                };

                let client = match self.client_pool.take() {
                    Some(c) => c,
                    None => {
                        otel_error!("azure_data_explorer_exporter.client_pool_exhausted");
                        self.record_preparation_failure(
                            signal.signal_type(),
                            item_count,
                            Some(compressed.len()),
                        )
                        .await;
                        self.metrics
                            .borrow_mut()
                            .record_batch(signal.signal_type(), Outcome::Failure);
                        let error_str = "client pool unexpectedly empty".to_string();
                        for pdata in sources {
                            effect_handler
                                .notify_nack(NackMsg::new(error_str.clone(), pdata))
                                .await?;
                        }
                        return Ok(true);
                    }
                };
                let metadata = Self::attempt_metadata(signal, item_count, compressed.len());
                self.in_flight_exports.push(
                    client,
                    signal,
                    table,
                    json_mapping,
                    sample_row,
                    failed_payload,
                    auth_header,
                    token_generation,
                    compressed,
                    row_count,
                    item_count,
                    message_count,
                    sources,
                    0,
                    first_attempt,
                    metadata,
                );
            }
            Err(e) => {
                otel_error!("azure_data_explorer_exporter.compression_failed", error = %e);
                self.record_started_failure(first_attempt, None).await;
                self.metrics
                    .borrow_mut()
                    .record_batch(signal.signal_type(), Outcome::Failure);
                let error_str = e.to_string();
                for pdata in sources {
                    effect_handler
                        .notify_nack(NackMsg::new(error_str.clone(), pdata))
                        .await?;
                }
            }
        }
        Ok(true)
    }

    async fn nack_batch_for_shutdown(
        &mut self,
        signal: Signal,
        batch: &mut BatchAccumulator,
        effect_handler: &EffectHandler<OtapPdata>,
        reason: &str,
        deadline: Instant,
    ) -> Result<bool, EngineError> {
        if batch.is_empty() {
            return Ok(true);
        }
        let (rows, sources, _, item_count, _) = batch.take();
        self.record_preparation_failure(
            signal.signal_type(),
            item_count,
            Some(Self::json_lines_size(&rows)),
        )
        .await;
        self.metrics
            .borrow_mut()
            .record_batch(signal.signal_type(), Outcome::Failure);
        for pdata in sources {
            let nack = NackMsg::new_with_cause(reason.to_owned(), pdata, NackCause::NodeShutdown);
            if !Self::notify_nack_until(effect_handler, nack, Some(deadline)).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn record_abandoned_exports(&mut self, exports: Vec<PendingExport>) -> usize {
        let mut message_count = 0usize;
        for export in exports {
            message_count = message_count.saturating_add(export.pdata_batch.len());
            self.record_preparation_failure(
                export.signal.signal_type(),
                export.item_count,
                Some(export.compressed.len()),
            )
            .await;
            self.metrics
                .borrow_mut()
                .record_batch(export.signal.signal_type(), Outcome::Failure);
        }
        message_count
    }

    async fn nack_messages_until(
        &mut self,
        messages: &mut VecDeque<OtapPdata>,
        effect_handler: &EffectHandler<OtapPdata>,
        reason: &str,
        deadline: Instant,
    ) -> Result<bool, EngineError> {
        while let Some(mut pdata) = messages.pop_front() {
            let signal = pdata.signal_type();
            let item_count = pdata.num_items() as u64;
            self.record_preparation_failure(signal, item_count, None)
                .await;
            let nack = NackMsg::new_with_cause(reason.to_owned(), pdata, NackCause::NodeShutdown);
            if !Self::notify_nack_until(effect_handler, nack, Some(deadline)).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn record_abandoned_messages(&self, messages: &mut VecDeque<OtapPdata>) -> usize {
        let mut message_count = 0usize;
        while let Some(mut pdata) = messages.pop_front() {
            message_count = message_count.saturating_add(1);
            self.record_preparation_failure(pdata.signal_type(), pdata.num_items() as u64, None)
                .await;
        }
        message_count
    }

    fn record_abandoned_retained(&mut self) -> usize {
        let mut message_count = 0usize;
        while let Some(export) = self.retained_exports.pop_front() {
            message_count = message_count.saturating_add(export.pdata_batch.len());
            self.metrics
                .borrow_mut()
                .record_batch(export.signal.signal_type(), Outcome::Failure);
        }
        message_count
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for AzureDataExplorerExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        otel_info!(
            "azure_data_explorer_exporter.start",
            cluster_uri = self.config.cluster_uri.as_str(),
            db_name = self.config.db_name.as_str(),
            logs_table = self.config.logs_table_name.as_str(),
            logs_mapping = self.config.logs_table_json_mapping.as_deref().unwrap_or(""),
            metrics_table = self.config.metrics_table_name.as_str(),
            metrics_mapping = self
                .config
                .metrics_table_json_mapping
                .as_deref()
                .unwrap_or(""),
            traces_table = self.config.traces_table_name.as_str(),
            traces_mapping = self
                .config
                .traces_table_json_mapping
                .as_deref()
                .unwrap_or(""),
            legacy_logs_body_string = self.config.legacy_logs_body_string,
            export_event_name = self.config.export_event_name,
            add_event_name_to_log_attributes = self.config.add_event_name_to_log_attributes,
            gzip_level = self.config.gzip_compression_level,
            coalesce_network_requests = self.config.network_requests.coalesce,
            max_pending_messages = self.config.network_requests.max_pending_messages as u64,
            request_max_rows = self.config.network_requests.max_rows as u64,
            request_max_bytes = self.config.network_requests.max_bytes as u64,
            request_flush_interval_ms =
                self.config.network_requests.flush_interval.as_millis() as u64
        );

        let mut auth = BearerAuth::new(
            self.token_provider
                .take()
                .expect("bearer token provider is present before startup"),
            AZURE_DATA_EXPLORER_BEARER_AUTH_EVENTS,
        );

        self.client_pool
            .initialize(&self.config.cluster_uri, &self.config.db_name)
            .await
            .map_err(|e| {
                let error = Error::ClientInit(Box::new(e));
                EngineError::InternalError {
                    message: error.to_string(),
                }
            })?;

        let telemetry_timer_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await
            .map_err(|e| EngineError::InternalError {
                message: format!("Failed to start telemetry timer: {e}"),
            })?;

        let gzip_level = self.config.gzip_compression_level;
        let max_in_flight = self.config.max_in_flight;
        let network_requests = self.config.network_requests.clone();

        let mut logs_batch = BatchAccumulator::new(
            network_requests.coalesce,
            network_requests.max_rows,
            network_requests.max_bytes,
        );
        let mut metrics_batch = BatchAccumulator::new(
            network_requests.coalesce,
            network_requests.max_rows,
            network_requests.max_bytes,
        );
        let mut traces_batch = BatchAccumulator::new(
            network_requests.coalesce,
            network_requests.max_rows,
            network_requests.max_bytes,
        );

        // A far-future instant used as a "disabled" deadline for select!
        // arms that should not fire until explicitly armed.
        let far_future = Instant::now() + Duration::from_secs(86400 * 365);

        let mut logs_deadline = far_future;
        let mut metrics_deadline = far_future;
        let mut traces_deadline = far_future;
        let mut deferred_pdata = VecDeque::new();
        let margin_sleep = tokio::time::sleep_until(Instant::now());
        tokio::pin!(margin_sleep);
        let mut armed_margin_deadline: Option<std::time::Instant> = None;

        loop {
            let has_token = auth.is_ready();
            if has_token
                && !self.retained_exports.is_empty()
                && self.in_flight_exports.len() < max_in_flight
                && self.dispatch_retained(&mut auth).await?
            {
                continue;
            }
            let at_capacity = self.in_flight_exports.len() >= max_in_flight;
            let pending_messages =
                self.pending_messages(&logs_batch, &metrics_batch, &traces_batch);
            if has_token
                && !at_capacity
                && network_requests.coalesce
                && pending_messages >= network_requests.max_pending_messages
            {
                let due_batch = if !logs_batch.is_empty() {
                    Some((Signal::Logs, &mut logs_batch))
                } else if !metrics_batch.is_empty() {
                    Some((Signal::Metrics, &mut metrics_batch))
                } else if !traces_batch.is_empty() {
                    Some((Signal::Traces, &mut traces_batch))
                } else {
                    None
                };
                if let Some((signal, batch)) = due_batch
                    && self
                        .flush_batch(signal, batch, gzip_level, &effect_handler, &mut auth)
                        .await?
                {
                    match signal {
                        Signal::Logs => logs_deadline = far_future,
                        Signal::Metrics => metrics_deadline = far_future,
                        Signal::Traces => traces_deadline = far_future,
                    }
                    continue;
                }
            }
            let below_pending_limit = !network_requests.coalesce
                || pending_messages < network_requests.max_pending_messages;
            let accepting_pdata = has_token && !at_capacity && below_pending_limit;

            let token_margin_deadline = auth.refresh_deadline();
            if token_margin_deadline != armed_margin_deadline {
                if let Some(deadline) = token_margin_deadline {
                    margin_sleep.as_mut().reset(Instant::from_std(deadline));
                }
                armed_margin_deadline = token_margin_deadline;
            }

            tokio::select! {
                biased;

                () = &mut margin_sleep, if token_margin_deadline.is_some() => {
                    continue;
                }

                () = auth.poll_refresh(), if auth.is_active() => {
                    continue;
                }

                _ = tokio::time::sleep_until(logs_deadline.min(metrics_deadline).min(traces_deadline)),
                    if has_token && !at_capacity && ((logs_batch.is_enabled() && !logs_batch.is_empty())
                        || (metrics_batch.is_enabled() && !metrics_batch.is_empty())
                        || (traces_batch.is_enabled() && !traces_batch.is_empty())) => {
                    let now = Instant::now();
                    if logs_deadline <= now && !logs_batch.is_empty() {
                        otel_debug!("azure_data_explorer_exporter.batch.timer_flush", table = "logs");
                        if self.flush_batch(Signal::Logs, &mut logs_batch, gzip_level, &effect_handler, &mut auth).await? {
                            logs_deadline = far_future;
                        }
                    }
                    if metrics_deadline <= now && !metrics_batch.is_empty() {
                        otel_debug!("azure_data_explorer_exporter.batch.timer_flush", table = "metrics");
                        if self.flush_batch(Signal::Metrics, &mut metrics_batch, gzip_level, &effect_handler, &mut auth).await? {
                            metrics_deadline = far_future;
                        }
                    }
                    if traces_deadline <= now && !traces_batch.is_empty() {
                        otel_debug!("azure_data_explorer_exporter.batch.timer_flush", table = "traces");
                        if self.flush_batch(Signal::Traces, &mut traces_batch, gzip_level, &effect_handler, &mut auth).await? {
                            traces_deadline = far_future;
                        }
                    }
                }

                completed = self.in_flight_exports.next_completion() => {
                    if let Some(completed_export) = completed {
                        self.finalize_export(&effect_handler, &mut auth, completed_export).await?;
                    }
                }

                msg = async {
                    if accepting_pdata
                        && let Some(pdata) = deferred_pdata.pop_front()
                    {
                        return Ok(Message::PData(pdata));
                    }
                    msg_chan.recv_when(accepting_pdata).await
                } => {
                    match msg {
                        Ok(Message::Control(NodeControlMsg::CollectTelemetry { mut metrics_reporter })) => {
                            let _ = self.metrics.borrow_mut().report(&mut metrics_reporter);
                        }
                        Ok(Message::Control(NodeControlMsg::Shutdown { deadline, .. })) => {
                            let shutdown_deadline = Instant::from_std(deadline);
                            let _ = tokio::time::timeout_at(
                                shutdown_deadline,
                                telemetry_timer_cancel_handle.cancel(),
                            )
                            .await;
                            let mut deadline_elapsed = Instant::now() >= shutdown_deadline;
                            if !deadline_elapsed && !deferred_pdata.is_empty() {
                                deadline_elapsed = !self
                                    .nack_messages_until(
                                        &mut deferred_pdata,
                                        &effect_handler,
                                        "shutdown before buffered ADX input could be exported",
                                        shutdown_deadline,
                                    )
                                    .await?;
                            }
                            if !deadline_elapsed && auth.is_ready() {
                                for signal in [Signal::Logs, Signal::Metrics, Signal::Traces] {
                                    let batch = match signal {
                                        Signal::Logs => &mut logs_batch,
                                        Signal::Metrics => &mut metrics_batch,
                                        Signal::Traces => &mut traces_batch,
                                    };
                                    while self.in_flight_exports.len() >= max_in_flight {
                                        if !self
                                            .settle_one_until(
                                                &effect_handler,
                                                &mut auth,
                                                shutdown_deadline,
                                            )
                                            .await?
                                        {
                                            deadline_elapsed = true;
                                            break;
                                        }
                                    }
                                    if deadline_elapsed {
                                        break;
                                    }
                                    let flushed = self
                                        .flush_batch(
                                            signal,
                                            batch,
                                            gzip_level,
                                            &effect_handler,
                                            &mut auth,
                                        )
                                        .await?;
                                    debug_assert!(flushed || batch.is_empty());
                                }
                            } else if !deadline_elapsed {
                                let reason = auth.not_ready_reason().to_owned();
                                deadline_elapsed = !self
                                    .nack_batch_for_shutdown(
                                        Signal::Logs,
                                        &mut logs_batch,
                                        &effect_handler,
                                        &reason,
                                        shutdown_deadline,
                                    )
                                    .await?;
                                if !deadline_elapsed {
                                    deadline_elapsed = !self
                                        .nack_batch_for_shutdown(
                                            Signal::Metrics,
                                            &mut metrics_batch,
                                            &effect_handler,
                                            &reason,
                                            shutdown_deadline,
                                        )
                                        .await?;
                                }
                                if !deadline_elapsed {
                                    deadline_elapsed = !self
                                        .nack_batch_for_shutdown(
                                            Signal::Traces,
                                            &mut traces_batch,
                                            &effect_handler,
                                            &reason,
                                            shutdown_deadline,
                                        )
                                        .await?;
                                }
                            }
                            while !deadline_elapsed && self.in_flight_exports.len() > 0 {
                                deadline_elapsed = !self
                                    .settle_one_until(
                                        &effect_handler,
                                        &mut auth,
                                        shutdown_deadline,
                                    )
                                    .await?;
                            }
                            if deadline_elapsed {
                                let reason = "ADX exporter shutdown deadline elapsed";
                                let pending_messages = self
                                    .pending_messages(
                                        &logs_batch,
                                        &metrics_batch,
                                        &traces_batch,
                                    )
                                    .saturating_add(deferred_pdata.len());
                                otel_warn!(
                                    "azure_data_explorer_exporter.shutdown.deadline_elapsed",
                                    pending_messages = pending_messages as u64
                                );
                                let _ = self
                                    .nack_batch_for_shutdown(
                                        Signal::Logs,
                                        &mut logs_batch,
                                        &effect_handler,
                                        reason,
                                        shutdown_deadline,
                                    )
                                    .await?;
                                let _ = self
                                    .nack_batch_for_shutdown(
                                        Signal::Metrics,
                                        &mut metrics_batch,
                                        &effect_handler,
                                        reason,
                                        shutdown_deadline,
                                    )
                                    .await?;
                                let _ = self
                                    .nack_batch_for_shutdown(
                                        Signal::Traces,
                                        &mut traces_batch,
                                        &effect_handler,
                                        reason,
                                        shutdown_deadline,
                                    )
                                    .await?;
                                let canceled = self.in_flight_exports.cancel_all();
                                let _ = self.record_abandoned_exports(canceled).await;
                                let _ = self.record_abandoned_retained();
                                let _ = self
                                    .record_abandoned_messages(&mut deferred_pdata)
                                    .await;
                            } else {
                                let retained_completed = self
                                    .nack_retained_exports_until(
                                    &effect_handler,
                                    "shutdown before retained ADX request could be re-authenticated",
                                    shutdown_deadline,
                                )
                                .await?;
                                if !retained_completed {
                                    let abandoned = self.record_abandoned_retained();
                                    otel_warn!(
                                        "azure_data_explorer_exporter.shutdown.deadline_elapsed",
                                        pending_messages = abandoned as u64
                                    );
                                }
                            }
                            otel_info!("azure_data_explorer_exporter.shutdown");
                            let snapshots = self.metrics.borrow_mut().terminal_snapshots();
                            return Ok(TerminalState::new(deadline, snapshots));
                        }
                        Ok(Message::PData(mut pdata)) => {
                            // Shutdown force-drains already buffered PData even
                            // when normal admission is closed. Park it so the
                            // next control poll can deliver Shutdown.
                            if !accepting_pdata {
                                deferred_pdata.push_back(pdata);
                                continue;
                            }
                            let signal = pdata.signal_type();
                            let input_item_count = pdata.num_items() as u64;
                            let (context, payload) = pdata.into_parts();

                            let extracted = match self.extract_records(&payload) {
                                Ok(extracted) => extracted,
                                Err(error) => {
                                    let refused = error.is_permanent_refusal();
                                    if refused {
                                        self.record_preparation_refusal(
                                            signal,
                                            input_item_count,
                                            None,
                                        )
                                        .await;
                                        self.metrics
                                            .borrow_mut()
                                            .record_batch(signal, Outcome::Refused);
                                    } else {
                                        self.record_preparation_failure(
                                            signal,
                                            input_item_count,
                                            None,
                                        )
                                        .await;
                                    }
                                    let size_refusal = matches!(
                                        &error,
                                        Error::RowTooLarge { .. }
                                            | Error::RequestTooLarge { .. }
                                            | Error::TooManyRows { .. }
                                    );
                                    let metric_refusal =
                                        matches!(&error, Error::InvalidMetricData { .. });
                                    let reason = error.to_string();
                                    if size_refusal {
                                        otel_warn!(
                                            "azure_data_explorer_exporter.request_too_large",
                                            signal = ?signal,
                                            error = reason.as_str()
                                        );
                                    } else if metric_refusal {
                                        otel_warn!(
                                            "azure_data_explorer_exporter.invalid_metric_data",
                                            signal = ?signal,
                                            error = reason.as_str()
                                        );
                                    } else if refused {
                                        otel_warn!(
                                            "azure_data_explorer_exporter.payload_refused",
                                            signal = ?signal,
                                            error = reason.as_str()
                                        );
                                    } else {
                                        otel_warn!(
                                            "azure_data_explorer_exporter.extraction_failed",
                                            signal = ?signal,
                                            error = reason.as_str()
                                        );
                                    }
                                    let pdata = OtapPdata::new(context, payload);
                                    let nack = if refused {
                                        NackMsg::new_permanent_with_cause(
                                            reason,
                                            pdata,
                                            NackCause::Refused,
                                        )
                                    } else {
                                        NackMsg::new_permanent(reason, pdata)
                                    };
                                    effect_handler.notify_nack(nack).await?;
                                    continue;
                                }
                            };

                            if let Some((signal, records)) = extracted {
                                if let Err(error) =
                                    Self::validate_row_sizes(&records, self.config.max_row_bytes)
                                {
                                    let reason = error.to_string();
                                    self.metrics
                                        .borrow_mut()
                                        .record_batch(signal.signal_type(), Outcome::Refused);
                                    let encoded_size = records
                                        .iter()
                                        .map(Bytes::len)
                                        .sum::<usize>()
                                        .saturating_add(records.len().saturating_sub(1));
                                    self.record_preparation_refusal(
                                        signal.signal_type(),
                                        input_item_count,
                                        Some(encoded_size),
                                    )
                                    .await;
                                    otel_warn!(
                                        "azure_data_explorer_exporter.row_too_large",
                                        signal = signal.name(),
                                        table = signal.table_name(&self.config),
                                        row_count = records.len() as u64,
                                        max_row_bytes = self.config.max_row_bytes as u64,
                                        error = reason.as_str()
                                    );
                                    effect_handler
                                        .notify_nack(NackMsg::new_permanent_with_cause(
                                            reason,
                                            OtapPdata::new(context, payload),
                                            NackCause::Refused,
                                        ))
                                        .await?;
                                    continue;
                                }
                                let batch = match signal {
                                    Signal::Logs => &mut logs_batch,
                                    Signal::Metrics => &mut metrics_batch,
                                    Signal::Traces => &mut traces_batch,
                                };
                                if records.is_empty() {
                                    otel_debug!("azure_data_explorer_exporter.message.no_records");
                                    self.record_preparation_success(
                                        signal.signal_type(),
                                        input_item_count,
                                    )
                                    .await;
                                    effect_handler
                                        .notify_ack(AckMsg::new(OtapPdata::new(context, payload)))
                                        .await?;
                                } else if batch.is_enabled() {
                                    // - Batching path -
                                    if !batch.is_empty()
                                        && batch.would_exceed(
                                            &records,
                                            network_requests.max_rows,
                                            network_requests.max_bytes,
                                        )
                                    {
                                        let flushed = self
                                            .flush_batch(
                                                signal,
                                                batch,
                                                gzip_level,
                                                &effect_handler,
                                                &mut auth,
                                            )
                                            .await?;
                                        debug_assert!(flushed);
                                    }
                                    let was_empty = batch.is_empty();
                                    let pdata = OtapPdata::new(context, payload);
                                    let mut should_flush =
                                        batch.push(records, pdata, input_item_count);
                                    should_flush |= batch.message_count()
                                        >= network_requests.max_pending_messages;

                                    // Start the batch timer when transitioning
                                    // from empty to non-empty.
                                    if was_empty {
                                        let new_deadline =
                                            Instant::now() + network_requests.flush_interval;
                                        match signal {
                                            Signal::Logs => logs_deadline = new_deadline,
                                            Signal::Metrics => metrics_deadline = new_deadline,
                                            Signal::Traces => traces_deadline = new_deadline,
                                        }
                                    }

                                    if should_flush
                                        && self.flush_batch(signal, batch, gzip_level, &effect_handler, &mut auth).await?
                                    {
                                        match signal {
                                            Signal::Logs => logs_deadline = far_future,
                                            Signal::Metrics => metrics_deadline = far_future,
                                            Signal::Traces => traces_deadline = far_future,
                                        }
                                    }
                                } else {
                                    let row_count = records.len() as u64;
                                    let mut first_attempt =
                                        self.start_attempt(signal, input_item_count);
                                    match Self::compress_batch(&records, gzip_level).await {
                                        Ok(compressed) => {
                                            first_attempt
                                                .set_payload_size_with(|| compressed.len());
                                            let client = match self.client_pool.take() {
                                                Some(c) => c,
                                                None => {
                                                    otel_error!("azure_data_explorer_exporter.client_pool_exhausted");
                                                    self.record_preparation_failure(
                                                        signal.signal_type(),
                                                        input_item_count,
                                                        Some(compressed.len()),
                                                    )
                                                    .await;
                                                    self.metrics
                                                        .borrow_mut()
                                                        .record_batch(
                                                            signal.signal_type(),
                                                            Outcome::Failure,
                                                        );
                                                    effect_handler
                                                        .notify_nack(NackMsg::new(
                                                            "client pool unexpectedly empty".to_string(),
                                                            OtapPdata::new(context, payload),
                                                        ))
                                                        .await?;
                                                    continue;
                                                }
                                            };
                                            let pdata = OtapPdata::new(context, payload);
                                            let table = signal.table_name(&self.config).to_owned();
                                            let json_mapping = signal
                                                .json_mapping(&self.config)
                                                .map(str::to_owned);
                                            let sample_row = Self::failed_sample_row(
                                                &records,
                                                self.config.log_failed_payload,
                                            );
                                            let failed_payload =
                                                Self::failed_payload(&records, self.config.log_failed_payload);
                                            let Some((auth_header, token_generation)) =
                                                auth.header()
                                            else {
                                                self.record_preparation_refusal(
                                                    signal.signal_type(),
                                                    input_item_count,
                                                    Some(compressed.len()),
                                                )
                                                .await;
                                                self.metrics
                                                    .borrow_mut()
                                                    .record_batch(
                                                        signal.signal_type(),
                                                        Outcome::Refused,
                                                    );
                                                effect_handler
                                                    .notify_nack(NackMsg::new(
                                                        auth.not_ready_reason(),
                                                        pdata,
                                                    ))
                                                    .await?;
                                                self.client_pool.release(client);
                                                continue;
                                            };
                                            let metadata = Self::attempt_metadata(
                                                signal,
                                                input_item_count,
                                                compressed.len(),
                                            );
                                            self.in_flight_exports.push(
                                                client,
                                                signal,
                                                table,
                                                json_mapping,
                                                sample_row,
                                                failed_payload,
                                                auth_header,
                                                token_generation,
                                                compressed,
                                                row_count,
                                                input_item_count,
                                                1,
                                                vec![pdata],
                                                0,
                                                first_attempt,
                                                metadata,
                                            );
                                        }
                                        Err(e) => {
                                            otel_error!("azure_data_explorer_exporter.compression_failed", error = %e);
                                            self.record_started_failure(first_attempt, None)
                                                .await;
                                            self.metrics
                                                .borrow_mut()
                                                .record_batch(
                                                    signal.signal_type(),
                                                    Outcome::Failure,
                                                );
                                            effect_handler
                                                .notify_nack(NackMsg::new(
                                                    e.to_string(),
                                                    OtapPdata::new(context, payload),
                                                ))
                                                .await?;
                                        }
                                    }
                                }
                            } else {
                                // Unsupported signal type - ack and drop
                                effect_handler
                                    .notify_ack(AckMsg::new(OtapPdata::new(context, payload)))
                                    .await?;
                            }
                        }
                        Ok(_) => {} // Ignore other message types
                        Err(e) => {
                            let error = Error::ChannelRecv(e);
                            return Err(EngineError::InternalError {
                                message: error.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AzureDataExplorerExporter, BatchAccumulator, COMPRESSION_YIELD_BYTES, Config, Error,
        PendingExport, Signal,
    };
    use async_trait::async_trait;
    use bytes::Bytes;
    use flate2::read::GzDecoder;
    use futures::StreamExt;
    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::capability::CapabilityError;
    use otel_arrow_dfe_engine::capability::auth::BearerToken;
    use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TokenStream;
    use otel_arrow_dfe_engine::control::NackCause;
    use otel_arrow_dfe_engine::control::{
        AckMsg, NackMsg, PipelineCompletionMsg, pipeline_completion_msg_channel,
    };
    use otel_arrow_dfe_engine::exporter::ExporterWrapper;
    use otel_arrow_dfe_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;
    use otel_arrow_dfe_engine::local::exporter::EffectHandler;
    use otel_arrow_dfe_engine::testing::exporter::TestRuntime;
    use otel_arrow_dfe_engine::testing::{
        test_node, test_pipeline_ctx_with_interests, test_pipeline_runtime_services,
    };
    use otel_arrow_dfe_otap::pdata::Context;
    use otel_arrow_dfe_otap::pdata::OtapPdata;
    use otel_arrow_dfe_otap::testing::TestCallData;
    use otel_arrow_dfe_pdata::OtapPayload;
    use otel_arrow_dfe_pdata::encode::{
        encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch,
    };
    use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, any_value};
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, ResourceLogs, ScopeLogs,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, span,
    };
    use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogsData;
    use otel_arrow_dfe_pdata::views::otlp::bytes::metrics::RawMetricsData;
    use otel_arrow_dfe_pdata::views::otlp::bytes::traces::RawTraceData;
    use otel_arrow_dfe_telemetry::common_attributes::Outcome;
    use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use prost::Message as _;
    use std::io::Read;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct TestTokenProvider {
        replacement_delay: Option<Duration>,
    }

    #[async_trait(?Send)]
    impl BearerTokenProvider for TestTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            Ok(BearerToken::without_expiry("token-1".to_owned()))
        }

        fn token_stream(&self) -> TokenStream {
            let first =
                futures::stream::once(async { BearerToken::without_expiry("token-1".to_owned()) });
            match self.replacement_delay {
                Some(delay) => Box::pin(first.chain(futures::stream::once(async move {
                    tokio::time::sleep(delay).await;
                    BearerToken::without_expiry("token-2".to_owned())
                }))),
                None => Box::pin(first.chain(futures::stream::pending())),
            }
        }
    }

    fn lifecycle_config(endpoint: &str, coalesce: bool, max_in_flight: usize) -> Config {
        serde_json::from_value(serde_json::json!({
            "cluster_uri": endpoint,
            "max_in_flight": max_in_flight,
            "max_retries": 1,
            "timeout": "10s",
            "network_requests": {
                "coalesce": coalesce,
                "max_pending_messages": 8,
                "max_rows": 1000,
                "max_bytes": 4194304,
                "flush_interval": "1ms"
            }
        }))
        .expect("deserialize lifecycle config")
    }

    fn backpressured_effect_handler() -> (
        EffectHandler<OtapPdata>,
        otel_arrow_dfe_engine::control::PipelineCompletionMsgSender<OtapPdata>,
        otel_arrow_dfe_engine::control::PipelineCompletionMsgReceiver<OtapPdata>,
    ) {
        let (_, reporter) = MetricsReporter::create_new_and_receiver(8);
        let mut effect_handler = EffectHandler::new(
            test_node("azure-data-explorer-backpressure-test"),
            reporter,
            test_pipeline_runtime_services(),
        );
        let (completion_tx, completion_rx) = pipeline_completion_msg_channel(1);
        effect_handler.set_pipeline_completion_msg_sender(completion_tx.clone());
        (effect_handler, completion_tx, completion_rx)
    }

    fn lifecycle_exporter(
        config: Config,
        replacement_delay: Option<Duration>,
    ) -> AzureDataExplorerExporter {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let interests = Interests::NODE_INPUT_METRICS
            | Interests::NODE_LOCAL_DURATION
            | Interests::NODE_ITEM_COUNTS
            | Interests::NODE_SIZE;
        let (pipeline_ctx, _registry) = test_pipeline_ctx_with_interests(interests);
        AzureDataExplorerExporter::new(
            pipeline_ctx,
            config,
            Box::new(TestTokenProvider { replacement_delay }),
        )
        .expect("create lifecycle exporter")
    }

    fn lifecycle_runtime(
        config: Config,
        replacement_delay: Option<Duration>,
    ) -> (TestRuntime<OtapPdata>, ExporterWrapper<OtapPdata>) {
        let runtime = TestRuntime::new();
        let exporter = lifecycle_exporter(config, replacement_delay);
        let wrapper = ExporterWrapper::local(
            exporter,
            test_node("azure-data-explorer-lifecycle-test"),
            Arc::new(NodeUserConfig::new_exporter_config(
                super::super::AZURE_DATA_EXPLORER_EXPORTER_URN,
            )),
            runtime.config(),
        );
        (runtime, wrapper)
    }

    fn logs_pdata(body: Option<&str>, interests: Interests, id: usize) -> OtapPdata {
        let request = match body {
            Some(body) => ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord {
                            body: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(body.to_owned())),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            },
            None => ExportLogsServiceRequest::default(),
        };
        OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(Bytes::from(request.encode_to_vec())).into(),
        )
        .test_subscribe_to(interests, TestCallData::default().into(), id)
    }

    fn metrics_request() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: "test.gauge".to_owned(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                time_unix_nano: 1,
                                value: Some(number_data_point::Value::AsDouble(42.0)),
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn traces_request() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                scope_spans: vec![ScopeSpans {
                    spans: vec![Span {
                        trace_id: vec![1; 16],
                        span_id: vec![2; 8],
                        parent_span_id: vec![3; 8],
                        name: "test-span".to_owned(),
                        kind: span::SpanKind::Server as i32,
                        start_time_unix_nano: 1,
                        end_time_unix_nano: 2,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn metrics_pdata(interests: Interests, id: usize) -> OtapPdata {
        OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportMetricsRequest(Bytes::from(metrics_request().encode_to_vec()))
                .into(),
        )
        .test_subscribe_to(interests, TestCallData::default().into(), id)
    }

    fn traces_pdata(interests: Interests, id: usize) -> OtapPdata {
        OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportTracesRequest(Bytes::from(traces_request().encode_to_vec()))
                .into(),
        )
        .test_subscribe_to(interests, TestCallData::default().into(), id)
    }

    fn attempted_messages(snapshots: &[MetricSetSnapshot], outcome: Outcome) -> u64 {
        let outcome = match outcome {
            Outcome::Success => "success",
            Outcome::Failure => "failure",
            Outcome::Refused => "refused",
        };
        snapshots
            .iter()
            .find(|snapshot| {
                snapshot.descriptor().name == "exporter.attempted"
                    && snapshot.measurement_attribute_value("signal") == Some("logs")
                    && snapshot.measurement_attribute_value("outcome") == Some(outcome)
                    && snapshot
                        .descriptor()
                        .metrics
                        .iter()
                        .any(|metric| metric.name == "messages")
            })
            .map(|snapshot| {
                let index = snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .position(|metric| metric.name == "messages")
                    .expect("messages metric");
                snapshot.get_metrics()[index].to_u64_lossy()
            })
            .unwrap_or(0)
    }

    /// Scenario: a signal has a non-empty ADX JSON mapping name.
    /// Guarantees: the configured mapping name is preserved for ingestion requests.
    #[test]
    fn non_empty_json_mapping_is_preserved() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "metrics_table_json_mapping": "MetricsMapping"
            }"#,
        )
        .expect("deserialize");

        assert_eq!(
            Signal::Metrics.json_mapping(&config),
            Some("MetricsMapping")
        );
    }

    /// Scenario: raw OTLP payloads contain invalid top-level protobuf framing.
    /// Guarantees: logs, metrics, and traces are rejected as permanent refusals instead of becoming no-op ACKs.
    #[test]
    fn malformed_raw_otlp_is_permanently_refused_for_all_signals() {
        let exporter =
            lifecycle_exporter(lifecycle_config("http://localhost:4319", false, 1), None);
        for bytes in [
            Bytes::from_static(b"\xff"),
            Bytes::from_static(b"\x0a\x00\xff"),
        ] {
            let payloads = [
                OtapPayload::from(OtlpProtoBytes::ExportLogsRequest(bytes.clone())),
                OtapPayload::from(OtlpProtoBytes::ExportMetricsRequest(bytes.clone())),
                OtapPayload::from(OtlpProtoBytes::ExportTracesRequest(bytes.clone())),
            ];
            for payload in payloads {
                let error = exporter
                    .extract_records(&payload)
                    .expect_err("malformed OTLP must be rejected");
                assert!(error.is_permanent_refusal());
            }
        }
    }

    /// Scenario: raw OTLP payloads are empty but valid protobuf messages.
    /// Guarantees: logs, metrics, and traces remain valid no-op inputs.
    #[test]
    fn empty_raw_otlp_is_valid_for_all_signals() {
        let exporter =
            lifecycle_exporter(lifecycle_config("http://localhost:4319", false, 1), None);
        let payloads = [
            OtapPayload::from(OtlpProtoBytes::ExportLogsRequest(Bytes::new())),
            OtapPayload::from(OtlpProtoBytes::ExportMetricsRequest(Bytes::new())),
            OtapPayload::from(OtlpProtoBytes::ExportTracesRequest(Bytes::new())),
        ];

        for payload in payloads {
            let (_, rows) = exporter
                .extract_records(&payload)
                .expect("valid framing")
                .expect("supported signal");
            assert!(rows.is_empty());
        }
    }

    /// Scenario: logs, metrics, and traces arrive as OTAP Arrow records.
    /// Guarantees: all three OTAP view paths produce the same ADX row shapes as raw OTLP input.
    #[test]
    fn otap_arrow_transforms_all_supported_signals() {
        let exporter =
            lifecycle_exporter(lifecycle_config("http://localhost:4319", false, 1), None);
        let logs_request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("otap-log".to_owned())),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let logs_bytes = logs_request.encode_to_vec();
        let logs_view = RawLogsData::try_new(&logs_bytes).expect("valid logs");
        let logs_payload =
            OtapPayload::from(encode_logs_otap_batch(&logs_view).expect("encode OTAP logs"));
        let (signal, logs_rows) = exporter
            .extract_records(&logs_payload)
            .expect("transform OTAP logs")
            .expect("logs signal");
        assert_eq!(signal, Signal::Logs);
        assert_eq!(logs_rows.len(), 1);

        let metrics_bytes = metrics_request().encode_to_vec();
        let metrics_view = RawMetricsData::try_new(&metrics_bytes).expect("valid metrics");
        let metrics_payload = OtapPayload::from(
            encode_metrics_otap_batch(&metrics_view).expect("encode OTAP metrics"),
        );
        let (signal, metrics_rows) = exporter
            .extract_records(&metrics_payload)
            .expect("transform OTAP metrics")
            .expect("metrics signal");
        assert_eq!(signal, Signal::Metrics);
        assert_eq!(metrics_rows.len(), 1);

        let traces_bytes = traces_request().encode_to_vec();
        let traces_view = RawTraceData::try_new(&traces_bytes).expect("valid traces");
        let traces_payload =
            OtapPayload::from(encode_spans_otap_batch(&traces_view).expect("encode OTAP traces"));
        let (signal, trace_rows) = exporter
            .extract_records(&traces_payload)
            .expect("transform OTAP traces")
            .expect("traces signal");
        assert_eq!(signal, Signal::Traces);
        assert_eq!(trace_rows.len(), 1);
        let trace_row: serde_json::Value =
            serde_json::from_slice(&trace_rows[0]).expect("valid trace row");
        assert_eq!(trace_row["SpanName"], "test-span");
        assert_eq!(trace_row["SpanKind"], "Server");
        assert_eq!(trace_row["TraceID"], "01010101010101010101010101010101");
    }

    /// Scenario: an ADX batch contains sensitive telemetry and payload logging is disabled.
    /// Guarantees: neither a failed payload nor its sample row is retained for logging.
    #[test]
    fn failed_payload_content_is_opt_in() {
        let records = [Bytes::from_static(b"sensitive telemetry")];

        assert_eq!(
            AzureDataExplorerExporter::failed_payload(&records, false),
            None
        );
        assert_eq!(
            AzureDataExplorerExporter::failed_sample_row(&records, false),
            None
        );
        assert!(AzureDataExplorerExporter::failed_payload(&records, true).is_some());
        assert!(AzureDataExplorerExporter::failed_sample_row(&records, true).is_some());
    }

    /// Scenario: appending one inbound PData would cross a coalesced request's row or byte bound.
    /// Guarantees: the boundary is detected before append so the inbound PData can remain intact in its own request.
    #[test]
    fn coalesced_request_boundaries_do_not_require_splitting_an_inbound_message() {
        let mut batch = BatchAccumulator::new(true, 3, 10);
        batch.rows = vec![Bytes::from_static(b"aa"), Bytes::from_static(b"bb")];
        batch.bytes = 5;

        assert!(batch.would_exceed(&[Bytes::from_static(b"c"), Bytes::from_static(b"d")], 3, 10));
        assert!(batch.would_exceed(&[Bytes::from_static(b"12345")], 3, 10));
        assert!(!batch.would_exceed(&[Bytes::from_static(b"1234")], 3, 10));
    }

    /// Scenario: appending a row crosses a coalesced request's byte limit only because of JSON Lines framing.
    /// Guarantees: newline separators count toward byte-boundary preflight.
    #[test]
    fn coalesced_request_byte_limit_includes_newline_separators() {
        let mut batch = BatchAccumulator::new(true, 0, 8);
        batch.rows.push(Bytes::from_static(b"1234"));
        batch.bytes = 4;

        assert!(batch.would_exceed(&[Bytes::from_static(b"5678")], 0, 8));
    }

    /// Scenario: request coalescing is disabled while row thresholds retain their defaults.
    /// Guarantees: threshold values alone cannot implicitly enable cross-message acknowledgement retention.
    #[test]
    fn request_coalescing_requires_explicit_opt_in() {
        let batch = BatchAccumulator::new(false, 1_000, 4 * 1024 * 1024);

        assert!(!batch.is_enabled());
    }

    /// Scenario: one coalesced source message expands into a different number of ADX rows.
    /// Guarantees: batch bookkeeping preserves source signal-item counts independently of row counts.
    #[test]
    fn batch_tracks_source_items_separately_from_rows() {
        let mut batch = BatchAccumulator::new(true, 100, 1024);
        let pdata = OtapPdata::new_default(OtapPayload::empty(SignalType::Metrics));

        assert!(!batch.push(
            vec![Bytes::from_static(b"one"), Bytes::from_static(b"two")],
            pdata,
            7,
        ));
        let (_, _, row_count, item_count, message_count) = batch.take();

        assert_eq!(row_count, 2);
        assert_eq!(item_count, 7);
        assert_eq!(message_count, 1);
    }

    /// Scenario: shutdown reaches its deadline while an export future remains pending.
    /// Guarantees: cancellation recovers ownership and records the abandoned attempt as a failure.
    #[tokio::test]
    async fn canceling_in_flight_exports_records_failure() {
        let pdata = OtapPdata::new_default(OtapPayload::empty(SignalType::Logs));
        let mut exporter =
            lifecycle_exporter(lifecycle_config("http://localhost:4319", false, 1), None);
        exporter.in_flight_exports.pending_messages = 1;
        exporter.in_flight_exports.exports.push(PendingExport {
            future: Box::pin(std::future::pending()),
            signal: Signal::Logs,
            table: "OTELLogs".to_string(),
            json_mapping: None,
            sample_row: None,
            failed_payload: None,
            row_count: 1,
            item_count: 1,
            message_count: 1,
            compressed: Bytes::new(),
            pdata_batch: vec![pdata],
            token_generation: 1,
            auth_retry_attempts: 0,
        });

        let canceled = exporter.in_flight_exports.cancel_all();
        let message_count = exporter.record_abandoned_exports(canceled).await;
        let snapshots = exporter.metrics.borrow_mut().terminal_snapshots();

        assert_eq!(message_count, 1);
        assert_eq!(exporter.in_flight_exports.len(), 0);
        assert_eq!(exporter.in_flight_exports.pending_messages(), 0);
        assert_eq!(attempted_messages(&snapshots, Outcome::Failure), 1);
    }

    /// Scenario: ADX returns terminal and transient export failures.
    /// Guarantees: terminal refusals are permanent and transient failures remain retryable.
    #[test]
    fn export_nacks_preserve_retry_disposition() {
        let refused = AzureDataExplorerExporter::nack_for_export_error(
            true,
            "bad request".to_string(),
            OtapPdata::new_default(OtapPayload::empty(SignalType::Logs)),
        );
        let transient = AzureDataExplorerExporter::nack_for_export_error(
            false,
            "server error".to_string(),
            OtapPdata::new_default(OtapPayload::empty(SignalType::Logs)),
        );

        assert!(refused.permanent);
        assert_eq!(refused.cause, NackCause::Refused);
        assert!(!transient.permanent);
        assert_eq!(transient.cause, NackCause::Unspecified);
    }

    /// Scenario: an empty OTLP logs request runs through the complete exporter loop.
    /// Guarantees: the request is ACKed and records one successful no-op exporter attempt.
    #[test]
    fn exporter_loop_records_successful_noop_attempt() {
        let (runtime, wrapper) =
            lifecycle_runtime(lifecycle_config("http://localhost:4319", false, 1), None);
        let validation = runtime.set_exporter(wrapper).run_test(|ctx| async move {
            ctx.send_pdata(logs_pdata(None, Interests::ACKS, 1))
                .await
                .expect("send empty logs");
            tokio::time::sleep(Duration::from_millis(20)).await;
            ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test complete")
                .await
                .expect("send shutdown");
        });

        validation.run_validation(|mut ctx, result| async move {
            result.expect("exporter terminates");
            let mut completion_rx = ctx
                .take_pipeline_completion_receiver()
                .expect("completion receiver");
            match completion_rx.recv().await.expect("receive completion") {
                PipelineCompletionMsg::DeliverAck { .. } => {}
                PipelineCompletionMsg::DeliverNack { .. } => panic!("expected ACK"),
            }
        });
    }

    /// Scenario: successful no-op handling completes without an HTTP submission.
    /// Guarantees: shared exporter metrics record one successful attempt.
    #[tokio::test]
    async fn successful_noop_records_attempt_metric() {
        let exporter =
            lifecycle_exporter(lifecycle_config("http://localhost:4319", false, 1), None);

        exporter
            .record_preparation_success(SignalType::Logs, 0)
            .await;
        let snapshots = exporter.metrics.borrow_mut().terminal_snapshots();

        assert_eq!(attempted_messages(&snapshots, Outcome::Success), 1);
    }

    /// Scenario: shutdown is requested while one request is stalled and another PData is buffered.
    /// Guarantees: the buffered PData is nacked as node shutdown and the exporter returns by the deadline.
    #[test]
    fn exporter_loop_honors_shutdown_while_at_capacity() {
        let server_runtime = tokio::runtime::Runtime::new().expect("wiremock runtime");
        let server = Arc::new(server_runtime.block_on(MockServer::start()));
        server_runtime.block_on(
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(204).set_delay(Duration::from_secs(5)))
                .mount(&server),
        );
        let (runtime, wrapper) = lifecycle_runtime(lifecycle_config(&server.uri(), false, 1), None);
        let started = Instant::now();
        let scenario_server = Arc::clone(&server);
        let validation = runtime.set_exporter(wrapper).run_test(|ctx| async move {
            ctx.send_pdata(logs_pdata(Some("first"), Interests::empty(), 1))
                .await
                .expect("send first logs");
            tokio::time::timeout(Duration::from_secs(1), async {
                loop {
                    if !scenario_server
                        .received_requests()
                        .await
                        .expect("requests recorded")
                        .is_empty()
                    {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .expect("first request starts");
            ctx.send_pdata(logs_pdata(Some("second"), Interests::NACKS, 2))
                .await
                .expect("send buffered logs");
            ctx.send_shutdown(
                Instant::now() + Duration::from_millis(150),
                "capacity shutdown",
            )
            .await
            .expect("send shutdown");
        });

        validation.run_validation(|mut ctx, result| async move {
            result.expect("exporter terminates");
            let mut completion_rx = ctx
                .take_pipeline_completion_receiver()
                .expect("completion receiver");
            match completion_rx.recv().await.expect("receive shutdown nack") {
                PipelineCompletionMsg::DeliverNack { nack } => {
                    assert_eq!(nack.cause, NackCause::NodeShutdown);
                }
                PipelineCompletionMsg::DeliverAck { .. } => panic!("expected shutdown NACK"),
            }
            assert!(started.elapsed() < Duration::from_secs(2));
        });
    }

    /// Scenario: shutdown completion routing is backpressured until after its deadline.
    /// Guarantees: deadline-aware NACK delivery returns without waiting for channel capacity.
    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_nack_delivery_observes_deadline() {
        let (effect_handler, completion_tx, _completion_rx) = backpressured_effect_handler();
        completion_tx
            .send(PipelineCompletionMsg::DeliverAck {
                ack: AckMsg::new(OtapPdata::new_default(OtapPayload::empty(SignalType::Logs))),
            })
            .await
            .expect("fill completion channel");
        let nack = NackMsg::new_with_cause(
            "shutdown",
            logs_pdata(None, Interests::NACKS, 1),
            NackCause::NodeShutdown,
        );

        let delivered = AzureDataExplorerExporter::notify_nack_until(
            &effect_handler,
            nack,
            Some(tokio::time::Instant::now() + Duration::from_millis(20)),
        )
        .await
        .expect("deadline-aware notification");

        assert!(!delivered);
    }

    /// Scenario: ADX rejects the first token and the provider publishes a replacement.
    /// Guarantees: a coalesced request is retained, retried with the new token, and ACKed.
    #[test]
    fn exporter_loop_retries_unauthorized_with_replacement_token() {
        let server_runtime = tokio::runtime::Runtime::new().expect("wiremock runtime");
        let server = server_runtime.block_on(MockServer::start());
        server_runtime.block_on(
            Mock::given(method("POST"))
                .and(header("authorization", "Bearer token-1"))
                .respond_with(ResponseTemplate::new(401))
                .expect(1)
                .mount(&server),
        );
        server_runtime.block_on(
            Mock::given(method("POST"))
                .and(header("authorization", "Bearer token-2"))
                .respond_with(ResponseTemplate::new(204))
                .expect(1)
                .mount(&server),
        );
        let (runtime, wrapper) = lifecycle_runtime(
            lifecycle_config(&server.uri(), true, 1),
            Some(Duration::from_millis(100)),
        );
        let validation = runtime.set_exporter(wrapper).run_test(|ctx| async move {
            ctx.send_pdata(logs_pdata(Some("reauth"), Interests::ACKS, 1))
                .await
                .expect("send logs");
            tokio::time::sleep(Duration::from_millis(300)).await;
            ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test complete")
                .await
                .expect("send shutdown");
        });

        validation.run_validation(|mut ctx, result| async move {
            result.expect("exporter terminates");
            let mut completion_rx = ctx
                .take_pipeline_completion_receiver()
                .expect("completion receiver");
            match completion_rx.recv().await.expect("receive completion") {
                PipelineCompletionMsg::DeliverAck { .. } => {}
                PipelineCompletionMsg::DeliverNack { nack } => {
                    panic!("expected ACK after token replacement: {}", nack.reason)
                }
            }
        });
    }

    /// Scenario: metrics and traces run through the complete exporter loop and ADX accepts both.
    /// Guarantees: each source message receives an ACK after its signal-specific request succeeds.
    #[test]
    fn exporter_loop_acks_metrics_and_traces() {
        let server_runtime = tokio::runtime::Runtime::new().expect("wiremock runtime");
        let server = server_runtime.block_on(MockServer::start());
        server_runtime.block_on(
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(204))
                .expect(2)
                .mount(&server),
        );
        let (runtime, wrapper) = lifecycle_runtime(lifecycle_config(&server.uri(), false, 2), None);
        let validation = runtime.set_exporter(wrapper).run_test(|ctx| async move {
            ctx.send_pdata(metrics_pdata(Interests::ACKS, 1))
                .await
                .expect("send metrics");
            ctx.send_pdata(traces_pdata(Interests::ACKS, 2))
                .await
                .expect("send traces");
            tokio::time::sleep(Duration::from_millis(100)).await;
            ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test complete")
                .await
                .expect("send shutdown");
        });

        validation.run_validation(|mut ctx, result| async move {
            result.expect("exporter terminates");
            let mut completion_rx = ctx
                .take_pipeline_completion_receiver()
                .expect("completion receiver");
            for _ in 0..2 {
                match completion_rx.recv().await.expect("receive completion") {
                    PipelineCompletionMsg::DeliverAck { .. } => {}
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        panic!("expected ACK: {}", nack.reason)
                    }
                }
            }
        });
    }

    /// Scenario: ADX permanently rejects metric and trace requests.
    /// Guarantees: both source messages receive permanent refused NACKs.
    #[test]
    fn exporter_loop_nacks_metrics_and_traces_on_client_refusal() {
        let server_runtime = tokio::runtime::Runtime::new().expect("wiremock runtime");
        let server = server_runtime.block_on(MockServer::start());
        server_runtime.block_on(
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(400).set_body_string("invalid schema"))
                .expect(2)
                .mount(&server),
        );
        let (runtime, wrapper) = lifecycle_runtime(lifecycle_config(&server.uri(), false, 2), None);
        let validation = runtime.set_exporter(wrapper).run_test(|ctx| async move {
            ctx.send_pdata(metrics_pdata(Interests::NACKS, 1))
                .await
                .expect("send metrics");
            ctx.send_pdata(traces_pdata(Interests::NACKS, 2))
                .await
                .expect("send traces");
            tokio::time::sleep(Duration::from_millis(100)).await;
            ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test complete")
                .await
                .expect("send shutdown");
        });

        validation.run_validation(|mut ctx, result| async move {
            result.expect("exporter terminates");
            let mut completion_rx = ctx
                .take_pipeline_completion_receiver()
                .expect("completion receiver");
            for _ in 0..2 {
                match completion_rx.recv().await.expect("receive completion") {
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        assert!(nack.permanent);
                        assert_eq!(nack.cause, NackCause::Refused);
                    }
                    PipelineCompletionMsg::DeliverAck { .. } => {
                        panic!("expected permanent NACK")
                    }
                }
            }
        });
    }

    /// Scenario: multiple JSON records are compressed into one ADX request body.
    /// Guarantees: the uncompressed payload is JSON Lines with exactly one complete object per line.
    #[tokio::test]
    async fn compressed_batch_uses_json_lines_framing() {
        let compressed = AzureDataExplorerExporter::compress_batch(
            &[
                Bytes::from_static(br#"{"id":1}"#),
                Bytes::from_static(br#"{"id":2}"#),
            ],
            6,
        )
        .await
        .expect("compress JSON Lines");
        let mut decoder = GzDecoder::new(compressed.as_ref());
        let mut decoded = String::new();
        let _ = decoder
            .read_to_string(&mut decoded)
            .expect("decompress JSON Lines");

        assert_eq!(decoded, "{\"id\":1}\n{\"id\":2}");
    }

    /// Scenario: compression receives more than one local-runtime work quantum.
    /// Guarantees: compression yields before processing the complete request.
    #[tokio::test]
    async fn compression_yields_at_bounded_intervals() {
        let records = [Bytes::from(vec![b'x'; COMPRESSION_YIELD_BYTES + 1])];
        let mut compression = Box::pin(AzureDataExplorerExporter::compress_batch(&records, 6));

        assert!(futures::poll!(compression.as_mut()).is_pending());
        let _ = compression.await.expect("compress bounded input");
    }

    /// Scenario: a serialized row exceeds the configured ADX row-size limit.
    /// Guarantees: preflight rejects it before the row enters batching or HTTP submission.
    #[test]
    fn oversized_row_is_rejected_before_batching() {
        let records = [Bytes::from_static(b"1234")];

        assert!(matches!(
            AzureDataExplorerExporter::validate_row_sizes(&records, 4),
            Err(Error::RowTooLarge {
                actual: 5,
                limit: 4
            })
        ));
        assert!(AzureDataExplorerExporter::validate_row_sizes(&records, 5).is_ok());
    }
}
