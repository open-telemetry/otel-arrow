// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the top-level API for executing data transformation pipelines on
//! streaming telemetry data in the OTAP columnar data format.
//!
// TODO example

use std::sync::Arc;

use async_trait::async_trait;
use data_engine_expressions::PipelineExpression;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use otap_df_pdata::OtapArrowRecords;

use crate::error::Result;
use crate::pipeline::planner::PipelinePlanner;

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
    /// Adapt to schema changes in the input data.
    ///
    /// Called before each execution. Stages should:
    /// - Check if input schemas have changed since last execution
    /// - Replan/recompile internal execution plans if needed
    /// - Return Ok(()) when ready to execute
    ///
    /// # Parameters
    /// - `otap_batch`: Can be inspected for schema changes that would alter the plan
    /// - `session_context`: For logical planning (e.g., creating DataFusion logical plans)
    /// - `config_options`: For accessing DataFusion configuration (e.g., in PhysicalOptimizerRules)
    ///   Passed separately to avoid copying on the hot path (copied once during pipeline construction)
    async fn adapt(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_context: &SessionContext,
        config_options: &ConfigOptions,
    ) -> Result<()>;

    /// Execute this stage's transformation on the current batch.
    ///
    /// The stage reads input from and writes output to the OtapContext it captured during planning.
    /// This method should be immutable - all mutable state changes happen in adapt_to_schemas.
    ///
    /// # Parameters
    /// - `task_context`: Runtime context for physical plan execution (memory pools, runtime config, etc.)
    async fn execute(
        &self,
        otap_batch: OtapArrowRecords,
        task_context: Arc<TaskContext>,
    ) -> Result<OtapArrowRecords>;
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
            // perform any pre-execution tasks necessary
            stage
                .adapt(
                    &otap_batch,
                    &pipeline.session_context,
                    pipeline.config_options.as_ref(),
                )
                .await?;

            // execute the pipeline stage
            otap_batch = stage
                .execute(otap_batch, pipeline.task_context.clone())
                .await?;
        }

        Ok(otap_batch)
    }
}
