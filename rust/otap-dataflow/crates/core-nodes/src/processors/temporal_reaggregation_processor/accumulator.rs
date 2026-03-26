// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics batch accumulator for temporal reaggregation.
//!
//! This module provides [`MetricsBatchAccumulator`], which buffers incoming
//! metrics data, deduplicates resources/scopes/metrics by identity, and
//! tracks the latest data point per stream. On flush it produces an
//! [`OtapArrowRecords`] batch from the accumulated state.

use std::borrow::Cow;
use std::collections::HashMap;

use otap_df_pdata::otap::OtapArrowRecords;
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
    AttributeHashBuffer, MetricId, ResourceId, ScopeId, StreamId, metric_id_of, resource_id_of,
    scope_id_of, stream_id_of,
};

// ---------------------------------------------------------------------------
// Aggregatability check
// ---------------------------------------------------------------------------

/// Returns `true` if this metric type should be aggregated (buffered and
/// flushed on timer), `false` if it should be passed through unchanged.
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
// MetricsBatchAccumulator
// ---------------------------------------------------------------------------

/// Accumulates metrics data over a collection interval.
pub struct MetricAggregator {
    hash_buf: AttributeHashBuffer<OtapAttributeView<'static>>,

    resource_ids: HashMap<ResourceId, u16>,
    next_resource_id: u16,
    scope_ids: HashMap<ScopeId<'static>, u16>,
    next_scope_id: u16,
    metric_ids: HashMap<MetricId<'static>, u16>,
    next_metric_id: u16,

    next_ndp_id: u32,
    stream_map: HashMap<StreamId<'static>, StreamState>,
    next_hdp_id: u32,
    next_ehdp_id: u32,
    next_sdp_id: u32,

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

            number_dps: NumberDataPointColumns::new(),
            histogram_dps: HistogramDataPointColumns::new(),
            exp_histogram_dps: ExpHistogramDataPointColumns::new(),
            summary_dps: SummaryDataPointColumns::new(),
        }
    }

    /// Process one incoming OTAP metrics view.
    pub fn ingest(&mut self, view: OtapMetricsView<'_>) {
        for resource_metrics in view.resources() {
            let Some(resource) = resource_metrics.resource() else {
                continue;
            };

            let hash_buf = std::mem::replace(&mut self.hash_buf, AttributeHashBuffer::new());
            let mut hash_buf: AttributeHashBuffer<OtapAttributeView<'_>> = hash_buf.recycle();
            let resource_id = resource_id_of(&resource, &mut hash_buf);
            self.hash_buf = hash_buf.recycle();

            let otap_resource_id = self.process_resource(resource_id);
        }
    }

    pub fn flush(&mut self) -> Option<OtapArrowRecords> {
        if self.is_empty() {
            return None;
        }
        self.clear();
        todo!("Stage 3b: assemble OtapArrowRecords from accumulated state")
    }

    pub fn stream_count(&self) -> usize {
        self.stream_map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stream_map.is_empty()
    }

    // -----------------------------------------------------------------------
    // Identity assignment
    // -----------------------------------------------------------------------

    fn process_resource<T: ResourceView>(&mut self, resource_id: ResourceId, view: T) -> u16 {
        if let Some(&id) = self.resource_ids.get(&resource_id) {
            return id;
        } else {
            todo!("Write the resource to our builder")
        }

        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.resource_ids.insert(resource_id, id);
        id
    }

    fn process_scope(&mut self, scope_id: ScopeId<'_>) -> u16 {
        if let Some(&id) = self.scope_ids.get(&scope_id) {
            return id;
        }
        let id = self.next_scope_id;
        self.next_scope_id += 1;
        self.scope_ids.insert(scope_id.into_owned(), id);
        id
    }

    fn process_metric(&mut self, metric_id: MetricId<'_>) -> u16 {
        if let Some(&id) = self.metric_ids.get(&metric_id) {
            return id;
        }
        let id = self.next_metric_id;
        self.next_metric_id += 1;
        self.metric_ids.insert(metric_id.into_owned(), id);
        id
    }

    // -----------------------------------------------------------------------
    // Data point ingestion (pre-computed stream ID)
    // -----------------------------------------------------------------------

    fn ingest_number_dp_with_id<V: NumberDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
        dp_type: DpType,
    ) {
        let time = dp.time_unix_nano();
        let stream_id = stream_id.into_owned();
        if let Some(state) = self.stream_map.get_mut(&stream_id) {
            if time > state.time_unix_nano {
                self.number_dps.overwrite(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_ndp_id;
            self.next_ndp_id += 1;
            let row_index = self.number_dps.push(dp_id, otap_metric_id, dp);
            self.stream_map.insert(
                stream_id,
                StreamState {
                    dp_row_index: row_index,
                    dp_type,
                    time_unix_nano: time,
                },
            );
        }
    }

    fn ingest_histogram_dp_with_id<V: HistogramDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) {
        let time = dp.time_unix_nano();
        let stream_id = stream_id.into_owned();
        if let Some(state) = self.stream_map.get_mut(&stream_id) {
            if time > state.time_unix_nano {
                self.histogram_dps.overwrite(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_hdp_id;
            self.next_hdp_id += 1;
            let row_index = self.histogram_dps.push(dp_id, otap_metric_id, dp);
            self.stream_map.insert(
                stream_id,
                StreamState {
                    dp_row_index: row_index,
                    dp_type: DpType::Histogram,
                    time_unix_nano: time,
                },
            );
        }
    }

    fn ingest_exp_histogram_dp_with_id<V: ExponentialHistogramDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) {
        let time = dp.time_unix_nano();
        let stream_id = stream_id.into_owned();
        if let Some(state) = self.stream_map.get_mut(&stream_id) {
            if time > state.time_unix_nano {
                self.exp_histogram_dps.overwrite(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_ehdp_id;
            self.next_ehdp_id += 1;
            let row_index = self.exp_histogram_dps.push(dp_id, otap_metric_id, dp);
            self.stream_map.insert(
                stream_id,
                StreamState {
                    dp_row_index: row_index,
                    dp_type: DpType::ExponentialHistogram,
                    time_unix_nano: time,
                },
            );
        }
    }

    fn ingest_summary_dp_with_id<V: SummaryDataPointView>(
        &mut self,
        dp: &V,
        stream_id: StreamId<'_>,
        otap_metric_id: u16,
    ) {
        let time = dp.time_unix_nano();
        let stream_id = stream_id.into_owned();
        if let Some(state) = self.stream_map.get_mut(&stream_id) {
            if time > state.time_unix_nano {
                self.summary_dps.overwrite(state.dp_row_index, dp);
                state.time_unix_nano = time;
            }
        } else {
            let dp_id = self.next_sdp_id;
            self.next_sdp_id += 1;
            let row_index = self.summary_dps.push(dp_id, otap_metric_id, dp);
            self.stream_map.insert(
                stream_id,
                StreamState {
                    dp_row_index: row_index,
                    dp_type: DpType::Summary,
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

        self.number_dps.clear();
        self.histogram_dps.clear();
        self.exp_histogram_dps.clear();
        self.summary_dps.clear();
    }
}
