// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, Float64Array, Int64Array, ListArray, RecordBatch, StringArray, StructArray,
    UInt8Array,
};
use arrow::compute::{cast, kernels::numeric::mul};
use arrow::datatypes::{DataType, Field, Schema};
use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::prelude::SessionContext;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otlp::metrics::MetricType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::planner::RecordType;
use crate::pipeline::state::ExecutionState;

pub(crate) struct ScaleMetricPipelineStage {
    multiplier: f64,
    unit: Option<String>,
}

impl ScaleMetricPipelineStage {
    pub(crate) fn new(multiplier: f64, unit: Option<String>) -> Self {
        Self { multiplier, unit }
    }

    fn execute_inner(&self, mut otap_batch: OtapArrowRecords) -> Result<OtapArrowRecords> {
        if !matches!(otap_batch, OtapArrowRecords::Metrics(_)) {
            return Err(Error::InvalidPipelineError {
                cause: "scale_metric can only be applied to metrics".into(),
                query_location: None,
            });
        }

        validate_multiplier(self.multiplier)?;

        let payload_type = otap_batch.root_payload_type();
        if let Some(root) = otap_batch.get(payload_type) {
            validate_metric_types(root)?;
        }

        if let Some(unit) = &self.unit
            && let Some(root) = otap_batch.get(payload_type)
        {
            otap_batch.set(payload_type, update_units(root, unit)?)?;
        }

        scale_columns(
            &mut otap_batch,
            ArrowPayloadType::NumberDataPoints,
            &[consts::INT_VALUE, consts::DOUBLE_VALUE],
            self.multiplier,
        )?;
        scale_columns(
            &mut otap_batch,
            ArrowPayloadType::NumberDpExemplars,
            &[consts::INT_VALUE, consts::DOUBLE_VALUE],
            self.multiplier,
        )?;
        scale_columns(
            &mut otap_batch,
            ArrowPayloadType::HistogramDataPoints,
            &[
                consts::HISTOGRAM_SUM,
                consts::HISTOGRAM_MIN,
                consts::HISTOGRAM_MAX,
            ],
            self.multiplier,
        )?;
        scale_list_values(
            &mut otap_batch,
            ArrowPayloadType::HistogramDataPoints,
            consts::HISTOGRAM_EXPLICIT_BOUNDS,
            self.multiplier,
        )?;
        scale_columns(
            &mut otap_batch,
            ArrowPayloadType::HistogramDpExemplars,
            &[consts::INT_VALUE, consts::DOUBLE_VALUE],
            self.multiplier,
        )?;
        scale_columns(
            &mut otap_batch,
            ArrowPayloadType::SummaryDataPoints,
            &[consts::SUMMARY_SUM],
            self.multiplier,
        )?;
        scale_summary_quantiles(&mut otap_batch, self.multiplier)?;
        Ok(otap_batch)
    }
}

fn validate_multiplier(multiplier: f64) -> Result<()> {
    if multiplier.is_finite() && multiplier > 0.0 {
        Ok(())
    } else {
        Err(Error::ExecutionError {
            cause: "scale_metric multiplier must be finite and greater than zero".into(),
        })
    }
}

fn validate_metric_types(record_batch: &RecordBatch) -> Result<()> {
    let metric_types = record_batch
        .column_by_name(consts::METRIC_TYPE)
        .and_then(|array| array.as_any().downcast_ref::<UInt8Array>())
        .ok_or_else(|| Error::ExecutionError {
            cause: "metrics batch is missing a UInt8 metric_type column".into(),
        })?;

    for metric_type in metric_types.iter() {
        let Some(metric_type) = metric_type else {
            return Err(Error::ExecutionError {
                cause: "scale_metric does not support a null metric type".into(),
            });
        };
        match MetricType::try_from(metric_type) {
            Ok(
                MetricType::Gauge | MetricType::Sum | MetricType::Histogram | MetricType::Summary,
            ) => {}
            Ok(MetricType::ExponentialHistogram) => {
                return Err(Error::ExecutionError {
                    cause: "exponential histograms are not supported by scale_metric".into(),
                });
            }
            Ok(MetricType::Empty) => {
                return Err(Error::ExecutionError {
                    cause: "scale_metric does not support empty metrics".into(),
                });
            }
            Err(_) => {
                return Err(Error::ExecutionError {
                    cause: format!("scale_metric does not support metric type {metric_type}"),
                });
            }
        }
    }

    Ok(())
}

#[async_trait(?Send)]
impl PipelineStage for ScaleMetricPipelineStage {
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        _session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        self.execute_inner(otap_batch)
    }

    fn supports_exec_on(&self, record_type: &RecordType) -> bool {
        matches!(record_type, RecordType::Signal)
    }
}

fn update_units(record_batch: &RecordBatch, unit: &str) -> Result<RecordBatch> {
    let units = StringArray::new_repeated(unit, record_batch.num_rows());

    if let Ok(index) = record_batch.schema().index_of(consts::UNIT) {
        let schema = record_batch.schema();
        let target_type = schema.field(index).data_type();
        replace_column(record_batch, index, cast(&units, target_type)?)
    } else {
        let mut fields = record_batch.schema().fields().to_vec();
        fields.push(Arc::new(Field::new(consts::UNIT, DataType::Utf8, true)));
        let mut columns = record_batch.columns().to_vec();
        columns.push(Arc::new(units));
        let schema = Schema::new(fields).with_metadata(record_batch.schema().metadata().clone());
        Ok(RecordBatch::try_new(Arc::new(schema), columns)?)
    }
}

fn scale_columns(
    otap_batch: &mut OtapArrowRecords,
    payload_type: ArrowPayloadType,
    column_names: &[&str],
    multiplier: f64,
) -> Result<()> {
    let Some(record_batch) = otap_batch.get(payload_type) else {
        return Ok(());
    };
    let mut result = record_batch.clone();

    for column_name in column_names {
        let Ok(index) = result.schema().index_of(column_name) else {
            continue;
        };
        let array = result.column(index);
        let scaled: ArrayRef = match array.data_type() {
            DataType::Int64 => {
                let values = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                    Error::ExecutionError {
                        cause: format!("{column_name} is not an Int64 array"),
                    }
                })?;
                Arc::new(Int64Array::from_iter(values.iter().map(|value| {
                    value.map(|value| (value as f64 * multiplier) as i64)
                })))
            }
            DataType::Float64 => mul(array, &Float64Array::new_scalar(multiplier))?,
            data_type => {
                return Err(Error::ExecutionError {
                    cause: format!(
                        "scale_metric does not support {data_type} column {column_name}"
                    ),
                });
            }
        };
        result = replace_column(&result, index, scaled)?;
    }

    otap_batch.set(payload_type, result)?;
    Ok(())
}

fn scale_list_values(
    otap_batch: &mut OtapArrowRecords,
    payload_type: ArrowPayloadType,
    column_name: &str,
    multiplier: f64,
) -> Result<()> {
    let Some(record_batch) = otap_batch.get(payload_type) else {
        return Ok(());
    };
    let Ok(index) = record_batch.schema().index_of(column_name) else {
        return Ok(());
    };
    let list = record_batch
        .column(index)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!("{column_name} is not a List array"),
        })?;
    let values = list
        .values()
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!("{column_name} values are not Float64"),
        })?;
    let scaled_values = mul(values, &Float64Array::new_scalar(multiplier))?;
    let DataType::List(field) = list.data_type() else {
        unreachable!("downcast ListArray has list data type");
    };
    let scaled = ListArray::new(
        Arc::clone(field),
        list.offsets().clone(),
        scaled_values,
        list.nulls().cloned(),
    );
    otap_batch.set(
        payload_type,
        replace_column(record_batch, index, Arc::new(scaled))?,
    )?;
    Ok(())
}

fn scale_summary_quantiles(otap_batch: &mut OtapArrowRecords, multiplier: f64) -> Result<()> {
    let payload_type = ArrowPayloadType::SummaryDataPoints;
    let Some(record_batch) = otap_batch.get(payload_type) else {
        return Ok(());
    };
    let Ok(index) = record_batch
        .schema()
        .index_of(consts::SUMMARY_QUANTILE_VALUES)
    else {
        return Ok(());
    };
    let list = record_batch
        .column(index)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| Error::ExecutionError {
            cause: "summary quantile column is not a List array".into(),
        })?;
    let values = list
        .values()
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| Error::ExecutionError {
            cause: "summary quantile values are not a Struct array".into(),
        })?;
    let value_index = values
        .fields()
        .iter()
        .position(|field| field.name() == consts::SUMMARY_VALUE)
        .ok_or_else(|| Error::ExecutionError {
            cause: "summary quantile struct is missing value".into(),
        })?;
    let value_array = values
        .column(value_index)
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| Error::ExecutionError {
            cause: "summary quantile value is not Float64".into(),
        })?;
    let scaled_values = mul(value_array, &Float64Array::new_scalar(multiplier))?;
    let mut struct_columns = values.columns().to_vec();
    struct_columns[value_index] = scaled_values;
    let scaled_struct = StructArray::new(
        values.fields().clone(),
        struct_columns,
        values.nulls().cloned(),
    );
    let DataType::List(field) = list.data_type() else {
        unreachable!("downcast ListArray has list data type");
    };
    let scaled_list = ListArray::new(
        Arc::clone(field),
        list.offsets().clone(),
        Arc::new(scaled_struct),
        list.nulls().cloned(),
    );
    otap_batch.set(
        payload_type,
        replace_column(record_batch, index, Arc::new(scaled_list))?,
    )?;
    Ok(())
}

fn replace_column(
    record_batch: &RecordBatch,
    index: usize,
    array: ArrayRef,
) -> Result<RecordBatch> {
    let mut columns = record_batch.columns().to_vec();
    columns[index] = array;
    Ok(RecordBatch::try_new(record_batch.schema(), columns)?)
}

#[cfg(test)]
mod tests {
    use super::validate_multiplier;

    /// Scenario: Validate finite positive scale multipliers.
    /// Guarantees: Positive finite multipliers are accepted by the execution stage.
    #[test]
    fn test_accept_positive_finite_multiplier() {
        for multiplier in [f64::MIN_POSITIVE, 1.0, f64::MAX] {
            assert!(validate_multiplier(multiplier).is_ok());
        }
    }

    /// Scenario: Validate non-positive and non-finite scale multipliers.
    /// Guarantees: Multipliers that cannot preserve metric semantics are rejected before mutation.
    #[test]
    fn test_reject_invalid_multiplier() {
        for multiplier in [
            f64::MIN,
            -1.0,
            -0.0,
            0.0,
            f64::NAN,
            f64::NEG_INFINITY,
            f64::INFINITY,
        ] {
            assert!(validate_multiplier(multiplier).is_err());
        }
    }
}
