// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ETW (Event Tracing for Windows) Receiver
//!
//! Receives ETW events from Windows ETW sessions and emits OTAP Arrow log records.
//!
//! This module is compiled only on Windows (`#[cfg(target_os = "windows")]`
//! in the parent `receivers/mod.rs`).
//!
//! ## Multi-core fan-out
//!
//! Windows allows only one real-time ETW session per session name. The engine
//! may instantiate the receiver once per allocated core.  To reconcile these
//! two models the ETW session is a **process-global singleton** with N consumer
//! channels (one per core).  The `ProcessTrace` callback round-robins events
//! across all channels so each core receives an even share of the event stream.
//!
//! ```text
//! ProcessTrace OS thread  (singleton, lazily spawned)
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
//!       - guid: "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"
//!         level: information
//! ```

mod session;

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
    effect_handler::TelemetryTimerCancelHandle, error::Error, local::receiver as local,
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

/// Top-level configuration for the ETW receiver.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Config {
    /// One or more ETW providers to subscribe to.
    pub providers: Vec<ProviderConfig>,

    /// Name of the ETW trace session. Defaults to `"OtelArrowETW"`.
    #[serde(default = "default_session_name")]
    pub session_name: String,
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

        Ok(())
    }
}

fn default_session_name() -> String {
    "OtelArrowETW".to_string()
}

// ── Receiver struct ──────────────────────────────────────────────────────────

/// ETW receiver that subscribes to Windows ETW trace sessions and converts
/// events into OTAP Arrow log records.
///
/// Each per-core instance holds its own `event_rx` obtained from the
/// process-global singleton session at factory time.
struct EtwReceiver {
    config: Config,
    metrics: Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
    /// Per-core consumer channel from the singleton ETW session.
    event_rx: tokio::sync::mpsc::Receiver<EtwEventData>,
}

impl EtwReceiver {
    /// Create a new receiver, acquiring one consumer channel from the
    /// process-global ETW session.
    ///
    /// Called at **factory time** (once per allocated core).  The first call
    /// lazily initializes the singleton session and spawns the `ProcessTrace`
    /// thread.
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

        // Acquire this core's consumer channel from the singleton session.
        // The first call initializes the session; subsequent calls pop from
        // the pre-allocated pool.
        let event_rx = session::subscribe(&cfg, num_cores).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: format!("ETW session initialization failed: {e}"),
            }
        })?;

        Ok(EtwReceiver {
            config: cfg,
            metrics: Rc::new(RefCell::new(metrics)),
            event_rx,
        })
    }
}

// ── Event processing loop ────────────────────────────────────────────────────

impl EtwReceiver {
    /// Main event loop when an ETW session is active.
    ///
    /// Consumes events from the per-core MPSC channel and processes control
    /// messages. Returns when a shutdown or drain-ingress control message is
    /// received.
    async fn run_event_loop(
        &mut self,
        ctrl_chan: &mut local::ControlChannel<OtapPdata>,
        effect_handler: &local::EffectHandler<OtapPdata>,
        telemetry_timer_handle: TelemetryTimerCancelHandle<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let mut event_count: u64 = 0;
        let mut channel_alive = true;

        loop {
            tokio::select! {
                biased; // Prioritise control messages over data.

                ctrl_msg = ctrl_chan.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            let _ = telemetry_timer_handle.cancel().await;
                            // TODO: drain buffered events before notifying
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

                // Receive event data from the singleton ETW session.
                // Only poll when the channel is still alive to avoid
                // spinning on a closed channel.
                event_data = self.event_rx.recv(), if channel_alive => {
                    match event_data {
                        Some(event) => {
                            self.metrics.borrow_mut().received_events_total.inc();
                            event_count += 1;

                            // Log first 100 events individually, then every 1000th.
                            if event_count <= 100 || event_count.is_multiple_of(1000) {
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
                            // exited (process shutdown or unrecoverable error).
                            // Stop polling event_rx; only handle control
                            // messages from now on.
                            channel_alive = false;
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

    /// Number of ETW events refused by downstream (backpressure/unavailable).
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
}
