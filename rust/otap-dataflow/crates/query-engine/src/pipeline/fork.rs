// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::RecordBatch;
use async_trait::async_trait;
use datafusion::{
    config::ConfigOptions,
    execution::{TaskContext, context::SessionContext},
};
use otap_df_pdata::{OtapArrowRecords, OtapPayloadHelpers};

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

        match otap_batch {
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
        }
    }

    fn supports_exec_on_attributes(&self) -> bool {
        true
    }

    async fn execute_on_attributes(
        &mut self,
        attrs_record_batch: RecordBatch,
        session_ctx: &SessionContext,
        config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
        exec_options: &mut ExecutionState,
    ) -> Result<RecordBatch> {
        todo!()
    }
}

#[cfg(test)]
mod test {

    use otap_df_pdata::{
        proto::opentelemetry::logs::v1::LogRecord, testing::round_trip::to_logs_data,
    };
    use otap_df_query_engine_languages::opl::parser::OplParser;

    use crate::pipeline::test::exec_logs_pipeline;

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

        println!("{result:#?}")
    }
}
