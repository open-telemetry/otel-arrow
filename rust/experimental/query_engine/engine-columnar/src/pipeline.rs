// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the top-level API for executing data transformation pipelines on
//! streaming telemetry data in the OTAP columnar data format.
//!
// TODO example

use std::sync::Arc;

use arrow::compute::concat_batches;
use async_trait::async_trait;
use data_engine_expressions::PipelineExpression;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use datafusion::physical_plan::common::collect;
use datafusion::physical_plan::{ExecutionPlan, execute_stream};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::error::Result;
use crate::pipeline::planner::PipelinePlanner;
use crate::table::RecordBatchPartitionStream;

mod planner;

/// A stage in the OPL pipeline.
///
/// Stages are compiled once and reused across multiple execute() calls. They follow a two-phase
/// execution model:
///
/// 1. **Adaptation Phase** (`adapt_to_schemas`): Called before each execution to handle schema
///    changes. Stages can replan/recompile their internal execution plans if needed.
///
/// 2. **Execution Phase** (`execute`): Runs the actual data transformation on the current batch.
///
/// Design rationale:
/// - SessionContext/ConfigOptions are passed to adapt_to_schemas for logical planning and physical
///   plan optimization
/// - TaskContext is passed to execute for physical execution
///
/// Various implementations can be created such as those backed by a DataFusion plan, and others
/// which simply transform the [`RecordBatch`]s using arrow compute kernels
#[async_trait]
pub trait PipelineStage {
    /// Execute this stage's transformation on the current batch.
    ///
    /// The stage reads input from and writes output to the OtapContext it captured during planning.
    /// This method should be immutable - all mutable state changes happen in adapt_to_schemas.
    ///
    /// # Parameters
    /// - `task_context`: Runtime context for physical plan execution (memory pools, runtime config, etc.)
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
    ) -> Result<OtapArrowRecords>;
}

pub struct DataFusionPipelineStage {
    /// TODO comment
    payload_type: ArrowPayloadType,

    /// TODO comment
    record_batch_stream: Arc<RecordBatchPartitionStream>,

    /// TODO comment
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
                // TODO need to handle if we expect there to be a record batch and it's None
                todo!()
            }
        };

        // TODO we also need to validate the schema here to ensure it's the same as the expected
        // schema. If not, we may need to make some modifications to the plan

        // update the record batch stream to produce the current batch
        self.record_batch_stream.update_batch(rb.clone());

        // TODO no unwrap
        let stream = execute_stream(self.execution_plan.clone(), task_context).unwrap();
        let batches = collect(stream).await.unwrap();

        match batches.len() {
            0 => {
                // TODO
                todo!("no batches are returned from DatafusionPipelineStage execute")
            }
            1 => {
                let new_rb = batches.into_iter().next().expect("batches not empty");
                otap_batch.set(self.payload_type, new_rb)
            }
            _ => {
                let new_rb = concat_batches(batches[0].schema_ref(), &batches).unwrap();
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

    /// DataFusion session context for logical planning during adaptation
    session_context: SessionContext,

    /// Configuration options, copied once during construction to avoid repeated copies
    /// on the hot path (execution). Passed by reference to stages during adapt_to_schemas.
    config_options: Arc<ConfigOptions>,

    /// Task context for physical execution, created once and reused
    task_context: Arc<TaskContext>,
}

impl PlannedPipeline {
    fn new(stages: Vec<Box<dyn PipelineStage>>, session_context: SessionContext) -> Self {
        // Extract TaskContext and ConfigOptions once during construction (cold path)
        // These will be passed by reference during execution (hot path)
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
    /// the expression tree (AST) defining this pipeline
    pipeline_definition: PipelineExpression,

    /// the compiled pipeline - this is initialized lazily on the first call to execute
    /// as some stages may need to inspect the schema of the data for planning
    planned_pipeline: Option<PlannedPipeline>,
}

impl Pipeline {
    /// Execute the pipeline on a batch of telemetry data.
    ///
    /// # Execution Flow
    ///
    /// 1. **First Execution** (Compilation):
    ///    - Create SessionContext, TaskContext, ConfigOptions
    ///    - Plan stages using schemas from the input batch
    ///    - Cache the compiled pipeline
    ///
    /// 2. **Every Execution** (including first):
    ///    - Load input batch into the shared context
    ///    - **Adaptation Phase**: Call adapt_to_schemas() on each stage
    ///      - Stages check if schemas changed and replan if needed
    ///    - **Execution Phase**: Call execute() on each stage
    ///      - Stages transform data in the context
    ///    - Extract and return the final result
    ///
    /// # Performance Characteristics
    ///
    /// - **Cold path (first execution)**: Allocates contexts, plans stages
    /// - **Hot path (subsequent executions)**:
    ///   - Zero allocation if schemas unchanged (stages skip replanning)
    ///   - Per-stage replanning if schemas change
    ///   - ConfigOptions passed by reference (copied once during compilation)
    ///
    /// # Arguments
    /// - `otap_batch`: The input telemetry data to process
    ///
    /// # Returns
    /// The transformed telemetry data after all stages have executed
    pub async fn execute(&mut self, mut otap_batch: OtapArrowRecords) -> Result<OtapArrowRecords> {
        // Lazy compilation: compile on first execution
        if self.planned_pipeline.is_none() {
            // TODO we need to set up the session context correctly here
            let session_context = SessionContext::new();

            let mut planner = PipelinePlanner::new();
            let stages =
                planner.plan_stages(&session_context, &self.pipeline_definition, &otap_batch)?;
            self.planned_pipeline = Some(PlannedPipeline::new(stages, session_context));
        }

        // safety: we've already planned the pipeline
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
}

#[cfg(test)]
mod test {
    use arrow::array::{RecordBatch, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use data_engine_expressions::PipelineExpression;
    use datafusion::prelude::{SessionConfig, col, lit};
    use datafusion::{catalog::streaming::StreamingTable, prelude::SessionContext};
    use otap_df_pdata::OtapArrowRecords;
    use otap_df_pdata::otap::Logs;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::schema::consts;
    use std::sync::Arc;

    use crate::pipeline::{DataFusionPipelineStage, Pipeline, PlannedPipeline};
    use crate::table::RecordBatchPartitionStream;

    #[tokio::test]
    async fn test_pipeline_execute() {
        // TODO - add comments about why the test is written this way

        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
            Field::new(consts::EVENT_NAME, DataType::Utf8, true),
        ]));

        let rb_stream = Arc::new(RecordBatchPartitionStream::new(RecordBatch::new_empty(
            schema.clone(),
        )));
        let table = StreamingTable::try_new(schema.clone(), vec![rb_stream.clone()]).unwrap();

        let session_config = SessionConfig::new()
            // since we're always executing in a single threaded runtime, it doesn't make sense
            // to spawn repartition tasks and run things like join and filtering in parallel
            .with_target_partitions(1)
            .with_repartition_joins(false)
            .with_repartition_file_scans(false)
            .with_repartition_windows(false)
            .with_repartition_aggregations(false)
            .with_repartition_sorts(false);
        let ctx = SessionContext::new_with_config(session_config);
        ctx.register_table("logs", Arc::new(table)).unwrap();

        let df = ctx
            .table("logs")
            .await
            .unwrap()
            .filter(col("severity_text").eq(lit("ERROR")))
            .unwrap();

        let state = ctx.state();
        let logical_plan = state.optimize(df.logical_plan()).unwrap();
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

        let logs_rb1 = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from_iter_values(["ERROR", "INFO", "WARN"])),
                Arc::new(StringArray::from_iter_values(["1", "2", "3"])),
            ],
        )
        .unwrap();
        let mut otap_batch1 = OtapArrowRecords::Logs(Logs::default());
        otap_batch1.set(ArrowPayloadType::Logs, logs_rb1);

        let logs_rb2 = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from_iter_values(["ERROR", "ERROR", "WARN"])),
                Arc::new(StringArray::from_iter_values(["4", "5", "6"])),
            ],
        )
        .unwrap();
        let mut otap_batch2 = OtapArrowRecords::Logs(Logs::default());
        otap_batch2.set(ArrowPayloadType::Logs, logs_rb2);

        for otap_batch in [otap_batch1, otap_batch2] {
            let result = pipeline.execute(otap_batch).await.unwrap();
            let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();
            arrow::util::pretty::print_batches(&[logs_rb.clone()]).unwrap();
        }
    }
}
