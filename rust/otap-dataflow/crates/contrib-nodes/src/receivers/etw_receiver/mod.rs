// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ETW (Event Tracing for Windows) Receiver
//!
//! Receives ETW events from Windows ETW sessions and emits OTAP Arrow log records.
//!
//! This module is compiled only on Windows (`#[cfg(target_os = "windows")]`
//! in the parent `receivers/mod.rs`).
//!
//! ## Quick start
//!
//! ```yaml
//! etw:
//!   type: receiver:etw
//!   config:
//!     providers:
//!       - guid: "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"
//!         level: information
//! ```

mod session;

use session::EtwEventData;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::{
    effect_handler::TelemetryTimerCancelHandle,
    error::Error,
    local::receiver as local,
};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::otel_info;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

/// URN for the ETW receiver.
pub const ETW_RECEIVER_URN: &str = "urn:otel:receiver:etw";

// ── Configuration ────────────────────────────────────────────────────────────

/// Trace level filter for ETW providers, matching the standard five ETW levels.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TraceLevel {
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
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderConfig {
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

impl ProviderConfig {
    /// Returns the display label for this provider (for logging and error messages).
    pub fn display_id(&self) -> &str {
        if let Some(name) = &self.name {
            name.as_str()
        } else if let Some(guid) = &self.guid {
            guid.as_str()
        } else {
            "<unknown>"
        }
    }
}

/// Top-level configuration for the ETW receiver.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// One or more ETW providers to subscribe to.
    pub providers: Vec<ProviderConfig>,

    /// Name of the ETW trace session. Defaults to `"OtelArrowETW"`.
    #[serde(default = "default_session_name")]
    pub session_name: String,
}

fn default_session_name() -> String {
    "OtelArrowETW".to_string()
}

// ── Receiver struct ──────────────────────────────────────────────────────────

/// ETW receiver that subscribes to Windows ETW trace sessions and converts
/// events into OTAP Arrow log records.
struct EtwReceiver {
    config: Config,
    metrics: Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
}

impl EtwReceiver {
    fn with_pipeline(pipeline: PipelineContext, config: Config) -> Self {
        let metrics = pipeline.register_metrics::<EtwReceiverMetrics>();
        EtwReceiver {
            config,
            metrics: Rc::new(RefCell::new(metrics)),
        }
    }

    fn from_config(
        pipeline: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let cfg: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        if cfg.providers.is_empty() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "at least one ETW provider must be configured".to_string(),
            });
        }

        // Validate that each provider specifies exactly one of `name` or `guid`.
        for (i, provider) in cfg.providers.iter().enumerate() {
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
                        error: format!(
                            "provider[{i}]: either 'name' or 'guid' must be specified"
                        ),
                    });
                }
                _ => {} // Exactly one is set — valid.
            }
        }

        Ok(EtwReceiver::with_pipeline(pipeline, cfg))
    }
}

// ── Event processing loop ────────────────────────────────────────────────────

impl EtwReceiver {
    /// Main event loop when an ETW session is active.
    ///
    /// Consumes events from the ETW session channel and processes control
    /// messages. Returns when a shutdown or drain-ingress control message is
    /// received.
    async fn run_with_session(
        &self,
        session_handle: session::SessionHandle,
        event_rx: &mut tokio::sync::mpsc::Receiver<EtwEventData>,
        ctrl_chan: &mut local::ControlChannel<OtapPdata>,
        effect_handler: &local::EffectHandler<OtapPdata>,
        telemetry_timer_handle: TelemetryTimerCancelHandle<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let mut event_count: u64 = 0;
        let mut session_alive = true;

        loop {
            tokio::select! {
                biased; // Prioritise control messages over data.

                ctrl_msg = ctrl_chan.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            let _ = telemetry_timer_handle.cancel().await;
                            session_handle.stop();
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
                            session_handle.stop();
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

                // Receive event data from the ETW session thread.
                // Only poll when the session is still alive to avoid
                // spinning on a closed channel.
                event_data = event_rx.recv(), if session_alive => {
                    match event_data {
                        Some(event) => {
                            self.metrics.borrow_mut().received_events_total.inc();
                            event_count += 1;

                            // Log first 10 events individually, then every 1000th.
                            if event_count <= 10 || event_count % 1000 == 0 {
                                otel_info!(
                                    "etw_receiver.event",
                                    total = event_count,
                                    event_id = event.event_id,
                                    level = event.level,
                                    opcode = event.opcode,
                                    pid = event.process_id,
                                    tid = event.thread_id,
                                    timestamp = event.timestamp,
                                    keywords = event.keywords,
                                );
                            }

                            // TODO: Convert event data to Arrow record batches
                            // and forward downstream via effect_handler.
                        }
                        None => {
                            // Channel closed — the ETW session thread has
                            // exited. Stop polling event_rx; only handle
                            // control messages from now on.
                            session_alive = false;
                            otel_info!(
                                "etw_receiver.session_ended",
                                message = "ETW session event channel closed",
                                total_events = event_count,
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
             receiver_config: &ReceiverConfig| {
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
        self: Box<Self>,
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
        );

        let (session_handle, mut event_rx) =
            session::start_etw_session(&self.config, self.metrics.clone())?;

        self.run_with_session(
            session_handle,
            &mut event_rx,
            &mut ctrl_chan,
            &effect_handler,
            telemetry_timer_handle,
        )
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

    /// Number of ETW events refused by downstream (backpressure/unavailable).
    #[metric(unit = "{event}")]
    pub received_events_forward_failed: Counter<u64>,

    /// Number of ETW events dropped due to process-wide memory pressure.
    #[metric(unit = "{event}")]
    pub received_events_rejected_memory_pressure: Counter<u64>,
}