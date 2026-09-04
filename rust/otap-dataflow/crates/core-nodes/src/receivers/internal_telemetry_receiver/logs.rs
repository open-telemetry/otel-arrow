// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal logs receiver state

use bytes::Bytes;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::error::Error;
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::Sizer;
use otel_arrow_dfe_pdata::otlp::ProtoBuffer;
use otel_arrow_dfe_pdata::otlp::common::EncodeFailure;
use otel_arrow_dfe_telemetry::event::{LogEvent, ObservedEvent};
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::self_tracing::{ScopeToBytesMap, encode_export_logs_request};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::mem::size_of;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::Instant;

pub const LOG_BATCH_MAX_BYTES: NonZeroUsize = NonZeroUsize::new(2 * 1024 * 1024).expect("non-zero");
const LOG_BATCH_DEFAULT_MIN_BYTES: NonZeroUsize = NonZeroUsize::new(64 * 1024).expect("non-zero");
const LOG_BATCH_DEFAULT_DURATION: Duration = Duration::from_millis(200);
const LOG_SCOPE_KEY_VALUE_ESTIMATE: usize = 32;

/// Configuration for internal telemetry system log batching.
/// This structure follows the batch_processor design.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LogsConfig {
    /// Sizing configuration reserved for OTAP log output.
    #[serde(default = "default_log_batch_otap")]
    pub otap: LogFormatConfig,

    /// Sizing configuration used for current OTLP log output.
    #[serde(default = "default_log_batch_otlp")]
    pub otlp: LogFormatConfig,

    /// Maximum time the oldest log can wait in a partial batch. If none,
    /// logs are flushed immediately.
    #[serde(default = "default_log_batch_duration", with = "humantime_serde")]
    pub max_batch_duration: Option<Duration>,
}

/// This structure follows the batch_processor design.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LogFormatConfig {
    /// Optional flush threshold measured by `sizer`.
    pub min_size: Option<NonZeroUsize>,

    /// Optional upper bound measured by `sizer`.
    pub max_size: Option<NonZeroUsize>,

    /// Sizing unit for this format.
    pub sizer: Sizer,
}

/// Accumulates log events and estimated size.
#[derive(Default)]
struct LogBatch {
    events: Vec<LogEvent>,
    current_estimate: usize,
}

const fn default_log_batch_otap() -> LogFormatConfig {
    LogFormatConfig {
        min_size: NonZeroUsize::new(8192),
        max_size: None,
        sizer: Sizer::Items,
    }
}

pub(super) const fn default_log_batch_otlp() -> LogFormatConfig {
    LogFormatConfig {
        min_size: Some(LOG_BATCH_DEFAULT_MIN_BYTES),
        max_size: Some(LOG_BATCH_MAX_BYTES),
        sizer: Sizer::Bytes,
    }
}

const fn default_log_batch_duration() -> Option<Duration> {
    Some(LOG_BATCH_DEFAULT_DURATION)
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            otap: default_log_batch_otap(),
            otlp: default_log_batch_otlp(),
            max_batch_duration: default_log_batch_duration(),
        }
    }
}

impl LogsConfig {
    fn otlp_max_size(self) -> usize {
        self.otlp
            .max_size
            .or(self.otlp.min_size)
            .expect("checked in validate")
            .get()
    }

    /// Validate the LogsConfig.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self
            .otlp
            .min_size
            .is_some_and(|size| size > LOG_BATCH_MAX_BYTES)
            || self
                .otlp
                .max_size
                .is_some_and(|size| size > LOG_BATCH_MAX_BYTES)
        {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver logs OTLP size limits must not exceed {LOG_BATCH_MAX_BYTES} bytes"
                ),
            });
        }
        let immediate_flush = self
            .max_batch_duration
            .is_none_or(|duration| duration.is_zero());
        self.otlp.validate(Sizer::Bytes, "OTLP", immediate_flush)?;
        Ok(())
    }
}

impl LogFormatConfig {
    fn lower_limit(self) -> usize {
        self.min_size
            .or(self.max_size)
            .expect("checked in validate")
            .get()
    }

    fn upper_limit(self) -> usize {
        self.max_size
            .or(self.min_size)
            .expect("checkedin validate")
            .get()
    }

    fn validate(
        self,
        expected_sizer: Sizer,
        format_name: &str,
        immediate_flush: bool,
    ) -> Result<(), ConfigError> {
        if self.min_size.or(self.max_size).is_none() {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver logs {format_name} max_size or min_size must be set"
                ),
            });
        }
        if self.sizer != expected_sizer {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver logs {format_name} sizer must be {}",
                    match expected_sizer {
                        Sizer::Requests => "requests",
                        Sizer::Items => "items",
                        Sizer::Bytes => "bytes",
                    }
                ),
            });
        }
        if let (Some(max_size), Some(min_size)) = (self.max_size, self.min_size)
            && max_size < min_size
        {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver logs {format_name} max_size ({max_size}) must be >= min_size ({min_size}) or unset"
                ),
            });
        }
        let max_size = self.upper_limit();
        if max_size > LOG_BATCH_MAX_BYTES.get() {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver logs {format_name} max_size {max_size} exceeds limit"
                ),
            });
        }
        if immediate_flush && self.min_size.is_some() {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver logs {format_name} min_size set requires max_batch_duration is set"
                ),
            });
        }
        if immediate_flush && self.max_size.is_none() {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver logs {format_name} max_batch_duration unset requires max_size is set"
                ),
            });
        }
        Ok(())
    }
}

impl LogBatch {
    fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Reports whether adding an event would cross the limit without splitting a single event.
    fn would_exceed(&self, event: &LogEvent, max_size: usize) -> bool {
        !self.is_empty()
            && self
                .current_estimate
                .saturating_add(estimate_log_bytes(event))
                > max_size
    }

    /// Reports whether size, age, or input closure makes the current batch ready.
    fn should_flush(&self, min_size: usize, deadline: Option<Instant>, channel_open: bool) -> bool {
        !self.is_empty()
            && (self.current_estimate >= min_size
                || deadline.is_none_or(|deadline| Instant::now() >= deadline)
                || !channel_open)
    }

    /// Adds an event and reports whether it started a new batch.
    fn push(&mut self, event: LogEvent) -> bool {
        let started_batch = self.is_empty();
        self.current_estimate = self
            .current_estimate
            .saturating_add(estimate_log_bytes(&event));
        self.events.push(event);
        started_batch
    }

    fn clear(&mut self) {
        self.events.clear();
        self.current_estimate = 0;
    }
}

/// Estimates encoded size of this record.
pub(super) fn estimate_log_bytes(event: &LogEvent) -> usize {
    size_of::<LogEvent>()
        .saturating_add(event.record.body_attrs_bytes.len())
        .saturating_add(event.record.context.len() * LOG_SCOPE_KEY_VALUE_ESTIMATE)
}

/// A downstream export across event-loop iterations.
type PendingExport = Pin<Box<dyn Future<Output = Result<(), Error>>>>;

/// Owns log batching, flush scheduling, and scope-encoding cache state.
pub(super) struct LogExportState {
    config: LogsConfig,
    enabled: bool,
    channel_open: bool,
    batch: LogBatch,
    batch_deadline: Option<Instant>,
    scope_cache: ScopeToBytesMap,
    pending_export: Option<PendingExport>,
}

impl LogExportState {
    pub(super) fn new(
        config: LogsConfig,
        enabled: bool,
        registry: TelemetryRegistryHandle,
    ) -> Self {
        Self {
            config,
            enabled,
            channel_open: enabled,
            batch: LogBatch::default(),
            batch_deadline: None,
            scope_cache: ScopeToBytesMap::new(registry),
            pending_export: None,
        }
    }

    /// Takes and encodes the batch when a configured flush condition has been met.
    fn take_ready_export(
        &mut self,
        resource_field_bytes: &Bytes,
    ) -> Result<Option<OtapPdata>, Error> {
        if !self.batch.should_flush(
            self.config.otlp.lower_limit(),
            self.batch_deadline,
            self.channel_open,
        ) {
            return Ok(None);
        }
        self.batch_deadline = None;
        self.take_batch(resource_field_bytes).map(Some)
    }

    /// Adds an event, first returning the previous batch if the size limit would be crossed.
    fn accept(
        &mut self,
        event: LogEvent,
        resource_field_bytes: &Bytes,
    ) -> Result<Option<OtapPdata>, Error> {
        let export = self
            .batch
            .would_exceed(&event, self.config.otlp_max_size())
            .then(|| self.take_batch(resource_field_bytes))
            .transpose()?;
        if self.batch.push(event)
            && let Some(duration) = self.config.max_batch_duration
        {
            // A new batch started with maximum-duration deadline.
            self.batch_deadline = Some(Instant::now() + duration);
        }
        Ok(export)
    }

    /// Removes and encodes the current batch while preserving reusable encoder state.
    fn take_batch(&mut self, resource_field_bytes: &Bytes) -> Result<OtapPdata, Error> {
        let mut inflight = std::mem::take(&mut self.batch);
        Self::encode_batch(
            &mut inflight.events,
            resource_field_bytes,
            &mut self.scope_cache,
        )
    }

    fn begin_send(
        effect_handler: &local::EffectHandler<OtapPdata>,
        pdata: OtapPdata,
    ) -> PendingExport {
        let effect_handler = effect_handler.clone();
        Box::pin(async move {
            effect_handler.send_message(pdata).await?;
            Ok(())
        })
    }

    /// Advances log input, batching, and downstream delivery by one event.
    pub(super) async fn run_once(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        internal: &otel_arrow_dfe_telemetry::InternalTelemetrySettings,
    ) -> Result<(), Error> {
        if let Some(export) = self.pending_export.as_mut() {
            let result = export.await;
            self.pending_export = None;
            return result;
        }

        if let Some(pdata) = self.take_ready_export(&internal.resource_field_bytes)? {
            self.pending_export = Some(Self::begin_send(effect_handler, pdata));
            return Ok(());
        }

        if !self.channel_open {
            std::future::pending().await
        }

        tokio::select! {
            result = internal.logs_receiver.recv_async() => {
                match result {
                    Ok(ObservedEvent::Log(log_event)) => {
                        if let Some(log_tap) = internal.log_tap.as_ref() {
                            log_tap.record(log_event.clone());
                        }
                        if let Some(pdata) =
                            self.accept(log_event, &internal.resource_field_bytes)?
                        {
                            self.pending_export = Some(Self::begin_send(effect_handler, pdata));
                        }
                    }
                    Ok(ObservedEvent::Engine(_)) => {}
                    Err(_) => {
                        self.channel_open = false;
                    }
                }
            }
            _ = tokio::time::sleep_until(
                self.batch_deadline.unwrap_or_else(Instant::now)
            ), if self.batch_deadline.is_some() => {}
        }
        Ok(())
    }

    /// Completes an in-flight send without retrying ambiguous channel ownership.
    pub(super) async fn complete_pending_until(
        &mut self,
        deadline: std::time::Instant,
    ) -> Result<(), Error> {
        let Some(export) = self.pending_export.take() else {
            return Ok(());
        };
        tokio::time::timeout_at(Instant::from_std(deadline), export)
            .await
            .map_err(|_| Error::InternalError {
                message: "timed out while completing an in-flight internal log export during terminal control".to_owned(),
            })?
    }

    /// Drains buffered and queued logs within the terminal-control deadline.
    pub(super) async fn flush_until(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        internal: &otel_arrow_dfe_telemetry::InternalTelemetrySettings,
        deadline: std::time::Instant,
    ) -> Result<(), Error> {
        if !self.enabled {
            return Ok(());
        }

        if !self.batch.is_empty() {
            self.send_batch_until(effect_handler, &internal.resource_field_bytes, deadline)
                .await?;
        }

        while let Ok(event) = internal.logs_receiver.try_recv() {
            Self::ensure_before_flush_deadline(deadline)?;
            if let ObservedEvent::Log(log_event) = event {
                if let Some(log_tap) = internal.log_tap.as_ref() {
                    log_tap.record(log_event.clone());
                }
                if self
                    .batch
                    .would_exceed(&log_event, self.config.otlp_max_size())
                {
                    self.send_batch_until(effect_handler, &internal.resource_field_bytes, deadline)
                        .await?;
                }
                let _ = self.batch.push(log_event);
                if self.batch.current_estimate >= self.config.otlp.lower_limit() {
                    self.send_batch_until(effect_handler, &internal.resource_field_bytes, deadline)
                        .await?;
                }
            }
        }
        if !self.batch.is_empty() {
            self.send_batch_until(effect_handler, &internal.resource_field_bytes, deadline)
                .await?;
        }
        Ok(())
    }

    async fn send_batch_until(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        resource_field_bytes: &Bytes,
        deadline: std::time::Instant,
    ) -> Result<(), Error> {
        Self::ensure_before_flush_deadline(deadline)?;
        tokio::time::timeout_at(
            Instant::from_std(deadline),
            self.send_batch(effect_handler, resource_field_bytes),
        )
        .await
        .map_err(|_| Error::InternalError {
            message: "timed out while flushing internal logs during shutdown; remaining terminal telemetry was not flushed".to_owned(),
        })?
    }

    fn ensure_before_flush_deadline(deadline: std::time::Instant) -> Result<(), Error> {
        if std::time::Instant::now() >= deadline {
            return Err(Error::InternalError {
                message: "timed out while flushing internal logs during shutdown; remaining terminal telemetry was not flushed".to_owned(),
            });
        }
        Ok(())
    }

    /// Encodes the batch, first sorts the events in place by scope.
    fn encode_batch(
        events: &mut [LogEvent],
        resource_field_bytes: &Bytes,
        scope_cache: &mut ScopeToBytesMap,
    ) -> Result<OtapPdata, Error> {
        let capacity = events.iter().fold(
            resource_field_bytes.len().saturating_add(512),
            |capacity, event| capacity.saturating_add(estimate_log_bytes(event)),
        );
        // Note: no limit is passed here, only capacity to start with
        // a correct allocation.
        let mut buf = ProtoBuffer::with_capacity(capacity);
        encode_export_logs_request(&mut buf, events, resource_field_bytes, scope_cache).map_err(
            |failure| Error::PdataConversionError {
                error: match failure {
                    EncodeFailure::Dropped => {
                        "internal log batch exceeded the encoder limit".to_owned()
                    }
                },
            },
        )?;
        Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(buf.into_bytes()).into(),
        ))
    }

    async fn send_batch(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        resource_field_bytes: &Bytes,
    ) -> Result<(), Error> {
        if self.batch.is_empty() {
            return Ok(());
        }
        let pdata = Self::encode_batch(
            &mut self.batch.events,
            resource_field_bytes,
            &mut self.scope_cache,
        )?;
        effect_handler.send_message(pdata).await?;
        self.batch.clear();
        Ok(())
    }
}
