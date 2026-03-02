// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for assigning the evaluation of an expression to the
//! TODO - are these the docs we want for REAL?

use std::rc::Rc;
use std::sync::Arc;

use arrow::array::RecordBatch;
use arrow::compute::take;
use arrow::datatypes::{Field, Schema};
use async_trait::async_trait;
use data_engine_expressions::{ScalarExpression, SourceScalarExpression};
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::logical_expr::ColumnarValue;
use datafusion::prelude::SessionContext;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::Logs;

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::expr::join::{JoinExec, RootToAttributesJoin};
use crate::pipeline::expr::types::{ExprLogicalType, root_field_type};
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

        // TODO - check the data type and ensure it matches the destination

        let already_aligned = eval_result.data_scope.is_scalar()
            || eval_result.data_scope.as_ref() == self.dest_scope.as_ref();

        let values = if already_aligned {
            eval_result.values.to_array(root_batch.num_rows())?
        } else {
            let DataScope::Attributes(attrs_id, _) = eval_result.data_scope.as_ref() else {
                // TODO comment on why this is unreachable
                unreachable!("unreachable")
            };

            // TODO comment on what this code does
            let join_exec = RootToAttributesJoin::new(attrs_id.clone());
            let root_as_join_arg = PhysicalExprEvalResult::new(
                ColumnarValue::Scalar(ScalarValue::Null), // placeholder,
                self.dest_scope.clone(),
                root_batch,
            );
            let vals_take_indices = join_exec.rows_to_take(
                &root_as_join_arg,
                &eval_result,
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )?;

            match eval_result.values {
                ColumnarValue::Array(arr) => take(&arr, &vals_take_indices, None)?,
                ColumnarValue::Scalar(_) => {
                    todo!("error or unreachable here") // err probably better
                }
            }
        };

        // TODO - need to attempt to cast to dictionary

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
        ExprLogicalType::Binary | ExprLogicalType::Boolean | ExprLogicalType::String => {
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
        ExprLogicalType::AnyValue => match source_type {
            ExprLogicalType::AnyValue
            | ExprLogicalType::AnyValueNumeric
            | ExprLogicalType::ScalarInt
            | ExprLogicalType::Binary
            | ExprLogicalType::Boolean
            | ExprLogicalType::String
            | ExprLogicalType::Float64
            | ExprLogicalType::Int64 => true,
            _ => false,
        },
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

#[cfg(test)]
mod test {
    use data_engine_kql_parser::Parser;
    use otap_df_opl::parser::OplParser;
    use otap_df_pdata::{
        OtapArrowRecords,
        otap::Logs,
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                arrow::v1::ArrowPayloadType,
                common::v1::{AnyValue, InstrumentationScope, KeyValue},
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                resource::v1::Resource,
            },
        },
        testing::round_trip::{otlp_to_otap, to_logs_data},
    };

    use crate::pipeline::{Pipeline, planner::PipelinePlanner, test::exec_logs_pipeline};

    fn gen_logs_test_data() -> LogsData {
        LogsData::new(vec![ResourceLogs::new(
            Resource::build()
                .attributes(vec![
                    KeyValue::new("xr1", AnyValue::new_string("a")),
                    KeyValue::new("xr2", AnyValue::new_string("a")),
                ])
                .finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .attributes(vec![
                        KeyValue::new("xs1", AnyValue::new_string("a")),
                        KeyValue::new("xs2", AnyValue::new_string("a")),
                    ])
                    .finish(),
                vec![],
            )],
        )])
    }

    #[tokio::test]
    async fn test_insert_root_column_from_scalar() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().finish(),
            LogRecord::build().finish(),
        ]);
        let result =
            exec_logs_pipeline::<OplParser>("logs | set severity_text = \"ERROR\"", logs_data)
                .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 2);
        for logs_record in logs_records {
            assert_eq!(logs_record.severity_text, "ERROR");
        }
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_scalar() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().finish(),
        ]);
        let result =
            exec_logs_pipeline::<OplParser>("logs | set severity_text = \"ERROR\"", logs_data)
                .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 2);
        for logs_record in logs_records {
            assert_eq!(logs_record.severity_text, "ERROR");
        }
    }

    #[tokio::test]
    async fn test_insert_root_column_from_other_column() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().severity_text("DEBUG").finish(),
        ]);

        // kind of a silly example, but just need two cols that have the same type for the test
        let result =
            exec_logs_pipeline::<OplParser>("logs | set event_name = severity_text", logs_data)
                .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 2);
        assert_eq!(logs_records[0].event_name, "INFO");
        assert_eq!(logs_records[1].event_name, "DEBUG");
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_other_column() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .severity_text("INFO")
                .event_name("event1")
                .finish(),
            LogRecord::build().severity_text("DEBUG").finish(),
        ]);

        // kind of a silly example, but just need two cols that have the same type for the test
        let result =
            exec_logs_pipeline::<OplParser>("logs | set event_name = severity_text", logs_data)
                .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 2);
        assert_eq!(logs_records[0].event_name, "INFO");
        assert_eq!(logs_records[1].event_name, "DEBUG");
    }

    #[tokio::test]
    async fn test_set_root_column_from_attribute() {
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

        let result = exec_logs_pipeline::<OplParser>(
            "logs | set event_name = attributes[\"event\"]",
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
    async fn test_set_root_column_from_non_root_attribute() {
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

        let result = exec_logs_pipeline::<OplParser>(
            "logs | set event_name = instrumentation_scope.attributes[\"attr1\"]",
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
    async fn test_set_root_column_rejects_invalid_type_during_planning() {
        let pipeline = OplParser::parse("logs | set event_name = 1")
            .unwrap()
            .pipeline;
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
    async fn test_set_root_column_rejects_invalid_column_during_planning() {
        let pipeline = OplParser::parse("logs | set bad_column = 1")
            .unwrap()
            .pipeline;
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

    // TODO test on empty batch
}
