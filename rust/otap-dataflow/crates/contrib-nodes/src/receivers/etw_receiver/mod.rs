// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ETW (Event Tracing for Windows) Receiver
//!
//! Receives ETW events from Windows ETW sessions, converts them to OTAP Arrow
//! log record batches, and forwards them through the pipeline.
//!
//! This module is compiled only on Windows (`#[cfg(target_os = "windows")]`
//! in the parent `receivers/mod.rs`).
//!
//! ## Multi-core fan-out
//!
//! Windows allows only one real-time ETW session per session name. The engine
//! may instantiate the receiver once per allocated core.  To reconcile these
//! two models, one session is created **per `session_name`** with N consumer
//! channels (one per core).  The `ProcessTrace` callback round-robins events
//! across all channels so each core receives an even share of the event stream.
//!
//! ```text
//! ProcessTrace OS thread  (one per session_name, lazily spawned)
//! callback: txs[next].try_send(data); next = (next+1) % N
//!       |          |          |
//!     tx[0]      tx[1]      tx[2]
//!       v          v          v
//!      mpsc       mpsc       mpsc
//!       v          v          v
//!  +--------+ +--------+ +--------+
//!  | core 0 | | core 1 | | core 2 |
//!  | rx[0]  | | rx[1]  | | rx[2]  |
//!  +--------+ +--------+ +--------+
//! ```
//!
//! ## Quick start
//!
//! ```yaml
//! etw:
//!   type: receiver:etw
//!   config:
//!     providers:
//!       - guid: "d2387720-2907-5677-8625-c1bdc4155197"
//!         level: verbose
//!     batching:
//!       max_size: 100
//!       max_duration: "100ms"
//! ```

mod arrow_records_encoder;
mod session;

use arrow_records_encoder::EtwArrowRecordsBuilder;
use session::EtwEventData;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{
    MessageSourceLocalEffectHandlerExtension,
    effect_handler::TelemetryTimerCancelHandle,
    error::{Error, ReceiverErrorKind, format_error_sources},
    local::receiver as local,
};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use tokio::time::{self, MissedTickBehavior};

use std::cell::RefCell;
use std::num::NonZeroU16;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

/// URN for the ETW receiver.
pub const ETW_RECEIVER_URN: &str = "urn:otel:receiver:etw";

// ── Defaults ─────────────────────────────────────────────────────────────────

// 512 is non-zero, so `unwrap()` never panics (evaluated at compile time).
const DEFAULT_BATCH_MAX_SIZE: NonZeroU16 = NonZeroU16::new(512).unwrap();
const DEFAULT_BATCH_MAX_DURATION: Duration = Duration::from_millis(100);

/// Upper bound on the time spent draining queued events during `DrainIngress`.
///
/// The drain budget is computed as 90% of the remaining time until the
/// deadline, but capped at this value so that a generous deadline does not
/// stall shutdown while a busy provider keeps producing events.
const MAX_DRAIN_BUDGET: Duration = Duration::from_secs(1);

// ── Configuration ────────────────────────────────────────────────────────────

/// Trace level filter for ETW providers, matching the standard five ETW levels.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
enum TraceLevel {
    /// Critical errors only.
    Critical,
    /// Errors and critical events.
    Error,
    /// Warnings, errors, and critical events.
    Warning,
    /// Informational events and above (default).
    #[default]
    Information,
    /// All events including verbose/debug output.
    Verbose,
}

/// Configuration for a single ETW provider to trace.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ProviderConfig {
    /// The ETW provider name (e.g. `"Microsoft-Windows-Kernel-Process"`).
    /// Mutually exclusive with `guid`.
    #[serde(default)]
    pub name: Option<String>,

    /// The ETW provider GUID (e.g. `"22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"`).
    /// Mutually exclusive with `name`.
    #[serde(default)]
    pub guid: Option<String>,

    /// Trace level filter. Defaults to `information`.
    #[serde(default)]
    pub level: TraceLevel,

    /// Optional keywords bitmask to further filter events.
    /// When omitted, all keywords are matched.
    #[serde(default)]
    pub keywords: Option<u64>,
}

/// In-memory OTAP log batching policy.
///
/// Decoded ETW events are accumulated into OTAP log batches before they are
/// sent downstream.  Batching improves throughput but increases the number of
/// decoded records that can be lost if the process exits before a flush.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BatchConfig {
    /// Maximum number of log records per emitted OTAP batch.
    /// Must be greater than zero; `0` is rejected at deserialization time.
    #[serde(default = "default_batch_max_size")]
    max_size: NonZeroU16,
    /// Maximum time to hold a non-empty batch before flushing it downstream.
    #[serde(default = "default_batch_max_duration")]
    #[serde(with = "humantime_serde")]
    max_duration: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_size: default_batch_max_size(),
            max_duration: default_batch_max_duration(),
        }
    }
}

/// Top-level configuration for the ETW receiver.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Config {
    /// One or more ETW providers to subscribe to.
    pub providers: Vec<ProviderConfig>,

    /// Name of the ETW trace session. Defaults to `"OtelArrowETW"`.
    #[serde(default = "default_session_name")]
    pub session_name: String,

    /// OTAP log batching limits.
    #[serde(default)]
    pub batching: Option<BatchConfig>,
}

impl Config {
    /// Validates domain rules that cannot be expressed by the type system alone.
    ///
    /// # Rules
    ///
    /// * At least one provider must be specified.
    /// * Each provider must specify exactly one of `name` or `guid` (not both, not neither).
    ///
    /// # Errors
    ///
    /// Returns [`otap_df_config::error::Error::InvalidUserConfig`] when a rule is violated.
    fn validate(&self) -> Result<(), otap_df_config::error::Error> {
        if self.providers.is_empty() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "at least one ETW provider must be configured".to_string(),
            });
        }

        for (i, provider) in self.providers.iter().enumerate() {
            match (&provider.name, &provider.guid) {
                (Some(_), Some(_)) => {
                    return Err(otap_df_config::error::Error::InvalidUserConfig {
                        error: format!(
                            "provider[{i}]: 'name' and 'guid' are mutually exclusive — specify one, not both"
                        ),
                    });
                }
                (None, None) => {
                    return Err(otap_df_config::error::Error::InvalidUserConfig {
                        error: format!("provider[{i}]: either 'name' or 'guid' must be specified"),
                    });
                }
                _ => {} // Valid: exactly one of name or guid is specified.
            }
        }

        if let Some(ref batching) = self.batching {
            if batching.max_duration.is_zero() {
                return Err(otap_df_config::error::Error::InvalidUserConfig {
                    error: "ETW receiver `batching.max_duration` must be greater than zero"
                        .to_string(),
                });
            }
        }

        Ok(())
    }
}

fn default_session_name() -> String {
    "OtelArrowETW".to_string()
}

const fn default_batch_max_size() -> NonZeroU16 {
    DEFAULT_BATCH_MAX_SIZE
}

const fn default_batch_max_duration() -> Duration {
    DEFAULT_BATCH_MAX_DURATION
}

// ── Receiver struct ──────────────────────────────────────────────────────────

/// ETW receiver that subscribes to Windows ETW trace sessions and converts
/// events into OTAP Arrow log records.
///
/// Each per-core instance holds its own `event_rx` obtained from the
/// per-session-name ETW session at factory time.
struct EtwReceiver {
    config: Config,
    batching: BatchConfig,
    metrics: Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
    /// Per-core consumer channel from the per-session-name ETW session.
    event_rx: tokio::sync::mpsc::Receiver<EtwEventData>,
}

impl EtwReceiver {
    /// Create a new receiver, acquiring one consumer channel from the
    /// per-session-name ETW session.
    ///
    /// Called at **factory time** (once per allocated core).  The first call
    /// for a given `session_name` lazily initializes the session and spawns
    /// the `ProcessTrace` thread.
    fn from_config(
        pipeline: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let cfg: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        cfg.validate()?;

        let num_cores = pipeline.num_cores();
        let metrics = pipeline.register_metrics::<EtwReceiverMetrics>();
        let batching = cfg.batching.clone().unwrap_or_default();

        // Acquire this core's consumer channel from the per-session-name
        // session.  The first call initializes the session; subsequent calls
        // pop from the pre-allocated pool.
        let event_rx = session::subscribe(&cfg, num_cores).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: format!("ETW session initialization failed: {e}"),
            }
        })?;

        Ok(EtwReceiver {
            config: cfg,
            batching,
            metrics: Rc::new(RefCell::new(metrics)),
            event_rx,
        })
    }
}

// ── Batch flush helpers ──────────────────────────────────────────────────────

/// Build the pending Arrow batch from `builder`, recording the failure metric
/// on error.
///
/// `builder` is always reset to a fresh empty builder (its contents are taken
/// via [`std::mem::take`]) regardless of outcome. Returns `Ok(None)` when the
/// builder is empty (nothing to flush). On a `build()` failure the
/// `received_events_forward_failed` metric is recorded and the error is
/// returned — a build failure indicates an encoding bug rather than a transient
/// downstream condition, so it is surfaced to the caller.
fn build_pending_batch(
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
    builder: &mut EtwArrowRecordsBuilder,
) -> Result<Option<(OtapPdata, u64)>, Error> {
    if builder.is_empty() {
        return Ok(None);
    }

    let item_count = u64::from(builder.len());

    let payload = match std::mem::take(builder).build() {
        Ok(payload) => payload,
        Err(error) => {
            metrics
                .borrow_mut()
                .received_events_forward_failed
                .add(item_count);
            return Err(Error::ReceiverError {
                receiver: effect_handler.receiver_id(),
                kind: ReceiverErrorKind::Transport,
                error: "failed to build ETW Arrow batch".to_owned(),
                source_detail: format_error_sources(&error),
            });
        }
    };

    Ok(Some((
        OtapPdata::new_todo_context(payload.into()),
        item_count,
    )))
}

/// Flush the current Arrow batch downstream via the awaiting effect handler.
///
/// `builder` is always reset to a fresh empty builder regardless of outcome.
///
/// A forward failure is treated as a **per-batch loss event**, not a fatal
/// error: the `received_events_forward_failed` metric is incremented, a warning
/// is logged, and `Ok(())` is returned so the receiver stays alive. On the
/// awaiting path the only reachable send failure is the downstream channel
/// being closed (backpressure parks rather than failing), which can happen
/// transiently while a downstream node restarts; the control plane
/// (`DrainIngress`/`Shutdown`) decides when the receiver actually stops.
/// A `build()` failure still propagates, since it signals an encoding bug.
async fn flush_batch(
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
    builder: &mut EtwArrowRecordsBuilder,
) -> Result<(), Error> {
    let Some((pdata, item_count)) = build_pending_batch(effect_handler, metrics, builder)? else {
        return Ok(());
    };

    if let Err(error) = effect_handler.send_message_with_source_node(pdata).await {
        metrics
            .borrow_mut()
            .received_events_forward_failed
            .add(item_count);
        let error_msg = error.to_string();
        otel_warn!(
            "etw_receiver.forward_failed",
            message = "failed to forward ETW Arrow batch downstream; dropping batch",
            dropped_events = item_count,
            error = error_msg.as_str(),
        );
        return Ok(());
    }

    metrics
        .borrow_mut()
        .received_events_forwarded
        .add(item_count);
    Ok(())
}

/// Non-blocking sibling of [`flush_batch`] used during shutdown/teardown paths
/// (`DrainIngress` and the channel-closed branch).
///
/// `builder` is always reset to a fresh empty builder regardless of outcome.
/// The flush uses `try_send_message_with_source_node` so a slow or
/// shutting-down downstream cannot park the task past the drain deadline.
///
/// Like [`flush_batch`], a forward failure (downstream channel full or closed)
/// is treated as a per-batch loss event: the `received_events_forward_failed`
/// metric is incremented, a warning is logged, and `Ok(())` is returned so the
/// receiver stays alive. A `build()` failure still propagates.
fn try_flush_batch(
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
    builder: &mut EtwArrowRecordsBuilder,
) -> Result<(), Error> {
    let Some((pdata, item_count)) = build_pending_batch(effect_handler, metrics, builder)? else {
        return Ok(());
    };

    if let Err(error) = effect_handler.try_send_message_with_source_node(pdata) {
        metrics
            .borrow_mut()
            .received_events_forward_failed
            .add(item_count);
        let error_msg = error.to_string();
        otel_warn!(
            "etw_receiver.forward_failed",
            message = "failed to forward ETW Arrow batch downstream; dropping batch",
            dropped_events = item_count,
            error = error_msg.as_str(),
        );
        return Ok(());
    }

    metrics
        .borrow_mut()
        .received_events_forwarded
        .add(item_count);
    Ok(())
}

// ── Event processing loop ────────────────────────────────────────────────────

impl EtwReceiver {
    /// Main event loop when an ETW session is active.
    ///
    /// Consumes events from the per-core MPSC channel, accumulates them into
    /// Arrow batches, and flushes when either `max_size` or `max_duration`
    /// thresholds are reached.  Control messages are processed with priority.
    async fn run_event_loop(
        &mut self,
        ctrl_chan: &mut local::ControlChannel<OtapPdata>,
        effect_handler: &local::EffectHandler<OtapPdata>,
        telemetry_timer_handle: TelemetryTimerCancelHandle<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let mut channel_alive = true;
        let mut builder = EtwArrowRecordsBuilder::new();
        let batch_max_size = self.batching.max_size.get();

        // Use `interval_at` so the first tick fires one `max_duration` from now
        // rather than immediately (which `interval` would do, causing a
        // pointless flush of an empty batch at startup).
        let mut flush_interval = time::interval_at(
            time::Instant::now() + self.batching.max_duration,
            self.batching.max_duration,
        );
        flush_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                biased; // Prioritise control messages over data.

                ctrl_msg = ctrl_chan.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            let _ = telemetry_timer_handle.cancel().await;

                            // Drain remaining events before flushing the final
                            // batch, but bound the work by a deadline.  The ETW
                            // `ProcessTrace` producer thread keeps round-robining
                            // events into this core's channel, so a busy provider
                            // would otherwise let `try_recv` spin past the
                            // deadline indefinitely and risk a forced kill.
                            let now = std::time::Instant::now();
                            let remaining = deadline.saturating_duration_since(now);
                            let drain_budget =
                                std::cmp::min(remaining * 9 / 10, MAX_DRAIN_BUDGET);
                            let drain_deadline = now + drain_budget;

                            while std::time::Instant::now() < drain_deadline {
                                match self.event_rx.try_recv() {
                                    Ok(event) => {
                                        self.metrics.borrow_mut().received_events_total.inc();
                                        builder.append(&event);
                                        if builder.len() >= batch_max_size {
                                            try_flush_batch(
                                                effect_handler,
                                                &self.metrics,
                                                &mut builder,
                                            )?;
                                        }
                                    }
                                    // Empty or Disconnected — nothing more to do.
                                    Err(_) => break,
                                }
                            }

                            // Flush the trailing partial batch (no-op if empty).
                            // Use the non-blocking flush so a slow downstream
                            // cannot park us past the deadline.
                            try_flush_batch(effect_handler, &self.metrics, &mut builder)?;
                            effect_handler.notify_receiver_drained().await?;
                            let snapshot = self.metrics.borrow().snapshot();
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            let _ = telemetry_timer_handle.cancel().await;
                            otel_info!(
                                "etw_receiver.shutdown",
                                message = "ETW receiver shutting down",
                            );
                            let snapshot = self.metrics.borrow().snapshot();
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        }
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            let mut m = self.metrics.borrow_mut();
                            let _ = metrics_reporter.report(&mut m);
                        }
                        Ok(NodeControlMsg::MemoryPressureChanged { .. }) => {
                            // TODO: Implement shedding if needed.
                        }
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // Other control messages — ignore for now.
                        }
                    }
                }

                // Timer-based batch flush. Skip once the channel is closed:
                // the final batch was already flushed when the channel ended,
                // so there is nothing left to flush periodically.
                _ = flush_interval.tick(), if channel_alive => {
                    flush_batch(effect_handler, &self.metrics, &mut builder).await?;
                }

                // Receive event data from the ETW session.
                // Only poll when the channel is still alive to avoid
                // spinning on a closed channel.
                event_data = self.event_rx.recv(), if channel_alive => {
                    match event_data {
                        Some(event) => {
                            self.metrics.borrow_mut().received_events_total.inc();
                            builder.append(&event);

                            // Flush when batch is full
                            if builder.len() >= batch_max_size {
                                flush_batch(effect_handler, &self.metrics, &mut builder).await?;
                            }
                        }
                        None => {
                            // Channel closed — the ETW session thread has
                            // exited (process shutdown or unrecoverable error).
                            // Flush any remaining events, then stop polling.
                            // Use the non-blocking flush so a slow or
                            // already-closed downstream cannot park us while
                            // the session is tearing down.
                            try_flush_batch(effect_handler, &self.metrics, &mut builder)?;
                            channel_alive = false;
                            otel_info!(
                                "etw_receiver.session_ended",
                                message = "ETW session event channel closed",
                            );
                        }
                    }
                }
            }
        }
    }
}

// ── Factory registration ─────────────────────────────────────────────────────

/// Register the ETW receiver in the pipeline factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static ETW_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: ETW_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ReceiverWrapper::local(
            EtwReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

// ── Receiver trait implementation ────────────────────────────────────────────

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for EtwReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_chan: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // Start periodic telemetry collection.
        let telemetry_timer_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        otel_info!(
            "etw_receiver.start",
            session_name = self.config.session_name.as_str(),
            provider_count = self.config.providers.len(),
            batch_max_size = self.batching.max_size.get(),
            batch_max_duration_ms = self.batching.max_duration.as_millis() as u64,
        );

        self.run_event_loop(&mut ctrl_chan, &effect_handler, telemetry_timer_handle)
            .await
    }
}

// ── Telemetry ────────────────────────────────────────────────────────────────

/// Receiver-level metrics for the ETW receiver.
#[metric_set(name = "etw.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct EtwReceiverMetrics {
    /// Total number of ETW events observed from the trace session.
    #[metric(unit = "{event}")]
    pub received_events_total: Counter<u64>,

    /// Number of ETW events successfully forwarded downstream.
    #[metric(unit = "{event}")]
    pub received_events_forwarded: Counter<u64>,

    /// Number of ETW events that failed to parse.
    #[metric(unit = "{event}")]
    pub received_events_invalid: Counter<u64>,

    /// Number of ETW events that could not be forwarded downstream and were
    /// therefore dropped. Counts both batch `build()` failures (an encoding
    /// bug) and send failures (the downstream channel was full or closed).
    #[metric(unit = "{event}")]
    pub received_events_forward_failed: Counter<u64>,

    /// Number of ETW events dropped due to process-wide memory pressure.
    #[metric(unit = "{event}")]
    pub received_events_rejected_memory_pressure: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(providers: Vec<ProviderConfig>) -> Config {
        Config {
            session_name: "test-session".to_string(),
            providers,
            batching: None,
        }
    }

    fn provider_with_guid(guid: &str) -> ProviderConfig {
        ProviderConfig {
            name: None,
            guid: Some(guid.to_string()),
            level: TraceLevel::default(),
            keywords: None,
        }
    }

    fn provider_with_name(name: &str) -> ProviderConfig {
        ProviderConfig {
            name: Some(name.to_string()),
            guid: None,
            level: TraceLevel::default(),
            keywords: None,
        }
    }

    #[test]
    fn validate_accepts_single_guid_provider() {
        let cfg = make_config(vec![provider_with_guid(
            "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716",
        )]);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_accepts_single_name_provider() {
        let cfg = make_config(vec![provider_with_name("Microsoft-Windows-PowerShell")]);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_accepts_multiple_providers() {
        let cfg = make_config(vec![
            provider_with_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"),
            provider_with_name("Microsoft-Windows-PowerShell"),
        ]);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_rejects_empty_providers() {
        let cfg = make_config(vec![]);
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("at least one ETW provider"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn validate_rejects_both_name_and_guid() {
        let cfg = make_config(vec![ProviderConfig {
            name: Some("SomeProvider".to_string()),
            guid: Some("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716".to_string()),
            level: TraceLevel::default(),
            keywords: None,
        }]);
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("mutually exclusive"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn validate_rejects_neither_name_nor_guid() {
        let cfg = make_config(vec![ProviderConfig {
            name: None,
            guid: None,
            level: TraceLevel::default(),
            keywords: None,
        }]);
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("either 'name' or 'guid' must be specified"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn validate_reports_correct_index() {
        let cfg = make_config(vec![
            provider_with_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"),
            ProviderConfig {
                name: None,
                guid: None,
                level: TraceLevel::default(),
                keywords: None,
            },
        ]);
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("provider[1]"),
            "expected error at index 1, got: {msg}"
        );
    }

    #[test]
    fn validate_rejects_zero_batch_duration() {
        let cfg = Config {
            session_name: "test".to_string(),
            providers: vec![provider_with_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716")],
            batching: Some(BatchConfig {
                max_size: NonZeroU16::new(100).expect("100 is non-zero"),
                max_duration: Duration::ZERO,
            }),
        };
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("max_duration"),
            "expected max_duration error, got: {msg}"
        );
    }

    #[test]
    fn default_batch_config() {
        let cfg = BatchConfig::default();
        assert_eq!(cfg.max_size, DEFAULT_BATCH_MAX_SIZE);
        assert_eq!(cfg.max_duration, DEFAULT_BATCH_MAX_DURATION);
    }
}

/// End-to-end Windows integration tests for the ETW receiver. See
/// [`windows_e2e_tests`] for details.
#[cfg(all(test, target_os = "windows"))]
mod windows_e2e_tests;
