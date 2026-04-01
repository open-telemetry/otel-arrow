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

use super::data_point_builder::{
    ExpHistogramDataPointBuilder, HistogramDataPointBuilder, NumberDataPointBuilder,
    SummaryDataPointBuilder,
};
use super::identity::{
    AttributeHash, AttributeHashBuffer, MetricId, MetricIdRef, ResourceId, ScopeId, ScopeIdRef,
    StreamId, StreamIdRef, metric_id_of, resource_id_of, scope_id_of, stream_id_of,
};

/// Accumulates metrics data over a collection interval.
pub struct MetricAggregator {
    // Reusable buffer for computing hashes.
    // TODO: Add another one of these for otlp views
    hash_buf: AttributeHashBuffer<OtapAttributeView<'static>>,

    // These track the id in the otap batch assigned to each unique resource,
    // scope, or metric.
    resource_ids: HashMap<ResourceId, u16>,
    next_resource_id: u16,
    scope_ids: HashMap<ScopeId<'static>, u16>,
    next_scope_id: u16,
    metric_ids: HashMap<MetricId<'static>, u16>,
    next_metric_id: u16,

    // Similar to the above, but for the various data point ids
    stream_map: HashMap<StreamId<'static>, StreamState>,
    next_ndp_id: u32,
    next_hdp_id: u32,
    next_ehdp_id: u32,
    next_sdp_id: u32,

    // This is additional data that corresponds to each resource/scope that
    // needs to be set for every single row such as schema_url.
    resource_meta: Vec<ResourceMeta>,
    scope_meta: Vec<ScopeMeta>,

    // Record batch builders for each of the payload types.
    //
    // TODO: Exemplars are not currently supported and just dropped by this
    // processor.
    metrics_builder: MetricsRecordBatchBuilder,
    resource_attrs_builder: AttributesRecordBatchBuilder<u16>,
    scope_attrs_builder: AttributesRecordBatchBuilder<u16>,
    ndp_attrs_builder: AttributesRecordBatchBuilder<u32>,
    hdp_attrs_builder: AttributesRecordBatchBuilder<u32>,
    ehdp_attrs_builder: AttributesRecordBatchBuilder<u32>,
    summary_attrs_builder: AttributesRecordBatchBuilder<u32>,

    // These are custom data point specific builder types that support random
    // writes so that we can replace old data points with newer ones.
    number_dps: NumberDataPointBuilder,
    histogram_dps: HistogramDataPointBuilder,
    exp_histogram_dps: ExpHistogramDataPointBuilder,
    summary_dps: SummaryDataPointBuilder,
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

            number_dps: NumberDataPointBuilder::new(),
            histogram_dps: HistogramDataPointBuilder::new(),
            exp_histogram_dps: ExpHistogramDataPointBuilder::new(),
            summary_dps: SummaryDataPointBuilder::new(),
        }
    }

    /// Process one incoming OTAP metrics view.
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
                                    self.ingest_number_dp(&dp, stream_id, otap_metric_id);
                                }
                            }
                        }
                        DataType::Sum => {
                            if let Some(sum) = data.as_sum() {
                                for dp in sum.data_points() {
                                    let attrs = self.compute_otap_attr_hash(dp.attributes());
                                    let stream_id = stream_id_of(metric_id.clone(), attrs);
                                    self.ingest_number_dp(&dp, stream_id, otap_metric_id);
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

    /// Flush accumulated state into an [`OtapArrowRecords`] batch and reset.
    pub fn finish(&mut self) -> otap_df_pdata::error::Result<Option<OtapArrowRecords>> {
        if self.is_empty() {
            return Ok(None);
        }

        let mut records = OtapArrowRecords::Metrics(Metrics::default());

        finish_payload(
            self.metrics_builder.finish(),
            ArrowPayloadType::UnivariateMetrics,
            &mut records,
        )?;
        finish_payload(
            self.resource_attrs_builder.finish(),
            ArrowPayloadType::ResourceAttrs,
            &mut records,
        )?;
        finish_payload(
            self.scope_attrs_builder.finish(),
            ArrowPayloadType::ScopeAttrs,
            &mut records,
        )?;
        finish_payload(
            self.number_dps.finish(),
            ArrowPayloadType::NumberDataPoints,
            &mut records,
        )?;
        finish_payload(
            self.ndp_attrs_builder.finish(),
            ArrowPayloadType::NumberDpAttrs,
            &mut records,
        )?;
        finish_payload(
            self.histogram_dps.finish(),
            ArrowPayloadType::HistogramDataPoints,
            &mut records,
        )?;
        finish_payload(
            self.hdp_attrs_builder.finish(),
            ArrowPayloadType::HistogramDpAttrs,
            &mut records,
        )?;
        finish_payload(
            self.exp_histogram_dps.finish(),
            ArrowPayloadType::ExpHistogramDataPoints,
            &mut records,
        )?;
        finish_payload(
            self.ehdp_attrs_builder.finish(),
            ArrowPayloadType::ExpHistogramDpAttrs,
            &mut records,
        )?;
        finish_payload(
            self.summary_dps.finish(),
            ArrowPayloadType::SummaryDataPoints,
            &mut records,
        )?;
        finish_payload(
            self.summary_attrs_builder.finish(),
            ArrowPayloadType::SummaryDpAttrs,
            &mut records,
        )?;

        self.clear();
        Ok(Some(records))
    }

    pub fn is_empty(&self) -> bool {
        self.stream_map.is_empty()
    }

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
        // TODO: Cap the number of IDs somehow
        debug_assert!(id < u16::MAX, "resource ID overflow");
        self.next_resource_id += 1;

        for attr in view.attributes() {
            self.resource_attrs_builder.append_parent_id(&id);
            let _ = append_attribute_value(&mut self.resource_attrs_builder, &attr);
        }

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
        // TODO: Cap the number of IDs
        debug_assert!(id < u16::MAX, "scope ID overflow");
        self.next_scope_id += 1;

        for attr in view.attributes() {
            self.scope_attrs_builder.append_parent_id(&id);
            let _ = append_attribute_value(&mut self.scope_attrs_builder, &attr);
        }

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
        // TODO: Cap the number of Ids
        debug_assert!(id < u16::MAX, "metric ID overflow");
        self.next_metric_id += 1;

        let resource_meta = &self.resource_meta[otap_resource_id as usize];
        let scope_meta = &self.scope_meta[otap_scope_id as usize];

        self.metrics_builder.append_id(id);
        self.metrics_builder
            .resource
            .append_id(Some(otap_resource_id));
        self.metrics_builder
            .resource
            .append_schema_url(Some(&resource_meta.schema_url));
        self.metrics_builder
            .resource
            .append_dropped_attributes_count(resource_meta.dropped_attributes_count);
        self.metrics_builder.scope.append_id(Some(otap_scope_id));
        self.metrics_builder
            .scope
            .append_name(Some(&scope_meta.name));
        self.metrics_builder
            .scope
            .append_version(Some(&scope_meta.version));
        self.metrics_builder
            .scope
            .append_dropped_attributes_count(scope_meta.dropped_attributes_count);
        self.metrics_builder
            .append_scope_schema_url(scope_schema_url);
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

    fn ingest_number_dp<V: NumberDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) {
        let time = dp.time_unix_nano();
        if let Some(state) = self.stream_map.get_mut(&StreamIdRef(&stream_id)) {
            if time > state.time_unix_nano {
                self.number_dps.replace(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_ndp_id;
            self.next_ndp_id += 1;
            let row_index = self.number_dps.append(dp_id, otap_metric_id, dp);

            for attr in dp.attributes() {
                self.ndp_attrs_builder.append_parent_id(&dp_id);
                let _ = append_attribute_value(&mut self.ndp_attrs_builder, &attr);
            }

            _ = self.stream_map.insert(
                stream_id.into_owned(),
                StreamState {
                    dp_row_index: row_index,
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
                self.histogram_dps.replace(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_hdp_id;
            self.next_hdp_id += 1;
            let row_index = self.histogram_dps.append(dp_id, otap_metric_id, dp);

            for attr in dp.attributes() {
                self.hdp_attrs_builder.append_parent_id(&dp_id);
                let _ = append_attribute_value(&mut self.hdp_attrs_builder, &attr);
            }

            _ = self.stream_map.insert(
                stream_id.into_owned(),
                StreamState {
                    dp_row_index: row_index,
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
                self.exp_histogram_dps.replace(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_ehdp_id;
            self.next_ehdp_id += 1;
            let row_index = self.exp_histogram_dps.append(dp_id, otap_metric_id, dp);

            for attr in dp.attributes() {
                self.ehdp_attrs_builder.append_parent_id(&dp_id);
                let _ = append_attribute_value(&mut self.ehdp_attrs_builder, &attr);
            }

            _ = self.stream_map.insert(
                stream_id.into_owned(),
                StreamState {
                    dp_row_index: row_index,
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
                self.summary_dps.replace(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_sdp_id;
            self.next_sdp_id += 1;
            let row_index = self.summary_dps.append(dp_id, otap_metric_id, dp);

            for attr in dp.attributes() {
                self.summary_attrs_builder.append_parent_id(&dp_id);
                let _ = append_attribute_value(&mut self.summary_attrs_builder, &attr);
            }

            _ = self.stream_map.insert(
                stream_id.into_owned(),
                StreamState {
                    dp_row_index: row_index,
                    time_unix_nano: time,
                },
            );
        }
    }

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

struct ResourceMeta {
    schema_url: Vec<u8>,
    dropped_attributes_count: u32,
}

struct ScopeMeta {
    name: Vec<u8>,
    version: Vec<u8>,
    dropped_attributes_count: u32,
}

struct StreamState {
    dp_row_index: usize,
    time_unix_nano: u64,
}

/// Returns `true` if this metric type should be aggregated (buffered and
/// flushed on timer), `false` if it should be passed through unchanged.
fn is_aggregatable(metric_id: &MetricId<'_>) -> bool {
    let Some(data_type) = DataType::from_u8(metric_id.data_type) else {
        return false;
    };

    let cumulative = metric_id.aggregation_temporality == AggregationTemporality::Cumulative as u8;
    let monotonic = metric_id.is_monotonic;

    match data_type {
        DataType::Gauge | DataType::Summary => true,
        DataType::Sum if cumulative && monotonic => true,
        DataType::Histogram | DataType::ExponentialHistogram if cumulative => true,
        _ => false,
    }
}

/// Finish building a payload type and set it on the output records.
fn finish_payload(
    result: Result<arrow::array::RecordBatch, arrow::error::ArrowError>,
    payload_type: ArrowPayloadType,
    records: &mut OtapArrowRecords,
) -> otap_df_pdata::error::Result<()> {
    // Safety: So long as the aggregation logic is keeping arrays
    // the same length, this operation should be infallible.
    let rb = result.expect("Valid record batch");
    if rb.num_rows() > 0 {
        records.set(payload_type, rb)?;
    }

    Ok(())
}
