// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    Int32ArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor, StringArrayAccessor,
    get_bool_array_opt, get_u8_array, get_u16_array,
};
use crate::error::{self, Error, Result};
use crate::otap::OtapArrowRecords;
use crate::otlp::attributes::{Attribute16Arrays, Attribute32Arrays};
use crate::otlp::common::{
    BatchSorter, ChildIndexIter, ResourceArrays, ScopeArrays, SortedBatchCursor,
    proto_encode_instrumentation_scope, proto_encode_resource,
};
use crate::otlp::metrics::data_points::exp_histogram::{
    ExpHistogramDpArrays, proto_encode_exp_hist_data_point,
};
use crate::otlp::metrics::data_points::histogram::{
    HistogramDpArrays, proto_encode_histogram_data_point,
};
use crate::otlp::metrics::data_points::number::{NumberDpArrays, proto_encode_number_data_point};
use crate::otlp::metrics::data_points::summary::{
    SummaryDpArrays, proto_encode_summary_data_point,
};
use crate::otlp::metrics::exemplar::ExemplarArrays;
use crate::otlp::{ProtoBuffer, ProtoBytesEncoder};
use crate::proto::consts::field_num::metrics::{
    EXPONENTIAL_HISTOGRAM_DATA_POINTS, GAUGE_DATA_POINTS, HISTOGRAM_AGGREGATION_TEMPORALITY,
    HISTOGRAM_DATA_POINTS, METRIC_DESCRIPTION, METRIC_EXPONENTIAL_HISTOGRAM, METRIC_GAUGE,
    METRIC_HISTOGRAM, METRIC_NAME, METRIC_SUM, METRIC_SUMMARY, METRIC_UNIT,
    METRICS_DATA_RESOURCE_METRICS, RESOURCE_METRICS_RESOURCE, RESOURCE_METRICS_SCHEMA_URL,
    RESOURCE_METRICS_SCOPE_METRICS, SCOPE_METRICS_METRICS, SCOPE_METRICS_SCHEMA_URL,
    SCOPE_METRICS_SCOPE, SUM_AGGREGATION_TEMPORALITY, SUM_DATA_POINTS, SUM_IS_MONOTONIC,
    SUMMARY_DATA_POINTS,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;
use arrow::array::{BooleanArray, RecordBatch, UInt8Array, UInt16Array};
use num_enum::TryFromPrimitive;
use snafu::{OptionExt, ResultExt};

pub mod data_points;
pub mod exemplar;

#[derive(Copy, Clone, Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum MetricType {
    Empty = 0,
    Gauge = 1,
    Sum = 2,
    Histogram = 3,
    ExponentialHistogram = 4,
    Summary = 5,
}

struct MetricsArrays<'a> {
    id: &'a UInt16Array,
    metric_type: &'a UInt8Array,
    schema_url: Option<StringArrayAccessor<'a>>,
    name: StringArrayAccessor<'a>,
    description: Option<StringArrayAccessor<'a>>,
    unit: Option<StringArrayAccessor<'a>>,
    aggregation_temporality: Option<Int32ArrayAccessor<'a>>,
    is_monotonic: Option<&'a BooleanArray>,
}

impl<'a> TryFrom<&'a RecordBatch> for MetricsArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let id = get_u16_array(rb, consts::ID)?;
        let metric_type = get_u8_array(rb, consts::METRIC_TYPE)?;
        let name = StringArrayAccessor::try_new(
            rb.column_by_name(consts::NAME)
                .context(error::ColumnNotFoundSnafu { name: consts::NAME })?,
        )?;

        let description = rb
            .column_by_name(consts::DESCRIPTION)
            .map(StringArrayAccessor::try_new)
            .transpose()?;
        let schema_url = rb
            .column_by_name(consts::SCHEMA_URL)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let unit = rb
            .column_by_name(consts::UNIT)
            .map(StringArrayAccessor::try_new)
            .transpose()?;
        let aggregation_temporality = rb
            .column_by_name(consts::AGGREGATION_TEMPORALITY)
            .map(Int32ArrayAccessor::try_new)
            .transpose()?;
        let is_monotonic = get_bool_array_opt(rb, consts::IS_MONOTONIC)?;
        Ok(Self {
            id,
            metric_type,
            name,
            description,
            schema_url,
            unit,
            aggregation_temporality,
            is_monotonic,
        })
    }
}

pub trait AppendAndGet<T> {
    fn append_and_get(&mut self) -> &mut T;
}

impl<T> AppendAndGet<T> for Vec<T>
where
    T: Default,
{
    fn append_and_get(&mut self) -> &mut T {
        self.push(T::default());
        self.last_mut().expect("vec is not empty")
    }
}

pub struct MetricsDataArrays<'a> {
    metrics_arrays: MetricsArrays<'a>,
    scope_arrays: ScopeArrays<'a>,
    resource_arrays: ResourceArrays<'a>,
    metrics_attrs: Option<Attribute16Arrays<'a>>,
    scope_attrs: Option<Attribute16Arrays<'a>>,
    resource_attrs: Option<Attribute16Arrays<'a>>,

    summary_dp_arrays: Option<SummaryDpArrays<'a>>,
    summary_dp_attrs: Option<Attribute32Arrays<'a>>,

    number_dp_arrays: Option<NumberDpArrays<'a>>,
    number_dp_attrs: Option<Attribute32Arrays<'a>>,
    number_dp_exemplar_arrays: Option<ExemplarArrays<'a>>,
    number_dp_exemplar_attrs: Option<Attribute32Arrays<'a>>,

    hist_dp_arrays: Option<HistogramDpArrays<'a>>,
    hist_dp_attrs: Option<Attribute32Arrays<'a>>,
    hist_dp_exemplar_arrays: Option<ExemplarArrays<'a>>,
    hist_dp_exemplar_attrs: Option<Attribute32Arrays<'a>>,

    exp_hist_dp_arrays: Option<ExpHistogramDpArrays<'a>>,
    exp_hist_dp_attrs: Option<Attribute32Arrays<'a>>,
    exp_hist_dp_exemplar_arrays: Option<ExemplarArrays<'a>>,
    exp_hist_dp_exemplar_attrs: Option<Attribute32Arrays<'a>>,
}

impl<'a> TryFrom<&'a OtapArrowRecords> for MetricsDataArrays<'a> {
    type Error = Error;

    fn try_from(otap_batch: &'a OtapArrowRecords) -> Result<Self> {
        let metrics_rb = otap_batch
            .get(ArrowPayloadType::UnivariateMetrics)
            .or_else(|| otap_batch.get(ArrowPayloadType::MultivariateMetrics))
            .context(error::MetricRecordNotFoundSnafu)?;

        Ok(Self {
            metrics_arrays: MetricsArrays::try_from(metrics_rb)?,
            scope_arrays: ScopeArrays::try_from(metrics_rb)?,
            resource_arrays: ResourceArrays::try_from(metrics_rb)?,
            metrics_attrs: otap_batch
                .get(ArrowPayloadType::MetricAttrs)
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            scope_attrs: otap_batch
                .get(ArrowPayloadType::ScopeAttrs)
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            resource_attrs: otap_batch
                .get(ArrowPayloadType::ResourceAttrs)
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            summary_dp_arrays: otap_batch
                .get(ArrowPayloadType::SummaryDataPoints)
                .map(SummaryDpArrays::try_from)
                .transpose()?,
            summary_dp_attrs: otap_batch
                .get(ArrowPayloadType::SummaryDpAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()?,
            number_dp_arrays: otap_batch
                .get(ArrowPayloadType::NumberDataPoints)
                .map(NumberDpArrays::try_from)
                .transpose()?,
            number_dp_attrs: otap_batch
                .get(ArrowPayloadType::NumberDpAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()?,
            number_dp_exemplar_arrays: otap_batch
                .get(ArrowPayloadType::NumberDpExemplars)
                .map(ExemplarArrays::try_from)
                .transpose()?,
            number_dp_exemplar_attrs: otap_batch
                .get(ArrowPayloadType::NumberDpExemplarAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()?,
            hist_dp_arrays: otap_batch
                .get(ArrowPayloadType::HistogramDataPoints)
                .map(HistogramDpArrays::try_from)
                .transpose()?,
            hist_dp_attrs: otap_batch
                .get(ArrowPayloadType::HistogramDpAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()?,
            hist_dp_exemplar_arrays: otap_batch
                .get(ArrowPayloadType::HistogramDpExemplars)
                .map(ExemplarArrays::try_from)
                .transpose()?,
            hist_dp_exemplar_attrs: otap_batch
                .get(ArrowPayloadType::HistogramDpExemplarAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()?,
            exp_hist_dp_arrays: otap_batch
                .get(ArrowPayloadType::ExpHistogramDataPoints)
                .map(ExpHistogramDpArrays::try_from)
                .transpose()?,
            exp_hist_dp_attrs: otap_batch
                .get(ArrowPayloadType::ExpHistogramDpAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()?,
            exp_hist_dp_exemplar_arrays: otap_batch
                .get(ArrowPayloadType::ExpHistogramDpExemplars)
                .map(ExemplarArrays::try_from)
                .transpose()?,
            exp_hist_dp_exemplar_attrs: otap_batch
                .get(ArrowPayloadType::ExpHistogramDpExemplarAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()?,
        })
    }
}

pub struct MetricsProtoBytesEncoder {
    batch_sorter: BatchSorter,
    root_cursor: SortedBatchCursor,
    resource_attrs_cursor: SortedBatchCursor,
    scope_attrs_cursor: SortedBatchCursor,
    metrics_attrs_cursor: SortedBatchCursor,

    summary_dp_cursor: SortedBatchCursor,
    summary_dp_attrs_cursor: SortedBatchCursor,

    number_dp_cursor: SortedBatchCursor,
    number_dp_attrs_cursor: SortedBatchCursor,
    number_dp_exemplars_cursor: SortedBatchCursor,
    number_dp_exemplars_attrs_cursor: SortedBatchCursor,

    hist_dp_cursor: SortedBatchCursor,
    hist_dp_attrs_cursor: SortedBatchCursor,
    hist_dp_exemplars_cursor: SortedBatchCursor,
    hist_dp_exemplars_attrs_cursor: SortedBatchCursor,

    exp_hist_dp_cursor: SortedBatchCursor,
    exp_hist_dp_attrs_cursor: SortedBatchCursor,
    exp_hist_exemplars_cursor: SortedBatchCursor,
    exp_hist_exemplars_attrs_cursor: SortedBatchCursor,
}

impl Default for MetricsProtoBytesEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtoBytesEncoder for MetricsProtoBytesEncoder {
    fn encode(
        &mut self,
        otap_batch: &mut OtapArrowRecords,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        otap_batch.decode_transport_optimized_ids()?;
        let metrics_data_arrays = MetricsDataArrays::try_from(&*otap_batch)?;

        self.reset();

        let metrics_rb = otap_batch
            .get(ArrowPayloadType::UnivariateMetrics)
            .or_else(|| otap_batch.get(ArrowPayloadType::MultivariateMetrics))
            .context(error::MetricRecordNotFoundSnafu)?;
        self.batch_sorter
            .init_cursor_for_root_batch(metrics_rb, &mut self.root_cursor)?;

        if let Some(res_attrs) = metrics_data_arrays.resource_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &res_attrs.parent_id,
                &mut self.resource_attrs_cursor,
            );
        }

        if let Some(scope_attrs) = metrics_data_arrays.scope_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &scope_attrs.parent_id,
                &mut self.scope_attrs_cursor,
            );
        }

        if let Some(metrics_attrs) = metrics_data_arrays.metrics_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &metrics_attrs.parent_id,
                &mut self.metrics_attrs_cursor,
            );
        }

        if let Some(summary_dp_arrays) = metrics_data_arrays.summary_dp_arrays.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &MaybeDictArrayAccessor::Native(summary_dp_arrays.parent_id),
                &mut self.summary_dp_cursor,
            );
        }

        if let Some(summary_dp_attrs) = metrics_data_arrays.summary_dp_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &summary_dp_attrs.parent_id,
                &mut self.summary_dp_attrs_cursor,
            );
        }

        if let Some(number_dp_arrays) = metrics_data_arrays.number_dp_arrays.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &MaybeDictArrayAccessor::Native(number_dp_arrays.parent_id),
                &mut self.number_dp_cursor,
            );
        }

        if let Some(number_dp_attrs) = metrics_data_arrays.number_dp_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &number_dp_attrs.parent_id,
                &mut self.number_dp_attrs_cursor,
            );
        }

        if let Some(number_dp_exemplar_arrays) =
            metrics_data_arrays.number_dp_exemplar_arrays.as_ref()
        {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &number_dp_exemplar_arrays.parent_id,
                &mut self.number_dp_exemplars_cursor,
            );
        }

        if let Some(number_dp_exemplar_attrs) =
            metrics_data_arrays.number_dp_exemplar_attrs.as_ref()
        {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &number_dp_exemplar_attrs.parent_id,
                &mut self.number_dp_exemplars_attrs_cursor,
            );
        }

        if let Some(hist_dp_arrays) = metrics_data_arrays.hist_dp_arrays.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &MaybeDictArrayAccessor::Native(hist_dp_arrays.parent_id),
                &mut self.hist_dp_cursor,
            );
        }

        if let Some(hist_dp_attrs) = metrics_data_arrays.hist_dp_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &hist_dp_attrs.parent_id,
                &mut self.hist_dp_attrs_cursor,
            );
        }

        if let Some(hist_dp_exemplar_arrays) = metrics_data_arrays.hist_dp_exemplar_arrays.as_ref()
        {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &hist_dp_exemplar_arrays.parent_id,
                &mut self.hist_dp_exemplars_cursor,
            );
        }

        if let Some(hist_dp_exemplar_attrs) = metrics_data_arrays.hist_dp_exemplar_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &hist_dp_exemplar_attrs.parent_id,
                &mut self.hist_dp_exemplars_attrs_cursor,
            );
        }

        if let Some(exp_hist_dp_arrays) = metrics_data_arrays.exp_hist_dp_arrays.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &MaybeDictArrayAccessor::Native(exp_hist_dp_arrays.parent_id),
                &mut self.exp_hist_dp_cursor,
            );
        }

        if let Some(exp_hist_dp_attrs) = metrics_data_arrays.exp_hist_dp_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &exp_hist_dp_attrs.parent_id,
                &mut self.exp_hist_dp_attrs_cursor,
            );
        }

        if let Some(exp_hist_dp_exemplar_arrays) =
            metrics_data_arrays.exp_hist_dp_exemplar_arrays.as_ref()
        {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &exp_hist_dp_exemplar_arrays.parent_id,
                &mut self.exp_hist_exemplars_cursor,
            );
        }

        if let Some(exp_hist_dp_exemplar_attrs) =
            metrics_data_arrays.exp_hist_dp_exemplar_attrs.as_ref()
        {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &exp_hist_dp_exemplar_attrs.parent_id,
                &mut self.exp_hist_exemplars_attrs_cursor,
            );
        }

        // encode all `ResourceMetrics` for this `MetricsData`
        loop {
            proto_encode_len_delimited_unknown_size!(
                METRICS_DATA_RESOURCE_METRICS,
                self.encode_resource_metrics(&metrics_data_arrays, result_buf)?,
                result_buf
            );

            if self.root_cursor.finished() {
                break;
            }
        }

        Ok(())
    }
}

impl MetricsProtoBytesEncoder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            batch_sorter: BatchSorter::new(),
            root_cursor: SortedBatchCursor::new(),
            resource_attrs_cursor: SortedBatchCursor::new(),
            scope_attrs_cursor: SortedBatchCursor::new(),
            metrics_attrs_cursor: SortedBatchCursor::new(),

            summary_dp_cursor: SortedBatchCursor::new(),
            summary_dp_attrs_cursor: SortedBatchCursor::new(),

            number_dp_cursor: SortedBatchCursor::new(),
            number_dp_attrs_cursor: SortedBatchCursor::new(),
            number_dp_exemplars_cursor: SortedBatchCursor::new(),
            number_dp_exemplars_attrs_cursor: SortedBatchCursor::new(),

            hist_dp_cursor: SortedBatchCursor::new(),
            hist_dp_attrs_cursor: SortedBatchCursor::new(),
            hist_dp_exemplars_cursor: SortedBatchCursor::new(),
            hist_dp_exemplars_attrs_cursor: SortedBatchCursor::new(),

            exp_hist_dp_cursor: SortedBatchCursor::new(),
            exp_hist_dp_attrs_cursor: SortedBatchCursor::new(),
            exp_hist_exemplars_cursor: SortedBatchCursor::new(),
            exp_hist_exemplars_attrs_cursor: SortedBatchCursor::new(),
        }
    }

    pub fn reset(&mut self) {
        self.root_cursor.reset();
        self.resource_attrs_cursor.reset();
        self.scope_attrs_cursor.reset();
        self.metrics_attrs_cursor.reset();

        self.summary_dp_cursor.reset();
        self.summary_dp_attrs_cursor.reset();

        self.number_dp_cursor.reset();
        self.number_dp_attrs_cursor.reset();
        self.number_dp_exemplars_cursor.reset();
        self.number_dp_exemplars_attrs_cursor.reset();

        self.hist_dp_cursor.reset();
        self.hist_dp_attrs_cursor.reset();
        self.hist_dp_exemplars_cursor.reset();
        self.hist_dp_exemplars_attrs_cursor.reset();

        self.exp_hist_dp_cursor.reset();
        self.exp_hist_dp_attrs_cursor.reset();
        self.exp_hist_exemplars_cursor.reset();
        self.exp_hist_exemplars_attrs_cursor.reset();
    }

    fn encode_resource_metrics(
        &mut self,
        metrics_data_arrays: &MetricsDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()), // no more rows to visit
        };

        // encode the `Resource`
        proto_encode_len_delimited_unknown_size!(
            RESOURCE_METRICS_RESOURCE,
            proto_encode_resource(
                index,
                &metrics_data_arrays.resource_arrays,
                metrics_data_arrays.resource_attrs.as_ref(),
                &mut self.resource_attrs_cursor,
                result_buf,
            )?,
            result_buf
        );

        // encode all the `ScopeMetrics` for this `ResourceMetrics`
        let resource_id = metrics_data_arrays.resource_arrays.id.value_at(index);
        loop {
            proto_encode_len_delimited_unknown_size!(
                RESOURCE_METRICS_SCOPE_METRICS,
                self.encode_scope_metrics(metrics_data_arrays, result_buf)?,
                result_buf
            );

            // break if we've reached the end of the record batch
            if self.root_cursor.finished() {
                break;
            }

            // safety: we've just checked if the cursor is finished, so safe to expect here
            let next_index = self.root_cursor.curr_index().expect("cursor not finished");

            // check if we've found a new resource ID. If so, break
            if resource_id != metrics_data_arrays.resource_arrays.id.value_at(next_index) {
                break;
            }
        }

        // encode schema url
        if let Some(col) = &metrics_data_arrays.resource_arrays.schema_url {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_string(RESOURCE_METRICS_SCHEMA_URL, val);
            }
        }

        Ok(())
    }

    fn encode_scope_metrics(
        &mut self,
        metrics_data_arrays: &MetricsDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()),
        };

        // encode the `InstrumentationScope`
        proto_encode_len_delimited_unknown_size!(
            SCOPE_METRICS_SCOPE,
            proto_encode_instrumentation_scope(
                index,
                &metrics_data_arrays.scope_arrays,
                metrics_data_arrays.scope_attrs.as_ref(),
                &mut self.scope_attrs_cursor,
                result_buf
            )?,
            result_buf
        );

        // encode all `Metrics` for this `ScopeMetrics`
        let scope_id = metrics_data_arrays.scope_arrays.id.value_at(index);

        loop {
            proto_encode_len_delimited_unknown_size!(
                SCOPE_METRICS_METRICS,
                self.encode_metrics(metrics_data_arrays, result_buf)?,
                result_buf
            );

            // break if we've reached the end of the record batch
            if self.root_cursor.finished() {
                break;
            }

            // safety: we've just checked if the cursor is finished, so safe to expect here
            let next_index = self.root_cursor.curr_index().expect("cursor not finished");

            // check if we've found a new scope ID. If so, break
            if scope_id != metrics_data_arrays.scope_arrays.id.value_at(next_index) {
                break;
            }
        }

        // encode the schema url
        if let Some(col) = &metrics_data_arrays.metrics_arrays.schema_url {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_string(SCOPE_METRICS_SCHEMA_URL, val);
            }
        }

        Ok(())
    }

    fn encode_metrics(
        &mut self,
        metrics_data_arrays: &MetricsDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()),
        };

        let metrics_arrays = &metrics_data_arrays.metrics_arrays;

        if let Some(val) = metrics_arrays.name.str_at(index) {
            result_buf.encode_string(METRIC_NAME, val);
        }

        if let Some(col) = &metrics_arrays.description {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_string(METRIC_DESCRIPTION, val);
            }
        }

        if let Some(col) = &metrics_arrays.unit {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_string(METRIC_UNIT, val);
            }
        }

        if let Some(metric_type_val) = &metrics_arrays.metric_type.value_at(index) {
            let metric_type = MetricType::try_from(*metric_type_val).context(
                error::UnrecognizedMetricTypeSnafu {
                    metric_type: *metric_type_val,
                },
            )?;

            if let Some(id) = metrics_arrays.id.value_at(index) {
                let field_num = match metric_type {
                    MetricType::Empty => 0,
                    MetricType::ExponentialHistogram => METRIC_EXPONENTIAL_HISTOGRAM,
                    MetricType::Histogram => METRIC_HISTOGRAM,
                    MetricType::Gauge => METRIC_GAUGE,
                    MetricType::Sum => METRIC_SUM,
                    MetricType::Summary => METRIC_SUMMARY,
                };

                // encode metric data points (if it's not the Empty metric type)
                if field_num != 0 {
                    proto_encode_len_delimited_unknown_size!(
                        field_num,
                        self.encode_metric_data_points(
                            id,
                            metric_type,
                            metrics_data_arrays,
                            result_buf
                        )?,
                        result_buf
                    )
                }
            }
        }

        self.root_cursor.advance();

        Ok(())
    }

    fn encode_metric_data_points(
        &mut self,
        parent_id: u16,
        metric_type: MetricType,
        metrics_data_arrays: &MetricsDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let root_index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()),
        };
        let aggregation_temporality = metrics_data_arrays
            .metrics_arrays
            .aggregation_temporality
            .value_at(root_index);
        let is_monotonic = metrics_data_arrays
            .metrics_arrays
            .is_monotonic
            .value_at(root_index);

        match metric_type {
            MetricType::Gauge | MetricType::Sum => {
                if let Some(number_dp_arrays) = &metrics_data_arrays.number_dp_arrays {
                    let parent_ids = MaybeDictArrayAccessor::Native(number_dp_arrays.parent_id);
                    let dp_index_iter =
                        ChildIndexIter::new(parent_id, &parent_ids, &mut self.number_dp_cursor);
                    let field_num = if metric_type == MetricType::Gauge {
                        GAUGE_DATA_POINTS
                    } else {
                        SUM_DATA_POINTS
                    };
                    for dp_index in dp_index_iter {
                        proto_encode_len_delimited_unknown_size!(
                            field_num,
                            proto_encode_number_data_point(
                                dp_index,
                                number_dp_arrays,
                                metrics_data_arrays.number_dp_attrs.as_ref(),
                                &mut self.number_dp_attrs_cursor,
                                metrics_data_arrays.number_dp_exemplar_arrays.as_ref(),
                                &mut self.number_dp_exemplars_cursor,
                                metrics_data_arrays.number_dp_exemplar_attrs.as_ref(),
                                &mut self.number_dp_exemplars_attrs_cursor,
                                result_buf
                            )?,
                            result_buf
                        );
                    }
                }
                if metric_type == MetricType::Sum {
                    if let Some(aggregation_temporality) = aggregation_temporality {
                        result_buf
                            .encode_field_tag(SUM_AGGREGATION_TEMPORALITY, wire_types::VARINT);
                        result_buf.encode_varint(aggregation_temporality as u64);
                    }

                    if let Some(is_monotonic) = is_monotonic {
                        result_buf.encode_field_tag(SUM_IS_MONOTONIC, wire_types::VARINT);
                        result_buf.encode_varint(is_monotonic as u64);
                    }
                }
            }

            MetricType::Histogram => {
                if let Some(hist_dp_arrays) = &metrics_data_arrays.hist_dp_arrays {
                    let parent_ids = MaybeDictArrayAccessor::Native(hist_dp_arrays.parent_id);
                    let dp_index_iter =
                        ChildIndexIter::new(parent_id, &parent_ids, &mut self.hist_dp_cursor);
                    for dp_index in dp_index_iter {
                        proto_encode_len_delimited_unknown_size!(
                            HISTOGRAM_DATA_POINTS,
                            proto_encode_histogram_data_point(
                                dp_index,
                                hist_dp_arrays,
                                metrics_data_arrays.hist_dp_attrs.as_ref(),
                                &mut self.hist_dp_attrs_cursor,
                                metrics_data_arrays.hist_dp_exemplar_arrays.as_ref(),
                                &mut self.hist_dp_exemplars_cursor,
                                metrics_data_arrays.hist_dp_exemplar_attrs.as_ref(),
                                &mut self.hist_dp_exemplars_attrs_cursor,
                                result_buf
                            )?,
                            result_buf
                        );
                    }
                }
                if let Some(aggregation_temporality) = aggregation_temporality {
                    result_buf
                        .encode_field_tag(HISTOGRAM_AGGREGATION_TEMPORALITY, wire_types::VARINT);
                    result_buf.encode_varint(aggregation_temporality as u64);
                }
            }
            MetricType::ExponentialHistogram => {
                if let Some(exp_hist_dp_arrays) = &metrics_data_arrays.exp_hist_dp_arrays {
                    let parent_ids = MaybeDictArrayAccessor::Native(exp_hist_dp_arrays.parent_id);
                    let dp_index_iter =
                        ChildIndexIter::new(parent_id, &parent_ids, &mut self.exp_hist_dp_cursor);
                    for dp_index in dp_index_iter {
                        proto_encode_len_delimited_unknown_size!(
                            EXPONENTIAL_HISTOGRAM_DATA_POINTS,
                            proto_encode_exp_hist_data_point(
                                dp_index,
                                exp_hist_dp_arrays,
                                metrics_data_arrays.exp_hist_dp_attrs.as_ref(),
                                &mut self.exp_hist_dp_attrs_cursor,
                                metrics_data_arrays.exp_hist_dp_exemplar_arrays.as_ref(),
                                &mut self.exp_hist_exemplars_cursor,
                                metrics_data_arrays.exp_hist_dp_exemplar_attrs.as_ref(),
                                &mut self.exp_hist_exemplars_attrs_cursor,
                                result_buf
                            )?,
                            result_buf
                        );
                    }
                }
                if let Some(aggregation_temporality) = aggregation_temporality {
                    result_buf
                        .encode_field_tag(HISTOGRAM_AGGREGATION_TEMPORALITY, wire_types::VARINT);
                    result_buf.encode_varint(aggregation_temporality as u64);
                }
            }
            MetricType::Summary => {
                if let Some(summary_dp_arrays) = &metrics_data_arrays.summary_dp_arrays {
                    let parent_ids = MaybeDictArrayAccessor::Native(summary_dp_arrays.parent_id);
                    let dp_index_iter =
                        ChildIndexIter::new(parent_id, &parent_ids, &mut self.summary_dp_cursor);
                    for dp_index in dp_index_iter {
                        proto_encode_len_delimited_unknown_size!(
                            SUMMARY_DATA_POINTS,
                            proto_encode_summary_data_point(
                                dp_index,
                                summary_dp_arrays,
                                metrics_data_arrays.summary_dp_attrs.as_ref(),
                                &mut self.summary_dp_attrs_cursor,
                                result_buf
                            )?,
                            result_buf
                        );
                    }
                }
            }
            MetricType::Empty => {
                // nothing to do
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{
        ArrayRef, FixedSizeBinaryArray, Float64Array, Int32Array, Int64Array, ListArray,
        StringArray, StructArray, TimestampNanosecondArray, UInt32Array, UInt64Array,
    };
    use arrow::buffer::OffsetBuffer;
    use arrow::datatypes::{DataType, Field, FieldRef, Fields, Schema, TimeUnit};
    use pretty_assertions::assert_eq;
    use prost::Message;
    use std::sync::Arc;

    use crate::otap::Metrics;
    use crate::otlp::attributes::AttributeValueType;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::metrics::v1::exemplar::Value as ExemplarValue;
    use crate::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
    use crate::proto::opentelemetry::metrics::v1::number_data_point::Value;
    use crate::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
    use crate::proto::opentelemetry::metrics::v1::{
        Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
        HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
        Sum, Summary, SummaryDataPoint, metric,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::schema::FieldExt;

    #[test]
    fn test_metrics_proto_encode() {
        let res_struct_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
        ]);
        let scope_struct_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
            Field::new(consts::NAME, DataType::Utf8, true),
        ]);

        let metrics_record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(res_struct_fields.clone()),
                    true,
                ),
                Field::new(
                    consts::SCOPE,
                    DataType::Struct(scope_struct_fields.clone()),
                    true,
                ),
                Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(consts::NAME, DataType::Utf8, true),
                Field::new(consts::DESCRIPTION, DataType::Utf8, true),
                Field::new(consts::UNIT, DataType::Utf8, true),
                Field::new(consts::METRIC_TYPE, DataType::UInt8, true),
                Field::new(consts::AGGREGATION_TEMPORALITY, DataType::Int32, true),
                Field::new(consts::IS_MONOTONIC, DataType::Boolean, true),
            ])),
            vec![
                Arc::new(StructArray::new(
                    res_struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values(
                        std::iter::repeat_n(0, 6),
                    ))],
                    None,
                )),
                Arc::new(StructArray::new(
                    scope_struct_fields.clone(),
                    vec![
                        Arc::new(UInt16Array::from_iter_values(std::iter::repeat_n(0, 6))),
                        Arc::new(StringArray::from_iter_values(std::iter::repeat_n(
                            "scope0", 6,
                        ))),
                    ],
                    None,
                )),
                Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3, 4, 5])),
                Arc::new(StringArray::from_iter_values([
                    "metric0", "metric1", "metric2", "metric3", "metric4", "metric5",
                ])),
                Arc::new(StringArray::from(vec![
                    Some("desc0"),
                    Some("desc1"),
                    None,
                    None,
                    None,
                    None,
                ])),
                Arc::new(StringArray::from(vec![
                    Some("unit0"),
                    Some("unit1"),
                    None,
                    None,
                    None,
                    None,
                ])),
                Arc::new(UInt8Array::from(vec![
                    Some(MetricType::Gauge as u8),
                    Some(MetricType::Sum as u8),
                    Some(MetricType::Summary as u8),
                    Some(MetricType::Histogram as u8),
                    Some(MetricType::ExponentialHistogram as u8),
                    Some(MetricType::Empty as u8),
                ])),
                Arc::new(Int32Array::from(vec![
                    None,
                    Some(42),
                    None,
                    Some(52),
                    Some(53),
                    None,
                ])),
                Arc::new(BooleanArray::from(vec![
                    None,
                    Some(true),
                    None,
                    Some(true),
                    Some(false),
                    None,
                ])),
            ],
        )
        .unwrap();

        let attrs_16_schema = Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        ]));

        let attrs_32_schema = Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt32, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        ]));

        let exemplars_schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true).with_plain_encoding(),
            Field::new(consts::PARENT_ID, DataType::UInt32, false).with_plain_encoding(),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(consts::INT_VALUE, DataType::Int64, true),
            Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
        ]));

        let resource_attrs = RecordBatch::try_new(
            attrs_16_schema.clone(),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 0])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter_values(["rka", "rkb"])),
                Arc::new(StringArray::from_iter_values(["rva", "rvb"])),
            ],
        )
        .unwrap();

        let scope_attrs = RecordBatch::try_new(
            attrs_16_schema.clone(),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 0, 0])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter_values(["ska", "skb", "skc"])),
                Arc::new(StringArray::from_iter_values(["sva", "svb", "svc"])),
            ],
        )
        .unwrap();

        let numbers_data_points = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true).with_plain_encoding(),
                Field::new(consts::PARENT_ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(
                    consts::START_TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(
                    consts::TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(consts::INT_VALUE, DataType::Int64, true),
                Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
                Field::new(consts::FLAGS, DataType::UInt32, true),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values([0, 1, 2, 3])),
                Arc::new(UInt16Array::from_iter_values([0, 0, 1, 1])),
                Arc::new(TimestampNanosecondArray::from_iter_values([1i64, 2, 3, 4])),
                Arc::new(TimestampNanosecondArray::from_iter_values([5i64, 6, 7, 8])),
                Arc::new(Int64Array::from_iter([None, Some(2), None, None])),
                Arc::new(Float64Array::from_iter([Some(1.0), None, Some(3.0), None])),
                Arc::new(UInt32Array::from_iter_values([5, 4, 3, 2])),
            ],
        )
        .unwrap();

        let number_dp_attrs = RecordBatch::try_new(
            attrs_32_schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values([1])),
                Arc::new(UInt8Array::from_iter_values(
                    [AttributeValueType::Str as u8],
                )),
                Arc::new(StringArray::from_iter_values(["dpka"])),
                Arc::new(StringArray::from_iter_values(["terry"])),
            ],
        )
        .unwrap();

        let number_dp_exemplars = RecordBatch::try_new(
            exemplars_schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(UInt32Array::from_iter_values([1])),
                Arc::new(TimestampNanosecondArray::from_iter_values([101i64])),
                Arc::new(Int64Array::from_iter([None])),
                Arc::new(Float64Array::from_iter([None])),
                Arc::new(FixedSizeBinaryArray::try_from_iter(vec![[2u8; 16]].into_iter()).unwrap()),
                Arc::new(FixedSizeBinaryArray::try_from_iter(vec![[3; 8]].into_iter()).unwrap()),
            ],
        )
        .unwrap();

        let number_dp_exemplar_attrs = RecordBatch::try_new(
            attrs_32_schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(UInt8Array::from_iter_values(
                    [AttributeValueType::Str as u8],
                )),
                Arc::new(StringArray::from_iter_values(["ndpka"])),
                Arc::new(StringArray::from_iter_values(["val7"])),
            ],
        )
        .unwrap();

        let summary_quantile_struct_fields = Fields::from(vec![
            Field::new(consts::SUMMARY_QUANTILE, DataType::Float64, false),
            Field::new(consts::SUMMARY_VALUE, DataType::Float64, false),
        ]);

        let quantiles = Arc::new(Float64Array::from(vec![0.1, 0.2, 0.3])) as ArrayRef;
        let quantile_values = Arc::new(Float64Array::from(vec![1.1, 2.2, 3.3])) as ArrayRef;
        let quantiles_field = Arc::new(Field::new(
            "item",
            DataType::Struct(summary_quantile_struct_fields.clone()),
            false,
        )) as FieldRef;

        let summary_data_points = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true).with_plain_encoding(),
                Field::new(consts::PARENT_ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(
                    consts::START_TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(
                    consts::TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(consts::SUMMARY_COUNT, DataType::UInt64, true),
                Field::new(consts::SUMMARY_SUM, DataType::Float64, true),
                Field::new(
                    consts::SUMMARY_QUANTILE_VALUES,
                    DataType::List(quantiles_field.clone()),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values([0, 1])),
                Arc::new(UInt16Array::from_iter_values([2, 2])),
                Arc::new(TimestampNanosecondArray::from_iter_values([11i64, 12])),
                Arc::new(TimestampNanosecondArray::from_iter_values([15i64, 16])),
                Arc::new(UInt64Array::from_iter([None, Some(42)])),
                Arc::new(Float64Array::from_iter([None, Some(123.456)])),
                Arc::new(ListArray::new(
                    quantiles_field.clone(),
                    OffsetBuffer::new(Int32Array::from_iter_values([0, 2, 3]).values().clone()),
                    Arc::new(StructArray::new(
                        summary_quantile_struct_fields.clone(),
                        vec![quantiles, quantile_values],
                        None,
                    )),
                    None,
                )),
            ],
        )
        .unwrap();

        let summary_dp_attrs = RecordBatch::try_new(
            attrs_32_schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values([0, 0, 1])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter_values(["sdpka", "sdpkb", "sdpka"])),
                Arc::new(StringArray::from_iter_values(["val2", "val3", "val4"])),
            ],
        )
        .unwrap();

        let hist_bucket_counts_field =
            Arc::new(Field::new("item", DataType::UInt64, false)) as FieldRef;
        let hist_explicit_bounds_field =
            Arc::new(Field::new("item", DataType::Float64, false)) as FieldRef;

        let histogram_data_points = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true).with_plain_encoding(),
                Field::new(consts::PARENT_ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(
                    consts::START_TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(
                    consts::TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(consts::HISTOGRAM_COUNT, DataType::UInt64, true),
                Field::new(consts::HISTOGRAM_SUM, DataType::Float64, true),
                Field::new(
                    consts::HISTOGRAM_BUCKET_COUNTS,
                    DataType::List(hist_bucket_counts_field.clone()),
                    true,
                ),
                Field::new(
                    consts::HISTOGRAM_EXPLICIT_BOUNDS,
                    DataType::List(hist_explicit_bounds_field.clone()),
                    true,
                ),
                Field::new(consts::FLAGS, DataType::UInt32, true),
                Field::new(consts::HISTOGRAM_MIN, DataType::Float64, true),
                Field::new(consts::HISTOGRAM_MAX, DataType::Float64, true),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(UInt16Array::from_iter_values([3])),
                Arc::new(TimestampNanosecondArray::from_iter_values([21i64])),
                Arc::new(TimestampNanosecondArray::from_iter_values([25i64])),
                Arc::new(UInt64Array::from_iter([Some(10)])),
                Arc::new(Float64Array::from_iter([Some(55.5)])),
                Arc::new(ListArray::new(
                    hist_bucket_counts_field.clone(),
                    OffsetBuffer::new(Int32Array::from_iter_values([0, 3]).values().clone()),
                    Arc::new(UInt64Array::from_iter([Some(1), Some(2), Some(7)])),
                    None,
                )),
                Arc::new(ListArray::new(
                    hist_explicit_bounds_field.clone(),
                    OffsetBuffer::new(Int32Array::from_iter_values([0, 2]).values().clone()),
                    Arc::new(Float64Array::from_iter([0.1, 0.5])),
                    None,
                )),
                Arc::new(UInt32Array::from_iter([Some(99)])),
                Arc::new(Float64Array::from_iter([Some(0.1)])),
                Arc::new(Float64Array::from_iter([Some(10.0)])),
            ],
        )
        .unwrap();

        let hist_dp_attrs = RecordBatch::try_new(
            attrs_32_schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(UInt8Array::from_iter_values(
                    [AttributeValueType::Str as u8],
                )),
                Arc::new(StringArray::from_iter_values(["hdpka"])),
                Arc::new(StringArray::from_iter_values(["val3"])),
            ],
        )
        .unwrap();

        let exp_hist_pos_neg_struct_fields = Fields::from(vec![
            Field::new(consts::EXP_HISTOGRAM_OFFSET, DataType::Int32, true),
            Field::new(
                consts::EXP_HISTOGRAM_BUCKET_COUNTS,
                DataType::List(hist_bucket_counts_field.clone()),
                true,
            ),
        ]);

        let hist_dp_exemplars = RecordBatch::try_new(
            exemplars_schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values([0, 1])),
                Arc::new(UInt32Array::from_iter_values([0, 0])),
                Arc::new(TimestampNanosecondArray::from_iter_values([26i64, 27])),
                Arc::new(Int64Array::from_iter([Some(123), None])),
                Arc::new(Float64Array::from_iter([None, Some(456.789)])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![0u128.to_be_bytes().to_vec(), 1u128.to_be_bytes().to_vec()]
                            .into_iter(),
                    )
                    .unwrap(),
                ),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![2u64.to_be_bytes().to_vec(), 3u64.to_be_bytes().to_vec()].into_iter(),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        let hist_exemplar_attrs = RecordBatch::try_new(
            attrs_32_schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values([0, 1])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter_values(["hdpeka", "hdpekb"])),
                Arc::new(StringArray::from_iter_values(["val3", "val4"])),
            ],
        )
        .unwrap();

        let exp_hist_data_points = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true).with_plain_encoding(),
                Field::new(consts::PARENT_ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(
                    consts::START_TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(
                    consts::TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(consts::HISTOGRAM_COUNT, DataType::UInt64, true),
                Field::new(consts::HISTOGRAM_SUM, DataType::Float64, true),
                Field::new(consts::EXP_HISTOGRAM_SCALE, DataType::Int32, true),
                Field::new(consts::EXP_HISTOGRAM_ZERO_COUNT, DataType::UInt64, true),
                Field::new(
                    consts::EXP_HISTOGRAM_POSITIVE,
                    DataType::Struct(exp_hist_pos_neg_struct_fields.clone()),
                    true,
                ),
                Field::new(
                    consts::EXP_HISTOGRAM_NEGATIVE,
                    DataType::Struct(exp_hist_pos_neg_struct_fields.clone()),
                    true,
                ),
                Field::new(consts::FLAGS, DataType::UInt32, true),
                Field::new(consts::HISTOGRAM_MIN, DataType::Float64, true),
                Field::new(consts::HISTOGRAM_MAX, DataType::Float64, true),
                Field::new(
                    consts::EXP_HISTOGRAM_ZERO_THRESHOLD,
                    DataType::Float64,
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(UInt16Array::from_iter_values([4])),
                Arc::new(TimestampNanosecondArray::from_iter_values([31i64])),
                Arc::new(TimestampNanosecondArray::from_iter_values([35i64])),
                Arc::new(UInt64Array::from_iter([Some(20)])),
                Arc::new(Float64Array::from_iter([Some(155.5)])),
                Arc::new(Int32Array::from_iter([3])),
                Arc::new(UInt64Array::from_iter([Some(5)])),
                Arc::new(StructArray::new(
                    exp_hist_pos_neg_struct_fields.clone(),
                    vec![
                        Arc::new(Int32Array::from_iter([Some(1)])),
                        Arc::new(ListArray::new(
                            hist_bucket_counts_field.clone(),
                            OffsetBuffer::new(
                                Int32Array::from_iter_values([0, 3]).values().clone(),
                            ),
                            Arc::new(UInt64Array::from_iter([Some(2), Some(3), Some(5)])),
                            None,
                        )),
                    ],
                    None,
                )),
                Arc::new(StructArray::new(
                    exp_hist_pos_neg_struct_fields.clone(),
                    vec![
                        Arc::new(Int32Array::from_iter([Some(-1)])),
                        Arc::new(ListArray::new(
                            hist_bucket_counts_field.clone(),
                            OffsetBuffer::new(
                                Int32Array::from_iter_values([0, 2]).values().clone(),
                            ),
                            Arc::new(UInt64Array::from_iter([Some(1), Some(4)])),
                            None,
                        )),
                    ],
                    None,
                )),
                Arc::new(UInt32Array::from_iter([Some(88)])),
                Arc::new(Float64Array::from_iter([Some(0.01)])),
                Arc::new(Float64Array::from_iter([Some(100.0)])),
                Arc::new(Float64Array::from_iter([Some(1.1)])),
            ],
        )
        .unwrap();

        let exp_hist_dp_attrs = RecordBatch::try_new(
            attrs_32_schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(UInt8Array::from_iter_values(
                    [AttributeValueType::Str as u8],
                )),
                Arc::new(StringArray::from_iter_values(["ehdpka"])),
                Arc::new(StringArray::from_iter_values(["val3"])),
            ],
        )
        .unwrap();

        let exp_hist_dp_exemplars = RecordBatch::try_new(
            exemplars_schema,
            vec![
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(TimestampNanosecondArray::from_iter_values([1])),
                Arc::new(Int64Array::from_iter([1])),
                Arc::new(Float64Array::from_iter([1.0])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![7u128.to_le_bytes().to_vec()].into_iter(),
                    )
                    .unwrap(),
                ),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![7u64.to_le_bytes().to_vec()].into_iter(),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        let exp_hist_exemplar_attrs = RecordBatch::try_new(
            attrs_32_schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(UInt8Array::from_iter_values(
                    [AttributeValueType::Str as u8],
                )),
                Arc::new(StringArray::from_iter_values(["ehdpeka"])),
                Arc::new(StringArray::from_iter_values(["val3"])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Metrics(Metrics::default());
        otap_batch.set(ArrowPayloadType::UnivariateMetrics, metrics_record_batch);
        otap_batch.set(ArrowPayloadType::ResourceAttrs, resource_attrs);
        otap_batch.set(ArrowPayloadType::ScopeAttrs, scope_attrs);
        otap_batch.set(ArrowPayloadType::NumberDataPoints, numbers_data_points);
        otap_batch.set(ArrowPayloadType::NumberDpAttrs, number_dp_attrs);
        otap_batch.set(ArrowPayloadType::NumberDpExemplars, number_dp_exemplars);
        otap_batch.set(
            ArrowPayloadType::NumberDpExemplarAttrs,
            number_dp_exemplar_attrs,
        );
        otap_batch.set(ArrowPayloadType::SummaryDataPoints, summary_data_points);
        otap_batch.set(ArrowPayloadType::SummaryDpAttrs, summary_dp_attrs);
        otap_batch.set(ArrowPayloadType::HistogramDataPoints, histogram_data_points);
        otap_batch.set(ArrowPayloadType::HistogramDpAttrs, hist_dp_attrs);
        otap_batch.set(ArrowPayloadType::HistogramDpExemplars, hist_dp_exemplars);
        otap_batch.set(
            ArrowPayloadType::HistogramDpExemplarAttrs,
            hist_exemplar_attrs,
        );
        otap_batch.set(
            ArrowPayloadType::ExpHistogramDataPoints,
            exp_hist_data_points,
        );
        otap_batch.set(ArrowPayloadType::ExpHistogramDpAttrs, exp_hist_dp_attrs);
        otap_batch.set(
            ArrowPayloadType::ExpHistogramDpExemplars,
            exp_hist_dp_exemplars,
        );
        otap_batch.set(
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
            exp_hist_exemplar_attrs,
        );

        let mut encoder = MetricsProtoBytesEncoder::new();
        let mut result_buf = ProtoBuffer::new();
        encoder.encode(&mut otap_batch, &mut result_buf).unwrap();
        let result = MetricsData::decode(result_buf.as_ref()).unwrap();

        let expected = MetricsData {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue::new("rka", AnyValue::new_string("rva")),
                        KeyValue::new("rkb", AnyValue::new_string("rvb")),
                    ],
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "scope0".to_string(),
                        attributes: vec![
                            KeyValue::new("ska", AnyValue::new_string("sva")),
                            KeyValue::new("skb", AnyValue::new_string("svb")),
                            KeyValue::new("skc", AnyValue::new_string("svc")),
                        ],
                        ..Default::default()
                    }),
                    metrics: vec![
                        Metric {
                            name: "metric0".to_string(),
                            description: "desc0".to_string(),
                            unit: "unit0".to_string(),
                            data: Some(metric::Data::Gauge(Gauge {
                                data_points: vec![
                                    NumberDataPoint {
                                        start_time_unix_nano: 1,
                                        time_unix_nano: 5,
                                        attributes: vec![],
                                        exemplars: vec![],
                                        flags: 5,
                                        value: Some(Value::AsDouble(1.0)),
                                    },
                                    NumberDataPoint {
                                        start_time_unix_nano: 2,
                                        time_unix_nano: 6,
                                        attributes: vec![KeyValue::new(
                                            "dpka",
                                            AnyValue::new_string("terry"),
                                        )],
                                        exemplars: vec![Exemplar {
                                            time_unix_nano: 101,
                                            value: None,
                                            span_id: vec![3; 8],
                                            trace_id: vec![2; 16],
                                            filtered_attributes: vec![KeyValue::new(
                                                "ndpka",
                                                AnyValue::new_string("val7"),
                                            )],
                                        }],
                                        flags: 4,
                                        value: Some(Value::AsInt(2)),
                                    },
                                ],
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "metric1".to_string(),
                            description: "desc1".to_string(),
                            unit: "unit1".to_string(),
                            data: Some(metric::Data::Sum(Sum {
                                data_points: vec![
                                    NumberDataPoint {
                                        start_time_unix_nano: 3,
                                        time_unix_nano: 7,
                                        attributes: vec![],
                                        exemplars: vec![],
                                        flags: 3,
                                        value: Some(Value::AsDouble(3.0)),
                                    },
                                    NumberDataPoint {
                                        start_time_unix_nano: 4,
                                        time_unix_nano: 8,
                                        attributes: vec![],
                                        exemplars: vec![],
                                        flags: 2,
                                        value: None,
                                    },
                                ],
                                aggregation_temporality: 42,
                                is_monotonic: true,
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "metric2".to_string(),
                            data: Some(metric::Data::Summary(Summary {
                                data_points: vec![
                                    SummaryDataPoint {
                                        start_time_unix_nano: 11,
                                        time_unix_nano: 15,
                                        quantile_values: vec![
                                            ValueAtQuantile {
                                                quantile: 0.1,
                                                value: 1.1,
                                            },
                                            ValueAtQuantile {
                                                quantile: 0.2,
                                                value: 2.2,
                                            },
                                        ],
                                        attributes: vec![
                                            KeyValue::new("sdpka", AnyValue::new_string("val2")),
                                            KeyValue::new("sdpkb", AnyValue::new_string("val3")),
                                        ],
                                        ..Default::default()
                                    },
                                    SummaryDataPoint {
                                        start_time_unix_nano: 12,
                                        time_unix_nano: 16,
                                        count: 42,
                                        sum: 123.456,
                                        quantile_values: vec![ValueAtQuantile {
                                            quantile: 0.3,
                                            value: 3.3,
                                        }],
                                        attributes: vec![KeyValue::new(
                                            "sdpka",
                                            AnyValue::new_string("val4"),
                                        )],
                                        ..Default::default()
                                    },
                                ],
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "metric3".to_string(),
                            data: Some(metric::Data::Histogram(Histogram {
                                data_points: vec![HistogramDataPoint {
                                    start_time_unix_nano: 21,
                                    time_unix_nano: 25,
                                    attributes: vec![KeyValue::new(
                                        "hdpka",
                                        AnyValue::new_string("val3"),
                                    )],
                                    count: 10,
                                    sum: Some(55.5),
                                    bucket_counts: vec![1, 2, 7],
                                    explicit_bounds: vec![0.1, 0.5],
                                    flags: 99,
                                    min: Some(0.1),
                                    max: Some(10.0),
                                    exemplars: vec![
                                        Exemplar {
                                            time_unix_nano: 26,
                                            span_id: 2u64.to_be_bytes().to_vec(),
                                            trace_id: 0u128.to_be_bytes().to_vec(),
                                            value: Some(ExemplarValue::AsInt(123)),
                                            filtered_attributes: vec![KeyValue::new(
                                                "hdpeka",
                                                AnyValue::new_string("val3"),
                                            )],
                                        },
                                        Exemplar {
                                            time_unix_nano: 27,
                                            span_id: 3u64.to_be_bytes().to_vec(),
                                            trace_id: 1u128.to_be_bytes().to_vec(),
                                            value: Some(ExemplarValue::AsDouble(456.789)),
                                            filtered_attributes: vec![KeyValue::new(
                                                "hdpekb",
                                                AnyValue::new_string("val4"),
                                            )],
                                        },
                                    ],
                                }],
                                aggregation_temporality: 52,
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "metric4".to_string(),
                            data: Some(metric::Data::ExponentialHistogram(ExponentialHistogram {
                                data_points: vec![ExponentialHistogramDataPoint {
                                    start_time_unix_nano: 31,
                                    time_unix_nano: 35,
                                    count: 20,
                                    sum: Some(155.5),
                                    scale: 3,
                                    zero_count: 5,
                                    positive: Some(Buckets {
                                        offset: 1,
                                        bucket_counts: vec![2, 3, 5],
                                    }),
                                    negative: Some(Buckets {
                                        offset: -1,
                                        bucket_counts: vec![1, 4],
                                    }),
                                    flags: 88,
                                    min: Some(0.01),
                                    max: Some(100.0),
                                    zero_threshold: 1.1,
                                    attributes: vec![KeyValue::new(
                                        "ehdpka",
                                        AnyValue::new_string("val3"),
                                    )],
                                    exemplars: vec![Exemplar {
                                        time_unix_nano: 1,
                                        span_id: 7u64.to_le_bytes().to_vec(),
                                        trace_id: 7u128.to_le_bytes().to_vec(),
                                        value: Some(ExemplarValue::AsDouble(1.0)),
                                        filtered_attributes: vec![KeyValue::new(
                                            "ehdpeka",
                                            AnyValue::new_string("val3"),
                                        )],
                                    }],
                                }],
                                aggregation_temporality: 53,
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "metric5".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        assert_eq!(result, expected);
    }
}
