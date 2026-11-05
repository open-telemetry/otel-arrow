// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded, node-local streaming ingestion with optional request coalescing.

use async_trait::async_trait;
use bytes::Bytes;
use flate2::{Compression, write::GzEncoder};
use futures::{StreamExt, future::LocalBoxFuture, stream::FuturesUnordered};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::ConsumerEffectHandlerExtension;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NackCause, NackMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::http_client_auth::{
    HttpClientAuthProvider, HttpClientAuthProviderEvents,
    new_http_client_auth_provider_from_bearer_token_provider,
};
use otel_arrow_dfe_otap::metrics::ExporterAttempt;
use otel_arrow_dfe_otap::pdata::{OtapPdata, PdataEffectHandlerExtension};
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::MetricsData;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::TracesData;
use otel_arrow_dfe_pdata::views::otap::{OtapLogsView, OtapMetricsView, OtapTracesView};
use otel_arrow_dfe_pdata_codec::{
    AcceptedEncodings, CodecError, CodecOperation, PdataView,
    builtins::{OTLP_ENCODING, resolve_otlp},
};
use otel_arrow_dfe_telemetry::common_attributes::Outcome;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::poll_fn;
use std::io::Write;
use std::rc::Rc;
use std::task::{Context as TaskContext, Poll, Waker};
use std::time::Duration;
use tokio::time::Instant;

use super::client::{AzureDataExplorerClient, ExportAttemptMetadata};
use super::config::Config;
use super::decode::{AdmissionError, DecodeAdmission, DecodeSchema};
use super::error::Error;
use super::metrics::{
    AzureDataExplorerExporterMetricsRc, AzureDataExplorerExporterMetricsTracker, PendingAttempt,
};
use super::transformer::Transformer;

const COMPRESSION_YIELD_BYTES: usize = 64 * 1024;
const COMPRESSION_TASK_WORK: usize = 4 * COMPRESSION_YIELD_BYTES;
const MAX_DECODE_ALLOCATION: usize = 16 * 1024 * 1024;
const MAX_DECODE_WORK: usize = 65_536;
const AUTH_EVENTS: HttpClientAuthProviderEvents = HttpClientAuthProviderEvents {
    validate_header_name: |name| {
        if *name == http::header::AUTHORIZATION {
            Ok(())
        } else {
            Err("ADX requires an Authorization header".to_string())
        }
    },
    on_invalid: |_, error| {
        otel_warn!("azure_data_explorer_exporter.auth.invalid_bearer_token", error = %error);
    },
    on_stream_closed: |_| {
        otel_warn!("azure_data_explorer_exporter.auth.token_stream_closed");
    },
};

struct Batch {
    sources: Vec<OtapPdata>,
    records: Vec<Bytes>,
    signal: SignalType,
    items: u64,
    bytes: usize,
    flush_at: Instant,
    sealed: bool,
    compressed: Option<Bytes>,
    auth_retries: u32,
    sample_row: Option<String>,
    failed_payload: Option<String>,
}

type ExportResult = (Result<Duration, Error>, Option<Bytes>);

struct PendingExport {
    future: LocalBoxFuture<'static, ExportResult>,
    batch: Batch,
    token_generation: u64,
}

type Resolutions = FuturesUnordered<LocalBoxFuture<'static, Result<(), EngineError>>>;

#[derive(Debug, PartialEq)]
struct ExtractedRecords {
    records: Vec<Bytes>,
    items: u64,
}

struct PreparingInput {
    // Delivery ownership stays outside the cancellable codec future.
    pdata: OtapPdata,
    future: LocalBoxFuture<'static, Result<ExtractedRecords, Error>>,
}

#[derive(Default)]
struct CompressionBudget {
    used: Cell<usize>,
    waiter: RefCell<Option<Waker>>,
    #[cfg(test)]
    started: Cell<usize>,
}

impl CompressionBudget {
    async fn take(&self, requested: usize) -> usize {
        poll_fn(|_| {
            let available = COMPRESSION_TASK_WORK - self.used.get();
            if available == 0 {
                return Poll::Pending;
            }
            let amount = available.min(requested);
            self.used.set(self.used.get() + amount);
            if self.used.get() == COMPRESSION_TASK_WORK
                && let Some(waiter) = self.waiter.borrow_mut().take()
            {
                waiter.wake();
            }
            Poll::Ready(amount)
        })
        .await
    }

    async fn charge(&self, mut amount: usize) {
        while amount != 0 {
            amount -= self.take(amount).await;
        }
    }

    fn poll_exhausted(&self, cx: &mut TaskContext<'_>) -> Poll<()> {
        if self.used.get() == COMPRESSION_TASK_WORK {
            Poll::Ready(())
        } else {
            let _ = self.waiter.borrow_mut().replace(cx.waker().clone());
            Poll::Pending
        }
    }

    async fn replenish(&self) {
        // Called in the outer task, never in a child selected alongside ready input.
        tokio::task::yield_now().await;
        self.used.set(0);
    }
}

/// Azure Data Explorer (ADX) exporter for logs, metrics, and traces.
pub struct AzureDataExplorerExporter {
    config: Config,
    transformer: Transformer,
    metrics: AzureDataExplorerExporterMetricsRc,
    client: AzureDataExplorerClient,
    auth: Box<dyn HttpClientAuthProvider>,
    accepted_encodings: AcceptedEncodings,
    // All compression futures run on this node's core and share one work quantum.
    compression_budget: Rc<CompressionBudget>,
}

impl AzureDataExplorerExporter {
    /// Build a bounded streaming exporter using the supplied token provider.
    pub fn new(
        pipeline_ctx: PipelineContext,
        config: Config,
        token_provider: Box<dyn BearerTokenProvider>,
    ) -> Result<Self, Error> {
        config.validate()?;
        let metrics = Rc::new(RefCell::new(
            AzureDataExplorerExporterMetricsTracker::register(&pipeline_ctx),
        ));
        let http_client = AzureDataExplorerClient::http_client_builder()
            .build()
            .map_err(Error::ClientInit)?;
        let client = AzureDataExplorerClient::new(
            &config.cluster_uri,
            &config.db_name,
            http_client,
            metrics.clone(),
            config.max_retries,
            config.timeout,
            config.log_response_body,
        );
        let transformer = Transformer::new()
            .with_legacy_logs_body_string(config.tables.logs.legacy_body_string)
            .with_log_event_name_options(
                config.tables.logs.export_event_name,
                config.tables.logs.add_event_name_to_attributes,
            );
        let otlp = resolve_otlp().map_err(|error| Error::Config(error.to_string()))?;
        Ok(Self {
            config,
            transformer,
            metrics,
            client,
            auth: Box::new(new_http_client_auth_provider_from_bearer_token_provider(
                token_provider,
            )),
            accepted_encodings: AcceptedEncodings::accept_encoded([otlp]),
            compression_budget: Rc::default(),
        })
    }

    fn table(&self, signal: SignalType) -> (&str, Option<&str>) {
        let (name, mapping) = match signal {
            SignalType::Logs => (
                &self.config.tables.logs.name,
                &self.config.tables.logs.mapping,
            ),
            SignalType::Metrics => (
                &self.config.tables.metrics.name,
                &self.config.tables.metrics.mapping,
            ),
            SignalType::Traces => (
                &self.config.tables.traces.name,
                &self.config.tables.traces.mapping,
            ),
        };
        (name, mapping.as_deref().filter(|name| !name.is_empty()))
    }

    async fn decode<T: DecodeSchema + prost::Message + Default>(bytes: &[u8]) -> Result<T, Error> {
        let decode_error = |error| match error {
            AdmissionError::Allocation(limit) => Error::DecodeLimit {
                resource: "allocation_bytes",
                limit,
            },
            AdmissionError::Work(limit) => Error::DecodeLimit {
                resource: "fields_or_elements",
                limit,
            },
            error => Error::Decode(Box::new(CodecError::operation(
                &OTLP_ENCODING,
                CodecOperation::Decode,
                error,
            ))),
        };
        let mut admission =
            DecodeAdmission::new::<T>(bytes, MAX_DECODE_ALLOCATION, MAX_DECODE_WORK)
                .map_err(decode_error)?;
        while !admission.step(256, 64 * 1024).map_err(decode_error)? {
            tokio::task::yield_now().await;
        }
        T::decode(bytes).map_err(|error| {
            Error::Decode(Box::new(CodecError::operation(
                &OTLP_ENCODING,
                CodecOperation::Decode,
                error,
            )))
        })
    }

    fn extract_records(
        &self,
        pdata: &OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> LocalBoxFuture<'static, Result<ExtractedRecords, Error>> {
        let rows = self.config.network_requests.max_rows;
        let row_bytes = self.config.max_row_bytes;
        let bytes = self.config.network_requests.max_bytes;
        let payload = pdata.payload_ref().clone();
        let signal = pdata.signal_type();
        let accepted = self.accepted_encodings.clone();
        let effect_handler = effect_handler.clone();
        let transformer = self.transformer.clone();
        Box::pin(async move {
            let view = effect_handler
                .view(&payload, &accepted)
                .await
                .map_err(|error| Error::Decode(Box::new(error)))?;
            let view_error = |error| Error::PayloadView(Box::new(error));
            let (records, items) = match view {
                PdataView::Native(records) => {
                    let items = records.num_items() as u64;
                    let records = match signal {
                        SignalType::Logs => transformer.convert_logs(
                            &OtapLogsView::try_from(records.as_ref()).map_err(view_error)?,
                            rows,
                            row_bytes,
                            bytes,
                        ),
                        SignalType::Metrics => transformer.convert_metrics(
                            &OtapMetricsView::try_from(records.as_ref()).map_err(view_error)?,
                            rows,
                            row_bytes,
                            bytes,
                        ),
                        SignalType::Traces => transformer.convert_traces(
                            &OtapTracesView::try_from(records.as_ref()).map_err(view_error)?,
                            rows,
                            row_bytes,
                            bytes,
                        ),
                    }?;
                    (records, items)
                }
                PdataView::Encoded(encoded) => {
                    let decoded = match encoded.signal_type() {
                        SignalType::Logs => {
                            OtlpProtoMessage::Logs(Self::decode::<LogsData>(encoded.bytes()).await?)
                        }
                        SignalType::Metrics => OtlpProtoMessage::Metrics(
                            Self::decode::<MetricsData>(encoded.bytes()).await?,
                        ),
                        SignalType::Traces => OtlpProtoMessage::Traces(
                            Self::decode::<TracesData>(encoded.bytes()).await?,
                        ),
                    };
                    let items = decoded.num_items() as u64;
                    let records = match &decoded {
                        OtlpProtoMessage::Logs(logs) => {
                            transformer.convert_logs(logs, rows, row_bytes, bytes)
                        }
                        OtlpProtoMessage::Metrics(metrics) => {
                            transformer.convert_metrics(metrics, rows, row_bytes, bytes)
                        }
                        OtlpProtoMessage::Traces(traces) => {
                            transformer.convert_traces(traces, rows, row_bytes, bytes)
                        }
                    }?;
                    (records, items)
                }
            };
            Ok(ExtractedRecords { records, items })
        })
    }

    fn start_preparation(
        &self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> PreparingInput {
        let future = if self.auth.is_ready() {
            self.extract_records(&pdata, effect_handler)
        } else {
            Box::pin(std::future::ready(Err(Error::AuthenticationUnavailable)))
        };
        PreparingInput { pdata, future }
    }

    async fn compress_batch(
        records: &[Bytes],
        level: u32,
        budget: &CompressionBudget,
    ) -> Result<Bytes, Error> {
        budget.charge(1024).await;
        #[cfg(test)]
        budget.started.set(budget.started.get() + 1);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        let mut written = 0;
        for (index, record) in records.iter().enumerate() {
            if index != 0 {
                budget.charge(1).await;
                encoder.write_all(b"\n")?;
                written += 1;
                if written == COMPRESSION_YIELD_BYTES {
                    tokio::task::yield_now().await;
                    written = 0;
                }
            }
            let mut remaining = record.as_ref();
            while !remaining.is_empty() {
                let length = budget
                    .take(remaining.len().min(COMPRESSION_YIELD_BYTES - written))
                    .await;
                encoder.write_all(&remaining[..length])?;
                remaining = &remaining[length..];
                written += length;
                if written == COMPRESSION_YIELD_BYTES {
                    tokio::task::yield_now().await;
                    written = 0;
                }
            }
        }
        budget.charge(1024).await;
        Ok(Bytes::from(encoder.finish()?))
    }

    fn attempt(&self, signal: SignalType, items: u64) -> ExporterAttempt {
        let mut attempt = self.metrics.borrow().boundary.attempt(signal);
        attempt.set_item_count_with(|| items);
        attempt
    }

    fn record_preparation(&self, signal: SignalType, items: u64, outcome: Outcome) {
        let attempt = self.attempt(signal, items);
        let result = match outcome {
            Outcome::Success => Ok(()),
            Outcome::Refused => Err(attempt.refused(())),
            _ => Err(attempt.failed(())),
        };
        let _ = self
            .metrics
            .borrow_mut()
            .boundary
            .record(attempt.complete(result));
    }

    fn queue_resolution(
        &self,
        pdata: OtapPdata,
        signal: SignalType,
        error: Option<&Error>,
        effect_handler: &EffectHandler<OtapPdata>,
        resolutions: &Resolutions,
    ) {
        self.queue_batch_resolution(vec![pdata], signal, error, effect_handler, resolutions);
    }

    fn prepare(
        &self,
        pdata: OtapPdata,
        extracted: Result<ExtractedRecords, Error>,
        effect_handler: &EffectHandler<OtapPdata>,
        resolutions: &Resolutions,
    ) -> Option<Batch> {
        let signal = pdata.signal_type();
        let items = extracted.as_ref().map_or_else(
            |_| pdata.payload_ref().known_item_count().unwrap_or(0) as u64,
            |extracted| extracted.items,
        );
        if matches!(&extracted, Err(Error::AuthenticationUnavailable)) {
            self.record_preparation(signal, items, Outcome::Failure);
            self.metrics
                .borrow_mut()
                .record_batch(signal, Outcome::Failure);
            let handler = effect_handler.clone();
            let nack = NackMsg::new(self.auth.not_ready_reason(), pdata);
            resolutions.push(Box::pin(async move { handler.notify_nack(nack).await }));
            return None;
        }
        let records = match extracted {
            Ok(extracted) => extracted.records,
            Err(error) => {
                self.metrics.borrow_mut().record_input_error(signal, &error);
                let outcome = if error.is_refusal() {
                    Outcome::Refused
                } else {
                    Outcome::Failure
                };
                self.record_preparation(signal, items, outcome);
                self.queue_resolution(pdata, signal, Some(&error), effect_handler, resolutions);
                return None;
            }
        };
        if records.is_empty() {
            otel_debug!("azure_data_explorer_exporter.message.no_records", signal = ?signal);
            self.record_preparation(signal, items, Outcome::Success);
            self.queue_resolution(pdata, signal, None, effect_handler, resolutions);
            return None;
        }
        Some(Batch {
            bytes: records.iter().map(|record| record.len() + 1).sum(),
            sources: vec![pdata],
            records,
            signal,
            items,
            flush_at: Instant::now() + self.config.network_requests.flush_interval,
            sealed: !self.config.network_requests.coalesce,
            compressed: None,
            auth_retries: 0,
            sample_row: None,
            failed_payload: None,
        })
    }

    fn append_batch(&self, batch: Batch, batches: &mut VecDeque<Batch>) {
        let limits = &self.config.network_requests;
        if limits.coalesce
            && let Some(index) = batches
                .iter()
                .position(|pending| pending.signal == batch.signal && !pending.sealed)
        {
            let pending = &mut batches[index];
            if pending.records.len() + batch.records.len() <= limits.max_rows
                && pending.bytes + batch.bytes <= limits.max_bytes
                && pending.sources.len() + batch.sources.len() <= limits.max_pending_messages
            {
                pending.records.extend(batch.records);
                pending.sources.extend(batch.sources);
                pending.items += batch.items;
                pending.bytes += batch.bytes;
                return;
            }
            pending.sealed = true;
        }
        batches.push_back(batch);
    }

    fn dispatch_batch(
        &self,
        mut batch: Batch,
        header: http::HeaderValue,
        token_generation: u64,
    ) -> PendingExport {
        let (table, mapping) = self.table(batch.signal);
        let table = table.to_string();
        let mapping = mapping.map(str::to_string);
        let records = std::mem::take(&mut batch.records);
        if self.config.log_failed_payload && batch.failed_payload.is_none() {
            batch.failed_payload = Some(
                records
                    .iter()
                    .map(|row| String::from_utf8_lossy(row))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            batch.sample_row = records
                .first()
                .map(|row| String::from_utf8_lossy(row).chars().take(2048).collect());
        }
        let encoded = batch.compressed.take();
        let signal = batch.signal;
        let items = batch.items;
        let mut attempt = PendingAttempt::new(self.metrics.clone(), self.attempt(signal, items));
        let client = self.client.clone();
        let level = self.config.gzip_compression_level;
        let compression_budget = self.compression_budget.clone();
        let future = Box::pin(async move {
            let compressed = match encoded {
                Some(encoded) => encoded,
                None => match Self::compress_batch(&records, level, &compression_budget).await {
                    Ok(compressed) => compressed,
                    Err(error) => {
                        otel_error!("azure_data_explorer_exporter.compression_failed", error = %error);
                        return (Err(error), None);
                    }
                },
            };
            drop(records);
            let metadata = ExportAttemptMetadata {
                signal,
                items,
                payload_size: compressed.len(),
            };
            let result = client
                .send(
                    &table,
                    mapping.as_deref(),
                    header,
                    compressed.clone(),
                    attempt.take(),
                    metadata,
                )
                .await;
            (result, Some(compressed))
        });
        PendingExport {
            future,
            batch,
            token_generation,
        }
    }

    async fn next_export(exports: &mut Vec<PendingExport>) -> (PendingExport, ExportResult) {
        let (result, index, remaining) =
            futures::future::select_all(exports.iter_mut().map(|export| &mut export.future)).await;
        drop(remaining);
        (exports.swap_remove(index), result)
    }

    fn complete_export(
        &mut self,
        mut export: PendingExport,
        (mut result, compressed): ExportResult,
        effect_handler: &EffectHandler<OtapPdata>,
        resolutions: &Resolutions,
        batches: &mut VecDeque<Batch>,
    ) {
        if let Err(error) = &result {
            if error.is_unauthorized() {
                self.auth.invalidate(export.token_generation);
                if self.config.network_requests.coalesce {
                    if export.batch.auth_retries < self.config.max_retries {
                        export.batch.auth_retries += 1;
                        export.batch.compressed = compressed;
                        export.batch.sealed = true;
                        otel_warn!(
                            "azure_data_explorer_exporter.export.retained_after_unauthorized",
                            signal = ?export.batch.signal,
                            messages = export.batch.sources.len(),
                            retry = export.batch.auth_retries
                        );
                        batches.push_front(export.batch);
                        return;
                    }
                    result = Err(Error::ExportFailed {
                        attempts: export.batch.auth_retries + 1,
                        last_error: Box::new(result.expect_err("unauthorized result")),
                    });
                }
            }
            if let Some(sample) = export.batch.sample_row {
                otel_debug!(
                    "azure_data_explorer_exporter.export.sample_row",
                    message = sample
                );
            }
            if let Some(payload) = export.batch.failed_payload {
                otel_debug!(
                    "azure_data_explorer_exporter.export.failed_payload",
                    message = payload
                );
            }
        }
        self.queue_batch_resolution(
            export.batch.sources,
            export.batch.signal,
            result.as_ref().err(),
            effect_handler,
            resolutions,
        );
    }

    fn queue_batch_resolution(
        &self,
        sources: Vec<OtapPdata>,
        signal: SignalType,
        error: Option<&Error>,
        effect_handler: &EffectHandler<OtapPdata>,
        resolutions: &Resolutions,
    ) {
        let outcome = match error {
            None => Outcome::Success,
            Some(error) if error.is_refusal() => Outcome::Refused,
            Some(_) => Outcome::Failure,
        };
        self.metrics.borrow_mut().record_batch(signal, outcome);
        if let Some(error) = error {
            let reason = error.log_safe_summary();
            otel_warn!(
                "azure_data_explorer_exporter.export.failed",
                signal = ?signal,
                table = self.table(signal).0,
                error = reason.as_str()
            );
        }
        for pdata in sources {
            let handler = effect_handler.clone();
            let resolution: LocalBoxFuture<'static, Result<(), EngineError>> = match error {
                Some(error) => {
                    let nack = if error.is_permanent_refusal() {
                        NackMsg::new_permanent_with_cause(
                            error.log_safe_summary(),
                            pdata,
                            NackCause::Refused,
                        )
                    } else {
                        NackMsg::new(error.log_safe_summary(), pdata)
                    };
                    Box::pin(async move { handler.notify_nack(nack).await })
                }
                None => Box::pin(async move { handler.notify_ack(AckMsg::new(pdata)).await }),
            };
            resolutions.push(resolution);
        }
    }

    fn pending_messages(
        exports: &[PendingExport],
        resolutions: &Resolutions,
        batches: &VecDeque<Batch>,
    ) -> usize {
        resolutions.len()
            + exports
                .iter()
                .map(|export| export.batch.sources.len())
                .sum::<usize>()
            + batches
                .iter()
                .map(|batch| batch.sources.len())
                .sum::<usize>()
    }

    fn dispatch_ready(
        &self,
        exports: &mut Vec<PendingExport>,
        batches: &mut VecDeque<Batch>,
        full: bool,
    ) {
        while exports.len() < self.config.max_in_flight {
            let Some((_, header, generation)) = self.auth.header().filter(|_| self.auth.is_ready())
            else {
                break;
            };
            let Some(index) = batches.iter().position(|batch| {
                batch.sealed
                    || full
                    || batch.flush_at <= Instant::now()
                    || batch.records.len() >= self.config.network_requests.max_rows
                    || batch.bytes >= self.config.network_requests.max_bytes
            }) else {
                break;
            };
            let batch = batches.remove(index).expect("selected batch");
            if !batch.sealed && batch.flush_at <= Instant::now() {
                otel_debug!("azure_data_explorer_exporter.batch.timer_flush", signal = ?batch.signal);
            }
            exports.push(self.dispatch_batch(batch, header, generation));
        }
    }

    fn queue_shutdown_batch(
        &self,
        batch: Batch,
        effect_handler: &EffectHandler<OtapPdata>,
        resolutions: &Resolutions,
    ) {
        if batch.compressed.is_none() {
            self.record_preparation(batch.signal, batch.items, Outcome::Failure);
        }
        self.metrics
            .borrow_mut()
            .record_batch(batch.signal, Outcome::Failure);
        for pdata in batch.sources {
            self.queue_shutdown_source(pdata, effect_handler, resolutions);
        }
    }

    fn queue_shutdown_source(
        &self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
        resolutions: &Resolutions,
    ) {
        let handler = effect_handler.clone();
        resolutions.push(Box::pin(async move {
            handler
                .notify_nack(NackMsg::new_with_cause(
                    "shutdown before ADX input could be exported",
                    pdata,
                    NackCause::NodeShutdown,
                ))
                .await
        }));
    }

    fn reject_shutdown_input(
        &self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
        resolutions: &Resolutions,
    ) {
        let signal = pdata.signal_type();
        self.record_preparation(
            signal,
            pdata.payload_ref().known_item_count().unwrap_or(0) as u64,
            Outcome::Failure,
        );
        self.metrics
            .borrow_mut()
            .record_batch(signal, Outcome::Failure);
        self.queue_shutdown_source(pdata, effect_handler, resolutions);
    }

    async fn shutdown(
        &mut self,
        exports: &mut Vec<PendingExport>,
        resolutions: &mut Resolutions,
        batches: &mut VecDeque<Batch>,
        preparing: &mut Option<PreparingInput>,
        effect_handler: &EffectHandler<OtapPdata>,
        deadline: Instant,
    ) -> Result<(), EngineError> {
        let compression_budget = self.compression_budget.clone();
        let drain = async {
            loop {
                self.dispatch_ready(exports, batches, true);
                if !self.auth.is_ready() {
                    if let Some(input) = preparing.take() {
                        drop(input.future);
                        self.reject_shutdown_input(input.pdata, effect_handler, resolutions);
                    }
                    // Shutdown must not wait indefinitely for credentials. Preserve
                    // the original contexts and resolve them while time remains.
                    while let Some(batch) = batches.pop_front() {
                        self.queue_shutdown_batch(batch, effect_handler, resolutions);
                    }
                }
                if exports.is_empty()
                    && resolutions.is_empty()
                    && batches.is_empty()
                    && preparing.is_none()
                {
                    break;
                }
                tokio::task::consume_budget().await;
                tokio::select! {
                    () = poll_fn(|cx| compression_budget.poll_exhausted(cx)) => {
                        compression_budget.replenish().await;
                    }
                    extracted = poll_fn(|cx| preparing.as_mut().expect("preparing input").future.as_mut().poll(cx)),
                        if preparing.is_some() => {
                        let input = preparing.take().expect("completed preparation");
                        if let Some(batch) = self.prepare(input.pdata, extracted, effect_handler, resolutions) {
                            self.append_batch(batch, batches);
                        }
                    }
                    completed = Self::next_export(exports), if !exports.is_empty() => {
                        self.complete_export(completed.0, completed.1, effect_handler, resolutions, batches);
                    }
                    result = resolutions.next(), if !resolutions.is_empty() => {
                        result.expect("nonempty resolution set")?;
                    }
                }
            }
            Ok::<(), EngineError>(())
        };
        let expired = match tokio::time::timeout_at(deadline, drain).await {
            Ok(result) => {
                result?;
                false
            }
            Err(_) => true,
        };
        if expired {
            otel_warn!(
                "azure_data_explorer_exporter.shutdown.deadline_elapsed",
                pending_messages = Self::pending_messages(exports, resolutions, batches) as u64
            );
            for export in exports.drain(..) {
                self.metrics
                    .borrow_mut()
                    .record_batch(export.batch.signal, Outcome::Failure);
                drop(export);
            }
            if let Some(input) = preparing.take() {
                drop(input.future);
                self.reject_shutdown_input(input.pdata, effect_handler, resolutions);
            }
            resolutions.clear();
            for batch in batches.drain(..) {
                if batch.compressed.is_none() {
                    self.record_preparation(batch.signal, batch.items, Outcome::Failure);
                }
                self.metrics
                    .borrow_mut()
                    .record_batch(batch.signal, Outcome::Failure);
            }
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for AzureDataExplorerExporter {
    async fn start(
        mut self: Box<Self>,
        mut inbox: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        otel_info!(
            "azure_data_explorer_exporter.start",
            cluster_uri = self.config.cluster_uri.as_str(),
            db_name = self.config.db_name.as_str(),
            logs_table = self.config.tables.logs.name.as_str(),
            metrics_table = self.config.tables.metrics.name.as_str(),
            traces_table = self.config.tables.traces.name.as_str()
        );
        let telemetry = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;
        let mut exports = Vec::with_capacity(self.config.max_in_flight);
        let mut resolutions = Resolutions::new();
        let mut batches: VecDeque<Batch> = VecDeque::new();
        let mut draining = false;
        let mut parked = None;
        let mut preparing: Option<PreparingInput> = None;
        let compression_budget = self.compression_budget.clone();
        let pending_limit = if self.config.network_requests.coalesce {
            self.config.network_requests.max_pending_messages
        } else {
            self.config.max_in_flight
        };
        let margin_sleep = tokio::time::sleep_until(Instant::now());
        tokio::pin!(margin_sleep);
        let mut armed_margin = None;
        let deadline = loop {
            // Empty inputs and context-free completion routes can remain ready.
            tokio::task::consume_budget().await;
            let shutdown_deadline = inbox.shutdown_deadline();
            draining |= shutdown_deadline.is_some();
            let ready = self.auth.is_ready();
            if !ready && draining {
                while let Some(batch) = batches.pop_front() {
                    self.queue_shutdown_batch(batch, &effect_handler, &resolutions);
                }
            } else if !ready && !self.auth.is_active() {
                while let Some(batch) = batches.pop_front() {
                    if batch.compressed.is_none() {
                        self.record_preparation(batch.signal, batch.items, Outcome::Failure);
                    }
                    self.queue_batch_resolution(
                        batch.sources,
                        batch.signal,
                        Some(&Error::AuthenticationUnavailable),
                        &effect_handler,
                        &resolutions,
                    );
                }
            }
            if preparing.is_none()
                && Self::pending_messages(&exports, &resolutions, &batches) < pending_limit
                && let Some(pdata) = parked.take()
            {
                if draining && !ready {
                    self.reject_shutdown_input(pdata, &effect_handler, &resolutions);
                } else {
                    preparing = Some(self.start_preparation(pdata, &effect_handler));
                }
            }
            let pending = Self::pending_messages(&exports, &resolutions, &batches)
                + usize::from(preparing.is_some());
            self.dispatch_ready(
                &mut exports,
                &mut batches,
                draining || (preparing.is_none() && pending >= pending_limit),
            );
            let accepting = preparing.is_none()
                && pending < pending_limit
                && (ready || !self.auth.is_active() || draining);
            let flush_at = batches
                .iter()
                .filter(|batch| !batch.sealed)
                .map(|batch| batch.flush_at)
                .min();
            let margin = self.auth.refresh_deadline();
            if margin != armed_margin {
                if let Some(deadline) = margin {
                    margin_sleep.as_mut().reset(Instant::from_std(deadline));
                }
                armed_margin = margin;
            }
            tokio::select! {
                biased;
                // A parked source pauses inbox polling, not the latched shutdown deadline.
                () = tokio::time::sleep_until(
                    shutdown_deadline.map(Instant::from_std).unwrap_or_else(Instant::now)
                ), if shutdown_deadline.is_some() => {
                    break shutdown_deadline.expect("armed shutdown deadline");
                }
                () = &mut margin_sleep, if margin.is_some() => {}
                () = poll_fn(|cx| compression_budget.poll_exhausted(cx)) => {
                    compression_budget.replenish().await;
                }
                () = poll_fn(|cx| self.auth.poll_refresh(cx, &AUTH_EVENTS)), if self.auth.is_active() => {}
                () = tokio::time::sleep_until(flush_at.unwrap_or_else(Instant::now)),
                    if flush_at.is_some() && ready && exports.len() < self.config.max_in_flight => {}
                result = resolutions.next(), if !resolutions.is_empty() => {
                    result.expect("nonempty resolution set")?;
                }
                completed = Self::next_export(&mut exports), if !exports.is_empty() => {
                    self.complete_export(completed.0, completed.1, &effect_handler, &resolutions, &mut batches);
                }
                extracted = poll_fn(|cx| preparing.as_mut().expect("preparing input").future.as_mut().poll(cx)),
                    if preparing.is_some() => {
                    let input = preparing.take().expect("completed preparation");
                    if let Some(batch) = self.prepare(input.pdata, extracted, &effect_handler, &resolutions) {
                        self.append_batch(batch, &mut batches);
                    }
                }
                message = inbox.recv_when_or_shutdown_started(accepting),
                    if parked.is_none() => match message? {
                    None => {}
                    Some(Message::PData(pdata)) => {
                        if !accepting {
                            // Only shutdown draining can bypass recv_when(false).
                            // Park one source rather than moving upstream backlog
                            // into an unbounded exporter-owned queue.
                            if !draining {
                                otel_debug!("azure_data_explorer_exporter.shutdown.draining");
                            }
                            draining = true;
                            parked = Some(pdata);
                        } else if draining && !ready {
                            self.reject_shutdown_input(pdata, &effect_handler, &resolutions);
                        } else {
                            preparing = Some(self.start_preparation(pdata, &effect_handler));
                        }
                    }

                    Some(Message::Control(NodeControlMsg::CollectTelemetry { mut metrics_reporter })) => {
                        if let Err(error) = self.metrics.borrow_mut().report(&mut metrics_reporter) {
                            otel_warn!("azure_data_explorer_exporter.telemetry.failed", error = %error);
                        }
                    }
                    Some(Message::Control(NodeControlMsg::Shutdown { deadline, .. })) => {
                        break deadline;
                    }
                    Some(Message::Control(_)) => {}
                }
            }
        };
        if let Some(pdata) = parked.take() {
            self.reject_shutdown_input(pdata, &effect_handler, &resolutions);
        }
        self.shutdown(
            &mut exports,
            &mut resolutions,
            &mut batches,
            &mut preparing,
            &effect_handler,
            Instant::from_std(deadline),
        )
        .await?;
        match tokio::time::timeout_at(Instant::from_std(deadline), telemetry.cancel()).await {
            Ok(Err(error)) => {
                otel_warn!("azure_data_explorer_exporter.telemetry.cancel_failed", error = %error);
            }
            Err(_) => {
                otel_warn!("azure_data_explorer_exporter.telemetry.cancel_deadline");
            }
            Ok(Ok(())) => {}
        }
        otel_info!("azure_data_explorer_exporter.shutdown");
        Ok(TerminalState::new(
            deadline,
            self.metrics.borrow_mut().terminal_snapshots(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::capability::CapabilityError;
    use otel_arrow_dfe_engine::capability::auth::BearerToken;
    use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TokenStream;
    use otel_arrow_dfe_engine::config::ExporterConfig;
    use otel_arrow_dfe_engine::control::{
        Controllable, PipelineCompletionMsg, PipelineCompletionMsgReceiver,
        pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
    };
    use otel_arrow_dfe_engine::exporter::ExporterWrapper;
    use otel_arrow_dfe_engine::local::message::{LocalReceiver, LocalSender};
    use otel_arrow_dfe_engine::message::{Receiver, Sender};
    use otel_arrow_dfe_engine::node::NodeWithPDataReceiver;
    use otel_arrow_dfe_engine::shared::message::{SharedReceiver, SharedSender};
    use otel_arrow_dfe_engine::testing::{
        create_not_send_channel, test_node, test_pipeline_ctx_with_interests,
        test_pipeline_runtime_services,
    };
    use otel_arrow_dfe_otap::pdata::Context;
    use otel_arrow_dfe_pdata::encode::{
        encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch,
    };
    use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, any_value};
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, ResourceLogs, ScopeLogs,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
    use otel_arrow_dfe_pdata::views::otlp::bytes::{
        logs::RawLogsData, metrics::RawMetricsData, traces::RawTraceData,
    };
    use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use prost::Message as _;
    use std::io::Read;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context as TaskContext, Poll, Wake, Waker};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::{mpsc, oneshot};
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const INTERESTS: Interests = Interests::ACKS
        .union(Interests::NACKS)
        .union(Interests::NODE_INPUT_METRICS)
        .union(Interests::NODE_LOCAL_DURATION)
        .union(Interests::NODE_ITEM_COUNTS)
        .union(Interests::NODE_SIZE);

    struct TestProvider(RefCell<Option<TokenStream>>);

    #[async_trait(?Send)]
    impl BearerTokenProvider for TestProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            panic!("the exporter must subscribe instead of acquiring credentials");
        }

        fn token_stream(&self) -> TokenStream {
            self.0.borrow_mut().take().expect("one subscription")
        }
    }

    fn config(endpoint: &str) -> Config {
        serde_json::from_value(serde_json::json!({
            "cluster_uri": endpoint,
            "max_in_flight": 2,
            "max_retries": 0,
            "timeout": "60s",
            "tables": {
                "logs": { "mapping": "logs-mapping" },
                "metrics": { "mapping": "metrics-mapping" },
                "traces": { "mapping": "traces-mapping" }
            }
        }))
        .expect("test config")
    }

    fn exporter(config: Config) -> (AzureDataExplorerExporter, mpsc::Sender<BearerToken>) {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let (tx, rx) = mpsc::channel(8);
        let stream = futures::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|token| (token, rx))
        });
        let (ctx, _) = test_pipeline_ctx_with_interests(INTERESTS);
        let exporter = AzureDataExplorerExporter::new(
            ctx,
            config,
            Box::new(TestProvider(RefCell::new(Some(Box::pin(stream))))),
        )
        .expect("exporter");
        (exporter, tx)
    }

    struct Driver {
        control: Sender<NodeControlMsg<OtapPdata>>,
        data: Sender<OtapPdata>,
        completions: PipelineCompletionMsgReceiver<OtapPdata>,
    }

    impl Driver {
        async fn completion(&mut self) -> PipelineCompletionMsg<OtapPdata> {
            self.completions.recv().await.expect("completion")
        }

        async fn shutdown(&self) {
            self.control
                .send(NodeControlMsg::Shutdown {
                    deadline: std::time::Instant::now() + Duration::from_secs(30),
                    reason: "test complete".to_string(),
                })
                .await
                .expect("shutdown");
        }
    }

    fn runtime(
        exporter: AzureDataExplorerExporter,
        input_capacity: usize,
    ) -> (
        impl Future<Output = Result<TerminalState, EngineError>>,
        Driver,
    ) {
        let (data_tx, data_rx) = create_not_send_channel(input_capacity);
        runtime_with_input(
            exporter,
            Sender::Local(LocalSender::mpsc(data_tx)),
            Receiver::Local(LocalReceiver::mpsc(data_rx)),
            INTERESTS,
        )
    }

    fn runtime_with_input(
        exporter: AzureDataExplorerExporter,
        data: Sender<OtapPdata>,
        receiver: Receiver<OtapPdata>,
        interests: Interests,
    ) -> (
        impl Future<Output = Result<TerminalState, EngineError>>,
        Driver,
    ) {
        let node = test_node("adx-test");
        let mut wrapper = ExporterWrapper::local(
            exporter,
            node.clone(),
            Arc::new(NodeUserConfig::new_exporter_config(
                super::super::AZURE_DATA_EXPLORER_EXPORTER_URN,
            )),
            &ExporterConfig::new("adx-test"),
        );
        let control = wrapper.control_sender();
        wrapper
            .set_pdata_receiver(node, receiver)
            .expect("pdata receiver");
        let (runtime_tx, runtime_rx) = runtime_ctrl_msg_channel(8);
        let (completion_tx, completions) = pipeline_completion_msg_channel(8);
        let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(32);
        let work = async move {
            let _receivers = (runtime_rx, metrics_rx);
            wrapper
                .start(
                    runtime_tx,
                    completion_tx,
                    reporter,
                    interests,
                    test_pipeline_runtime_services(),
                )
                .await
        };
        (
            work,
            Driver {
                control,
                data,
                completions,
            },
        )
    }

    async fn run(
        exporter: AzureDataExplorerExporter,
        scenario: impl AsyncFnOnce(Driver),
    ) -> TerminalState {
        let (runtime, driver) = runtime(exporter, 8);
        let work = async { tokio::join!(runtime, scenario(driver)) };
        // Real loopback I/O must not race Tokio's paused-time auto-advance.
        let (result, ()) = tokio::time::timeout(Duration::from_secs(60), work)
            .await
            .expect("exporter scenario must finish");
        result.expect("exporter runtime")
    }

    fn pdata(bytes: OtlpProtoBytes) -> OtapPdata {
        OtapPdata::new(Context::default(), bytes.into()).test_subscribe_to(
            INTERESTS,
            Default::default(),
            1,
        )
    }

    fn logs(body: &str) -> OtapPdata {
        pdata(OtlpProtoBytes::ExportLogsRequest(Bytes::from(
            ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord {
                            body: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(body.to_string())),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }
            .encode_to_vec(),
        )))
    }

    fn syslog_batch(count: u32) -> OtapPdata {
        use otel_arrow_dfe_pdata_codec::{CodecRegistry, OtapPayload, PdataEncoding};
        // Nil optional fields keep 8192 records within the separate transformation work limit.
        let message = b"<34>1 - - - - - -";
        // Version-1 receiver framing: one receipt-time/year run.
        let mut bytes = b"SLG1".to_vec();
        bytes.extend(count.to_be_bytes());
        bytes.extend(1_735_787_045_000_000_000_i64.to_be_bytes());
        bytes.extend(count.to_be_bytes());
        bytes.extend(2025_i32.to_be_bytes());
        for _ in 0..count {
            bytes.extend(
                u32::try_from(message.len())
                    .expect("message length")
                    .to_be_bytes(),
            );
            bytes.extend(message);
        }
        let codec = CodecRegistry::global()
            .expect("registry")
            .resolve(&PdataEncoding::new("syslog-batch"))
            .expect("syslog codec");
        OtapPdata::new(
            Context::default(),
            OtapPayload::from_encoded(
                codec
                    .admit(SignalType::Logs, Bytes::from(bytes))
                    .expect("syslog source"),
            ),
        )
        .test_subscribe_to(INTERESTS, Default::default(), 1)
    }

    fn signal_pdata(signal: SignalType, arrow: bool) -> OtapPdata {
        let raw = match signal {
            SignalType::Logs => {
                return if arrow {
                    to_arrow(logs("hello"))
                } else {
                    logs("hello")
                };
            }
            SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(Bytes::from(
                ExportMetricsServiceRequest {
                    resource_metrics: vec![ResourceMetrics {
                        scope_metrics: vec![ScopeMetrics {
                            metrics: vec![Metric {
                                name: "temperature".to_string(),
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
                .encode_to_vec(),
            )),
            SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(Bytes::from(
                ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        scope_spans: vec![ScopeSpans {
                            spans: vec![Span {
                                start_time_unix_nano: 1,
                                end_time_unix_nano: 2,
                                trace_id: vec![1; 16],
                                span_id: vec![2; 8],
                                name: "test-span".to_string(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                }
                .encode_to_vec(),
            )),
        };
        let input = pdata(raw);
        if arrow { to_arrow(input) } else { input }
    }

    fn to_arrow(input: OtapPdata) -> OtapPdata {
        let (context, payload) = input.into_parts();
        let encoded = payload.encoded_view().expect("OTLP input");
        let records = match encoded.signal_type() {
            SignalType::Logs => {
                encode_logs_otap_batch(&RawLogsData::try_new(encoded.bytes()).expect("logs"))
                    .expect("OTAP logs")
            }
            SignalType::Metrics => encode_metrics_otap_batch(
                &RawMetricsData::try_new(encoded.bytes()).expect("metrics"),
            )
            .expect("OTAP metrics"),
            SignalType::Traces => {
                encode_spans_otap_batch(&RawTraceData::try_new(encoded.bytes()).expect("traces"))
                    .expect("OTAP traces")
            }
        };
        OtapPdata::new(context, records.into())
    }

    fn measurement(
        snapshots: &[MetricSetSnapshot],
        name: &str,
        signal: &str,
        outcome: &str,
        field: &str,
    ) -> u64 {
        snapshots
            .iter()
            .filter(|snapshot| {
                snapshot.descriptor().name == name
                    && snapshot.measurement_attribute_value("signal") == Some(signal)
                    && snapshot.measurement_attribute_value("outcome") == Some(outcome)
            })
            .filter_map(|snapshot| {
                snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .position(|metric| metric.name == field)
                    .map(|index| snapshot.get_metrics()[index].to_u64_lossy())
            })
            .sum()
    }

    /// Scenario: native OTAP and raw OTLP inputs contain one item of each supported signal.
    /// Guarantees: all six messages are ACKed after correctly routed, mapped, gzip-compressed HTTP requests.
    #[tokio::test]
    async fn exports_all_signals_and_representations() {
        let server = MockServer::start().await;
        for (table, mapping) in [
            ("OTELLogs", "logs-mapping"),
            ("OTELMetrics", "metrics-mapping"),
            ("OTELTraces", "traces-mapping"),
        ] {
            Mock::given(method("POST"))
                .and(path(format!("/v1/rest/ingest/oteldb/{table}")))
                .and(query_param("mappingName", mapping))
                .and(query_param("streamFormat", "JSON"))
                .and(header("content-encoding", "gzip"))
                .and(header("content-type", "application/json"))
                .and(header("authorization", "Bearer token-1"))
                .respond_with(ResponseTemplate::new(204))
                .expect(2)
                .mount(&server)
                .await;
        }
        let (exporter, tokens) = exporter(config(&server.uri()));
        tokens
            .send(BearerToken::without_expiry("token-1".to_string()))
            .await
            .expect("initial token");
        let terminal = run(exporter, async move |mut driver| {
            for arrow in [false, true] {
                for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
                    driver
                        .data
                        .send(signal_pdata(signal, arrow))
                        .await
                        .expect("pdata");
                    assert!(matches!(
                        driver.completion().await,
                        PipelineCompletionMsg::DeliverAck { .. }
                    ));
                }
            }
            driver.shutdown().await;
        })
        .await;
        for signal in ["logs", "metrics", "traces"] {
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    signal,
                    "success",
                    "messages"
                ),
                2
            );
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    signal,
                    "success",
                    "items"
                ),
                2
            );
        }
        for request in server.received_requests().await.expect("requests") {
            let mut json = String::new();
            let _ = GzDecoder::new(request.body.as_slice())
                .read_to_string(&mut json)
                .expect("gzip");
            let row: serde_json::Value = serde_json::from_str(&json).expect("JSON row");
            assert!(row.is_object());
            assert!(!json.ends_with('\n'));
        }
    }

    /// Scenario: ADX returns client refusals, authentication errors, timeouts, throttling, or server failures.
    /// Guarantees: every source gets one correctly classified NACK without service response content in its reason.
    #[tokio::test]
    async fn nacks_preserve_retry_disposition() {
        for status in [400, 401, 403, 408, 429, 500, 503] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status).set_body_string("private-response"))
                .expect(1)
                .mount(&server)
                .await;
            let (exporter, tokens) = exporter(config(&server.uri()));
            tokens
                .send(BearerToken::without_expiry("token-1".to_string()))
                .await
                .expect("token");
            let _ = run(exporter, async move |mut driver| {
                driver
                    .data
                    .send(logs("private-payload"))
                    .await
                    .expect("pdata");
                let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await else {
                    panic!("expected NACK for {status}");
                };
                assert_eq!(nack.permanent, matches!(status, 400 | 403));
                assert_eq!(
                    nack.cause,
                    if nack.permanent {
                        NackCause::Refused
                    } else {
                        NackCause::Unspecified
                    }
                );
                assert!(!nack.reason.contains("private-response"));
                assert!(!nack.reason.contains("private-payload"));
                driver.shutdown().await;
            })
            .await;
        }
    }

    /// Scenario: each raw OTLP signal has invalid top-level protobuf framing.
    /// Guarantees: malformed inputs receive permanent refused NACKs without any HTTP submission.
    #[tokio::test]
    async fn malformed_payloads_are_permanently_refused() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;
        let (exporter, tokens) = exporter(config(&server.uri()));
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let terminal = run(exporter, async move |mut driver| {
            for input in [
                OtlpProtoBytes::ExportLogsRequest(Bytes::from_static(b"\xff")),
                OtlpProtoBytes::ExportMetricsRequest(Bytes::from_static(b"\xff")),
                OtlpProtoBytes::ExportTracesRequest(Bytes::from_static(b"\xff")),
            ] {
                driver.data.send(pdata(input)).await.expect("pdata");
                let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await else {
                    panic!("NACK")
                };
                assert!(nack.permanent);
                assert_eq!(nack.cause, NackCause::Refused);
            }
            driver.shutdown().await;
        })
        .await;
        for signal in ["logs", "metrics", "traces"] {
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    signal,
                    "refused",
                    "messages"
                ),
                1
            );
        }
    }

    fn nested_message(path: &[u32], content: &[u8]) -> Vec<u8> {
        let mut bytes = content.to_vec();
        for &field in path.iter().rev() {
            let mut parent = Vec::new();
            prost::encoding::encode_key(
                field,
                prost::encoding::WireType::LengthDelimited,
                &mut parent,
            );
            prost::encoding::encode_varint(bytes.len() as u64, &mut parent);
            parent.extend_from_slice(&bytes);
            bytes = parent;
        }
        bytes
    }

    async fn assert_nested_payloads_refused(signal: SignalType, paths: &[&[u32]]) {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;
        let (exporter, tokens) = exporter(config(&server.uri()));
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let terminal = run(exporter, async move |mut driver| {
            for path in paths {
                for valid_prefix in [false, true] {
                    let mut bytes = if valid_prefix {
                        signal_pdata(signal, false)
                            .payload_ref()
                            .encoded_view()
                            .expect("OTLP source")
                            .bytes()
                            .to_vec()
                    } else {
                        Vec::new()
                    };
                    bytes.extend_from_slice(&nested_message(path, b"\x1a\x05\x00"));
                    driver
                        .data
                        .send(pdata(OtlpProtoBytes::new_from_bytes(signal, bytes)))
                        .await
                        .expect("malformed source");
                    let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await
                    else {
                        panic!("corrupt {signal:?} source must not be ACKed");
                    };
                    assert!(nack.permanent);
                    assert_eq!(nack.cause, NackCause::Refused);
                }
            }
            driver.shutdown().await;
        })
        .await;
        let signal_name = match signal {
            SignalType::Logs => "logs",
            SignalType::Metrics => "metrics",
            SignalType::Traces => "traces",
        };
        assert_eq!(
            measurement(
                terminal.metrics(),
                "exporter.attempted",
                signal_name,
                "refused",
                "messages",
            ),
            (paths.len() * 2) as u64,
        );
    }

    /// Scenario: nested logs framing is corrupt, with or without preceding valid records.
    /// Guarantees: the entire source is permanently refused without exporting a partial request.
    #[tokio::test]
    async fn nested_malformed_logs_are_permanently_refused() {
        assert_nested_payloads_refused(
            SignalType::Logs,
            &[
                &[1],
                &[1, 1],
                &[1, 2],
                &[1, 2, 1],
                &[1, 2, 2],
                &[1, 2, 2, 5],
                &[1, 2, 2, 6],
            ],
        )
        .await;
    }

    /// Scenario: nested metrics framing is corrupt, with or without preceding valid points.
    /// Guarantees: the entire source is permanently refused, including corrupt data types and exemplars.
    #[tokio::test]
    async fn nested_malformed_metrics_are_permanently_refused() {
        assert_nested_payloads_refused(
            SignalType::Metrics,
            &[
                &[1],
                &[1, 1],
                &[1, 2],
                &[1, 2, 1],
                &[1, 2, 2],
                &[1, 2, 2, 5],
                &[1, 2, 2, 5, 1],
                &[1, 2, 2, 5, 1, 5],
                &[1, 2, 2, 7],
                &[1, 2, 2, 9],
                &[1, 2, 2, 10],
                &[1, 2, 2, 10, 1, 8],
                &[1, 2, 2, 11],
                &[1, 2, 2, 11, 1, 6],
            ],
        )
        .await;
    }

    /// Scenario: nested traces framing is corrupt, with or without preceding valid spans.
    /// Guarantees: the entire source is permanently refused, including corrupt events, links, and status.
    #[tokio::test]
    async fn nested_malformed_traces_are_permanently_refused() {
        assert_nested_payloads_refused(
            SignalType::Traces,
            &[
                &[1],
                &[1, 1],
                &[1, 2],
                &[1, 2, 1],
                &[1, 2, 2],
                &[1, 2, 2, 9],
                &[1, 2, 2, 11],
                &[1, 2, 2, 13],
                &[1, 2, 2, 15],
            ],
        )
        .await;
    }

    /// Scenario: small protobuf sources contain a million empty records or data points.
    /// Guarantees: every decoder permanently refuses allocation amplification without sending a partial export.
    #[tokio::test]
    async fn protobuf_admission_rejects_all_signal_allocation_amplification() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;
        let (exporter, tokens) = exporter(config(&server.uri()));
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let terminal = run(exporter, async move |mut driver| {
            for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
                let bytes = if signal == SignalType::Metrics {
                    nested_message(&[1, 2, 2, 5], &[0x0a, 0].repeat(1_000_000))
                } else {
                    nested_message(&[1, 2], &[0x12, 0].repeat(1_000_000))
                };
                driver
                    .data
                    .send(pdata(OtlpProtoBytes::new_from_bytes(signal, bytes)))
                    .await
                    .expect("input");
                let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await else {
                    panic!("amplified source must be refused");
                };
                assert!(nack.permanent);
                assert_eq!(nack.cause, NackCause::Refused);
                assert!(
                    nack.reason
                        == format!(
                            "OTLP decoding allocation_bytes exceeds {MAX_DECODE_ALLOCATION}"
                        )
                        || nack.reason
                            == format!(
                                "OTLP decoding fields_or_elements exceeds {MAX_DECODE_WORK}"
                            ),
                    "admission must reject the source before transformation: {}",
                    nack.reason,
                );
            }
            driver.shutdown().await;
        })
        .await;
        for signal in ["logs", "metrics", "traces"] {
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    signal,
                    "refused",
                    "messages"
                ),
                1
            );
        }
    }

    /// Scenario: an 8192-message syslog source is prepared, then another preparation is canceled.
    /// Guarantees: real codec decoding yields to another local task and cancellation retains routable original bytes.
    #[tokio::test]
    async fn large_syslog_preparation_is_cooperative_and_preserves_delivery_ownership() {
        tokio::task::LocalSet::new()
            .run_until(async {
                let mut cfg = config("http://localhost:4319");
                cfg.network_requests.max_rows = 8192;
                let (mut exporter, tokens) = exporter(cfg);
                tokens
                    .send(BearerToken::without_expiry("token".to_string()))
                    .await
                    .expect("token");
                poll_fn(|cx| exporter.auth.poll_refresh(cx, &AUTH_EVENTS)).await;
                let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
                let mut handler = EffectHandler::new(
                    test_node("adx"),
                    reporter,
                    test_pipeline_runtime_services(),
                );
                let (tx, mut completions) = pipeline_completion_msg_channel(8);
                handler.set_pipeline_completion_msg_sender(tx);
                let mut input = exporter.start_preparation(syslog_batch(8192), &handler);
                assert!(futures::poll!(&mut input.future).is_pending());
                let progress = Rc::new(Cell::new(false));
                let observed = progress.clone();
                let probe = tokio::task::spawn_local(async move {
                    observed.set(true);
                });
                let extracted = tokio::time::timeout(Duration::from_secs(30), input.future)
                    .await
                    .expect("syslog preparation")
                    .expect("decoded syslog");
                assert!(
                    progress.get(),
                    "another task must run before preparation finishes"
                );
                probe.await.expect("progress task");
                assert_eq!(extracted.items, 8192);
                assert_eq!(extracted.records.len(), 8192);

                let mut canceled = exporter.start_preparation(syslog_batch(8192), &handler);
                let original = canceled
                    .pdata
                    .payload_ref()
                    .encoded_bytes()
                    .expect("encoded syslog")
                    .clone();
                assert!(futures::poll!(&mut canceled.future).is_pending());
                drop(canceled.future);
                let resolutions = Resolutions::new();
                exporter.reject_shutdown_input(canceled.pdata, &handler, &resolutions);
                let mut resolutions = resolutions;
                resolutions
                    .next()
                    .await
                    .expect("resolution")
                    .expect("shutdown route");
                let PipelineCompletionMsg::DeliverNack { nack } =
                    completions.recv().await.expect("completion")
                else {
                    panic!("canceled source must be NACKed");
                };
                assert_eq!(nack.cause, NackCause::NodeShutdown);
                assert_eq!(nack.refused.payload_ref().encoded_bytes(), Some(&original));
            })
            .await;
    }

    /// Scenario: the documented item batching settings split an ordinary 8192-log native source.
    /// Guarantees: default ADX row limits accept every split, reject the unsplit source, and preserve all source items.
    #[tokio::test]
    async fn documented_upstream_item_batching_matches_exporter_limits() {
        use otel_arrow_dfe_core_nodes::processors::batch_processor::Config as BatchConfig;
        use otel_arrow_dfe_pdata::otap::batching::make_item_batches;
        let batch_config: BatchConfig = serde_yaml::from_str(
            "format: otap\notap:\n  min_size: 1000\n  max_size: 1000\n  sizer: items\n",
        )
        .expect("documented batch configuration");
        let defaults: BatchConfig =
            serde_yaml::from_str("{}").expect("default batch configuration");
        assert_eq!(defaults.otap.min_size, Some(8192));
        assert!(defaults.otap.max_size.is_none());
        let source = to_arrow(pdata(OtlpProtoBytes::ExportLogsRequest(Bytes::from(
            ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord::default(); 8192],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }
            .encode_to_vec(),
        ))));
        let (exporter, _tokens) = exporter(config("http://localhost:4319"));
        let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
        let handler =
            EffectHandler::new(test_node("adx"), reporter, test_pipeline_runtime_services());
        assert!(matches!(
            exporter.extract_records(&source, &handler).await,
            Err(Error::TooManyRows {
                actual: 1001,
                limit: 1000
            })
        ));
        let view = handler
            .view(source.payload_ref(), &exporter.accepted_encodings)
            .await
            .expect("native source");
        let PdataView::Native(records) = view else {
            panic!("native OTAP required");
        };
        let batches = make_item_batches(
            SignalType::Logs,
            batch_config.otap.max_size,
            vec![records.into_owned()],
        )
        .expect("item splitting");
        assert_eq!(batches.len(), 9);
        let mut total = 0;
        for records in batches {
            let source = OtapPdata::new(Context::default(), records.into());
            let extracted = exporter
                .extract_records(&source, &handler)
                .await
                .expect("compatible source");
            assert!(extracted.records.len() <= exporter.config.network_requests.max_rows);
            assert_eq!(extracted.items as usize, extracted.records.len());
            total += extracted.items;
        }
        assert_eq!(total, 8192);
    }

    async fn assert_unframed_inputs_yield(bytes: Bytes, outcome: &'static str) {
        tokio::task::LocalSet::new()
            .run_until(async move {
                // A fresh task gives the first poll a fresh cooperative budget.
                tokio::task::spawn_local(async move {
                    const SOURCES: usize = 1024;
                    let (exporter, tokens) = exporter(config("http://localhost:4319"));
                    let compression_budget = exporter.compression_budget.clone();
                    tokens
                        .try_send(BearerToken::without_expiry("token".to_string()))
                        .expect("token");
                    let (tx, rx) = flume::bounded(SOURCES);
                    let remaining = rx.clone();
                    let (runtime, driver) = runtime_with_input(
                        exporter,
                        Sender::Shared(SharedSender::mpmc(tx)),
                        Receiver::Shared(SharedReceiver::mpmc(rx)),
                        Interests::empty(),
                    );
                    for _ in 0..SOURCES {
                        let input = OtapPdata::new(
                            Context::default(),
                            OtlpProtoBytes::ExportLogsRequest(bytes.clone()).into(),
                        );
                        driver.data.try_send(input).expect("buffered source");
                    }
                    tokio::pin!(runtime);
                    assert!(futures::poll!(&mut runtime).is_pending());
                    assert!(
                        remaining.len() < SOURCES,
                        "the exporter must process input before yielding"
                    );
                    assert!(
                        !remaining.is_empty(),
                        "the exporter must yield before draining ready input"
                    );
                    assert_eq!(compression_budget.used.get(), 0);
                    assert!(driver.completions.is_empty());
                    let probe = tokio::task::spawn_local(async move {
                        assert!(
                            !remaining.is_empty(),
                            "another local task must run while input remains ready"
                        );
                    });
                    probe.await.expect("progress task");
                    driver.shutdown().await;
                    drop(driver);
                    let terminal = tokio::time::timeout(Duration::from_secs(30), runtime)
                        .await
                        .expect("source completion and shutdown")
                        .expect("runtime");
                    assert_eq!(
                        measurement(
                            terminal.metrics(),
                            "exporter.attempted",
                            "logs",
                            outcome,
                            "messages"
                        ),
                        SOURCES as u64
                    );
                })
                .await
                .expect("unframed input scenario");
            })
            .await;
    }

    /// Scenario: shared input contains only empty OTLP sources with no completion context frames.
    /// Guarantees: the exporter yields before draining input, another local task runs, and every source succeeds.
    #[tokio::test(start_paused = true)]
    async fn unframed_empty_sources_yield_to_other_local_tasks() {
        assert_unframed_inputs_yield(Bytes::new(), "success").await;
    }

    /// Scenario: shared input contains only malformed nested OTLP sources with no completion context frames.
    /// Guarantees: the exporter yields before draining input, another local task runs, and every source is refused.
    #[tokio::test(start_paused = true)]
    async fn unframed_malformed_sources_yield_to_other_local_tasks() {
        assert_unframed_inputs_yield(Bytes::from_static(b"\x0a\x03\x1a\x05\x00"), "refused").await;
    }

    /// Scenario: several compressors run with buffered upstream data on a current-thread runtime.
    /// Guarantees: their combined work stops at the task quantum and another local task runs before input drains.
    #[tokio::test]
    async fn concurrent_compression_with_buffered_input_yields_the_exporter_task() {
        tokio::task::LocalSet::new()
            .run_until(async {
                let server = MockServer::start().await;
                Mock::given(method("POST"))
                    .respond_with(ResponseTemplate::new(204))
                    .expect(16)
                    .mount(&server)
                    .await;
                let mut cfg = config(&server.uri());
                cfg.max_in_flight = 8;
                let (exporter, tokens) = exporter(cfg);
                let budget = exporter.compression_budget.clone();
                tokens
                    .send(BearerToken::without_expiry("token".to_string()))
                    .await
                    .expect("token");
                let (tx, rx) = flume::bounded(32);
                let remaining = rx.clone();
                let (runtime, mut driver) = runtime_with_input(
                    exporter,
                    Sender::Shared(SharedSender::mpmc(tx)),
                    Receiver::Shared(SharedReceiver::mpmc(rx)),
                    INTERESTS,
                );
                for _ in 0..16 {
                    driver
                        .data
                        .try_send(logs(&"x".repeat(2 * COMPRESSION_TASK_WORK)))
                        .expect("buffered source");
                }
                tokio::pin!(runtime);
                assert!(futures::poll!(&mut runtime).is_pending());
                assert_eq!(budget.used.get(), COMPRESSION_TASK_WORK);
                assert!(
                    budget.started.get() >= 2,
                    "multiple streams must share the work quantum"
                );
                assert!(!remaining.is_empty());
                let probe = tokio::task::spawn_local(async move {
                    assert!(
                        !remaining.is_empty(),
                        "another task must run while input remains buffered"
                    );
                });
                tokio::time::timeout(Duration::from_secs(60), async {
                    let (terminal, ()) = tokio::join!(&mut runtime, async {
                        for _ in 0..16 {
                            assert!(matches!(
                                driver.completion().await,
                                PipelineCompletionMsg::DeliverAck { .. }
                            ));
                        }
                        driver.shutdown().await;
                    });
                    let _ = terminal.expect("runtime");
                    probe.await.expect("progress task");
                })
                .await
                .expect("compression and export completion");
            })
            .await;
    }

    /// Scenario: a decoded histogram point expands into several ADX rows after an unknown protobuf group.
    /// Guarantees: attempted items count source data points rather than expanded rows or the raw counter's zero.
    #[tokio::test]
    async fn attempted_metric_items_are_counted_before_histogram_expansion() {
        use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
            Histogram, HistogramDataPoint,
        };
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let (exporter, tokens) = exporter(config(&server.uri()));
        let mut bytes = b"\xa3\x06\xa4\x06".to_vec();
        bytes.extend(
            MetricsData {
                resource_metrics: vec![ResourceMetrics {
                    scope_metrics: vec![ScopeMetrics {
                        metrics: vec![Metric {
                            name: "latency".to_string(),
                            data: Some(metric::Data::Histogram(Histogram {
                                data_points: vec![HistogramDataPoint {
                                    count: 3,
                                    sum: Some(6.0),
                                    explicit_bounds: vec![5.0],
                                    bucket_counts: vec![1, 2],
                                    ..Default::default()
                                }],
                                aggregation_temporality: 2,
                            })),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }
            .encode_to_vec(),
        );
        let input = pdata(OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes)));
        let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
        let handler =
            EffectHandler::new(test_node("adx"), reporter, test_pipeline_runtime_services());
        let extracted = exporter
            .extract_records(&input, &handler)
            .await
            .expect("histogram rows");
        assert_eq!(extracted.items, 1);
        assert!(extracted.records.len() > 1);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let terminal = run(exporter, async move |mut driver| {
            driver.data.send(input).await.expect("histogram source");
            assert!(matches!(
                driver.completion().await,
                PipelineCompletionMsg::DeliverAck { .. }
            ));
            driver.shutdown().await;
        })
        .await;
        assert_eq!(
            measurement(
                terminal.metrics(),
                "exporter.attempted",
                "metrics",
                "success",
                "items"
            ),
            1
        );
    }

    /// Scenario: valid OTLP contains unknown groups at the root, resource, scope, and resource metadata.
    /// Guarantees: all signals retain their valid rows instead of being refused or silently truncated.
    #[tokio::test]
    async fn protobuf_unknown_groups_preserve_all_signal_rows() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(12)
            .mount(&server)
            .await;
        let (exporter, tokens) = exporter(config(&server.uri()));
        let (_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
        let handler = EffectHandler::<OtapPdata>::new(
            test_node("adx"),
            reporter,
            test_pipeline_runtime_services(),
        );
        let mut inputs = Vec::new();
        for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
            let source = signal_pdata(signal, false);
            let valid = source.payload_ref().encoded_view().expect("OTLP source");
            let expected = exporter
                .extract_records(&source, &handler)
                .await
                .expect("valid rows");
            assert_eq!(expected.items, 1);
            for path in [&[][..], &[1][..], &[1, 2][..], &[1, 1, 3][..]] {
                let mut bytes = nested_message(path, b"\xa3\x06\xa4\x06");
                bytes.extend_from_slice(valid.bytes());
                let input = pdata(OtlpProtoBytes::new_from_bytes(signal, bytes));
                assert_eq!(
                    exporter
                        .extract_records(&input, &handler)
                        .await
                        .expect("unknown protobuf groups must be accepted"),
                    expected,
                );
                inputs.push(input);
            }
        }
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let terminal = run(exporter, async move |mut driver| {
            for input in inputs {
                driver.data.send(input).await.expect("input");
                assert!(matches!(
                    driver.completion().await,
                    PipelineCompletionMsg::DeliverAck { .. }
                ));
            }
            driver.shutdown().await;
        })
        .await;
        for signal in ["logs", "metrics", "traces"] {
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    signal,
                    "success",
                    "items"
                ),
                4
            );
        }
    }

    /// Scenario: a log body repeats oneof alternatives in one message or across merged body messages.
    /// Guarantees: transformation uses protobuf's final oneof value rather than the first raw-view field.
    #[tokio::test]
    async fn protobuf_oneof_and_message_merge_use_the_final_value() {
        let (exporter, _tokens) = exporter(config("http://localhost:4319"));
        let (_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
        let handler = EffectHandler::<OtapPdata>::new(
            test_node("adx"),
            reporter,
            test_pipeline_runtime_services(),
        );
        for record in [
            &b"\x2a\x05\x0a\x01x\x18\x07"[..],
            &b"\x2a\x03\x0a\x01x\x2a\x02\x18\x07"[..],
        ] {
            let input = pdata(OtlpProtoBytes::ExportLogsRequest(Bytes::from(
                nested_message(&[1, 2, 2], record),
            )));
            let rows = exporter
                .extract_records(&input, &handler)
                .await
                .expect("valid repeated body")
                .records;
            assert_eq!(rows.len(), 1);
            let row: serde_json::Value = serde_json::from_slice(&rows[0]).expect("row");
            assert_eq!(row["Body"], serde_json::json!(7));
        }
    }

    /// Scenario: valid records precede corrupt known scalars, packed numbers, groups, or excessive nesting.
    /// Guarantees: decoding refuses the complete source instead of returning rows from its valid prefix.
    #[tokio::test]
    async fn protobuf_scalar_corruption_is_permanently_refused() {
        let (exporter, _tokens) = exporter(config("http://localhost:4319"));
        let (_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
        let handler = EffectHandler::<OtapPdata>::new(
            test_node("adx"),
            reporter,
            test_pipeline_runtime_services(),
        );
        let mut recursive_body = nested_message(&[1], b"leaf");
        for _ in 0..128 {
            recursive_body = nested_message(&[5, 1], &recursive_body);
        }
        for (signal, corrupt) in [
            (SignalType::Logs, nested_message(&[1, 2, 2, 5, 1], b"\xff")),
            (SignalType::Logs, nested_message(&[1, 2, 2], b"\x28\x00")),
            (
                SignalType::Metrics,
                nested_message(&[1, 2, 2, 9, 1, 6], &[0; 7]),
            ),
            (
                SignalType::Metrics,
                nested_message(&[1, 2, 2, 10, 1, 8, 2], b"\x80"),
            ),
            (SignalType::Traces, b"\xa3\x06".to_vec()),
            (SignalType::Traces, b"\xa3\x06\xac\x06".to_vec()),
            (
                SignalType::Logs,
                nested_message(&[1, 2, 2, 5], &recursive_body),
            ),
        ] {
            let source = signal_pdata(signal, false);
            let mut bytes = source
                .payload_ref()
                .encoded_view()
                .expect("OTLP source")
                .bytes()
                .to_vec();
            bytes.extend(corrupt);
            let input = pdata(OtlpProtoBytes::new_from_bytes(signal, bytes));
            let error = exporter
                .extract_records(&input, &handler)
                .await
                .expect_err("corrupt source must not yield partial rows");
            assert!(error.is_permanent_refusal(), "{error}");
        }
    }

    /// Scenario: valid empty protobuf messages arrive for logs, metrics, and traces.
    /// Guarantees: each input is ACKed as one successful no-op attempt without an HTTP request.
    #[tokio::test]
    async fn empty_signals_record_successful_noop_attempts() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;
        let (exporter, tokens) = exporter(config(&server.uri()));
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let terminal = run(exporter, async move |mut driver| {
            for input in [
                OtlpProtoBytes::ExportLogsRequest(Bytes::new()),
                OtlpProtoBytes::ExportMetricsRequest(Bytes::new()),
                OtlpProtoBytes::ExportTracesRequest(Bytes::new()),
            ] {
                driver.data.send(pdata(input)).await.expect("pdata");
                assert!(matches!(
                    driver.completion().await,
                    PipelineCompletionMsg::DeliverAck { .. }
                ));
            }
            driver.shutdown().await;
        })
        .await;
        for signal in ["logs", "metrics", "traces"] {
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    signal,
                    "success",
                    "messages"
                ),
                1
            );
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    signal,
                    "success",
                    "items"
                ),
                0
            );
        }
    }

    /// Scenario: ADX rejects the first token and the provider subsequently publishes a replacement.
    /// Guarantees: the rejected message is retryably NACKed, and the next message uses the replacement token.
    #[tokio::test]
    async fn unauthorized_invalidates_token_for_subsequent_messages() {
        let server = MockServer::start().await;
        Mock::given(header("authorization", "Bearer token-1"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(header("authorization", "Bearer token-2"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let (exporter, tokens) = exporter(config(&server.uri()));
        tokens
            .send(BearerToken::without_expiry("token-1".to_string()))
            .await
            .expect("token");
        let _ = run(exporter, async move |mut driver| {
            driver.data.send(logs("first")).await.expect("pdata");
            let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await else {
                panic!("NACK")
            };
            assert!(!nack.permanent);
            tokens
                .send(BearerToken::without_expiry("token-2".to_string()))
                .await
                .expect("refresh");
            driver.data.send(logs("second")).await.expect("pdata");
            assert!(matches!(
                driver.completion().await,
                PipelineCompletionMsg::DeliverAck { .. }
            ));
            driver.shutdown().await;
        })
        .await;
    }

    /// Scenario: the credential stream closes without ever supplying a token.
    /// Guarantees: subsequent input receives a retryable NACK rather than hanging or sending unauthenticated.
    #[tokio::test]
    async fn closed_token_stream_nacks_without_http() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;
        let (exporter, tokens) = exporter(config(&server.uri()));
        drop(tokens);
        let _ = run(exporter, async move |mut driver| {
            driver.data.send(logs("data")).await.expect("pdata");
            let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await else {
                panic!("NACK")
            };
            assert!(!nack.permanent);
            assert!(nack.reason.contains("unavailable"));
            driver.shutdown().await;
        })
        .await;
    }

    /// Scenario: JSON output crosses either the per-row or whole-request byte budget.
    /// Guarantees: the complete source is permanently refused before compression or network submission.
    #[tokio::test]
    async fn oversized_sources_are_refused_before_submission() {
        for row_limit in [true, false] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(204))
                .expect(0)
                .mount(&server)
                .await;
            let mut cfg = config(&server.uri());
            if row_limit {
                cfg.max_row_bytes = 16;
            } else {
                cfg.network_requests.max_bytes = 16;
            }
            let (exporter, tokens) = exporter(cfg);
            tokens
                .send(BearerToken::without_expiry("token".to_string()))
                .await
                .expect("token");
            let source = logs("input larger than limit");
            let (_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
            let handler: EffectHandler<OtapPdata> =
                EffectHandler::new(test_node("adx"), reporter, test_pipeline_runtime_services());
            let reason = exporter
                .extract_records(&source, &handler)
                .await
                .expect_err("source exceeds configured budget")
                .log_safe_summary();
            assert!(reason.contains("16"));
            let _ = run(exporter, async move |mut driver| {
                driver.data.send(source).await.expect("pdata");
                let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await else {
                    panic!("NACK")
                };
                assert!(nack.permanent);
                assert_eq!(nack.cause, NackCause::Refused);
                assert_eq!(nack.reason, reason);
                driver.shutdown().await;
            })
            .await;
        }
    }

    /// Scenario: one request occupies the only slot when shutdown reaches an immediate deadline.
    /// Guarantees: buffered input is not submitted after the deadline, and the admitted HTTP attempt is canceled and counted once.
    #[tokio::test]
    async fn shutdown_at_capacity_cancels_inflight_attempt() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let endpoint = format!("http://{}", listener.local_addr().expect("address"));
        let (started_tx, started_rx) = oneshot::channel();
        let (stop_tx, stop_rx) = oneshot::channel();
        let server = async {
            let (mut stream, _) = listener.accept().await.expect("connection");
            let mut bytes = [0; 4096];
            assert!(stream.read(&mut bytes).await.expect("request") > 0);
            started_tx.send(()).expect("started signal");
            stop_rx.await.expect("stop signal");
        };
        let mut cfg = config(&endpoint);
        cfg.max_in_flight = 1;
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let work = async {
            let terminal = run(exporter, async move |driver| {
                driver
                    .data
                    .send(logs("in flight"))
                    .await
                    .expect("first pdata");
                started_rx.await.expect("HTTP request started");
                driver
                    .data
                    .send(logs("buffered"))
                    .await
                    .expect("second pdata");
                driver
                    .control
                    .send(NodeControlMsg::Shutdown {
                        deadline: std::time::Instant::now(),
                        reason: "deadline".to_string(),
                    })
                    .await
                    .expect("shutdown");
            })
            .await;
            stop_tx.send(()).expect("stop server");
            terminal
        };
        let (terminal, ()) = tokio::join!(work, server);
        assert_eq!(
            measurement(
                terminal.metrics(),
                "exporter.attempted",
                "logs",
                "failure",
                "messages"
            ),
            1
        );
        assert_eq!(
            measurement(
                terminal.metrics(),
                "exporter.azure_data_explorer",
                "logs",
                "failure",
                "batches"
            ),
            1
        );
    }

    /// Scenario: upstream closes after shutdown is latched while a one-slot exporter still has a request in flight.
    /// Guarantees: disabled admission preserves the original grace deadline and the request can finish successfully.
    #[tokio::test]
    async fn shutdown_drains_inflight_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let endpoint = format!("http://{}", listener.local_addr().expect("address"));
        let (started_tx, started_rx) = oneshot::channel();
        let (respond_tx, respond_rx) = oneshot::channel();
        let server = async {
            let (mut stream, _) = listener.accept().await.expect("connection");
            let mut request = [0; 4096];
            assert!(stream.read(&mut request).await.expect("request") > 0);
            started_tx.send(()).expect("started");
            respond_rx.await.expect("response permission");
            stream
                .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
                .await
                .expect("response");
        };
        let mut cfg = config(&endpoint);
        cfg.max_in_flight = 1;
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        let work = async move {
            let (runtime, mut driver) = runtime(exporter, 8);
            tokio::pin!(runtime);
            driver.data.send(logs("data")).await.expect("pdata");
            tokio::select! {
                result = &mut runtime => panic!("exporter stopped before request: {}", result.is_ok()),
                result = started_rx => result.expect("request started"),
            }
            driver
                .control
                .send(NodeControlMsg::Shutdown {
                    deadline,
                    reason: "preserve grace period".to_string(),
                })
                .await
                .expect("shutdown");
            assert!(futures::poll!(&mut runtime).is_pending());
            drop(driver.data);
            assert!(futures::poll!(&mut runtime).is_pending());
            respond_tx.send(()).expect("release response");
            let (terminal, completion) = tokio::join!(runtime, driver.completions.recv());
            assert!(matches!(
                completion.expect("completion"),
                PipelineCompletionMsg::DeliverAck { .. }
            ));
            terminal.expect("runtime")
        };
        let (terminal, ()) = tokio::time::timeout(Duration::from_secs(60), async {
            tokio::join!(work, server)
        })
        .await
        .expect("shutdown and server completion");
        assert_eq!(terminal.deadline(), deadline);
        assert_eq!(
            measurement(
                terminal.metrics(),
                "exporter.attempted",
                "logs",
                "success",
                "messages"
            ),
            1
        );
    }

    /// Scenario: completion routing is backpressured by a full pipeline completion channel.
    /// Guarantees: shutdown cancels the blocked route at its deadline instead of waiting indefinitely.
    #[tokio::test(start_paused = true)]
    async fn shutdown_bounds_backpressured_completion_routing() {
        let (mut exporter, _tokens) = exporter(config("http://localhost:4319"));
        let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
        let mut handler =
            EffectHandler::new(test_node("adx"), reporter, test_pipeline_runtime_services());
        let (tx, _rx) = pipeline_completion_msg_channel(1);
        tx.send(PipelineCompletionMsg::DeliverAck {
            ack: AckMsg::new(logs("occupy route")),
        })
        .await
        .expect("fill");
        handler.set_pipeline_completion_msg_sender(tx);
        let mut resolutions = Resolutions::new();
        exporter.queue_resolution(
            logs("blocked"),
            SignalType::Logs,
            None,
            &handler,
            &resolutions,
        );
        assert!(matches!(futures::poll!(resolutions.next()), Poll::Pending));
        exporter
            .shutdown(
                &mut Vec::new(),
                &mut resolutions,
                &mut VecDeque::new(),
                &mut None,
                &handler,
                Instant::now() + Duration::from_secs(1),
            )
            .await
            .expect("bounded shutdown");
        assert!(resolutions.is_empty());
    }

    /// Scenario: multiple rows, including a row larger than the compression work quantum, are encoded.
    /// Guarantees: compression yields at the quantum and produces exact JSON Lines without a trailing separator.
    #[tokio::test]
    async fn compression_is_cooperative_and_preserves_framing() {
        let rows = vec![
            Bytes::from(vec![b'x'; COMPRESSION_YIELD_BYTES + 1]),
            Bytes::from_static(b"second"),
        ];
        let budget = CompressionBudget::default();
        let compression = AzureDataExplorerExporter::compress_batch(&rows, 6, &budget);
        tokio::pin!(compression);
        assert!(futures::poll!(&mut compression).is_pending());
        assert_eq!(budget.used.get(), COMPRESSION_YIELD_BYTES + 1024);
        let compressed = compression.await.expect("compression");
        let mut output = Vec::new();
        let _ = GzDecoder::new(compressed.as_ref())
            .read_to_end(&mut output)
            .expect("gzip");
        assert_eq!(output, [rows[0].as_ref(), b"\n", rows[1].as_ref()].concat());
    }

    /// Scenario: a one-slot exporter has no initial token and later has one stalled HTTP operation.
    /// Guarantees: explicit polls leave buffered data untouched at both auth and capacity gates while shutdown remains responsive.
    #[tokio::test]
    async fn admission_is_gated_by_initial_token_and_capacity() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let endpoint = format!("http://{}", listener.local_addr().expect("address"));
        let (started_tx, started_rx) = oneshot::channel();
        let (stop_tx, stop_rx) = oneshot::channel();
        let server = async {
            let (mut stream, _) = listener.accept().await.expect("connection");
            let mut bytes = [0; 4096];
            assert!(stream.read(&mut bytes).await.expect("request") > 0);
            started_tx.send(()).expect("request started");
            stop_rx.await.expect("stop");
        };
        let mut cfg = config(&endpoint);
        cfg.max_in_flight = 1;
        let (exporter, tokens) = exporter(cfg);
        let work = async {
            let (runtime, driver) = runtime(exporter, 1);
            tokio::pin!(runtime);
            driver.data.send(logs("first")).await.expect("input");
            assert!(futures::poll!(&mut runtime).is_pending());
            assert!(matches!(
                driver.data.try_send(logs("before token")),
                Err(otel_arrow_dfe_channel::error::SendError::Full(_))
            ));
            tokens
                .send(BearerToken::without_expiry("token".to_string()))
                .await
                .expect("token");
            tokio::select! {
                result = &mut runtime => panic!("exporter stopped before request: {}", result.is_ok()),
                result = started_rx => result.expect("request started"),
            }
            driver
                .data
                .send(logs("second"))
                .await
                .expect("buffered input");
            assert!(futures::poll!(&mut runtime).is_pending());
            assert!(matches!(
                driver.data.try_send(logs("at capacity")),
                Err(otel_arrow_dfe_channel::error::SendError::Full(_))
            ));
            driver
                .control
                .send(NodeControlMsg::Shutdown {
                    deadline: std::time::Instant::now(),
                    reason: "cancel stalled request".to_string(),
                })
                .await
                .expect("shutdown");
            let terminal = tokio::time::timeout(Duration::from_secs(30), runtime)
                .await
                .expect("responsive shutdown")
                .expect("shutdown result");
            stop_tx.send(()).expect("stop server");
            terminal
        };
        let (terminal, ()) = tokio::join!(work, server);
        assert_eq!(
            measurement(
                terminal.metrics(),
                "exporter.attempted",
                "logs",
                "failure",
                "messages"
            ),
            1
        );
    }

    /// Scenario: failed-payload debugging is disabled by default and then explicitly enabled.
    /// Guarantees: request state retains payload diagnostics only on opt-in, and the sample is bounded.
    #[tokio::test]
    async fn failed_payload_diagnostics_require_opt_in() {
        for enabled in [false, true] {
            let mut cfg = config("http://localhost:4319");
            cfg.log_failed_payload = enabled;
            let (mut exporter, tokens) = exporter(cfg);
            tokens
                .send(BearerToken::without_expiry("token".to_string()))
                .await
                .expect("token");
            poll_fn(|cx| exporter.auth.poll_refresh(cx, &AUTH_EVENTS)).await;
            let (_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
            let handler =
                EffectHandler::new(test_node("adx"), reporter, test_pipeline_runtime_services());
            let input = logs(&"x".repeat(4096));
            let extracted = exporter.extract_records(&input, &handler).await;
            let export = exporter
                .prepare(input, extracted, &handler, &Resolutions::new())
                .expect("prepared export");
            let (_, header, generation) = exporter.auth.header().expect("token");
            let export = exporter.dispatch_batch(export, header, generation);
            assert_eq!(export.batch.failed_payload.is_some(), enabled);
            assert_eq!(export.batch.sample_row.is_some(), enabled);
            if let Some(sample) = export.batch.sample_row {
                assert_eq!(sample.chars().count(), 2048);
            }
        }
    }

    /// Scenario: upstream closes during graceful shutdown with a partial batch far from its timer deadline.
    /// Guarantees: shutdown flushes that batch immediately and ACKs the source only after ADX accepts it.
    #[tokio::test]
    async fn shutdown_flushes_partial_coalesced_requests() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let mut cfg = config(&server.uri());
        cfg.network_requests.coalesce = true;
        cfg.network_requests.flush_interval = Duration::from_secs(3600);
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let _ = run(exporter, async move |driver| {
            let Driver {
                control,
                data,
                mut completions,
            } = driver;
            data.send(logs("partial")).await.expect("input");
            control
                .send(NodeControlMsg::Shutdown {
                    deadline: std::time::Instant::now() + Duration::from_secs(30),
                    reason: "flush partial batch".to_string(),
                })
                .await
                .expect("shutdown");
            drop(data);
            assert!(matches!(
                completions.recv().await.expect("completion"),
                PipelineCompletionMsg::DeliverAck { .. }
            ));
        })
        .await;
    }

    /// Scenario: shutdown is latched with a partial batch and an empty upstream channel whose sender stays open.
    /// Guarantees: the batch is exported and ACKed before upstream closes, and late input still drains successfully.
    #[tokio::test]
    async fn shutdown_flushes_partial_batch_while_upstream_remains_open() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(2)
            .mount(&server)
            .await;
        let mut cfg = config(&server.uri());
        cfg.network_requests.coalesce = true;
        cfg.network_requests.flush_interval = Duration::from_secs(7200);
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let (runtime, mut driver) = runtime(exporter, 1);
        tokio::pin!(runtime);
        driver.data.send(logs("partial")).await.expect("input");
        assert!(futures::poll!(&mut runtime).is_pending());
        driver
            .control
            .send(NodeControlMsg::Shutdown {
                deadline: std::time::Instant::now() + Duration::from_secs(3600),
                reason: "flush while upstream drains".to_string(),
            })
            .await
            .expect("shutdown");
        let scenario = async move {
            assert!(matches!(
                driver.completion().await,
                PipelineCompletionMsg::DeliverAck { .. }
            ));
            driver
                .data
                .send(logs("late input"))
                .await
                .expect("late input");
            assert!(matches!(
                driver.completion().await,
                PipelineCompletionMsg::DeliverAck { .. }
            ));
            drop(driver.data);
        };
        // The socket exchange uses real I/O; this timeout is only a failure guard.
        let (terminal, ()) = tokio::time::timeout(Duration::from_secs(60), async {
            tokio::join!(&mut runtime, scenario)
        })
        .await
        .expect("partial batch must flush while the upstream sender is open");
        assert_eq!(
            measurement(
                terminal.expect("shutdown result").metrics(),
                "exporter.attempted",
                "logs",
                "success",
                "messages",
            ),
            2,
        );
    }

    // Wakers require Send + Sync even though this test polls a core-local
    // exporter. This per-test counter observes registration, not timing.
    #[derive(Default)]
    struct WakeCounter(AtomicUsize);

    impl Wake for WakeCounter {
        fn wake(self: Arc<Self>) {
            self.wake_by_ref();
        }

        fn wake_by_ref(self: &Arc<Self>) {
            let _ = self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Scenario: shutdown is latched on empty shared-MPMC input before upstream sends its final source.
    /// Guarantees: late input wakes the pending exporter without auxiliary events and is ACKed before closure.
    #[tokio::test]
    async fn shutdown_shared_input_wakes_for_late_data() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let (exporter, tokens) = exporter(config(&server.uri()));
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let (data_tx, data_rx) = flume::bounded(1);
        let (runtime, mut driver) = runtime_with_input(
            exporter,
            Sender::Shared(SharedSender::mpmc(data_tx)),
            Receiver::Shared(SharedReceiver::mpmc(data_rx)),
            INTERESTS,
        );
        tokio::pin!(runtime);
        let counter = Arc::new(WakeCounter::default());
        let waker = Waker::from(counter.clone());
        let mut cx = TaskContext::from_waker(&waker);
        assert!(runtime.as_mut().poll(&mut cx).is_pending());
        driver.shutdown().await;
        assert!(runtime.as_mut().poll(&mut cx).is_pending());
        counter.0.store(0, Ordering::SeqCst);
        driver
            .data
            .try_send(logs("late shared input"))
            .expect("input");
        assert!(
            counter.0.load(Ordering::SeqCst) > 0,
            "late shared input must wake the exporter receive waiter",
        );
        let scenario = async move {
            assert!(matches!(
                driver.completion().await,
                PipelineCompletionMsg::DeliverAck { .. }
            ));
            drop(driver.data);
        };
        let (terminal, ()) = tokio::time::timeout(Duration::from_secs(60), async {
            tokio::join!(&mut runtime, scenario)
        })
        .await
        .expect("late shared input must export and drain");
        assert_eq!(
            measurement(
                terminal.expect("shutdown result").metrics(),
                "exporter.attempted",
                "logs",
                "success",
                "messages",
            ),
            1,
        );
    }

    /// Scenario: shutdown has admitted source contexts but no usable authentication token.
    /// Guarantees: pending sources are retryably NACKed with node_shutdown while the deadline permits.
    #[tokio::test(start_paused = true)]
    async fn shutdown_resolves_coalesced_sources_without_credentials() {
        let mut cfg = config("http://localhost:4319");
        cfg.network_requests.coalesce = true;
        let (mut exporter, _tokens) = exporter(cfg);
        let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
        let mut handler =
            EffectHandler::new(test_node("adx"), reporter, test_pipeline_runtime_services());
        let (tx, mut completions) = pipeline_completion_msg_channel(8);
        handler.set_pipeline_completion_msg_sender(tx);
        let mut batches = VecDeque::new();
        exporter.append_batch(batch(SignalType::Logs, b"{}"), &mut batches);
        exporter.append_batch(batch(SignalType::Logs, b"{}"), &mut batches);
        exporter
            .shutdown(
                &mut Vec::new(),
                &mut Resolutions::new(),
                &mut batches,
                &mut None,
                &handler,
                Instant::now() + Duration::from_secs(30),
            )
            .await
            .expect("shutdown");
        assert!(batches.is_empty());
        for _ in 0..2 {
            let PipelineCompletionMsg::DeliverNack { nack } =
                completions.recv().await.expect("completion")
            else {
                panic!("NACK");
            };
            assert!(!nack.permanent);
            assert_eq!(nack.cause, NackCause::NodeShutdown);
        }
    }

    /// Scenario: shutdown force-drains input while a two-source coalesced request is stalled.
    /// Guarantees: only one overflow source is parked, upstream remains backpressured, and partial batches flush without their timer.
    #[tokio::test]
    async fn forced_shutdown_drain_preserves_batches_with_one_parked_source() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let endpoint = format!("http://{}", listener.local_addr().expect("address"));
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let server = async {
            let mut started = Some(started_tx);
            let mut release = Some(release_rx);
            for index in 0..3 {
                let (mut stream, _) = listener.accept().await.expect("connection");
                let mut request = Vec::new();
                loop {
                    let mut chunk = [0; 4096];
                    let count = stream.read(&mut chunk).await.expect("request");
                    assert!(count > 0, "request must finish before connection closes");
                    request.extend_from_slice(&chunk[..count]);
                    if let Some(end) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                        let headers = std::str::from_utf8(&request[..end]).expect("headers");
                        let length: usize = headers
                            .lines()
                            .find_map(|line| {
                                let (name, value) = line.split_once(':')?;
                                name.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse().expect("content length"))
                            })
                            .expect("content length header");
                        if request.len() >= end + 4 + length {
                            break;
                        }
                    }
                }
                if index == 0 {
                    started
                        .take()
                        .expect("first request")
                        .send(())
                        .expect("started");
                    release
                        .take()
                        .expect("release receiver")
                        .await
                        .expect("release first response");
                }
                stream.write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await.expect("response");
                stream.shutdown().await.expect("close");
            }
        };
        let mut cfg = config(&endpoint);
        cfg.max_in_flight = 1;
        cfg.network_requests.coalesce = true;
        cfg.network_requests.max_pending_messages = 2;
        cfg.network_requests.flush_interval = Duration::from_secs(3600);
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let work = async {
            let (runtime, mut driver) = runtime(exporter, 1);
            tokio::pin!(runtime);
            driver
                .data
                .send(logs("first source"))
                .await
                .expect("source");
            assert!(futures::poll!(&mut runtime).is_pending());
            driver
                .data
                .send(logs("second source"))
                .await
                .expect("source");
            tokio::select! {
                result = &mut runtime => panic!("runtime stopped before request: {}", result.is_ok()),
                result = started_rx => result.expect("first request started"),
            }
            driver
                .data
                .send(logs("parked source"))
                .await
                .expect("buffered source");
            driver
                .control
                .send(NodeControlMsg::Shutdown {
                    deadline: std::time::Instant::now() + Duration::from_secs(30),
                    reason: "force buffered input".to_string(),
                })
                .await
                .expect("shutdown");
            assert!(futures::poll!(&mut runtime).is_pending());
            driver
                .data
                .try_send(logs("buffered fourth"))
                .expect("one freed channel slot");
            for _ in 0..4 {
                assert!(futures::poll!(&mut runtime).is_pending());
                assert!(matches!(
                    driver.data.try_send(logs("cannot refill")),
                    Err(otel_arrow_dfe_channel::error::SendError::Full(_))
                ));
            }
            assert!(futures::poll!(Box::pin(driver.completion())).is_pending());
            release_tx.send(()).expect("release stalled ADX");
            let Driver {
                data,
                control: _control,
                mut completions,
            } = driver;
            drop(data);
            let completions = async {
                for _ in 0..4 {
                    assert!(matches!(
                        completions.recv().await.expect("completion"),
                        PipelineCompletionMsg::DeliverAck { .. }
                    ));
                }
            };
            let (terminal, ()) = tokio::join!(&mut runtime, completions);
            terminal.expect("shutdown result")
        };
        let (terminal, ()) = tokio::time::timeout(Duration::from_secs(60), async {
            tokio::join!(work, server)
        })
        .await
        .expect("shutdown scenario");
        assert_eq!(
            measurement(
                terminal.metrics(),
                "exporter.attempted",
                "logs",
                "success",
                "items"
            ),
            4
        );
    }

    /// Scenario: shutdown parks an overflow source while an HTTP response never arrives.
    /// Guarantees: retention stays bounded and the actual shutdown deadline cancels the request without a response or further inbox polling.
    #[tokio::test]
    async fn forced_shutdown_drain_bounds_stalled_http_with_parked_source() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let endpoint = format!("http://{}", listener.local_addr().expect("address"));
        let (started_tx, started_rx) = oneshot::channel();
        let (stop_tx, stop_rx) = oneshot::channel();
        let server = async {
            let (mut stream, _) = listener.accept().await.expect("connection");
            let mut request = [0; 4096];
            assert!(stream.read(&mut request).await.expect("request") > 0);
            started_tx.send(()).expect("request started");
            stop_rx.await.expect("stop only after exporter exits");
        };
        let mut cfg = config(&endpoint);
        cfg.max_in_flight = 1;
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let work = async {
            let (runtime, driver) = runtime(exporter, 1);
            tokio::pin!(runtime);
            driver.data.send(logs("in flight")).await.expect("input");
            tokio::select! {
                result = &mut runtime => panic!("premature exit: {}", result.is_ok()),
                result = started_rx => result.expect("request started"),
            }
            // Pause only after real I/O is ready; no response is needed to finish shutdown.
            tokio::time::pause();
            let deadline = Instant::now() + Duration::from_secs(30);
            driver.data.send(logs("parked")).await.expect("overflow");
            driver
                .control
                .send(NodeControlMsg::Shutdown {
                    deadline: deadline.into_std(),
                    reason: "stalled request".to_string(),
                })
                .await
                .expect("shutdown");
            assert!(futures::poll!(&mut runtime).is_pending());
            driver
                .data
                .try_send(logs("upstream backlog"))
                .expect("freed input slot");
            assert!(futures::poll!(&mut runtime).is_pending());
            assert!(matches!(
                driver.data.try_send(logs("still bounded")),
                Err(otel_arrow_dfe_channel::error::SendError::Full(_))
            ));
            tokio::time::advance(Duration::from_secs(29)).await;
            assert!(futures::poll!(&mut runtime).is_pending());
            tokio::time::sleep_until(deadline).await;
            let Poll::Ready(terminal) = futures::poll!(&mut runtime) else {
                panic!("parked HTTP work must terminate at the shutdown deadline");
            };
            let terminal = terminal.expect("deadline shutdown");
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    "logs",
                    "failure",
                    "messages"
                ),
                2
            );
            tokio::time::resume();
            stop_tx.send(()).expect("stop server");
        };
        tokio::time::timeout(Duration::from_secs(60), async {
            tokio::join!(work, server)
        })
        .await
        .expect("bounded scenario");
    }

    /// Scenario: shutdown parks overflow behind a full completion channel and no credentials are available.
    /// Guarantees: neither blocked NACK routing nor parked pdata can delay shutdown past the latched deadline.
    #[tokio::test(start_paused = true)]
    async fn forced_shutdown_drain_bounds_blocked_completions_with_parked_source() {
        let mut cfg = config("http://localhost:4319");
        cfg.max_in_flight = 1;
        let (exporter, _tokens) = exporter(cfg);
        let (runtime, mut driver) = runtime(exporter, 1);
        tokio::pin!(runtime);
        let deadline = Instant::now() + Duration::from_secs(30);
        driver.data.send(logs("first")).await.expect("input");
        driver
            .control
            .send(NodeControlMsg::Shutdown {
                deadline: deadline.into_std(),
                reason: "blocked completions".to_string(),
            })
            .await
            .expect("shutdown");
        assert!(futures::poll!(&mut runtime).is_pending());
        for _ in 1..10 {
            driver
                .data
                .try_send(logs("next"))
                .expect("admission before overflow");
            assert!(futures::poll!(&mut runtime).is_pending());
        }
        driver
            .data
            .try_send(logs("upstream backlog"))
            .expect("last freed slot");
        assert!(futures::poll!(&mut runtime).is_pending());
        assert!(matches!(
            driver.data.try_send(logs("still bounded")),
            Err(otel_arrow_dfe_channel::error::SendError::Full(_))
        ));
        tokio::time::advance(Duration::from_secs(29)).await;
        assert!(futures::poll!(&mut runtime).is_pending());
        tokio::time::sleep_until(deadline).await;
        let Poll::Ready(terminal) = futures::poll!(&mut runtime) else {
            panic!("parked completion work must terminate at the shutdown deadline");
        };
        let terminal = terminal.expect("deadline shutdown");
        assert_eq!(
            measurement(
                terminal.metrics(),
                "exporter.attempted",
                "logs",
                "failure",
                "messages"
            ),
            10
        );
        for _ in 0..8 {
            let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await else {
                panic!("shutdown NACK");
            };
            assert_eq!(nack.cause, NackCause::NodeShutdown);
        }
    }

    /// Scenario: forced shutdown input arrives while a live token provider has not supplied a credential.
    /// Guarantees: shutdown draining does not wait for credentials, and original contexts receive node_shutdown NACKs.
    #[tokio::test]
    async fn forced_shutdown_drain_without_token_nacks_original_sources() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;
        let mut cfg = config(&server.uri());
        cfg.network_requests.coalesce = true;
        let (exporter, _tokens) = exporter(cfg);
        let _ = run(exporter, async move |mut driver| {
            driver
                .data
                .send(logs("cannot authenticate"))
                .await
                .expect("source");
            driver.shutdown().await;
            let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await else {
                panic!("NACK");
            };
            assert!(!nack.permanent);
            assert_eq!(nack.cause, NackCause::NodeShutdown);
        })
        .await;
    }

    fn batch(signal: SignalType, row: &'static [u8]) -> Batch {
        Batch {
            sources: vec![signal_pdata(signal, false)],
            records: vec![Bytes::from_static(row)],
            signal,
            items: 1,
            bytes: row.len() + 1,
            flush_at: Instant::now() + Duration::from_secs(10),
            sealed: false,
            compressed: None,
            auth_retries: 0,
            sample_row: None,
            failed_payload: None,
        }
    }

    /// Scenario: appending a complete source reaches or crosses a row, byte, or source-message boundary.
    /// Guarantees: exact fits merge, overflow seals the preceding batch, and no source is split.
    #[tokio::test]
    async fn coalescing_enforces_exact_request_boundaries() {
        for (max_rows, max_bytes, max_messages) in [(2, 1024, 8), (8, 6, 8), (8, 1024, 2)] {
            let mut cfg = config("http://localhost:4319");
            cfg.network_requests.coalesce = true;
            cfg.network_requests.max_rows = max_rows;
            cfg.network_requests.max_bytes = max_bytes;
            cfg.network_requests.max_pending_messages = max_messages;
            let (exporter, _tokens) = exporter(cfg);
            let mut batches = VecDeque::new();
            for _ in 0..2 {
                exporter.append_batch(batch(SignalType::Logs, b"{}"), &mut batches);
            }
            assert_eq!(batches.len(), 1);
            assert_eq!(batches[0].sources.len(), 2);
            assert_eq!(batches[0].records.len(), 2);
            assert_eq!(batches[0].bytes, 6);
            exporter.append_batch(batch(SignalType::Logs, b"{}"), &mut batches);
            assert_eq!(batches.len(), 2);
            assert!(batches[0].sealed);
            assert_eq!(batches[0].sources.len(), 2);
            assert_eq!(batches[1].sources.len(), 1);
        }
    }

    /// Scenario: a partial coalesced batch has not reached any size or source threshold.
    /// Guarantees: no HTTP operation starts before its virtual flush deadline, and the deadline flushes it.
    #[tokio::test(start_paused = true)]
    async fn partial_coalescing_flushes_at_its_deadline() {
        let mut cfg = config("http://localhost:4319");
        cfg.network_requests.coalesce = true;
        let (mut exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        poll_fn(|cx| exporter.auth.poll_refresh(cx, &AUTH_EVENTS)).await;
        let mut batches = VecDeque::from([batch(SignalType::Logs, b"{}")]);
        let mut exports = Vec::new();
        exporter.dispatch_ready(&mut exports, &mut batches, false);
        assert!(exports.is_empty());
        tokio::time::advance(Duration::from_secs(9)).await;
        exporter.dispatch_ready(&mut exports, &mut batches, false);
        assert!(exports.is_empty());
        tokio::time::advance(Duration::from_secs(1)).await;
        exporter.dispatch_ready(&mut exports, &mut batches, false);
        assert_eq!(exports.len(), 1);
        assert!(batches.is_empty());
    }

    /// Scenario: each signal supplies one raw and one native source with a two-message admission limit.
    /// Guarantees: compatible sources share one request, every source is ACKed, and telemetry counts source items not rows.
    #[tokio::test]
    async fn coalesces_all_signals_without_losing_source_acknowledgements() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(3)
            .mount(&server)
            .await;
        let mut cfg = config(&server.uri());
        cfg.network_requests.coalesce = true;
        cfg.network_requests.max_pending_messages = 2;
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let terminal = run(exporter, async move |mut driver| {
            for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
                driver
                    .data
                    .send(signal_pdata(signal, false))
                    .await
                    .expect("raw source");
                driver
                    .data
                    .send(signal_pdata(signal, true))
                    .await
                    .expect("native source");
                for _ in 0..2 {
                    assert!(matches!(
                        driver.completion().await,
                        PipelineCompletionMsg::DeliverAck { .. }
                    ));
                }
            }
            driver.shutdown().await;
        })
        .await;
        for signal in ["logs", "metrics", "traces"] {
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    signal,
                    "success",
                    "messages"
                ),
                1
            );
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.attempted",
                    signal,
                    "success",
                    "items"
                ),
                2
            );
            assert_eq!(
                measurement(
                    terminal.metrics(),
                    "exporter.azure_data_explorer",
                    signal,
                    "success",
                    "batches"
                ),
                1
            );
        }
        for request in server.received_requests().await.expect("requests") {
            let mut jsonl = String::new();
            let _ = GzDecoder::new(request.body.as_slice())
                .read_to_string(&mut jsonl)
                .expect("gzip");
            assert_eq!(jsonl.lines().count(), 2);
        }
    }

    /// Scenario: pending capacity is shared across different signal accumulators.
    /// Guarantees: a full global cap flushes partial batches without combining signals or deadlocking admission.
    #[tokio::test]
    async fn global_coalescing_limit_flushes_separate_signals() {
        let server = MockServer::start().await;
        for table in ["OTELLogs", "OTELMetrics", "OTELTraces"] {
            Mock::given(path(format!("/v1/rest/ingest/oteldb/{table}")))
                .respond_with(ResponseTemplate::new(204))
                .expect(1)
                .mount(&server)
                .await;
        }
        let mut cfg = config(&server.uri());
        cfg.network_requests.coalesce = true;
        cfg.network_requests.max_pending_messages = 3;
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let _ = run(exporter, async move |mut driver| {
            for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
                driver
                    .data
                    .send(signal_pdata(signal, false))
                    .await
                    .expect("source");
            }
            for _ in 0..3 {
                assert!(matches!(
                    driver.completion().await,
                    PipelineCompletionMsg::DeliverAck { .. }
                ));
            }
            driver.shutdown().await;
        })
        .await;
    }

    /// Scenario: ADX rejects a coalesced request, then a replacement credential is supplied.
    /// Guarantees: original sources remain unresolved until recovery, encoded bytes are reused, and retries are bounded.
    #[tokio::test]
    async fn retained_unauthorized_requests_reuse_payload_and_bound_recovery() {
        for (recover, late_refresh) in [(true, false), (true, true), (false, false)] {
            let server = MockServer::start().await;
            Mock::given(header("authorization", "Bearer token-1"))
                .respond_with(ResponseTemplate::new(401))
                .expect(1)
                .mount(&server)
                .await;
            Mock::given(header("authorization", "Bearer token-2"))
                .respond_with(ResponseTemplate::new(if recover { 204 } else { 401 }))
                .expect(1)
                .mount(&server)
                .await;
            let mut cfg = config(&server.uri());
            cfg.network_requests.coalesce = true;
            cfg.max_retries = 1;
            let (mut exporter, tokens) = exporter(cfg);
            tokens
                .send(BearerToken::without_expiry("token-1".to_string()))
                .await
                .expect("token");
            poll_fn(|cx| exporter.auth.poll_refresh(cx, &AUTH_EVENTS)).await;
            let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(8);
            let mut handler =
                EffectHandler::new(test_node("adx"), reporter, test_pipeline_runtime_services());
            let (tx, mut completions) = pipeline_completion_msg_channel(8);
            handler.set_pipeline_completion_msg_sender(tx);
            let mut batches = VecDeque::new();
            let resolutions = Resolutions::new();
            for _ in 0..2 {
                let input = logs("retained");
                let extracted = exporter.extract_records(&input, &handler).await;
                let source = exporter
                    .prepare(input, extracted, &handler, &resolutions)
                    .expect("source");
                exporter.append_batch(source, &mut batches);
            }
            let (_, header, generation) = exporter.auth.header().expect("token");
            let mut exports = vec![exporter.dispatch_batch(
                batches.pop_front().expect("batch"),
                header,
                generation,
            )];
            assert_eq!(
                AzureDataExplorerExporter::pending_messages(&exports, &resolutions, &batches),
                2
            );
            let (export, result) = AzureDataExplorerExporter::next_export(&mut exports).await;
            if late_refresh {
                tokens
                    .send(BearerToken::without_expiry("token-2".to_string()))
                    .await
                    .expect("early replacement");
                poll_fn(|cx| exporter.auth.poll_refresh(cx, &AUTH_EVENTS)).await;
            }
            exporter.complete_export(export, result, &handler, &resolutions, &mut batches);
            assert_eq!(batches.len(), 1);
            assert_eq!(batches[0].sources.len(), 2);
            assert_eq!(batches[0].auth_retries, 1);
            assert!(resolutions.is_empty());
            assert_eq!(
                AzureDataExplorerExporter::pending_messages(&exports, &resolutions, &batches),
                2
            );
            assert_eq!(exporter.auth.is_ready(), late_refresh);
            assert!(futures::poll!(Box::pin(completions.recv())).is_pending());
            let encoded = batches[0].compressed.clone().expect("retained payload");
            if !late_refresh {
                tokens
                    .send(BearerToken::without_expiry("token-2".to_string()))
                    .await
                    .expect("replacement");
                poll_fn(|cx| exporter.auth.poll_refresh(cx, &AUTH_EVENTS)).await;
            }
            exporter.dispatch_ready(&mut exports, &mut batches, true);
            let (export, result) = AzureDataExplorerExporter::next_export(&mut exports).await;
            exporter.complete_export(export, result, &handler, &resolutions, &mut batches);
            assert!(batches.is_empty());
            assert_eq!(
                AzureDataExplorerExporter::pending_messages(&exports, &resolutions, &batches),
                2
            );
            let mut resolutions = resolutions;
            while let Some(result) = resolutions.next().await {
                result.expect("route");
            }
            assert_eq!(
                AzureDataExplorerExporter::pending_messages(&exports, &resolutions, &batches),
                0
            );
            for _ in 0..2 {
                match completions.recv().await.expect("source completion") {
                    PipelineCompletionMsg::DeliverAck { .. } => assert!(recover),
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        assert!(!recover);
                        assert!(!nack.permanent);
                        assert!(nack.reason.contains("after 2 attempts"));
                    }
                }
            }
            let requests = server.received_requests().await.expect("requests");
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].body, requests[1].body);
            assert_eq!(requests[1].body.as_slice(), encoded.as_ref());
        }
    }

    /// Scenario: ADX returns terminal and transient errors for a two-source coalesced request.
    /// Guarantees: every original source receives the same safe refusal or retryable NACK without premature ACKs.
    #[tokio::test]
    async fn coalesced_errors_resolve_every_original_source() {
        for status in [400, 401, 429, 500] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status).set_body_string("private body"))
                .expect(1)
                .mount(&server)
                .await;
            let mut cfg = config(&server.uri());
            cfg.network_requests.coalesce = true;
            cfg.network_requests.max_pending_messages = 2;
            let (exporter, tokens) = exporter(cfg);
            tokens
                .send(BearerToken::without_expiry("token".to_string()))
                .await
                .expect("token");
            let _ = run(exporter, async move |mut driver| {
                for _ in 0..2 {
                    driver.data.send(logs("source")).await.expect("input");
                }
                for _ in 0..2 {
                    let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await
                    else {
                        panic!("original source must be NACKed");
                    };
                    assert_eq!(nack.permanent, status == 400);
                    if status == 400 {
                        assert_eq!(nack.cause, NackCause::Refused);
                    }
                    assert!(!nack.reason.contains("private body"));
                }
                driver.shutdown().await;
            })
            .await;
        }
    }

    /// Scenario: an oversized source follows a valid source waiting in a partial request.
    /// Guarantees: only the oversized source is refused; the preceding source is exported and ACKed unchanged.
    #[tokio::test]
    async fn oversized_source_does_not_poison_a_partial_coalesced_batch() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let mut cfg = config(&server.uri());
        cfg.network_requests.coalesce = true;
        cfg.network_requests.max_pending_messages = 2;
        cfg.max_row_bytes = 4096;
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let _ = run(exporter, async move |mut driver| {
            driver
                .data
                .send(logs("valid source"))
                .await
                .expect("valid input");
            driver
                .data
                .send(logs(&"x".repeat(8192)))
                .await
                .expect("oversized input");
            let mut acks = 0;
            let mut refusals = 0;
            for _ in 0..2 {
                match driver.completion().await {
                    PipelineCompletionMsg::DeliverAck { .. } => acks += 1,
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        assert!(nack.permanent);
                        assert_eq!(nack.cause, NackCause::Refused);
                        refusals += 1;
                    }
                }
            }
            assert_eq!((acks, refusals), (1, 1));
            driver.shutdown().await;
        })
        .await;
    }

    /// Scenario: the provider closes after a coalesced request is sent with a token ADX rejects.
    /// Guarantees: retained source contexts receive retryable NACKs without replaying a rejected credential or hanging.
    #[tokio::test]
    async fn closed_token_stream_resolves_retained_coalesced_sources() {
        let server = MockServer::start().await;
        let (sent, mut received) = mpsc::channel(1);
        Mock::given(method("POST"))
            .respond_with(move |_: &wiremock::Request| {
                sent.try_send(()).expect("one request");
                ResponseTemplate::new(401)
            })
            .expect(1)
            .mount(&server)
            .await;
        let mut cfg = config(&server.uri());
        cfg.network_requests.coalesce = true;
        cfg.network_requests.max_pending_messages = 2;
        cfg.max_retries = 1;
        let (exporter, tokens) = exporter(cfg);
        tokens
            .send(BearerToken::without_expiry("token".to_string()))
            .await
            .expect("token");
        let _ = run(exporter, async move |mut driver| {
            for _ in 0..2 {
                driver.data.send(logs("retained")).await.expect("input");
            }
            received.recv().await.expect("ADX received request");
            drop(tokens);
            for _ in 0..2 {
                let PipelineCompletionMsg::DeliverNack { nack } = driver.completion().await else {
                    panic!("NACK");
                };
                assert!(!nack.permanent);
                assert!(nack.reason.contains("unavailable"));
            }
            driver.shutdown().await;
        })
        .await;
    }
}
