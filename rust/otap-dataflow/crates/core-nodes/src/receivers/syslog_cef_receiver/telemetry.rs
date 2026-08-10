// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]
use otap_df_telemetry::instrument::{Counter, UpDownCounter};
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SyslogOutcomeAttributes {
    pub outcome: SyslogOutcome,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default, AttributeEnum)]
pub enum SyslogOutcome {
    #[default]
    Forwarded,
    Invalid,
    Truncated,
    ForwardFailed,
    RejectedMemoryPressure,
}

impl SyslogOutcome {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Forwarded => "forwarded",
            Self::Invalid => "invalid",
            Self::Truncated => "truncated",
            Self::ForwardFailed => "forward_failed",
            Self::RejectedMemoryPressure => "rejected_memory_pressure",
        }
    }
}

impl std::fmt::Display for SyslogOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[metric_set(
    name = "receiver.syslog_cef",
    measurement_attributes = SyslogOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct SyslogItemMetrics {
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TcpConnectionAttributes {
    pub state: TcpConnectionState,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default, AttributeEnum)]
pub enum TcpConnectionState {
    #[default]
    Active,
    RejectedMemoryPressure,
}

impl TcpConnectionState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::RejectedMemoryPressure => "rejected_memory_pressure",
        }
    }
}

impl std::fmt::Display for TcpConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[metric_set(
    name = "receiver.syslog_cef.tcp",
    measurement_attributes = TcpConnectionAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct SyslogTcpMetrics {
    #[metric(unit = "{conn}")]
    pub connections: UpDownCounter<f64>,
}

#[metric_set(name = "receiver.syslog_cef")]
#[derive(Debug, Default, Clone)]
pub struct SyslogGlobalMetrics {
    #[metric(unit = "{item}")]
    pub received_logs_total: Counter<u64>,
    #[metric(unit = "{error}")]
    pub tls_handshake_failures: Counter<u64>,
}

/// Aggregate struct holding the registered metric sets.
pub struct SyslogCefReceiverMetrics {
    pub items: MeasurementMetricSet<SyslogItemMetrics>,
    pub tcp_connections: MeasurementMetricSet<SyslogTcpMetrics>,
    pub global: MetricSet<SyslogGlobalMetrics>,
}

impl SyslogCefReceiverMetrics {
    pub fn new(telemetry_registry: &TelemetryRegistryHandle) -> Self {
        Self {
            items: telemetry_registry.register_metric_set_with_measurement_attributes(
                otap_df_telemetry::testing::EmptyAttributes(),
            ),
            tcp_connections: telemetry_registry.register_metric_set_with_measurement_attributes(
                otap_df_telemetry::testing::EmptyAttributes(),
            ),
            global: telemetry_registry.register_metric_set(
                otap_df_telemetry::testing::EmptyAttributes(),
            ),
        }
    }

    pub fn snapshots(&mut self) -> Vec<otap_df_telemetry::metrics::MetricSetSnapshot> {
        let mut snaps = Vec::new();
        snaps.extend(self.items.terminal_snapshots());
        snaps.extend(self.tcp_connections.terminal_snapshots());
        snaps.push(self.global.snapshot());
        snaps
    }

    pub fn report(&mut self, reporter: &mut otap_df_telemetry::reporter::MetricsReporter) {
        let _ = reporter.report_measurement(&mut self.items);
        let _ = reporter.report_measurement(&mut self.tcp_connections);
        let _ = reporter.report(&mut self.global);
    }
}
