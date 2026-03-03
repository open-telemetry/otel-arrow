// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of a [`PipelineStage`] for assigning the result of
//! the evaluation of an expression to a column in an OTAP record batch.
//!
//! It services queries such as:
//! ```text
//! logs | set severity_text = "INFO"
//! ```
//!
//! Note: implementation is currently a work in progress, and not all destinations are supported
//!
//! TODO
//! - support assigning to more destinations (struct fields, attributes)
//! - attempt automatic result type coercion (binary -> FSB, int types) (not sure if needed/wanted)
//! -

use std::rc::Rc;
use std::sync::Arc;

use arrow::array::{ArrayRef, DictionaryArray, RecordBatch, UInt8Array};
use arrow::compute::{cast, take};
use arrow::datatypes::{DataType, Field, Schema, UInt16Type};
use async_trait::async_trait;
use data_engine_expressions::{
    Expression, QueryLocation, ScalarExpression, SourceScalarExpression,
};
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::logical_expr::ColumnarValue;
use datafusion::prelude::SessionContext;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::Logs;
use otap_df_pdata::otap::transform::concatenate::{Cardinality, FieldInfo, estimate_cardinality};

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::expr::join::{JoinExec, RootToAttributesJoin};
use crate::pipeline::expr::types::{
    ExprLogicalType, root_field_supports_dict_encoding, root_field_type,
};
use crate::pipeline::expr::{
    DataScope, ExprLogicalPlanner, ExprPhysicalPlanner, PhysicalExprEvalResult, ScopedLogicalExpr,
    ScopedPhysicalExpr,
};
use crate::pipeline::planner::ColumnAccessor;
use crate::pipeline::state::ExecutionState;

/// Pipeline stage for assigning the result of an expression evaluation to an OTAP column
pub(crate) struct AssignPipelineStage {
    /// Identifier of the destination column
    dest_column: ColumnAccessor,

    /// Data Scope of the destination column.
    ///
    /// This is used at execution time to join the results which may have been computed using data
    /// that has a different row order from the destination column. Although this type can be
    /// computed from dest_column, we create it up-front to avoid cloning data during evaluation
    dest_scope: Rc<DataScope>,

    /// Expression that will produce the data to be assigned to the destination
    source: ScopedPhysicalExpr,
}

impl AssignPipelineStage {
    /// Create a new instance of [`AssignPipelineStage`]
    pub fn try_new(dest: &SourceScalarExpression, source: &ScalarExpression) -> Result<Self> {
        let logical_planner = ExprLogicalPlanner::default();
        let source_logical_plan = logical_planner.plan_scalar_expr(source)?;

        let dest_column = ColumnAccessor::try_from(dest.get_value_accessor())?;
        validate_assign(
            &dest_column,
            dest.get_query_location(),
            &source_logical_plan,
        )?;

        let physical_planner = ExprPhysicalPlanner::default();
        let physical_expr = physical_planner.plan(source_logical_plan)?;

        Ok(Self {
            dest_scope: Rc::new(dest_column.clone().into()),
            dest_column: dest_column,
            source: physical_expr,
        })
    }

    /// Assign the result of the expression evaluation to a column on the root record batch.
    fn assign_to_root(
        &self,
        mut otap_batch: OtapArrowRecords,
        mut eval_result: PhysicalExprEvalResult,
        dest_column_name: &str,
    ) -> Result<OtapArrowRecords> {
        let root_batch = match otap_batch.root_record_batch() {
            Some(rb) => rb,
            None => {
                // nothing to do
                return Ok(otap_batch);
            }
        };

        // check that the result type of the expr eval can be assigned to this field
        let expected_column_logical_type = root_field_type(dest_column_name)
            // safety: this will only return None if the destination column does not exist in OTAP
            // data model, but this has been validated in the constructor of this type, so it's
            // safe to expect here
            .expect("dest column found");

        let expected_column_data_type = expected_column_logical_type
            .datatype()
            // safety: this will only return None if the logical data type for the field is
            // ambiguous, which is the case for attributes/AnyValues, but all the fields on the
            // root batch are known/un-ambiguous, so this will return Some and is safe to expect
            .expect("dest column data type");

        // coerce static scalar int ...
        // if the result was a static scalar integer, it will have been produced as an int64 by
        // default, however the expression tree doesn't actually specify the type, so we assume the
        // type should have matched the expected type
        let mut eval_result_column_type = eval_result.values.data_type();
        if eval_result.data_scope.as_ref() == &DataScope::StaticScalar
            && eval_result_column_type.is_integer()
            && expected_column_data_type.is_integer()
        {
            eval_result.values = eval_result
                .values
                .cast_to(&expected_column_data_type, None)?;
            eval_result_column_type = expected_column_data_type.clone();
        }

        // check that the result type of the expr eval can be assigned to this field
        let column_supports_dict_encoding = root_field_supports_dict_encoding(dest_column_name);
        let mut type_compatible = expected_column_data_type == eval_result_column_type;
        if !type_compatible && column_supports_dict_encoding {
            // if the field can be dictionary encoded, check if the eval result is also dictionary
            // encoded. If so, we're allowed to make the assignment
            if let DataType::Dictionary(_, val) = &eval_result_column_type {
                if val.as_ref() == &expected_column_data_type {
                    type_compatible = true
                }
            }
        }

        if !type_compatible {
            return Err(Error::ExecutionError {
                cause: format!(
                    "cannot assign expression result of type {:?} to column expecting type {:?}",
                    eval_result_column_type, expected_column_data_type
                ),
            });
        }

        // convert the expression evaluation result to an array, with the correct dict encoding if
        // the destination column supports it
        let mut values = eval_result_to_array(
            &eval_result.values,
            column_supports_dict_encoding,
            root_batch.num_rows(),
        )?;

        // align the rows in the new values with the rows in the root batch, if not already aligned
        let already_aligned = eval_result.data_scope.is_scalar()
            || eval_result.data_scope.as_ref() == self.dest_scope.as_ref();

        if !already_aligned {
            // if we're here, it means we have received a column value that has the row order
            // of something other than the root attribute batch, meaning the result was computed
            // from attributes. We'll need to join the result's values column to the root column
            // to get the values in the correct order
            let DataScope::Attributes(attrs_id, _) = eval_result.data_scope.as_ref() else {
                // safety: if the data_scope were anything other than attributes, we'd have taken
                // the if branch (not the else branch) above when we checked if the data was
                // already aligned
                unreachable!("unexpected data_scope")
            };

            // create a JoinExec implementation that will compute a join of root to attrs on
            // `root.id == attrs.parent_id`` and use this to get the rows to take from the result
            let join_exec = RootToAttributesJoin::new(attrs_id.clone());
            let vals_take_indices = join_exec.rows_to_take(
                &PhysicalExprEvalResult::new(
                    ColumnarValue::Scalar(ScalarValue::Null), // empty placeholder,
                    self.dest_scope.clone(),
                    root_batch,
                ),
                &eval_result,
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )?;

            values = take(&values, &vals_take_indices, None)?;
        };

        // build the new record batch ..
        let mut columns = root_batch.columns().to_vec();
        let schema = root_batch.schema();
        let fields = schema.fields();
        let target_col_index = fields.find(dest_column_name).map(|(position, _)| position);
        let mut fields = fields.to_vec();

        if let Some(target_col_index) = target_col_index {
            // replace field if the datatype has changed. Note, we wont have changed the logical
            // type of the field, but the dictionary encoding may be what has changed
            let needs_field_update = fields[target_col_index].data_type() != values.data_type();
            if needs_field_update {
                fields
                    .iter_mut()
                    .enumerate()
                    .for_each(|(curr_index, field)| {
                        if target_col_index == curr_index {
                            let new_field = field
                                .as_ref()
                                .clone()
                                .with_data_type(values.data_type().clone());
                            *field = Arc::new(new_field)
                        }
                    });
            }

            // replace column
            columns
                .iter_mut()
                .enumerate()
                .for_each(|(curr_index, col)| {
                    if target_col_index == curr_index {
                        *col = Arc::clone(&values)
                    }
                });
        } else {
            // just insert the new column at the end
            fields.push(Arc::new(Field::new(
                dest_column_name,
                values.data_type().clone(),
                // Note: here we're assuming that since the column was missing that it was an
                // optional column which means that it is nullable
                true,
            )));
            columns.push(values)
        }

        // safety: try_new will only fail here if the column types are invalid for the schema, or
        // if the columns don't all have the same length, but we've computed the fields/columns in
        // such a way that they should be valid and this should return Ok
        let new_root_batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("can create record batch");

        // replace the root record batch with the new one
        let root_payload_type = otap_batch.root_payload_type();
        otap_batch.set(root_payload_type, new_root_batch);

        Ok(otap_batch)
    }
}

#[async_trait(?Send)]
impl PipelineStage for AssignPipelineStage {
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_options: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        let eval_result = self.source.execute(&otap_batch, session_context)?;

        match eval_result {
            Some(eval_result) => match &self.dest_column {
                ColumnAccessor::ColumnName(col_name) => {
                    self.assign_to_root(otap_batch, eval_result, col_name)
                }
                other_dest => {
                    return Err(Error::NotYetSupportedError {
                        message: format!(
                            "assignment to column destination {:?} not yet supported",
                            other_dest
                        ),
                    });
                }
            },
            None => {
                todo!() // remove data
            }
        }
    }
}

/// Validate that the results of the passed expression can be assigned to the destination.
/// This validates three things:
///
/// 1. that the destination exists, that it is a known column in OTAP
///
/// 2. This will validate the types:
///
/// Specifically it will check that the type could possibly be
/// assigned to the destination, but it does not guarantee that the expression will produce
/// a valid type for the assignment. For example, in an expression like:
/// ```text
/// severity_text = attributes["x"]
/// ```
/// This would pass validation because `attributes["x"]` could be a string, which is what the
/// destination `severity_text` expects. However, when this is evaluated, we may find that
/// `attributes["x"]` is not a string, in which case this would fail at runtime.
///
/// 3. This validates that there is not ambiguity in the assignment based on the cardinality of
/// the relationship between source and destination. Specifically, if the dest:source relationship
/// is one:many, then we cannot do the assignment because it's unclear which of the many source
/// values should be assigned to the destination.
///
/// Here is an example of this type of invalid assignment:
/// ```text
/// logs | set resource.attributes["x"] = severity_text
/// ```
/// because there are many logs w/ possibly different severities for any given resource, we
/// consider this assignment invalid.
///
fn validate_assign(
    dest_column: &ColumnAccessor,
    dest_query_location: &QueryLocation,
    source_logical_plan: &ScopedLogicalExpr,
) -> Result<()> {
    match dest_column {
        ColumnAccessor::ColumnName(col_name) => {
            // No relationship cardinality validation needs to happen for these columns which
            // are on the root record because they are not one:many with anything else in that
            // could be assigned, so validation only checks the types:

            let dest_type =
                root_field_type(col_name).ok_or_else(|| Error::InvalidPipelineError {
                    cause: format!("cannot assign to non-existent column '{col_name}'"),
                    query_location: Some(dest_query_location.clone()),
                })?;

            let source_type = &source_logical_plan.expr_type;
            if !can_assign_type(&dest_type, source_type) {
                return Err(Error::InvalidPipelineError {
                    cause: format!(
                        "cannot assign expression of type {source_type:?} to type {dest_type:?}"
                    ),
                    query_location: Some(dest_query_location.clone()),
                });
            }
        }
        other_dest => {
            // TODO other assignment destinations will be supported soon
            return Err(Error::NotYetSupportedError {
                message: format!(
                    "assignment to column destination {:?} not yet supported",
                    other_dest
                ),
            });
        }
    }

    return Ok(());
}

/// Determine if the source type could be assigned to the destination
fn can_assign_type(dest_type: &ExprLogicalType, source_type: &ExprLogicalType) -> bool {
    if dest_type == source_type {
        return true;
    }

    // scalar int type can be converted to any integer type
    if dest_type.is_integer() && source_type == &ExprLogicalType::ScalarInt {
        return true;
    }

    match dest_type {
        ExprLogicalType::Boolean | ExprLogicalType::String => {
            source_type == &ExprLogicalType::AnyValue
        }
        ExprLogicalType::Float64 => {
            source_type == &ExprLogicalType::AnyValue
                || source_type == &ExprLogicalType::AnyValueNumeric
        }
        ExprLogicalType::Int64 => match source_type {
            ExprLogicalType::AnyValue
            | ExprLogicalType::AnyValueNumeric
            | ExprLogicalType::ScalarInt => true,
            _ => false,
        },

        // TODO - handle other cases as we support a greater variety of destinations
        _ => false,
    }
}

// TOOD comment on what this does
fn eval_result_to_array(
    expr_eval_result: &ColumnarValue,
    accept_dict_encoding: bool,
    dest_num_rows: usize,
) -> Result<ArrayRef> {
    match expr_eval_result {
        ColumnarValue::Scalar(scalar_val) => {
            if accept_dict_encoding {
                // create a dictionary with a single value, and all keys selecting this value
                let dict_values = scalar_val.to_array()?;
                let dict_keys = UInt8Array::from_iter_values(std::iter::repeat_n(0, dest_num_rows));
                Ok(Arc::new(DictionaryArray::new(dict_keys, dict_values)))
            } else {
                Ok(scalar_val.to_array_of_size(dest_num_rows)?)
            }
        }
        ColumnarValue::Array(array_vals) => {
            if accept_dict_encoding {
                // here we're going to try to select the smallest dictionary key that could contain
                // all the unique values
                match array_vals.data_type() {
                    DataType::Dictionary(k, v) => match k.as_ref() {
                        DataType::UInt8 => {
                            // already smallest dict size
                            Ok(Arc::clone(array_vals))
                        }
                        DataType::UInt16 => {
                            // check if we can use a smaller dictionary key
                            let values_as_dict = array_vals
                                .as_any()
                                .downcast_ref::<DictionaryArray<UInt16Type>>()
                                .expect("can downcast to dict");
                            if values_as_dict.values().len() <= 255 {
                                // values can fit in a smaller dict
                                Ok(cast(
                                    &array_vals,
                                    &DataType::Dictionary(
                                        Box::new(DataType::UInt8),
                                        Box::new(v.as_ref().clone()),
                                    ),
                                )?)
                            } else {
                                // values already a dict, but won't fit in a smaller dict
                                Ok(Arc::clone(array_vals))
                            }
                        }
                        other_key_type => Err(Error::ExecutionError {
                            cause: format!(
                                "invalid dictionary key in evaluation result {other_key_type:?}"
                            ),
                        }),
                    },
                    _ => {
                        // array is not dictionary encoded -- determine if we should convert it
                        let field_info = FieldInfo::try_new_from_array(&array_vals);
                        let cardinality = estimate_cardinality(&field_info);
                        let key_type = match cardinality {
                            Cardinality::WithinU8 => Some(DataType::UInt8),
                            Cardinality::WithinU16 => Some(DataType::UInt16),
                            _ => None,
                        };

                        if let Some(key_type) = key_type {
                            // convert to smallest dictionary key allowed by cardinality
                            Ok(cast(
                                &array_vals,
                                &DataType::Dictionary(
                                    Box::new(key_type),
                                    Box::new(array_vals.data_type().clone()),
                                ),
                            )?)
                        } else {
                            Ok(Arc::clone(array_vals))
                        }
                    }
                }
            } else {
                // TODO - do we need to remove dictionary encoding?
                Ok(Arc::clone(array_vals))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use data_engine_kql_parser::{KqlParser, Parser};
    use otap_df_opl::parser::OplParser;
    use otap_df_pdata::{
        OtapArrowRecords,
        otap::Logs,
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                common::v1::{AnyValue, InstrumentationScope, KeyValue},
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                resource::v1::Resource,
            },
        },
        testing::round_trip::{otlp_to_otap, to_logs_data},
    };

    use crate::pipeline::{Pipeline, planner::PipelinePlanner, test::exec_logs_pipeline};

    async fn test_insert_root_column_from_scalar<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().finish(),
            LogRecord::build().finish(),
        ]);
        let result =
            exec_logs_pipeline::<P>("logs | extend severity_text = \"ERROR\"", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 2);
        for logs_record in logs_records {
            assert_eq!(logs_record.severity_text, "ERROR");
        }
    }

    #[tokio::test]
    async fn test_insert_root_column_from_scalar_opl_parser() {
        test_insert_root_column_from_scalar::<OplParser>().await
    }

    #[tokio::test]
    async fn test_insert_root_column_from_scalar_kql_parser() {
        test_insert_root_column_from_scalar::<KqlParser>().await
    }

    async fn test_upsert_root_column_from_scalar<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().finish(),
        ]);
        let result =
            exec_logs_pipeline::<P>("logs | extend severity_text = \"ERROR\"", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 2);
        for logs_record in logs_records {
            assert_eq!(logs_record.severity_text, "ERROR");
        }
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_scalar_opl_parser() {
        test_upsert_root_column_from_scalar::<OplParser>().await
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_scalar_kql_parser() {
        test_upsert_root_column_from_scalar::<KqlParser>().await
    }

    async fn test_insert_root_column_from_int_scalar<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().finish(),
            LogRecord::build().finish(),
        ]);
        let result = exec_logs_pipeline::<P>("logs | extend severity_number = 1", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 2);
        for logs_record in logs_records {
            assert_eq!(logs_record.severity_number, 1);
        }
    }

    #[tokio::test]
    async fn test_insert_root_column_from_int_scalar_opl_parser() {
        test_insert_root_column_from_int_scalar::<OplParser>().await
    }

    #[tokio::test]
    async fn test_insert_root_column_from_int_scalar_kql_parser() {
        test_insert_root_column_from_int_scalar::<KqlParser>().await
    }

    async fn test_insert_root_column_from_other_column<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().severity_text("DEBUG").finish(),
        ]);

        // kind of a silly example, but just need two cols that have the same type for the test
        let result =
            exec_logs_pipeline::<P>("logs | extend event_name = severity_text", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 2);
        assert_eq!(logs_records[0].event_name, "INFO");
        assert_eq!(logs_records[1].event_name, "DEBUG");
    }

    #[tokio::test]
    async fn test_insert_root_column_from_other_column_opl_parser() {
        test_insert_root_column_from_other_column::<OplParser>().await
    }

    #[tokio::test]
    async fn test_insert_root_column_from_other_column_kql_parser() {
        test_insert_root_column_from_other_column::<KqlParser>().await
    }

    async fn test_upsert_root_column_from_other_column<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .severity_text("INFO")
                .event_name("event1")
                .finish(),
            LogRecord::build().severity_text("DEBUG").finish(),
        ]);

        // kind of a silly example, but just need two cols that have the same type for the test
        let result =
            exec_logs_pipeline::<P>("logs | extend event_name = severity_text", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 2);
        assert_eq!(logs_records[0].event_name, "INFO");
        assert_eq!(logs_records[1].event_name, "DEBUG");
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_other_column_opl_parser() {
        test_upsert_root_column_from_other_column::<OplParser>().await
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_other_column_kql_parser() {
        test_upsert_root_column_from_other_column::<KqlParser>().await
    }

    async fn test_set_root_column_from_attribute<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("event", AnyValue::new_string("hello"))])
                .finish(),
            LogRecord::build()
                .event_name("replaceme")
                .attributes(vec![KeyValue::new("event", AnyValue::new_string("world"))])
                .finish(),
            // no event attribute, result should be ""..
            LogRecord::build().finish(),
            LogRecord::build().event_name("replaceme").finish(),
        ]);

        let result = exec_logs_pipeline::<P>(
            "logs | extend event_name = attributes[\"event\"]",
            logs_data,
        )
        .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 4);
        assert_eq!(logs_records[0].event_name, "hello");
        assert_eq!(logs_records[1].event_name, "world");
        assert_eq!(logs_records[2].event_name, "");
        assert_eq!(logs_records[3].event_name, "");
    }

    #[tokio::test]
    async fn test_set_root_column_from_attribute_opl_parser() {
        test_set_root_column_from_attribute::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_from_attribute_kql_parser() {
        test_set_root_column_from_attribute::<KqlParser>().await
    }

    async fn test_set_root_column_from_arithmetic_expression<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .severity_number(2)
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(1))])
                .finish(),
            LogRecord::build()
                .severity_number(3)
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(2))])
                .finish(),
            LogRecord::build().finish(),
            LogRecord::build().event_name("replaceme").finish(),
        ]);

        // kind of a weird expression in practice, but this is just checking if the expr evaluates
        let result = exec_logs_pipeline::<P>(
            "logs | extend severity_number = 5 + severity_number * 10",
            logs_data,
        )
        .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 4);
        assert_eq!(logs_records[0].severity_number, 25);
        assert_eq!(logs_records[1].severity_number, 35);
        assert_eq!(logs_records[2].severity_number, 0);
        assert_eq!(logs_records[3].severity_number, 0);
    }

    #[tokio::test]
    async fn test_set_root_column_from_arithmetic_expression_opl_parser() {
        test_set_root_column_from_arithmetic_expression::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_from_arithmetic_expression_kql_parser() {
        test_set_root_column_from_arithmetic_expression::<KqlParser>().await
    }

    async fn test_set_root_column_from_non_root_attribute<P: Parser>() {
        let logs_data = LogsData::new(vec![ResourceLogs::new(
            Resource::build().finish(),
            vec![
                ScopeLogs::new(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("a"))])
                        .finish(),
                    vec![LogRecord::build().finish()],
                ),
                ScopeLogs::new(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("b"))])
                        .finish(),
                    vec![LogRecord::build().finish()],
                ),
            ],
        )]);

        let result = exec_logs_pipeline::<P>(
            "logs | extend event_name = instrumentation_scope.attributes[\"attr1\"]",
            logs_data,
        )
        .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 1);
        assert_eq!(logs_records[0].event_name, "a");
        let logs_records = result.resource_logs[0].scope_logs[1].log_records.clone();
        assert_eq!(logs_records.len(), 1);
        assert_eq!(logs_records[0].event_name, "b");
    }

    #[tokio::test]
    async fn test_set_root_column_from_non_root_attribute_opl_parser() {
        test_set_root_column_from_non_root_attribute::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_from_non_root_attribute_kql_parser() {
        test_set_root_column_from_non_root_attribute::<KqlParser>().await
    }

    async fn test_set_root_column_rejects_invalid_type_during_planning<P: Parser>() {
        let pipeline = P::parse("logs | extend event_name = 1").unwrap().pipeline;
        let session_ctx = Pipeline::create_session_context();
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let planner = PipelinePlanner::new();
        let result = planner.plan_stages(&pipeline, &session_ctx, &otap_batch);
        match result {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("cannot assign expression of type ScalarInt to type String"),
                    "unexpected error message: {err_msg:?}"
                )
            }
            Ok(_) => {
                panic!("expected error")
            }
        };
    }

    #[tokio::test]
    async fn test_set_root_column_rejects_invalid_type_during_planning_opl_parser() {
        test_set_root_column_rejects_invalid_type_during_planning::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_rejects_invalid_type_during_planning_kql_parser() {
        test_set_root_column_rejects_invalid_type_during_planning::<OplParser>().await
    }

    async fn test_set_root_column_rejects_invalid_column_during_planning<P: Parser>() {
        let pipeline = P::parse("logs | extend bad_column = 1").unwrap().pipeline;
        let session_ctx = Pipeline::create_session_context();
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let planner = PipelinePlanner::new();
        let result = planner.plan_stages(&pipeline, &session_ctx, &otap_batch);
        match result {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("cannot assign to non-existent column 'bad_column'"),
                    "unexpected error message: {err_msg:?}"
                )
            }
            Ok(_) => {
                panic!("expected error")
            }
        };
    }

    #[tokio::test]
    async fn test_set_root_column_rejects_invalid_column_during_planning_opl_parser() {
        test_set_root_column_rejects_invalid_column_during_planning::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_rejects_invalid_column_during_planning_kql_parser() {
        test_set_root_column_rejects_invalid_column_during_planning::<KqlParser>().await
    }

    async fn test_set_root_invalid_expr_result_type_rejected_at_runtime<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("attr", AnyValue::new_int(1))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("attr", AnyValue::new_int(1))])
                .finish(),
        ]);

        let pipeline_expr = P::parse("logs | extend event_name = attributes[\"attr\"]")
            .unwrap()
            .pipeline;
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await;

        match result {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("Pipeline execution error: cannot assign expression result of type Dictionary(UInt16, Int64) to column expecting type Utf8"),
                    "unexpected error message: {err_msg:?}"
                )
            }
            Ok(_) => {
                panic!("expected error")
            }
        }
    }

    #[tokio::test]
    async fn test_set_root_invalid_expr_result_type_rejected_at_runtime_opl_parser() {
        test_set_root_invalid_expr_result_type_rejected_at_runtime::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_invalid_expr_result_type_rejected_at_runtime_kql_parser() {
        test_set_root_invalid_expr_result_type_rejected_at_runtime::<KqlParser>().await
    }

    #[tokio::test]
    async fn test_assign_empty_batch() {
        let pipeline_expr = OplParser::parse("logs | set severity_number = 1")
            .unwrap()
            .pipeline;
        let input = OtapArrowRecords::Logs(Logs::default());
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input.clone()).await.unwrap();
        assert_eq!(result, input)
    }

    // TODO - test all attribute types to root ...
}
