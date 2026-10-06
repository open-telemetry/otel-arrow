// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementation of a pipeline stage that can optionally apply pipeline
//! stages to rows that match some predicate conditions.

use std::sync::Arc;

use arrow::array::{BooleanArray, RecordBatch};
use arrow::buffer::BooleanBuffer;
use arrow::compute::{and, filter_record_batch, not, or};
use arrow::datatypes::UInt16Type;
use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::prelude::SessionContext;
use otel_arrow_dfe_pdata::OtapArrowRecords;

use otel_arrow_dfe_pdata::otap::filter::{
    ChildBatchFilterIdHelper, IdBitmapPool, filter_otap_batch,
};
use otel_arrow_dfe_pdata::otap::transform::concatenate::ConcatOptions;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;

use crate::error::Result;
use crate::pipeline::concat::{
    concatenate_attrs_record_batches, concatenate_logs, concatenate_metrics, concatenate_traces,
};
use crate::pipeline::expr::eval::EvalContext;
use crate::pipeline::expr::{DataScope, ScopedExpr};
use crate::pipeline::filter::{align_selection_to_root, scoped_value_to_boolean_array};
use crate::pipeline::planner::RecordType;
use crate::pipeline::state::ExecutionState;
use crate::pipeline::{BoxedPipelineStage, ParentBehavior, PipelineStage};

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

/// A branch within a conditional pipeline stage
pub struct ConditionalPipelineStageBranch {
    /// This condition will be evaluated to determine for which rows to execute the pipeline
    /// stages. The semantics are such that rows will be selected that pass this condition and
    /// did not pass the condition for previous branches.
    condition: ScopedExpr,

    /// These pipeline stages will be executed for rows selected for this branch, producing a new
    /// OTAP Batch for the branch which will be concatenated with batches from the other branches
    /// to produce the final result.
    pipeline_stages: Vec<BoxedPipelineStage>,
}

impl ConditionalPipelineStageBranch {
    pub fn new(predicate: ScopedExpr, pipeline_stages: Vec<BoxedPipelineStage>) -> Self {
        Self {
            condition: predicate,
            pipeline_stages,
        }
    }
}

fn stages_require_parent_reindex(stages: &[BoxedPipelineStage]) -> bool {
    stages
        .iter()
        .any(|stage| stage.parent_behavior().requires_reindex())
}

fn retain_shared_attribute_payload_once(
    branch_results: &mut [OtapArrowRecords],
    payload_type: ArrowPayloadType,
    pool: &mut IdBitmapPool,
) -> Result<()> {
    let mut claimed_ids = pool.acquire();
    let mut branch_ids = pool.acquire();

    let result = (|| {
        for branch_result in branch_results {
            let Some(root_batch) = branch_result.root_record_batch() else {
                _ = branch_result.remove(payload_type);
                continue;
            };
            let Some(id_column) = UInt16Type::get_id_col_from_parent(root_batch, payload_type)?
            else {
                _ = branch_result.remove(payload_type);
                continue;
            };

            branch_ids.populate(id_column.iter().flatten().map(u32::from));
            branch_ids.difference_with(&claimed_ids);

            if branch_ids.is_empty() {
                _ = branch_result.remove(payload_type);
                continue;
            }

            if let Some(attrs_batch) = branch_result.get(payload_type) {
                let parent_ids =
                    attrs_batch
                        .column_by_name(consts::PARENT_ID)
                        .ok_or_else(|| crate::error::Error::ExecutionError {
                            cause: "attribute batch is missing parent_id".into(),
                        })?;
                let selection = UInt16Type::build_selection_vec(parent_ids, &branch_ids)?;
                if selection.true_count() == 0 {
                    _ = branch_result.remove(payload_type);
                } else if selection.true_count() != attrs_batch.num_rows() {
                    let filtered = filter_record_batch(attrs_batch, &selection)?;
                    branch_result.set(payload_type, filtered)?;
                }
            }

            claimed_ids.union_with(&branch_ids);
        }
        Ok(())
    })();

    pool.release(branch_ids);
    pool.release(claimed_ids);
    result
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
        let mut reindex_branch_results = false;

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
            let eval_ctx = EvalContext::new(session_ctx);
            let predicate_result = branch.condition.execute_as_value(&otap_batch, &eval_ctx)?;

            let predicate_selection_vec = match predicate_result {
                None => BooleanArray::new(BooleanBuffer::new_unset(root_batch.num_rows()), None),
                Some(scoped_value) => {
                    if !(matches!(
                        scoped_value.scope,
                        DataScope::Record(_) | DataScope::RootParent(_)
                    )) && scoped_value.scope != DataScope::StaticScalar
                    {
                        align_selection_to_root(Some(scoped_value), &otap_batch, &eval_ctx)?
                    } else {
                        // extract the BooleanArray from the ScopedValue
                        scoped_value_to_boolean_array(scoped_value.values, root_batch.num_rows())?
                    }
                }
            };

            let branch_selection_vec = and(&predicate_selection_vec, &not(&already_selected_vec)?)?;

            // update the list of rows that were already selected by branches
            already_selected_vec = or(&already_selected_vec, &predicate_selection_vec)?;

            if branch_selection_vec.true_count() == 0 {
                // no rows selected - no sense executing the branch on zero rows
                continue;
            }

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

            reindex_branch_results |= stages_require_parent_reindex(&branch.pipeline_stages);
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
                for stage in default_branch.iter_mut() {
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
                reindex_branch_results |= stages_require_parent_reindex(default_branch);
            }
            branch_results.push(default_branch_batch);
        }

        if !reindex_branch_results {
            retain_shared_attribute_payload_once(
                &mut branch_results,
                ArrowPayloadType::ScopeAttrs,
                &mut self.id_bitmap_pool,
            )?;
            retain_shared_attribute_payload_once(
                &mut branch_results,
                ArrowPayloadType::ResourceAttrs,
                &mut self.id_bitmap_pool,
            )?;
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
        let concat_options = if reindex_branch_results {
            ConcatOptions::reindex()
        } else {
            ConcatOptions::preserve_ids()
        };
        match otap_batch {
            OtapArrowRecords::Logs(_) => concatenate_logs(&mut branch_results, concat_options),
            OtapArrowRecords::Metrics(_) => {
                concatenate_metrics(&mut branch_results, concat_options)
            }
            OtapArrowRecords::Traces(_) => concatenate_traces(&mut branch_results, concat_options),
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

            // evaluate the branch condition directly on the attributes record batch
            let predicate = branch
                .condition
                .evaluate_on_attrs_batch(&attrs_record_batch, &EvalContext::new(session_ctx))?;
            let predicate_selection_vec =
                scoped_value_to_boolean_array(predicate, attrs_record_batch.num_rows())?;

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

    fn supports_exec_on(&self, record_type: &RecordType) -> bool {
        matches!(record_type, RecordType::Attributes | RecordType::Signal)
    }

    // Propagate parent splitting required by any nested conditional branch.
    fn parent_behavior(&self) -> ParentBehavior {
        let requires_reindex = self.branches.iter().any(|branch| {
            branch
                .pipeline_stages
                .iter()
                .any(|stage| stage.parent_behavior().requires_reindex())
        }) || self.default_branch.as_ref().is_some_and(|branch| {
            branch
                .iter()
                .any(|stage| stage.parent_behavior().requires_reindex())
        });

        if requires_reindex {
            ParentBehavior::RequiresReindex
        } else {
            ParentBehavior::Preserves
        }
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
    use otel_arrow_contrib_data_engine_parser_abstractions::Parser;
    use otel_arrow_dfe_pdata::{
        otap::Logs,
        proto::opentelemetry::{
            arrow::v1::ArrowPayloadType,
            metrics::v1::Metric,
            trace::v1::{Status, span::SpanKind},
        },
        schema::consts,
        testing::round_trip::{otap_to_otlp, to_logs_data},
    };
    use otel_arrow_dfe_pdata::{
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                common::v1::{AnyValue, InstrumentationScope, KeyValue},
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                metrics::v1::{MetricsData, ResourceMetrics, ScopeMetrics},
                resource::v1::Resource,
                trace::v1::Span,
                trace::v1::{ResourceSpans, ScopeSpans},
            },
        },
        testing::round_trip::{otlp_to_otap, to_metrics_data, to_traces_data},
    };
    use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;

    use super::*;

    fn resource_with_id(id: &str) -> Resource {
        Resource::build()
            .attributes(vec![
                KeyValue::new("resource.id", AnyValue::new_string(id)),
                KeyValue::new("resource.keep", AnyValue::new_string("yes")),
            ])
            .finish()
    }

    fn shared_resource() -> Resource {
        resource_with_id("r1")
    }

    fn scope_with_pipeline_id(id: &str) -> InstrumentationScope {
        InstrumentationScope::build()
            .attributes(vec![
                KeyValue::new("pipeline.id", AnyValue::new_string(id)),
                KeyValue::new("scope.keep", AnyValue::new_string("yes")),
            ])
            .finish()
    }

    fn shared_scope() -> InstrumentationScope {
        scope_with_pipeline_id("p1")
    }

    /// Scenario: Log records sharing scope and resource attributes take different branches.
    /// Guarantees: Shared non-record attributes are emitted once after branch concatenation.
    #[tokio::test]
    async fn test_conditional_deduplicates_shared_log_attributes() {
        let input = LogsData::new(vec![ResourceLogs::new(
            shared_resource(),
            vec![ScopeLogs::new(
                shared_scope(),
                vec![
                    LogRecord::build().severity_text("a").finish(),
                    LogRecord::build().severity_text("b").finish(),
                ],
            )],
        )]);

        let result = exec_logs_pipeline::<OplParser>(
            r#"logs | if (severity_text == "a") { extend attributes["x"] = 1 }"#,
            input,
        )
        .await;

        assert_eq!(
            result.resource_logs[0].resource.as_ref().unwrap(),
            &shared_resource()
        );
        assert_eq!(
            result.resource_logs[0].scope_logs[0]
                .scope
                .as_ref()
                .unwrap(),
            &shared_scope()
        );
    }

    /// Scenario: Metrics sharing scope and resource attributes take different branches.
    /// Guarantees: Shared non-record attributes are emitted once after branch concatenation.
    #[tokio::test]
    async fn test_conditional_deduplicates_shared_metric_attributes() {
        let input = MetricsData::new(vec![ResourceMetrics::new(
            shared_resource(),
            vec![ScopeMetrics::new(
                shared_scope(),
                vec![
                    Metric::build().name("a").finish(),
                    Metric::build().name("b").finish(),
                ],
            )],
        )]);

        let result = exec_metrics_pipeline::<OplParser>(
            r#"metrics | if (name == "a") { set description = "selected" }"#,
            input,
        )
        .await;

        assert_eq!(
            result.resource_metrics[0].resource.as_ref().unwrap(),
            &shared_resource()
        );
        assert_eq!(
            result.resource_metrics[0].scope_metrics[0]
                .scope
                .as_ref()
                .unwrap(),
            &shared_scope()
        );
    }

    /// Scenario: Spans sharing scope and resource attributes take different branches.
    /// Guarantees: Shared non-record attributes are emitted once after branch concatenation.
    #[tokio::test]
    async fn test_conditional_deduplicates_shared_trace_attributes() {
        let input = otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::TracesData::new(vec![
            ResourceSpans::new(
                shared_resource(),
                vec![ScopeSpans::new(
                    shared_scope(),
                    vec![
                        Span::build()
                            .name("a")
                            .span_id([1; 8])
                            .trace_id([1; 16])
                            .status(Status::default())
                            .finish(),
                        Span::build()
                            .name("b")
                            .span_id([2; 8])
                            .trace_id([2; 16])
                            .status(Status::default())
                            .finish(),
                    ],
                )],
            ),
        ]);

        let result = exec_traces_pipeline::<OplParser>(
            r#"traces | if (name == "a") { set kind = 1 }"#,
            input,
        )
        .await;

        assert_eq!(
            result.resource_spans[0].resource.as_ref().unwrap(),
            &shared_resource()
        );
        assert_eq!(
            result.resource_spans[0].scope_spans[0]
                .scope
                .as_ref()
                .unwrap(),
            &shared_scope()
        );
    }

    /// Scenario: A selected branch mutates attributes shared with records in the default branch.
    /// Guarantees: The selected and default records use distinct parent identities and values.
    #[tokio::test]
    async fn test_conditional_preserves_shared_attribute_mutations() {
        let input = LogsData::new(vec![ResourceLogs::new(
            shared_resource(),
            vec![ScopeLogs::new(
                shared_scope(),
                vec![
                    LogRecord::build().severity_text("a").finish(),
                    LogRecord::build().severity_text("b").finish(),
                ],
            )],
        )]);

        let result = exec_logs_pipeline::<OplParser>(
            r#"
            logs | if (severity_text == "a") {
                set instrumentation_scope.attributes["pipeline.id"] = "modified" |
                set resource.attributes["resource.id"] = "modified"
            }"#,
            input,
        )
        .await;

        assert_eq!(result.resource_logs.len(), 2);
        assert_eq!(
            result.resource_logs[0].resource.as_ref().unwrap(),
            &resource_with_id("modified")
        );
        assert_eq!(
            result.resource_logs[0].scope_logs[0]
                .scope
                .as_ref()
                .unwrap(),
            &scope_with_pipeline_id("modified")
        );
        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records[0].severity_text,
            "a"
        );
        assert_eq!(
            result.resource_logs[1].resource.as_ref().unwrap(),
            &shared_resource()
        );
        assert_eq!(
            result.resource_logs[1].scope_logs[0]
                .scope
                .as_ref()
                .unwrap(),
            &shared_scope()
        );
        assert_eq!(
            result.resource_logs[1].scope_logs[0].log_records[0].severity_text,
            "b"
        );
    }

    /// Scenario: Only the default branch mutates attributes shared with a selected branch.
    /// Guarantees: The default and selected records use distinct parent identities and values.
    #[tokio::test]
    async fn test_conditional_preserves_default_branch_attribute_mutations() {
        let input = LogsData::new(vec![ResourceLogs::new(
            shared_resource(),
            vec![ScopeLogs::new(
                shared_scope(),
                vec![
                    LogRecord::build().severity_text("a").finish(),
                    LogRecord::build().severity_text("b").finish(),
                ],
            )],
        )]);

        let result = exec_logs_pipeline::<OplParser>(
            r#"
            logs | if (severity_text == "a") {
                set severity_number = 1
            } else {
                set instrumentation_scope.attributes["pipeline.id"] = "p2" |
                set resource.attributes["resource.id"] = "r2"
            }"#,
            input,
        )
        .await;

        assert_eq!(result.resource_logs.len(), 2);
        assert_eq!(
            result.resource_logs[0].resource.as_ref().unwrap(),
            &shared_resource()
        );
        assert_eq!(
            result.resource_logs[0].scope_logs[0]
                .scope
                .as_ref()
                .unwrap(),
            &shared_scope()
        );
        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records[0].severity_text,
            "a"
        );
        assert_eq!(
            result.resource_logs[1].resource.as_ref().unwrap(),
            &resource_with_id("r2")
        );
        assert_eq!(
            result.resource_logs[1].scope_logs[0]
                .scope
                .as_ref()
                .unwrap(),
            &scope_with_pipeline_id("p2")
        );
        assert_eq!(
            result.resource_logs[1].scope_logs[0].log_records[0].severity_text,
            "b"
        );
    }

    /// Scenario: A branch assigns a resource attribute its existing value.
    /// Guarantees: Conservative parent reindexing safely splits unchanged visible metadata.
    #[tokio::test]
    async fn test_conditional_allows_parent_reindex_false_positive() {
        let input = LogsData::new(vec![ResourceLogs::new(
            shared_resource(),
            vec![ScopeLogs::new(
                shared_scope(),
                vec![
                    LogRecord::build().severity_text("a").finish(),
                    LogRecord::build().severity_text("b").finish(),
                ],
            )],
        )]);

        let result = exec_logs_pipeline::<OplParser>(
            r#"
            logs | if (severity_text == "a") {
                set resource.attributes["resource.id"] = "r1"
            }"#,
            input,
        )
        .await;

        assert_eq!(result.resource_logs.len(), 2);
        for resource_logs in &result.resource_logs {
            assert_eq!(resource_logs.resource.as_ref().unwrap(), &shared_resource());
            assert_eq!(
                resource_logs.scope_logs[0].scope.as_ref().unwrap(),
                &shared_scope()
            );
        }
        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records[0].severity_text,
            "a"
        );
        assert_eq!(
            result.resource_logs[1].scope_logs[0].log_records[0].severity_text,
            "b"
        );
    }

    /// Scenario: A conditional branch forks a record into multiple reindexed branch results.
    /// Guarantees: Forked records and the conditional default retain distinct valid parents.
    #[tokio::test]
    async fn test_conditional_reindexes_forked_branch_results() {
        let input = LogsData::new(vec![ResourceLogs::new(
            shared_resource(),
            vec![ScopeLogs::new(
                shared_scope(),
                vec![
                    LogRecord::build().severity_text("a").finish(),
                    LogRecord::build().severity_text("b").finish(),
                ],
            )],
        )]);

        let result = exec_logs_pipeline::<OplParser>(
            r#"
            logs | if (severity_text == "a") {
                fork {
                    set event_name = "left"
                } {
                    set event_name = "right"
                }
            }"#,
            input,
        )
        .await;

        let records = result
            .resource_logs
            .iter()
            .flat_map(|resource| &resource.scope_logs)
            .flat_map(|scope| &scope.log_records)
            .map(|record| (record.severity_text.as_str(), record.event_name.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(records, vec![("a", "left"), ("a", "right"), ("b", "")]);
        assert_eq!(result.resource_logs.len(), 3);
    }

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
                rename attributes "x" as "y"
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

    /// Scenario: Evaluate a conditional branch using a nested serialized attribute leaf.
    /// Guarantees: The branch updates only records whose nested leaf matches.
    #[tokio::test]
    async fn test_conditional_with_nested_serialized_attribute() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "complex",
                    AnyValue::new_kvlist(vec![KeyValue::new("name", AnyValue::new_string("a"))]),
                )])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "complex",
                    AnyValue::new_kvlist(vec![KeyValue::new("name", AnyValue::new_string("b"))]),
                )])
                .finish(),
        ];

        let result = exec_logs_pipeline::<OplParser>(
            r#"
            logs | if (attributes["complex"]["name"] == "a") {
                set severity_text = "MATCHED"
            }"#,
            to_logs_data(log_records.clone()),
        )
        .await;
        let mut expected = log_records;
        expected[0].severity_text = "MATCHED".into();

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
                rename attributes "x" as "y"
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
                rename attributes "x" as "y"
            } else {
                rename attributes "x" as "z"
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
                rename attributes "x" as "y"
            } else if (event_name == "test") {
                rename attributes "x" as "z"
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
                rename attributes "x" as "y"
            } else if (severity_text == "ERROR") {
                rename attributes "x" as "z"
            } else if (severity_text == "WARN") {
                rename attributes "x" as "a"
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
                rename attributes "x" as "y"
            } else if (event_name == "test") {
                rename attributes "x" as "z"
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
