// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Record batch builders for temporal reaggregation output.
//!
//! This module contains [`MetricSignalBuilder`] which orchestrates all the
//! individual payload type builders, plus the specialized data point builders
//! that support random-access writes for replacing stale data points.
//!
//! The data point builders exist because APIs to write at a specific Array
//! position are typically not supported by arrow-rs ArrayBuilder types. Random
//! write APIs _are_, however, generally supported by the arrow-rs BufferBuilder
//! types, so these builders use `Vec<T>` as much as possible along with
//! [`BooleanBufferBuilder`] directly for nulls.
//!
//! Note: These builders currently make no attempt to optimize for space via
//! dictionary encoding. We may want to add this in the future, however a typical
//! pipeline configuration may be to place a batch processor immediately following
//! this component in which case the dictionary could get re-written or removed
//! anyway. Figuring out under what circumstances it's better to use dictionaries
//! here needs investigation.

use std::sync::{Arc, LazyLock};

use arrow::array::{
    Array, ArrowPrimitiveType, BooleanBufferBuilder, Float64Array, Int32Array, ListArray,
    PrimitiveArray, RecordBatch, StructArray, TimestampNanosecondArray, UInt16Array, UInt32Array,
    UInt64Array,
};
use arrow::buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow::datatypes::{DataType, Field, Fields, Float64Type, Int64Type, Schema, TimeUnit};
use otap_df_pdata::encode::append_attribute_value;
use otap_df_pdata::encode::record::attributes::AttributesRecordBatchBuilder;
use otap_df_pdata::encode::record::metrics::{
    ExemplarsRecordBatchBuilder, MetricsRecordBatchBuilder,
};
use otap_df_pdata::otap::{Metrics, OtapArrowRecords};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::{FieldExt, consts};
use otap_df_pdata_views::views::common::InstrumentationScopeView;
use otap_df_pdata_views::views::metrics::{
    AggregationTemporality, BucketsView, ExemplarView, ExponentialHistogramDataPointView,
    HistogramDataPointView, MetricView, NumberDataPointView, SummaryDataPointView, Value,
    ValueAtQuantileView,
};
use otap_df_pdata_views::views::resource::ResourceView;

/// Snapshot of builder positions, used to slice back to the last valid position
/// of each payload on error.
#[derive(Debug)]
pub struct Checkpoint {
    metrics: usize,
    resource_attrs: usize,
    scope_attrs: usize,
    ndp: usize,
    ndp_attrs: usize,
    ndp_exemplars: usize,
    ndp_exemplar_attrs: usize,
    hdp: usize,
    hdp_attrs: usize,
    hdp_exemplars: usize,
    hdp_exemplar_attrs: usize,
    ehdp: usize,
    ehdp_attrs: usize,
    ehdp_exemplars: usize,
    ehdp_exemplar_attrs: usize,
    sdp: usize,
    sdp_attrs: usize,
}

/// Record batch builders for all metric signal payload types.
pub struct MetricSignalBuilder {
    metrics: MetricsRecordBatchBuilder,
    resource_attrs: AttributesRecordBatchBuilder<u16>,
    scope_attrs: AttributesRecordBatchBuilder<u16>,
    ndp_attrs: AttributesRecordBatchBuilder<u32>,
    hdp_attrs: AttributesRecordBatchBuilder<u32>,
    ehdp_attrs: AttributesRecordBatchBuilder<u32>,
    summary_attrs: AttributesRecordBatchBuilder<u32>,
    number_dps: NumberDataPointBuilder,
    histogram_dps: HistogramDataPointBuilder,
    exp_histogram_dps: ExpHistogramDataPointBuilder,
    summary_dps: SummaryDataPointBuilder,
    ndp_exemplars: ExemplarsRecordBatchBuilder,
    ndp_exemplar_attrs: AttributesRecordBatchBuilder<u32>,
    hdp_exemplars: ExemplarsRecordBatchBuilder,
    hdp_exemplar_attrs: AttributesRecordBatchBuilder<u32>,
    ehdp_exemplars: ExemplarsRecordBatchBuilder,
    ehdp_exemplar_attrs: AttributesRecordBatchBuilder<u32>,
}

impl MetricSignalBuilder {
    pub fn new() -> Self {
        Self {
            metrics: MetricsRecordBatchBuilder::new(),
            resource_attrs: AttributesRecordBatchBuilder::new(),
            scope_attrs: AttributesRecordBatchBuilder::new(),
            ndp_attrs: AttributesRecordBatchBuilder::new(),
            hdp_attrs: AttributesRecordBatchBuilder::new(),
            ehdp_attrs: AttributesRecordBatchBuilder::new(),
            summary_attrs: AttributesRecordBatchBuilder::new(),
            number_dps: NumberDataPointBuilder::new(),
            histogram_dps: HistogramDataPointBuilder::new(),
            exp_histogram_dps: ExpHistogramDataPointBuilder::new(),
            summary_dps: SummaryDataPointBuilder::new(),
            ndp_exemplars: ExemplarsRecordBatchBuilder::new(),
            ndp_exemplar_attrs: AttributesRecordBatchBuilder::new(),
            hdp_exemplars: ExemplarsRecordBatchBuilder::new(),
            hdp_exemplar_attrs: AttributesRecordBatchBuilder::new(),
            ehdp_exemplars: ExemplarsRecordBatchBuilder::new(),
            ehdp_exemplar_attrs: AttributesRecordBatchBuilder::new(),
        }
    }

    /// Finish all builders and assemble an [`OtapArrowRecords`] batch.
    ///
    /// When `checkpoint` is `Some`, each output record batch is sliced to the
    /// lengths captured in the checkpoint, discarding rows appended afterward.
    /// This is used by [super::TemporalReaggregationProcessor::flush_at] to
    /// discard some changes if we couldn't fully append an incoming pdata.
    pub fn finish(
        &mut self,
        checkpoint: Option<Checkpoint>,
    ) -> otap_df_pdata::error::Result<OtapArrowRecords> {
        let mut records = OtapArrowRecords::Metrics(Metrics::default());
        let checkpoint = checkpoint.as_ref();

        finish_payload(
            self.metrics.finish(),
            ArrowPayloadType::UnivariateMetrics,
            &mut records,
            checkpoint.map(|c| c.metrics),
        )?;
        finish_payload(
            self.resource_attrs.finish(),
            ArrowPayloadType::ResourceAttrs,
            &mut records,
            checkpoint.map(|c| c.resource_attrs),
        )?;
        finish_payload(
            self.scope_attrs.finish(),
            ArrowPayloadType::ScopeAttrs,
            &mut records,
            checkpoint.map(|c| c.scope_attrs),
        )?;
        finish_payload(
            self.number_dps.finish(),
            ArrowPayloadType::NumberDataPoints,
            &mut records,
            checkpoint.map(|c| c.ndp),
        )?;
        finish_payload(
            self.ndp_attrs.finish(),
            ArrowPayloadType::NumberDpAttrs,
            &mut records,
            checkpoint.map(|c| c.ndp_attrs),
        )?;
        finish_exemplar_payload(
            self.ndp_exemplars.finish(),
            ArrowPayloadType::NumberDpExemplars,
            &mut records,
            checkpoint.map(|c| c.ndp_exemplars),
        )?;
        finish_payload(
            self.ndp_exemplar_attrs.finish(),
            ArrowPayloadType::NumberDpExemplarAttrs,
            &mut records,
            checkpoint.map(|c| c.ndp_exemplar_attrs),
        )?;
        finish_payload(
            self.histogram_dps.finish(),
            ArrowPayloadType::HistogramDataPoints,
            &mut records,
            checkpoint.map(|c| c.hdp),
        )?;
        finish_payload(
            self.hdp_attrs.finish(),
            ArrowPayloadType::HistogramDpAttrs,
            &mut records,
            checkpoint.map(|c| c.hdp_attrs),
        )?;
        finish_exemplar_payload(
            self.hdp_exemplars.finish(),
            ArrowPayloadType::HistogramDpExemplars,
            &mut records,
            checkpoint.map(|c| c.hdp_exemplars),
        )?;
        finish_payload(
            self.hdp_exemplar_attrs.finish(),
            ArrowPayloadType::HistogramDpExemplarAttrs,
            &mut records,
            checkpoint.map(|c| c.hdp_exemplar_attrs),
        )?;
        finish_payload(
            self.exp_histogram_dps.finish(),
            ArrowPayloadType::ExpHistogramDataPoints,
            &mut records,
            checkpoint.map(|c| c.ehdp),
        )?;
        finish_payload(
            self.ehdp_attrs.finish(),
            ArrowPayloadType::ExpHistogramDpAttrs,
            &mut records,
            checkpoint.map(|c| c.ehdp_attrs),
        )?;
        finish_exemplar_payload(
            self.ehdp_exemplars.finish(),
            ArrowPayloadType::ExpHistogramDpExemplars,
            &mut records,
            checkpoint.map(|c| c.ehdp_exemplars),
        )?;
        finish_payload(
            self.ehdp_exemplar_attrs.finish(),
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
            &mut records,
            checkpoint.map(|c| c.ehdp_exemplar_attrs),
        )?;
        finish_payload(
            self.summary_dps.finish(),
            ArrowPayloadType::SummaryDataPoints,
            &mut records,
            checkpoint.map(|c| c.sdp),
        )?;
        finish_payload(
            self.summary_attrs.finish(),
            ArrowPayloadType::SummaryDpAttrs,
            &mut records,
            checkpoint.map(|c| c.sdp_attrs),
        )?;

        Ok(records)
    }

    /// Capture the current builder positions as a [`Checkpoint`].
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            metrics: self.metrics.len(),
            resource_attrs: self.resource_attrs.len(),
            scope_attrs: self.scope_attrs.len(),
            ndp: self.number_dps.id.len(),
            ndp_attrs: self.ndp_attrs.len(),
            ndp_exemplars: self.ndp_exemplars.len(),
            ndp_exemplar_attrs: self.ndp_exemplar_attrs.len(),
            hdp: self.histogram_dps.id.len(),
            hdp_attrs: self.hdp_attrs.len(),
            hdp_exemplars: self.hdp_exemplars.len(),
            hdp_exemplar_attrs: self.hdp_exemplar_attrs.len(),
            ehdp: self.exp_histogram_dps.id.len(),
            ehdp_attrs: self.ehdp_attrs.len(),
            ehdp_exemplars: self.ehdp_exemplars.len(),
            ehdp_exemplar_attrs: self.ehdp_exemplar_attrs.len(),
            sdp: self.summary_dps.id.len(),
            sdp_attrs: self.summary_attrs.len(),
        }
    }

    pub fn clear(&mut self) {
        self.metrics = MetricsRecordBatchBuilder::new();
        self.resource_attrs = AttributesRecordBatchBuilder::new();
        self.scope_attrs = AttributesRecordBatchBuilder::new();
        self.ndp_attrs = AttributesRecordBatchBuilder::new();
        self.hdp_attrs = AttributesRecordBatchBuilder::new();
        self.ehdp_attrs = AttributesRecordBatchBuilder::new();
        self.summary_attrs = AttributesRecordBatchBuilder::new();
        self.number_dps.clear();
        self.histogram_dps.clear();
        self.exp_histogram_dps.clear();
        self.summary_dps.clear();
        self.ndp_exemplars = ExemplarsRecordBatchBuilder::new();
        self.ndp_exemplar_attrs = AttributesRecordBatchBuilder::new();
        self.hdp_exemplars = ExemplarsRecordBatchBuilder::new();
        self.hdp_exemplar_attrs = AttributesRecordBatchBuilder::new();
        self.ehdp_exemplars = ExemplarsRecordBatchBuilder::new();
        self.ehdp_exemplar_attrs = AttributesRecordBatchBuilder::new();
    }

    /// Append resource attributes for a newly seen resource.
    pub fn append_resource<R: ResourceView>(&mut self, id: u16, view: &R) {
        for attr in view.attributes() {
            self.resource_attrs.append_parent_id(&id);
            let _ = append_attribute_value(&mut self.resource_attrs, &attr);
        }
    }

    /// Append scope attributes for a newly seen scope.
    pub fn append_scope<S: InstrumentationScopeView>(&mut self, id: u16, view: &S) {
        for attr in view.attributes() {
            self.scope_attrs.append_parent_id(&id);
            let _ = append_attribute_value(&mut self.scope_attrs, &attr);
        }
    }

    /// Append a complete metric row for a newly seen metric.
    pub fn append_metric<M: MetricView, R: ResourceView, S: InstrumentationScopeView>(
        &mut self,
        id: u16,
        view: &M,
        data_type: u8,
        aggregation_temporality: u8,
        is_monotonic: bool,
        resource_otap_id: u16,
        resource_schema_url: &[u8],
        resource_view: Option<&R>,
        scope_otap_id: u16,
        scope_view: Option<&S>,
        scope_schema_url: &[u8],
    ) {
        self.metrics.append_id(id);
        self.metrics.resource.append_id(Some(resource_otap_id));
        self.metrics
            .resource
            .append_schema_url(Some(resource_schema_url));
        self.metrics.resource.append_dropped_attributes_count(
            resource_view.map_or(0, |r| r.dropped_attributes_count()),
        );
        self.metrics.scope.append_id(Some(scope_otap_id));
        self.metrics
            .scope
            .append_name(scope_view.and_then(|s| s.name()));
        self.metrics
            .scope
            .append_version(scope_view.and_then(|s| s.version()));
        self.metrics.scope.append_dropped_attributes_count(
            scope_view.map_or(0, |s| s.dropped_attributes_count()),
        );
        self.metrics.append_scope_schema_url(scope_schema_url);
        self.metrics.append_metric_type(data_type);
        self.metrics.append_name(view.name());
        self.metrics.append_description(view.description());
        self.metrics.append_unit(view.unit());

        let agg_temp = if aggregation_temporality == AggregationTemporality::Unspecified as u8 {
            None
        } else {
            Some(aggregation_temporality as i32)
        };
        self.metrics.append_aggregation_temporality(agg_temp);
        self.metrics.append_is_monotonic(Some(is_monotonic));
    }

    /// Append a new number data point row including its attributes.
    pub fn append_number_dp<V: NumberDataPointView>(
        &mut self,
        dp_id: u32,
        metric_id: u16,
        dp: &V,
    ) -> usize {
        let row = self.number_dps.append(dp_id, metric_id, dp);
        for attr in dp.attributes() {
            self.ndp_attrs.append_parent_id(&dp_id);
            let _ = append_attribute_value(&mut self.ndp_attrs, &attr);
        }
        row
    }

    /// Replace an existing number data point row with newer data.
    ///
    /// Attributes are not updated — they are part of stream identity and
    /// never change for the same stream.
    pub fn replace_number_dp<V: NumberDataPointView>(&mut self, row: usize, dp: &V) {
        self.number_dps.replace(row, dp);
    }

    /// Append a new histogram data point row including its attributes.
    pub fn append_histogram_dp<V: HistogramDataPointView>(
        &mut self,
        dp_id: u32,
        metric_id: u16,
        dp: &V,
    ) -> usize {
        let row = self.histogram_dps.append(dp_id, metric_id, dp);
        for attr in dp.attributes() {
            self.hdp_attrs.append_parent_id(&dp_id);
            let _ = append_attribute_value(&mut self.hdp_attrs, &attr);
        }
        row
    }

    /// Replace an existing histogram data point row with newer data.
    pub fn replace_histogram_dp<V: HistogramDataPointView>(&mut self, row: usize, dp: &V) {
        self.histogram_dps.replace(row, dp);
    }

    /// Append a new exponential histogram data point row including its attributes.
    pub fn append_exp_histogram_dp<V: ExponentialHistogramDataPointView>(
        &mut self,
        dp_id: u32,
        metric_id: u16,
        dp: &V,
    ) -> usize {
        let row = self.exp_histogram_dps.append(dp_id, metric_id, dp);
        for attr in dp.attributes() {
            self.ehdp_attrs.append_parent_id(&dp_id);
            let _ = append_attribute_value(&mut self.ehdp_attrs, &attr);
        }
        row
    }

    /// Replace an existing exponential histogram data point row with newer data.
    pub fn replace_exp_histogram_dp<V: ExponentialHistogramDataPointView>(
        &mut self,
        row: usize,
        dp: &V,
    ) {
        self.exp_histogram_dps.replace(row, dp);
    }

    /// Append a new summary data point row including its attributes.
    pub fn append_summary_dp<V: SummaryDataPointView>(
        &mut self,
        dp_id: u32,
        metric_id: u16,
        dp: &V,
    ) -> usize {
        let row = self.summary_dps.append(dp_id, metric_id, dp);
        for attr in dp.attributes() {
            self.summary_attrs.append_parent_id(&dp_id);
            let _ = append_attribute_value(&mut self.summary_attrs, &attr);
        }
        row
    }

    /// Replace an existing summary data point row with newer data.
    pub fn replace_summary_dp<V: SummaryDataPointView>(&mut self, row: usize, dp: &V) {
        self.summary_dps.replace(row, dp);
    }

    /// Append exemplars for a number data point. The `dp_id` is the OTAP ID
    /// of the parent data point row. The caller provides the exemplar ID
    /// counter which is incremented for each exemplar appended.
    pub fn append_number_dp_exemplars<V: NumberDataPointView>(
        &mut self,
        dp_id: u32,
        dp: &V,
        next_exemplar_id: &mut u32,
    ) -> Result<(), super::ProcessPdataError> {
        for exemplar in dp.exemplars() {
            let id = super::next_id_32(next_exemplar_id)?;
            append_exemplar(
                &mut self.ndp_exemplars,
                &mut self.ndp_exemplar_attrs,
                id,
                dp_id,
                &exemplar,
            );
        }
        Ok(())
    }

    /// Append exemplars for a histogram data point.
    pub fn append_histogram_dp_exemplars<V: HistogramDataPointView>(
        &mut self,
        dp_id: u32,
        dp: &V,
        next_exemplar_id: &mut u32,
    ) -> Result<(), super::ProcessPdataError> {
        for exemplar in dp.exemplars() {
            let id = super::next_id_32(next_exemplar_id)?;
            append_exemplar(
                &mut self.hdp_exemplars,
                &mut self.hdp_exemplar_attrs,
                id,
                dp_id,
                &exemplar,
            );
        }
        Ok(())
    }

    /// Append exemplars for an exponential histogram data point.
    pub fn append_exp_histogram_dp_exemplars<V: ExponentialHistogramDataPointView>(
        &mut self,
        dp_id: u32,
        dp: &V,
        next_exemplar_id: &mut u32,
    ) -> Result<(), super::ProcessPdataError> {
        for exemplar in dp.exemplars() {
            let id = super::next_id_32(next_exemplar_id)?;
            append_exemplar(
                &mut self.ehdp_exemplars,
                &mut self.ehdp_exemplar_attrs,
                id,
                dp_id,
                &exemplar,
            );
        }
        Ok(())
    }
}

/// Finish building a payload type and set it on the output records.
///
/// When `truncate_len` is `Some(n)`, the record batch is sliced to `n` rows,
/// discarding any rows appended after a checkpoint.
fn finish_payload(
    result: Result<RecordBatch, arrow::error::ArrowError>,
    payload_type: ArrowPayloadType,
    records: &mut OtapArrowRecords,
    truncate_len: Option<usize>,
) -> otap_df_pdata::error::Result<()> {
    // safety: So long as the aggregation logic is keeping arrays
    // the same length, this operation should be infallible.
    let rb = result.expect("Valid record batch");
    let rb = match truncate_len {
        Some(len) => rb.slice(0, len),
        None => rb,
    };
    if rb.num_rows() > 0 {
        records.set(payload_type, rb)?;
    }
    Ok(())
}

/// Finish building an exemplar payload type and set it on the output records.
///
/// Like [`finish_payload`] but the exemplar builder uses adaptive schemas so
/// the `finish` result may produce an empty batch that we can skip.
fn finish_exemplar_payload(
    result: Result<RecordBatch, arrow::error::ArrowError>,
    payload_type: ArrowPayloadType,
    records: &mut OtapArrowRecords,
    truncate_len: Option<usize>,
) -> otap_df_pdata::error::Result<()> {
    let rb = result.expect("Valid record batch");
    let rb = match truncate_len {
        Some(len) if len < rb.num_rows() => rb.slice(0, len),
        _ => rb,
    };
    if rb.num_rows() > 0 {
        records.set(payload_type, rb)?;
    }
    Ok(())
}

/// Append a single exemplar row to the given exemplar builder and its attribute
/// builder. The caller is responsible for allocating the `id`.
fn append_exemplar<E: ExemplarView>(
    exemplar_builder: &mut ExemplarsRecordBatchBuilder,
    attr_builder: &mut AttributesRecordBatchBuilder<u32>,
    id: u32,
    parent_dp_id: u32,
    exemplar: &E,
) {
    exemplar_builder.append_id(id);
    exemplar_builder.append_parent_id(parent_dp_id);
    exemplar_builder.append_time_unix_nano(exemplar.time_unix_nano() as i64);

    let (double, integer) = match exemplar.value() {
        Some(Value::Double(val)) => (Some(val), None),
        Some(Value::Integer(val)) => (None, Some(val)),
        None => (None, None),
    };
    exemplar_builder.append_double_value(double);
    exemplar_builder.append_int_value(integer);

    // Ignore span_id/trace_id errors — they're non-fatal encoding issues
    let _ = exemplar_builder.append_span_id(exemplar.span_id().unwrap_or(&[0; 8]));
    let _ = exemplar_builder.append_trace_id(exemplar.trace_id().unwrap_or(&[0; 16]));

    for kv in exemplar.filtered_attributes() {
        attr_builder.append_parent_id(&id);
        let _ = append_attribute_value(attr_builder, &kv);
    }
}

/// A column of nullable primitive values that supports random-access writes.
///
/// At flush time, converts to an Arrow [`PrimitiveArray`] via zero-copy for
/// the values buffer and a constructed null bitmap.
pub struct NullableColumnBuilder<T: ArrowPrimitiveType> {
    values: Vec<T::Native>,
    validity: BooleanBufferBuilder,
}

impl<T: ArrowPrimitiveType> NullableColumnBuilder<T> {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            validity: BooleanBufferBuilder::new(0),
        }
    }

    pub fn append_value(&mut self, value: T::Native) {
        self.values.push(value);
        self.validity.append(true);
    }

    pub fn append_null(&mut self) {
        self.values.push(T::Native::default());
        self.validity.append(false);
    }

    /// Write a value at `index`. If `index == len`, appends; otherwise
    /// overwrites the existing entry.
    pub fn write_value(&mut self, index: usize, value: T::Native) {
        if index == self.values.len() {
            self.append_value(value);
        } else {
            self.set_value(index, value);
        }
    }

    fn set_value(&mut self, index: usize, value: T::Native) {
        self.values[index] = value;
        self.validity.set_bit(index, true);
    }

    fn set_null(&mut self, index: usize) {
        self.validity.set_bit(index, false);
    }

    /// Write a null at `index`. If `index == len`, appends; otherwise
    /// overwrites the existing entry.
    pub fn write_null(&mut self, index: usize) {
        if index == self.values.len() {
            self.append_null();
        } else {
            self.set_null(index);
        }
    }

    /// Convert to an Arrow [`PrimitiveArray`]. The internal buffers are consumed
    /// and replaced with new 0 size buffers.
    pub fn finish(&mut self) -> PrimitiveArray<T> {
        let values = ScalarBuffer::from(std::mem::take(&mut self.values));
        let nulls = NullBuffer::new(self.validity.finish());
        self.validity = BooleanBufferBuilder::new(0);
        PrimitiveArray::<T>::new(values, Some(nulls))
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.validity = BooleanBufferBuilder::new(0);
    }
}

pub struct NumberDataPointBuilder {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: Vec<i64>,
    pub time_unix_nano: Vec<i64>,
    pub int_value: NullableColumnBuilder<Int64Type>,
    pub double_value: NullableColumnBuilder<Float64Type>,
    pub flags: Vec<u32>,
}

impl NumberDataPointBuilder {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: Vec::new(),
            time_unix_nano: Vec::new(),
            int_value: NullableColumnBuilder::new(),
            double_value: NullableColumnBuilder::new(),
            flags: Vec::new(),
        }
    }

    /// Append a new data point
    pub fn append<V: NumberDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        self.write(row, dp);
        row
    }

    /// Replace a data point's data with the given view
    pub fn replace<V: NumberDataPointView>(&mut self, row: usize, dp: &V) {
        self.write(row, dp);
    }

    /// Helper to write the data point's data which excludes id/parent_id
    fn write<V: NumberDataPointView>(&mut self, row: usize, dp: &V) {
        write_or_append(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano() as i64,
        );
        write_or_append(&mut self.time_unix_nano, row, dp.time_unix_nano() as i64);

        match dp.value() {
            Some(Value::Integer(v)) => {
                self.int_value.write_value(row, v);
                self.double_value.write_null(row);
            }
            Some(Value::Double(v)) => {
                self.int_value.write_null(row);
                self.double_value.write_value(row, v);
            }
            None => {
                self.int_value.write_null(row);
                self.double_value.write_null(row);
            }
        }

        write_or_append(&mut self.flags, row, dp.flags().into_inner());
    }

    /// Consume all of the internal buffers to produce a record batch. If finish
    /// is successful, then this builder is in a cleared state to begin building
    /// a new record batch.
    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return empty_record_batch();
        }

        RecordBatch::try_new(
            NUMBER_DP_SCHEMA.clone(),
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.start_time_unix_nano,
                ))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.time_unix_nano,
                ))),
                Arc::new(self.int_value.finish()),
                Arc::new(self.double_value.finish()),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))),
            ],
        )
    }

    /// Clear all internal buffers
    pub fn clear(&mut self) {
        self.id.clear();
        self.parent_id.clear();
        self.start_time_unix_nano.clear();
        self.time_unix_nano.clear();
        self.int_value.clear();
        self.double_value.clear();
        self.flags.clear();
    }
}

static NUMBER_DP_SCHEMA: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false).with_plain_encoding(),
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(consts::INT_VALUE, DataType::Int64, true),
        Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
        Field::new(consts::FLAGS, DataType::UInt32, false),
    ]))
});

pub struct HistogramDataPointBuilder {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: Vec<i64>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: NullableColumnBuilder<Float64Type>,
    pub bucket_counts: Vec<Vec<u64>>,
    pub explicit_bounds: Vec<Vec<f64>>,
    pub flags: Vec<u32>,
    pub min: NullableColumnBuilder<Float64Type>,
    pub max: NullableColumnBuilder<Float64Type>,
}

impl HistogramDataPointBuilder {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: Vec::new(),
            time_unix_nano: Vec::new(),
            count: Vec::new(),
            sum: NullableColumnBuilder::new(),
            bucket_counts: Vec::new(),
            explicit_bounds: Vec::new(),
            flags: Vec::new(),
            min: NullableColumnBuilder::new(),
            max: NullableColumnBuilder::new(),
        }
    }

    pub fn append<V: HistogramDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        self.write(row, dp);
        row
    }

    pub fn replace<V: HistogramDataPointView>(&mut self, row: usize, dp: &V) {
        self.write(row, dp);
    }

    fn write<V: HistogramDataPointView>(&mut self, row: usize, dp: &V) {
        write_or_append(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano() as i64,
        );
        write_or_append(&mut self.time_unix_nano, row, dp.time_unix_nano() as i64);
        write_or_append(&mut self.count, row, dp.count());
        write_optional_f64(&mut self.sum, row, dp.sum());
        write_or_append(&mut self.bucket_counts, row, dp.bucket_counts().collect());
        write_or_append(
            &mut self.explicit_bounds,
            row,
            dp.explicit_bounds().collect(),
        );
        write_or_append(&mut self.flags, row, dp.flags().into_inner());
        write_optional_f64(&mut self.min, row, dp.min());
        write_optional_f64(&mut self.max, row, dp.max());
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return empty_record_batch();
        }

        RecordBatch::try_new(
            HISTOGRAM_DP_SCHEMA.clone(),
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.start_time_unix_nano,
                ))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.time_unix_nano,
                ))),
                Arc::new(UInt64Array::from(std::mem::take(&mut self.count))),
                Arc::new(build_list_u64(&self.bucket_counts, "item")),
                Arc::new(build_list_f64(&self.explicit_bounds, "item")),
                Arc::new(self.sum.finish()),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))),
                Arc::new(self.min.finish()),
                Arc::new(self.max.finish()),
            ],
        )
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.parent_id.clear();
        self.start_time_unix_nano.clear();
        self.time_unix_nano.clear();
        self.count.clear();
        self.sum.clear();
        self.bucket_counts.clear();
        self.explicit_bounds.clear();
        self.flags.clear();
        self.min.clear();
        self.max.clear();
    }
}

static HISTOGRAM_DP_SCHEMA: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false).with_plain_encoding(),
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(consts::HISTOGRAM_COUNT, DataType::UInt64, false),
        Field::new(
            consts::HISTOGRAM_BUCKET_COUNTS,
            DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
            false,
        ),
        Field::new(
            consts::HISTOGRAM_EXPLICIT_BOUNDS,
            DataType::List(Arc::new(Field::new("item", DataType::Float64, false))),
            false,
        ),
        Field::new(consts::HISTOGRAM_SUM, DataType::Float64, true),
        Field::new(consts::FLAGS, DataType::UInt32, false),
        Field::new(consts::HISTOGRAM_MIN, DataType::Float64, true),
        Field::new(consts::HISTOGRAM_MAX, DataType::Float64, true),
    ]))
});

pub struct ExpHistogramDataPointBuilder {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: Vec<i64>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: NullableColumnBuilder<Float64Type>,
    pub scale: Vec<i32>,
    pub zero_count: Vec<u64>,
    pub positive_offset: Vec<i32>,
    pub positive_bucket_counts: Vec<Vec<u64>>,
    pub negative_offset: Vec<i32>,
    pub negative_bucket_counts: Vec<Vec<u64>>,
    pub flags: Vec<u32>,
    pub min: NullableColumnBuilder<Float64Type>,
    pub max: NullableColumnBuilder<Float64Type>,
    pub zero_threshold: Vec<f64>,
}

impl ExpHistogramDataPointBuilder {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: Vec::new(),
            time_unix_nano: Vec::new(),
            count: Vec::new(),
            sum: NullableColumnBuilder::new(),
            scale: Vec::new(),
            zero_count: Vec::new(),
            positive_offset: Vec::new(),
            positive_bucket_counts: Vec::new(),
            negative_offset: Vec::new(),
            negative_bucket_counts: Vec::new(),
            flags: Vec::new(),
            min: NullableColumnBuilder::new(),
            max: NullableColumnBuilder::new(),
            zero_threshold: Vec::new(),
        }
    }

    pub fn append<V: ExponentialHistogramDataPointView>(
        &mut self,
        id: u32,
        parent_id: u16,
        dp: &V,
    ) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        self.write(row, dp);
        row
    }

    pub fn replace<V: ExponentialHistogramDataPointView>(&mut self, row: usize, dp: &V) {
        // We keep the same parent_id/id here and just write the rest of the fields
        self.write(row, dp);
    }

    fn write<V: ExponentialHistogramDataPointView>(&mut self, row: usize, dp: &V) {
        write_or_append(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano() as i64,
        );
        write_or_append(&mut self.time_unix_nano, row, dp.time_unix_nano() as i64);
        write_or_append(&mut self.count, row, dp.count());
        write_optional_f64(&mut self.sum, row, dp.sum());
        write_or_append(&mut self.scale, row, dp.scale());
        write_or_append(&mut self.zero_count, row, dp.zero_count());
        let (pos_offset, pos_counts) = match dp.positive() {
            Some(b) => (b.offset(), b.bucket_counts().collect()),
            None => (0, Vec::new()),
        };
        write_or_append(&mut self.positive_offset, row, pos_offset);
        write_or_append(&mut self.positive_bucket_counts, row, pos_counts);
        let (neg_offset, neg_counts) = match dp.negative() {
            Some(b) => (b.offset(), b.bucket_counts().collect()),
            None => (0, Vec::new()),
        };
        write_or_append(&mut self.negative_offset, row, neg_offset);
        write_or_append(&mut self.negative_bucket_counts, row, neg_counts);
        write_or_append(&mut self.flags, row, dp.flags().into_inner());
        write_optional_f64(&mut self.min, row, dp.min());
        write_optional_f64(&mut self.max, row, dp.max());
        write_or_append(&mut self.zero_threshold, row, dp.zero_threshold());
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return empty_record_batch();
        }

        let positive = StructArray::from(vec![
            (
                EXP_HISTOGRAM_OFFSET_FIELD.clone(),
                Arc::new(Int32Array::from(std::mem::take(&mut self.positive_offset)))
                    as Arc<dyn Array>,
            ),
            (
                EXP_HISTOGRAM_BUCKET_COUNTS_FIELD.clone(),
                Arc::new(build_list_u64(&self.positive_bucket_counts, "item")) as Arc<dyn Array>,
            ),
        ]);
        self.positive_bucket_counts.clear();

        let negative = StructArray::from(vec![
            (
                EXP_HISTOGRAM_OFFSET_FIELD.clone(),
                Arc::new(Int32Array::from(std::mem::take(&mut self.negative_offset)))
                    as Arc<dyn Array>,
            ),
            (
                EXP_HISTOGRAM_BUCKET_COUNTS_FIELD.clone(),
                Arc::new(build_list_u64(&self.negative_bucket_counts, "item")) as Arc<dyn Array>,
            ),
        ]);
        self.negative_bucket_counts.clear();

        RecordBatch::try_new(
            EXP_HISTOGRAM_DP_SCHEMA.clone(),
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.start_time_unix_nano,
                ))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.time_unix_nano,
                ))),
                Arc::new(UInt64Array::from(std::mem::take(&mut self.count))),
                Arc::new(self.sum.finish()),
                Arc::new(Int32Array::from(std::mem::take(&mut self.scale))),
                Arc::new(UInt64Array::from(std::mem::take(&mut self.zero_count))),
                Arc::new(positive),
                Arc::new(negative),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))),
                Arc::new(self.min.finish()),
                Arc::new(self.max.finish()),
                Arc::new(Float64Array::from(std::mem::take(&mut self.zero_threshold))),
            ],
        )
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.parent_id.clear();
        self.start_time_unix_nano.clear();
        self.time_unix_nano.clear();
        self.count.clear();
        self.sum.clear();
        self.scale.clear();
        self.zero_count.clear();
        self.positive_offset.clear();
        self.positive_bucket_counts.clear();
        self.negative_offset.clear();
        self.negative_bucket_counts.clear();
        self.flags.clear();
        self.min.clear();
        self.max.clear();
        self.zero_threshold.clear();
    }
}

static EXP_HISTOGRAM_OFFSET_FIELD: LazyLock<Arc<Field>> = LazyLock::new(|| {
    Arc::new(Field::new(
        consts::EXP_HISTOGRAM_OFFSET,
        DataType::Int32,
        false,
    ))
});

static EXP_HISTOGRAM_BUCKET_COUNTS_FIELD: LazyLock<Arc<Field>> = LazyLock::new(|| {
    Arc::new(Field::new(
        consts::EXP_HISTOGRAM_BUCKET_COUNTS,
        DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
        false,
    ))
});

static EXP_HISTOGRAM_BUCKETS_FIELDS: LazyLock<Fields> = LazyLock::new(|| {
    Fields::from(vec![
        Field::clone(&EXP_HISTOGRAM_OFFSET_FIELD),
        Field::clone(&EXP_HISTOGRAM_BUCKET_COUNTS_FIELD),
    ])
});

static EXP_HISTOGRAM_DP_SCHEMA: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false).with_plain_encoding(),
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(consts::HISTOGRAM_COUNT, DataType::UInt64, false),
        Field::new(consts::HISTOGRAM_SUM, DataType::Float64, true),
        Field::new(consts::EXP_HISTOGRAM_SCALE, DataType::Int32, false),
        Field::new(consts::EXP_HISTOGRAM_ZERO_COUNT, DataType::UInt64, false),
        Field::new(
            consts::EXP_HISTOGRAM_POSITIVE,
            DataType::Struct(EXP_HISTOGRAM_BUCKETS_FIELDS.clone()),
            false,
        ),
        Field::new(
            consts::EXP_HISTOGRAM_NEGATIVE,
            DataType::Struct(EXP_HISTOGRAM_BUCKETS_FIELDS.clone()),
            false,
        ),
        Field::new(consts::FLAGS, DataType::UInt32, false),
        Field::new(consts::HISTOGRAM_MIN, DataType::Float64, true),
        Field::new(consts::HISTOGRAM_MAX, DataType::Float64, true),
        Field::new(
            consts::EXP_HISTOGRAM_ZERO_THRESHOLD,
            DataType::Float64,
            true,
        ),
    ]))
});

pub struct SummaryDataPointBuilder {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: Vec<i64>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: Vec<f64>,
    pub quantiles: Vec<Vec<(f64, f64)>>,
    pub flags: Vec<u32>,
}

impl SummaryDataPointBuilder {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: Vec::new(),
            time_unix_nano: Vec::new(),
            count: Vec::new(),
            sum: Vec::new(),
            quantiles: Vec::new(),
            flags: Vec::new(),
        }
    }

    pub fn append<V: SummaryDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        self.write(row, dp);
        row
    }

    pub fn replace<V: SummaryDataPointView>(&mut self, row: usize, dp: &V) {
        self.write(row, dp);
    }

    fn write<V: SummaryDataPointView>(&mut self, row: usize, dp: &V) {
        write_or_append(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano() as i64,
        );
        write_or_append(&mut self.time_unix_nano, row, dp.time_unix_nano() as i64);
        write_or_append(&mut self.count, row, dp.count());
        write_or_append(&mut self.sum, row, dp.sum());
        write_or_append(
            &mut self.quantiles,
            row,
            dp.quantile_values()
                .map(|q| (q.quantile(), q.value()))
                .collect(),
        );
        write_or_append(&mut self.flags, row, dp.flags().into_inner());
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return empty_record_batch();
        }

        // Build the quantiles list of structs
        let mut offsets = Vec::with_capacity(self.quantiles.len() + 1);
        let mut q_vals = Vec::new();
        let mut v_vals = Vec::new();
        offsets.push(0i32);
        for row in &self.quantiles {
            for &(q, v) in row {
                q_vals.push(q);
                v_vals.push(v);
            }
            offsets.push(q_vals.len() as i32);
        }
        let quantile_struct = StructArray::from(vec![
            (
                SUMMARY_QUANTILE_FIELD.clone(),
                Arc::new(Float64Array::from(q_vals)) as Arc<dyn Array>,
            ),
            (
                SUMMARY_VALUE_FIELD.clone(),
                Arc::new(Float64Array::from(v_vals)) as Arc<dyn Array>,
            ),
        ]);
        let quantile_list = ListArray::new(
            SUMMARY_QUANTILE_LIST_FIELD.clone(),
            OffsetBuffer::new(ScalarBuffer::from(offsets)),
            Arc::new(quantile_struct),
            None,
        );
        self.quantiles.clear();

        RecordBatch::try_new(
            SUMMARY_DP_SCHEMA.clone(),
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.start_time_unix_nano,
                ))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.time_unix_nano,
                ))),
                Arc::new(UInt64Array::from(std::mem::take(&mut self.count))),
                Arc::new(Float64Array::from(std::mem::take(&mut self.sum))),
                Arc::new(quantile_list),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))),
            ],
        )
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.parent_id.clear();
        self.start_time_unix_nano.clear();
        self.time_unix_nano.clear();
        self.count.clear();
        self.sum.clear();
        self.quantiles.clear();
        self.flags.clear();
    }
}

static SUMMARY_QUANTILE_FIELD: LazyLock<Arc<Field>> = LazyLock::new(|| {
    Arc::new(Field::new(
        consts::SUMMARY_QUANTILE,
        DataType::Float64,
        false,
    ))
});

static SUMMARY_VALUE_FIELD: LazyLock<Arc<Field>> =
    LazyLock::new(|| Arc::new(Field::new(consts::SUMMARY_VALUE, DataType::Float64, false)));

static SUMMARY_QUANTILE_FIELDS: LazyLock<Fields> = LazyLock::new(|| {
    Fields::from(vec![
        Field::clone(&SUMMARY_QUANTILE_FIELD),
        Field::clone(&SUMMARY_VALUE_FIELD),
    ])
});

static SUMMARY_QUANTILE_LIST_FIELD: LazyLock<Arc<Field>> = LazyLock::new(|| {
    Arc::new(Field::new(
        "item",
        DataType::Struct(SUMMARY_QUANTILE_FIELDS.clone()),
        false,
    ))
});

static SUMMARY_DP_SCHEMA: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false).with_plain_encoding(),
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(consts::SUMMARY_COUNT, DataType::UInt64, false),
        Field::new(consts::SUMMARY_SUM, DataType::Float64, false),
        Field::new(
            consts::SUMMARY_QUANTILE_VALUES,
            DataType::List(SUMMARY_QUANTILE_LIST_FIELD.clone()),
            false,
        ),
        Field::new(consts::FLAGS, DataType::UInt32, false),
    ]))
});

fn empty_record_batch() -> Result<RecordBatch, arrow::error::ArrowError> {
    Ok(RecordBatch::new_empty(Schema::empty().into()))
}

/// Write `value` into `vec` at `index`. Appends if `index == vec.len()`,
/// otherwise overwrites.
fn write_or_append<T>(vec: &mut Vec<T>, index: usize, value: T) {
    if index == vec.len() {
        vec.push(value);
    } else {
        vec[index] = value;
    }
}

fn write_optional_f64(
    col: &mut NullableColumnBuilder<Float64Type>,
    index: usize,
    value: Option<f64>,
) {
    match value {
        Some(v) => col.write_value(index, v),
        None => col.write_null(index),
    }
}

fn build_list_u64(data: &[Vec<u64>], field_name: &str) -> ListArray {
    let mut offsets = Vec::with_capacity(data.len() + 1);
    let mut values = Vec::new();
    offsets.push(0i32);
    for row in data {
        values.extend_from_slice(row);
        offsets.push(values.len() as i32);
    }
    let values_array = UInt64Array::from(values);
    let offsets_buffer = OffsetBuffer::new(ScalarBuffer::from(offsets));
    ListArray::new(
        Arc::new(Field::new(field_name, DataType::UInt64, false)),
        offsets_buffer,
        Arc::new(values_array),
        None,
    )
}

fn build_list_f64(data: &[Vec<f64>], field_name: &str) -> ListArray {
    let mut offsets = Vec::with_capacity(data.len() + 1);
    let mut values = Vec::new();
    offsets.push(0i32);
    for row in data {
        values.extend_from_slice(row);
        offsets.push(values.len() as i32);
    }
    let values_array = Float64Array::from(values);
    let offsets_buffer = OffsetBuffer::new(ScalarBuffer::from(offsets));
    ListArray::new(
        Arc::new(Field::new(field_name, DataType::Float64, false)),
        offsets_buffer,
        Arc::new(values_array),
        None,
    )
}

/// Tracks the builder row index and latest timestamp for a data point stream.
pub(super) struct StreamMeta {
    pub(super) dp_row_index: usize,
    pub(super) time_unix_nano: u64,
}
