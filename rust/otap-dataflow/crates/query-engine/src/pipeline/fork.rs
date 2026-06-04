// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::RecordBatch;
use async_trait::async_trait;
use datafusion::{
    config::ConfigOptions,
    execution::{TaskContext, context::SessionContext},
};
use otap_df_pdata::{
    OtapArrowRecords, OtapPayloadHelpers,
    otap::{Logs, Metrics, Traces},
};

use crate::pipeline::{BoxedPipelineStage, PipelineStage, state::ExecutionState};
use crate::{
    error::Result,
    pipeline::concat::{
        concatenate_logs, concatenate_metrics, concatenate_traces, reindex_logs, reindex_metrics,
        reindex_traces,
    },
};

/// This [`PipelineStage`] implementation duplicates each batch of telemetry and processes it on
/// each nested branch. The results are then unioned back together to produce the output.
///
/// This may produce duplicate telemetry records, however the nominal use case for fork is really
/// to duplicate the telemetry, process it, and send it to different destinations using an operator
/// such as "route_to".
pub struct ForkPipelineStage {
    /// Branches that wil process a copy of each telemetry batch
    branches: Vec<ForkPipelineStageBranch>,

    /// Scratch space for storing temporary batch results before concatenating results.
    /// T
    branch_results: Vec<OtapArrowRecords>,
}

impl ForkPipelineStage {
    pub fn new(branches: Vec<ForkPipelineStageBranch>) -> Self {
        Self {
            branches,
            branch_results: Vec::new(),
        }
    }
}

/// Branch that will process a copy of each telemetry batch
pub struct ForkPipelineStageBranch {
    /// The stages that will be executed in this branch
    pipeline_stages: Vec<BoxedPipelineStage>,
}

impl ForkPipelineStageBranch {
    pub fn new(pipeline_stages: Vec<BoxedPipelineStage>) -> Self {
        Self { pipeline_stages }
    }
}

#[async_trait(?Send)]
impl PipelineStage for ForkPipelineStage {
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        session_ctx: &SessionContext,
        config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
        exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        self.branch_results.clear();

        for branch in &mut self.branches {
            let mut branch_batch = otap_batch.clone();
            for stage in &mut branch.pipeline_stages {
                branch_batch = stage
                    .execute(
                        branch_batch,
                        session_ctx,
                        config_options,
                        Arc::clone(&task_context),
                        exec_state,
                    )
                    .await?;
            }

            if !branch_batch.is_empty() {
                self.branch_results.push(branch_batch);
            }
        }

        match self.branch_results.len() {
            // no branch returned any results, return empty OTAP batch:
            0 => Ok(match otap_batch {
                OtapArrowRecords::Logs(_) => OtapArrowRecords::Logs(Logs::default()),
                OtapArrowRecords::Metrics(_) => OtapArrowRecords::Metrics(Metrics::default()),
                OtapArrowRecords::Traces(_) => OtapArrowRecords::Traces(Traces::default()),
            }),

            // only one branch returned a non-empty result, simply return this
            1 => Ok(self.branch_results.pop().expect("one record")),

            // concatenate batches together, adjusting IDs to avoid ID duplicates:
            _ => match otap_batch {
                OtapArrowRecords::Logs(_) => {
                    reindex_logs(&mut self.branch_results)?;
                    concatenate_logs(&mut self.branch_results)
                }
                OtapArrowRecords::Metrics(_) => {
                    reindex_metrics(&mut self.branch_results)?;
                    concatenate_metrics(&mut self.branch_results)
                }
                OtapArrowRecords::Traces(_) => {
                    reindex_traces(&mut self.branch_results)?;
                    concatenate_traces(&mut self.branch_results)
                }
            },
        }
    }
}

#[cfg(test)]
mod test {

    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::Metric;
    use otap_df_pdata::proto::opentelemetry::trace::v1::{Span, Status};
    use otap_df_pdata::testing::round_trip::{to_logs_data, to_metrics_data, to_traces_data};
    use otap_df_query_engine_languages::opl::parser::OplParser;

    use crate::pipeline::test::{exec_logs_pipeline, exec_metrics_pipeline, exec_traces_pipeline};

    #[tokio::test]
    async fn test_simple_fork() {
        let log_records = vec![LogRecord::build().finish()];
        let result = exec_logs_pipeline::<OplParser>(
            r#"logs |
            fork
            {
                set body="hello", attributes["x"] = "hello"
            }
            {
                set body="world", attributes["x"] = "world"
            }"#,
            to_logs_data(log_records),
        )
        .await;

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[LogRecord::build()
                .body(AnyValue::new_string("hello"))
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("hello"))])
                .finish()]
        );
        assert_eq!(
            &result.resource_logs[1].scope_logs[0].log_records,
            &[LogRecord::build()
                .body(AnyValue::new_string("world"))
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("world"))])
                .finish()]
        );
    }

    #[tokio::test]
    async fn test_fork_only_one_branch_has_rows() {
        let log_records = vec![LogRecord::build().severity_number(5).finish()];
        let result = exec_logs_pipeline::<OplParser>(
            r#"logs |
            fork
            {
                where severity_number < 5 | set body = "branch1"
            }
            {
                where severity_number == 5 | set body = "branch2"
            }"#,
            to_logs_data(log_records),
        )
        .await;

        assert_eq!(result.resource_logs.len(), 1);
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[LogRecord::build()
                .severity_number(5)
                .body(AnyValue::new_string("branch2"))
                .finish()]
        );
    }

    #[tokio::test]
    async fn test_fork_no_branch_returns_results() {
        let log_records = vec![LogRecord::build().severity_number(5).finish()];
        let result = exec_logs_pipeline::<OplParser>(
            r#"logs |
            fork
            {
                where severity_number < 5 | set body = "branch1"
            }
            {
                where severity_number > 5 | set body = "branch2"
            }"#,
            to_logs_data(log_records),
        )
        .await;

        assert_eq!(result.resource_logs.len(), 0);
    }

    #[tokio::test]
    async fn test_fork_metrics() {
        let metrics = vec![Metric::build().name("my_metric").finish()];
        let result = exec_metrics_pipeline::<OplParser>(
            r#"metrics |
            fork
            {
                set description = "my metric"
            }
            {
                set description = "some metric"
            }"#,
            to_metrics_data(metrics),
        )
        .await;

        assert_eq!(
            &result.resource_metrics[0].scope_metrics[0].metrics,
            &[Metric::build()
                .name("my_metric")
                .description("my metric")
                .finish()]
        );

        assert_eq!(
            &result.resource_metrics[0].scope_metrics[1].metrics,
            &[Metric::build()
                .name("my_metric")
                .description("some metric")
                .finish()]
        );
    }

    #[tokio::test]
    async fn test_fork_spans() {
        let spans = vec![
            Span::build()
                .trace_id(vec![0; 16])
                .span_id(vec![0; 8])
                .status(Status::default())
                .finish(),
        ];
        let result = exec_traces_pipeline::<OplParser>(
            r#"
            traces |
            fork
            {
                set name = "my_span"
            }
            {
                set attributes["x"] = "hello"
            }
        "#,
            to_traces_data(spans),
        )
        .await;

        assert_eq!(
            &result.resource_spans[0].scope_spans[0].spans,
            &[Span::build()
                .name("my_span")
                .trace_id(vec![0; 16])
                .span_id(vec![0; 8])
                .status(Status::default())
                .finish()]
        );

        assert_eq!(
            &result.resource_spans[1].scope_spans[0].spans,
            &[Span::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("hello"))])
                .trace_id(vec![0; 16])
                .span_id(vec![0; 8])
                .status(Status::default())
                .finish()]
        );
    }
}
