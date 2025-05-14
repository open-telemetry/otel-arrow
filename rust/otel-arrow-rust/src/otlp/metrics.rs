// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::arrays::{
    Int32ArrayAccessor, NullableArrayAccessor, StringArrayAccessor, get_bool_array_opt,
    get_u8_array, get_u16_array,
};
use crate::error;
use crate::otlp::common::{ResourceArrays, ScopeArrays};
use crate::otlp::metrics::related_data::RelatedData;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::metrics::v1::metric;
use crate::schema::consts;
use arrow::array::{BooleanArray, RecordBatch, UInt8Array, UInt16Array};
use num_enum::TryFromPrimitive;
use snafu::{OptionExt, ResultExt};

pub mod data_points;
pub mod exemplar;
pub mod related_data;

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
    type Error = error::Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self, Self::Error> {
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

/// Builds [ExportMetricsServiceRequest] from given record batch.
pub fn metrics_from(
    rb: &RecordBatch,
    related_data: &mut RelatedData,
) -> error::Result<ExportMetricsServiceRequest> {
    let mut metrics = ExportMetricsServiceRequest::default();

    let mut prev_res_id: Option<u16> = None;
    let mut prev_scope_id: Option<u16> = None;

    let mut res_id = 0;
    let mut scope_id = 0;

    let resource_arrays = ResourceArrays::try_from(rb)?;
    let scope_arrays = ScopeArrays::try_from(rb)?;
    let metrics_arrays = MetricsArrays::try_from(rb)?;

    for idx in 0..rb.num_rows() {
        let res_delta_id = resource_arrays.id.value_at(idx).unwrap_or_default();
        res_id += res_delta_id;

        if prev_res_id != Some(res_id) {
            // new resource id
            prev_res_id = Some(res_id);
            let res_metrics = metrics.resource_metrics.append_and_get();
            prev_scope_id = None;

            // Update the resource field of current resource metrics.
            let resource = res_metrics.resource.get_or_insert_default();
            if let Some(dropped_attributes_count) =
                resource_arrays.dropped_attributes_count.value_at(idx)
            {
                resource.dropped_attributes_count = dropped_attributes_count;
            }

            if let Some(res_id) = resource_arrays.id.value_at(idx)
                && let Some(attrs) = related_data
                    .res_attr_map_store
                    .attribute_by_delta_id(res_id)
            {
                resource.attributes = attrs.to_vec();
            }

            res_metrics.schema_url = resource_arrays.schema_url.value_at(idx).unwrap_or_default();
        }

        let scope_delta_id_opt = scope_arrays.id.value_at(idx);
        scope_id += scope_delta_id_opt.unwrap_or_default();

        if prev_scope_id != Some(scope_id) {
            prev_scope_id = Some(scope_id);
            // safety: We must have appended at least one resource metrics when reach here
            let current_scope_metrics_slice = &mut metrics
                .resource_metrics
                .last_mut()
                .expect("At this stage, we should have at least one resource metrics.")
                .scope_metrics;
            let scope_metrics = current_scope_metrics_slice.append_and_get();
            let mut scope = scope_arrays.create_instrumentation_scope(idx);
            if let Some(scope_id) = scope_delta_id_opt
                && let Some(attrs) = related_data
                    .scope_attr_map_store
                    .attribute_by_delta_id(scope_id)
            {
                scope.attributes = attrs.to_vec();
            }
            scope_metrics.scope = Some(scope);
            // ScopeMetrics uses the schema_url from metrics arrays.
            scope_metrics.schema_url = metrics_arrays.schema_url.value_at(idx).unwrap_or_default();
        }

        // Creates a metric at the end of current scope metrics slice.
        // safety: we've append at least one value at each slice when reach here.
        let current_scope_metrics = &mut metrics
            .resource_metrics
            .last_mut()
            .expect("At this stage, we should have at least one resource metrics.")
            .scope_metrics
            .last_mut()
            .expect("At this stage, we should ahve added at least one scope metrics.");
        let current_metric = current_scope_metrics.metrics.append_and_get();
        let delta_id = metrics_arrays.id.value_at_or_default(idx);
        let metric_id = related_data.metric_id_from_delta(delta_id);
        let metric_type_val = metrics_arrays.metric_type.value_at_or_default(idx);
        let metric_type =
            MetricType::try_from(metric_type_val).context(error::UnrecognizedMetricTypeSnafu {
                metric_type: metric_type_val,
            })?;

        let aggregation_temporality = metrics_arrays
            .aggregation_temporality
            .value_at_or_default(idx);
        let is_monotonic = metrics_arrays.is_monotonic.value_at_or_default(idx);
        current_metric.name = metrics_arrays.name.value_at(idx).unwrap_or_default();
        current_metric.description = metrics_arrays.description.value_at(idx).unwrap_or_default();
        current_metric.unit = metrics_arrays.unit.value_at_or_default(idx);

        match metric_type {
            MetricType::Gauge => {
                let dps = related_data
                    .number_data_points_store
                    .get_or_default(metric_id);
                current_metric.data = Some(metric::Data::Gauge(
                    crate::proto::opentelemetry::metrics::v1::Gauge {
                        data_points: std::mem::take(dps),
                    },
                ));
            }
            MetricType::Sum => {
                let dps = related_data
                    .number_data_points_store
                    .get_or_default(metric_id);
                let sum = crate::proto::opentelemetry::metrics::v1::Sum {
                    data_points: std::mem::take(dps),
                    aggregation_temporality,
                    is_monotonic,
                };
                current_metric.data = Some(metric::Data::Sum(sum));
            }
            MetricType::Histogram => {
                let dps = related_data
                    .histogram_data_points_store
                    .get_or_default(metric_id);
                let histogram = crate::proto::opentelemetry::metrics::v1::Histogram {
                    data_points: std::mem::take(dps),
                    aggregation_temporality,
                };
                current_metric.data = Some(metric::Data::Histogram(histogram));
            }
            MetricType::ExponentialHistogram => {
                let dps = related_data
                    .e_histogram_data_points_store
                    .get_or_default(metric_id);
                let e_histogram = crate::proto::opentelemetry::metrics::v1::ExponentialHistogram {
                    data_points: std::mem::take(dps),
                    aggregation_temporality,
                };
                current_metric.data = Some(metric::Data::ExponentialHistogram(e_histogram));
            }
            MetricType::Summary => {
                let dps = related_data
                    .summary_data_points_store
                    .get_or_default(metric_id);
                let summary = crate::proto::opentelemetry::metrics::v1::Summary {
                    data_points: std::mem::take(dps),
                };
                current_metric.data = Some(metric::Data::Summary(summary));
            }
            MetricType::Empty => return error::EmptyMetricTypeSnafu.fail(),
        }
    }

    Ok(metrics)
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
