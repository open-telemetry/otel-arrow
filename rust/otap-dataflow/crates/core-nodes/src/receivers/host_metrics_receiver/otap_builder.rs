// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct OTAP Arrow record construction for host metrics.
//!
//! Builds `OtapArrowRecords::Metrics` without constructing intermediate OTLP
//! protobuf objects.  All metric names, units, and attribute keys are written
//! directly into Arrow column buffers.

use arrow::error::ArrowError;
use otap_df_pdata::encode::record::attributes::StrKeysAttributesRecordBatchBuilder;
use otap_df_pdata::encode::record::metrics::{
    MetricsRecordBatchBuilder, NumberDataPointsRecordBatchBuilder,
};
use otap_df_pdata::otap::{Metrics, OtapArrowRecords};
use otap_df_pdata::otlp::metrics::MetricType;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

#[cfg(target_os = "linux")]
use crate::receivers::host_metrics_receiver::procfs::HostResource;
use crate::receivers::host_metrics_receiver::semconv;

const SCOPE_NAME: &[u8] = b"otap-df-core-nodes/host-metrics";
const SCOPE_VERSION: &[u8] = env!("CARGO_PKG_VERSION").as_bytes();

// AggregationTemporality::Cumulative = 2 (OTLP proto enum value).
const AGGREGATION_TEMPORALITY_CUMULATIVE: i32 = 2;

#[derive(Clone, Copy)]
pub(crate) struct MetricHandle(u16);

/// Wraps the per-datapoint attribute builder and hides the dp_id from callers.
pub(crate) struct DpAttrWriter<'a> {
    attrs: &'a mut StrKeysAttributesRecordBatchBuilder<u32>,
    dp_id: u32,
}

impl DpAttrWriter<'_> {
    /// Append a string-valued attribute.
    pub fn str(&mut self, key: &'static str, value: &str) {
        self.attrs.append_parent_id(&self.dp_id);
        self.attrs.append_key(key);
        self.attrs.any_values_builder.append_str(value.as_bytes());
    }

    /// Append an integer-valued attribute.
    pub fn int(&mut self, key: &'static str, value: i64) {
        self.attrs.append_parent_id(&self.dp_id);
        self.attrs.append_key(key);
        self.attrs.any_values_builder.append_int(value);
    }
}

/// Wraps the resource attribute builder and hides the resource_id from callers.
pub(crate) struct ResourceAttrWriter<'a> {
    attrs: &'a mut StrKeysAttributesRecordBatchBuilder<u16>,
}

impl ResourceAttrWriter<'_> {
    fn str(&mut self, key: &'static str, value: &str) {
        self.attrs.append_parent_id(&0u16);
        self.attrs.append_key(key);
        self.attrs.any_values_builder.append_str(value.as_bytes());
    }
}

/// Builds an `OtapArrowRecords::Metrics` batch directly from host metric values.
///
/// Call a `begin_*` method to open a metric, then one of the typed data point
/// appenders for each value.
/// Call [`finish`] to produce the final batch.
pub(crate) struct HostMetricsArrowBuilder {
    metrics: MetricsRecordBatchBuilder,
    ndp: NumberDataPointsRecordBatchBuilder,
    resource_attrs: StrKeysAttributesRecordBatchBuilder<u16>,
    ndp_attrs: StrKeysAttributesRecordBatchBuilder<u32>,
    curr_metric_id: u16,
    curr_dp_id: u32,
    resource_appended: bool,
}

impl HostMetricsArrowBuilder {
    pub(crate) fn new() -> Self {
        Self {
            metrics: MetricsRecordBatchBuilder::new(),
            ndp: NumberDataPointsRecordBatchBuilder::new(),
            resource_attrs: StrKeysAttributesRecordBatchBuilder::new(),
            ndp_attrs: StrKeysAttributesRecordBatchBuilder::new(),
            curr_metric_id: 0,
            curr_dp_id: 0,
            resource_appended: false,
        }
    }

    /// Append resource attributes (host.id, host.name, host.arch, os.type).
    /// Must be called exactly once per batch before any metrics are appended.
    #[cfg(target_os = "linux")]
    pub(crate) fn append_resource(&mut self, resource: &HostResource) {
        debug_assert!(!self.resource_appended, "resource already appended");
        self.resource_appended = true;
        let mut w = ResourceAttrWriter {
            attrs: &mut self.resource_attrs,
        };
        w.str(semconv::attr::OS_TYPE, "linux");
        if let Some(id) = &resource.host_id {
            w.str(semconv::attr::HOST_ID, id);
        }
        if let Some(name) = &resource.host_name {
            w.str(semconv::attr::HOST_NAME, name);
        }
        if let Some(arch) = resource.host_arch {
            w.str(semconv::attr::HOST_ARCH, arch);
        }
    }

    // ── Metric openers ──────────────────────────────────────────────────────

    /// Open a monotonic cumulative Sum metric (i64 data points).
    pub(crate) fn begin_counter_i64(&mut self, name: &str, unit: &str) -> MetricHandle {
        self.begin_metric(name, unit, MetricType::Sum, true)
    }

    /// Open a monotonic cumulative Sum metric (f64 data points).
    pub(crate) fn begin_counter_f64(&mut self, name: &str, unit: &str) -> MetricHandle {
        self.begin_metric(name, unit, MetricType::Sum, true)
    }

    /// Open a non-monotonic cumulative Sum metric / UpDownCounter (i64 data points).
    pub(crate) fn begin_updown_i64(&mut self, name: &str, unit: &str) -> MetricHandle {
        self.begin_metric(name, unit, MetricType::Sum, false)
    }

    /// Open a Gauge metric (f64 data points).
    pub(crate) fn begin_gauge_f64(&mut self, name: &str, unit: &str) -> MetricHandle {
        self.begin_metric(name, unit, MetricType::Gauge, false)
    }

    /// Open a Gauge metric (i64 data points).
    pub(crate) fn begin_gauge_i64(&mut self, name: &str, unit: &str) -> MetricHandle {
        self.begin_metric(name, unit, MetricType::Gauge, false)
    }

    fn begin_metric(
        &mut self,
        name: &str,
        unit: &str,
        metric_type: MetricType,
        is_monotonic: bool,
    ) -> MetricHandle {
        let id = self.curr_metric_id;
        self.metrics.append_id(id);
        self.metrics.append_metric_type(metric_type as u8);
        self.metrics.append_name(name.as_bytes());
        self.metrics.append_description(&[]);
        self.metrics.append_unit(unit.as_bytes());
        match metric_type {
            MetricType::Sum => {
                self.metrics
                    .append_aggregation_temporality(Some(AGGREGATION_TEMPORALITY_CUMULATIVE));
                self.metrics.append_is_monotonic(Some(is_monotonic));
            }
            _ => {
                self.metrics.append_aggregation_temporality(None);
                self.metrics.append_is_monotonic(None);
            }
        }
        self.curr_metric_id = self
            .curr_metric_id
            .checked_add(1)
            .expect("metric_id overflow: more than u16::MAX metrics in one batch");
        MetricHandle(id)
    }

    // ── Datapoint appenders ─────────────────────────────────────────────────

    /// Append one i64 Sum data point.
    pub(crate) fn append_i64_sum_dp<F>(
        &mut self,
        metric: MetricHandle,
        start: u64,
        now: u64,
        value: i64,
        attrs: F,
    ) where
        F: FnOnce(&mut DpAttrWriter<'_>),
    {
        self.append_i64_dp(metric, Some(start), now, value, attrs);
    }

    /// Append one i64 Gauge data point.
    pub(crate) fn append_i64_gauge_dp<F>(
        &mut self,
        metric: MetricHandle,
        now: u64,
        value: i64,
        attrs: F,
    ) where
        F: FnOnce(&mut DpAttrWriter<'_>),
    {
        self.append_i64_dp(metric, None, now, value, attrs);
    }

    fn append_i64_dp<F>(
        &mut self,
        metric: MetricHandle,
        start: Option<u64>,
        now: u64,
        value: i64,
        attrs: F,
    ) where
        F: FnOnce(&mut DpAttrWriter<'_>),
    {
        let dp_id = self.curr_dp_id;
        self.ndp.append_id(dp_id);
        self.ndp.append_parent_id(metric.0);
        self.ndp
            .append_start_time_unix_nano(start.map(|v| v as i64));
        self.ndp.append_time_unix_nano(now as i64);
        self.ndp.append_int_value(Some(value));
        self.ndp.append_double_value(None);
        self.ndp.append_flags(0);
        let mut w = DpAttrWriter {
            attrs: &mut self.ndp_attrs,
            dp_id,
        };
        attrs(&mut w);
        self.curr_dp_id = self
            .curr_dp_id
            .checked_add(1)
            .expect("dp_id overflow: more than u32::MAX datapoints in one batch");
    }

    /// Append one f64 Sum data point.
    pub(crate) fn append_f64_sum_dp<F>(
        &mut self,
        metric: MetricHandle,
        start: u64,
        now: u64,
        value: f64,
        attrs: F,
    ) where
        F: FnOnce(&mut DpAttrWriter<'_>),
    {
        self.append_f64_dp(metric, Some(start), now, value, attrs);
    }

    /// Append one f64 Gauge data point.
    pub(crate) fn append_f64_gauge_dp<F>(
        &mut self,
        metric: MetricHandle,
        now: u64,
        value: f64,
        attrs: F,
    ) where
        F: FnOnce(&mut DpAttrWriter<'_>),
    {
        self.append_f64_dp(metric, None, now, value, attrs);
    }

    fn append_f64_dp<F>(
        &mut self,
        metric: MetricHandle,
        start: Option<u64>,
        now: u64,
        value: f64,
        attrs: F,
    ) where
        F: FnOnce(&mut DpAttrWriter<'_>),
    {
        let dp_id = self.curr_dp_id;
        self.ndp.append_id(dp_id);
        self.ndp.append_parent_id(metric.0);
        self.ndp
            .append_start_time_unix_nano(start.map(|v| v as i64));
        self.ndp.append_time_unix_nano(now as i64);
        self.ndp.append_int_value(None);
        self.ndp.append_double_value(Some(value));
        self.ndp.append_flags(0);
        let mut w = DpAttrWriter {
            attrs: &mut self.ndp_attrs,
            dp_id,
        };
        attrs(&mut w);
        self.curr_dp_id = self
            .curr_dp_id
            .checked_add(1)
            .expect("dp_id overflow: more than u32::MAX datapoints in one batch");
    }

    // ── Finalization ─────────────────────────────────────────────────────────

    /// Finalize all builders and produce an `OtapArrowRecords::Metrics` batch.
    pub(crate) fn finish(mut self) -> Result<OtapArrowRecords, ArrowError> {
        let n = self.metrics.len();
        // Resource: single entry with id=0 repeated for every metric row.
        self.metrics.resource.append_id_n(0, n);
        self.metrics
            .resource
            .append_schema_url_n(Some(semconv::SCHEMA_URL), n);
        self.metrics
            .resource
            .append_dropped_attributes_count_n(0, n);
        // Scope: single entry with id=0.
        self.metrics.scope.append_id_n(0, n);
        self.metrics.scope.append_name_n(Some(SCOPE_NAME), n);
        self.metrics.scope.append_version_n(Some(SCOPE_VERSION), n);
        self.metrics.scope.append_dropped_attributes_count_n(0, n);
        // Schema URL on scope column.
        self.metrics
            .append_scope_schema_url_n(semconv::SCHEMA_URL, n);

        let mut records = OtapArrowRecords::Metrics(Metrics::default());
        finish_batch(
            &mut records,
            ArrowPayloadType::UnivariateMetrics,
            self.metrics.finish()?,
        )?;
        finish_batch(
            &mut records,
            ArrowPayloadType::NumberDataPoints,
            self.ndp.finish()?,
        )?;
        finish_batch(
            &mut records,
            ArrowPayloadType::ResourceAttrs,
            self.resource_attrs.finish()?,
        )?;
        finish_batch(
            &mut records,
            ArrowPayloadType::NumberDpAttrs,
            self.ndp_attrs.finish()?,
        )?;
        Ok(records)
    }
}

fn finish_batch(
    records: &mut OtapArrowRecords,
    payload_type: ArrowPayloadType,
    rb: arrow::array::RecordBatch,
) -> Result<(), ArrowError> {
    if rb.num_rows() > 0 {
        records
            .set(payload_type, rb)
            .map_err(|e| ArrowError::ExternalError(Box::new(e)))?;
    }
    Ok(())
}
