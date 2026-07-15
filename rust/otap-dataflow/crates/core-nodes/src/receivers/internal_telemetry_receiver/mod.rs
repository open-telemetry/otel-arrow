// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry receiver.
//!
//! This receiver consumes internal logs from the logging channel and emits
//! the logs as OTLP ExportLogsRequest messages into the pipeline.

use async_trait::async_trait;
use bytes::Bytes;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_telemetry::event::{LogEvent, ObservedEvent};
use otap_df_telemetry::log_tap::InternalLogTapHandle;
use otap_df_telemetry::metrics::MetricSetSnapshot;
use otap_df_telemetry::self_tracing::{ScopeToBytesMap, encode_export_logs_request_batch};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

/// The URN for the internal telemetry receiver.
pub use otap_df_telemetry::INTERNAL_TELEMETRY_RECEIVER_URN;

/// Default upper bound on how long a record waits in a partial batch before
/// being flushed. Mirrors the OTAP batch processor default.
const DEFAULT_MAX_BATCH_DURATION_MS: u64 = 200;

fn default_max_batch_duration() -> Duration {
    Duration::from_millis(DEFAULT_MAX_BATCH_DURATION_MS)
}

/// Rough per-record protobuf overhead added to the record body when estimating
/// a message's size (timestamp, severity, event name, length prefixes).
const RECORD_FRAMING_BYTES: usize = 64;

/// Estimated on-wire size of `event`: its pre-encoded body plus
/// [`RECORD_FRAMING_BYTES`]. Shared by the running batch-byte counter and the
/// flush/split logic so both use the same estimate.
fn record_bytes(event: &LogEvent) -> usize {
    event.record.body_attrs_bytes.len() + RECORD_FRAMING_BYTES
}

/// Default `min_size` (64 KiB), so the receiver batches without configuration.
const fn default_min_size() -> NonZeroUsize {
    NonZeroUsize::new(64 * 1024).expect("64 KiB is nonzero")
}

/// Default `max_size` (2 MiB).
const fn default_max_size() -> NonZeroUsize {
    NonZeroUsize::new(1 << 21).expect("2 MiB is nonzero")
}

/// Configuration for the internal telemetry receiver.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Bytes to accumulate (each record's estimated size, see
    /// [`record_bytes`]) before flushing the current batch, as a minimum.
    /// Records sharing a scope are grouped into one `ExportLogsRequest`.
    /// Default 64 KiB. A configured `0` is rejected.
    #[serde(default = "default_min_size")]
    min_size: NonZeroUsize,

    /// Upper bound on a single flushed message's estimated size; a batch that
    /// would exceed it is split instead (see [`chunk_end`]). The estimate omits
    /// some protobuf overhead, so it is a safety margin, not a hard guarantee.
    /// Default 2 MiB. Must be at least `min_size`.
    #[serde(default = "default_max_size")]
    max_size: NonZeroUsize,

    /// How long a record may wait in a partial batch before being flushed.
    /// Default 200ms.
    #[serde(with = "humantime_serde", default = "default_max_batch_duration")]
    max_batch_duration: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_size: default_min_size(),
            max_size: default_max_size(),
            max_batch_duration: default_max_batch_duration(),
        }
    }
}

impl Config {
    /// Rejects a `max_size` smaller than `min_size`, which would force the
    /// receiver to split every batch it just flushed.
    fn validate(&self) -> Result<(), ConfigError> {
        if self.max_size < self.min_size {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "max_size ({}) must be >= min_size ({})",
                    self.max_size, self.min_size
                ),
            });
        }
        Ok(())
    }
}

/// A receiver that consumes internal logs from the logging channel and emits OTLP logs.
pub struct InternalTelemetryReceiver {
    config: Config,
    /// Internal telemetry settings obtained from the pipeline context during construction.
    /// Contains the logs receiver channel, pre-encoded resource bytes, and registry handle.
    internal_telemetry: otap_df_telemetry::InternalTelemetrySettings,
}

/// Declares the internal telemetry receiver as a local receiver factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static INTERNAL_TELEMETRY_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: INTERNAL_TELEMETRY_RECEIVER_URN,
    create: |mut pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        // Get internal telemetry settings from the pipeline context
        let internal_telemetry = pipeline.take_internal_telemetry().ok_or_else(|| {
            ConfigError::InvalidUserConfig {
                error: "InternalTelemetryReceiver requires internal telemetry settings in pipeline context".to_owned(),
            }
        })?;

        Ok(ReceiverWrapper::local(
            InternalTelemetryReceiver::new_with_telemetry(
                InternalTelemetryReceiver::parse_config(&node_config.config)?,
                internal_telemetry,
            ),
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    // Full parse (deserialize + cross-field checks) so an invalid `max_size <
    // min_size` is rejected at config-validation time, not only at start.
    validate_config: |config| InternalTelemetryReceiver::parse_config(config).map(|_| ()),
};

impl InternalTelemetryReceiver {
    /// Create a new receiver with the given configuration and internal telemetry settings.
    #[must_use]
    pub const fn new_with_telemetry(
        config: Config,
        internal_telemetry: otap_df_telemetry::InternalTelemetrySettings,
    ) -> Self {
        Self {
            config,
            internal_telemetry,
        }
    }

    /// Parse configuration from a JSON value.
    pub fn parse_config(config: &Value) -> Result<Config, ConfigError> {
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        config.validate()?;
        Ok(config)
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for InternalTelemetryReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let internal = self.internal_telemetry.clone();
        let logs_receiver = internal.logs_receiver;
        let resource_bytes = internal.resource_bytes;
        let log_tap = internal.log_tap;
        let mut scope_cache = ScopeToBytesMap::new(internal.registry);
        let flush_threshold_bytes = self.config.min_size.get();
        let max_message_bytes = self.config.max_size.get();
        let flush_period = self.config.max_batch_duration;
        let mut batch: Vec<LogEvent> = Vec::new();
        // Running total of `record_bytes` for everything currently in `batch`;
        // reset alongside every flush. Compared against `flush_threshold_bytes`
        // to decide when a batch is full.
        let mut batch_bytes: usize = 0;
        // Set when the first record lands in an empty batch, cleared on every
        // flush. The timer below only sleeps while a deadline is pending, so an
        // idle receiver never wakes up.
        let mut batch_deadline: Option<tokio::time::Instant> = None;

        loop {
            // Enforce `max_batch_duration` regardless of select priority. Under a
            // steady sub-threshold stream the biased select keeps choosing the log
            // arm, so the timer branch alone cannot guarantee the deadline; this
            // top-of-loop check makes it a hard upper bound.
            if batch_deadline.is_some_and(|d| tokio::time::Instant::now() >= d) {
                Self::flush_batch(
                    &effect_handler,
                    &mut batch,
                    &resource_bytes,
                    &mut scope_cache,
                    max_message_bytes,
                )
                .await?;
                batch_deadline = None;
                batch_bytes = 0;
            }

            tokio::select! {
                biased;

                // Handle control messages with priority
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            // Chunk by `flush_threshold_bytes` so a large backlog
                            // drains as several bounded messages, not one oversized
                            // payload.
                            while let Ok(event) = logs_receiver.try_recv() {
                                if let ObservedEvent::Log(log_event) = event {
                                    batch_bytes += Self::buffer_log(&log_tap, &mut batch, log_event);
                                    if batch_bytes >= flush_threshold_bytes {
                                        Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache, max_message_bytes).await?;
                                        batch_bytes = 0;
                                    }
                                }
                            }
                            Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache, max_message_bytes).await?;
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            while let Ok(event) = logs_receiver.try_recv() {
                                if let ObservedEvent::Log(log_event) = event {
                                    batch_bytes += Self::buffer_log(&log_tap, &mut batch, log_event);
                                    if batch_bytes >= flush_threshold_bytes {
                                        Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache, max_message_bytes).await?;
                                        batch_bytes = 0;
                                    }
                                }
                            }
                            Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache, max_message_bytes).await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::CollectTelemetry { .. }) => {
                            // No metrics to report for now
                        }
                        Err(e) => {
                            // Best-effort flush so a control-channel failure does
                            // not drop records already buffered for batching.
                            let _ = Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache, max_message_bytes).await;
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                             // Ignore other control messages
                        }
                    }
                }

                // Receive logs from the channel
                result = logs_receiver.recv_async() => {
                    match result {
                        Ok(ObservedEvent::Log(log_event)) => {
                            if batch.is_empty() {
                                batch_deadline = Some(tokio::time::Instant::now() + flush_period);
                            }
                            batch_bytes += Self::buffer_log(&log_tap, &mut batch, log_event);
                            if batch_bytes >= flush_threshold_bytes {
                                Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache, max_message_bytes).await?;
                                batch_deadline = None;
                                batch_bytes = 0;
                            }
                        }
                        Ok(ObservedEvent::Engine(_)) => {
                            // Engine events are not yet processed
                        }
                        Err(_) => {
                            // Channel closed: flush anything pending, then exit.
                            Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache, max_message_bytes).await?;
                            return Ok(TerminalState::default());
                        }
                    }
                }

                // Wake the loop when the deadline arrives so the top-of-loop check
                // flushes (covers the idle case with no incoming logs). Disabled
                // while the batch is empty, so an idle receiver never wakes.
                _ = tokio::time::sleep_until(batch_deadline.unwrap_or_else(tokio::time::Instant::now)),
                    if batch_deadline.is_some() => {}
            }
        }
    }
}

impl InternalTelemetryReceiver {
    /// Tap a log event for admin consumers, then buffer it for the next flush.
    /// Returns the event's estimated size in bytes (see [`record_bytes`]) so
    /// the caller can track the running batch-byte total.
    fn buffer_log(
        log_tap: &Option<InternalLogTapHandle>,
        batch: &mut Vec<LogEvent>,
        log_event: LogEvent,
    ) -> usize {
        if let Some(tap) = log_tap.as_ref() {
            tap.record(log_event.clone());
        }
        let size = record_bytes(&log_event);
        batch.push(log_event);
        size
    }

    /// Encode and send the accumulated batch, grouping records by scope. A batch
    /// that would exceed `max_size` is split (see [`chunk_end`]) so every
    /// message stays within that budget. Clears the batch; a no-op when empty.
    async fn flush_batch(
        effect_handler: &local::EffectHandler<OtapPdata>,
        batch: &mut Vec<LogEvent>,
        resource_bytes: &Bytes,
        scope_cache: &mut ScopeToBytesMap,
        max_size: usize,
    ) -> Result<(), Error> {
        if batch.is_empty() {
            return Ok(());
        }
        let base = resource_bytes.len();
        let total = base + batch.iter().map(record_bytes).sum::<usize>();

        // Common case: the whole batch fits one message, so encode it directly
        // with no per-record size vector or chunk loop. `min_size` defaults
        // well under `max_size`, so most flushes land here.
        if total <= max_size {
            Self::send_chunk(effect_handler, batch, total, resource_bytes, scope_cache).await?;
            batch.clear();
            return Ok(());
        }

        // Oversized batch: split into byte-bounded messages.
        let sizes: Vec<usize> = batch.iter().map(record_bytes).collect();
        for (start, end) in chunk_ranges(&sizes, base, max_size) {
            let capacity = base + sizes[start..end].iter().sum::<usize>();
            Self::send_chunk(
                effect_handler,
                &batch[start..end],
                capacity,
                resource_bytes,
                scope_cache,
            )
            .await?;
        }
        batch.clear();
        Ok(())
    }

    /// Encode one `ExportLogsRequest` from `events` and send it downstream.
    async fn send_chunk(
        effect_handler: &local::EffectHandler<OtapPdata>,
        events: &[LogEvent],
        capacity: usize,
        resource_bytes: &Bytes,
        scope_cache: &mut ScopeToBytesMap,
    ) -> Result<(), Error> {
        let mut buf = ProtoBuffer::with_capacity(capacity);
        encode_export_logs_request_batch(&mut buf, events, resource_bytes, scope_cache);
        let pdata = OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(buf.into_bytes()).into(),
        );
        effect_handler.send_message(pdata).await?;
        Ok(())
    }
}

/// Index one past the last record of the chunk starting at `start`: as many
/// records as fit the byte budget, but always at least one (even if the first
/// record alone would exceed `budget`). Pure, so it is unit-tested directly
/// without driving the full receiver.
fn chunk_end(record_sizes: &[usize], start: usize, base_bytes: usize, budget: usize) -> usize {
    let mut end = start;
    let mut bytes = base_bytes;
    while end < record_sizes.len() {
        let size = record_sizes[end];
        if end > start && bytes + size > budget {
            break;
        }
        bytes += size;
        end += 1;
    }
    end
}

/// Split a batch of `record_sizes` into consecutive `[start, end)` windows, each
/// staying within `budget` once `base_bytes` of per-message overhead is counted
/// (a lone record larger than `budget` still gets its own window). Pure, so the
/// split loop is unit-tested directly instead of driving the receiver with a
/// budget-sized batch.
fn chunk_ranges(record_sizes: &[usize], base_bytes: usize, budget: usize) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    while start < record_sizes.len() {
        let end = chunk_end(record_sizes, start, base_bytes, budget);
        ranges.push((start, end));
        start = end;
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{receiver::TestRuntime, test_node};
    use otap_df_pdata::OtapPayload;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_telemetry::__log_record_impl;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::{InternalTelemetrySettings, Level, LogContext};
    use prost::Message;
    use std::time::{Instant, SystemTime};

    // Empty entity-key context: every record shares one scope, so a batch can
    // collapse into a single `ScopeLogs` with multiple records.
    fn log_event() -> LogEvent {
        let record =
            __log_record_impl!(Level::INFO, "test.itr.batching").into_record(LogContext::default());
        LogEvent {
            time: SystemTime::UNIX_EPOCH,
            record,
        }
    }

    fn wire_receiver(
        test_runtime: &TestRuntime<OtapPdata>,
        config: Config,
    ) -> (ReceiverWrapper<OtapPdata>, flume::Sender<ObservedEvent>) {
        let (sender, logs_receiver) = flume::bounded(64);
        let settings = InternalTelemetrySettings {
            logs_receiver,
            resource_bytes: Bytes::new(),
            registry: TelemetryRegistryHandle::new(),
            log_tap: None,
        };
        let receiver = ReceiverWrapper::local(
            InternalTelemetryReceiver::new_with_telemetry(config, settings),
            test_node(test_runtime.config().name.clone()),
            Arc::new(NodeUserConfig::new_receiver_config(
                INTERNAL_TELEMETRY_RECEIVER_URN,
            )),
            test_runtime.config(),
        );
        (receiver, sender)
    }

    fn decode_logs(pdata: OtapPdata) -> ExportLogsServiceRequest {
        let bytes = match pdata.payload() {
            OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(b)) => b,
            other => panic!("unexpected payload: {other:?}"),
        };
        ExportLogsServiceRequest::decode(bytes.as_ref()).unwrap()
    }

    #[test]
    fn parse_config_rejects_zero_min_size() {
        let result = InternalTelemetryReceiver::parse_config(&serde_json::json!({ "min_size": 0 }));
        assert!(result.is_err());
    }

    #[test]
    fn validate_config_rejects_max_size_below_min_size() {
        // The factory hook runs the cross-field check, so a bad config is
        // caught during config validation rather than only at receiver start.
        let bad = serde_json::json!({ "min_size": 100, "max_size": 10 });
        assert!((INTERNAL_TELEMETRY_RECEIVER.validate_config)(&bad).is_err());

        let ok = serde_json::json!({ "min_size": 10, "max_size": 100 });
        assert!((INTERNAL_TELEMETRY_RECEIVER.validate_config)(&ok).is_ok());
    }

    #[test]
    fn batches_multiple_records_into_one_scope_logs() {
        let test_runtime = TestRuntime::new();
        // min_size (bytes) exceeds what we send, so the flush happens on
        // shutdown drain rather than on the byte threshold.
        let config = Config {
            min_size: NonZeroUsize::new(10_000).unwrap(),
            max_size: default_max_size(),
            max_batch_duration: Duration::from_secs(60),
        };
        let (receiver, sender) = wire_receiver(&test_runtime, config);

        test_runtime
            .set_receiver(receiver)
            .run_test(move |ctx| async move {
                for _ in 0..3 {
                    sender.send(ObservedEvent::Log(log_event())).unwrap();
                }
                ctx.sleep(Duration::from_millis(50)).await;
                ctx.send_shutdown(Instant::now(), "done").await.unwrap();
            })
            .run_validation(|mut ctx| async move {
                let request = decode_logs(ctx.recv().await.expect("one batched message"));
                let scope_logs = &request.resource_logs[0].scope_logs;
                // Three records, one shared scope: one ScopeLogs holding all three.
                assert_eq!(scope_logs.len(), 1);
                assert_eq!(scope_logs[0].log_records.len(), 3);
            });
    }

    #[test]
    fn default_config_combines_records_into_one_message() {
        let test_runtime = TestRuntime::new();
        // Default config: min_size (64 KiB) is far above two tiny records, so
        // neither the threshold nor the 200ms duration is reached before the
        // 50ms sleep ends; both records are still pending and go out together
        // in the shutdown-drain flush. This only proves batching is on by
        // default (records don't flush one per message, as the pre-batching
        // receiver did); it does not exercise the duration timer itself — see
        // `timer_flushes_partial_batch_before_shutdown` for that.
        let (receiver, sender) = wire_receiver(&test_runtime, Config::default());

        test_runtime
            .set_receiver(receiver)
            .run_test(move |ctx| async move {
                for _ in 0..2 {
                    sender.send(ObservedEvent::Log(log_event())).unwrap();
                }
                ctx.sleep(Duration::from_millis(50)).await;
                ctx.send_shutdown(Instant::now(), "done").await.unwrap();
            })
            .run_validation(|mut ctx| async move {
                let request = decode_logs(ctx.recv().await.expect("one batched message"));
                let scope_logs = &request.resource_logs[0].scope_logs;
                assert_eq!(scope_logs.len(), 1);
                assert_eq!(scope_logs[0].log_records.len(), 2);
            });
    }

    #[test]
    fn small_min_size_flushes_each_record_separately() {
        let test_runtime = TestRuntime::new();
        // min_size of 1 byte is smaller than any single record, so each one
        // crosses the threshold on its own and flushes immediately. Long
        // max_batch_duration and a shutdown that arrives well after both sends
        // mean a passing recv can only be explained by the byte threshold, not
        // the timer or the shutdown drain.
        let config = Config {
            min_size: NonZeroUsize::new(1).unwrap(),
            max_size: default_max_size(),
            max_batch_duration: Duration::from_secs(60),
        };
        let (receiver, sender) = wire_receiver(&test_runtime, config);

        test_runtime
            .set_receiver(receiver)
            .run_test(move |ctx| async move {
                for _ in 0..2 {
                    sender.send(ObservedEvent::Log(log_event())).unwrap();
                }
                ctx.sleep(Duration::from_millis(300)).await;
                ctx.send_shutdown(Instant::now(), "done").await.unwrap();
            })
            .run_validation_concurrent(|mut ctx| async move {
                for _ in 0..2 {
                    let pdata = tokio::time::timeout(Duration::from_millis(150), ctx.recv())
                        .await
                        .expect("threshold flush within bound")
                        .expect("a message");
                    let request = decode_logs(pdata);
                    let scope_logs = &request.resource_logs[0].scope_logs;
                    assert_eq!(scope_logs.len(), 1);
                    assert_eq!(scope_logs[0].log_records.len(), 1);
                }
            });
    }

    #[test]
    fn flush_splits_batch_that_overshoots_max_size() {
        let test_runtime = TestRuntime::new();
        // A batch only flushes once it reaches min_size, so the last record can
        // push its total past max_size. `log_event()` is a fixed fixture, so
        // every record encodes to the same size; setting min_size == max_size to
        // one byte under two records means the second record tips the batch over
        // and the flush splits it into two single-record messages. Built through
        // parse_config so the split is shown reachable from a valid config, not
        // a max_size < min_size one the public path would reject.
        let budget = 2 * record_bytes(&log_event()) - 1;
        let config = InternalTelemetryReceiver::parse_config(&serde_json::json!({
            "min_size": budget,
            "max_size": budget,
            "max_batch_duration": "60s",
        }))
        .expect("min_size == max_size is valid");
        let (receiver, sender) = wire_receiver(&test_runtime, config);

        test_runtime
            .set_receiver(receiver)
            .run_test(move |ctx| async move {
                sender.send(ObservedEvent::Log(log_event())).unwrap();
                sender.send(ObservedEvent::Log(log_event())).unwrap();
                ctx.sleep(Duration::from_millis(50)).await;
                ctx.send_shutdown(Instant::now(), "done").await.unwrap();
            })
            .run_validation(|mut ctx| async move {
                for _ in 0..2 {
                    let request = decode_logs(ctx.recv().await.expect("a split message"));
                    let scope_logs = &request.resource_logs[0].scope_logs;
                    assert_eq!(scope_logs.len(), 1);
                    assert_eq!(scope_logs[0].log_records.len(), 1);
                }
                // Exactly two messages: the receiver has stopped, so the channel
                // is closed and a double-flush would surface as a third here.
                assert!(ctx.recv().await.is_err(), "expected exactly two messages");
            });
    }

    #[test]
    fn timer_flushes_partial_batch_before_shutdown() {
        let test_runtime = TestRuntime::new();
        // High min_size so size never triggers; short duration so the timer does.
        let config = Config {
            min_size: NonZeroUsize::new(10_000).unwrap(),
            max_size: default_max_size(),
            max_batch_duration: Duration::from_millis(20),
        };
        let (receiver, sender) = wire_receiver(&test_runtime, config);

        test_runtime
            .set_receiver(receiver)
            .run_test(move |ctx| async move {
                sender.send(ObservedEvent::Log(log_event())).unwrap();
                sender.send(ObservedEvent::Log(log_event())).unwrap();
                // Stay alive well past the timer, then shut down. The message must
                // be flushed by the timer during this window, not by the shutdown.
                ctx.sleep(Duration::from_millis(300)).await;
                ctx.send_shutdown(Instant::now(), "done").await.unwrap();
            })
            .run_validation_concurrent(|mut ctx| async move {
                // A 150ms bound (well under the 300ms shutdown) fails if the timer
                // never fires, so a passing recv proves the timer-driven flush.
                let pdata = tokio::time::timeout(Duration::from_millis(150), ctx.recv())
                    .await
                    .expect("timer flush within bound")
                    .expect("a message");
                let request = decode_logs(pdata);
                assert_eq!(request.resource_logs[0].scope_logs[0].log_records.len(), 2);
            });
    }

    #[test]
    fn chunk_end_splits_on_byte_budget() {
        let sizes = [300, 300, 300, 300];
        // 300 + 300 = 600 fits a 700 budget; the third would reach 900, so stop.
        assert_eq!(chunk_end(&sizes, 0, 0, 700), 2);
        assert_eq!(chunk_end(&sizes, 2, 0, 700), 4);
        // A single record always advances, even if it alone exceeds the budget.
        assert_eq!(chunk_end(&[1000], 0, 0, 100), 1);
        // Base (resource) bytes count against the budget.
        assert_eq!(chunk_end(&[300, 300], 0, 500, 700), 1);
        // Everything fits one chunk when the budget is ample.
        assert_eq!(chunk_end(&sizes, 0, 0, 10_000), 4);
    }

    #[test]
    fn chunk_ranges_splits_batch_into_bounded_windows() {
        // 4 records of 300 under a 700 budget: two records per window.
        assert_eq!(
            chunk_ranges(&[300, 300, 300, 300], 0, 700),
            [(0, 2), (2, 4)]
        );
        // Ample budget: the whole batch is one window.
        assert_eq!(chunk_ranges(&[300, 300, 300, 300], 0, 10_000), [(0, 4)]);
        // Records that each exceed the budget still advance one at a time, so the
        // loop always terminates and never drops a record.
        assert_eq!(chunk_ranges(&[1000, 1000], 0, 100), [(0, 1), (1, 2)]);
        // Base (resource) bytes count against every window's budget.
        assert_eq!(chunk_ranges(&[300, 300], 500, 700), [(0, 1), (1, 2)]);
        // An empty batch yields no windows.
        assert!(chunk_ranges(&[], 0, 700).is_empty());
    }
}
