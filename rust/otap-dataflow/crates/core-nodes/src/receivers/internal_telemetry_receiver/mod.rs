// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry receiver.
//!
//! This receiver consumes internal logs from the logging channel and emits
//! the logs as OTLP ExportLogsRequest messages into the pipeline.

use async_trait::async_trait;
use bytes::Bytes;
use linkme::distributed_slice;
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

/// Per-message byte budget for splitting a batch, so each emitted
/// `ExportLogsRequest` stays well under typical gRPC/OTLP size limits. The split
/// is a heuristic: it sums each record's pre-encoded body plus framing and does
/// not count all protobuf overhead (e.g. scope attributes), so it is a safety
/// margin, not a hard cap. Internal log records are small, leaving wide headroom.
const MAX_BATCH_BYTES: usize = 1 << 20; // 1 MiB

/// Rough per-record protobuf overhead added to the record body when estimating
/// a message's size (timestamp, severity, event name, length prefixes).
const RECORD_FRAMING_BYTES: usize = 64;

/// Configuration for the internal telemetry receiver.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Records to accumulate before emitting one batched `ExportLogsRequest`,
    /// grouping those that share a scope. Unset (the default) emits each record
    /// immediately, as before. A configured `0` is rejected.
    #[serde(default)]
    batch_size: Option<NonZeroUsize>,

    /// How long a record may wait in a partial batch before being flushed.
    /// Only relevant when `batch_size` is set. Default 200ms.
    #[serde(with = "humantime_serde", default = "default_max_batch_duration")]
    max_batch_duration: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            batch_size: None,
            max_batch_duration: default_max_batch_duration(),
        }
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
            otap_df_config::error::Error::InvalidUserConfig {
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
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
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
    pub fn parse_config(config: &Value) -> Result<Config, otap_df_config::error::Error> {
        serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })
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
        // With no `batch_size` the threshold is 1, so every record flushes on its
        // own, exactly as the receiver behaved before batching.
        let threshold = self.config.batch_size.map_or(1, NonZeroUsize::get);
        let flush_period = self.config.max_batch_duration;
        let mut batch: Vec<LogEvent> = Vec::new();
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
                )
                .await?;
                batch_deadline = None;
            }

            tokio::select! {
                biased;

                // Handle control messages with priority
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            // Chunk by `threshold` so a large backlog drains as
                            // several bounded messages, not one oversized payload.
                            while let Ok(event) = logs_receiver.try_recv() {
                                if let ObservedEvent::Log(log_event) = event {
                                    Self::buffer_log(&log_tap, &mut batch, log_event);
                                    if batch.len() >= threshold {
                                        Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache).await?;
                                    }
                                }
                            }
                            Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache).await?;
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            while let Ok(event) = logs_receiver.try_recv() {
                                if let ObservedEvent::Log(log_event) = event {
                                    Self::buffer_log(&log_tap, &mut batch, log_event);
                                    if batch.len() >= threshold {
                                        Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache).await?;
                                    }
                                }
                            }
                            Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache).await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::CollectTelemetry { .. }) => {
                            // No metrics to report for now
                        }
                        Err(e) => {
                            // Best-effort flush so a control-channel failure does
                            // not drop records already buffered for batching.
                            let _ = Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache).await;
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
                            Self::buffer_log(&log_tap, &mut batch, log_event);
                            if batch.len() >= threshold {
                                Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache).await?;
                                batch_deadline = None;
                            }
                        }
                        Ok(ObservedEvent::Engine(_)) => {
                            // Engine events are not yet processed
                        }
                        Err(_) => {
                            // Channel closed: flush anything pending, then exit.
                            Self::flush_batch(&effect_handler, &mut batch, &resource_bytes, &mut scope_cache).await?;
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
    fn buffer_log(
        log_tap: &Option<InternalLogTapHandle>,
        batch: &mut Vec<LogEvent>,
        log_event: LogEvent,
    ) {
        if let Some(tap) = log_tap.as_ref() {
            tap.record(log_event.clone());
        }
        batch.push(log_event);
    }

    /// Encode and send the accumulated batch, grouping records by scope. A batch
    /// that would exceed [`MAX_BATCH_BYTES`] is split (see [`chunk_end`]) so every
    /// message stays well below the transport size limit. Clears the batch; a
    /// no-op when empty.
    async fn flush_batch(
        effect_handler: &local::EffectHandler<OtapPdata>,
        batch: &mut Vec<LogEvent>,
        resource_bytes: &Bytes,
        scope_cache: &mut ScopeToBytesMap,
    ) -> Result<(), Error> {
        if batch.is_empty() {
            return Ok(());
        }
        let base = resource_bytes.len();
        let total = base
            + batch
                .iter()
                .map(|e| e.record.body_attrs_bytes.len() + RECORD_FRAMING_BYTES)
                .sum::<usize>();

        // Common case: the whole batch fits one message, so encode it directly
        // with no per-record size vector or chunk loop. The default (batching
        // off) one-record flush always lands here.
        if total <= MAX_BATCH_BYTES {
            Self::send_chunk(effect_handler, batch, total, resource_bytes, scope_cache).await?;
            batch.clear();
            return Ok(());
        }

        // Oversized batch: split into byte-bounded messages.
        let sizes: Vec<usize> = batch
            .iter()
            .map(|e| e.record.body_attrs_bytes.len() + RECORD_FRAMING_BYTES)
            .collect();
        let mut start = 0;
        while start < batch.len() {
            let end = chunk_end(&sizes, start, base, MAX_BATCH_BYTES);
            let capacity = base + sizes[start..end].iter().sum::<usize>();
            Self::send_chunk(
                effect_handler,
                &batch[start..end],
                capacity,
                resource_bytes,
                scope_cache,
            )
            .await?;
            start = end;
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
/// records as fit the byte budget, but always at least one (a record's body is
/// size-bounded, so a single one always fits). Pure, so it is unit-tested
/// directly without driving the full receiver.
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
    fn batches_multiple_records_into_one_scope_logs() {
        let test_runtime = TestRuntime::new();
        // batch_size exceeds what we send, so the flush happens on shutdown drain.
        let config = Config {
            batch_size: NonZeroUsize::new(10),
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
    fn default_config_emits_one_record_per_message() {
        let test_runtime = TestRuntime::new();
        // Batching off (the default): each record is its own message.
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
                for _ in 0..2 {
                    let request = decode_logs(ctx.recv().await.expect("a per-record message"));
                    let scope_logs = &request.resource_logs[0].scope_logs;
                    assert_eq!(scope_logs.len(), 1);
                    assert_eq!(scope_logs[0].log_records.len(), 1);
                }
            });
    }

    #[test]
    fn timer_flushes_partial_batch_before_shutdown() {
        let test_runtime = TestRuntime::new();
        // High batch_size so size never triggers; short duration so the timer does.
        let config = Config {
            batch_size: NonZeroUsize::new(10),
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
}
