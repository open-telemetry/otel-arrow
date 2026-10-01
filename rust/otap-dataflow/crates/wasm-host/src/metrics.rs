// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the WASM processor node.

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::SignalAttributes;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::metric_set;

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

    // ---- host-services counter tracking ----
    /// Number of guest `counter-add` calls accepted (name within both the
    /// length and cardinality bounds).
    #[metric(unit = "{item}")]
    pub guest_counter_add_calls: Counter<u64>,
    /// Number of guest `counter-add` calls rejected (silently no-op'd)
    /// because the counter name exceeded the host's maximum name length.
    /// Distinct from the cardinality rejection below: this one means the
    /// plugin is emitting a malformed name.
    #[metric(unit = "{item}")]
    pub guest_counter_add_rejected_name_len: Counter<u64>,
    /// Number of guest `counter-add` calls rejected (silently no-op'd)
    /// because accepting the name would exceed the host's distinct-counter
    /// cardinality bound. Means the plugin is tracking more counters than the
    /// host allows, and some of its counts are being lost.
    #[metric(unit = "{item}")]
    pub guest_counter_add_rejected_cardinality: Counter<u64>,
    /// Sum of values accepted through guest `counter-add` calls.
    #[metric(unit = "{item}")]
    pub guest_counter_add_value: Counter<u64>,
    /// Number of guest `log`/`counter-add`/`get-config` calls rejected
    /// (silently no-op'd) because the shared token-bucket rate limiter was
    /// empty. The limiter is scoped to the plugin instance's whole lifetime,
    /// not reset per `initialize`/`process`/`shutdown` call. A non-zero value
    /// is how an operator tells a throttled plugin apart from a quiet one.
    #[metric(unit = "{item}")]
    pub guest_host_service_calls_rejected: Counter<u64>,
    /// Number of guest `log` messages truncated to the host's maximum
    /// retained length before being forwarded to telemetry.
    #[metric(unit = "{item}")]
    pub guest_log_message_truncated: Counter<u64>,
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
