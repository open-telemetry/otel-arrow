// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Journald receiver.
//!
//! This is the first incremental slice. It introduces the receiver factory,
//! configuration types, process-local source lease, and a no-op
//! receiver loop that honors lifecycle/drain/shutdown control messages.
//! The blocking `sd-journal` worker, batch handoff, Ack/Nack tracking, and
//! checkpoint persistence land in follow-up PRs as described in
//! [`docs/journald-receiver.md`](../../../../../../docs/journald-receiver.md).
//!
//! Most runtime code is Linux-only (`#[cfg(target_os = "linux")]`); on other
//! platforms the factory rejects construction with a clear error.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

#[cfg(target_os = "linux")]
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
#[cfg(target_os = "linux")]
use otap_df_engine::control::NodeControlMsg;
#[cfg(target_os = "linux")]
use otap_df_engine::error::Error;
#[cfg(target_os = "linux")]
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
#[cfg(target_os = "linux")]
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
#[cfg(target_os = "linux")]
use otap_df_telemetry::metrics::MetricSetSnapshot;
#[cfg(target_os = "linux")]
use otap_df_telemetry::otel_info;
use otap_df_telemetry_macros::metric_set;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};

mod config;

use config::RuntimeConfig;
pub use config::{
    BatchConfig, CheckpointConfig, Config, DEFAULT_JOURNAL_ROOT_PATH, DEFAULT_SOURCE_ID,
    JournalConfig, MaxPriority, OnNack, StartAt, severity_number_from_priority,
};

/// URN for the journald receiver.
pub const JOURNALD_RECEIVER_URN: &str = "urn:otel:receiver:journald";

/// Telemetry metrics for the journald receiver.
///
/// The set is intentionally small in this first slice; richer counters
/// (records emitted, cursor commits, batches held by backpressure, etc.) land
/// alongside the worker thread that produces them.
#[metric_set(name = "journald.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct JournaldReceiverMetrics {
    /// Number of times the receiver was started.
    #[metric(unit = "{start}")]
    pub starts: Counter<u64>,
    /// Number of clean drain transitions.
    #[metric(unit = "{drain}")]
    pub drains: Counter<u64>,
    /// Number of clean shutdown transitions.
    #[metric(unit = "{shutdown}")]
    pub shutdowns: Counter<u64>,
}

/// Journald receiver instance.
pub struct JournaldReceiver {
    #[allow(dead_code)]
    config: RuntimeConfig,
    _lease: SourceLease,
    metrics: Option<MetricSet<JournaldReceiverMetrics>>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Declares the journald receiver as a local receiver factory.
pub static JOURNALD_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: JOURNALD_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        create_journald_receiver(pipeline, node, node_config, receiver_config)
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: validate_journald_config,
};

#[cfg(target_os = "linux")]
fn create_journald_receiver(
    pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
) -> Result<ReceiverWrapper<OtapPdata>, otap_df_config::error::Error> {
    if pipeline.num_cores() > 1 {
        return Err(otap_df_config::error::Error::InvalidUserConfig {
            error: "journald must run in a one-core source pipeline; use \
                 receiver:journald -> exporter:topic and fan out downstream"
                .to_owned(),
        });
    }
    let mut receiver = JournaldReceiver::from_config(&node_config.config)?;
    receiver.metrics = Some(pipeline.register_metrics::<JournaldReceiverMetrics>());
    Ok(ReceiverWrapper::local(
        receiver,
        node,
        node_config,
        receiver_config,
    ))
}

#[cfg(not(target_os = "linux"))]
fn create_journald_receiver(
    _pipeline: PipelineContext,
    _node: NodeId,
    _node_config: Arc<NodeUserConfig>,
    _receiver_config: &ReceiverConfig,
) -> Result<ReceiverWrapper<OtapPdata>, otap_df_config::error::Error> {
    Err(unsupported_platform_error())
}

fn validate_journald_config(config: &Value) -> Result<(), otap_df_config::error::Error> {
    let parsed: Config = serde_json::from_value(config.clone()).map_err(|e| {
        otap_df_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        }
    })?;
    RuntimeConfig::try_from(parsed).map(|_| ())
}

#[cfg(not(target_os = "linux"))]
fn unsupported_platform_error() -> otap_df_config::error::Error {
    otap_df_config::error::Error::InvalidUserConfig {
        error: "journald receiver is supported only on Linux".to_owned(),
    }
}

impl JournaldReceiver {
    /// Builds a receiver from a JSON config value.
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let parsed: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Self::new(parsed)
    }

    /// Builds a receiver from an already-deserialized `Config`.
    pub fn new(config: Config) -> Result<Self, otap_df_config::error::Error> {
        let runtime = RuntimeConfig::try_from(config)?;
        let lease = SourceLease::acquire(&runtime.lease_key)?;
        Ok(Self {
            config: runtime,
            _lease: lease,
            metrics: None,
        })
    }
}

#[cfg(target_os = "linux")]
fn terminal_state(
    deadline: std::time::Instant,
    metrics: &Option<MetricSet<JournaldReceiverMetrics>>,
) -> TerminalState {
    if let Some(metrics) = metrics {
        TerminalState::new(deadline, [metrics.snapshot()])
    } else {
        TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, [])
    }
}

#[cfg(target_os = "linux")]
#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for JournaldReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let JournaldReceiver {
            config,
            _lease,
            mut metrics,
        } = *self;

        if let Some(metrics) = metrics.as_mut() {
            metrics.starts.add(1);
        }

        otel_info!(
            "journald_receiver.start",
            source_id = config.source_id.as_str(),
            journal_root_path = config.journal.root_path.display().to_string()
        );

        // Periodic telemetry collection mirrors host_metrics_receiver.
        let _ = effect_handler
            .start_periodic_telemetry(std::time::Duration::from_secs(1))
            .await?;

        // The blocking sd-journal worker, bounded handoff channel, and Ack
        // tracker are introduced in a follow-up PR. Until then this loop only
        // services control messages so the engine sees a well-behaved node
        // that drains and shuts down on demand.
        loop {
            match ctrl_msg_recv.recv().await {
                Ok(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    if let Some(metrics) = metrics.as_mut() {
                        let _ = metrics_reporter.report(metrics);
                    }
                }
                Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                    if let Some(metrics) = metrics.as_mut() {
                        metrics.drains.add(1);
                    }
                    otel_info!(
                        "journald_receiver.drain_ingress",
                        source_id = config.source_id.as_str()
                    );
                    effect_handler.notify_receiver_drained().await?;
                    return Ok(terminal_state(deadline, &metrics));
                }
                Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                    if let Some(metrics) = metrics.as_mut() {
                        metrics.shutdowns.add(1);
                    }
                    otel_info!(
                        "journald_receiver.shutdown",
                        source_id = config.source_id.as_str()
                    );
                    return Ok(terminal_state(deadline, &metrics));
                }
                Ok(_) => {}
                Err(e) => return Err(Error::ChannelRecvError(e)),
            }
        }
    }
}

// --- Process-local source lease ----------------------------------------------
//
// The receiver design (see `docs/journald-receiver.md`) requires that, within
// a single process, no two journald receivers target the same concrete journal
// source selection. Cross-process duplication is left to operators in v1.

static JOURNALD_LEASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

struct SourceLease {
    key: String,
}

impl SourceLease {
    fn acquire(key: &str) -> Result<Self, otap_df_config::error::Error> {
        let mut leases = JOURNALD_LEASES.lock().map_err(|_| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: "journald lease registry is unavailable".to_owned(),
            }
        })?;
        if !leases.insert(key.to_owned()) {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: format!("another journald receiver already targets source `{key}`"),
            });
        }
        Ok(Self {
            key: key.to_owned(),
        })
    }
}

impl Drop for SourceLease {
    fn drop(&mut self) {
        if let Ok(mut leases) = JOURNALD_LEASES.lock() {
            let _ = leases.remove(&self.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_config_field() {
        let json = serde_json::json!({
            "source_id": "system",
            "unknown_field": true,
        });
        assert!(validate_journald_config(&json).is_err());
    }

    #[test]
    fn validates_default_config() {
        let json = serde_json::json!({});
        validate_journald_config(&json).expect("default config must validate");
    }

    #[test]
    fn lease_blocks_duplicate_source_in_process() {
        // Use a key not used elsewhere in tests to avoid races.
        let key = "journald:/test-lease-duplicate:<default>";
        let lease1 = SourceLease::acquire(key).expect("first lease must succeed");
        let lease2 = SourceLease::acquire(key);
        assert!(lease2.is_err(), "duplicate source lease must be rejected");
        drop(lease1);
        // Lease must be released on drop.
        let lease3 = SourceLease::acquire(key).expect("lease must be reacquirable after drop");
        drop(lease3);
    }

    #[test]
    fn distinct_sources_can_coexist() {
        let a = SourceLease::acquire("journald:/test-lease-a:<default>").expect("a");
        let b = SourceLease::acquire("journald:/test-lease-b:<default>").expect("b");
        drop(a);
        drop(b);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn factory_rejects_multi_core_pipeline() {
        let registry = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller = otap_df_engine::context::ControllerContext::new(registry);
        let pipeline =
            controller.pipeline_context_with("test-group".into(), "test-pipeline".into(), 0, 2, 0);
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(JOURNALD_RECEIVER_URN));
        let receiver_config = ReceiverConfig::new("journald");
        let result = create_journald_receiver(
            pipeline,
            NodeId {
                index: 0,
                name: "journald".into(),
            },
            node_config,
            &receiver_config,
        );
        match result {
            Err(otap_df_config::error::Error::InvalidUserConfig { error }) => {
                assert!(error.contains("one-core"));
            }
            Err(other) => panic!("unexpected error: {other:?}"),
            Ok(_) => panic!("multi-core journald receiver must be rejected"),
        }
    }
}
