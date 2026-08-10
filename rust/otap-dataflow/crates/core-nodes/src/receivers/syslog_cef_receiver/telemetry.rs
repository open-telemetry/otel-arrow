// Copyright 2024, OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(missing_docs)]
use otap_df_telemetry::instrument::{Counter, UpDownCounter};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry_macros::{attribute_set, metric_set};

#[attribute_set(name = "outcome")]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SyslogOutcomeAttributes {
    #[attribute]
    pub outcome: std::borrow::Cow<'static, str>,
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
    #[attribute]
    pub state: std::borrow::Cow<'static, str>,
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
                outcome: std::borrow::Cow::Borrowed("forwarded"),
            }),
            invalid: telemetry_registry.register_metric_set(SyslogOutcomeAttributes {
                outcome: std::borrow::Cow::Borrowed("invalid"),
            }),
            truncated: telemetry_registry.register_metric_set(SyslogOutcomeAttributes {
                outcome: std::borrow::Cow::Borrowed("truncated"),
            }),
            forward_failed: telemetry_registry.register_metric_set(SyslogOutcomeAttributes {
                outcome: std::borrow::Cow::Borrowed("forward_failed"),
            }),
            rejected_memory_pressure: telemetry_registry.register_metric_set(
                SyslogOutcomeAttributes {
                    outcome: std::borrow::Cow::Borrowed("rejected_memory_pressure"),
                },
            ),

            tcp_active: telemetry_registry.register_metric_set(TcpConnectionAttributes {
                state: std::borrow::Cow::Borrowed("active"),
            }),
            tcp_rejected: telemetry_registry.register_metric_set(TcpConnectionAttributes {
                state: std::borrow::Cow::Borrowed("rejected_memory_pressure"),
            }),

            global: telemetry_registry
                .register_metric_set(otap_df_telemetry::testing::EmptyAttributes()),
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
