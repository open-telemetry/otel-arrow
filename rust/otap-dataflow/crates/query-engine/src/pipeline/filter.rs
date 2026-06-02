// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::expr::{DataScope, ScopedExpr, ScopedValue, eval::resolve_attrs_payload_type};
use crate::pipeline::planner::AttributesIdentifier;
use crate::pipeline::state::ExecutionState;

use arrow::array::{
    Array, ArrayRef, BooleanArray, BooleanBufferBuilder, RecordBatch, UInt16Array, UInt32Array,
};
use arrow::buffer::BooleanBuffer;
use arrow::compute::{filter_record_batch, take};
use arrow::datatypes::UInt16Type;
use async_trait::async_trait;
use datafusion::common::cast::as_boolean_array;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::ColumnarValue;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::arrays::MaybeDictArrayAccessor;
use otap_df_pdata::otap::filter::{ChildBatchFilterIdHelper, IdBitmapPool, filter_otap_batch};

// TODO - need to wire this back into the expression evaluation
#[allow(dead_code)]
pub(crate) mod compare;

/// This stage evaluates a `ScopedExpr` tree to produce a root-aligned boolean selection
/// vector, then filters the OTAP batch using that vector.
pub struct FilterPipelineStage {
    predicate: ScopedExpr,
    id_bitmap_pool: IdBitmapPool,
}

impl FilterPipelineStage {
    pub fn new(scoped_op: ScopedExpr) -> Self {
        Self {
            predicate: scoped_op,
            id_bitmap_pool: IdBitmapPool::new(),
        }
    }
}

#[async_trait(?Send)]
impl PipelineStage for FilterPipelineStage {
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        let root_rb = match otap_batch.root_record_batch() {
            Some(rb) => rb,
            None => return Ok(otap_batch), // empty batch, nothing to filter
        };

        let num_rows = root_rb.num_rows();

        // Evaluate the ScopedExpr tree to produce a boolean result, then align to root.
        let result = self
            .predicate
            .execute_as_value(&otap_batch, session_context)?;

        // Convert the result to a root-aligned BooleanArray selection vector.
        let selection_vec = match result {
            None => {
                // expression data was absent — no rows pass the filter
                BooleanArray::new(BooleanBuffer::new_unset(num_rows), None)
            }
            Some(scoped_value) => {
                // if not root-scoped, align to root
                if scoped_value.scope != DataScope::Root
                    && !(matches!(scoped_value.scope, DataScope::RootParent(_)))
                    && scoped_value.scope != DataScope::StaticScalar
                {
                    align_selection_to_root(Some(scoped_value), &otap_batch)?
                } else {
                    // extract the BooleanArray from the ScopedValue
                    scoped_value_to_boolean_array(scoped_value.values, num_rows)?
                }
            }
        };

        let otap_batch = filter_otap_batch(&selection_vec, &otap_batch, &mut self.id_bitmap_pool)?;

        Ok(otap_batch)
    }

    async fn execute_on_attributes(
        &mut self,
        attrs_record_batch: RecordBatch,
        session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_options: &mut ExecutionState,
    ) -> Result<RecordBatch> {
        let result = self
            .predicate
            .evaluate_on_batch(session_context, &attrs_record_batch)?;

        let selection_vec = scoped_value_to_boolean_array(result, attrs_record_batch.num_rows())?;
        let new_batch = filter_record_batch(&attrs_record_batch, &selection_vec)?;

        Ok(new_batch)
    }

    fn supports_exec_on_attributes(&self) -> bool {
        true
    }
}

/// Convert a `ColumnarValue` into a `BooleanArray` selection vector of the given number of rows.
///
/// Handles:
/// - `ColumnarValue::Array` — casts to `BooleanArray`, strips null buffer by ANDing values with
///   the null buffer (null predicate results are treated as false)
/// - `ColumnarValue::Scalar(Boolean(true))` — all-true array
/// - `ColumnarValue::Scalar(Boolean(false))` or `Scalar(Null)` — all-false array
pub(crate) fn scoped_value_to_boolean_array(
    values: ColumnarValue,
    num_rows: usize,
) -> Result<BooleanArray> {
    match values {
        ColumnarValue::Scalar(scalar) => match scalar {
            ScalarValue::Boolean(Some(true)) => {
                Ok(BooleanArray::new(BooleanBuffer::new_set(num_rows), None))
            }
            ScalarValue::Boolean(Some(false)) | ScalarValue::Boolean(None) | ScalarValue::Null => {
                Ok(BooleanArray::new(BooleanBuffer::new_unset(num_rows), None))
            }
            other => Err(Error::ExecutionError {
                cause: format!(
                    "expected boolean scalar for filter selection, found {:?}",
                    other.data_type()
                ),
            }),
        },
        ColumnarValue::Array(arr) => {
            let boolean_arr = as_boolean_array(&arr).map_err(|_| Error::ExecutionError {
                cause: format!(
                    "expected boolean array for filter selection, found {}",
                    arr.data_type()
                ),
            })?;

            Ok(boolean_arr.clone())
        }
    }
}

/// Align a predicate evaluation result to the root scope and produce a `BooleanArray`
/// selection vector.
///
/// This is the standard way for filter and conditional consumers to convert a `ScopedValue`
/// (which may be in any scope) into a root-aligned boolean selection vector:
///
/// - If the result is already root-scoped or scalar, extracts the boolean directly
/// - If the result is child-scoped (attributes), aligns to root using the `align` function
///   which maps child parent_ids to root ids via `IdBitmap`
/// - If the result is `None` (missing data), returns an all-false selection vector
pub(crate) fn align_selection_to_root(
    result: Option<ScopedValue>,
    otap_batch: &OtapArrowRecords,
) -> Result<BooleanArray> {
    let num_rows = otap_batch
        .root_record_batch()
        .map(|rb| rb.num_rows())
        .unwrap_or(0);

    match result {
        None => Ok(BooleanArray::new(BooleanBuffer::new_unset(num_rows), None)),
        Some(scoped_value) => {
            let aligned = if scoped_value.scope != DataScope::Root
                && scoped_value.scope != DataScope::StaticScalar
            {
                // copy out the attrs_id before moving value, since AttributesIdentifier is Copy
                let maybe_attrs_id = match &scoped_value.scope {
                    DataScope::Attribute(attrs_id, _) | DataScope::AttributesAll(attrs_id) => {
                        Some(*attrs_id)
                    }
                    _ => None,
                };

                match maybe_attrs_id {
                    Some(attrs_id) => {
                        align_selection_vec_from_atts(scoped_value, &attrs_id, otap_batch)
                    }
                    _ => Err(Error::NotYetSupportedError {
                        message: format!(
                            "alignment from {:?} to root is not yet supported",
                            scoped_value.scope
                        ),
                    }),
                }?
            } else {
                scoped_value
            };
            scoped_value_to_boolean_array(aligned.values, num_rows)
        }
    }
}

/// Align a child-scoped (attribute) `ScopedValue` to the root scope.
///
/// Uses the parent_id column from the child result and the id column on the root batch
/// to map each child row to its corresponding root row. Root rows with no matching child
/// row get null values.
fn align_selection_vec_from_atts(
    value: ScopedValue,
    attrs_id: &AttributesIdentifier,
    otap_batch: &OtapArrowRecords,
) -> Result<ScopedValue> {
    let root_rb = otap_batch
        .root_record_batch()
        .ok_or_else(|| Error::ExecutionError {
            cause: "root batch not present for alignment".into(),
        })?;

    let num_rows = root_rb.num_rows();

    // get the parent_id column from the child result
    let parent_ids = value
        .parent_ids
        .as_ref()
        .ok_or_else(|| Error::ExecutionError {
            cause: "child-scoped result missing parent_id column for alignment".into(),
        })?;
    let parent_id_col = parent_ids
        .as_any()
        .downcast_ref::<UInt16Array>()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!(
                "expected parent_id to be UInt16, found {:?}",
                parent_ids.data_type()
            ),
        })?;

    // get the id column from the root batch for this attribute type
    let attrs_payload_type = resolve_attrs_payload_type(attrs_id, otap_batch);
    let id_col = match UInt16Type::get_id_col_from_parent(root_rb, attrs_payload_type)? {
        Some(MaybeDictArrayAccessor::Native(id_col)) => id_col,
        Some(_) => {
            return Err(Error::ExecutionError {
                cause: "invalid type for ID column on root batch".into(),
            });
        }
        None => {
            // no ID column means no attributes exist — return all-null for the root
            return Ok(ScopedValue::new(
                null_columnar_value_for_rows(&value.values, num_rows)?,
                DataScope::Root,
                root_rb,
            ));
        }
    };

    // materialize the child values into an array
    let child_values = value.values.into_array(parent_id_col.len())?;

    // For boolean arrays,
    let is_boolean = child_values.data_type() == &arrow::datatypes::DataType::Boolean;

    if is_boolean {
        // Build a boolean result using OR logic for duplicate parent_ids.
        //
        // Use OR semantics when multiple child rows map to the same parent. This handles the
        // case-insensitive key matching scenario where a single parent has multiple attribute
        // rows that match the key filter. If ANY matching row's predicate
        // is true, the parent should be true.

        let child_bools = child_values
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("checked data type above");

        // If no rows passed, short-circuit to all-false
        if child_bools.true_count() == 0 {
            let all_false = BooleanArray::new(BooleanBuffer::new_unset(num_rows), None);
            return Ok(ScopedValue::new(
                ColumnarValue::Array(Arc::new(all_false)),
                DataScope::Root,
                root_rb,
            ));
        }

        // Populate an IdBitmap with the parent_ids of rows that passed the predicate
        let mut pool = IdBitmapPool::new();
        let mut id_bitmap = pool.acquire();
        id_bitmap.populate(
            parent_id_col
                .iter()
                .enumerate()
                .filter_map(|(idx, pid_opt)| {
                    if child_bools.value(idx) {
                        pid_opt.map(|pid| pid as u32)
                    } else {
                        None
                    }
                }),
        );

        // Map the bitmap to a root-aligned BooleanArray
        let mut builder = BooleanBufferBuilder::new(id_col.len());
        let mut segment_val = false;
        let mut segment_len = 0usize;

        for idx in 0..id_col.len() {
            let row_val = if id_col.is_valid(idx) {
                id_bitmap.contains(id_col.value(idx) as u32)
            } else {
                false
            };

            if segment_val != row_val {
                if segment_len > 0 {
                    builder.append_n(segment_len, segment_val);
                }
                segment_val = row_val;
                segment_len = 0;
            }
            segment_len += 1;
        }
        if segment_len > 0 {
            builder.append_n(segment_len, segment_val);
        }

        pool.release(id_bitmap);

        let aligned_values: ArrayRef = Arc::new(BooleanArray::new(builder.finish(), None));

        return Ok(ScopedValue::new(
            ColumnarValue::Array(aligned_values),
            DataScope::Root,
            root_rb,
        ));
    }

    // Non-boolean case: use take-based alignment (assumes at most one child per parent)
    let mut pid_to_child_idx: Vec<Option<u32>> = vec![None; 65536];
    for i in 0..parent_id_col.len() {
        if parent_id_col.is_valid(i) {
            let pid = parent_id_col.value(i) as usize;
            pid_to_child_idx[pid] = Some(i as u32);
        }
    }

    // build take indices: for each root row, find the child index via the id column
    let take_indices: Vec<Option<u32>> = (0..id_col.len())
        .map(|idx| {
            if id_col.is_valid(idx) {
                let id = id_col.value(idx) as usize;
                pid_to_child_idx[id]
            } else {
                None
            }
        })
        .collect();

    let take_indices_arr = UInt32Array::from(take_indices);
    let aligned_values = take(&child_values, &take_indices_arr, None)?;

    Ok(ScopedValue::new(
        ColumnarValue::Array(aligned_values),
        DataScope::Root,
        root_rb,
    ))
}

/// Create a null `ColumnarValue` matching the type of the given value, with the specified
/// number of rows.
fn null_columnar_value_for_rows(values: &ColumnarValue, num_rows: usize) -> Result<ColumnarValue> {
    let data_type = match values {
        ColumnarValue::Scalar(scalar) => scalar.data_type(),
        ColumnarValue::Array(arr) => arr.data_type().clone(),
    };

    let null_scalar = ScalarValue::try_from(&data_type)?;
    let null_arr = null_scalar.to_array_of_size(num_rows)?;
    Ok(ColumnarValue::Array(null_arr))
}

#[cfg(test)]
mod test {
    use crate::pipeline::id_mask::IdMask;
    use crate::pipeline::{Pipeline, PipelineOptions};

    use super::*;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::schema::consts;

    /// Test helper to build an IdBitmap from a slice of u32 values.
    fn id_bitmap_from(ids: &[u32]) -> IdBitmap {
        let mut bm = IdBitmap::new();
        for &id in ids {
            bm.insert(id);
        }
        bm
    }
    use data_engine_kql_parser::{KqlParser, Parser};
    use otap_df_pdata::otap::filter::IdBitmap;
    use otap_df_pdata::otap::{Logs, Traces};
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otap_df_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
        HistogramDataPoint, Metric, NumberDataPoint, Summary, SummaryDataPoint,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::proto::opentelemetry::trace::v1::span::{Event, Link};
    use otap_df_pdata::proto::opentelemetry::trace::v1::{Span, Status};
    use otap_df_pdata::testing::round_trip::{
        otap_to_otlp, otlp_to_otap, to_logs_data, to_otap_logs, to_otap_metrics, to_otap_traces,
        to_traces_data,
    };
    use otap_df_query_engine_languages::opl::parser::OplParser;

    use crate::pipeline::test::{
        exec_logs_pipeline, otap_to_logs_data, otap_to_metrics_data, otap_to_traces_data,
    };

    async fn test_simple_filter<P: Parser, F: Fn(&str) -> String>(date_time_formatter: F) {
        let ns_per_second: u64 = 1000 * 1000 * 1000;
        let log_records = vec![
            LogRecord::build()
                .severity_text("TRACE")
                .severity_number(1)
                .time_unix_nano(ns_per_second)
                .event_name("1")
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .severity_number(9)
                .event_name("2")
                .time_unix_nano(2 * ns_per_second)
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .severity_number(17)
                .time_unix_nano(3 * ns_per_second)
                .event_name("3")
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            "logs | where severity_text == \"ERROR\"",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );

        // test same filter where the literal is on the left and column name on the right
        let result = exec_logs_pipeline::<P>(
            "logs | where \"ERROR\" == severity_text",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );

        // test filtering by some other field types (u32, int32, timestamp)
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_number == 17",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_number == 17",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );

        let result = exec_logs_pipeline::<P>(
            &format!(
                "logs | where time_unix_nano > {} ",
                date_time_formatter("1970-01-01T00:00:01.1")
            ),
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()]
        );

        let result = exec_logs_pipeline::<P>(
            &format!(
                "logs | where {} > time_unix_nano",
                date_time_formatter("1970-01-01T00:00:01.1")
            ),
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()]
        );

        let result =
            exec_logs_pipeline::<P>("logs | where true", to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &log_records
        );

        // assert everything filtered out:
        let result =
            exec_logs_pipeline::<P>("logs | where false", to_logs_data(log_records.clone())).await;
        assert_eq!(result.resource_logs.len(), 0);
    }

    #[tokio::test]
    async fn test_simple_filter_kql_parser() {
        test_simple_filter::<KqlParser, _>(|dt| format!("datetime({dt})")).await;
    }

    #[tokio::test]
    async fn test_simple_filter_opl_parser() {
        test_simple_filter::<OplParser, _>(|dt| format!("timestamp\"{dt}\"")).await
    }

    async fn test_simple_attrs_filter<P: Parser>() {
        let otap_batch = to_otap_logs(vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                .event_name("2")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("c"))])
                .event_name("3")
                .finish(),
        ]);

        let expected = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                .event_name("2")
                .finish(),
        ];

        let parser_result = P::parse("logs | where attributes[\"x\"] == \"b\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &expected,
        );

        // test same filter where the literal is on the left and the attribute is on the right
        let parser_result = P::parse("logs | where \"b\" == attributes[\"x\"]").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &expected,
        )
    }

    #[tokio::test]
    async fn test_simple_attrs_filter_kql_parser() {
        test_simple_attrs_filter::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_simple_attrs_filter_opl_parser() {
        test_simple_attrs_filter::<OplParser>().await;
    }

    #[tokio::test]
    async fn test_simple_filter_logs_body() {
        let input = vec![
            LogRecord::build()
                .body(AnyValue::new_string("hello"))
                .event_name("1")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_string("hello"))
                .event_name("2")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_string("world"))
                .event_name("3")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_int(418))
                .event_name("4")
                .finish(),
            LogRecord::build().event_name("5").finish(),
            LogRecord::build()
                .body(AnyValue::new_string("hello"))
                .event_name("6")
                .finish(),
        ];

        let query = "logs | where body == \"hello\"";
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone(), input[1].clone(), input[5].clone()]
        );

        // ensure same result when body column ref on the right
        let query = "logs | where \"hello\" == body";
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone(), input[1].clone(), input[5].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_logs_body_is_null() {
        let input = vec![
            LogRecord::build()
                .body(AnyValue::new_string("hello"))
                .event_name("1")
                .finish(),
            LogRecord::build().event_name("2").finish(),
            LogRecord::build()
                .body(AnyValue::new_string("hello"))
                .event_name("3")
                .finish(),
            LogRecord::build().event_name("4").finish(),
        ];

        let query = "logs | where body == null";
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[1].clone(), input[3].clone()]
        );

        // ensure same result when body column ref on the right
        let query = "logs | where null == body";
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[1].clone(), input[3].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_logs_body_using_matches() {
        let input = vec![
            LogRecord::build()
                .body(AnyValue::new_string("hello world"))
                .event_name("1")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_string("hello arrow"))
                .event_name("2")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_string("world"))
                .event_name("3")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_int(418))
                .event_name("4")
                .finish(),
            LogRecord::build().event_name("5").finish(),
            LogRecord::build()
                .body(AnyValue::new_string("hello"))
                .event_name("6")
                .finish(),
        ];

        let query = "logs | where matches(body, \"hello .*\")";
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone(), input[1].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_logs_body_using_contains() {
        let input = vec![
            LogRecord::build()
                .body(AnyValue::new_string("hello world"))
                .event_name("1")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_string("hello arrow"))
                .event_name("2")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_string("world"))
                .event_name("3")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_int(418))
                .event_name("4")
                .finish(),
            LogRecord::build().event_name("5").finish(),
            LogRecord::build()
                .body(AnyValue::new_string("hello"))
                .event_name("6")
                .finish(),
        ];

        let query = "logs | where contains(body, \"hello \")";
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone(), input[1].clone()]
        );

        // check it works w/ body on the right
        let query = "logs | where contains(\"hello world\", body)";
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone(), input[2].clone(), input[5].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_logs_by_body_using_expression() {
        let input = vec![
            LogRecord::build()
                .body(AnyValue::new_string("hello world"))
                .event_name("1")
                .finish(),
            LogRecord::build()
                .body(AnyValue::new_string("hello arrow"))
                .event_name("2")
                .finish(),
        ];

        let query = r#"logs | where replace(body, "hello", "bonjour") == "bonjour world""#;
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone()]
        );

        // check it works when the expressions on either side of predicate are flipped
        let query = r#"logs | where "bonjour world" == replace(body, "hello", "bonjour")"#;
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone()]
        );
    }

    async fn test_filter_text_contains<P: Parser>(
        q_event_name_contains_error: &str,
        q_1234_contains_event_name: &str,
        q_attrs_username_contains_y: &str,
        q_albert_contains_attrs_username: &str,
    ) {
        let log_records = vec![
            LogRecord::build()
                .event_name("error happen")
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("bert"),
                )])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("username", AnyValue::new_string("tim"))])
                .event_name("the error was caught")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("terry"),
                )])
                .event_name("3")
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            q_event_name_contains_error,
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );

        // check we could specify the column on the right
        let result = exec_logs_pipeline::<P>(
            q_1234_contains_event_name,
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );

        // also check we can filter by attributes using contains
        let result = exec_logs_pipeline::<P>(
            q_attrs_username_contains_y,
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        // check that we could also specify the column on the right for attributes
        let result = exec_logs_pipeline::<P>(
            q_albert_contains_attrs_username,
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_text_contains_kql() {
        test_filter_text_contains::<KqlParser>(
            r#"logs | where event_name contains "error""#,
            r#"logs | where "1234" contains event_name"#,
            r#"logs | where attributes["username"] contains "y""#,
            r#"logs | where "albert" contains attributes["username"]"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_text_contains_opl() {
        test_filter_text_contains::<OplParser>(
            r#"logs | where contains(event_name, "error")"#,
            r#"logs | where contains("1234", event_name)"#,
            r#"logs | where contains(attributes["username"], "y")"#,
            r#"logs | where contains("albert", attributes["username"])"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_contains_args_from_func_call_expr() {
        let log_records = vec![
            LogRecord::build()
                .event_name("error happen")
                .attributes(vec![
                    KeyValue::new("username", AnyValue::new_string("bort")),
                    KeyValue::new("email", AnyValue::new_string("foo@bar.co")),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("username", AnyValue::new_string("tim")),
                    KeyValue::new("email", AnyValue::new_string("hello@foo.com")),
                ])
                .event_name("the error was caught")
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("username", AnyValue::new_string("terry")),
                    KeyValue::new("email", AnyValue::new_string("mail@foo.com")),
                ])
                .event_name("3")
                .finish(),
        ];

        let result = exec_logs_pipeline::<OplParser>(
            "logs | where contains(concat(attributes[\"username\"], attributes[\"email\"]), \"e\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()]
        );

        // test w/ scalar on the left too
        let result = exec_logs_pipeline::<OplParser>(
            "logs | where contains(\"t\", substring(concat(attributes[\"username\"], attributes[\"email\"]), 0, 1))",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()]
        );
    }

    async fn test_filter_text_contains_struct_cols<P: Parser>(q1: &str, q2: &str) {
        let input = LogsData {
            resource_logs: vec![
                ResourceLogs {
                    schema_url: "version1".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("a"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                        ..Default::default()
                    }],
                },
                ResourceLogs {
                    schema_url: "experimental".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("b"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r2.e1").finish()],
                        ..Default::default()
                    }],
                },
            ],
        };

        let result = exec_logs_pipeline::<P>(q1, input.clone()).await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );

        // test same as above, but with literal contains the column value
        let result = exec_logs_pipeline::<P>(q2, input.clone()).await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[1].clone()],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_text_contains_struct_cols_kql() {
        test_filter_text_contains_struct_cols::<KqlParser>(
            r#"logs | where resource.schema_url contains "version""#,
            r#"logs | where "experimental version" contains resource.schema_url"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_text_contains_struct_cols_opl() {
        test_filter_text_contains_struct_cols::<OplParser>(
            r#"logs | where contains(resource.schema_url, "version")"#,
            r#"logs | where contains("experimental version", resource.schema_url)"#,
        )
        .await;
    }

    #[tokio::test]
    async fn filter_contains_where_haystack_is_expression() {
        let log_records = vec![
            LogRecord::build()
                .event_name("hello")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("hello"))])
                .finish(),
            LogRecord::build()
                .event_name("bonjour")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("bonjour"))])
                .finish(),
            LogRecord::build()
                .event_name("HI")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("HI"))])
                .finish(),
        ];
        let result = exec_logs_pipeline::<OplParser>(
            "logs | where contains(lower_case(attributes[\"x\"]), \"h\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );

        let result = exec_logs_pipeline::<OplParser>(
            "logs | where contains(lower_case(event_name), \"h\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_event_name_using_starts_with_opl() {
        let input = vec![
            LogRecord::build().event_name("hello world").finish(),
            LogRecord::build().event_name("hello arrow").finish(),
            LogRecord::build().event_name("world hello").finish(),
            LogRecord::build().finish(),
            LogRecord::build().event_name("hello").finish(),
        ];

        let query = r#"logs | where starts_with(event_name, "hello")"#;
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone(), input[1].clone(), input[4].clone()]
        );

        // column on the right
        let query = r#"logs | where starts_with("hello world", event_name)"#;
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone(), input[4].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_event_name_using_ends_with_opl() {
        let input = vec![
            LogRecord::build().event_name("hello world").finish(),
            LogRecord::build().event_name("goodbye world").finish(),
            LogRecord::build().event_name("hello arrow").finish(),
            LogRecord::build().finish(),
            LogRecord::build().event_name("world").finish(),
        ];

        let query = r#"logs | where ends_with(event_name, "world")"#;
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone(), input[1].clone(), input[4].clone()]
        );

        // column on the right
        let query = r#"logs | where ends_with("hello world", event_name)"#;
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(input.clone())).await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[input[0].clone(), input[4].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_attrs_using_starts_with_opl() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("albert"),
                )])
                .event_name("1")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("alice"),
                )])
                .event_name("2")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("username", AnyValue::new_string("bob"))])
                .event_name("3")
                .finish(),
        ];

        let query = r#"logs | where starts_with(attributes["username"], "al")"#;
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_attrs_using_ends_with_opl() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "filename",
                    AnyValue::new_string("report.pdf"),
                )])
                .event_name("1")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "filename",
                    AnyValue::new_string("notes.pdf"),
                )])
                .event_name("2")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "filename",
                    AnyValue::new_string("README.md"),
                )])
                .event_name("3")
                .finish(),
        ];

        let query = r#"logs | where ends_with(attributes["filename"], ".pdf")"#;
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    async fn test_filter_matches_regex<P: Parser>(q1: &str, q2: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("error happen")
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("bert"),
                )])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("username", AnyValue::new_string("tim"))])
                .event_name("the error was caught")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("terry"),
                )])
                .event_name("3")
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(q1, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()]
        );

        // also check we can filter by attributes using matches/regex
        let result = exec_logs_pipeline::<P>(q2, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_matches_regex_kql() {
        test_filter_matches_regex::<KqlParser>(
            r#"logs | where event_name matches regex "^err.*""#,
            r#"logs | where attributes["username"] matches regex "^t.*""#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_matches_regex_opl() {
        test_filter_matches_regex::<OplParser>(
            r#"logs | where matches(event_name, "^err.*")"#,
            r#"logs | where matches(attributes["username"], "^t.*")"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_matches_regex_opl_regex_literal() {
        test_filter_matches_regex::<OplParser>(
            r#"logs | where matches(event_name, r"^err.*")"#,
            r#"logs | where matches(attributes["username"], r"^t.*")"#,
        )
        .await;
    }

    #[tokio::test]
    async fn filter_matches_where_haystack_is_expression() {
        let log_records = vec![
            LogRecord::build()
                .event_name("hello")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("hello"))])
                .finish(),
            LogRecord::build()
                .event_name("bonjour")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("bonjour"))])
                .finish(),
            LogRecord::build()
                .event_name("HI")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("HI"))])
                .finish(),
        ];
        let result = exec_logs_pipeline::<OplParser>(
            "logs | where matches(lower_case(attributes[\"x\"]), \"h.*\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );

        let result = exec_logs_pipeline::<OplParser>(
            "logs | where matches(lower_case(event_name), \"h.*\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    async fn test_filter_text_matches_regex_struct_cols<P: Parser>(q1: &str) {
        let input = LogsData {
            resource_logs: vec![
                ResourceLogs {
                    schema_url: "version1".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("a"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                        ..Default::default()
                    }],
                },
                ResourceLogs {
                    schema_url: "experimental".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("b"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r2.e1").finish()],
                        ..Default::default()
                    }],
                },
            ],
        };

        let result = exec_logs_pipeline::<P>(q1, input.clone()).await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_text_matches_regex_struct_cols_kql() {
        test_filter_text_matches_regex_struct_cols::<KqlParser>(
            r#"logs | where resource.schema_url matches regex "v.*1""#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_text_matches_regex_struct_cols_opl() {
        test_filter_text_matches_regex_struct_cols::<OplParser>(
            r#"logs | where matches(resource.schema_url, "v.*1")"#,
        )
        .await;
    }

    async fn test_filter_by_resources<P: Parser>() {
        let input = LogsData {
            resource_logs: vec![
                ResourceLogs {
                    schema_url: "schema1".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("a"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                        ..Default::default()
                    }],
                },
                ResourceLogs {
                    schema_url: "schema2".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("b"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r2.e1").finish()],
                        ..Default::default()
                    }],
                },
            ],
        };

        // test filter by resource properties
        let result = exec_logs_pipeline::<P>(
            "logs | where resource.schema_url == \"schema1\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );

        // test same as above, but with the literal on the right
        let result = exec_logs_pipeline::<P>(
            "logs | where \"schema2\" == resource.schema_url",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[1].clone()],
            }
        );

        // test filter by resource attributes
        let result = exec_logs_pipeline::<P>(
            "logs | where resource.attributes[\"x\"] == \"a\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_by_resources_kql_parser() {
        test_filter_by_resources::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_by_resources_opl_parser() {
        test_filter_by_resources::<OplParser>().await;
    }

    async fn test_simple_filter_traces<P: Parser>() {
        let spans = vec![
            Span::build()
                .name("span1")
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                .events(vec![
                    Event::build()
                        .name("event1.1")
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .finish(),
                ])
                .links(vec![
                    Link::build()
                        .trace_id(vec![11; 16])
                        .span_id(vec![11; 8])
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .finish(),
                ])
                .finish(),
            Span::build()
                .name("span2")
                .trace_id(vec![2; 16])
                .span_id(vec![2; 8])
                .status(Status::default())
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                .events(vec![
                    Event::build()
                        .name("event2.1")
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Event::build()
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .name("event2.2")
                        .finish(),
                ])
                .links(vec![
                    Link::build()
                        .trace_id(vec![21; 16])
                        .span_id(vec![21; 8])
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Link::build()
                        .trace_id(vec![22; 16])
                        .span_id(vec![22; 8])
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .finish(),
                ])
                .finish(),
            Span::build()
                .name("span3")
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val3"))])
                .events(vec![
                    Event::build()
                        .name("event3.1")
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Event::build().name("event3.2").finish(),
                    Event::build().name("event3.2").finish(),
                ])
                .links(vec![
                    Link::build()
                        .trace_id(vec![31; 16])
                        .span_id(vec![31; 8])
                        .finish(),
                    Link::build()
                        .trace_id(vec![32; 16])
                        .span_id(vec![32; 8])
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Link::build()
                        .trace_id(vec![33; 16])
                        .span_id(vec![33; 8])
                        .finish(),
                ])
                .finish(),
        ];

        let input = to_otap_traces(spans.clone());
        let parser_result = P::parse("traces | where name == \"span2\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        // assert everything got filtered to the right size
        let result_spans = result.get(ArrowPayloadType::Spans).unwrap();
        assert_eq!(result_spans.num_rows(), 1);

        let span_attrs = result.get(ArrowPayloadType::SpanAttrs).unwrap();
        assert_eq!(span_attrs.num_rows(), 1);

        let span_events = result.get(ArrowPayloadType::SpanEvents).unwrap();
        assert_eq!(span_events.num_rows(), 2);

        let span_links = result.get(ArrowPayloadType::SpanLinks).unwrap();
        assert_eq!(span_links.num_rows(), 2);

        let span_link_attrs = result.get(ArrowPayloadType::SpanLinkAttrs).unwrap();
        assert_eq!(span_link_attrs.num_rows(), 3);

        let span_event_attrs = result.get(ArrowPayloadType::SpanEventAttrs).unwrap();
        assert_eq!(span_event_attrs.num_rows(), 3);

        let traces_data = otap_to_traces_data(result);
        assert_eq!(traces_data.resource_spans.len(), 1);
        assert_eq!(traces_data.resource_spans[0].scope_spans.len(), 1);
        pretty_assertions::assert_eq!(
            &traces_data.resource_spans[0].scope_spans[0].spans,
            &[spans[1].clone()]
        )
    }

    #[tokio::test]
    async fn test_simple_filter_traces_kql_parser() {
        test_simple_filter_traces::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_simple_filter_traces_opl_parser() {
        test_simple_filter_traces::<OplParser>().await;
    }

    async fn test_filter_traces_by_attrs<P: Parser>() {
        let spans = vec![
            Span::build()
                .name("span1")
                .trace_id(vec![1; 16])
                .span_id(vec![1; 8])
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                .status(Status::default())
                .finish(),
            Span::build()
                .name("span2")
                .trace_id(vec![2; 16])
                .span_id(vec![2; 8])
                .status(Status::default())
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let input = to_otap_traces(spans.clone());
        let parser_result = P::parse("traces | where attributes[\"key\"] == \"val2\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        let traces_data = otap_to_traces_data(result);
        assert_eq!(traces_data.resource_spans.len(), 1);
        assert_eq!(traces_data.resource_spans[0].scope_spans.len(), 1);
        pretty_assertions::assert_eq!(
            &traces_data.resource_spans[0].scope_spans[0].spans,
            &[spans[1].clone()]
        )
    }

    #[tokio::test]
    async fn test_filter_traces_by_attrs_kql_parser() {
        test_filter_traces_by_attrs::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_traces_by_attrs_opl_parser() {
        test_filter_traces_by_attrs::<OplParser>().await;
    }

    async fn test_simple_filter_metrics<P: Parser>() {
        let metrics = vec![
            Metric::build()
                .name("metric1")
                .data_gauge(Gauge {
                    data_points: vec![
                        NumberDataPoint::build()
                            .time_unix_nano(1000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(100u64)
                                    .trace_id(vec![1; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_gauge(Gauge {
                    data_points: vec![
                        NumberDataPoint::build()
                            .time_unix_nano(2000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(200u64)
                                    .trace_id(vec![2; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric1")
                .data_histogram(Histogram {
                    data_points: vec![
                        HistogramDataPoint::build()
                            .time_unix_nano(3000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(300u64)
                                    .trace_id(vec![1; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                    aggregation_temporality: 0,
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_histogram(Histogram {
                    data_points: vec![
                        HistogramDataPoint::build()
                            .time_unix_nano(4000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(400u64)
                                    .trace_id(vec![2; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                    aggregation_temporality: 0,
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric1")
                .data_exponential_histogram(ExponentialHistogram {
                    data_points: vec![
                        ExponentialHistogramDataPoint::build()
                            .time_unix_nano(5000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(500u64)
                                    .trace_id(vec![1; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val1"),
                                    )])
                                    .finish(),
                            ])
                            .positive(Buckets::default())
                            .negative(Buckets::default())
                            .finish(),
                    ],
                    aggregation_temporality: 0,
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_exponential_histogram(ExponentialHistogram {
                    data_points: vec![
                        ExponentialHistogramDataPoint::build()
                            .time_unix_nano(6000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(600u64)
                                    .trace_id(vec![2; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                    aggregation_temporality: 0,
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric1")
                .data_summary(Summary {
                    data_points: vec![
                        SummaryDataPoint::build()
                            .time_unix_nano(7000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .finish(),
                    ],
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_summary(Summary {
                    data_points: vec![
                        SummaryDataPoint::build()
                            .time_unix_nano(8000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .finish(),
                    ],
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
        ];

        let input = to_otap_metrics(metrics.clone());
        let parser_result = P::parse("metrics | where name == \"metric1\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        // assert everything got filtered to the right size
        let result_metrics = result.get(ArrowPayloadType::UnivariateMetrics).unwrap();
        assert_eq!(result_metrics.num_rows(), 4);

        let attrs = result.get(ArrowPayloadType::MetricAttrs).unwrap();
        assert_eq!(attrs.num_rows(), 4);

        let number_dps = result.get(ArrowPayloadType::NumberDataPoints).unwrap();
        assert_eq!(number_dps.num_rows(), 1);

        let number_dp_attrs = result.get(ArrowPayloadType::NumberDpAttrs).unwrap();
        assert_eq!(number_dp_attrs.num_rows(), 1);

        let number_dp_exemplars = result.get(ArrowPayloadType::NumberDpExemplars).unwrap();
        assert_eq!(number_dp_exemplars.num_rows(), 1);

        let number_dp_exemplar_attrs = result.get(ArrowPayloadType::NumberDpExemplarAttrs).unwrap();
        assert_eq!(number_dp_exemplar_attrs.num_rows(), 1);

        let hist_dps = result.get(ArrowPayloadType::HistogramDataPoints).unwrap();
        assert_eq!(hist_dps.num_rows(), 1);

        let hist_dp_attrs = result.get(ArrowPayloadType::HistogramDpAttrs).unwrap();
        assert_eq!(hist_dp_attrs.num_rows(), 1);

        let hist_dp_exemplars = result.get(ArrowPayloadType::HistogramDpExemplars).unwrap();
        assert_eq!(hist_dp_exemplars.num_rows(), 1);

        let hist_dp_exemplar_attrs = result
            .get(ArrowPayloadType::HistogramDpExemplarAttrs)
            .unwrap();
        assert_eq!(hist_dp_exemplar_attrs.num_rows(), 1);

        let exp_hist_dps = result
            .get(ArrowPayloadType::ExpHistogramDataPoints)
            .unwrap();
        assert_eq!(exp_hist_dps.num_rows(), 1);

        let exp_hist_dp_attrs = result.get(ArrowPayloadType::ExpHistogramDpAttrs).unwrap();
        assert_eq!(exp_hist_dp_attrs.num_rows(), 1);

        let exp_hist_dp_exemplars = result
            .get(ArrowPayloadType::ExpHistogramDpExemplars)
            .unwrap();
        assert_eq!(exp_hist_dp_exemplars.num_rows(), 1);

        let exp_hist_dp_exemplar_attrs = result
            .get(ArrowPayloadType::ExpHistogramDpExemplarAttrs)
            .unwrap();
        assert_eq!(exp_hist_dp_exemplar_attrs.num_rows(), 1);

        let summary_dps = result.get(ArrowPayloadType::SummaryDataPoints).unwrap();
        assert_eq!(summary_dps.num_rows(), 1);

        let summary_dp_attrs = result.get(ArrowPayloadType::SummaryDpAttrs).unwrap();
        assert_eq!(summary_dp_attrs.num_rows(), 1);

        let metrics_data = otap_to_metrics_data(result);
        assert_eq!(metrics_data.resource_metrics.len(), 1);
        assert_eq!(metrics_data.resource_metrics[0].scope_metrics.len(), 1);
        pretty_assertions::assert_eq!(
            &metrics_data.resource_metrics[0].scope_metrics[0].metrics,
            &[
                metrics[0].clone(),
                metrics[2].clone(),
                metrics[4].clone(),
                metrics[6].clone()
            ]
        )
    }

    #[tokio::test]
    async fn test_simple_filter_metrics_kql_parser() {
        test_simple_filter_metrics::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_simple_filter_metrics_opl_parser() {
        test_simple_filter_metrics::<OplParser>().await;
    }

    async fn test_filter_metrics_by_attrs<P: Parser>() {
        let metrics = vec![
            Metric::build()
                .name("metric1")
                .data_gauge(Gauge {
                    data_points: Vec::default(),
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_gauge(Gauge {
                    data_points: Vec::default(),
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let input = to_otap_metrics(metrics.clone());
        let parser_result = P::parse("metrics | where attributes[\"key\"] == \"val1\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        let metrics_data = otap_to_metrics_data(result);
        assert_eq!(metrics_data.resource_metrics.len(), 1);
        assert_eq!(metrics_data.resource_metrics[0].scope_metrics.len(), 1);
        pretty_assertions::assert_eq!(
            &metrics_data.resource_metrics[0].scope_metrics[0].metrics,
            &[metrics[0].clone(),]
        )
    }

    #[tokio::test]
    async fn test_filter_metrics_by_attrs_kql_parser() {
        test_filter_metrics_by_attrs::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_metrics_by_attrs_opl_parser() {
        test_filter_metrics_by_attrs::<OplParser>().await;
    }

    async fn test_removes_child_record_batch_if_parent_fully_filtered_out<P: Parser>() {
        let spans = vec![
            Span::build()
                .name("span1")
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                .finish(),
            Span::build()
                .name("span2")
                .trace_id(vec![2; 16])
                .span_id(vec![2; 8])
                .status(Status::default())
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                .events(vec![
                    Event::build()
                        .name("event2.1")
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Event::build()
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .name("event2.2")
                        .finish(),
                ])
                .finish(),
        ];

        let input = to_otap_traces(spans.clone());
        let parser_result = P::parse("traces | where name == \"span1\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        // since we've filtered for span1, which has no events, the event and event attrs batches
        // should no longer be present
        assert!(result.get(ArrowPayloadType::SpanEvents).is_none());
        assert!(result.get(ArrowPayloadType::SpanEventAttrs).is_none())
    }

    #[tokio::test]
    async fn test_removes_child_record_batch_if_parent_fully_filtered_out_kql_parser() {
        test_removes_child_record_batch_if_parent_fully_filtered_out::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_removes_child_record_batch_if_parent_fully_filtered_out_opl_parser() {
        test_removes_child_record_batch_if_parent_fully_filtered_out::<OplParser>().await;
    }

    async fn test_filter_by_scope<P: Parser>() {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name2")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        // test filter by resource properties
        let result = exec_logs_pipeline::<P>(
            "logs | where instrumentation_scope.name == \"name1\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );

        // test same as above, but with the literal on the right
        let result = exec_logs_pipeline::<P>(
            "logs | where \"name2\" == instrumentation_scope.name",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[1].clone()],
                    ..Default::default()
                }],
            }
        );

        // test filter by resource attributes
        let result = exec_logs_pipeline::<P>(
            "logs | where instrumentation_scope.attributes[\"x\"] == \"a\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_by_scope_kql_parser() {
        test_filter_by_scope::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_by_scope_opl_parser() {
        test_filter_by_scope::<OplParser>().await;
    }

    async fn test_filter_with_and<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];
        let otap_batch = to_otap_logs(log_records.clone());

        // check simple filter "and" properties
        let parser_result =
            P::parse("logs | where severity_text == \"ERROR\" and event_name == \"2\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );

        // check simple filter "and" with mixed attributes and properties
        let parser_result =
            P::parse("logs | where severity_text == \"ERROR\" and attributes[\"x\"] == \"c\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        // check simple filter "and" two attributes
        let parser_result =
            P::parse("logs | where attributes[\"y\"] == \"d\" and attributes[\"x\"] == \"a\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_and_kql_parser() {
        test_filter_with_and::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_with_and_opl_parser() {
        test_filter_with_and::<OplParser>().await;
    }

    async fn test_filter_with_or<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];
        let otap_batch = to_otap_logs(log_records.clone());

        // check simple filter "or" with properties predicates
        let parser_result =
            P::parse("logs | where severity_text == \"INFO\" or severity_text == \"ERROR\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );

        // check simple filter "or" with mixed attributes/properties predicates
        let parser_result =
            P::parse("logs | where severity_text == \"ERROR\" or attributes[\"x\"] == \"c\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check simple filter "or" two attributes predicates
        let parser_result =
            P::parse("logs | where attributes[\"x\"] == \"a\" or attributes[\"y\"] == \"e\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_or_kql_parser() {
        test_filter_with_or::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_with_or_opl_parser() {
        test_filter_with_or::<OplParser>().await;
    }

    async fn test_filter_with_not<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // check simple filter "not" with properties predicate
        let result = exec_logs_pipeline::<P>(
            "logs | where not(severity_text == \"INFO\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check simple filter "not" with attributes predicate
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"b\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_not_kql_parser() {
        test_filter_with_not::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_with_not_opl_parser() {
        test_filter_with_not::<OplParser>().await;
    }

    async fn test_filter_not_and<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // check simple inverted "and" filter with properties predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(severity_text == \"INFO\" and event_name == \"1\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check simple inverted "and" filter with attributes predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"b\" and attributes[\"y\"] == \"e\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );

        // check simple inverted "and" filter with mixed attributes & properties predicates
        // check simple inverted "and" filter with attributes predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"c\" and severity_text == \"DEBUG\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_not_and_kql_parser() {
        test_filter_not_and::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_not_and_opl_parser() {
        test_filter_not_and::<OplParser>().await;
    }

    async fn test_filter_not_or<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // check simple inverted "or" filter with properties predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(severity_text == \"INFO\" or event_name == \"2\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        // check simple inverted "or" filter with attributes predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"b\" or attributes[\"y\"] == \"f\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );

        // check simple inverted "or" filter with mixed attributes & properties predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"c\" or severity_text == \"INFO\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_not_or_kql_parser() {
        test_filter_not_or::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_not_or_opl_parser() {
        test_filter_not_or::<OplParser>().await;
    }

    async fn test_filter_with_nulls<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .finish(),
            LogRecord::build()
                .event_name("3")
                // severity_text == null
                .finish(),
        ];

        // check simple filter to ensure we filter out the value with null in the column
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_text == \"ERROR\"",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );

        // test a few scenarios where if we had null in the selection vector (which we
        // shouldn't have), they would not pass:
        let result = exec_logs_pipeline::<P>(
            "logs | where not(severity_text == \"ERROR\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_text == \"ERROR\" or event_name == \"3\"",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_nulls_kql_parser() {
        test_filter_with_nulls::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_with_nulls_opl_parser() {
        test_filter_with_nulls::<OplParser>().await;
    }

    async fn run_filter_numeric_comparison_binary_operators_test<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("z", AnyValue::new_int(1)),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("z", AnyValue::new_int(2)),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("z", AnyValue::new_int(3)),
                ])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            "logs | where attributes[\"z\"] > 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        let result = exec_logs_pipeline::<P>(
            "logs | where attributes[\"z\"] >= 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        let result = exec_logs_pipeline::<P>(
            "logs | where attributes[\"z\"] < 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );

        let result = exec_logs_pipeline::<P>(
            "logs | where attributes[\"z\"] <= 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_numeric_comparison_binary_operators_kql_parser() {
        run_filter_numeric_comparison_binary_operators_test::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_numeric_comparison_binary_operators_opl_parser() {
        run_filter_numeric_comparison_binary_operators_test::<OplParser>().await;
    }

    async fn test_filter_nomatch<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        let parser_result = P::parse("logs | where event_name == \"5\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        // assert it's equal to empty batch because there were no matches
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // assert we have the correct behaviour when filtering by attributes as well
        let parser_result = KqlParser::parse("logs | where attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()))
    }

    #[tokio::test]
    async fn test_filter_nomatch_kql_parser() {
        test_filter_nomatch::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_nomatch_opl_parser() {
        test_filter_nomatch::<OplParser>().await;
    }

    async fn test_empty_batch<P: Parser>() {
        let input = OtapArrowRecords::Logs(Logs::default());
        let parser_result = P::parse("logs | where event_name == \"5\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input.clone()).await.unwrap();
        assert_eq!(result, input);
    }

    #[tokio::test]
    async fn test_empty_batch_kql_parser() {
        test_empty_batch::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_empty_batch_opl_parser() {
        test_empty_batch::<OplParser>().await;
    }

    async fn test_filter_no_attrs<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("INFO")
                .finish(),
        ];

        // check that if there are no attributes to filter by then, we get the empty batch
        let parser_result = P::parse("logs | where attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // check that the same result happens when filtering by resource and scope attrs
        let parser_result =
            P::parse("logs | where resource.attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // check that the same result happens when filtering by resource and scope attrs
        let parser_result =
            P::parse("logs | where instrumentation_scope.attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // check that inverting the filters above basically just return the original record batch
        for inverted_attrs_filter in [
            "logs | where not(attributes[\"a\"] == \"1234\")",
            "logs | where not(resource.attributes[\"a\"] == \"1234\")",
            "logs | where not(instrumentation_scope.attributes[\"a\"] == \"1234\")",
        ] {
            let parser_result = P::parse(inverted_attrs_filter).unwrap();
            let mut pipeline = Pipeline::new(parser_result.pipeline);
            let input = to_otap_logs(log_records.clone());
            let result = pipeline.execute(input.clone()).await.unwrap();
            assert_eq!(result, input);
        }
    }

    #[tokio::test]
    async fn test_filter_no_attrs_kql_parser() {
        test_filter_no_attrs::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_no_attrs_opl_parser() {
        test_filter_no_attrs::<OplParser>().await;
    }

    async fn test_filter_property_is_null<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where severity_text == {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );

        // check it's supported if null literal on the left and column on the right
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == severity_text"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_property_is_null_kql_parser() {
        test_filter_property_is_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_property_is_null_opl_parser() {
        test_filter_property_is_null::<OplParser>("null").await;
    }

    async fn run_filter_property_is_not_null<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // severity_text != <null>
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where severity_text != {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );

        // <null> != severity_text
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} != severity_text"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_property_is_not_null_kql_parser() {
        run_filter_property_is_not_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_property_is_not_null_opl_parser() {
        run_filter_property_is_not_null::<OplParser>("null").await;
    }

    async fn run_filter_property_is_null_missing_column<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // just double check this gets encoded as something w/out the column we're using
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        assert!(logs_rb.column_by_name(consts::SEVERITY_TEXT).is_none());

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where severity_text == {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[
                log_records[0].clone(),
                log_records[1].clone(),
                log_records[2].clone()
            ],
        );

        // check it's supported if null literal on the left and column on the right
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == severity_text"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[
                log_records[0].clone(),
                log_records[1].clone(),
                log_records[2].clone()
            ],
        );
    }

    #[tokio::test]
    async fn test_filter_property_is_null_missing_column_kql_parser() {
        run_filter_property_is_null_missing_column::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_property_is_null_missing_column_opl_parser() {
        run_filter_property_is_null_missing_column::<OplParser>("null").await;
    }

    async fn run_filter_property_is_not_null_missing_column<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // just double check this gets encoded as something w/out the column we're using
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        assert!(logs_rb.column_by_name(consts::SEVERITY_TEXT).is_none());

        let parser_result = P::parse(&format!("logs | where severity_text != {null_lit}")).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        // assert it's equal to empty batch because there were no matches
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // assert we do the right thing where the null is on the left and value on the right
        let parser_result = P::parse(&format!("logs | where {null_lit} != severity_text")).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()))
    }

    #[tokio::test]
    async fn test_filter_property_is_not_null_missing_column_kql_parser() {
        run_filter_property_is_not_null_missing_column::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_property_is_not_null_missing_column_opl_parser() {
        run_filter_property_is_not_null_missing_column::<OplParser>("null").await;
    }

    async fn run_filter_struct_property_is_null<P: Parser>(null_lit: &str) {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        // test filter by scope properties
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where instrumentation_scope.name == {null_lit}"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[1].clone()],
                    ..Default::default()
                }],
            }
        );

        // test filter by scope properties, this time the null is on the left
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == instrumentation_scope.name"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[1].clone()],
                    ..Default::default()
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_null_kql_parser() {
        run_filter_struct_property_is_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_null_opl_parser() {
        run_filter_struct_property_is_null::<OplParser>("null").await;
    }

    async fn run_filter_struct_property_is_null_missing_column<P: Parser>(null_lit: &str) {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        // test filter by scope properties
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where instrumentation_scope.name == {null_lit}"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone(), scope_logs[1].clone()],
                    ..Default::default()
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_null_missing_column_kql_parser() {
        run_filter_struct_property_is_null_missing_column::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_null_missing_column_opl_parser() {
        run_filter_struct_property_is_null_missing_column::<OplParser>("null").await;
    }

    async fn run_struct_property_is_not_null<P: Parser>(null_lit: &str) {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        // test filter by scope properties
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where instrumentation_scope.name != {null_lit}"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );

        // test filter by scope properties, this time the null is on the left
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} != instrumentation_scope.name"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_struct_property_is_not_null_kql_parser() {
        run_struct_property_is_not_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_struct_property_is_not_null_opl_parser() {
        run_struct_property_is_not_null::<OplParser>("null").await;
    }

    async fn run_filter_struct_property_is_not_null_missing_column<P: Parser>(null_lit: &str) {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        let parser_result = P::parse(&format!(
            "logs | where instrumentation_scope.name != {null_lit}"
        ))
        .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(otlp_to_otap(&OtlpProtoMessage::Logs(input)))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()))
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_not_null_missing_column_kql_parser() {
        run_filter_struct_property_is_not_null_missing_column::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_not_null_missing_column_opl_parser() {
        run_filter_struct_property_is_not_null_missing_column::<OplParser>("null").await;
    }

    async fn run_filter_attribute_is_null<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build().event_name("2").finish(),
            LogRecord::build()
                .event_name("3")
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("b"))])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where attributes[\"x\"] == {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check the same thing works if we put null on the left
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == attributes[\"x\"]"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_attribute_is_null_kql_parser() {
        run_filter_attribute_is_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_attribute_is_null_opl_parser() {
        run_filter_attribute_is_null::<OplParser>("null").await;
    }

    async fn run_filter_attribute_is_null_no_attrs<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build().event_name("1").finish(),
            LogRecord::build().event_name("2").finish(),
            LogRecord::build().event_name("3").finish(),
        ];

        // double check that when we encode this as OTLP that the attributes
        // record batch is not present
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_none());

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where attributes[\"x\"] == {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &log_records.clone()
        );

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == attributes[\"x\"]"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &log_records.clone()
        );
    }

    #[tokio::test]
    async fn test_filter_attribute_is_null_no_attrs_kql_parser() {
        run_filter_attribute_is_null_no_attrs::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_attribute_is_null_no_attrs_opl_parser() {
        run_filter_attribute_is_null_no_attrs::<OplParser>("null").await;
    }

    async fn run_filter_attribute_is_not_null<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("b"))])
                .finish(),
            LogRecord::build().event_name("3").finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where attributes[\"x\"] != {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );

        // check the same thing works if we put null on the left
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} != attributes[\"x\"]"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_attribute_is_not_null_kql_parser() {
        run_filter_attribute_is_not_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_attribute_is_not_null_opl_parser() {
        run_filter_attribute_is_not_null::<OplParser>("null").await;
    }

    async fn run_filter_attribute_is_not_null_no_attrs<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build().event_name("1").finish(),
            LogRecord::build().event_name("2").finish(),
            LogRecord::build().event_name("3").finish(),
        ];

        // double check that when we encode this as OTLP that the attributes
        // record batch is not present
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_none());

        let parser_result =
            P::parse(&format!("logs | where attributes[\"x\"] != {null_lit}")).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        // assert it's equal to empty batch because there were no matches
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // assert we do the right thing where the null is on the left and value on the right
        let parser_result =
            P::parse(&format!("logs | where {null_lit} != attributes[\"x\"]")).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()))
    }

    #[tokio::test]
    async fn test_filter_attribute_is_not_null_no_attrs_kql_parser() {
        run_filter_attribute_is_not_null_no_attrs::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_attribute_is_not_null_no_attrs_opl_parser() {
        run_filter_attribute_is_not_null_no_attrs::<OplParser>("null").await;
    }

    async fn test_optional_attrs_existence_changes<P: Parser>() {
        // what happens if some optional attributes are present one batch, then not present in the
        // next, then present in the next, etc.

        let query = "logs | where attributes[\"a\"] == \"1234\"";
        let parser_result = P::parse(query).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);

        // no attrs to start
        let batch1 = to_otap_logs(vec![LogRecord::build().event_name("a").finish()]);
        let result = pipeline.execute(batch1).await.unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // now process a batch with some attrs
        let log_records = vec![
            LogRecord::build().finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("a", AnyValue::new_string("1234"))])
                .finish(),
        ];
        let batch2 = to_otap_logs(log_records.clone());
        let result = pipeline.execute(batch2).await.unwrap();
        let expected = to_otap_logs(log_records[1..2].to_vec());
        assert_eq!(result, expected);

        // handle another record batch with missing attributes
        let batch3 = to_otap_logs(vec![LogRecord::build().event_name("a").finish()]);
        let result = pipeline.execute(batch3).await.unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));
    }

    #[tokio::test]
    async fn test_filter_all_match() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // assert the behaviour is correct when nothing is filtered out
        let otap_input = to_otap_logs(log_records);
        let parser_result = KqlParser::parse("logs | where severity_text == \"INFO\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_input.clone()).await.unwrap();

        assert_eq!(result, otap_input)
    }

    #[tokio::test]
    async fn test_optional_attrs_existence_changes_kql_parser() {
        test_optional_attrs_existence_changes::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_optional_attrs_existence_changes_opl_parser() {
        test_optional_attrs_existence_changes::<OplParser>().await;
    }

    #[test]
    fn test_id_mask_contains() {
        let all = IdMask::All;
        let none = IdMask::None;
        let some = IdMask::Some(id_bitmap_from(&[1, 2, 3]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[1, 2, 3]));

        assert!(all.contains(5));
        assert!(!none.contains(5));
        assert!(some.contains(2));
        assert!(!some.contains(5));
        assert!(!not_some.contains(2));
        assert!(not_some.contains(5));
    }

    #[test]
    fn test_id_mask_bitor_basic() {
        let mut pool = IdBitmapPool::new();
        let some1 = IdMask::Some(id_bitmap_from(&[1, 2]));
        let some2 = IdMask::Some(id_bitmap_from(&[2, 3]));

        match some1.combine_or(some2, &mut pool) {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(1));
                assert!(bitmap.contains(2));
                assert!(bitmap.contains(3));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_id_mask_bitor_with_all_none() {
        let mut pool = IdBitmapPool::new();

        assert!(matches!(
            IdMask::All.combine_or(IdMask::Some(id_bitmap_from(&[1, 2])), &mut pool),
            IdMask::All
        ));
        assert!(matches!(
            IdMask::Some(id_bitmap_from(&[1, 2])).combine_or(IdMask::All, &mut pool),
            IdMask::All
        ));
        assert!(matches!(
            IdMask::None.combine_or(IdMask::Some(id_bitmap_from(&[1, 2])), &mut pool),
            IdMask::Some(_)
        ));
        assert!(matches!(
            IdMask::Some(id_bitmap_from(&[1, 2])).combine_or(IdMask::None, &mut pool),
            IdMask::Some(_)
        ));
    }

    #[test]
    fn test_id_mask_bitor_some_notsome() {
        let mut pool = IdBitmapPool::new();
        let some = IdMask::Some(id_bitmap_from(&[1, 2, 3]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[2, 3, 4]));

        // Some([1,2,3]) | NotSome([2,3,4]) = NotSome([4])
        // Because we select 1,2,3 plus everything except 2,3,4
        // Result: everything except 4
        match some.combine_or(not_some, &mut pool) {
            IdMask::NotSome(bitmap) => {
                assert!(bitmap.contains(4));
                assert!(!bitmap.contains(1));
                assert!(!bitmap.contains(2));
            }
            _ => panic!("Expected NotSome variant"),
        }
    }

    #[test]
    fn test_bitor_some_notsome_becomes_all() {
        let mut pool = IdBitmapPool::new();
        // For this to become All, we need the NotSome set to be a subset of Some
        let some = IdMask::Some(id_bitmap_from(&[1, 2, 3, 4, 5]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[2, 3]));

        // Some([1,2,3,4,5]) | NotSome([2,3])
        // = [1,2,3,4,5] plus everything except [2,3]
        // = everything (because [2,3] - [1,2,3,4,5] = empty)
        assert!(matches!(some.combine_or(not_some, &mut pool), IdMask::All));
    }

    #[test]
    fn test_id_mask_bitor_notsome_notsome() {
        let mut pool = IdBitmapPool::new();
        let not_some1 = IdMask::NotSome(id_bitmap_from(&[1, 2]));
        let not_some2 = IdMask::NotSome(id_bitmap_from(&[2, 3]));

        // NotSome([1,2]) | NotSome([2,3]) = NotSome([2])
        match not_some1.combine_or(not_some2, &mut pool) {
            IdMask::NotSome(bitmap) => {
                assert!(bitmap.contains(2));
                assert!(!bitmap.contains(1));
                assert!(!bitmap.contains(3));
            }
            _ => panic!("Expected NotSome variant"),
        }
    }

    #[test]
    fn test_id_mask_bitand_basic() {
        let mut pool = IdBitmapPool::new();
        let some1 = IdMask::Some(id_bitmap_from(&[1, 2, 3]));
        let some2 = IdMask::Some(id_bitmap_from(&[2, 3, 4]));

        match some1.combine_and(some2, &mut pool) {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(2));
                assert!(bitmap.contains(3));
                assert!(!bitmap.contains(1));
                assert!(!bitmap.contains(4));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_id_mask_bitand_with_all_none() {
        let mut pool = IdBitmapPool::new();

        assert!(matches!(
            IdMask::None.combine_and(IdMask::Some(id_bitmap_from(&[1, 2])), &mut pool),
            IdMask::None
        ));
        assert!(matches!(
            IdMask::Some(id_bitmap_from(&[1, 2])).combine_and(IdMask::None, &mut pool),
            IdMask::None
        ));
        assert!(matches!(
            IdMask::All.combine_and(IdMask::Some(id_bitmap_from(&[1, 2])), &mut pool),
            IdMask::Some(_)
        ));
        assert!(matches!(
            IdMask::Some(id_bitmap_from(&[1, 2])).combine_and(IdMask::All, &mut pool),
            IdMask::Some(_)
        ));
    }

    #[test]
    fn test_id_mask_bitand_some_notsome() {
        let mut pool = IdBitmapPool::new();
        let some = IdMask::Some(id_bitmap_from(&[1, 2, 3, 4]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[3, 4, 5]));

        // Some([1,2,3,4]) & NotSome([3,4,5]) = Some([1,2])
        match some.combine_and(not_some, &mut pool) {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(1));
                assert!(bitmap.contains(2));
                assert!(!bitmap.contains(3));
                assert!(!bitmap.contains(4));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_id_mask_bitand_some_notsome_becomes_none() {
        let mut pool = IdBitmapPool::new();
        let some = IdMask::Some(id_bitmap_from(&[1, 2]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[3, 4]));

        // Some([1,2]) & NotSome([3,4]) = Some([1,2])
        // (since [1,2] are not in [3,4])
        match some.combine_and(not_some, &mut pool) {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(1));
                assert!(bitmap.contains(2));
            }
            _ => panic!("Expected Some variant"),
        }

        // But Some([1,2]) & NotSome([1,2,3]) = None
        let some2 = IdMask::Some(id_bitmap_from(&[1, 2]));
        let not_some2 = IdMask::NotSome(id_bitmap_from(&[1, 2, 3]));
        assert!(matches!(
            some2.combine_and(not_some2, &mut pool),
            IdMask::None
        ));
    }

    #[test]
    fn test_id_mask_bitand_notsome_notsome() {
        let mut pool = IdBitmapPool::new();
        let not_some1 = IdMask::NotSome(id_bitmap_from(&[1, 2]));
        let not_some2 = IdMask::NotSome(id_bitmap_from(&[2, 3]));

        // NotSome([1,2]) & NotSome([2,3]) = NotSome([1,2,3])
        match not_some1.combine_and(not_some2, &mut pool) {
            IdMask::NotSome(bitmap) => {
                assert!(bitmap.contains(1));
                assert!(bitmap.contains(2));
                assert!(bitmap.contains(3));
            }
            _ => panic!("Expected NotSome variant"),
        }
    }
    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_key_match() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("KEY1", AnyValue::new_string("val1"))])
                .finish(),
        ];

        let query = "logs | where attributes[\"key1\"] == \"val1\"";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_key_match_escape_special_chars() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key%1_1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "keyabcd1x1",
                    AnyValue::new_string("val1"),
                )])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("KEY%1_1", AnyValue::new_string("val1"))])
                .finish(),
        ];

        let query = "logs | where attributes[\"key%1_1\"] == \"val1\"";
        let pipeline_expr = KqlParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_key_match_record_has_same_key_different_case()
     {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("key1", AnyValue::new_string("val1")),
                    KeyValue::new("KEY1", AnyValue::new_string("val2")),
                ])
                .finish(),
        ];

        let query = "logs | where attributes[\"key1\"] == \"val1\"";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        // test that since at least one of the attributes passes predicate, we get the result
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()]
        );

        // test the negation as well: since one of the attributes having "key1" is equal to
        // "val1", we filter out the record
        let query = "logs | where attributes[\"key1\"] != \"val2\"";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let result = pipeline.execute(input).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };
        assert!(&result.resource_logs.is_empty());
    }

    async fn test_filter_by_attributes_case_insensitive_equals<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("VAL1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let query = "logs | where attributes[\"key1\"] =~ \"val1\"";
        let pipeline_expr = P::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );

        // check it also works w/ the literal on the left
        let query = "logs | where \"val1\" =~ attributes[\"key1\"]";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_equals_opl_parser() {
        test_filter_by_attributes_case_insensitive_equals::<OplParser>().await
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_equals_kql_parser() {
        test_filter_by_attributes_case_insensitive_equals::<KqlParser>().await
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_equals_escapes_special_chars() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val%1_1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("VAL%1_1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("valA1B1"))])
                .finish(),
        ];

        let query = "logs | where attributes[\"key1\"] =~ \"val%1_1\"";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );

        // check it also escapes correctly when the literal is on the left
        let query = "logs | where  \"val%1_1\" =~ attributes[\"key1\"]";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_contains_case_insensitive_key_match() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("KEY1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let query = "logs | where contains(attributes[\"key1\"], \"1\")";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_matches_case_insensitive_key_match() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("KEY1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let query = "logs | where matches(attributes[\"key1\"], \".*1\")";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    // -----------------------------------------------------------------------
    // Tests for expression-backed filter predicates
    // -----------------------------------------------------------------------

    /// Filter comparing two root columns: severity_text == event_name
    async fn test_filter_column_vs_column<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .severity_text("match")
                .event_name("match")
                .finish(),
            LogRecord::build()
                .severity_text("a")
                .event_name("b")
                .finish(),
            LogRecord::build()
                .severity_text("other")
                .event_name("other")
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            "logs | where severity_text == event_name",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_column_vs_column_kql_parser() {
        test_filter_column_vs_column::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_column_vs_column_opl_parser() {
        test_filter_column_vs_column::<OplParser>().await;
    }

    /// Filter comparing a root column to an attribute: severity_number == attributes["x"]
    async fn test_filter_column_vs_attribute<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .severity_number(10)
                .event_name("1")
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(10))])
                .finish(),
            LogRecord::build()
                .severity_number(10)
                .event_name("2")
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(99))])
                .finish(),
            LogRecord::build()
                .severity_number(5)
                .event_name("3")
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(5))])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            "logs | where severity_number == attributes[\"x\"]",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_column_vs_attribute_kql_parser() {
        test_filter_column_vs_attribute::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_column_vs_attribute_opl_parser() {
        test_filter_column_vs_attribute::<OplParser>().await;
    }

    /// Filter comparing two attributes: attributes["x"] == attributes["y"]
    async fn test_filter_attribute_vs_attribute<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("same")),
                    KeyValue::new("y", AnyValue::new_string("same")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("b")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("match")),
                    KeyValue::new("y", AnyValue::new_string("match")),
                ])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            "logs | where attributes[\"x\"] == attributes[\"y\"]",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_attribute_vs_attribute_kql_parser() {
        test_filter_attribute_vs_attribute::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_attribute_vs_attribute_opl_parser() {
        test_filter_attribute_vs_attribute::<OplParser>().await;
    }

    /// Filter with arithmetic in the predicate: severity_number + 1 > 10
    async fn test_filter_arithmetic_in_predicate<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .severity_number(9)
                .event_name("1")
                .finish(),
            LogRecord::build()
                .severity_number(10)
                .event_name("2")
                .finish(),
            LogRecord::build()
                .severity_number(17)
                .event_name("3")
                .finish(),
        ];

        // severity_number + 1 > 10 means severity_number > 9
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_number + 1 > 10",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_arithmetic_in_predicate_kql_parser() {
        test_filter_arithmetic_in_predicate::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_arithmetic_in_predicate_opl_parser() {
        test_filter_arithmetic_in_predicate::<OplParser>().await;
    }

    /// Filter with cross-scope arithmetic: severity_number > attributes["threshold"]
    async fn test_filter_cross_scope_gt<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .severity_number(10)
                .event_name("1")
                .attributes(vec![KeyValue::new("threshold", AnyValue::new_int(5))])
                .finish(),
            LogRecord::build()
                .severity_number(3)
                .event_name("2")
                .attributes(vec![KeyValue::new("threshold", AnyValue::new_int(5))])
                .finish(),
            LogRecord::build()
                .severity_number(20)
                .event_name("3")
                .attributes(vec![KeyValue::new("threshold", AnyValue::new_int(15))])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            "logs | where severity_number > attributes[\"threshold\"]",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_cross_scope_gt_kql_parser() {
        test_filter_cross_scope_gt::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_cross_scope_gt_opl_parser() {
        test_filter_cross_scope_gt::<OplParser>().await;
    }

    /// Filter combining expr-based predicate with traditional fast-path filter via AND
    async fn test_filter_expr_combined_with_fast_path<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .severity_number(17)
                .event_name("match_both")
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(17))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .severity_number(10)
                .event_name("match_text_only")
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(99))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .severity_number(5)
                .event_name("match_neither")
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(5))])
                .finish(),
        ];

        // severity_text == "ERROR" uses fast path; severity_number == attributes["x"] uses expr path
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_text == \"ERROR\" and severity_number == attributes[\"x\"]",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_expr_combined_with_fast_path_kql_parser() {
        test_filter_expr_combined_with_fast_path::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_expr_combined_with_fast_path_opl_parser() {
        test_filter_expr_combined_with_fast_path::<OplParser>().await;
    }

    #[tokio::test]
    async fn test_filter_comparing_root_parent_fields_with_attributes() {
        let log_record0 = LogRecord::build()
            .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
            .finish();
        let log_record1 = LogRecord::build()
            .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
            .finish();
        let log_record2 = LogRecord::build()
            .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
            .finish();

        let logs_data = LogsData::new(vec![
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("y", AnyValue::new_string("0"))])
                        .finish(),
                ),
                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![log_record0.clone(), log_record1.clone()],
                )],
                schema_url: "0".into(),
            },
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("y", AnyValue::new_string("1"))])
                        .finish(),
                ),
                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![log_record2.clone()],
                )],
                schema_url: "2".into(),
            },
        ]);

        // severity_text == "ERROR" uses fast path; severity_number == attributes["x"] uses expr path
        let result = exec_logs_pipeline::<OplParser>(
            "logs | where resource.schema_url == resource.attributes[\"y\"]",
            logs_data,
        )
        .await;
        assert_eq!(result.resource_logs.len(), 1);
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_record0.clone(), log_record1.clone()]
        );
    }

    /// Filter where no rows match the expression predicate
    async fn test_filter_expr_no_match<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .severity_text("a")
                .event_name("x")
                .finish(),
            LogRecord::build()
                .severity_text("b")
                .event_name("y")
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            "logs | where severity_text == event_name",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert!(result.resource_logs.is_empty());
    }

    #[tokio::test]
    async fn test_filter_expr_no_match_kql_parser() {
        test_filter_expr_no_match::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_expr_no_match_opl_parser() {
        test_filter_expr_no_match::<OplParser>().await;
    }

    /// Filter with missing attributes -- rows without the attribute should not pass
    async fn test_filter_expr_missing_attribute<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .severity_number(10)
                .event_name("has_attr")
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(10))])
                .finish(),
            LogRecord::build()
                .severity_number(10)
                .event_name("no_attr")
                .finish(),
        ];

        // the second record has no attributes so the cross-scope join produces no result
        // for it -- it should not pass the filter
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_number == attributes[\"x\"]",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_expr_missing_attribute_kql_parser() {
        test_filter_expr_missing_attribute::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_expr_missing_attribute_opl_parser() {
        test_filter_expr_missing_attribute::<OplParser>().await;
    }

    /// Filter using substring function result in the predicate
    async fn test_filter_with_substring<P: Parser>(q: &str) {
        let log_records = vec![
            LogRecord::build()
                .severity_text("ERR_timeout")
                .event_name("1")
                .finish(),
            LogRecord::build()
                .severity_text("INFO_normal")
                .event_name("2")
                .finish(),
            LogRecord::build()
                .severity_text("ERR_connection")
                .event_name("3")
                .finish(),
        ];

        // substring(severity_text, 0, 3) extracts the first 3 chars
        let result = exec_logs_pipeline::<P>(q, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_with_substring_kql_parser() {
        test_filter_with_substring::<KqlParser>(
            r#"logs | where substring(severity_text, 0, 3) == "ERR""#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_with_substring_opl_parser() {
        test_filter_with_substring::<OplParser>(
            r#"logs | where substring(severity_text, 0, 3) == "ERR""#,
        )
        .await;
    }

    /// Filter using substring on an attribute value
    async fn test_filter_with_substring_on_attribute<P: Parser>(q: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![KeyValue::new("code", AnyValue::new_string("ERR-001"))])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![KeyValue::new("code", AnyValue::new_string("OK-200"))])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .attributes(vec![KeyValue::new("code", AnyValue::new_string("ERR-502"))])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(q, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_with_substring_on_attribute_kql_parser() {
        test_filter_with_substring_on_attribute::<KqlParser>(
            r#"logs | where substring(attributes["code"], 0, 3) == "ERR""#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_with_substring_on_attribute_opl_parser() {
        test_filter_with_substring_on_attribute::<OplParser>(
            r#"logs | where substring(attributes["code"], 0, 3) == "ERR""#,
        )
        .await;
    }

    /// Filter using contains where the needle is a column reference (not a literal).
    /// This exercises the `try_from_contains_expr_via_expr_eval` fallback.
    async fn test_filter_contains_column_needle<P: Parser>(q: &str) {
        let log_records = vec![
            LogRecord::build()
                .severity_text("error in module auth")
                .event_name("auth")
                .finish(),
            LogRecord::build()
                .severity_text("warning from module payments")
                .event_name("payments")
                .finish(),
            LogRecord::build()
                .severity_text("error in module auth")
                .event_name("missing")
                .finish(),
        ];

        // contains(severity_text, event_name) -- needle is a column, not a literal
        let result = exec_logs_pipeline::<P>(q, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_contains_column_needle_kql_parser() {
        test_filter_contains_column_needle::<KqlParser>(
            r#"logs | where severity_text contains event_name"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_contains_column_needle_opl_parser() {
        test_filter_contains_column_needle::<OplParser>(
            r#"logs | where contains(severity_text, event_name)"#,
        )
        .await;
    }

    /// Filter using contains where the needle is an attribute (cross-scope contains).
    /// This exercises the expression-eval fallback for contains with cross-scope args.
    async fn test_filter_contains_attribute_needle<P: Parser>(q: &str) {
        let log_records = vec![
            LogRecord::build()
                .severity_text("error: timeout occurred")
                .event_name("1")
                .attributes(vec![KeyValue::new(
                    "keyword",
                    AnyValue::new_string("timeout"),
                )])
                .finish(),
            LogRecord::build()
                .severity_text("info: all good")
                .event_name("2")
                .attributes(vec![KeyValue::new(
                    "keyword",
                    AnyValue::new_string("failure"),
                )])
                .finish(),
            LogRecord::build()
                .severity_text("warn: disk failure detected")
                .event_name("3")
                .attributes(vec![KeyValue::new(
                    "keyword",
                    AnyValue::new_string("failure"),
                )])
                .finish(),
        ];

        // contains(severity_text, attributes["keyword"]) -- cross-scope contains
        let result = exec_logs_pipeline::<P>(q, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_contains_attribute_needle_kql_parser() {
        test_filter_contains_attribute_needle::<KqlParser>(
            r#"logs | where severity_text contains attributes["keyword"]"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_contains_attribute_needle_opl_parser() {
        test_filter_contains_attribute_needle::<OplParser>(
            r#"logs | where contains(severity_text, attributes["keyword"])"#,
        )
        .await;
    }

    /// Filter using contains where both haystack and needle are attributes (cross-scope
    /// on both sides). This exercises `build_scoped_contains_expr` with a Join.
    async fn test_filter_contains_attribute_both_sides<P: Parser>(q: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![
                    KeyValue::new("haystack", AnyValue::new_string("hello world")),
                    KeyValue::new("needle", AnyValue::new_string("world")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("haystack", AnyValue::new_string("foo bar")),
                    KeyValue::new("needle", AnyValue::new_string("baz")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .attributes(vec![
                    KeyValue::new("haystack", AnyValue::new_string("quick brown fox")),
                    KeyValue::new("needle", AnyValue::new_string("brown")),
                ])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(q, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_contains_attribute_both_sides_kql_parser() {
        test_filter_contains_attribute_both_sides::<KqlParser>(
            r#"logs | where attributes["haystack"] contains attributes["needle"]"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_contains_attribute_both_sides_opl_parser() {
        test_filter_contains_attribute_both_sides::<OplParser>(
            r#"logs | where contains(attributes["haystack"], attributes["needle"])"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_check_signal_types() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![])
                .finish(),
        ];

        let query = "signals | where is Log";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        let logs_input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records)));
        let logs_ouptut = pipeline.execute(logs_input.clone()).await.unwrap();

        assert_eq!(logs_input, logs_ouptut);

        let traces_input = otlp_to_otap(&OtlpProtoMessage::Traces(to_traces_data(vec![
            Span::default(),
        ])));
        let traces_ouptut = pipeline.execute(traces_input).await.unwrap();

        // assert it returns empty traces
        assert_eq!(traces_ouptut, OtapArrowRecords::Traces(Traces::default()));
    }

    #[tokio::test]
    async fn test_filter_check_signal_type_inverted() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![])
                .finish(),
        ];

        let query = "signals | where not(is Log)";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        let logs_input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records)));
        let logs_ouptut = pipeline.execute(logs_input).await.unwrap();

        // assert it returns empty traces
        assert_eq!(logs_ouptut, OtapArrowRecords::Logs(Logs::default()));

        let traces_input = otlp_to_otap(&OtlpProtoMessage::Traces(to_traces_data(vec![
            Span::default(),
        ])));
        let traces_ouptut = pipeline.execute(traces_input.clone()).await.unwrap();

        assert_eq!(traces_input, traces_ouptut);
    }

    #[tokio::test]
    async fn test_filter_by_attr_type() {
        let log_records = vec![
            LogRecord::build()
                .attributes([
                    KeyValue::new("attr_always_string", AnyValue::new_string("hello")),
                    KeyValue::new("attr_always_int", AnyValue::new_int(1)),
                    KeyValue::new("mixed_types", AnyValue::new_string("hello")),
                ])
                .finish(),
            LogRecord::build()
                .attributes([
                    KeyValue::new("attr_always_string", AnyValue::new_string("hello")),
                    KeyValue::new("mixed_types", AnyValue::new_bool(false)),
                ])
                .finish(),
        ];

        // check selects correctly all rows when they all have the attribute with the matching type
        let query = "logs | where attributes[\"attr_always_string\"] is String";
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;

        let expected = vec![log_records[0].clone(), log_records[1].clone()];
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &expected
        );

        // check we can invert the case above
        let query = "logs | where not(attributes[\"attr_always_string\"] is String)";
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;
        assert_eq!(result.resource_logs.len(), 0, "expected empty result");

        // check omits correctly all rows when  they all have the attribute with a non-matching type
        let query = "logs | where attributes[\"attr_always_string\"] is Integer";
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;
        assert_eq!(result.resource_logs.len(), 0, "expected empty result");

        // check we select correctly only some rows have the attribute
        let query = "logs | where attributes[\"attr_always_int\"] is Integer";
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;
        let expected = vec![log_records[0].clone()];
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &expected
        );

        // check we select correctly only some rows have the attribute
        let query = "logs | where attributes[\"mixed_types\"] is String";
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;
        let expected = vec![log_records[0].clone()];
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &expected
        );

        // check we select correctly only some rows have the attribute
        let query = "logs | where attributes[\"mixed_types\"] is Boolean";
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;
        let expected = vec![log_records[1].clone()];
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &expected
        );
    }

    #[tokio::test]
    async fn test_filter_by_attr_array_map_null_type() {
        let log_records = vec![
            LogRecord::build()
                .attributes([KeyValue::new(
                    "complex",
                    AnyValue::new_array(vec![AnyValue::new_int(1), AnyValue::new_int(2)]),
                )])
                .finish(),
            LogRecord::build()
                .attributes([KeyValue::new(
                    "complex",
                    AnyValue::new_kvlist(vec![KeyValue::new("nested", AnyValue::new_bool(true))]),
                )])
                .finish(),
            LogRecord::build()
                .attributes([KeyValue::new("complex", AnyValue::default())])
                .finish(),
            LogRecord::build()
                .attributes([KeyValue::new("complex", AnyValue::new_string("hello"))])
                .finish(),
        ];

        let cases = [
            ("Array", log_records[0].clone()),
            ("Map", log_records[1].clone()),
            ("Null", log_records[2].clone()),
        ];

        for (value_type, expected_record) in cases {
            let query = format!("logs | where attributes[\"complex\"] is {value_type}");
            let result =
                exec_logs_pipeline::<OplParser>(&query, to_logs_data(log_records.clone())).await;
            assert_eq!(
                &result.resource_logs[0].scope_logs[0].log_records,
                &[expected_record],
                "value_type={value_type}"
            );
        }
    }

    #[tokio::test]
    async fn test_filter_by_known_type() {
        let log_records = vec![
            LogRecord::build().severity_text("DEBUG").finish(),
            LogRecord::build().severity_text("INFO").finish(),
        ];

        let query = "logs | where severity_text is String";
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;

        let expected = vec![log_records[0].clone(), log_records[1].clone()];
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &expected
        );

        let query = "logs | where severity_text is Integer";
        let result =
            exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records.clone())).await;
        assert_eq!(result.resource_logs.len(), 0, "expected empty result");
    }
}
