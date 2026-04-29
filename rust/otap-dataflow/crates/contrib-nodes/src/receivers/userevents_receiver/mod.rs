// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

//! Linux userevents receiver.

mod arrow_records_encoder;
mod decoder;
mod metrics;
mod one_collect_adapter;
mod session;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::memory_limiter::LocalReceiverAdmissionState;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{
    MessageSourceLocalEffectHandlerExtension, ReceiverFactory,
    error::{Error, ReceiverErrorKind, format_error_sources},
    local::receiver as local,
};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::metrics::MetricSet;
use serde::Deserialize;
use serde_json::Value;

use self::arrow_records_encoder::ArrowRecordsBuilder;
use self::decoder::DecodedUsereventsRecord;
use self::metrics::UsereventsReceiverMetrics;
#[cfg(target_os = "linux")]
use self::session::SessionInitError;
use self::session::{RawUsereventsRecord, SessionDrainStats, UsereventsSession};
#[cfg(target_os = "linux")]
use otap_df_engine::control::NodeControlMsg;
#[cfg(target_os = "linux")]
use otap_df_telemetry::{otel_info, otel_warn};
#[cfg(target_os = "linux")]
use tokio::time::{self, MissedTickBehavior};

const DEFAULT_PER_CPU_BUFFER_SIZE: usize = 1024 * 1024;
const DEFAULT_WAKEUP_WATERMARK: usize = 256 * 1024;
const DEFAULT_MAX_PENDING_EVENTS: usize = 4096;
const DEFAULT_MAX_PENDING_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_MAX_RECORDS_PER_TURN: usize = 1024;
const DEFAULT_MAX_BYTES_PER_TURN: usize = 1024 * 1024;
const DEFAULT_MAX_DRAIN_NS: Duration = Duration::from_millis(2);
const DEFAULT_BATCH_MAX_SIZE: u16 = 512;
const DEFAULT_BATCH_MAX_DURATION: Duration = Duration::from_millis(50);
const DEFAULT_LATE_REGISTRATION_POLL: Duration = Duration::from_secs(1);

/// URN for the Linux userevents receiver.
///
/// The receiver identity is vendor-neutral: it collects Linux `user_events`
/// and structurally decodes records into OTAP logs.
pub const USEREVENTS_RECEIVER_URN: &str = "urn:otel:receiver:userevents";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum FormatConfig {
    /// Decode the sample using the Linux tracefs `format` metadata registered
    /// for the tracepoint.
    ///
    /// This is the standard Linux user_events/tracepoint shape used by tools
    /// such as perf and ftrace: field names, offsets, sizes, and C-like type
    /// names come from tracefs, while values come from the raw sample bytes.
    #[default]
    Tracefs,
    /// Decode the user payload as an EventHeader self-describing event.
    ///
    /// EventHeader is still decoded structurally and vendor-neutrally here: the
    /// receiver flattens EventHeader structs into `Struct.field` attributes but
    /// does not attach semantic meaning to field names. Schema-specific
    /// interpretation belongs in processors.
    EventHeader,
}

/// One user_events tracepoint subscription.
///
/// Each subscription opens the named tracepoint and chooses the payload decoder
/// used for samples read from that tracepoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubscriptionConfig {
    /// Tracepoint name to subscribe to, for example `my_provider:event_name`.
    tracepoint: String,
    /// Payload decoding format for records emitted by this tracepoint.
    #[serde(default)]
    format: FormatConfig,
}

/// Optional polling for tracepoints that may be registered after startup.
///
/// Linux user_events tracepoints can appear after the receiver starts if the
/// producer process registers them later. When enabled, the receiver retries
/// missing subscriptions instead of treating startup absence as final.
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
struct LateRegistrationConfig {
    /// Enables periodic retries for subscriptions whose tracepoints do not
    /// exist yet.
    #[serde(default)]
    enabled: bool,
    /// Delay, in milliseconds, between late-registration retry attempts.
    #[serde(default = "default_late_registration_poll_ms")]
    poll_interval_ms: u64,
}

/// Low-level tracepoint session settings.
///
/// These values tune how the receiver opens and services the per-CPU perf ring
/// buffers used by the underlying user_events tracepoint session.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct SessionConfig {
    /// Ring buffer capacity allocated per CPU for tracepoint samples.
    #[serde(default = "default_per_cpu_buffer_size")]
    per_cpu_buffer_size: usize,
    #[expect(
        dead_code,
        reason = "reserved for future one-collect wakeup watermark support"
    )]
    /// Planned readiness threshold for waking the reader when buffered data is
    /// available.
    ///
    /// This is currently parsed for forward compatibility but not applied until
    /// one_collect exposes wakeup/readiness and watermark configuration.
    // TODO: Wire this into the perf ring setup once one_collect exposes
    // wakeup/readiness and watermark configuration for tracepoint sessions.
    #[serde(default = "default_wakeup_watermark")]
    wakeup_watermark: usize,
    /// Maximum number of parsed events buffered between one_collect callbacks
    /// and the receiver drain loop.
    #[serde(default = "default_max_pending_events")]
    max_pending_events: usize,
    /// Maximum raw event payload bytes buffered between one_collect callbacks
    /// and the receiver drain loop.
    #[serde(default = "default_max_pending_bytes")]
    max_pending_bytes: usize,
    /// Behavior for tracepoints that are absent during initial session setup.
    #[serde(default)]
    late_registration: LateRegistrationConfig,
}

/// Per-turn limits for reading samples from the tracepoint session.
///
/// The receiver runs cooperatively with the local pipeline task. These limits
/// bound how much work one drain pass can do before yielding back to the engine.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct DrainConfig {
    /// Maximum number of decoded records to read during one receiver turn.
    #[serde(default = "default_max_records_per_turn")]
    max_records_per_turn: usize,
    /// Maximum raw payload bytes to read during one receiver turn.
    #[serde(default = "default_max_bytes_per_turn")]
    max_bytes_per_turn: usize,
    /// Maximum wall-clock time spent draining records during one receiver turn.
    #[serde(default = "default_max_drain_ns")]
    #[serde(with = "humantime_serde")]
    max_drain_ns: Duration,
}

/// In-memory OTAP log batching policy.
///
/// Decoded user_events records are accumulated into OTAP log batches before
/// they are sent downstream. Batching improves throughput but also increases
/// how many decoded records can be lost if the process exits before a flush.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct BatchConfig {
    // NOTE: These batches are in-memory only. Unlike a network receiver such as
    // syslog, there is typically no practical sender-side retry or replay path
    // for user_events payloads after this process crashes. Syslog transport
    // also does not generally provide per-message acknowledgements, but some
    // senders/agents may buffer and retry on reconnect; user_events producers
    // normally cannot. Larger `max_size` / `max_duration` values therefore
    // increase the amount of irrecoverable data that can be lost before a flush.
    // TODO: Revisit these batching tradeoffs if we add durable buffering or
    // upstream retry/replay support for this ingestion path.
    /// Maximum number of log records per emitted OTAP batch.
    #[serde(default = "default_batch_max_size")]
    max_size: u16,
    /// Maximum time to hold a non-empty batch before flushing it downstream.
    #[serde(default = "default_batch_max_duration")]
    #[serde(with = "humantime_serde")]
    max_duration: Duration,
}

/// Policy for handling records when the receiver cannot send downstream.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum OverflowMode {
    /// Drop the current record or batch and continue reading future events.
    Drop,
}

/// Downstream backpressure behavior.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct OverflowConfig {
    /// Action to take when the downstream local channel is full.
    #[serde(default = "default_overflow_mode")]
    on_downstream_full: OverflowMode,
}

/// User-supplied configuration for the Linux user_events receiver.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct UsereventsReceiverConfig {
    /// Required non-empty list of tracepoints to subscribe to.
    subscriptions: Vec<SubscriptionConfig>,
    /// Tracepoint session setup and late-registration settings.
    #[serde(default)]
    session: Option<SessionConfig>,
    /// Cooperative drain-loop limits.
    #[serde(default)]
    drain: Option<DrainConfig>,
    /// OTAP log batching limits.
    #[serde(default)]
    batching: Option<BatchConfig>,
    /// Backpressure behavior when downstream cannot accept records.
    #[serde(default)]
    overflow: Option<OverflowConfig>,
}

/// Runtime state for one local user_events receiver task.
struct UsereventsReceiver {
    /// Tracepoint subscriptions owned by this receiver instance.
    subscriptions: Vec<SubscriptionConfig>,
    /// Low-level session configuration used when opening tracepoint readers.
    session: SessionConfig,
    /// Per-turn drain limits used to keep receiver work cooperative.
    drain: DrainConfig,
    /// In-memory batching policy for decoded log records.
    batching: BatchConfig,
    /// Policy applied when downstream admission or send capacity is exhausted.
    overflow: OverflowConfig,
    /// Local pipeline CPU/shard assigned by the engine.
    cpu_id: usize,
    /// Receiver metric set shared with the local pipeline task.
    metrics: Rc<RefCell<MetricSet<UsereventsReceiverMetrics>>>,
    /// Local admission state used to coordinate with memory limiting.
    admission_state: LocalReceiverAdmissionState,
}

impl UsereventsReceiver {
    fn from_config(
        pipeline: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: UsereventsReceiverConfig =
            serde_json::from_value(config.clone()).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?;

        Self::validate_subscriptions(&config.subscriptions)?;
        let session = config.session.clone().unwrap_or(SessionConfig {
            per_cpu_buffer_size: default_per_cpu_buffer_size(),
            wakeup_watermark: default_wakeup_watermark(),
            max_pending_events: default_max_pending_events(),
            max_pending_bytes: default_max_pending_bytes(),
            late_registration: LateRegistrationConfig {
                enabled: false,
                poll_interval_ms: default_late_registration_poll_ms(),
            },
        });
        Self::validate_session(&session)?;
        let drain = config.drain.clone().unwrap_or(DrainConfig {
            max_records_per_turn: default_max_records_per_turn(),
            max_bytes_per_turn: default_max_bytes_per_turn(),
            max_drain_ns: default_max_drain_ns(),
        });
        Self::validate_drain(&drain)?;
        let batching = config.batching.clone().unwrap_or(BatchConfig {
            max_size: default_batch_max_size(),
            max_duration: default_batch_max_duration(),
        });
        Self::validate_batching(&batching)?;
        let overflow = config.overflow.clone().unwrap_or(OverflowConfig {
            on_downstream_full: default_overflow_mode(),
        });

        Ok(Self {
            subscriptions: config.subscriptions,
            session,
            drain,
            batching,
            overflow,
            cpu_id: pipeline.core_id(),
            metrics: Rc::new(RefCell::new(
                pipeline.register_metrics::<UsereventsReceiverMetrics>(),
            )),
            admission_state: LocalReceiverAdmissionState::from_process_state(
                &pipeline.memory_pressure_state(),
            ),
        })
    }

    fn validate_subscriptions(
        subscriptions: &[SubscriptionConfig],
    ) -> Result<(), otap_df_config::error::Error> {
        if subscriptions.is_empty() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "userevents receiver requires at least one subscription".to_owned(),
            });
        }
        for subscription in subscriptions {
            Self::validate_tracepoint(&subscription.tracepoint)?;
        }
        Ok(())
    }

    fn validate_tracepoint(tracepoint: &str) -> Result<(), otap_df_config::error::Error> {
        let Some((group, event_name)) = tracepoint.split_once(':') else {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "userevents receiver tracepoint `{tracepoint}` must use `user_events:<event>`"
                ),
            });
        };
        if group != "user_events" || event_name.is_empty() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "userevents receiver tracepoint `{tracepoint}` must use `user_events:<event>`"
                ),
            });
        }
        Ok(())
    }

    fn validate_session(session: &SessionConfig) -> Result<(), otap_df_config::error::Error> {
        if session.max_pending_events == 0 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "userevents receiver `session.max_pending_events` must be greater than zero"
                    .to_owned(),
            });
        }
        if session.max_pending_bytes == 0 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "userevents receiver `session.max_pending_bytes` must be greater than zero"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_drain(drain: &DrainConfig) -> Result<(), otap_df_config::error::Error> {
        if drain.max_records_per_turn == 0 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "userevents receiver `drain.max_records_per_turn` must be greater than zero"
                    .to_owned(),
            });
        }
        if drain.max_bytes_per_turn == 0 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "userevents receiver `drain.max_bytes_per_turn` must be greater than zero"
                    .to_owned(),
            });
        }
        if drain.max_drain_ns.is_zero() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "userevents receiver `drain.max_drain_ns` must be greater than zero; \
                        a zero budget would starve either parsing or the pending-queue \
                        pop phase under continuous load"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_batching(batching: &BatchConfig) -> Result<(), otap_df_config::error::Error> {
        if batching.max_duration.is_zero() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "userevents receiver `batching.max_duration` must be greater than zero"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

fn drop_batch(
    metrics: &Rc<RefCell<MetricSet<UsereventsReceiverMetrics>>>,
    builder: &mut ArrowRecordsBuilder,
) {
    let dropped = u64::from(builder.len());
    if dropped == 0 {
        return;
    }

    *builder = ArrowRecordsBuilder::new();
    metrics.borrow_mut().dropped_memory_pressure.add(dropped);
}

async fn flush_batch(
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &Rc<RefCell<MetricSet<UsereventsReceiverMetrics>>>,
    builder: &mut ArrowRecordsBuilder,
    overflow_mode: &OverflowMode,
) -> Result<(), Error> {
    if builder.is_empty() {
        return Ok(());
    }

    let payload = std::mem::take(builder)
        .build()
        .map_err(|error| Error::ReceiverError {
            receiver: effect_handler.receiver_id(),
            kind: ReceiverErrorKind::Transport,
            error: "failed to build userevents Arrow batch".to_owned(),
            source_detail: format_error_sources(&error),
        })?;

    let item_count = payload.num_items() as u64;
    let pdata = OtapPdata::new_todo_context(payload.into());
    match effect_handler.try_send_message_with_source_node(pdata) {
        Ok(()) => {
            let mut guard = metrics.borrow_mut();
            guard.forwarded_samples.add(item_count);
            guard.flushed_batches.inc();
            Ok(())
        }
        Err(otap_df_engine::error::TypedError::ChannelSendError(
            otap_df_channel::error::SendError::Full(_),
        )) => {
            match overflow_mode {
                OverflowMode::Drop => {
                    metrics.borrow_mut().dropped_downstream_full.add(item_count);
                }
            }
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

async fn process_drained_records(
    effect_handler: &local::EffectHandler<OtapPdata>,
    subscriptions: &[SubscriptionConfig],
    admission_state: &LocalReceiverAdmissionState,
    metrics: &Rc<RefCell<MetricSet<UsereventsReceiverMetrics>>>,
    builder: &mut ArrowRecordsBuilder,
    drained_records: &mut Vec<RawUsereventsRecord>,
    drain_stats: SessionDrainStats,
    batch_cfg: &BatchConfig,
    overflow_mode: &OverflowMode,
) -> Result<(), Error> {
    let received_samples = drain_stats.received_samples;
    let mut dropped_memory_pressure: u64 = 0;
    let mut dropped_no_subscription = drain_stats.dropped_no_subscription;

    for raw in drained_records.drain(..) {
        if admission_state.should_shed_ingress() {
            dropped_memory_pressure = dropped_memory_pressure.saturating_add(1);
            continue;
        }

        let Some(subscription) = subscriptions.get(raw.subscription_index) else {
            dropped_no_subscription = dropped_no_subscription.saturating_add(1);
            continue;
        };

        let decoded = DecodedUsereventsRecord::from_raw(
            subscription.tracepoint.as_str(),
            raw,
            &subscription.format,
        );
        builder.append(decoded);
        if builder.len() >= batch_cfg.max_size {
            flush_batch(effect_handler, metrics, builder, overflow_mode).await?;
        }
    }

    let mut metrics = metrics.borrow_mut();
    if drain_stats.lost_samples > 0 {
        metrics.lost_perf_samples.add(drain_stats.lost_samples);
    }
    if drain_stats.dropped_pending_overflow > 0 {
        metrics
            .dropped_pending_overflow
            .add(drain_stats.dropped_pending_overflow);
    }
    if received_samples > 0 {
        metrics.received_samples.add(received_samples);
    }
    if dropped_memory_pressure > 0 {
        metrics.dropped_memory_pressure.add(dropped_memory_pressure);
    }
    if dropped_no_subscription > 0 {
        metrics.dropped_no_subscription.add(dropped_no_subscription);
    }

    Ok(())
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Declares the Linux userevents receiver as a local receiver factory.
pub static USEREVENTS_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: USEREVENTS_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            UsereventsReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<UsereventsReceiverConfig>,
};

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for UsereventsReceiver {
    async fn start(
        self: Box<Self>,
        _ctrl_chan: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = self;
            let _ = _ctrl_chan;
            return Err(Error::ReceiverError {
                receiver: effect_handler.receiver_id(),
                kind: ReceiverErrorKind::Configuration,
                error: "userevents receiver is supported only on Linux".to_owned(),
                source_detail: String::new(),
            });
        }

        #[cfg(target_os = "linux")]
        {
            let mut ctrl_chan = _ctrl_chan;
            let telemetry_timer_handle = effect_handler
                .start_periodic_telemetry(Duration::from_secs(1))
                .await?;
            let batch_cfg = self.batching.clone();
            let session_cfg = self.session.clone();
            let drain_cfg = self.drain.clone();
            let overflow_mode = self.overflow.on_downstream_full.clone();

            let mut flush_interval = time::interval(batch_cfg.max_duration);
            flush_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            let retry_period =
                Duration::from_millis(session_cfg.late_registration.poll_interval_ms);
            let mut retry_interval = time::interval(retry_period.max(Duration::from_millis(1)));
            retry_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            let mut session: Option<UsereventsSession> = None;
            let mut builder = ArrowRecordsBuilder::new();
            let mut drained_records = Vec::with_capacity(drain_cfg.max_records_per_turn);
            let node_name = effect_handler.receiver_id().name.as_ref().to_owned();
            otel_info!(
                "userevents_receiver.start",
                node = node_name.as_str(),
                pipeline_core = self.cpu_id,
                message = "Linux userevents receiver started"
            );

            loop {
                tokio::select! {
                    biased;

                    ctrl = ctrl_chan.recv() => {
                        match ctrl {
                            Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                                let mut metrics = self.metrics.borrow_mut();
                                let _ = metrics_reporter.report(&mut metrics);
                            }
                            Ok(NodeControlMsg::MemoryPressureChanged { update }) => {
                                self.admission_state.apply(update);
                            }
                            Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                                let _ = telemetry_timer_handle.cancel().await;
                                if let Some(session) = session.as_mut() {
                                    if Instant::now() < deadline {
                                        let drain_stats = session
                                            .drain_once(&drain_cfg, &mut drained_records)
                                            .map_err(|error| Error::ReceiverError {
                                                receiver: effect_handler.receiver_id(),
                                                kind: ReceiverErrorKind::Transport,
                                                error: "failed to drain Linux userevents perf ring during ingress drain".to_owned(),
                                                source_detail: format_error_sources(&error),
                                            })?;

                                        process_drained_records(
                                            &effect_handler,
                                            &self.subscriptions,
                                            &self.admission_state,
                                            &self.metrics,
                                            &mut builder,
                                            &mut drained_records,
                                            drain_stats,
                                            &batch_cfg,
                                            &overflow_mode,
                                        )
                                        .await?;
                                    }
                                }
                                if self.admission_state.should_shed_ingress() {
                                    drop_batch(&self.metrics, &mut builder);
                                } else {
                                    flush_batch(&effect_handler, &self.metrics, &mut builder, &overflow_mode).await?;
                                }
                                effect_handler.notify_receiver_drained().await?;
                                let snapshot = self.metrics.borrow().snapshot();
                                return Ok(TerminalState::new(deadline, [snapshot]));
                            }
                            Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                                let _ = telemetry_timer_handle.cancel().await;
                                let snapshot = self.metrics.borrow().snapshot();
                                return Ok(TerminalState::new(deadline, [snapshot]));
                            }
                            Err(error) => return Err(Error::ChannelRecvError(error)),
                            _ => {}
                        }
                    }

                    _ = flush_interval.tick() => {
                        if self.admission_state.should_shed_ingress() {
                            drop_batch(&self.metrics, &mut builder);
                        } else {
                            flush_batch(&effect_handler, &self.metrics, &mut builder, &overflow_mode).await?;
                        }
                    }

                    _ = retry_interval.tick(), if session.is_none() && session_cfg.late_registration.enabled => {
                        self.metrics.borrow_mut().late_registration_retries.inc();
                        match UsereventsSession::open(&self.subscriptions, &session_cfg, self.cpu_id) {
                            Ok(opened) => {
                                self.metrics.borrow_mut().sessions_started.inc();
                                otel_info!(
                                    "userevents_receiver.session_opened",
                                    node = node_name.as_str(),
                                    subscriptions = opened.subscription_count(),
                                    message = "Opened Linux userevents perf session"
                                );
                                session = Some(opened);
                            }
                            Err(SessionInitError::MissingTracepoint(tracepoint)) => {
                                otel_warn!(
                                    "userevents_receiver.tracepoint_pending",
                                    node = node_name.as_str(),
                                    tracepoint = tracepoint.as_str(),
                                    message = "Waiting for late tracepoint registration"
                                );
                            }
                            Err(error) => {
                                let source_detail = {
                                    let nested = format_error_sources(&error);
                                    if nested.is_empty() {
                                        error.to_string()
                                    } else {
                                        format!("{}: {}", error, nested)
                                    }
                                };
                                return Err(Error::ReceiverError {
                                    receiver: effect_handler.receiver_id(),
                                    kind: ReceiverErrorKind::Configuration,
                                    error: "failed to open Linux userevents perf session".to_owned(),
                                    source_detail,
                                });
                            }
                        }
                    }

                    drained = async {
                        let session = session
                            .as_mut()
                            .expect("userevents session branch is gated by is_some()");
                        session.drain_ready(&drain_cfg, &mut drained_records).await
                    }, if session.is_some() => {
                        // TODO: Reopen the session for recoverable mid-stream
                        // collection failures once one_collect exposes typed
                        // error classification. Today drain errors are reported
                        // as terminal transport failures.
                        let drain_stats = drained.map_err(|error| Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            kind: ReceiverErrorKind::Transport,
                            error: "failed to drain Linux userevents perf ring".to_owned(),
                            source_detail: format_error_sources(&error),
                        })?;

                        process_drained_records(
                            &effect_handler,
                            &self.subscriptions,
                            &self.admission_state,
                            &self.metrics,
                            &mut builder,
                            &mut drained_records,
                            drain_stats,
                            &batch_cfg,
                            &overflow_mode,
                        )
                        .await?;
                    }
                }

                if session.is_none() && !session_cfg.late_registration.enabled {
                    match UsereventsSession::open(&self.subscriptions, &session_cfg, self.cpu_id) {
                        Ok(opened) => {
                            self.metrics.borrow_mut().sessions_started.inc();
                            session = Some(opened);
                        }
                        Err(error) => {
                            return Err(Error::ReceiverError {
                                receiver: effect_handler.receiver_id(),
                                kind: ReceiverErrorKind::Configuration,
                                error: "failed to open Linux userevents perf session".to_owned(),
                                source_detail: format_error_sources(&error),
                            });
                        }
                    }
                }
            }
        }
    }
}

const fn default_per_cpu_buffer_size() -> usize {
    DEFAULT_PER_CPU_BUFFER_SIZE
}

const fn default_wakeup_watermark() -> usize {
    DEFAULT_WAKEUP_WATERMARK
}

const fn default_max_pending_events() -> usize {
    DEFAULT_MAX_PENDING_EVENTS
}

const fn default_max_pending_bytes() -> usize {
    DEFAULT_MAX_PENDING_BYTES
}

const fn default_late_registration_poll_ms() -> u64 {
    DEFAULT_LATE_REGISTRATION_POLL.as_millis() as u64
}

const fn default_max_records_per_turn() -> usize {
    DEFAULT_MAX_RECORDS_PER_TURN
}

const fn default_max_bytes_per_turn() -> usize {
    DEFAULT_MAX_BYTES_PER_TURN
}

const fn default_max_drain_ns() -> Duration {
    DEFAULT_MAX_DRAIN_NS
}

const fn default_batch_max_size() -> u16 {
    DEFAULT_BATCH_MAX_SIZE
}

const fn default_batch_max_duration() -> Duration {
    DEFAULT_BATCH_MAX_DURATION
}

const fn default_overflow_mode() -> OverflowMode {
    OverflowMode::Drop
}

#[cfg(all(test, target_os = "linux"))]
mod linux_integration_tests {
    use super::*;

    #[test]
    #[ignore = "requires a Linux runner with a pre-registered user_events tracepoint"]
    fn session_open_smoke_test_for_pre_registered_tracepoint() {
        let tracepoint = std::env::var("OTAP_DF_USEREVENTS_TEST_TRACEPOINT").expect(
            "set OTAP_DF_USEREVENTS_TEST_TRACEPOINT to a pre-registered user_events tracepoint",
        );

        let config = SessionConfig {
            per_cpu_buffer_size: default_per_cpu_buffer_size(),
            wakeup_watermark: default_wakeup_watermark(),
            max_pending_events: default_max_pending_events(),
            max_pending_bytes: default_max_pending_bytes(),
            late_registration: LateRegistrationConfig::default(),
        };
        let subscriptions = vec![SubscriptionConfig {
            tracepoint,
            format: FormatConfig::Tracefs,
        }];

        let session = UsereventsSession::open(&subscriptions, &config, 0)
            .expect("open a perf session for the registered tracepoint");
        assert_eq!(session.subscription_count(), 1);
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use std::collections::HashMap;

    use super::session::TracefsField;
    use otap_df_channel::mpsc;
    use otap_df_config::SignalType;
    use otap_df_engine::control::runtime_ctrl_msg_channel;
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::memory_limiter::{
        MemoryPressureChanged, MemoryPressureLevel, MemoryPressureState,
    };
    use otap_df_engine::message::Sender;
    use otap_df_engine::testing::{test_node, test_pipeline_ctx};
    use otap_df_pdata::OtapPayload;
    use otap_df_telemetry::reporter::MetricsReporter;

    fn test_metrics() -> Rc<RefCell<MetricSet<UsereventsReceiverMetrics>>> {
        let (pipeline_ctx, _) = test_pipeline_ctx();
        Rc::new(RefCell::new(
            pipeline_ctx.register_metrics::<UsereventsReceiverMetrics>(),
        ))
    }

    fn test_effect_handler(
        capacity: usize,
        prefill: bool,
    ) -> (local::EffectHandler<OtapPdata>, mpsc::Receiver<OtapPdata>) {
        let (tx, rx) = mpsc::Channel::new(capacity);
        if prefill {
            tx.send(OtapPdata::new_todo_context(OtapPayload::empty(
                SignalType::Logs,
            )))
            .expect("prefill downstream channel");
        }

        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx)));
        let (runtime_tx, _) = runtime_ctrl_msg_channel(4);
        let (_, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);

        (
            local::EffectHandler::new(
                test_node("userevents_receiver"),
                senders,
                None,
                runtime_tx,
                metrics_reporter,
            ),
            rx,
        )
    }

    fn test_subscription() -> SubscriptionConfig {
        SubscriptionConfig {
            tracepoint: "user_events:test".to_owned(),
            format: FormatConfig::Tracefs,
        }
    }

    fn test_batch_config(max_size: u16) -> BatchConfig {
        BatchConfig {
            max_size,
            max_duration: default_batch_max_duration(),
        }
    }

    fn test_raw_record(subscription_index: usize) -> RawUsereventsRecord {
        RawUsereventsRecord {
            subscription_index,
            timestamp_unix_nano: 123,
            process_id: None,
            thread_id: None,
            event_data: Vec::new(),
            user_data_offset: 0,
            fields: Arc::<[TracefsField]>::from(Vec::<TracefsField>::new().into_boxed_slice()),
        }
    }

    fn normal_admission_state() -> LocalReceiverAdmissionState {
        LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default())
    }

    fn hard_admission_state() -> LocalReceiverAdmissionState {
        let state = normal_admission_state();
        state.apply(MemoryPressureChanged {
            generation: 1,
            level: MemoryPressureLevel::Hard,
            retry_after_secs: 1,
            usage_bytes: 1,
        });
        state
    }

    #[tokio::test(flavor = "current_thread")]
    async fn process_drained_records_drops_records_under_memory_pressure() {
        let (effect_handler, _rx) = test_effect_handler(4, false);
        let subscriptions = vec![test_subscription()];
        let admission_state = hard_admission_state();
        let metrics = test_metrics();
        let mut builder = ArrowRecordsBuilder::new();
        let mut drained_records = vec![test_raw_record(0), test_raw_record(0)];
        let drain_stats = SessionDrainStats {
            received_samples: 2,
            dropped_no_subscription: 0,
            lost_samples: 1,
            dropped_pending_overflow: 1,
        };

        process_drained_records(
            &effect_handler,
            &subscriptions,
            &admission_state,
            &metrics,
            &mut builder,
            &mut drained_records,
            drain_stats,
            &test_batch_config(10),
            &OverflowMode::Drop,
        )
        .await
        .expect("drained records processed");

        let metrics = metrics.borrow();
        assert!(builder.is_empty());
        assert!(drained_records.is_empty());
        assert_eq!(metrics.received_samples.get(), 2);
        assert_eq!(metrics.dropped_memory_pressure.get(), 2);
        assert_eq!(metrics.lost_perf_samples.get(), 1);
        assert_eq!(metrics.dropped_pending_overflow.get(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn process_drained_records_counts_downstream_full_drop() {
        let (effect_handler, _rx) = test_effect_handler(1, true);
        let subscriptions = vec![test_subscription()];
        let admission_state = normal_admission_state();
        let metrics = test_metrics();
        let mut builder = ArrowRecordsBuilder::new();
        let mut drained_records = vec![test_raw_record(0)];
        let drain_stats = SessionDrainStats {
            received_samples: 1,
            ..Default::default()
        };

        process_drained_records(
            &effect_handler,
            &subscriptions,
            &admission_state,
            &metrics,
            &mut builder,
            &mut drained_records,
            drain_stats,
            &test_batch_config(1),
            &OverflowMode::Drop,
        )
        .await
        .expect("drained records processed");

        let metrics = metrics.borrow();
        assert!(builder.is_empty());
        assert_eq!(metrics.received_samples.get(), 1);
        assert_eq!(metrics.dropped_downstream_full.get(), 1);
        assert_eq!(metrics.forwarded_samples.get(), 0);
        assert_eq!(metrics.flushed_batches.get(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn partial_batch_flushes_after_drain_ingress_processing() {
        let (effect_handler, rx) = test_effect_handler(2, false);
        let subscriptions = vec![test_subscription()];
        let admission_state = normal_admission_state();
        let metrics = test_metrics();
        let mut builder = ArrowRecordsBuilder::new();
        let mut drained_records = vec![test_raw_record(0)];
        let drain_stats = SessionDrainStats {
            received_samples: 1,
            ..Default::default()
        };

        process_drained_records(
            &effect_handler,
            &subscriptions,
            &admission_state,
            &metrics,
            &mut builder,
            &mut drained_records,
            drain_stats,
            &test_batch_config(10),
            &OverflowMode::Drop,
        )
        .await
        .expect("drained records processed");

        assert_eq!(builder.len(), 1);
        flush_batch(&effect_handler, &metrics, &mut builder, &OverflowMode::Drop)
            .await
            .expect("partial batch flushed");

        let pdata = rx.recv().await.expect("flushed batch received");
        assert_eq!(pdata.num_items(), 1);
        let metrics = metrics.borrow();
        assert!(builder.is_empty());
        assert_eq!(metrics.received_samples.get(), 1);
        assert_eq!(metrics.forwarded_samples.get(), 1);
        assert_eq!(metrics.flushed_batches.get(), 1);
    }

    #[test]
    fn validate_subscriptions_accepts_single_subscription() {
        let config = UsereventsReceiverConfig {
            subscriptions: vec![SubscriptionConfig {
                tracepoint: "user_events:example_L5K1".to_owned(),
                format: FormatConfig::Tracefs,
            }],
            session: None,
            drain: None,
            batching: None,
            overflow: None,
        };

        UsereventsReceiver::validate_subscriptions(&config.subscriptions)
            .expect("subscriptions accepted");
    }

    #[test]
    fn deserialize_config_rejects_legacy_tracepoint_shorthand() {
        let error = serde_json::from_value::<UsereventsReceiverConfig>(serde_json::json!({
            "tracepoint": "user_events:example_L5K1"
        }))
        .expect_err("legacy shorthand rejected");

        assert!(
            error.to_string().contains("unknown field `tracepoint`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_subscriptions_rejects_empty_list() {
        let config = UsereventsReceiverConfig {
            subscriptions: Vec::new(),
            session: None,
            drain: None,
            batching: None,
            overflow: None,
        };

        let error = UsereventsReceiver::validate_subscriptions(&config.subscriptions)
            .expect_err("config rejected");
        assert!(
            error.to_string().contains("at least one subscription"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_subscriptions_rejects_non_userevents_tracepoint_group() {
        let config = UsereventsReceiverConfig {
            subscriptions: vec![SubscriptionConfig {
                tracepoint: "foo:example_L2K1".to_owned(),
                format: FormatConfig::Tracefs,
            }],
            session: None,
            drain: None,
            batching: None,
            overflow: None,
        };

        let error = UsereventsReceiver::validate_subscriptions(&config.subscriptions)
            .expect_err("config rejected");
        assert!(
            error.to_string().contains("user_events:<event>"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_subscriptions_rejects_tracepoint_without_group() {
        let config = UsereventsReceiverConfig {
            subscriptions: vec![SubscriptionConfig {
                tracepoint: "example_L2K1".to_owned(),
                format: FormatConfig::Tracefs,
            }],
            session: None,
            drain: None,
            batching: None,
            overflow: None,
        };

        let error = UsereventsReceiver::validate_subscriptions(&config.subscriptions)
            .expect_err("config rejected");
        assert!(
            error.to_string().contains("user_events:<event>"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_session_rejects_zero_max_pending_events() {
        let session = SessionConfig {
            per_cpu_buffer_size: default_per_cpu_buffer_size(),
            wakeup_watermark: default_wakeup_watermark(),
            max_pending_events: 0,
            max_pending_bytes: default_max_pending_bytes(),
            late_registration: LateRegistrationConfig::default(),
        };
        let error = UsereventsReceiver::validate_session(&session).expect_err("zero rejected");
        assert!(
            error.to_string().contains("max_pending_events"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_session_rejects_zero_max_pending_bytes() {
        let session = SessionConfig {
            per_cpu_buffer_size: default_per_cpu_buffer_size(),
            wakeup_watermark: default_wakeup_watermark(),
            max_pending_events: default_max_pending_events(),
            max_pending_bytes: 0,
            late_registration: LateRegistrationConfig::default(),
        };
        let error = UsereventsReceiver::validate_session(&session).expect_err("zero rejected");
        assert!(
            error.to_string().contains("max_pending_bytes"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_drain_rejects_zero_max_drain_ns() {
        let drain = DrainConfig {
            max_records_per_turn: default_max_records_per_turn(),
            max_bytes_per_turn: default_max_bytes_per_turn(),
            max_drain_ns: Duration::ZERO,
        };
        let error = UsereventsReceiver::validate_drain(&drain).expect_err("zero rejected");
        assert!(
            error.to_string().contains("max_drain_ns"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_drain_rejects_zero_max_records_per_turn() {
        let drain = DrainConfig {
            max_records_per_turn: 0,
            max_bytes_per_turn: default_max_bytes_per_turn(),
            max_drain_ns: default_max_drain_ns(),
        };
        let error = UsereventsReceiver::validate_drain(&drain).expect_err("zero rejected");
        assert!(
            error.to_string().contains("max_records_per_turn"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_drain_rejects_zero_max_bytes_per_turn() {
        let drain = DrainConfig {
            max_records_per_turn: default_max_records_per_turn(),
            max_bytes_per_turn: 0,
            max_drain_ns: default_max_drain_ns(),
        };
        let error = UsereventsReceiver::validate_drain(&drain).expect_err("zero rejected");
        assert!(
            error.to_string().contains("max_bytes_per_turn"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_drain_accepts_nonzero_values() {
        let drain = DrainConfig {
            max_records_per_turn: default_max_records_per_turn(),
            max_bytes_per_turn: default_max_bytes_per_turn(),
            max_drain_ns: Duration::from_millis(2),
        };
        UsereventsReceiver::validate_drain(&drain).expect("non-zero accepted");
    }

    #[test]
    fn validate_batching_rejects_zero_max_duration() {
        let batching = BatchConfig {
            max_size: default_batch_max_size(),
            max_duration: Duration::ZERO,
        };
        let error = UsereventsReceiver::validate_batching(&batching).expect_err("zero rejected");
        assert!(
            error.to_string().contains("max_duration"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_batching_accepts_nonzero_max_duration() {
        let batching = BatchConfig {
            max_size: default_batch_max_size(),
            max_duration: default_batch_max_duration(),
        };
        UsereventsReceiver::validate_batching(&batching).expect("non-zero accepted");
    }
}
