// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the WASM processor node.

use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::common_attributes::SignalAttributes;
use otap_df_telemetry::error::Error;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

/// Non-signal-partitioned operational metrics for the WASM processor node.
#[metric_set(name = "processor.wasm_processor.pdata")]
#[derive(Debug, Default, Clone)]
pub struct WasmProcessorMetrics {
    // ---- guest process call tracking ----
    /// Number of guest `process` calls attempted.
    #[metric(unit = "{item}")]
    pub guest_process_calls: Counter<u64>,
    /// Number of guest `process` calls that failed or trapped.
    #[metric(unit = "{item}")]
    pub guest_process_errors: Counter<u64>,
    /// Number of pdata messages intentionally dropped by guest `process` returning `none`.
    #[metric(unit = "{item}")]
    pub pdata_dropped: Counter<u64>,

    // ---- host kernel call tracking ----
    /// Total host kernel invocations dispatched by the guest.
    #[metric(unit = "{item}")]
    pub kernel_calls: Counter<u64>,
}

/// Record throughput metrics partitioned by OpenTelemetry signal type.
#[metric_set(
    name = "processor.wasm_processor.pdata.records",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct WasmProcessorRecordMetrics {
    /// Records entering the guest (root batch row count).
    #[metric(unit = "{item}")]
    pub records_in: Counter<u64>,
    /// Records leaving the guest after filtering.
    #[metric(unit = "{item}")]
    pub records_out: Counter<u64>,
}

/// All metrics emitted by the WASM processor node.
pub struct WasmProcessorAllMetrics {
    /// Non-signal operational counters.
    pub pdata: MetricSet<WasmProcessorMetrics>,
    /// Signal-partitioned record throughput counters.
    pub records: MeasurementMetricSet<WasmProcessorRecordMetrics>,
}

impl WasmProcessorAllMetrics {
    /// Register all WASM processor metric sets against the pipeline context.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            pdata: WasmProcessorMetrics::register(pipeline_ctx),
            records: WasmProcessorRecordMetrics::register(pipeline_ctx),
        }
    }

    /// Report all metric sets to the provided reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter
            .report(&mut self.pdata)
            .and_then(|()| reporter.report_measurement(&mut self.records))
    }

    /// Return the record throughput counters partitioned by `signal`.
    pub fn records_for(&mut self, signal: SignalType) -> &mut WasmProcessorRecordMetrics {
        self.records.with(SignalAttributes { signal })
    }
}
