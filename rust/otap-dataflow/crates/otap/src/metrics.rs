// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generic metrics used in the OTAP pipeline.
//!
//! Note: We try as much as possible to follow the following
//! [RFC Pipeline Component Telemetry](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).

use otap_df_config::SignalType;
use otap_df_telemetry::instrument::DeltaCounter;
use otap_df_telemetry_macros::metric_set;

/// PData metrics for exporters.
/// Namespace grouped under `exporter.pdata`.
/// Uses a monotonic counter per outcome.
#[metric_set(name = "exporter.pdata")]
#[derive(Debug, Default, Clone)]
pub struct ExporterPDataMetrics {
    /// Number of pdata metrics consumed by this exporter.
    #[metric(unit = "{msg}")]
    pub metrics_consumed: DeltaCounter<u64>,

    /// Number of pdata metrics successfully exported.
    #[metric(unit = "{msg}")]
    pub metrics_exported: DeltaCounter<u64>,

    /// Number of pdata metrics that failed to be exported.
    #[metric(unit = "{msg}")]
    pub metrics_failed: DeltaCounter<u64>,

    /// Number of pdata logs consumed by this exporter.
    #[metric(unit = "{msg}")]
    pub logs_consumed: DeltaCounter<u64>,

    /// Number of pdata logs successfully exported.
    #[metric(unit = "{msg}")]
    pub logs_exported: DeltaCounter<u64>,

    /// Number of pdata logs that failed to be exported.
    #[metric(unit = "{msg}")]
    pub logs_failed: DeltaCounter<u64>,

    /// Number of pdata traces consumed by this exporter.
    #[metric(unit = "{msg}")]
    pub traces_consumed: DeltaCounter<u64>,

    /// Number of pdata traces successfully exported.
    #[metric(unit = "{msg}")]
    pub traces_exported: DeltaCounter<u64>,

    /// Number of pdata traces that failed to be exported.
    #[metric(unit = "{msg}")]
    pub traces_failed: DeltaCounter<u64>,
}

impl ExporterPDataMetrics {
    /// Increment the number of pdata messages consumed.
    pub fn inc_consumed(&mut self, st: SignalType) {
        self.add_consumed(st, 1);
    }

    /// Increment the number of pdata messages exported.
    pub fn inc_exported(&mut self, st: SignalType) {
        self.add_exported(st, 1);
    }

    /// Increment the number of pdata messages that failed to be exported.
    pub fn inc_failed(&mut self, st: SignalType) {
        self.add_failed(st, 1);
    }

    /// Add to the number of of pdata messages consumed.
    pub fn add_consumed(&mut self, st: SignalType, count: u64) {
        if count == 0 {
            return;
        }
        match st {
            SignalType::Metrics => self.metrics_consumed.add(count),
            SignalType::Logs => self.logs_consumed.add(count),
            SignalType::Traces => self.traces_consumed.add(count),
        }
    }

    /// Add to the number of pdata messages exported.
    pub fn add_exported(&mut self, st: SignalType, count: u64) {
        if count == 0 {
            return;
        }
        match st {
            SignalType::Metrics => self.metrics_exported.add(count),
            SignalType::Logs => self.logs_exported.add(count),
            SignalType::Traces => self.traces_exported.add(count),
        }
    }

    /// Add to the number of pdata messages that failed to be exported.
    pub fn add_failed(&mut self, st: SignalType, count: u64) {
        if count == 0 {
            return;
        }
        match st {
            SignalType::Metrics => self.metrics_failed.add(count),
            SignalType::Logs => self.logs_failed.add(count),
            SignalType::Traces => self.traces_failed.add(count),
        }
    }
}
