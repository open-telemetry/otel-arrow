// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for assigning the evaluation of an expression to the
//! TODO - are these the docs we want for REAL?

use std::rc::Rc;
use std::sync::Arc;

use arrow::array::{ArrayRef, DictionaryArray, RecordBatch, UInt8Array, cast};
use arrow::compute::{cast, take};
use arrow::datatypes::{DataType, Field, Schema, UInt16Type};
use async_trait::async_trait;
use data_engine_expressions::{ScalarExpression, SourceScalarExpression};
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::functions_aggregate::count::Count;
use datafusion::logical_expr::function::AccumulatorArgs;
use datafusion::logical_expr::{Accumulator, AggregateUDFImpl, ColumnarValue};
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

pub(crate) struct AssignPipelineStage {
    ///
    dest_column: ColumnAccessor,

    dest_scope: Rc<DataScope>,

    /// expression that will produce the data to be assigned to the destination
    source: ScopedPhysicalExpr,
}

impl AssignPipelineStage {
    pub fn try_new(dest: &SourceScalarExpression, source: &ScalarExpression) -> Result<Self> {
        let dest_column = ColumnAccessor::try_from(dest.get_value_accessor())?;

        let logical_planner = ExprLogicalPlanner::default();
        let source_logical_plan = logical_planner.plan_scalar_expr(source)?;

        validate_assign(&dest_column, &source_logical_plan)?;

        let physical_planner = ExprPhysicalPlanner::default();
        let physical_expr = physical_planner.plan(source_logical_plan)?;

        Ok(Self {
            dest_scope: Rc::new(dest_column.clone().into()),
            dest_column: dest_column,
            source: physical_expr,
        })
    }

    fn assign_to_root(
        &self,
        mut otap_batch: OtapArrowRecords,
        eval_result: PhysicalExprEvalResult,
        target_col_name: &str,
    ) -> Result<OtapArrowRecords> {
        let root_batch = match otap_batch.root_record_batch() {
            Some(rb) => rb,
            None => {
                // nothing to do
                return Ok(otap_batch);
            }
        };

        // check that the result type of the expr eval can be assigned to this field
        // TODO explain why we can expect here
        let expected_column_type = root_field_type(target_col_name)
            .expect("TODO")
            .datatype()
            .expect("TODO");
        let eval_result_column_type = eval_result.values.data_type();
        let column_supports_dict_encoding = root_field_supports_dict_encoding(target_col_name);
        let mut type_compatible = expected_column_type == eval_result_column_type;
        if !type_compatible && column_supports_dict_encoding {
            // if the field can be dictionary encoded, check if the eval result is also dictionary
            // encoded. If so, we're allowed to make the assignment
            if let DataType::Dictionary(_, val) = &eval_result_column_type {
                if val.as_ref() == &expected_column_type {
                    type_compatible = true
                }
            }
        }

        if !type_compatible {
            return Err(Error::ExecutionError {
                cause: format!(
                    "cannot assign expression result of type {:?} to column expecting type {:?}",
                    eval_result_column_type, expected_column_type
                ),
            });
        }

        let mut values = eval_result_to_array(
            &eval_result.values,
            column_supports_dict_encoding,
            root_batch.num_rows(),
        )?;

        // convert the result values into an array that has the same row order as the root batch.
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
                    ColumnarValue::Scalar(ScalarValue::Null), // placeholder,
                    self.dest_scope.clone(),
                    root_batch,
                ),
                &eval_result,
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )?;

            values = take(&values, &vals_take_indices, None)?;
        };

        let mut columns = root_batch.columns().to_vec();
        let schema = root_batch.schema();
        let fields = schema.fields();
        let target_col_index = fields.find(target_col_name).map(|(position, _)| position);
        let mut fields = fields.to_vec();

        if let Some(target_col_index) = target_col_index {
            // replace field
            // TODO - this might not be needed if the datatype didn't change ...
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
            // Note: assuming if the column is optional, it is nullable
            fields.push(Arc::new(Field::new(
                target_col_name,
                values.data_type().clone(),
                true,
            )));
            columns.push(values)
        }

        // TODO comment about why we can expect here
        let new_root_batch =
            RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).expect("TODO");
        let root_payload_type = otap_batch.root_payload_type();
        otap_batch.set(root_payload_type, new_root_batch);

        Ok(otap_batch)
    }
}

fn validate_assign(
    dest_column: &ColumnAccessor,
    source_logical_plan: &ScopedLogicalExpr,
) -> Result<()> {
    match dest_column {
        ColumnAccessor::ColumnName(col_name) => {
            let dest_type =
                root_field_type(col_name).ok_or_else(|| Error::InvalidPipelineError {
                    cause: format!("cannot assign to non-existent column '{col_name}'"),
                    query_location: None,
                })?;
            let source_type = &source_logical_plan.expr_type;
            if !can_assign_type(&dest_type, source_type) {
                return Err(Error::InvalidPipelineError {
                    cause: format!(
                        "cannot assign expression of type {source_type:?} to type {dest_type:?}"
                    ),
                    query_location: None, // TODO it'd be good to put a location here
                });
            }
        }
        _ => {
            // TODO check 1:many
            todo!()
        }
    }

    return Ok(());
}

// TODO put more unit tests for this
fn can_assign_type(dest_type: &ExprLogicalType, source_type: &ExprLogicalType) -> bool {
    if dest_type == source_type {
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

        // TODO - leave this for later when we implement other assignments?
        // ExprLogicalType::AnyValue => match source_type {
        //     ExprLogicalType::AnyValue
        //     | ExprLogicalType::AnyValueNumeric
        //     | ExprLogicalType::ScalarInt
        //     | ExprLogicalType::Boolean
        //     | ExprLogicalType::String
        //     | ExprLogicalType::Float64
        //     | ExprLogicalType::Int64 => true,
        //     _ => false,
        // },
        _ => false,
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
                _ => {
                    todo!()
                }
            },
            None => {
                todo!() // remove data
            }
        }
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
                let dict_values = scalar_val.to_array()?;
                let dict_keys = UInt8Array::from_iter_values(std::iter::repeat_n(0, dest_num_rows));
                Ok(Arc::new(DictionaryArray::new(dict_keys, dict_values)))
            } else {
                Ok(scalar_val.to_array_of_size(dest_num_rows)?)
            }
        }
        ColumnarValue::Array(array_vals) => {
            if accept_dict_encoding {
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
                                        Box::new(k.as_ref().clone()),
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

    // TODO - test on empty batch
    // TODO - test all attribute types to root ...
}
