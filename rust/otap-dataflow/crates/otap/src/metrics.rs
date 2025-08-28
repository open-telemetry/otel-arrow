// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generic metrics used in the OTAP pipeline.
//!
//! Note: We try as much as possible to follow the following
//! [RFC Pipeline Component Telemetry](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).

use otap_df_config::experimental::SignalType;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// PData metrics for exporters.
/// Namespace grouped under `exporter.pdata`.
/// Uses a monotonic counter per outcome.
#[metric_set(name = "exporter.pdata")]
#[derive(Debug, Default, Clone)]
pub struct ExporterPDataMetrics {
    /// Number of pdata metrics consumed by this exporter.
    #[metric(unit = "{msg}")]
    pub metrics_consumed: Counter<u64>,

    /// Number of pdata metrics successfully exported.
    #[metric(unit = "{msg}")]
    pub metrics_exported: Counter<u64>,

    /// Number of pdata metrics that failed to be exported.
    #[metric(unit = "{msg}")]
    pub metrics_failed: Counter<u64>,

    /// Number of pdata logs consumed by this exporter.
    #[metric(unit = "{msg}")]
    pub logs_consumed: Counter<u64>,

    /// Number of pdata logs successfully exported.
    #[metric(unit = "{msg}")]
    pub logs_exported: Counter<u64>,

    /// Number of pdata logs that failed to be exported.
    #[metric(unit = "{msg}")]
    pub logs_failed: Counter<u64>,

    /// Number of pdata traces consumed by this exporter.
    #[metric(unit = "{msg}")]
    pub traces_consumed: Counter<u64>,

    /// Number of pdata traces successfully exported.
    #[metric(unit = "{msg}")]
    pub traces_exported: Counter<u64>,

    /// Number of pdata traces that failed to be exported.
    #[metric(unit = "{msg}")]
    pub traces_failed: Counter<u64>,
}

impl ExporterPDataMetrics {
    pub fn inc_consumed(&mut self, st: SignalType) {
        match st {
            SignalType::Metrics => self.metrics_consumed.inc(),
            SignalType::Logs => self.logs_consumed.inc(),
            SignalType::Traces => self.traces_consumed.inc(),
        }
    }

    pub fn inc_exported(&mut self, st: SignalType) {
        match st {
            SignalType::Metrics => self.metrics_exported.inc(),
            SignalType::Logs => self.logs_exported.inc(),
            SignalType::Traces => self.traces_exported.inc(),
        }
    }

    pub fn inc_failed(&mut self, st: SignalType) {
        match st {
            SignalType::Metrics => self.metrics_failed.inc(),
            SignalType::Logs => self.logs_failed.inc(),
            SignalType::Traces => self.traces_failed.inc(),
        }
    }
}
