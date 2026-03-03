// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the top-level API for executing data transformation pipelines on
//! streaming telemetry data in the OTAP columnar format.

use arrow::array::RecordBatch;
use async_trait::async_trait;
use data_engine_expressions::PipelineExpression;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::config::SessionConfig;
use datafusion::execution::context::SessionContext;
use otap_df_pdata::OtapArrowRecords;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::pipeline::planner::PipelinePlanner;
use crate::pipeline::state::ExecutionState;

mod apply_attrs;
mod assign;
mod attributes;
mod conditional;
mod expr;
mod filter;
mod functions;
mod planner;
mod project;

pub mod routing;
pub mod state;

/// A stage in the pipeline.
///
/// Used for the physical execution of one or more pipeline expressions. Stages are compiled
/// once and reused across multiple execute() calls.
///
/// Implementations may be backed by a DataFusion [`ExecutionPlan`], but this is not a strict
/// requirement. Other implementations may, for example, simply transform the [`RecordBatch`]s
/// using Arrow compute kernels.
#[async_trait(?Send)]
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
        exec_options: &mut ExecutionState,
    ) -> Result<OtapArrowRecords>;

    /// TODO comments about what this is
    async fn execute_on_attributes(
        &mut self,
        _attrs_record_batch: RecordBatch,
        _session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_options: &mut ExecutionState,
    ) -> Result<RecordBatch> {
        return Err(Error::ExecutionError {
            cause: "Unexpected invocation of pipeline stage that does not support processing attributes".into()
        });
    }
}

type BoxedPipelineStage = Box<dyn PipelineStage>;

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
    /// Create a new instance of [`PlannedPipeline`]
    pub fn new(stages: Vec<Box<dyn PipelineStage>>, session_context: SessionContext) -> Self {
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
    /// Create a new [`Pipeline`] instance that will evaluate the passed [`PipelineExpression`]
    #[must_use]
    pub const fn new(pipeline_definition: PipelineExpression) -> Self {
        Self {
            pipeline_definition,
            planned_pipeline: None,
        }
    }

    /// Create a new  [`Pipeline`] instance that contains pipeline stages that were planned
    /// externally.
    #[must_use]
    pub const fn new_planned(
        pipeline_definition: PipelineExpression,
        planned_pipeline: PlannedPipeline,
    ) -> Self {
        Self {
            pipeline_definition,
            planned_pipeline: Some(planned_pipeline),
        }
    }

    /// Execute the pipeline on a batch of telemetry data.
    ///
    /// # Arguments
    /// - `otap_batch`: The input telemetry data to process
    ///
    /// # Returns
    /// The transformed telemetry data after all stages have executed
    pub async fn execute(&mut self, otap_batch: OtapArrowRecords) -> Result<OtapArrowRecords> {
        let mut exec_state = ExecutionState::default();
        self.execute_with_state(otap_batch, &mut exec_state).await
    }

    /// Execute the pipeline on a batch of telemetry data, using the provided execution state.
    ///
    /// Any query planning happens during the first call to execute, including setting up any
    /// DataFusion SessionContext, TaskContext, etc. Subsequent calls will not have to redo
    /// the full planning, although individual stages may do light re-plannings to adapt to
    /// changing OTAP batch schemas.
    ///
    /// # Arguments
    /// - `otap_batch`: The input telemetry data to process
    /// - `exec_state`: The execution state to use for the pipeline execution
    ///
    /// # Returns
    /// The transformed telemetry data after all stages have executed
    pub async fn execute_with_state(
        &mut self,
        mut otap_batch: OtapArrowRecords,
        exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        // lazily plan the pipeline if have not already done so
        if self.planned_pipeline.is_none() {
            let session_ctx = Self::create_session_context();
            let planner = PipelinePlanner::new();
            let stages =
                planner.plan_stages(&self.pipeline_definition, &session_ctx, &otap_batch)?;
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
                    exec_state,
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

    use data_engine_expressions::PipelineExpression;

    use data_engine_parser_abstractions::Parser;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogsData;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::MetricsData;
    use otap_df_pdata::proto::opentelemetry::trace::v1::TracesData;
    use otap_df_pdata::testing::round_trip::otlp_to_otap;
    use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
    use prost::Message;

    use super::*;

    // This module just contains some helpers that are used in other tests on PipelineStage impls

    /// helper function for converting [`OtapArrowRecords`] to [`LogsData`]
    pub fn otap_to_logs_data(otap_batch: OtapArrowRecords) -> LogsData {
        let otap_payload: OtapPayload = otap_batch.into();
        let otlp_bytes: OtlpProtoBytes = otap_payload.try_into().unwrap();
        LogsData::decode(otlp_bytes.as_bytes()).unwrap()
    }

    /// helper function for converting [`OtapArrowRecords`] to [`TracesData`]
    pub fn otap_to_traces_data(otap_batch: OtapArrowRecords) -> TracesData {
        let otap_payload: OtapPayload = otap_batch.into();
        let otlp_bytes: OtlpProtoBytes = otap_payload.try_into().unwrap();
        TracesData::decode(otlp_bytes.as_bytes()).unwrap()
    }

    /// helper function for converting [`OtapArrowRecords`] to [`MetricsData`]
    pub fn otap_to_metrics_data(otap_batch: OtapArrowRecords) -> MetricsData {
        let otap_payload: OtapPayload = otap_batch.into();
        let otlp_bytes: OtlpProtoBytes = otap_payload.try_into().unwrap();
        MetricsData::decode(otlp_bytes.as_bytes()).unwrap()
    }

    pub async fn exec_logs_pipeline<P: Parser>(query: &str, logs_data: LogsData) -> LogsData {
        let parser_result = P::parse(query).unwrap();
        exec_logs_pipeline_expr(parser_result.pipeline, logs_data).await
    }

    pub async fn exec_logs_pipeline_expr(
        pipeline_expr: PipelineExpression,
        logs_data: LogsData,
    ) -> LogsData {
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await.unwrap();
        otap_to_logs_data(result)
    }
}
