// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Built-in read-only controller monitor extension.
//!
//! This extension is a small reference implementation for controller-level
//! extension hooks and a useful default monitor for the stock `df_engine`
//! binary. It is enabled only when declared under `engine.extensions` with the
//! [`CONTROLLER_MONITOR_EXTENSION_URN`] type.
//!
//! The monitor periodically reads the shared observed-state handle and
//! telemetry registry exposed through [`ControllerExtensionContext`]. It reports
//! aggregate controller gauges under the `controller.monitor` metric set and,
//! when configured, emits compact internal snapshot logs.
//!
//! The extension is intentionally non-mutating: it does not invoke rollout,
//! shutdown, or reconfiguration operations on the control-plane handle. It exits
//! through the standard controller extension cancellation token during engine
//! shutdown.

use crate::{
    ControllerExtensionContext, ControllerExtensionError, ControllerExtensionRegistry,
    ControllerExtensionTaskFactory,
};
use otap_df_config::ExtensionId;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_state::store::ObservedStateHandle;
use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};
use otap_df_telemetry::descriptor::{
    AttributeField, AttributeValueType, Instrument, MetricValueType, MetricsDescriptor,
    MetricsField,
};
use otap_df_telemetry::instrument::Gauge;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler, MetricValue};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::{otel_info, otel_warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::time::{Instant as TokioInstant, MissedTickBehavior, interval_at};
use tokio_util::sync::CancellationToken;

/// Built-in controller monitor extension type URN.
pub const CONTROLLER_MONITOR_EXTENSION_URN: &str = "urn:otel:extension:controller_monitor";

const fn default_monitor_interval() -> Duration {
    Duration::from_secs(30)
}

const fn default_log_snapshots() -> bool {
    true
}

/// Configuration for the built-in controller monitor extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ControllerMonitorConfig {
    /// Interval between controller monitor snapshots.
    #[serde(default = "default_monitor_interval", with = "humantime_serde")]
    pub interval: Duration,

    /// Whether each monitor snapshot should also be emitted as an internal info log.
    #[serde(default = "default_log_snapshots")]
    pub log_snapshots: bool,
}

impl Default for ControllerMonitorConfig {
    fn default() -> Self {
        Self {
            interval: default_monitor_interval(),
            log_snapshots: default_log_snapshots(),
        }
    }
}

impl ControllerMonitorConfig {
    fn from_extension(extension: &ExtensionUserConfig) -> Result<Self, ControllerMonitorError> {
        let config = if extension.config.is_null() {
            Self::default()
        } else {
            serde_json::from_value(extension.config.clone())
                .map_err(|source| ControllerMonitorError::InvalidConfig { source })?
        };
        if config.interval.is_zero() {
            return Err(ControllerMonitorError::ZeroInterval);
        }
        Ok(config)
    }
}

/// Errors returned while starting or running the built-in controller monitor.
#[derive(Debug, Error)]
pub enum ControllerMonitorError {
    /// The configured monitor extension payload could not be parsed.
    #[error("invalid controller monitor config: {source}")]
    InvalidConfig {
        /// Deserialization error.
        #[source]
        source: serde_json::Error,
    },

    /// The configured monitor interval is invalid.
    #[error("controller monitor interval must be greater than zero")]
    ZeroInterval,
}

/// Registers controller extension factories built into the controller crate.
pub fn register_builtin_controller_extensions(registry: &mut ControllerExtensionRegistry) {
    registry.register(
        CONTROLLER_MONITOR_EXTENSION_URN.into(),
        start_controller_monitor_extension,
    );
}

fn start_controller_monitor_extension(
    context: ControllerExtensionContext,
) -> Result<ControllerExtensionTaskFactory, ControllerExtensionError> {
    let config = ControllerMonitorConfig::from_extension(&context.extension)
        .map_err(|source| Box::new(source) as ControllerExtensionError)?;
    let monitor = ControllerMonitor::new(
        context.extension_id.clone(),
        context.observed_state,
        context.telemetry_registry,
        config,
    );

    Ok(Box::new(move |cancellation_token| {
        Box::pin(async move {
            monitor
                .run(cancellation_token)
                .await
                .map_err(|source| Box::new(source) as ControllerExtensionError)
        })
    }))
}

static CONTROLLER_MONITOR_ATTRIBUTES_DESCRIPTOR:
    otap_df_telemetry::descriptor::AttributesDescriptor =
    otap_df_telemetry::descriptor::AttributesDescriptor {
        name: "controller.monitor.attrs",
        fields: &[AttributeField {
            key: "extension.id",
            r#type: AttributeValueType::String,
            brief: "Configured controller monitor extension instance identifier.",
        }],
    };

struct ControllerMonitorAttributes {
    values: [AttributeValue; 1],
}

impl ControllerMonitorAttributes {
    fn new(extension_id: &ExtensionId) -> Self {
        Self {
            values: [AttributeValue::String(extension_id.as_ref().to_owned())],
        }
    }
}

impl AttributeSetHandler for ControllerMonitorAttributes {
    fn descriptor(&self) -> &'static otap_df_telemetry::descriptor::AttributesDescriptor {
        &CONTROLLER_MONITOR_ATTRIBUTES_DESCRIPTOR
    }

    fn attribute_values(&self) -> &[AttributeValue] {
        &self.values
    }
}

static CONTROLLER_MONITOR_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
    name: "controller.monitor",
    metrics: &[
        MetricsField {
            name: "observed_pipelines.total",
            unit: "{pipeline}",
            brief: "Observed logical pipelines currently tracked by the controller.",
            instrument: Instrument::Gauge,
            temporality: None,
            value_type: MetricValueType::U64,
        },
        MetricsField {
            name: "observed_pipelines.live",
            unit: "{pipeline}",
            brief: "Observed logical pipelines currently considered live.",
            instrument: Instrument::Gauge,
            temporality: None,
            value_type: MetricValueType::U64,
        },
        MetricsField {
            name: "observed_pipelines.ready",
            unit: "{pipeline}",
            brief: "Observed logical pipelines currently considered ready.",
            instrument: Instrument::Gauge,
            temporality: None,
            value_type: MetricValueType::U64,
        },
        MetricsField {
            name: "telemetry.entities",
            unit: "{entity}",
            brief: "Registered telemetry entities visible to the controller monitor.",
            instrument: Instrument::Gauge,
            temporality: None,
            value_type: MetricValueType::U64,
        },
        MetricsField {
            name: "telemetry.metric_sets",
            unit: "{metric_set}",
            brief: "Registered telemetry metric sets visible to the controller monitor.",
            instrument: Instrument::Gauge,
            temporality: None,
            value_type: MetricValueType::U64,
        },
    ],
};

#[derive(Debug, Default)]
struct ControllerMonitorMetrics {
    observed_pipelines_total: Gauge<u64>,
    observed_pipelines_live: Gauge<u64>,
    observed_pipelines_ready: Gauge<u64>,
    telemetry_entities: Gauge<u64>,
    telemetry_metric_sets: Gauge<u64>,
}

impl MetricSetHandler for ControllerMonitorMetrics {
    fn descriptor(&self) -> &'static MetricsDescriptor {
        &CONTROLLER_MONITOR_METRICS_DESCRIPTOR
    }

    fn snapshot_values(&self) -> Vec<MetricValue> {
        vec![
            MetricValue::from(self.observed_pipelines_total.get()),
            MetricValue::from(self.observed_pipelines_live.get()),
            MetricValue::from(self.observed_pipelines_ready.get()),
            MetricValue::from(self.telemetry_entities.get()),
            MetricValue::from(self.telemetry_metric_sets.get()),
        ]
    }

    fn clear_values(&mut self) {
        self.observed_pipelines_total.reset();
        self.observed_pipelines_live.reset();
        self.observed_pipelines_ready.reset();
        self.telemetry_entities.reset();
        self.telemetry_metric_sets.reset();
    }

    fn needs_flush(&self) -> bool {
        true
    }
}

struct ControllerMonitor {
    extension_id: ExtensionId,
    observed_state: ObservedStateHandle,
    telemetry_registry: TelemetryRegistryHandle,
    metrics: MetricSet<ControllerMonitorMetrics>,
    config: ControllerMonitorConfig,
}

impl ControllerMonitor {
    fn new(
        extension_id: ExtensionId,
        observed_state: ObservedStateHandle,
        telemetry_registry: TelemetryRegistryHandle,
        config: ControllerMonitorConfig,
    ) -> Self {
        let metrics = telemetry_registry.register_metric_set::<ControllerMonitorMetrics>(
            ControllerMonitorAttributes::new(&extension_id),
        );
        Self {
            extension_id,
            observed_state,
            telemetry_registry,
            metrics,
            config,
        }
    }

    async fn run(
        mut self,
        cancellation_token: CancellationToken,
    ) -> Result<(), ControllerMonitorError> {
        self.publish_snapshot();

        let mut ticker = interval_at(
            TokioInstant::now() + self.config.interval,
            self.config.interval,
        );
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    return Ok(());
                }
                _ = ticker.tick() => {
                    self.publish_snapshot();
                }
            }
        }
    }

    fn publish_snapshot(&mut self) {
        let snapshot = self.observed_state.snapshot();
        let observed_pipelines = snapshot.len() as u64;
        let live_pipelines = snapshot.values().filter(|status| status.liveness()).count() as u64;
        let ready_pipelines = snapshot
            .values()
            .filter(|status| status.readiness())
            .count() as u64;
        let telemetry_entities = self.telemetry_registry.entity_count() as u64;
        let telemetry_metric_sets = self.telemetry_registry.metric_set_count() as u64;

        self.metrics
            .observed_pipelines_total
            .set(observed_pipelines);
        self.metrics.observed_pipelines_live.set(live_pipelines);
        self.metrics.observed_pipelines_ready.set(ready_pipelines);
        self.metrics.telemetry_entities.set(telemetry_entities);
        self.metrics
            .telemetry_metric_sets
            .set(telemetry_metric_sets);

        let snapshot = self.metrics.snapshot();
        self.telemetry_registry
            .accumulate_metric_set_snapshot(snapshot.key(), snapshot.get_metrics());

        if self.config.log_snapshots {
            otel_info!(
                "controller.monitor.snapshot",
                extension_id = self.extension_id.as_ref(),
                observed_pipelines = observed_pipelines,
                live_pipelines = live_pipelines,
                ready_pipelines = ready_pipelines,
                telemetry_entities = telemetry_entities,
                telemetry_metric_sets = telemetry_metric_sets,
                message = "Controller monitor snapshot"
            );
        }
    }
}

impl Drop for ControllerMonitor {
    fn drop(&mut self) {
        if !self
            .telemetry_registry
            .unregister_metric_set(self.metrics.metric_set_key())
        {
            otel_warn!(
                "controller.monitor.unregister_failed",
                extension_id = self.extension_id.as_ref(),
                message = "Controller monitor metric set was already unregistered"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::observed_state::ObservedStateSettings;
    use otap_df_state::store::ObservedStateStore;
    use std::collections::HashMap;

    fn extension_config(config: serde_json::Value) -> ExtensionUserConfig {
        ExtensionUserConfig {
            r#type: CONTROLLER_MONITOR_EXTENSION_URN.into(),
            description: None,
            config,
        }
    }

    fn monitor_test_parts() -> (
        TelemetryRegistryHandle,
        ObservedStateHandle,
        ControllerMonitorConfig,
    ) {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let observed_state = ObservedStateStore::new(
            &ObservedStateSettings::default(),
            telemetry_registry.clone(),
        )
        .handle();
        (
            telemetry_registry,
            observed_state,
            ControllerMonitorConfig {
                interval: Duration::from_millis(10),
                log_snapshots: false,
            },
        )
    }

    #[test]
    fn controller_monitor_config_defaults_from_null_config() {
        let config =
            ControllerMonitorConfig::from_extension(&extension_config(serde_json::Value::Null))
                .expect("null config should use defaults");
        assert_eq!(config, ControllerMonitorConfig::default());
    }

    #[test]
    fn controller_monitor_config_rejects_unknown_fields() {
        let err = ControllerMonitorConfig::from_extension(&extension_config(serde_json::json!({
            "unknown": true,
        })))
        .expect_err("unknown fields should be rejected");

        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn controller_monitor_config_rejects_zero_interval() {
        let err = ControllerMonitorConfig::from_extension(&extension_config(serde_json::json!({
            "interval": "0s",
        })))
        .expect_err("zero interval should be rejected");

        assert!(matches!(err, ControllerMonitorError::ZeroInterval));
    }

    #[test]
    fn controller_monitor_publishes_snapshot_metrics() {
        let (telemetry_registry, observed_state, config) = monitor_test_parts();
        let mut monitor = ControllerMonitor::new(
            "controller_monitor".into(),
            observed_state,
            telemetry_registry.clone(),
            config,
        );

        monitor.publish_snapshot();

        let mut values = HashMap::new();
        telemetry_registry.visit_current_metrics_with_zeroes(
            |descriptor, _attributes, metrics| {
                if descriptor.name == "controller.monitor" {
                    for (field, value) in metrics {
                        let _ = values.insert(field.name, value.to_u64_lossy());
                    }
                }
            },
            true,
        );

        assert_eq!(values["observed_pipelines.total"], 0);
        assert_eq!(values["observed_pipelines.live"], 0);
        assert_eq!(values["observed_pipelines.ready"], 0);
        assert!(values["telemetry.entities"] >= 1);
        assert!(values["telemetry.metric_sets"] >= 1);
    }

    #[tokio::test]
    async fn controller_monitor_task_stops_on_cancellation() {
        let (telemetry_registry, observed_state, config) = monitor_test_parts();
        let monitor = ControllerMonitor::new(
            "controller_monitor".into(),
            observed_state,
            telemetry_registry,
            config,
        );
        let cancellation_token = CancellationToken::new();
        cancellation_token.cancel();

        monitor
            .run(cancellation_token)
            .await
            .expect("cancelled monitor should stop cleanly");
    }
}
