#![allow(missing_docs)]
use otap_df_telemetry::instrument::{Counter, UpDownCounter};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry_macros::{attribute_set, metric_set};

#[attribute_set(name = "outcome")]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SyslogOutcomeAttributes {
    pub outcome: SyslogOutcome,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
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
    registration_attributes = "SyslogOutcomeAttributes"
)]
#[derive(Debug, Default, Clone)]
pub struct SyslogItemMetrics {
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
}

#[attribute_set(name = "state")]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TcpConnectionAttributes {
    pub state: TcpConnectionState,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
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
    registration_attributes = "TcpConnectionAttributes"
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
    pub forwarded: MetricSet<SyslogItemMetrics>,
    pub invalid: MetricSet<SyslogItemMetrics>,
    pub truncated: MetricSet<SyslogItemMetrics>,
    pub forward_failed: MetricSet<SyslogItemMetrics>,
    pub rejected_memory_pressure: MetricSet<SyslogItemMetrics>,

    pub tcp_active: MetricSet<SyslogTcpMetrics>,
    pub tcp_rejected: MetricSet<SyslogTcpMetrics>,

    pub global: MetricSet<SyslogGlobalMetrics>,
}

impl SyslogCefReceiverMetrics {
    pub fn new(telemetry_registry: &TelemetryRegistryHandle) -> Self {
        Self {
            forwarded: telemetry_registry.register_metric_set(SyslogOutcomeAttributes {
                outcome: SyslogOutcome::Forwarded,
            }),
            invalid: telemetry_registry.register_metric_set(SyslogOutcomeAttributes {
                outcome: SyslogOutcome::Invalid,
            }),
            truncated: telemetry_registry.register_metric_set(SyslogOutcomeAttributes {
                outcome: SyslogOutcome::Truncated,
            }),
            forward_failed: telemetry_registry.register_metric_set(SyslogOutcomeAttributes {
                outcome: SyslogOutcome::ForwardFailed,
            }),
            rejected_memory_pressure: telemetry_registry.register_metric_set(SyslogOutcomeAttributes {
                outcome: SyslogOutcome::RejectedMemoryPressure,
            }),

            tcp_active: telemetry_registry.register_metric_set(TcpConnectionAttributes {
                state: TcpConnectionState::Active,
            }),
            tcp_rejected: telemetry_registry.register_metric_set(TcpConnectionAttributes {
                state: TcpConnectionState::RejectedMemoryPressure,
            }),

            global: telemetry_registry.register_metric_set(otap_df_telemetry::testing::EmptyAttributes()),
        }
    }

    pub fn snapshots(&self) -> Vec<otap_df_telemetry::metrics::MetricSetSnapshot> {
        vec![
            self.forwarded.snapshot(),
            self.invalid.snapshot(),
            self.truncated.snapshot(),
            self.forward_failed.snapshot(),
            self.rejected_memory_pressure.snapshot(),
            self.tcp_active.snapshot(),
            self.tcp_rejected.snapshot(),
            self.global.snapshot(),
        ]
    }

    pub fn report(&mut self, reporter: &mut otap_df_telemetry::reporter::MetricsReporter) {
        let _ = reporter.report(&mut self.forwarded);
        let _ = reporter.report(&mut self.invalid);
        let _ = reporter.report(&mut self.truncated);
        let _ = reporter.report(&mut self.forward_failed);
        let _ = reporter.report(&mut self.rejected_memory_pressure);
        let _ = reporter.report(&mut self.tcp_active);
        let _ = reporter.report(&mut self.tcp_rejected);
        let _ = reporter.report(&mut self.global);
    }
}
