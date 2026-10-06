// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the top-level API for executing data transformation pipelines on
//! streaming telemetry data in the OTAP columnar format.

use arrow::array::RecordBatch;
use arrow::compute::concat_batches;
use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::config::SessionConfig;
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::lit;
use datafusion::physical_plan::common::collect;
use datafusion::physical_plan::streaming::PartitionStream;
use datafusion::physical_plan::{ExecutionPlan, execute_stream};
use datafusion::prelude::col;
use otel_arrow_contrib_data_engine_expressions::PipelineExpression;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::error::Error as PdataError;
use otel_arrow_dfe_pdata::otlp::metrics::MetricType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::pipeline::conditional::{ConditionalPipelineStage, ConditionalPipelineStageBranch};
use crate::pipeline::expr::{DataScope, LeafEval, RecordScope, ScopedExpr};
use crate::pipeline::planner::{PipelinePlanner, RecordType};
use crate::pipeline::state::ExecutionState;
use crate::table::RecordBatchPartitionStream;

mod apply;
mod assign;
mod attributes;
mod concat;
mod conditional;
mod expr;
mod filter;
mod fork;
mod functions;
pub(crate) mod id_mask;
mod planner;
mod project;
mod scale_metric;

pub mod partition;
pub mod routing;
pub mod state;

// Re-export planner types that callers need for configuring pipeline options.
pub use planner::{MetricTypeContext, SignalContext, SignalKind};

#[cfg(feature = "bench")]
#[doc(hidden)]
pub mod bench_support {
    pub mod join {
        pub use crate::pipeline::expr::join::bench_support::*;
    }
}

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
        exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords>;

    /// Execute this stage of the pipeline on a [`RecordBatch`] containing a set of attributes.
    ///
    /// Not all pipeline stages are required to support this, and the default is that it is not
    /// supported. If an type chooses to implement this, it should also implement
    /// `supports_exec_on_attributes` and return `true`.
    async fn execute_on_attributes(
        &mut self,
        _attrs_record_batch: RecordBatch,
        _session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_state: &mut ExecutionState,
    ) -> Result<RecordBatch> {
        return Err(Error::ExecutionError {
            cause: "Unexpected invocation of pipeline stage that does not support processing attributes".into()
        });
    }

    /// Execute this stage on the data points of the metric.
    ///
    /// When the pipeline stage is executed via this method call, it should perform its operation
    /// as if the "root" of any expression is the metric data points record batch. It may need to
    /// perform multiple evaluations on each of the various metric data point types.
    async fn execute_on_metric_data_points(
        &mut self,
        _otap_batch: OtapArrowRecords,
        _session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        return Err(Error::ExecutionError {
            cause: "Unexpected invocation of pipeline stage that does not support execution on metric data points".into()
         });
    }

    /// Returns a flag indicating that this stage of the pipeline can execute where the passed
    /// type of record would be the root of the expression.
    ///
    /// This is used by the planner to reject invalid/unsupported operations applied to some type
    /// of record. For example, if some pipeline stage returns true/false when record type is
    /// the `Attributes` variant, the planner will either accept/reject this type of pipeline stage
    /// being used in an operation call like `apply attributes { ... }`
    ///
    /// If an implementation overrides this to return `true` for `RecordType::Attributes`,
    /// should also implement `execute_on_attributes`. Likewise for `RecordType::DataPoint`
    /// and `execute_on_metric_data_points`.
    fn supports_exec_on(&self, record_type: &RecordType) -> bool {
        matches!(record_type, RecordType::Signal(_))
    }

    /// When pipeline stages execute within the context of a conditional branch, they will only see
    /// the batch that is local to that branch. However, there may be cases where some global state
    /// may need to be maintained across branches. This method provides an opportunity to
    /// initialize the state. It will be called with the OTAP batch that is the input to the
    /// conditional pipeline stage.
    fn init_state_for_conditional_branch(
        &mut self,
        _otap_batch: &OtapArrowRecords,
        _exec_state: &mut ExecutionState,
    ) -> Result<()> {
        // default is to do nothing
        Ok(())
    }

    /// Implementation of this trait method can be used to clear any state that was added in
    /// [`init_state_for_conditional_branch`]
    fn clear_state_for_conditional_branch(
        &mut self,
        _exec_state: &mut ExecutionState,
    ) -> Result<()> {
        // default is to do nothing
        Ok(())
    }
}

type BoxedPipelineStage = Box<dyn PipelineStage>;

/// Implementation of pipeline that executes a datafusion `ExecutionPlan` on the record batch
/// associated with the payload type
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

#[async_trait(?Send)]
impl PipelineStage for DataFusionPipelineStage {
    async fn execute(
        &mut self,
        mut otap_batch: OtapArrowRecords,
        _session_context: &SessionContext,
        _config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
        _execution_options: &mut ExecutionState,
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
                otap_batch.set(self.payload_type, new_rb)?;
            }
            _ => {
                let new_rb = concat_batches(batches[0].schema_ref(), &batches)?;
                otap_batch.set(self.payload_type, new_rb)?;
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
    /// Create a new instance of [`PlannedPipeline`]
    #[must_use]
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

/// Options for pipeline
#[derive(Clone)]
pub struct PipelineOptions {
    /// Whether to treat attribute key match as case sensitive during filtering stages
    pub filter_attribute_keys_case_sensitive: bool,

    /// Which signal types the pipeline may encounter
    pub signal_context: SignalContext,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            filter_attribute_keys_case_sensitive: true,
            signal_context: SignalContext::All,
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

    /// Options controlling planning and execution of transformation pipeline
    options: PipelineOptions,
}

impl Pipeline {
    /// Create a new [`Pipeline`] instance that will evaluate the passed [`PipelineExpression`].
    ///
    /// # Errors
    ///
    /// Returns an error if the signal type cannot be determined from the query source.
    pub fn try_new(pipeline_definition: PipelineExpression) -> Result<Self> {
        let signal_context = SignalContext::try_infer(&pipeline_definition)?;
        let options = PipelineOptions {
            signal_context,
            ..Default::default()
        };
        Ok(Self::new_with_options(pipeline_definition, options))
    }

    /// Create a new [`Pipeline`] instance that will evaluate the passed [`PipelineExpression`]
    /// with the specified options
    #[must_use]
    pub const fn new_with_options(
        pipeline_definition: PipelineExpression,
        options: PipelineOptions,
    ) -> Self {
        Self {
            pipeline_definition,
            planned_pipeline: None,
            options,
        }
    }

    /// Returns true if this pipeline should process the given signal type.
    ///
    /// This allows callers to gate execution at the batch level before calling
    /// [`Pipeline::execute`] or [`Pipeline::execute_with_state`].
    #[must_use]
    pub fn accepts_signal_type(&self, signal_type: SignalType) -> bool {
        match &self.options.signal_context {
            SignalContext::All => true,
            SignalContext::Single(kind) => matches!(
                (kind, signal_type),
                (SignalKind::Logs, SignalType::Logs)
                    | (SignalKind::Metrics(_), SignalType::Metrics)
                    | (SignalKind::Traces, SignalType::Traces)
            ),
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
        // Reject batches with incompatible signal types
        let batch_signal_type = Self::batch_signal_type(&otap_batch);
        if !self.accepts_signal_type(batch_signal_type) {
            let expected = match &self.options.signal_context {
                SignalContext::Single(SignalKind::Logs) => SignalType::Logs,
                SignalContext::Single(SignalKind::Metrics(_)) => SignalType::Metrics,
                SignalContext::Single(SignalKind::Traces) => SignalType::Traces,
                // safety: All accepts everything -- unreachable since accepts_signal_type returns
                // true for SignalContext::All
                SignalContext::All => unreachable!(),
            };
            return Err(PdataError::UnexpectedSignalType {
                found: batch_signal_type,
                expected,
            }
            .into());
        }

        // lazily plan the pipeline if have not already done so
        if self.planned_pipeline.is_none() {
            let session_ctx = Self::create_session_context();
            let planner = PipelinePlanner::new_with_record_type(RecordType::Signal(
                self.options.signal_context.clone(),
            ))
            .with_filter_attribute_keys_case_sensitive(
                self.options.filter_attribute_keys_case_sensitive,
            );
            let mut stages =
                planner.plan_stages(&self.pipeline_definition, &session_ctx, &otap_batch)?;

            // If scoped to a concrete metric type, wrap all planned stages in a conditional that
            // filters by the metric_type column. Non-matching rows pass through unmodified.
            //
            // TODO: this wrapping in ConditionalPipelineStage is functionally correct but not
            // optimal -- it splits the OTAP batch, runs stages on the matching subset, then
            // concatenates back. A dedicated metric-type-aware execution path could avoid the
            // split/concat overhead.
            if let SignalContext::Single(SignalKind::Metrics(MetricTypeContext::Single(
                metric_type,
            ))) = &self.options.signal_context
            {
                stages = vec![Self::wrap_in_metric_type_filter(stages, *metric_type)?];
            }

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

    /// Get the signal type from an OTAP batch.
    fn batch_signal_type(batch: &OtapArrowRecords) -> SignalType {
        match batch {
            OtapArrowRecords::Logs(_) => SignalType::Logs,
            OtapArrowRecords::Metrics(_) => SignalType::Metrics,
            OtapArrowRecords::Traces(_) => SignalType::Traces,
        }
    }

    /// Wrap a vec of pipeline stages in a `ConditionalPipelineStage` that filters
    /// by the `metric_type` column. Non-matching metric rows pass through unmodified.
    fn wrap_in_metric_type_filter(
        stages: Vec<BoxedPipelineStage>,
        metric_type: MetricType,
    ) -> Result<BoxedPipelineStage> {
        let predicate = ScopedExpr::Eval {
            scope: DataScope::Record(RecordScope::Signal),
            eval: LeafEval::new_df_expr(
                col(consts::METRIC_TYPE).eq(lit(metric_type as u8)),
                false,
            )?,
        };
        let branch = ConditionalPipelineStageBranch::new(predicate, stages);
        Ok(Box::new(ConditionalPipelineStage::new(
            vec![branch],
            None, // no default branch -- non-matching rows pass through
        )))
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use otel_arrow_contrib_data_engine_expressions::PipelineExpression;

    use datafusion::catalog::streaming::StreamingTable;
    use datafusion::logical_expr::{col, lit};
    use otel_arrow_contrib_data_engine_kql_parser::KqlParser;
    use otel_arrow_contrib_data_engine_parser_abstractions::Parser;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData};
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, MetricsData, Sum,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::TracesData;
    use otel_arrow_dfe_pdata::testing::round_trip::{
        otap_to_otlp, otlp_to_otap, to_otap_logs, to_otap_metrics,
    };
    use otel_arrow_dfe_pdata::{OtapPayload, OtlpProtoBytes, TryIntoWithOptions};
    use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;
    use prost::Message;

    use crate::parser::default_parser_options;

    use super::*;

    /// helper function for converting [`OtapArrowRecords`] to [`LogsData`]
    pub fn otap_to_logs_data(otap_batch: OtapArrowRecords) -> LogsData {
        let otap_payload: OtapPayload = otap_batch.into();
        let otlp_bytes: OtlpProtoBytes = otap_payload.try_into_with_default().unwrap();
        LogsData::decode(otlp_bytes.as_bytes()).unwrap()
    }

    /// helper function for converting [`OtapArrowRecords`] to [`TracesData`]
    pub fn otap_to_traces_data(otap_batch: OtapArrowRecords) -> TracesData {
        let otap_payload: OtapPayload = otap_batch.into();
        let otlp_bytes: OtlpProtoBytes = otap_payload.try_into_with_default().unwrap();
        TracesData::decode(otlp_bytes.as_bytes()).unwrap()
    }

    /// helper function for converting [`OtapArrowRecords`] to [`MetricsData`]
    pub fn otap_to_metrics_data(otap_batch: OtapArrowRecords) -> MetricsData {
        let otap_payload: OtapPayload = otap_batch.into();
        let otlp_bytes: OtlpProtoBytes = otap_payload.try_into_with_default().unwrap();
        MetricsData::decode(otlp_bytes.as_bytes()).unwrap()
    }

    pub async fn exec_logs_pipeline<P: Parser>(query: &str, logs_data: LogsData) -> LogsData {
        let options = default_parser_options();
        let parser_result = P::parse_with_options(query, options).unwrap();
        exec_logs_pipeline_expr(parser_result.pipeline, logs_data).await
    }

    pub async fn exec_logs_pipeline_expr(
        pipeline_expr: PipelineExpression,
        logs_data: LogsData,
    ) -> LogsData {
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        let options = PipelineOptions {
            signal_context: SignalContext::Single(SignalKind::Logs),
            ..Default::default()
        };
        let mut pipeline = Pipeline::new_with_options(pipeline_expr, options);
        let result = pipeline.execute(otap_batch).await.unwrap();
        otap_to_logs_data(result)
    }

    pub async fn exec_metrics_pipeline<P: Parser>(
        query: &str,
        metrics_data: MetricsData,
    ) -> MetricsData {
        let parser_result = P::parse(query).unwrap();
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(metrics_data));
        let options = PipelineOptions {
            signal_context: SignalContext::Single(SignalKind::Metrics(MetricTypeContext::All)),
            ..Default::default()
        };
        let mut pipeline = Pipeline::new_with_options(parser_result.pipeline, options);
        let result = pipeline.execute(otap_batch).await.unwrap();
        otap_to_metrics_data(result)
    }

    pub async fn exec_traces_pipeline<P: Parser>(
        query: &str,
        traces_data: TracesData,
    ) -> TracesData {
        let parser_result = P::parse(query).unwrap();
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Traces(traces_data));
        let options = PipelineOptions {
            signal_context: SignalContext::Single(SignalKind::Traces),
            ..Default::default()
        };
        let mut pipeline = Pipeline::new_with_options(parser_result.pipeline, options);
        let result = pipeline.execute(otap_batch).await.unwrap();
        otap_to_traces_data(result)
    }

    #[tokio::test]
    async fn test_pipeline_execute_multi_batch() {
        // TODO eventually we might want to drive this test from a pipeline expression, which we
        // can do once we have the query planning implemented. For now, we are manually creating
        // the `PlannedPipeline` and its `PipelineStage`s and any additional datafusion context
        // they need

        let otap_batch1 = to_otap_logs(vec![
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
        ]);

        let otap_batch2 = to_otap_logs(vec![
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
        ]);

        let schema = otap_batch1.get(ArrowPayloadType::Logs).unwrap().schema();
        let rb_stream = Arc::new(RecordBatchPartitionStream::new(schema.clone()));
        let table = StreamingTable::try_new(schema.clone(), vec![rb_stream.clone()]).unwrap();

        let ctx = Pipeline::create_session_context();
        _ = ctx.register_table("logs", Arc::new(table)).unwrap();
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
            options: PipelineOptions::default(),
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

    /// Scenario: Run a pipeline sourced from a concrete metric type over mixed inputs.
    /// Guarantees: Only gauge metric rows are transformed; other metric types and other signals
    /// pass through unchanged.
    #[tokio::test]
    async fn test_pipelines_selecting_concrete_metrics_type_skip_exec_only_on_selected_rows() {
        let gauge_metric = Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge::default())
            .finish();

        let sum_metric = Metric::build()
            .name("sum_metric")
            .data_sum(Sum::default())
            .finish();

        let query = "gauges | set name =\"gauge_name_updated\"";
        let parser_result = OplParser::parse(query).unwrap();
        let mut pipeline = Pipeline::try_new(parser_result.pipeline).unwrap();

        // Pipeline should reject non-metrics signal types
        assert!(!pipeline.accepts_signal_type(SignalType::Logs));
        assert!(!pipeline.accepts_signal_type(SignalType::Traces));
        assert!(pipeline.accepts_signal_type(SignalType::Metrics));

        // Pipeline should reject logs batches at execution time
        let logs_input = to_otap_logs(vec![LogRecord::build().finish()]);
        assert!(pipeline.execute(logs_input).await.is_err());

        // Pipeline should transform only gauge rows, leaving sum rows unchanged
        let metrics_input = to_otap_metrics(vec![gauge_metric.clone(), sum_metric.clone()]);
        let result = pipeline.execute(metrics_input).await.unwrap();
        let OtlpProtoMessage::Metrics(metrics_result) = otap_to_otlp(&result) else {
            panic!("invalid signal type result")
        };
        assert_eq!(metrics_result.resource_metrics.len(), 1);
        assert_eq!(metrics_result.resource_metrics[0].scope_metrics.len(), 1);
        pretty_assertions::assert_eq!(
            metrics_result.resource_metrics[0].scope_metrics[0].metrics,
            vec![
                Metric::build()
                    .name("gauge_name_updated")
                    .data_gauge(Gauge::default())
                    .finish(),
                sum_metric.clone(),
            ]
        );
    }

    /// Scenario: Execute scale_metric against a logs batch.
    /// Guarantees: The metric-only operation returns an explicit pipeline error for other signals.
    #[tokio::test]
    async fn test_scale_metric_rejects_non_metric_signals() {
        let parser_result = OplParser::parse("logs | scale_metric 2").unwrap();
        let mut pipeline = Pipeline::try_new(parser_result.pipeline).unwrap();
        let result = pipeline
            .execute(to_otap_logs(vec![LogRecord::build().finish()]))
            .await;

        assert!(matches!(
            result,
            Err(Error::InvalidPipelineError { cause, .. })
                if cause == "scale_metric can only be applied to metrics"
        ));
    }

    // -- Signal context inference tests --

    /// Scenario: Pipeline::try_new infers signal context from each source keyword.
    /// Guarantees: every recognized source keyword produces the correct SignalContext.
    #[test]
    fn test_infer_signal_context_all_sources() {
        use super::planner::MetricTypeContext;

        let cases = [
            ("logs | where true", SignalContext::Single(SignalKind::Logs)),
            (
                "traces | where true",
                SignalContext::Single(SignalKind::Traces),
            ),
            (
                "metrics | where true",
                SignalContext::Single(SignalKind::Metrics(MetricTypeContext::All)),
            ),
            ("signals | where true", SignalContext::All),
            (
                "gauges | where true",
                SignalContext::Single(SignalKind::Metrics(MetricTypeContext::Single(
                    MetricType::Gauge,
                ))),
            ),
            (
                "sums | where true",
                SignalContext::Single(SignalKind::Metrics(MetricTypeContext::Single(
                    MetricType::Sum,
                ))),
            ),
            (
                "histograms | where true",
                SignalContext::Single(SignalKind::Metrics(MetricTypeContext::Single(
                    MetricType::Histogram,
                ))),
            ),
            (
                "exponential_histograms | where true",
                SignalContext::Single(SignalKind::Metrics(MetricTypeContext::Single(
                    MetricType::ExponentialHistogram,
                ))),
            ),
            (
                "summaries | where true",
                SignalContext::Single(SignalKind::Metrics(MetricTypeContext::Single(
                    MetricType::Summary,
                ))),
            ),
        ];

        for (query, expected) in cases {
            let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
            let inferred = SignalContext::try_infer(&pipeline_expr)
                .unwrap_or_else(|e| panic!("inference failed for '{query}': {e}"));
            assert_eq!(
                format!("{inferred:?}"),
                format!("{expected:?}"),
                "wrong signal context for source '{query}'"
            );
        }
    }

    /// Scenario: Pipeline::try_new rejects unrecognized source keywords.
    /// Guarantees: an unknown source produces an error containing the source token.
    #[test]
    fn test_infer_signal_context_rejects_unknown_source() {
        let pipeline_expr = KqlParser::parse("bogus | where true").unwrap().pipeline;
        let result = SignalContext::try_infer(&pipeline_expr);
        match result {
            Err(Error::InvalidPipelineError { cause, .. }) => {
                assert!(
                    cause.contains("bogus"),
                    "error should mention the bad source token, got: {cause}"
                );
            }
            other => panic!("expected InvalidPipelineError, got: {other:?}"),
        }
    }

    /// Scenario: Pipeline::try_new does exact token matching, not prefix matching.
    /// Guarantees: a source like "logsfoo" is rejected rather than matching "logs".
    #[test]
    fn test_infer_signal_context_exact_token_match() {
        let pipeline_expr = KqlParser::parse("logsfoo | where true").unwrap().pipeline;
        assert!(SignalContext::try_infer(&pipeline_expr).is_err());
    }
}
