// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics batch accumulator for temporal reaggregation.
//!
//! This module provides [`MetricAggregator`], which buffers incoming metrics
//! data, deduplicates resources/scopes/metrics by identity, and tracks the
//! latest data point per stream. On flush it produces an [`OtapArrowRecords`]
//! batch from the accumulated state.

use hashbrown::HashMap;
use otap_df_pdata::encode::append_attribute_value;
use otap_df_pdata::encode::record::attributes::AttributesRecordBatchBuilder;
use otap_df_pdata::encode::record::metrics::MetricsRecordBatchBuilder;
use otap_df_pdata::otap::{Metrics, OtapArrowRecords};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::views::otap::OtapMetricsView;
use otap_df_pdata::views::otap::common::OtapAttributeView;
use otap_df_pdata_views::views::common::InstrumentationScopeView;
use otap_df_pdata_views::views::metrics::{
    AggregationTemporality, DataType, DataView, ExponentialHistogramDataPointView,
    ExponentialHistogramView, GaugeView, HistogramDataPointView, HistogramView, MetricView,
    MetricsView, NumberDataPointView, ResourceMetricsView, ScopeMetricsView, SumView,
    SummaryDataPointView, SummaryView,
};
use otap_df_pdata_views::views::resource::ResourceView;

use super::data_points::{
    ExpHistogramDataPointColumns, HistogramDataPointColumns, NumberDataPointColumns,
    SummaryDataPointColumns,
};
use super::identity::{
    AttributeHash, AttributeHashBuffer, MetricId, MetricIdRef, ResourceId, ScopeId, ScopeIdRef,
    StreamId, StreamIdRef, metric_id_of, resource_id_of, scope_id_of, stream_id_of,
};

// ---------------------------------------------------------------------------
// Aggregatability check
// ---------------------------------------------------------------------------

fn is_aggregatable(metric_id: &MetricId<'_>) -> bool {
    let data_type = metric_id.data_type;
    let temporality = metric_id.aggregation_temporality;
    let monotonic = metric_id.is_monotonic;

    if data_type == DataType::Gauge as u8 || data_type == DataType::Summary as u8 {
        return true;
    }

    if data_type == DataType::Sum as u8
        && temporality == AggregationTemporality::Cumulative as u8
        && monotonic
    {
        return true;
    }

    if (data_type == DataType::Histogram as u8 || data_type == DataType::ExponentialHistogram as u8)
        && temporality == AggregationTemporality::Cumulative as u8
    {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Metadata stored alongside OTAP IDs
// ---------------------------------------------------------------------------

/// Metadata for a resource, indexed by OTAP resource ID.
struct ResourceMeta {
    schema_url: Vec<u8>,
    dropped_attributes_count: u32,
}

/// Metadata for a scope, indexed by OTAP scope ID.
struct ScopeMeta {
    name: Vec<u8>,
    version: Vec<u8>,
    dropped_attributes_count: u32,
}

// ---------------------------------------------------------------------------
// Stream state
// ---------------------------------------------------------------------------

struct StreamState {
    dp_row_index: usize,
    #[allow(dead_code)]
    dp_type: DpType,
    time_unix_nano: u64,
}

#[derive(Clone, Copy)]
enum DpType {
    Number,
    Histogram,
    ExponentialHistogram,
    Summary,
}

// ---------------------------------------------------------------------------
// MetricAggregator
// ---------------------------------------------------------------------------

/// Accumulates metrics data over a collection interval.
pub struct MetricAggregator {
    hash_buf: AttributeHashBuffer<OtapAttributeView<'static>>,

    // Identity maps
    resource_ids: HashMap<ResourceId, u16>,
    next_resource_id: u16,
    scope_ids: HashMap<ScopeId<'static>, u16>,
    next_scope_id: u16,
    metric_ids: HashMap<MetricId<'static>, u16>,
    next_metric_id: u16,

    // Metadata indexed by OTAP ID
    resource_meta: Vec<ResourceMeta>,
    scope_meta: Vec<ScopeMeta>,

    // Metric-level builder (append-only)
    metrics_builder: MetricsRecordBatchBuilder,

    // Attribute builders (append-only)
    resource_attrs_builder: AttributesRecordBatchBuilder<u16>,
    scope_attrs_builder: AttributesRecordBatchBuilder<u16>,
    ndp_attrs_builder: AttributesRecordBatchBuilder<u32>,
    hdp_attrs_builder: AttributesRecordBatchBuilder<u32>,
    ehdp_attrs_builder: AttributesRecordBatchBuilder<u32>,
    summary_attrs_builder: AttributesRecordBatchBuilder<u32>,

    // Stream tracking
    stream_map: HashMap<StreamId<'static>, StreamState>,
    next_ndp_id: u32,
    next_hdp_id: u32,
    next_ehdp_id: u32,
    next_sdp_id: u32,

    // Data point SOA storage
    number_dps: NumberDataPointColumns,
    histogram_dps: HistogramDataPointColumns,
    exp_histogram_dps: ExpHistogramDataPointColumns,
    summary_dps: SummaryDataPointColumns,
}

impl MetricAggregator {
    pub fn new() -> Self {
        Self {
            hash_buf: AttributeHashBuffer::new(),

            resource_ids: HashMap::new(),
            scope_ids: HashMap::new(),
            metric_ids: HashMap::new(),
            stream_map: HashMap::new(),

            next_resource_id: 0,
            next_scope_id: 0,
            next_metric_id: 0,
            next_ndp_id: 0,
            next_hdp_id: 0,
            next_ehdp_id: 0,
            next_sdp_id: 0,

            resource_meta: Vec::new(),
            scope_meta: Vec::new(),

            metrics_builder: MetricsRecordBatchBuilder::new(),
            resource_attrs_builder: AttributesRecordBatchBuilder::new(),
            scope_attrs_builder: AttributesRecordBatchBuilder::new(),
            ndp_attrs_builder: AttributesRecordBatchBuilder::new(),
            hdp_attrs_builder: AttributesRecordBatchBuilder::new(),
            ehdp_attrs_builder: AttributesRecordBatchBuilder::new(),
            summary_attrs_builder: AttributesRecordBatchBuilder::new(),

            number_dps: NumberDataPointColumns::new(),
            histogram_dps: HistogramDataPointColumns::new(),
            exp_histogram_dps: ExpHistogramDataPointColumns::new(),
            summary_dps: SummaryDataPointColumns::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Attribute hashing
    // -----------------------------------------------------------------------

    fn compute_otap_attr_hash<'v>(
        &mut self,
        attrs: impl Iterator<Item = OtapAttributeView<'v>>,
    ) -> AttributeHash {
        let hash_buf = std::mem::replace(&mut self.hash_buf, AttributeHashBuffer::new());
        let mut hash_buf: AttributeHashBuffer<OtapAttributeView<'_>> = hash_buf.recycle();
        let hash = AttributeHash::of(&mut hash_buf, attrs);
        self.hash_buf = hash_buf.recycle();
        hash
    }

    // -----------------------------------------------------------------------
    // Ingestion
    // -----------------------------------------------------------------------

    pub fn ingest(&mut self, view: OtapMetricsView<'_>) {
        for resource_metrics in view.resources() {
            let Some(resource) = resource_metrics.resource() else {
                continue;
            };

            let attrs = self.compute_otap_attr_hash(resource.attributes());
            let resource_id = resource_id_of(attrs);
            let otap_resource_id = self.process_resource(
                resource_id,
                &resource,
                resource_metrics.schema_url().unwrap_or(b""),
            );

            for scope_metrics in resource_metrics.scopes() {
                let Some(scope) = scope_metrics.scope() else {
                    continue;
                };

                let attrs = self.compute_otap_attr_hash(scope.attributes());
                let scope_id = scope_id_of(resource_id, &scope, attrs);
                let scope_schema_url = scope_metrics.schema_url();
                let otap_scope_id = self.process_scope(scope_id.clone(), &scope);

                for metric in scope_metrics.metrics() {
                    let Some(metric_id) = metric_id_of(scope_id.clone(), &metric) else {
                        continue;
                    };

                    if !is_aggregatable(&metric_id) {
                        // TODO: Build passthrough batch.
                        continue;
                    }

                    let otap_metric_id = self.process_metric(
                        metric_id.clone(),
                        &metric,
                        otap_resource_id,
                        otap_scope_id,
                        scope_schema_url,
                    );

                    let Some(data) = metric.data() else {
                        continue;
                    };

                    match data.value_type() {
                        DataType::Gauge => {
                            if let Some(gauge) = data.as_gauge() {
                                for dp in gauge.data_points() {
                                    let attrs = self.compute_otap_attr_hash(dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.ingest_number_dp(
                                        &dp,
                                        stream_id,
                                        otap_metric_id,
                                        DpType::Number,
                                    );
                                }
                            }
                        }
                        DataType::Sum => {
                            if let Some(sum) = data.as_sum() {
                                for dp in sum.data_points() {
                                    let attrs = self.compute_otap_attr_hash(dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.ingest_number_dp(
                                        &dp,
                                        stream_id,
                                        otap_metric_id,
                                        DpType::Number,
                                    );
                                }
                            }
                        }
                        DataType::Histogram => {
                            if let Some(hist) = data.as_histogram() {
                                for dp in hist.data_points() {
                                    let attrs = self.compute_otap_attr_hash(dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.ingest_histogram_dp(&dp, stream_id, otap_metric_id);
                                }
                            }
                        }
                        DataType::ExponentialHistogram => {
                            if let Some(exp) = data.as_exponential_histogram() {
                                for dp in exp.data_points() {
                                    let attrs = self.compute_otap_attr_hash(dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.ingest_exp_histogram_dp(&dp, stream_id, otap_metric_id);
                                }
                            }
                        }
                        DataType::Summary => {
                            if let Some(summary) = data.as_summary() {
                                for dp in summary.data_points() {
                                    let attrs = self.compute_otap_attr_hash(dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.ingest_summary_dp(&dp, stream_id, otap_metric_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Flush
    // -----------------------------------------------------------------------

    pub fn flush(&mut self) -> Option<OtapArrowRecords> {
        if self.is_empty() {
            return None;
        }

        let mut records = OtapArrowRecords::Metrics(Metrics::default());

        // Metrics table
        if let Ok(rb) = self.metrics_builder.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::UnivariateMetrics, rb);
            }
        }

        // Attribute tables
        if let Ok(rb) = self.resource_attrs_builder.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::ResourceAttrs, rb);
            }
        }
        if let Ok(rb) = self.scope_attrs_builder.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::ScopeAttrs, rb);
            }
        }

        // Data point tables
        if let Ok(rb) = self.number_dps.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::NumberDataPoints, rb);
            }
        }
        if let Ok(rb) = self.ndp_attrs_builder.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::NumberDpAttrs, rb);
            }
        }
        if let Ok(rb) = self.histogram_dps.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::HistogramDataPoints, rb);
            }
        }
        if let Ok(rb) = self.hdp_attrs_builder.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::HistogramDpAttrs, rb);
            }
        }
        if let Ok(rb) = self.exp_histogram_dps.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::ExpHistogramDataPoints, rb);
            }
        }
        if let Ok(rb) = self.ehdp_attrs_builder.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::ExpHistogramDpAttrs, rb);
            }
        }
        if let Ok(rb) = self.summary_dps.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::SummaryDataPoints, rb);
            }
        }
        if let Ok(rb) = self.summary_attrs_builder.finish() {
            if rb.num_rows() > 0 {
                let _ = records.set(ArrowPayloadType::SummaryDpAttrs, rb);
            }
        }

        self.clear();
        Some(records)
    }

    pub fn stream_count(&self) -> usize {
        self.stream_map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stream_map.is_empty()
    }

    // -----------------------------------------------------------------------
    // Identity processing
    // -----------------------------------------------------------------------

    fn process_resource<R: ResourceView>(
        &mut self,
        resource_id: ResourceId,
        view: &R,
        schema_url: &[u8],
    ) -> u16 {
        if let Some(&id) = self.resource_ids.get(&resource_id) {
            return id;
        }
        let id = self.next_resource_id;
        self.next_resource_id += 1;

        // Write resource attributes
        for attr in view.attributes() {
            self.resource_attrs_builder.append_parent_id(&id);
            let _ = append_attribute_value(&mut self.resource_attrs_builder, &attr);
        }

        // Store metadata for use when writing metric rows
        self.resource_meta.push(ResourceMeta {
            schema_url: schema_url.to_vec(),
            dropped_attributes_count: view.dropped_attributes_count(),
        });

        _ = self.resource_ids.insert(resource_id, id);
        id
    }

    fn process_scope<S: InstrumentationScopeView>(
        &mut self,
        scope_id: ScopeId<'_>,
        view: &S,
    ) -> u16 {
        if let Some(&id) = self.scope_ids.get(&ScopeIdRef(&scope_id)) {
            return id;
        }
        let id = self.next_scope_id;
        self.next_scope_id += 1;

        // Write scope attributes
        for attr in view.attributes() {
            self.scope_attrs_builder.append_parent_id(&id);
            let _ = append_attribute_value(&mut self.scope_attrs_builder, &attr);
        }

        // Store metadata for use when writing metric rows
        self.scope_meta.push(ScopeMeta {
            name: view.name().unwrap_or(b"").to_vec(),
            version: view.version().unwrap_or(b"").to_vec(),
            dropped_attributes_count: view.dropped_attributes_count(),
        });

        _ = self.scope_ids.insert(scope_id.into_owned(), id);
        id
    }

    fn process_metric<M: MetricView>(
        &mut self,
        metric_id: MetricId<'_>,
        view: &M,
        otap_resource_id: u16,
        otap_scope_id: u16,
        scope_schema_url: &[u8],
    ) -> u16 {
        if let Some(&id) = self.metric_ids.get(&MetricIdRef(&metric_id)) {
            return id;
        }
        let id = self.next_metric_id;
        self.next_metric_id += 1;

        // Look up stored metadata
        let resource_meta = &self.resource_meta[otap_resource_id as usize];
        let scope_meta = &self.scope_meta[otap_scope_id as usize];

        // Write metric row
        self.metrics_builder.append_id(id);
        self.metrics_builder
            .resource
            .append_id_n(otap_resource_id, 1);
        self.metrics_builder
            .resource
            .append_schema_url_n(Some(&resource_meta.schema_url), 1);
        self.metrics_builder
            .resource
            .append_dropped_attributes_count_n(resource_meta.dropped_attributes_count, 1);
        self.metrics_builder.scope.append_id_n(otap_scope_id, 1);
        self.metrics_builder
            .scope
            .append_name_n(Some(&scope_meta.name), 1);
        self.metrics_builder
            .scope
            .append_version_n(Some(&scope_meta.version), 1);
        self.metrics_builder
            .scope
            .append_dropped_attributes_count_n(scope_meta.dropped_attributes_count, 1);
        self.metrics_builder
            .append_scope_schema_url_n(scope_schema_url, 1);
        self.metrics_builder.append_metric_type(metric_id.data_type);
        self.metrics_builder.append_name(view.name());
        self.metrics_builder.append_description(view.description());
        self.metrics_builder.append_unit(view.unit());

        let agg_temp =
            if metric_id.aggregation_temporality == AggregationTemporality::Unspecified as u8 {
                None
            } else {
                Some(metric_id.aggregation_temporality as i32)
            };
        self.metrics_builder
            .append_aggregation_temporality(agg_temp);
        self.metrics_builder
            .append_is_monotonic(Some(metric_id.is_monotonic));

        _ = self.metric_ids.insert(metric_id.into_owned(), id);
        id
    }

    // -----------------------------------------------------------------------
    // Data point ingestion
    // -----------------------------------------------------------------------

    fn ingest_number_dp<V: NumberDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
        dp_type: DpType,
    ) {
        let time = dp.time_unix_nano();
        if let Some(state) = self.stream_map.get_mut(&StreamIdRef(&stream_id)) {
            if time > state.time_unix_nano {
                self.number_dps.overwrite(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_ndp_id;
            self.next_ndp_id += 1;
            let row_index = self.number_dps.push(dp_id, otap_metric_id, dp);

            // Write dp attributes (once per stream)
            for attr in dp.attributes() {
                self.ndp_attrs_builder.append_parent_id(&dp_id);
                let _ = append_attribute_value(&mut self.ndp_attrs_builder, &attr);
            }

            _ = self.stream_map.insert(
                stream_id.into_owned(),
                StreamState {
                    dp_row_index: row_index,
                    dp_type,
                    time_unix_nano: time,
                },
            );
        }
    }

    fn ingest_histogram_dp<V: HistogramDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) {
        let time = dp.time_unix_nano();
        if let Some(state) = self.stream_map.get_mut(&StreamIdRef(&stream_id)) {
            if time > state.time_unix_nano {
                self.histogram_dps.overwrite(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_hdp_id;
            self.next_hdp_id += 1;
            let row_index = self.histogram_dps.push(dp_id, otap_metric_id, dp);

            for attr in dp.attributes() {
                self.hdp_attrs_builder.append_parent_id(&dp_id);
                let _ = append_attribute_value(&mut self.hdp_attrs_builder, &attr);
            }

            _ = self.stream_map.insert(
                stream_id.into_owned(),
                StreamState {
                    dp_row_index: row_index,
                    dp_type: DpType::Histogram,
                    time_unix_nano: time,
                },
            );
        }
    }

    fn ingest_exp_histogram_dp<V: ExponentialHistogramDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) {
        let time = dp.time_unix_nano();
        if let Some(state) = self.stream_map.get_mut(&StreamIdRef(&stream_id)) {
            if time > state.time_unix_nano {
                self.exp_histogram_dps.overwrite(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_ehdp_id;
            self.next_ehdp_id += 1;
            let row_index = self.exp_histogram_dps.push(dp_id, otap_metric_id, dp);

            for attr in dp.attributes() {
                self.ehdp_attrs_builder.append_parent_id(&dp_id);
                let _ = append_attribute_value(&mut self.ehdp_attrs_builder, &attr);
            }

            _ = self.stream_map.insert(
                stream_id.into_owned(),
                StreamState {
                    dp_row_index: row_index,
                    dp_type: DpType::ExponentialHistogram,
                    time_unix_nano: time,
                },
            );
        }
    }

    fn ingest_summary_dp<V: SummaryDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) {
        let time = dp.time_unix_nano();
        if let Some(state) = self.stream_map.get_mut(&StreamIdRef(&stream_id)) {
            if time > state.time_unix_nano {
                self.summary_dps.overwrite(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_sdp_id;
            self.next_sdp_id += 1;
            let row_index = self.summary_dps.push(dp_id, otap_metric_id, dp);

            for attr in dp.attributes() {
                self.summary_attrs_builder.append_parent_id(&dp_id);
                let _ = append_attribute_value(&mut self.summary_attrs_builder, &attr);
            }

            _ = self.stream_map.insert(
                stream_id.into_owned(),
                StreamState {
                    dp_row_index: row_index,
                    dp_type: DpType::Summary,
                    time_unix_nano: time,
                },
            );
        }
    }

    // -----------------------------------------------------------------------
    // Reset
    // -----------------------------------------------------------------------

    fn clear(&mut self) {
        self.resource_ids.clear();
        self.scope_ids.clear();
        self.metric_ids.clear();
        self.stream_map.clear();

        self.next_resource_id = 0;
        self.next_scope_id = 0;
        self.next_metric_id = 0;
        self.next_ndp_id = 0;
        self.next_hdp_id = 0;
        self.next_ehdp_id = 0;
        self.next_sdp_id = 0;

        self.resource_meta.clear();
        self.scope_meta.clear();

        // Note: pdata builders don't have clear() -- they're consumed by finish().
        // After flush(), finish() has already been called, so they're in a fresh state.
        // We reinitialize them to be safe.
        self.metrics_builder = MetricsRecordBatchBuilder::new();
        self.resource_attrs_builder = AttributesRecordBatchBuilder::new();
        self.scope_attrs_builder = AttributesRecordBatchBuilder::new();
        self.ndp_attrs_builder = AttributesRecordBatchBuilder::new();
        self.hdp_attrs_builder = AttributesRecordBatchBuilder::new();
        self.ehdp_attrs_builder = AttributesRecordBatchBuilder::new();
        self.summary_attrs_builder = AttributesRecordBatchBuilder::new();

        self.number_dps.clear();
        self.histogram_dps.clear();
        self.exp_histogram_dps.clear();
        self.summary_dps.clear();
    }
}
