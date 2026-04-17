// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

//! Linux userevents receiver.

mod arrow_records_encoder;
mod decoder;
mod metrics;
mod session;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

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
#[cfg(target_os = "linux")]
use self::decoder::DecodedUsereventsRecord;
use self::metrics::UsereventsReceiverMetrics;
#[cfg(target_os = "linux")]
use self::session::SessionInitError;
use self::session::UsereventsSession;
#[cfg(target_os = "linux")]
use otap_df_engine::control::NodeControlMsg;
#[cfg(target_os = "linux")]
use otap_df_telemetry::{otel_info, otel_warn};
#[cfg(target_os = "linux")]
use tokio::time::{self, MissedTickBehavior};

const DEFAULT_PER_CPU_BUFFER_SIZE: usize = 1024 * 1024;
const DEFAULT_WAKEUP_WATERMARK: usize = 256 * 1024;
const DEFAULT_MAX_RECORDS_PER_TURN: usize = 1024;
const DEFAULT_MAX_BYTES_PER_TURN: usize = 1024 * 1024;
const DEFAULT_MAX_DRAIN_NS: Duration = Duration::from_millis(2);
const DEFAULT_BATCH_MAX_SIZE: u16 = 512;
const DEFAULT_BATCH_MAX_DURATION: Duration = Duration::from_millis(50);
const DEFAULT_LATE_REGISTRATION_POLL: Duration = Duration::from_secs(1);

/// URN for the Linux userevents receiver.
pub const USEREVENTS_RECEIVER_URN: &str = "urn:otel:receiver:userevents";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum FormatConfig {
    #[default]
    Raw,
    CommonSchemaOtelLogs,
    EventheaderFlat {
        #[serde(default = "default_flatten_prefix")]
        flatten_prefix: String,
    },
    CustomEventheader {
        body_field: Option<String>,
        severity_number_field: Option<String>,
        severity_text_field: Option<String>,
        event_name_field: Option<String>,
        #[serde(default)]
        attributes_from: Vec<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubscriptionConfig {
    tracepoint: String,
    #[serde(default)]
    format: FormatConfig,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
struct LateRegistrationConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_late_registration_poll_ms")]
    poll_interval_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct SessionConfig {
    #[serde(default = "default_per_cpu_buffer_size")]
    per_cpu_buffer_size: usize,
    #[serde(default = "default_wakeup_watermark")]
    wakeup_watermark: usize,
    #[serde(default)]
    late_registration: LateRegistrationConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct DrainConfig {
    #[serde(default = "default_max_records_per_turn")]
    max_records_per_turn: usize,
    #[serde(default = "default_max_bytes_per_turn")]
    max_bytes_per_turn: usize,
    #[serde(default = "default_max_drain_ns")]
    #[serde(with = "humantime_serde")]
    max_drain_ns: Duration,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct BatchConfig {
    #[serde(default = "default_batch_max_size")]
    max_size: u16,
    #[serde(default = "default_batch_max_duration")]
    #[serde(with = "humantime_serde")]
    max_duration: Duration,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum OverflowMode {
    Drop,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct OverflowConfig {
    #[serde(default = "default_overflow_mode")]
    on_downstream_full: OverflowMode,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct Config {
    #[serde(default)]
    tracepoint: Option<String>,
    #[serde(default)]
    format: Option<FormatConfig>,
    #[serde(default)]
    subscriptions: Option<Vec<SubscriptionConfig>>,
    #[serde(default)]
    session: Option<SessionConfig>,
    #[serde(default)]
    drain: Option<DrainConfig>,
    #[serde(default)]
    batching: Option<BatchConfig>,
    #[serde(default)]
    overflow: Option<OverflowConfig>,
}

struct UsereventsReceiver {
    subscriptions: Vec<SubscriptionConfig>,
    subscription_tracepoints: Vec<Arc<str>>,
    session: SessionConfig,
    drain: DrainConfig,
    batching: BatchConfig,
    overflow: OverflowConfig,
    cpu_id: usize,
    metrics: Rc<RefCell<MetricSet<UsereventsReceiverMetrics>>>,
    admission_state: LocalReceiverAdmissionState,
}

impl UsereventsReceiver {
    fn from_config(
        pipeline: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        let subscriptions = Self::normalize_subscriptions(&config)?;
        let subscription_tracepoints = subscriptions
            .iter()
            .map(|subscription| Arc::<str>::from(subscription.tracepoint.as_str()))
            .collect();
        let session = config.session.clone().unwrap_or(SessionConfig {
            per_cpu_buffer_size: default_per_cpu_buffer_size(),
            wakeup_watermark: default_wakeup_watermark(),
            late_registration: LateRegistrationConfig {
                enabled: false,
                poll_interval_ms: default_late_registration_poll_ms(),
            },
        });
        let drain = config.drain.clone().unwrap_or(DrainConfig {
            max_records_per_turn: default_max_records_per_turn(),
            max_bytes_per_turn: default_max_bytes_per_turn(),
            max_drain_ns: default_max_drain_ns(),
        });
        let batching = config.batching.clone().unwrap_or(BatchConfig {
            max_size: default_batch_max_size(),
            max_duration: default_batch_max_duration(),
        });
        let overflow = config.overflow.clone().unwrap_or(OverflowConfig {
            on_downstream_full: default_overflow_mode(),
        });

        Ok(Self {
            subscriptions,
            subscription_tracepoints,
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

    fn normalize_subscriptions(
        config: &Config,
    ) -> Result<Vec<SubscriptionConfig>, otap_df_config::error::Error> {
        match (&config.tracepoint, &config.subscriptions) {
            (Some(_), Some(_)) => Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "configure either `tracepoint` or `subscriptions`, not both".to_owned(),
            }),
            (None, None) => Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "userevents receiver requires either `tracepoint` or `subscriptions`"
                    .to_owned(),
            }),
            (Some(tracepoint), None) => Ok(vec![SubscriptionConfig {
                tracepoint: tracepoint.clone(),
                format: config.format.clone().unwrap_or_default(),
            }]),
            (None, Some(subscriptions)) if subscriptions.is_empty() => {
                Err(otap_df_config::error::Error::InvalidUserConfig {
                    error: "userevents receiver requires at least one subscription".to_owned(),
                })
            }
            (None, Some(subscriptions)) => Ok(subscriptions.clone()),
        }
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
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
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
                pipeline_core = node_name.as_str(),
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
                        let drain_stats = drained.map_err(|error| Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            kind: ReceiverErrorKind::Transport,
                            error: "failed to drain Linux userevents perf ring".to_owned(),
                            source_detail: format_error_sources(&error),
                        })?;

                        let received_samples = drain_stats.received_samples;
                        let mut dropped_memory_pressure: u64 = 0;
                        let dropped_no_subscription = drain_stats.dropped_no_subscription;
                        let mut cs_decode_fallbacks: u64 = 0;
                        for raw in drained_records.drain(..) {
                            if self.admission_state.should_shed_ingress() {
                                dropped_memory_pressure += 1;
                                continue;
                            }

                            let subscription_index = raw.subscription_index;
                            let subscription = &self.subscriptions[subscription_index];
                            let tracepoint =
                                Arc::clone(&self.subscription_tracepoints[subscription_index]);

                            let (decoded, cs_failed) =
                                DecodedUsereventsRecord::from_raw(tracepoint, raw, &subscription.format);
                            if cs_failed {
                                cs_decode_fallbacks += 1;
                            }
                            builder.append(decoded);
                            if builder.len() >= batch_cfg.max_size {
                                flush_batch(&effect_handler, &self.metrics, &mut builder, &overflow_mode).await?;
                            }
                        }

                        let mut metrics = self.metrics.borrow_mut();
                        if drain_stats.lost_samples > 0 {
                            metrics.lost_perf_samples.add(drain_stats.lost_samples);
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
                        if cs_decode_fallbacks > 0 {
                            metrics.cs_decode_fallbacks.add(cs_decode_fallbacks);
                        }
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

fn default_flatten_prefix() -> String {
    "userevents.field".to_owned()
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
            late_registration: LateRegistrationConfig::default(),
        };
        let subscriptions = vec![SubscriptionConfig {
            tracepoint,
            format: FormatConfig::Raw,
        }];

        let session = UsereventsSession::open(&subscriptions, &config, 0)
            .expect("open a perf session for the registered tracepoint");
        assert_eq!(session.subscription_count(), 1);
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn normalize_single_tracepoint_shorthand() {
        let config = Config {
            tracepoint: Some("user_events:example_L2K1".to_owned()),
            format: Some(FormatConfig::CommonSchemaOtelLogs),
            subscriptions: None,
            session: None,
            drain: None,
            batching: None,
            overflow: None,
        };

        let normalized =
            UsereventsReceiver::normalize_subscriptions(&config).expect("normalized subscriptions");
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0].tracepoint, "user_events:example_L2K1");
        assert!(matches!(
            normalized[0].format,
            FormatConfig::CommonSchemaOtelLogs
        ));
    }

    #[test]
    fn normalize_subscriptions_list() {
        let config = Config {
            tracepoint: None,
            format: None,
            subscriptions: Some(vec![SubscriptionConfig {
                tracepoint: "user_events:example_L5K1".to_owned(),
                format: FormatConfig::EventheaderFlat {
                    flatten_prefix: "custom.field".to_owned(),
                },
            }]),
            session: None,
            drain: None,
            batching: None,
            overflow: None,
        };

        let normalized =
            UsereventsReceiver::normalize_subscriptions(&config).expect("normalized subscriptions");
        assert_eq!(normalized.len(), 1);
        match &normalized[0].format {
            FormatConfig::EventheaderFlat { flatten_prefix } => {
                assert_eq!(flatten_prefix, "custom.field");
            }
            other => panic!("unexpected format: {other:?}"),
        }
    }

    #[test]
    fn normalize_rejects_both_tracepoint_and_subscriptions() {
        let config = Config {
            tracepoint: Some("user_events:example_L2K1".to_owned()),
            format: Some(FormatConfig::Raw),
            subscriptions: Some(vec![SubscriptionConfig {
                tracepoint: "user_events:example_L3K1".to_owned(),
                format: FormatConfig::Raw,
            }]),
            session: None,
            drain: None,
            batching: None,
            overflow: None,
        };

        let error =
            UsereventsReceiver::normalize_subscriptions(&config).expect_err("config rejected");
        assert!(
            error
                .to_string()
                .contains("either `tracepoint` or `subscriptions`")
        );
    }
}
