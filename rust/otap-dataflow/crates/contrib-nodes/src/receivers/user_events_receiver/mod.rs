// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux user_events receiver.

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
use self::decoder::DecodedUserEventsRecord;
use self::metrics::UserEventsReceiverMetrics;
use self::session::SessionInitError;
use self::session::{RawUserEventsRecord, SessionDrainStats, UserEventsSession};
use otap_df_engine::control::NodeControlMsg;
use otap_df_telemetry::{otel_info, otel_warn};
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

/// URN for the Linux user_events receiver.
///
/// The receiver identity is vendor-neutral: it collects Linux `user_events`
/// and structurally decodes records into OTAP logs.
pub const USER_EVENTS_RECEIVER_URN: &str = "urn:otel:receiver:user_events";

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
    #[cfg(feature = "user_events-eventheader")]
    EventHeader,
}

/// One user_events tracepoint subscription.
///
/// Each subscription opens the named tracepoint and chooses the payload decoder
/// used for samples read from that tracepoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubscriptionConfig {
    /// user_events tracepoint name, with or without the `user_events:` prefix.
    tracepoint: String,
    /// Payload decoding format for records emitted by this tracepoint.
    #[serde(default)]
    format: FormatConfig,
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
    /// Optional retry interval for tracepoints that may be registered after
    /// startup. When absent, missing tracepoints fail startup immediately.
    #[serde(default, with = "humantime_serde::option")]
    late_registration_poll_interval: Option<Duration>,
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

/// User-supplied configuration for the Linux user_events receiver.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct UserEventsReceiverConfig {
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
}

/// Runtime state for one local user_events receiver task.
struct UserEventsReceiver {
    /// Tracepoint subscriptions owned by this receiver instance.
    subscriptions: Vec<SubscriptionConfig>,
    /// Low-level session configuration used when opening tracepoint readers.
    session: SessionConfig,
    /// Per-turn drain limits used to keep receiver work cooperative.
    drain: DrainConfig,
    /// In-memory batching policy for decoded log records.
    batching: BatchConfig,
    /// Local pipeline CPU/shard assigned by the engine.
    cpu_id: usize,
    /// Receiver metric set shared with the local pipeline task.
    metrics: Rc<RefCell<MetricSet<UserEventsReceiverMetrics>>>,
    /// Local admission state used to coordinate with memory limiting.
    admission_state: LocalReceiverAdmissionState,
}

impl UserEventsReceiver {
    fn from_config(
        pipeline: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let mut config: UserEventsReceiverConfig =
            serde_json::from_value(config.clone()).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?;

        Self::normalize_subscriptions(&mut config.subscriptions)?;
        let session = config.session.clone().unwrap_or(SessionConfig {
            per_cpu_buffer_size: default_per_cpu_buffer_size(),
            wakeup_watermark: default_wakeup_watermark(),
            max_pending_events: default_max_pending_events(),
            max_pending_bytes: default_max_pending_bytes(),
            late_registration_poll_interval: None,
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

        Ok(Self {
            subscriptions: config.subscriptions,
            session,
            drain,
            batching,
            cpu_id: pipeline.core_id(),
            metrics: Rc::new(RefCell::new(
                pipeline.register_metrics::<UserEventsReceiverMetrics>(),
            )),
            admission_state: LocalReceiverAdmissionState::from_process_state(
                &pipeline.memory_pressure_state(),
            ),
        })
    }

    fn normalize_subscriptions(
        subscriptions: &mut [SubscriptionConfig],
    ) -> Result<(), otap_df_config::error::Error> {
        if subscriptions.is_empty() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "user_events receiver requires at least one subscription".to_owned(),
            });
        }
        for subscription in subscriptions {
            subscription.tracepoint = Self::normalize_tracepoint(&subscription.tracepoint)?;
        }
        Ok(())
    }

    fn normalize_tracepoint(tracepoint: &str) -> Result<String, otap_df_config::error::Error> {
        let tracepoint = tracepoint.trim();
        if tracepoint.is_empty() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "user_events receiver tracepoint must not be empty".to_owned(),
            });
        }
        if let Some((group, event_name)) = tracepoint.split_once(':') {
            if group == "user_events" && !event_name.is_empty() {
                return Ok(tracepoint.to_owned());
            }
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "user_events receiver tracepoint `{tracepoint}` must be an event name or `user_events:<event>`"
                ),
            });
        }
        Ok(format!("user_events:{tracepoint}"))
    }

    fn validate_session(session: &SessionConfig) -> Result<(), otap_df_config::error::Error> {
        if session.max_pending_events == 0 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error:
                    "user_events receiver `session.max_pending_events` must be greater than zero"
                        .to_owned(),
            });
        }
        if session.max_pending_bytes == 0 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "user_events receiver `session.max_pending_bytes` must be greater than zero"
                    .to_owned(),
            });
        }
        if session
            .late_registration_poll_interval
            .is_some_and(|poll_interval| poll_interval.is_zero())
        {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "user_events receiver `session.late_registration_poll_interval` must be greater than zero"
                    .to_owned(),
            });
        }
        Ok(())
    }

    fn validate_drain(drain: &DrainConfig) -> Result<(), otap_df_config::error::Error> {
        if drain.max_records_per_turn == 0 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error:
                    "user_events receiver `drain.max_records_per_turn` must be greater than zero"
                        .to_owned(),
            });
        }
        if drain.max_bytes_per_turn == 0 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "user_events receiver `drain.max_bytes_per_turn` must be greater than zero"
                    .to_owned(),
            });
        }
        if drain.max_drain_ns.is_zero() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "user_events receiver `drain.max_drain_ns` must be greater than zero; \
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
                error: "user_events receiver `batching.max_duration` must be greater than zero"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

fn drop_batch(
    metrics: &Rc<RefCell<MetricSet<UserEventsReceiverMetrics>>>,
    builder: &mut ArrowRecordsBuilder,
) {
    let dropped = u64::from(builder.len());
    if dropped == 0 {
        return;
    }

    *builder = ArrowRecordsBuilder::new();
    metrics.borrow_mut().dropped_memory_pressure.add(dropped);
}

fn add_dropped_send_error(
    metrics: &Rc<RefCell<MetricSet<UserEventsReceiverMetrics>>>,
    dropped: u64,
) {
    if dropped > 0 {
        metrics.borrow_mut().dropped_send_error.add(dropped);
    }
}

async fn flush_batch(
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &Rc<RefCell<MetricSet<UserEventsReceiverMetrics>>>,
    builder: &mut ArrowRecordsBuilder,
) -> Result<(), Error> {
    if builder.is_empty() {
        return Ok(());
    }

    let payload = std::mem::take(builder)
        .build()
        .map_err(|error| Error::ReceiverError {
            receiver: effect_handler.receiver_id(),
            kind: ReceiverErrorKind::Transport,
            error: "failed to build user_events Arrow batch".to_owned(),
            source_detail: format_error_sources(&error),
        })?;

    let item_count = payload.num_items() as u64;
    let pdata = OtapPdata::new_todo_context(payload.into());
    let send_started = Instant::now();
    // Await the send after a select! branch has already won; do not race this
    // future inside select!, where cancellation could drop a ready batch.
    if let Err(error) = effect_handler.send_message_with_source_node(pdata).await {
        add_dropped_send_error(metrics, item_count);
        return Err(error.into());
    }
    let send_elapsed_ns = send_started.elapsed().as_nanos();
    // Practically unreachable, but keep the metric saturated instead of
    // failing the receiver if the elapsed duration ever exceeds u64::MAX ns.
    let send_elapsed_ns = u64::try_from(send_elapsed_ns).unwrap_or(u64::MAX);

    let mut guard = metrics.borrow_mut();
    if send_elapsed_ns > 0 {
        guard.downstream_send_blocked_ns.add(send_elapsed_ns);
    }
    guard.forwarded_samples.add(item_count);
    guard.flushed_batches.inc();
    Ok(())
}

async fn process_drained_records(
    effect_handler: &local::EffectHandler<OtapPdata>,
    subscriptions: &[SubscriptionConfig],
    admission_state: &LocalReceiverAdmissionState,
    metrics: &Rc<RefCell<MetricSet<UserEventsReceiverMetrics>>>,
    builder: &mut ArrowRecordsBuilder,
    drained_records: &mut Vec<RawUserEventsRecord>,
    drain_stats: SessionDrainStats,
    batch_cfg: &BatchConfig,
) -> Result<(), Error> {
    let received_samples = drain_stats.received_samples;
    let mut dropped_memory_pressure: u64 = 0;
    let mut dropped_no_subscription = drain_stats.dropped_no_subscription;

    let mut drained = drained_records.drain(..);
    while let Some(raw) = drained.next() {
        if admission_state.should_shed_ingress() {
            dropped_memory_pressure = dropped_memory_pressure.saturating_add(1);
            continue;
        }

        let Some(subscription) = subscriptions.get(raw.subscription_index) else {
            dropped_no_subscription = dropped_no_subscription.saturating_add(1);
            continue;
        };

        let decoded = DecodedUserEventsRecord::from_raw(
            subscription.tracepoint.as_str(),
            raw,
            &subscription.format,
        );
        builder.append(decoded);
        if builder.len() >= batch_cfg.max_size {
            if let Err(error) = flush_batch(effect_handler, metrics, builder).await {
                let remaining = u64::try_from(drained.count()).unwrap_or(u64::MAX);
                add_dropped_send_error(metrics, remaining);
                return Err(error);
            }
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
/// Declares the Linux user_events receiver as a local receiver factory.
pub static USER_EVENTS_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: USER_EVENTS_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            UserEventsReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<UserEventsReceiverConfig>,
};

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for UserEventsReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_chan: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let telemetry_timer_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;
        let batch_cfg = self.batching.clone();
        let session_cfg = self.session.clone();
        let drain_cfg = self.drain.clone();

        let mut flush_interval = time::interval(batch_cfg.max_duration);
        flush_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let late_registration_enabled = session_cfg.late_registration_poll_interval.is_some();
        let retry_period = session_cfg
            .late_registration_poll_interval
            .unwrap_or(DEFAULT_LATE_REGISTRATION_POLL);
        let mut retry_interval = time::interval(retry_period.max(Duration::from_millis(1)));
        retry_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut session: Option<UserEventsSession> = None;
        let mut builder = ArrowRecordsBuilder::new();
        let mut drained_records = Vec::with_capacity(drain_cfg.max_records_per_turn);
        let node_name = effect_handler.receiver_id().name.as_ref().to_owned();
        otel_info!(
            "user_events_receiver.start",
            node = node_name.as_str(),
            pipeline_core = self.cpu_id,
            message = "Linux user_events receiver started"
        );

        loop {
            tokio::select! {
                // Control traffic is expected to be occasional, not per-drain
                // or per-record. Prioritize it so shutdown, memory pressure,
                // and telemetry requests are handled promptly on the
                // current-thread runtime.
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
                                            error: "failed to drain Linux user_events perf ring during ingress drain".to_owned(),
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
                                    )
                                    .await?;
                                }
                            }
                            if self.admission_state.should_shed_ingress() {
                                drop_batch(&self.metrics, &mut builder);
                            } else {
                                flush_batch(&effect_handler, &self.metrics, &mut builder).await?;
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
                        flush_batch(&effect_handler, &self.metrics, &mut builder).await?;
                    }
                }

                _ = retry_interval.tick(), if session.is_none() && late_registration_enabled => {
                    self.metrics.borrow_mut().late_registration_retries.inc();
                    match UserEventsSession::open(&self.subscriptions, &session_cfg, self.cpu_id) {
                        Ok(opened) => {
                            self.metrics.borrow_mut().sessions_started.inc();
                            otel_info!(
                                "user_events_receiver.session_opened",
                                node = node_name.as_str(),
                                subscriptions = opened.subscription_count(),
                                message = "Opened Linux user_events perf session"
                            );
                            session = Some(opened);
                        }
                        Err(SessionInitError::MissingTracepoint(tracepoint)) => {
                            otel_warn!(
                                "user_events_receiver.tracepoint_pending",
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
                                error: "failed to open Linux user_events perf session".to_owned(),
                                source_detail,
                            });
                        }
                    }
                }

                drained = async {
                    let Some(session) = session.as_mut() else {
                        return Err(std::io::Error::other(
                            "user_events session branch selected without an active session",
                        ));
                    };
                    session.drain_ready(&drain_cfg, &mut drained_records).await
                }, if session.is_some() => {
                    // TODO: Reopen the session for recoverable mid-stream
                    // collection failures once one_collect exposes typed
                    // error classification. Today drain errors are reported
                    // as terminal transport failures.
                    let drain_stats = drained.map_err(|error| Error::ReceiverError {
                        receiver: effect_handler.receiver_id(),
                        kind: ReceiverErrorKind::Transport,
                        error: "failed to drain Linux user_events perf ring".to_owned(),
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
                    )
                    .await?;
                }
            }

            if session.is_none() && !late_registration_enabled {
                match UserEventsSession::open(&self.subscriptions, &session_cfg, self.cpu_id) {
                    Ok(opened) => {
                        self.metrics.borrow_mut().sessions_started.inc();
                        session = Some(opened);
                    }
                    Err(error) => {
                        return Err(Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            kind: ReceiverErrorKind::Configuration,
                            error: "failed to open Linux user_events perf session".to_owned(),
                            source_detail: format_error_sources(&error),
                        });
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

#[cfg(all(test, target_os = "linux"))]
#[allow(unsafe_code)]
mod linux_integration_tests {
    use super::*;
    use std::ffi::CString;
    use std::io;
    use std::time::Duration;

    #[cfg(feature = "user_events-eventheader")]
    use eventheader_dynamic::{EventBuilder, FieldFormat, Level, Provider};
    use tokio::time;

    const RUN_USEREVENTS_E2E_ENV: &str = "OTAP_DF_RUN_USEREVENTS_E2E";

    fn fail_user_events_e2e(reason: impl std::fmt::Display) -> ! {
        panic!("user_events e2e smoke test failed: {reason}");
    }

    const fn user_events_unavailable_errno(errno: i32) -> bool {
        matches!(errno, 1 | 2 | 13 | 95)
    }

    fn test_session_config() -> SessionConfig {
        SessionConfig {
            per_cpu_buffer_size: default_per_cpu_buffer_size(),
            wakeup_watermark: default_wakeup_watermark(),
            max_pending_events: default_max_pending_events(),
            max_pending_bytes: default_max_pending_bytes(),
            late_registration_poll_interval: None,
        }
    }

    fn test_drain_config() -> DrainConfig {
        DrainConfig {
            max_records_per_turn: default_max_records_per_turn(),
            max_bytes_per_turn: default_max_bytes_per_turn(),
            max_drain_ns: default_max_drain_ns(),
        }
    }

    fn open_session_or_skip(tracepoint: &str, format: FormatConfig) -> Option<UserEventsSession> {
        let subscriptions = vec![SubscriptionConfig {
            tracepoint: tracepoint.to_owned(),
            format,
        }];

        match UserEventsSession::open(&subscriptions, &test_session_config(), 0) {
            Ok(session) => Some(session),
            Err(SessionInitError::MissingTracepoint(tracepoint)) => {
                fail_user_events_e2e(format!(
                    "registered tracepoint `{tracepoint}` was not visible to the receiver"
                ));
            }
            Err(SessionInitError::Io(error))
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound
                        | io::ErrorKind::PermissionDenied
                        | io::ErrorKind::Unsupported
                        | io::ErrorKind::Other
                ) =>
            {
                None
            }
            Err(error) => panic!("failed to open user_events receiver session: {error}"),
        }
    }

    async fn write_tracefs_sample(tracepoint_state: &tracepoint::TracepointState) -> bool {
        for _ in 0..20 {
            if tracepoint_state.enabled() {
                break;
            }
            time::sleep(Duration::from_millis(10)).await;
        }

        if !tracepoint_state.enabled() {
            fail_user_events_e2e(
                "tracefs tracepoint did not become enabled after opening receiver session",
            );
        }

        let ci_answer = 42u32;
        let ci_message = *b"hello-from-ci\0";
        let mut data = [
            tracepoint::EventDataDescriptor::zero(),
            tracepoint::EventDataDescriptor::from_value(&ci_answer),
            tracepoint::EventDataDescriptor::from_bytes(&ci_message),
        ];
        let write_result = tracepoint_state.write(&mut data);
        if write_result != 0 {
            fail_user_events_e2e(format!("tracefs write returned errno {write_result}"));
        }

        true
    }

    #[cfg(feature = "user_events-eventheader")]
    async fn write_eventheader_sample(event_set: &eventheader_dynamic::EventSet) -> bool {
        for _ in 0..20 {
            if event_set.enabled() {
                break;
            }
            time::sleep(Duration::from_millis(10)).await;
        }

        if !event_set.enabled() {
            fail_user_events_e2e(
                "EventHeader tracepoint did not become enabled after opening receiver session",
            );
        }

        let write_result = EventBuilder::new()
            .reset("CiSmoke", 0)
            .add_str("ci_message", b"hello-from-ci", FieldFormat::Default, 0)
            .add_value("ci_answer", 42u32, FieldFormat::UnsignedInt, 0)
            .write(event_set, None, None);
        if write_result != 0 {
            fail_user_events_e2e(format!("EventHeader write returned errno {write_result}"));
        }

        true
    }

    #[tokio::test(flavor = "current_thread")]
    async fn user_events_linux_e2e_smoke_when_available() {
        if std::env::var_os(RUN_USEREVENTS_E2E_ENV).is_none() {
            return;
        }

        // The smoke test opens the perf session on CPU 0, but
        // `tokio::test(flavor = "current_thread")` does not pin the test thread
        // to a specific CPU. If the OS schedules the writer thread off CPU 0
        // during `tracepoint_state.write(...)`, the sample lands in a different
        // ring than the receiver is watching and `drain_ready` times out. Pin
        // the test thread to CPU 0 to keep the writer and reader on the same
        // ring buffer.
        if !core_affinity::set_for_current(core_affinity::CoreId { id: 0 }) {
            eprintln!("user_events smoke skipped: could not pin to CPU 0");
            return;
        }

        let tracefs_event_name = format!("otap_df_tracefs_ci_{}", std::process::id());
        let tracefs_tracepoint = format!("user_events:{tracefs_event_name}");
        let tracefs_definition = CString::new(format!(
            "{tracefs_event_name} u32 ci_answer; char ci_message[14]"
        ))
        .expect("tracefs definition should not contain interior NUL bytes");
        let tracefs_state = Box::pin(tracepoint::TracepointState::new(0));
        let tracefs_register_errno =
            unsafe { tracefs_state.as_ref().register(&tracefs_definition) };
        if tracefs_register_errno != 0 {
            if user_events_unavailable_errno(tracefs_register_errno) {
                return;
            }
            fail_user_events_e2e(format!(
                "tracefs registration returned errno {tracefs_register_errno}"
            ));
        }

        let mut tracefs_session =
            match open_session_or_skip(&tracefs_tracepoint, FormatConfig::Tracefs) {
                Some(session) => session,
                None => return,
            };
        assert_eq!(tracefs_session.subscription_count(), 1);
        if !write_tracefs_sample(&tracefs_state).await {
            return;
        }

        let mut tracefs_records = Vec::new();
        let drain_config = test_drain_config();
        let tracefs_stats = time::timeout(
            Duration::from_secs(2),
            tracefs_session.drain_ready(&drain_config, &mut tracefs_records),
        )
        .await
        .expect("timed out draining tracefs sample")
        .expect("drain tracefs sample");
        assert_eq!(tracefs_stats.dropped_no_subscription, 0);

        let decoded_tracefs = tracefs_records
            .into_iter()
            .map(|record| {
                DecodedUserEventsRecord::from_raw(
                    &tracefs_tracepoint,
                    record,
                    &FormatConfig::Tracefs,
                )
            })
            .find(|record| {
                let has_answer = record.attributes.iter().any(|(key, value)| {
                    key.as_ref() == "ci_answer"
                        && matches!(value, decoder::DecodedAttrValue::Int(42))
                });
                let has_message = record.attributes.iter().any(|(key, value)| {
                    key.as_ref() == "ci_message"
                        && matches!(
                            value,
                            decoder::DecodedAttrValue::Str(value) if value == "hello-from-ci"
                        )
                });
                has_answer && has_message
            });

        assert!(
            decoded_tracefs.is_some(),
            "tracefs session should decode the emitted ci_answer and ci_message fields"
        );

        #[cfg(feature = "user_events-eventheader")]
        {
            let provider_name = format!("otap_df_ci_{}", std::process::id());
            let tracepoint = format!("user_events:{provider_name}_L4K1");
            let mut provider = Provider::new(&provider_name, &Provider::new_options());
            let event_set = provider.register_set(Level::Informational, 1);
            if event_set.errno() != 0 {
                if user_events_unavailable_errno(event_set.errno()) {
                    return;
                }
                fail_user_events_e2e(format!(
                    "EventHeader registration returned errno {}",
                    event_set.errno()
                ));
            }

            let mut eventheader_tracefs_session =
                match open_session_or_skip(&tracepoint, FormatConfig::Tracefs) {
                    Some(session) => session,
                    None => return,
                };
            assert_eq!(eventheader_tracefs_session.subscription_count(), 1);
            if !write_eventheader_sample(&event_set).await {
                return;
            }

            let mut eventheader_tracefs_records = Vec::new();
            let drain_config = test_drain_config();
            let eventheader_tracefs_stats = time::timeout(
                Duration::from_secs(2),
                eventheader_tracefs_session
                    .drain_ready(&drain_config, &mut eventheader_tracefs_records),
            )
            .await
            .expect("timed out draining EventHeader tracefs sample")
            .expect("drain EventHeader tracefs sample");
            assert_eq!(eventheader_tracefs_stats.dropped_no_subscription, 0);
            assert!(
                !eventheader_tracefs_records.is_empty(),
                "tracefs session should collect at least one EventHeader sample"
            );

            let mut eventheader_session =
                match open_session_or_skip(&tracepoint, FormatConfig::EventHeader) {
                    Some(session) => session,
                    None => return,
                };
            assert_eq!(eventheader_session.subscription_count(), 1);
            if !write_eventheader_sample(&event_set).await {
                return;
            }

            let mut eventheader_records = Vec::new();
            let drain_config = test_drain_config();
            let eventheader_stats = time::timeout(
                Duration::from_secs(2),
                eventheader_session.drain_ready(&drain_config, &mut eventheader_records),
            )
            .await
            .expect("timed out draining EventHeader sample")
            .expect("drain EventHeader sample");
            assert_eq!(eventheader_stats.dropped_no_subscription, 0);

            let decoded = eventheader_records
                .into_iter()
                .map(|record| {
                    DecodedUserEventsRecord::from_raw(
                        &tracepoint,
                        record,
                        &FormatConfig::EventHeader,
                    )
                })
                .find(|record| {
                    record.attributes.iter().any(|(key, value)| {
                        key.as_ref() == "ci_message"
                            && matches!(
                                value,
                                decoder::DecodedAttrValue::Str(value) if value == "hello-from-ci"
                            )
                    })
                });

            assert!(
                decoded.is_some(),
                "EventHeader session should decode the emitted ci_message field"
            );
        }
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

    fn test_metrics() -> Rc<RefCell<MetricSet<UserEventsReceiverMetrics>>> {
        let (pipeline_ctx, _) = test_pipeline_ctx();
        Rc::new(RefCell::new(
            pipeline_ctx.register_metrics::<UserEventsReceiverMetrics>(),
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
                test_node("user_events_receiver"),
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

    fn test_raw_record(subscription_index: usize) -> RawUserEventsRecord {
        RawUserEventsRecord {
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
    async fn flush_batch_waits_for_downstream_capacity() {
        let (effect_handler, rx) = test_effect_handler(1, true);
        let metrics = test_metrics();
        let mut builder = ArrowRecordsBuilder::new();
        builder.append(DecodedUserEventsRecord::from_raw(
            "user_events:test",
            test_raw_record(0),
            &FormatConfig::Tracefs,
        ));

        {
            let flush = flush_batch(&effect_handler, &metrics, &mut builder);
            tokio::pin!(flush);

            tokio::select! {
                result = &mut flush => {
                    panic!("flush completed before downstream capacity was available: {result:?}");
                }
                _ = time::sleep(Duration::from_millis(10)) => {}
            }

            let prefilled = rx.recv().await.expect("prefilled item received");
            assert_eq!(prefilled.num_items(), 0);
            flush.await.expect("flush completed after capacity opened");
        }

        let pdata = rx.recv().await.expect("flushed batch received");
        assert_eq!(pdata.num_items(), 1);

        let metrics = metrics.borrow();
        assert!(builder.is_empty());
        assert_eq!(metrics.forwarded_samples.get(), 1);
        assert_eq!(metrics.flushed_batches.get(), 1);
        assert!(metrics.downstream_send_blocked_ns.get() > 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn flush_batch_unblocks_when_downstream_closes() {
        let (effect_handler, rx) = test_effect_handler(1, true);
        let metrics = test_metrics();
        let mut builder = ArrowRecordsBuilder::new();
        builder.append(DecodedUserEventsRecord::from_raw(
            "user_events:test",
            test_raw_record(0),
            &FormatConfig::Tracefs,
        ));

        {
            let flush = flush_batch(&effect_handler, &metrics, &mut builder);
            tokio::pin!(flush);

            tokio::select! {
                result = &mut flush => {
                    panic!("flush completed before downstream closed: {result:?}");
                }
                _ = time::sleep(Duration::from_millis(10)) => {}
            }

            drop(rx);
            let error = flush
                .await
                .expect_err("closed downstream should fail the flush");
            assert!(
                matches!(error, Error::ChannelSendError { closed: true, .. }),
                "unexpected error: {error}"
            );
        }

        let metrics = metrics.borrow();
        assert_eq!(metrics.forwarded_samples.get(), 0);
        assert_eq!(metrics.flushed_batches.get(), 0);
        assert_eq!(metrics.dropped_send_error.get(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn process_drained_records_flushes_with_downstream_capacity() {
        let (effect_handler, rx) = test_effect_handler(1, false);
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
        )
        .await
        .expect("drained records processed");

        let pdata = rx.recv().await.expect("flushed batch received");
        assert_eq!(pdata.num_items(), 1);
        let metrics = metrics.borrow();
        assert!(builder.is_empty());
        assert_eq!(metrics.received_samples.get(), 1);
        assert_eq!(metrics.forwarded_samples.get(), 1);
        assert_eq!(metrics.flushed_batches.get(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn process_drained_records_counts_remaining_records_on_mid_batch_send_error() {
        let (effect_handler, rx) = test_effect_handler(1, false);
        drop(rx);
        let subscriptions = vec![test_subscription()];
        let admission_state = normal_admission_state();
        let metrics = test_metrics();
        let mut builder = ArrowRecordsBuilder::new();
        let mut drained_records = vec![test_raw_record(0), test_raw_record(0), test_raw_record(0)];
        let drain_stats = SessionDrainStats {
            received_samples: 3,
            ..Default::default()
        };

        let error = process_drained_records(
            &effect_handler,
            &subscriptions,
            &admission_state,
            &metrics,
            &mut builder,
            &mut drained_records,
            drain_stats,
            &test_batch_config(2),
        )
        .await
        .expect_err("closed downstream should fail the mid-batch flush");

        assert!(
            matches!(error, Error::ChannelSendError { closed: true, .. }),
            "unexpected error: {error}"
        );

        let metrics = metrics.borrow();
        assert!(builder.is_empty());
        assert!(drained_records.is_empty());
        assert_eq!(metrics.forwarded_samples.get(), 0);
        assert_eq!(metrics.flushed_batches.get(), 0);
        assert_eq!(metrics.dropped_send_error.get(), 3);
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
        )
        .await
        .expect("drained records processed");

        assert_eq!(builder.len(), 1);
        flush_batch(&effect_handler, &metrics, &mut builder)
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
        let mut subscriptions = vec![SubscriptionConfig {
            tracepoint: "user_events:example_L5K1".to_owned(),
            format: FormatConfig::Tracefs,
        }];

        UserEventsReceiver::normalize_subscriptions(&mut subscriptions)
            .expect("subscriptions accepted");
        assert_eq!(subscriptions[0].tracepoint, "user_events:example_L5K1");
    }

    #[test]
    fn normalize_subscriptions_accepts_bare_user_events_name() {
        let mut subscriptions = vec![SubscriptionConfig {
            tracepoint: "example_L5K1".to_owned(),
            format: FormatConfig::Tracefs,
        }];

        UserEventsReceiver::normalize_subscriptions(&mut subscriptions)
            .expect("subscriptions accepted");
        assert_eq!(subscriptions[0].tracepoint, "user_events:example_L5K1");
    }

    #[test]
    fn deserialize_config_rejects_legacy_tracepoint_shorthand() {
        let error = serde_json::from_value::<UserEventsReceiverConfig>(serde_json::json!({
            "tracepoint": "user_events:example_L5K1"
        }))
        .expect_err("legacy shorthand rejected");

        assert!(
            error.to_string().contains("unknown field `tracepoint`"),
            "unexpected error: {error}"
        );
    }

    #[cfg(not(feature = "user_events-eventheader"))]
    #[test]
    fn deserialize_config_rejects_event_header_without_feature() {
        let error = serde_json::from_value::<UserEventsReceiverConfig>(serde_json::json!({
            "subscriptions": [
                {
                    "tracepoint": "user_events:example_L5K1",
                    "format": {
                        "type": "event_header"
                    }
                }
            ]
        }))
        .expect_err("event_header rejected without feature");

        assert!(
            error.to_string().contains("unknown variant `event_header`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_subscriptions_rejects_empty_list() {
        let config = UserEventsReceiverConfig {
            subscriptions: Vec::new(),
            session: None,
            drain: None,
            batching: None,
        };

        let mut subscriptions = config.subscriptions;
        let error = UserEventsReceiver::normalize_subscriptions(&mut subscriptions)
            .expect_err("config rejected");
        assert!(
            error.to_string().contains("at least one subscription"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_subscriptions_rejects_non_user_events_tracepoint_group() {
        let config = UserEventsReceiverConfig {
            subscriptions: vec![SubscriptionConfig {
                tracepoint: "foo:example_L2K1".to_owned(),
                format: FormatConfig::Tracefs,
            }],
            session: None,
            drain: None,
            batching: None,
        };

        let mut subscriptions = config.subscriptions;
        let error = UserEventsReceiver::normalize_subscriptions(&mut subscriptions)
            .expect_err("config rejected");
        assert!(
            error
                .to_string()
                .contains("event name or `user_events:<event>`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_subscriptions_rejects_empty_tracepoint_name() {
        let mut subscriptions = vec![SubscriptionConfig {
            tracepoint: String::new(),
            format: FormatConfig::Tracefs,
        }];

        let error = UserEventsReceiver::normalize_subscriptions(&mut subscriptions)
            .expect_err("config rejected");
        assert!(
            error.to_string().contains("must not be empty"),
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
            late_registration_poll_interval: None,
        };
        let error = UserEventsReceiver::validate_session(&session).expect_err("zero rejected");
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
            late_registration_poll_interval: None,
        };
        let error = UserEventsReceiver::validate_session(&session).expect_err("zero rejected");
        assert!(
            error.to_string().contains("max_pending_bytes"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_session_rejects_zero_late_registration_poll_interval() {
        let session = SessionConfig {
            per_cpu_buffer_size: default_per_cpu_buffer_size(),
            wakeup_watermark: default_wakeup_watermark(),
            max_pending_events: default_max_pending_events(),
            max_pending_bytes: default_max_pending_bytes(),
            late_registration_poll_interval: Some(Duration::ZERO),
        };
        let error = UserEventsReceiver::validate_session(&session).expect_err("zero rejected");
        assert!(
            error
                .to_string()
                .contains("late_registration_poll_interval"),
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
        let error = UserEventsReceiver::validate_drain(&drain).expect_err("zero rejected");
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
        let error = UserEventsReceiver::validate_drain(&drain).expect_err("zero rejected");
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
        let error = UserEventsReceiver::validate_drain(&drain).expect_err("zero rejected");
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
        UserEventsReceiver::validate_drain(&drain).expect("non-zero accepted");
    }

    #[test]
    fn validate_batching_rejects_zero_max_duration() {
        let batching = BatchConfig {
            max_size: default_batch_max_size(),
            max_duration: Duration::ZERO,
        };
        let error = UserEventsReceiver::validate_batching(&batching).expect_err("zero rejected");
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
        UserEventsReceiver::validate_batching(&batching).expect("non-zero accepted");
    }
}
