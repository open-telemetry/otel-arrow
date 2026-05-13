// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementation of a pipeline stage that can optionally apply pipeline
//! stages to rows that match some predicate conditions.

use std::sync::Arc;

use arrow::array::{BooleanArray, RecordBatch};
use arrow::buffer::BooleanBuffer;
use arrow::compute::{and, filter_record_batch, not, or};
use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::prelude::SessionContext;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::raw_batch_store::{
    LOGS_TYPE_MASK, METRICS_TYPE_MASK, POSITION_LOOKUP, RawBatchStore, TRACES_TYPE_MASK,
};
use otap_df_pdata::otap::transform::concatenate::concatenate;
use otap_df_pdata::otap::{Logs, Metrics, OtapBatchStore, Traces};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use otap_df_pdata::otap::filter::IdBitmapPool;

use crate::error::{Error, Result};
use crate::pipeline::filter::{Composite, FilterExec, filter_otap_batch};
use crate::pipeline::state::ExecutionState;
use crate::pipeline::{BoxedPipelineStage, PipelineStage};

/// This [`PipelineStage`] implementation will conditionally apply child pipeline stages on rows
/// which match some condition. This can be used to implement `if/else if/else` type control flow
/// for items in a telemetry batch.
///
/// When executed, this will evaluate the pipeline in each branch on a disjoint set of rows that are
/// selected by the branches' conditions. The output will be the results of each branch & default
/// branch concatenated together.
///
/// Note: the order of the rows in the incoming [`RecordBatch`](arrow::array::RecordBatch)s may not
/// be preserved.
pub struct ConditionalPipelineStage {
    /// The data in the batches will be checked against the conditions for each branch, and the
    /// stages in each branch will be executed for the rows that pass the condition and didn't
    /// pass the condition for previous branches.
    ///
    /// These branches are analogous to `if`/`else if` control flow statements
    branches: Vec<ConditionalPipelineStageBranch>,

    /// The "default branch", if not `None`, will be executed for rows that did not pass a
    /// condition in any of the branches. If this branch is `None`, the remaining rows will be
    /// appended to the result batch with no transformation.
    ///
    /// This is analogous to the `else` branch of an if/else control flow statement.
    default_branch: Option<Vec<BoxedPipelineStage>>,

    /// Pool of reusable bitmaps for attribute filter execution and child batch filtering.
    id_bitmap_pool: IdBitmapPool,
}

impl ConditionalPipelineStage {
    pub fn new(
        branches: Vec<ConditionalPipelineStageBranch>,
        default_branch: Option<Vec<BoxedPipelineStage>>,
    ) -> Self {
        Self {
            branches,
            default_branch,
            id_bitmap_pool: IdBitmapPool::new(),
        }
    }
}

fn concat_generic<T, const TYPE_MASK: u64, const COUNT: usize>(
    branch_results: &mut Vec<OtapArrowRecords>,
) -> Result<OtapArrowRecords>
where
    T: OtapBatchStore<BatchArray = [Option<RecordBatch>; COUNT]>
        + TryFrom<RawBatchStore<TYPE_MASK, COUNT>, Error = otap_df_pdata::error::Error>
        + TryFrom<OtapArrowRecords, Error = otap_df_pdata::error::Error>,
    OtapArrowRecords: From<T>,
{
    let mut batches = Vec::new();
    for branch_result in branch_results.drain(..) {
        let batch_store: T = branch_result.try_into()?;
        batches.push(batch_store.into_batches())
    }

    let concatenated_batches = concatenate(&mut batches)?;
    let raw_store = RawBatchStore::<TYPE_MASK, COUNT>::from_batches(concatenated_batches);
    let result_store = T::try_from(raw_store)?;

    Ok(OtapArrowRecords::from(result_store))
}

fn concatenate_logs(branch_results: &mut Vec<OtapArrowRecords>) -> Result<OtapArrowRecords> {
    concat_generic::<Logs, { LOGS_TYPE_MASK }, { Logs::COUNT }>(branch_results)
}

fn concatenate_metrics(branch_results: &mut Vec<OtapArrowRecords>) -> Result<OtapArrowRecords> {
    concat_generic::<Metrics, { METRICS_TYPE_MASK }, { Metrics::COUNT }>(branch_results)
}

fn concatenate_traces(branch_results: &mut Vec<OtapArrowRecords>) -> Result<OtapArrowRecords> {
    concat_generic::<Traces, { TRACES_TYPE_MASK }, { Traces::COUNT }>(branch_results)
}

fn concatenate_attrs_record_batches(branch_results: &mut Vec<RecordBatch>) -> Result<RecordBatch> {
    // to call schema aware `concatenate` on just the attributes record batch, we stick it in
    // a Logs OTAP batch and treat it as log attributes. This is a bit of a hack until we have
    // a better top-level interface for calling concatenate.

    let mut otap_batches = branch_results
        .drain(..)
        .map(|attrs_record_batch| {
            let mut logs_record_batches = Logs::default();
            logs_record_batches.set(ArrowPayloadType::LogAttrs, attrs_record_batch)?;
            Ok(OtapArrowRecords::from(logs_record_batches))
        })
        .collect::<Result<Vec<_>>>()?;
    let concatenated_logs = concatenate_logs(&mut otap_batches)?;
    let mut concatenated_logs_batches = Logs::try_from(concatenated_logs)?.into_batches();
    let concatenated_attrs_batch = concatenated_logs_batches
        [POSITION_LOOKUP[ArrowPayloadType::LogAttrs as usize]]
        .take()
        .ok_or_else(|| Error::ExecutionError {
            cause: "expected concatenate to produce non 'None' batch".into(),
        })?;

    Ok(concatenated_attrs_batch)
}
/// A branch within a conditional pipeline stage
pub struct ConditionalPipelineStageBranch {
    /// This condition will be evaluated to determine for which rows to execute the pipeline
    /// stages. The semantics are such that rows will be selected that pass this condition and
    /// did not pass the condition for previous branches.
    condition: Composite<FilterExec>,

    /// These pipeline stages will be executed for rows selected for this branch, producing a new
    /// OTAP Batch for the branch which will be concatenated with batches from the other branches
    /// to produce the final result.
    pipeline_stages: Vec<BoxedPipelineStage>,
}

impl ConditionalPipelineStageBranch {
    pub fn new(predicate: Composite<FilterExec>, pipeline_stages: Vec<BoxedPipelineStage>) -> Self {
        Self {
            condition: predicate,
            pipeline_stages,
        }
    }
}

#[async_trait(?Send)]
impl PipelineStage for ConditionalPipelineStage {
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        session_ctx: &SessionContext,
        config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
        exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        // give the pipeline stages within each branch the opportunity to initialize any
        // necessary state:
        for branch in &mut self.branches {
            for stage in &mut branch.pipeline_stages {
                stage.init_state_for_conditional_branch(&otap_batch, exec_state)?;
            }
        }
        if let Some(branch) = self.default_branch.as_mut() {
            for stage in branch {
                stage.init_state_for_conditional_branch(&otap_batch, exec_state)?;
            }
        }

        let root_batch = match otap_batch.root_record_batch() {
            Some(root_batch) => root_batch,
            None => {
                // empty batch, nothing to do
                return Ok(otap_batch);
            }
        };

        // keep track of the rows that were selected by previous branches
        let mut already_selected_vec =
            BooleanArray::new(BooleanBuffer::new_unset(root_batch.num_rows()), None);

        let mut branch_results = Vec::with_capacity(
            self.branches.len() + if self.default_branch.is_some() { 1 } else { 0 },
        );

        for branch in &mut self.branches {
            if already_selected_vec.true_count() == root_batch.num_rows() {
                // all rows have been selected by previous branches, so there is no need to continue
                // executing the next branches with empty batches
                break;
            }

            // determine which rows are selected by this branch
            //
            // TODO: here we're evaluating the filter against all rows in the incoming batch for
            // each branch. There's probably some optimization we can make here if this becomes
            // a bottleneck. for example:
            // if previous branches have been very selective, we might consider materializing a
            // batch specifically containing the rows that have not already been selected and
            // feeding that into next iterations. This is extra overhead, but the resulting batch
            // would have less rows which could make filter faster.
            let predicate_selection_vec =
                branch
                    .condition
                    .execute(&otap_batch, session_ctx, &mut self.id_bitmap_pool)?;

            let branch_selection_vec = and(&predicate_selection_vec, &not(&already_selected_vec)?)?;

            // update the list of rows that were already selected by branches
            already_selected_vec = or(&already_selected_vec, &predicate_selection_vec)?;

            // create a batch with only the rows that match the condition
            //
            // TODO: the function we're calling here will materialize all the child record batches
            // with parent_ids of rows associated with the selected parent rows. If this becomes a
            // bottleneck, there might be an optimization to here where we don't do this for every
            // branch for every batch. For example, if the branch doesn't read or write some child
            // batch, we could avoid materializing it and sync up the rows once all branches have
            // been evaluated.
            let mut branch_otap_batch =
                filter_otap_batch(&branch_selection_vec, &otap_batch, &mut self.id_bitmap_pool)?;

            for stage in &mut branch.pipeline_stages {
                branch_otap_batch = stage
                    .execute(
                        branch_otap_batch,
                        session_ctx,
                        config_options,
                        task_context.clone(),
                        exec_state,
                    )
                    .await?;
            }

            branch_results.push(branch_otap_batch);
        }

        // handle the default branch - e.g. the rows that did not match the condition from any of
        // the previous branches
        if already_selected_vec.true_count() != root_batch.num_rows() {
            let mut default_branch_batch = filter_otap_batch(
                &not(&already_selected_vec)?,
                &otap_batch,
                &mut self.id_bitmap_pool,
            )?;

            if let Some(default_branch) = self.default_branch.as_mut() {
                for stage in default_branch {
                    default_branch_batch = stage
                        .execute(
                            default_branch_batch,
                            session_ctx,
                            config_options,
                            task_context.clone(),
                            exec_state,
                        )
                        .await?;
                }
            }
            branch_results.push(default_branch_batch);
        }

        // give the pipeline stages within each branch the opportunity to clear any
        // state that was initialized for this batch.
        for branch in &mut self.branches {
            for stage in &mut branch.pipeline_stages {
                stage.clear_state_for_conditional_branch(exec_state)?;
            }
        }
        if let Some(branch) = self.default_branch.as_mut() {
            for stage in branch {
                stage.clear_state_for_conditional_branch(exec_state)?;
            }
        }

        // reconstruct the result with the results of each branch
        match otap_batch {
            OtapArrowRecords::Logs(_) => concatenate_logs(&mut branch_results),
            OtapArrowRecords::Metrics(_) => concatenate_metrics(&mut branch_results),
            OtapArrowRecords::Traces(_) => concatenate_traces(&mut branch_results),
        }
    }

    async fn execute_on_attributes(
        &mut self,
        attrs_record_batch: RecordBatch,
        session_ctx: &SessionContext,
        config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
        exec_options: &mut ExecutionState,
    ) -> Result<RecordBatch> {
        if attrs_record_batch.num_rows() == 0 {
            // no branches would handle any rows, so nothing to do
            return Ok(attrs_record_batch);
        }

        // keep track of the rows that were selected by previous branches
        let mut already_selected_vec = BooleanArray::new(
            BooleanBuffer::new_unset(attrs_record_batch.num_rows()),
            None,
        );

        let mut branch_results = Vec::with_capacity(
            self.branches.len() + if self.default_branch.is_some() { 1 } else { 0 },
        );

        for branch in &mut self.branches {
            if already_selected_vec.true_count() == attrs_record_batch.num_rows() {
                // all rows have been selected by previous branches, so there is no need to continue
                // executing the next branches with empty batches
                break;
            }

            // extract the base filter predicate from the branch condition
            let filter_exec = match &mut branch.condition {
                Composite::Base(filter) => filter,
                _ => {
                    return Err(Error::InvalidPipelineError {
                        cause: "invalid filter plan variant. This pipeline stage was not optimized for attribute filtering".into(),
                        query_location: None,
                    });
                }
            };

            // determine which rows are selected by this branch's predicate
            let predicate = filter_exec.predicate
                .as_mut()
                .ok_or_else(||Error::InvalidPipelineError {
                    cause: "invalid filter plan variant. This pipeline stage was not optimized for attribute filtering".into(),
                    query_location: None,
                })?;
            let predicate_selection_vec =
                predicate.evaluate_filter(&attrs_record_batch, session_ctx)?;

            // select only the rows that match this branch AND were not already selected
            // by a previous branch
            let branch_selection_vec = and(&predicate_selection_vec, &not(&already_selected_vec)?)?;

            // update the list of rows that were already selected by branches
            already_selected_vec = or(&already_selected_vec, &predicate_selection_vec)?;

            // create a record batch with only the rows that match the condition and execute
            // the branch's pipeline stages on it
            let mut branch_record_batch =
                filter_record_batch(&attrs_record_batch, &branch_selection_vec)?;
            for stage in &mut branch.pipeline_stages {
                branch_record_batch = stage
                    .execute_on_attributes(
                        branch_record_batch,
                        session_ctx,
                        config_options,
                        task_context.clone(),
                        exec_options,
                    )
                    .await?;
            }

            branch_results.push(branch_record_batch)
        }

        // handle the default branch - e.g. the rows that did not match the condition from any of
        // the previous branches
        if already_selected_vec.true_count() != attrs_record_batch.num_rows() {
            let mut default_branch_batch =
                filter_record_batch(&attrs_record_batch, &not(&already_selected_vec)?)?;

            if let Some(default_branch) = self.default_branch.as_mut() {
                for stage in default_branch {
                    default_branch_batch = stage
                        .execute_on_attributes(
                            default_branch_batch,
                            session_ctx,
                            config_options,
                            task_context.clone(),
                            exec_options,
                        )
                        .await?;
                }
            }
            branch_results.push(default_branch_batch);
        }

        // reconstruct the result by concatenating the record batches from all branches
        let final_result = concatenate_attrs_record_batches(&mut branch_results)?;

        Ok(final_result)
    }

    fn supports_exec_on_attributes(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use crate::pipeline::{
        Pipeline, PipelineOptions,
        test::{
            exec_logs_pipeline, exec_metrics_pipeline, exec_traces_pipeline, otap_to_logs_data,
        },
    };
    use arrow::array::UInt16Array;
    use data_engine_parser_abstractions::Parser;
    use otap_df_pdata::{
        proto::opentelemetry::{
            metrics::v1::Metric,
            trace::v1::{Status, span::SpanKind},
        },
        schema::consts,
        testing::round_trip::{otap_to_otlp, to_logs_data},
    };
    use otap_df_pdata::{
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                common::v1::{AnyValue, KeyValue},
                logs::v1::LogRecord,
                trace::v1::Span,
            },
        },
        testing::round_trip::{otlp_to_otap, to_metrics_data, to_traces_data},
    };
    use otap_df_query_engine_languages::opl::parser::OplParser;

    use super::*;

    #[tokio::test]
    async fn test_conditional_no_default_branch() {
        let log_records = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
        ];

        let result = exec_logs_pipeline::<OplParser>(
            r#"
            logs | if (severity_text == "ERROR") {
                project-rename attributes["y"] = attributes["x"]
            }"#,
            to_logs_data(log_records),
        )
        .await;
        let expected = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
        ];

        pretty_assertions::assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected)
    }

    #[tokio::test]
    async fn test_conditional_with_condition_match_statement() {
        let log_records = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
        ];

        // internally, try_fold must be called on the match expression here to convert the string
        // argument into a regex scalar expression. This test is to help avoid regressions of
        // cases where try_fold might not be called on the condition statements
        let result = exec_logs_pipeline::<OplParser>(
            r#"
            logs | if (matches(severity_text, ".*E.*")) {
                project-rename attributes["y"] = attributes["x"]
            }"#,
            to_logs_data(log_records),
        )
        .await;
        let expected = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
        ];

        pretty_assertions::assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected)
    }

    #[tokio::test]
    async fn test_conditional_with_default_branch() {
        let log_records = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
        ];

        let result = exec_logs_pipeline::<OplParser>(
            r#"
            logs | if (severity_text == "ERROR") {
                project-rename attributes["y"] = attributes["x"]
            } else {
                project-rename attributes["z"] = attributes["x"]
            }"#,
            to_logs_data(log_records),
        )
        .await;

        let expected = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .attributes(vec![KeyValue::new("z", AnyValue::new_string("test"))])
                .finish(),
        ];

        pretty_assertions::assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected)
    }

    #[tokio::test]
    async fn test_conditional_multiple_branches() {
        let log_records = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .event_name("test")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .event_name("test")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .event_name("test_2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("test")),
                    KeyValue::new("a", AnyValue::new_string("test")),
                ])
                .finish(),
        ];

        let query = r#"logs |
            if (severity_text == "ERROR") {
                project-rename attributes["y"] = attributes["x"]
            } else if (event_name == "test") {
                project-rename attributes["z"] = attributes["x"]
            } else {
                project-away attributes["x"]
            }
        "#;
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records)).await;

        let expected = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .event_name("test")
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .event_name("test")
                .attributes(vec![KeyValue::new("z", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .event_name("test_2")
                .attributes(vec![KeyValue::new("a", AnyValue::new_string("test"))])
                .finish(),
        ];

        pretty_assertions::assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected)
    }

    #[tokio::test]
    async fn test_conditional_early_branch_selects_all() {
        // there's a shortcut where we can stop checking the conditions for branches
        // once all rows have been selected and processed. This test ensures we get
        // correct results when we use that code path
        let log_records = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("test"))])
                .finish(),
        ];
        let query = r#"logs |
            if (severity_text == "INFO") {
                project-rename attributes["y"] = attributes["x"]
            } else if (severity_text == "ERROR") {
                project-rename attributes["z"] = attributes["x"]
            } else if (severity_text == "WARN") {
                project-rename attributes["a"] = attributes["x"]
            } else {
                project-away attributes["x"]
            }
        "#;
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records)).await;

        let expected = vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("z", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("z", AnyValue::new_string("test"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("z", AnyValue::new_string("test"))])
                .finish(),
        ];

        pretty_assertions::assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected)
    }

    #[tokio::test]
    async fn test_empty_batch() {
        let pipeline_expr = OplParser::parse(
            r#"logs |
            if (severity_text == "ERROR") {
                project-rename attributes["y"] = attributes["x"]
            } else if (event_name == "test") {
                project-rename attributes["z"] = attributes["x"]
            } else {
                project-away attributes["x"]
            }
        "#,
        )
        .unwrap()
        .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        let input = OtapArrowRecords::Logs(Logs::default());
        let result = pipeline.execute(input.clone()).await.unwrap();
        assert_eq!(result, input);
    }

    #[tokio::test]
    async fn test_conditional_concat_handles_stages_that_modified_logs_batch() {
        let log_records = vec![
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().severity_text("ERROR").finish(),
        ];

        // the logs that take the "if" branch will get the optional severity_number column added
        // whereas the logs that take the "else" branch will get the optional event_name column
        // added. This test ensures we still concatenate them correctly despite the schema mismatch
        let query = r#"logs |
            if (severity_text == "INFO") {
                set severity_number = 10
            } else {
                set event_name = "hello"
            }
        "#;
        let result = exec_logs_pipeline::<OplParser>(query, to_logs_data(log_records)).await;

        let expected = vec![
            LogRecord::build()
                .severity_text("INFO")
                .severity_number(10)
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .event_name("hello")
                .finish(),
        ];

        pretty_assertions::assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected)
    }

    #[tokio::test]
    async fn test_conditional_concat_handles_id_remapping() {
        let log_records = vec![
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("y"))])
                .finish(),
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().severity_text("ERROR").finish(),
        ];

        // some logs will not have the 'id' column set, because they do not have attributes.
        // when the attribute is assigned, the id column will be set, but since this is happening
        // inside each branch, the assigned ids may conflict. When the batches from each branch are
        // concatenated, the conflicting IDs must be reconciled, and what we're testing here is
        // that this reconciliation actually happens.
        let query = r#"logs |
            if (severity_text == "INFO") {
                set attributes["x"] = "hello"
            } else {
                set attributes["x"] = "world"
            }
        "#;
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        let mut execution_state = ExecutionState::new();

        let result = pipeline
            .execute_with_state(
                otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records))),
                &mut execution_state,
            )
            .await
            .unwrap();
        let result = otap_to_logs_data(result);

        let expected = vec![
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("hello"))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("hello"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("world"))])
                .finish(),
        ];

        pretty_assertions::assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected);

        // ensure that if we send in a second batch, the ID tracking state gets reset and we don't
        // end up overwriting IDs somehow. We'll have inserted IDs 1 and 2 for the batch above,
        // so the next ID would be 3. But in the following batch, we have rows 4 rows with
        // attributes, so the next ID should be 4
        let log_records = vec![
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("y"))])
                .finish(),
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("y"))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("y"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("y"))])
                .finish(),
        ];

        let result = pipeline
            .execute_with_state(
                otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records))),
                &mut execution_state,
            )
            .await
            .unwrap();

        let id_column = result
            .get(ArrowPayloadType::Logs)
            .unwrap()
            .column_by_name(consts::ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();
        assert_eq!(id_column, &UInt16Array::from_iter_values([0, 4, 2, 1, 3]));

        let result = otap_to_logs_data(result);
        // ensure the attributes are still properly assigned
        let expected = vec![
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("hello"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("world"))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("hello"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("world"))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("hello"))])
                .finish(),
        ];
        pretty_assertions::assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected);
    }

    #[tokio::test]
    async fn test_conditional_concat_handles_stages_that_modified_traces_batch() {
        let spans = vec![
            Span::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .span_id([0; 8])
                .trace_id([0; 16])
                .status(Status::default())
                .finish(),
            Span::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                .span_id([0; 8])
                .trace_id([0; 16])
                .status(Status::default())
                .finish(),
        ];
        // the spans that take the "if" branch will get the optional kind column added whereas the
        // spans that take the "else" branch wont. This test ensures we still concatenate them
        // correctly despite the schema mismatch
        let query = r#"traces |
            if (attributes["x"] == "a") {
                set kind = 1
            }
        "#;
        let result = exec_traces_pipeline::<OplParser>(query, to_traces_data(spans)).await;

        let expected = vec![
            Span::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .span_id([0; 8])
                .trace_id([0; 16])
                .kind(SpanKind::Internal)
                .status(Status::default())
                .finish(),
            Span::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                .span_id([0; 8])
                .trace_id([0; 16])
                .status(Status::default())
                .finish(),
        ];

        pretty_assertions::assert_eq!(result.resource_spans[0].scope_spans[0].spans, expected)
    }

    #[tokio::test]
    async fn test_conditional_concat_handles_stages_that_modified_metric_batch() {
        let metrics = vec![
            Metric::build().name("metric1").finish(),
            Metric::build().name("metric2").finish(),
        ];

        // the metrics that take the "if" branch will get the optional description column added
        // whereas the logs that take the "else" branch will get the optional unit column
        // added. This test ensures we still concatenate them correctly despite the schema mismatch
        let query = r#"metrics |
            if (name == "metric1") {
                set description = "description"
            } else {
                set unit = "centimeters"
            }
        "#;
        let result = exec_metrics_pipeline::<OplParser>(query, to_metrics_data(metrics)).await;

        let expected = vec![
            Metric::build()
                .name("metric1")
                .description("description")
                .finish(),
            Metric::build().name("metric2").unit("centimeters").finish(),
        ];

        pretty_assertions::assert_eq!(
            result.resource_metrics[0].scope_metrics[0].metrics,
            expected
        )
    }

    #[tokio::test]
    async fn test_conditional_with_case_insensitive_attribute_key_match_in_condition() {
        let log_records = vec![
            LogRecord::build()
                .event_name("event1")
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .event_name("event2")
                .attributes(vec![KeyValue::new("KEY1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .event_name("event3")
                .attributes(vec![KeyValue::new("KEY1", AnyValue::new_string("val2"))])
                .finish(),
            LogRecord::build()
                .event_name("event4")
                .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val1"))])
                .finish(),
        ];

        let query = r#"
            logs |
            if (attributes["key1"] == "val1") {
                set attributes["modified"] = true
            }
        "#;
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );

        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records)));
        let result = pipeline.execute(input).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("Invalid signal variant {result:?}")
        };

        let expected = vec![
            LogRecord::build()
                .event_name("event1")
                .attributes(vec![
                    KeyValue::new("key1", AnyValue::new_string("val1")),
                    KeyValue::new("modified", AnyValue::new_bool(true)),
                ])
                .finish(),
            LogRecord::build()
                .event_name("event2")
                .attributes(vec![
                    KeyValue::new("KEY1", AnyValue::new_string("val1")),
                    KeyValue::new("modified", AnyValue::new_bool(true)),
                ])
                .finish(),
            LogRecord::build()
                .event_name("event3")
                .attributes(vec![KeyValue::new("KEY1", AnyValue::new_string("val2"))])
                .finish(),
            LogRecord::build()
                .event_name("event4")
                .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val1"))])
                .finish(),
        ];

        pretty_assertions::assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected)
    }
}
