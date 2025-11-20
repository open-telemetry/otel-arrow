// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the top-level API for executing data transformation pipelines on
//! streaming telemetry data in the OTAP columnar format.

use std::sync::Arc;

use arrow::compute::concat_batches;
use async_trait::async_trait;
use data_engine_expressions::PipelineExpression;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::config::SessionConfig;
use datafusion::execution::context::SessionContext;
use datafusion::physical_plan::common::collect;
use datafusion::physical_plan::streaming::PartitionStream;
use datafusion::physical_plan::{ExecutionPlan, execute_stream};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::error::{Error, Result};
use crate::pipeline::planner::PipelinePlanner;
use crate::table::RecordBatchPartitionStream;

mod planner;

/// A stage in the pipeline.
///
/// Used for the physical execution of one or more pipeline expressions. Stages are compiled
/// once and reused across multiple execute() calls.
///
/// Implementations may be backed by a DataFusion [`ExecutionPlan`], but this is not a strict
/// requirement. Other implementations may, for example, simply transform the [`RecordBatch`]s
/// using Arrow compute kernels.
#[async_trait]
pub trait PipelineStage {
    /// Execute this stage's transformation on the current OTAP batch.
    ///
    /// The implementation may need to inspect the batch to determine if the schema has changed,
    /// or if some optional [`RecordBatch`] for some payload type has changed presence.
    ///
    /// In the case of changes, some light-weight replanning may be required.
    /// - [`SessionContext`] is available for logical planning
    /// - [`ConfigOptions`] are passed in case the re-planning will involve
    ///   [`PhysicalOptimizerRule`s](datafusion::physical_optimizer::optimizer::PhysicalOptimizerRule)
    ///
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
    ) -> Result<OtapArrowRecords>;
}

pub struct DataFusionPipelineStage {
    /// The payload in the OtapArrowRecords this stage reads from and writes to.
    payload_type: ArrowPayloadType,

    /// Input source for the execution plan. Updated with new data before each execution
    /// to inject the current batch for the payload type into DataFusion's streaming model.
    record_batch_stream: Arc<RecordBatchPartitionStream>,

    /// The DataFusion query plan to execute. Reads from record_batch_stream and produces
    /// the transformed output batch.
    execution_plan: Arc<dyn ExecutionPlan>,
}

#[async_trait]
impl PipelineStage for DataFusionPipelineStage {
    async fn execute(
        &mut self,
        mut otap_batch: OtapArrowRecords,
        _session_context: &SessionContext,
        _config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
    ) -> Result<OtapArrowRecords> {
        let rb = match otap_batch.get(self.payload_type) {
            Some(rb) => rb,
            None => {
                // TODO eventually we'll need to handle when an optional RecordBatch is no longer
                // present in the OTAP batch. How this is handled depends on the type of operation.
                // For example, if we're filtering then no action is required because all the
                // records have already been filtered out. By contrast, if we're inserting an
                // attribute we may need to create a new empty attributes RecordBatch
                return Err(Error::NotYetSupportedError {
                    message: "missing RecordBatch for payload type".into(),
                });
            }
        };

        // validate that the schema hasn't changed
        if rb.schema_ref() != self.record_batch_stream.schema() {
            // TODO we may need to make slight adjustments to the plan in cases where the order of
            // the columns have changed, the presence of optional columns has changed, or if some
            // columns have changed types.
            //
            // How we'll handle this depends on the nature of the schema change, and the query plan.
            // For example if columns are just changing order, we could add a `ProjectionExec`.
            // By contrast if we're filtering on attributes that are no longer present, we could
            // possibly optimize the query plan into a simple [`EmptyExec`].
            return Err(Error::NotYetSupportedError {
                message: "adapting plan for RecordBatch schema change".into(),
            });
        }

        // update the record batch stream to produce the current batch
        self.record_batch_stream.update_batch(rb.clone());

        // execute the physical plan
        let stream = execute_stream(self.execution_plan.clone(), task_context)?;
        let batches = collect(stream).await?;

        // update the OTAP batch
        match batches.len() {
            0 => {
                // TODO: handle this properly. This would happen if say, a filtering query returns
                // no records. The logic we should use is:
                // - for non-root payload type to `None` in `OtapArrowRecords` and also maybe drop
                //   the ID column on the parent record batch.
                // - for root payload type, we would just return an empty OtapArrowRecords
                return Err(Error::NotYetSupportedError {
                    message: "queries returning empty result set".into(),
                });
            }
            1 => {
                let new_rb = batches.into_iter().next().expect("batches not empty");
                otap_batch.set(self.payload_type, new_rb)
            }
            _ => {
                let new_rb = concat_batches(batches[0].schema_ref(), &batches)?;
                otap_batch.set(self.payload_type, new_rb)
            }
        };

        Ok(otap_batch)
    }
}

/// A compiled pipeline ready for execution. Contains all the state needed for execution
/// and adapting the pipeline between OTAP batches
pub struct PlannedPipeline {
    /// the stages of the compiled pipeline
    stages: Vec<Box<dyn PipelineStage>>,

    /// DataFusion session context for logical planning during plan adaptation
    session_context: SessionContext,

    /// Configuration options, for physical plan optimizations during plan adaptation
    config_options: Arc<ConfigOptions>,

    /// Task context for physical execution
    task_context: Arc<TaskContext>,
}

impl PlannedPipeline {
    fn new(stages: Vec<Box<dyn PipelineStage>>, session_context: SessionContext) -> Self {
        let state = session_context.state();
        let task_context = Arc::new(TaskContext::from(&state));
        let config_options = session_context.copied_config().options().clone();

        Self {
            stages,
            session_context,
            task_context,
            config_options,
        }
    }
}

/// The main entrypoint for transform pipeline execution
pub struct Pipeline {
    /// The expression tree (AST) defining this pipeline
    pipeline_definition: PipelineExpression,

    /// The compiled pipeline - this is initialized lazily on the first call to execute
    /// as some stages may need to inspect the schema of the data for planning
    planned_pipeline: Option<PlannedPipeline>,
}

impl Pipeline {
    /// Execute the pipeline on a batch of telemetry data.
    ///
    /// Any query planning happens during the first call to execute, including setting up any
    /// DataFusion SessionContext, TaskContext, etc. Subsequent calls will not have to redo
    /// the full planning, although individual stages may do light re-plannings to adapt to
    /// changing OTAP batch schemas.
    ///
    /// # Arguments
    /// - `otap_batch`: The input telemetry data to process
    ///
    /// # Returns
    /// The transformed telemetry data after all stages have executed
    pub async fn execute(&mut self, mut otap_batch: OtapArrowRecords) -> Result<OtapArrowRecords> {
        // lazily plan the pipeline if have not already done so
        if self.planned_pipeline.is_none() {
            let session_ctx = Self::create_session_context();
            let mut planner = PipelinePlanner::new();
            let stages =
                planner.plan_stages(&session_ctx, &self.pipeline_definition, &otap_batch)?;
            self.planned_pipeline = Some(PlannedPipeline::new(stages, session_ctx));
        }

        // safety: we've already planned the pipeline, so expect is safe
        let pipeline = self.planned_pipeline.as_mut().expect("pipeline is planned");

        // Execution phase: run the transformations
        for stage in &mut pipeline.stages {
            // execute the pipeline stage
            otap_batch = stage
                .execute(
                    otap_batch,
                    &pipeline.session_context,
                    pipeline.config_options.as_ref(),
                    pipeline.task_context.clone(),
                )
                .await?;
        }

        Ok(otap_batch)
    }

    /// setup a new session context with the configuration for planning and executing datafusion
    /// pipeline stages.
    fn create_session_context() -> SessionContext {
        let session_config = SessionConfig::new()
            // since we're typically executing in a single threaded runtime, it doesn't make sense
            // to spawn repartition tasks and run things like join and filtering in parallel
            .with_target_partitions(1)
            .with_repartition_joins(false)
            .with_repartition_file_scans(false)
            .with_repartition_windows(false)
            .with_repartition_aggregations(false)
            .with_repartition_sorts(false);

        SessionContext::new_with_config(session_config)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use data_engine_expressions::PipelineExpression;
    use datafusion::catalog::streaming::StreamingTable;
    use datafusion::logical_expr::{col, lit};
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otap_df_pdata::testing::round_trip::encode_logs;

    use super::*;

    #[tokio::test]
    async fn test_pipeline_execute() {
        // TODO eventually we might want to drive this test from a pipeline expression, which we
        // can do once we have the query planning implemented. For now, we are manually creating
        // the `PlannedPipeline` and its `PipelineStage`s and any additional datafusion context
        // they need

        let otap_batch1 = encode_logs(&LogsData::new(vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![
                    LogRecord {
                        severity_text: "ERROR".into(),
                        event_name: "1".into(),
                        ..Default::default()
                    },
                    LogRecord {
                        severity_text: "ERROR".into(),
                        event_name: "2".into(),
                        ..Default::default()
                    },
                    LogRecord {
                        severity_text: "WARN".into(),
                        event_name: "3".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }]));

        let otap_batch2 = encode_logs(&LogsData::new(vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![
                    LogRecord {
                        severity_text: "DEBUG".into(),
                        event_name: "4".into(),
                        ..Default::default()
                    },
                    LogRecord {
                        severity_text: "TRACE".into(),
                        event_name: "5".into(),
                        ..Default::default()
                    },
                    LogRecord {
                        severity_text: "ERROR".into(),
                        event_name: "6".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }]));

        let schema = otap_batch1.get(ArrowPayloadType::Logs).unwrap().schema();
        let rb_stream = Arc::new(RecordBatchPartitionStream::new(schema.clone()));
        let table = StreamingTable::try_new(schema.clone(), vec![rb_stream.clone()]).unwrap();

        let ctx = Pipeline::create_session_context();
        ctx.register_table("logs", Arc::new(table)).unwrap();
        let query = ctx
            .table("logs")
            .await
            .unwrap()
            .filter(col("severity_text").eq(lit("ERROR")))
            .unwrap();

        let state = ctx.state();
        let logical_plan = state.optimize(query.logical_plan()).unwrap();
        let physical_plan = state.create_physical_plan(&logical_plan).await.unwrap();

        let stage = DataFusionPipelineStage {
            payload_type: ArrowPayloadType::Logs,
            record_batch_stream: rb_stream,
            execution_plan: physical_plan,
        };

        let planned_pipeline = PlannedPipeline::new(vec![Box::new(stage)], ctx);

        let mut pipeline = Pipeline {
            pipeline_definition: PipelineExpression::default(),
            planned_pipeline: Some(planned_pipeline),
        };

        let input1_logs = otap_batch1.get(ArrowPayloadType::Logs).unwrap().clone();
        let result1 = pipeline.execute(otap_batch1).await.unwrap();
        let result1_logs = result1.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(result1_logs, &input1_logs.slice(0, 2));

        let input2_logs = otap_batch2.get(ArrowPayloadType::Logs).unwrap().clone();
        let result2 = pipeline.execute(otap_batch2).await.unwrap();
        let result2_logs = result2.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(result2_logs, &input2_logs.slice(2, 1));
    }
}
