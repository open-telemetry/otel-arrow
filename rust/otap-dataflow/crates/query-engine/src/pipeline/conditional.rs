// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementation of a pipeline stage that can optionally apply pipeline
//! stages to rows that match some predicate conditions.

use std::sync::Arc;

use arrow::array::BooleanArray;
use arrow::buffer::BooleanBuffer;
use arrow::compute::{and, concat_batches, not, or};
use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::prelude::SessionContext;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::{Logs, Metrics, Traces};

use crate::error::Result;
use crate::pipeline::filter::{Composite, FilterExec, filter_otap_batch};
use crate::pipeline::{BoxedPipelineStage, PipelineStage};

/// This [`PipelineStage`] implementation will conditionally execute child pipeline stages on rows
/// which match some condition. This can be used to execute `if/else if/else` type control flow for
/// items in a telemetry batch.
///
/// When executed, this will evaluate the pipeline in each branch on a disjoint set of rows that are
/// selected by the branches' conditions. The output will be the results of each branch & default
/// branch concatenated/union'd together.
pub struct ConditionalPipelineStage {
    /// The data in the batches will be checked against the conditions for each branch, and the
    /// stages in each branch will be executed for the rows that pass the condition and did not
    /// pass the condition for previous branches.
    ///
    /// These branches can be thought of like `if`/`else if` control flow structures.
    branches: Vec<ConditionalPipelineStageBranch>,

    /// The "default branch", if not `None`, will be executed for rows that did not pass a
    /// condition in any of the branches. If this branch is `None`, the remaining rows will be
    /// appended to the result batch with no transformation.
    ///
    /// This is analogous to the `else` branch.
    default_branch: Option<Vec<BoxedPipelineStage>>,
}

impl ConditionalPipelineStage {
    pub fn new(
        branches: Vec<ConditionalPipelineStageBranch>,
        default_branch: Option<Vec<BoxedPipelineStage>>,
    ) -> Self {
        Self {
            branches,
            default_branch,
        }
    }
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
    ) -> Result<OtapArrowRecords> {
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
            // each branch. There's probably a some optimization we can make here if this becomes
            // a bottleneck:
            // if previous branches have been very selective, we might consider materializing a
            // batch specifically containing the rows that have not already been selected and
            // feeding that into next iterations. This is extra overhead but the resulting batch
            // would have less rows which could make filter faster
            let predicate_selection_vec = branch.condition.execute(&otap_batch, session_ctx)?;

            let branch_selection_vec = and(&predicate_selection_vec, &not(&already_selected_vec)?)?;

            // update the list of rows that were already selected by branches
            already_selected_vec = or(&already_selected_vec, &predicate_selection_vec)?;

            // create a batch with only the rows that match the condition
            //
            // TODO: the function we're calling here will materialize all the child record batches
            // with parent_ids of rows associated with the selected parent rows. There might be an
            // optimization to here where we don't do this for every branch for every batch. For
            // example, if the branch doesn't read or write some child batch, we could avoid
            // materializing it and sync up the rows once all branches have been evaluated.
            let mut branch_otap_batch =
                filter_otap_batch(&branch_selection_vec, otap_batch.clone())?;

            for stage in &mut branch.pipeline_stages {
                branch_otap_batch = stage
                    .execute(
                        branch_otap_batch,
                        session_ctx,
                        config_options,
                        task_context.clone(),
                    )
                    .await?;
            }

            branch_results.push(branch_otap_batch);
        }

        // handle the default branch - e.g. the rows that did not match the condition from any of
        // the previous branches
        if already_selected_vec.true_count() != root_batch.num_rows() {
            let mut default_branch_batch =
                filter_otap_batch(&not(&already_selected_vec)?, otap_batch.clone())?;

            if let Some(default_branch) = self.default_branch.as_mut() {
                for stage in default_branch {
                    default_branch_batch = stage
                        .execute(
                            default_branch_batch,
                            session_ctx,
                            config_options,
                            task_context.clone(),
                        )
                        .await?;
                }
            }
            branch_results.push(default_branch_batch);
        }

        // reconstruct the result with the results of each branch
        let mut result = match otap_batch {
            OtapArrowRecords::Logs(_) => OtapArrowRecords::Logs(Logs::default()),
            OtapArrowRecords::Traces(_) => OtapArrowRecords::Traces(Traces::default()),
            OtapArrowRecords::Metrics(_) => OtapArrowRecords::Metrics(Metrics::default()),
        };

        // concat all the record batches for each payload type and set in the result
        for payload_type in otap_batch.allowed_payload_types() {
            // TODO - when we have filter stages that can modify the schema, such as adding or
            // deleting columns, we'll need extra logic here to combine project the record batches
            // we're iterating to a common superset schema.

            let schema = branch_results
                .iter()
                .filter_map(|branch_result| branch_result.get(*payload_type))
                .map(|record_batch| record_batch.schema())
                .next();

            if let Some(schema) = schema {
                let record_batch = concat_batches(
                    &schema,
                    branch_results
                        .iter()
                        .filter_map(|branch_result| branch_result.get(*payload_type)),
                )?;
                result.set(*payload_type, record_batch);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use data_engine_expressions::{
        ConditionalDataExpression, ConditionalDataExpressionBranch, DataExpression,
        LogicalExpression, PipelineExpression, PipelineExpressionBuilder, QueryLocation,
    };
    use data_engine_kql_parser::{KqlParser, Parser};
    use otap_df_pdata::proto::opentelemetry::{
        common::v1::{AnyValue, KeyValue},
        logs::v1::LogRecord,
    };

    use crate::pipeline::{
        Pipeline,
        test::{exec_logs_pipeline_expr, to_logs_data},
    };

    use super::*;

    /// helper for constructing a pipeline with a conditional data expression.
    ///
    /// this is needed because we don't yet have parser implemented for these types of expressions
    #[derive(Default)]
    struct ConditionalTest {
        /// tuples of (logical_expr, data_exprs) in kql
        branches: Vec<(&'static str, &'static str)>,

        /// data expr for the default branch in kql
        default_branch: Option<&'static str>,
    }

    impl ConditionalTest {
        fn as_logs_pipeline(&self) -> PipelineExpression {
            let mut branch_exprs = vec![];
            for (condition, data_exprs) in &self.branches {
                let pipeline = KqlParser::parse(&format!("logs | where {condition}"))
                    .unwrap()
                    .pipeline;

                println!("pipeline {}", pipeline);
                let condition = match pipeline.get_expressions().first() {
                    Some(DataExpression::Discard(discard)) => match discard.get_predicate() {
                        Some(LogicalExpression::Not(not)) => not.get_inner_expression().clone(),
                        // shouldn't happen unless we change how we parse discard expressions
                        other => unreachable!("unexpected expr {other:?}"),
                    },
                    // shouldn't happen as the pipeline has an expression
                    other => unreachable!("unexpected expr {other:?}"),
                };

                let exprs_pipeline = KqlParser::parse(&format!("logs | {data_exprs}"))
                    .unwrap()
                    .pipeline;

                branch_exprs.push(ConditionalDataExpressionBranch::new(
                    QueryLocation::new_fake(),
                    condition,
                    exprs_pipeline.get_expressions().to_vec(),
                ));
            }

            let mut conditional_expr = ConditionalDataExpression::new(QueryLocation::new_fake());
            for branch in branch_exprs {
                conditional_expr = conditional_expr.with_branch(branch);
            }

            if let Some(data_exprs) = self.default_branch {
                let exprs_pipeline = KqlParser::parse(&format!("logs | {data_exprs}"))
                    .unwrap()
                    .pipeline;
                conditional_expr =
                    conditional_expr.with_default_branch(exprs_pipeline.get_expressions().to_vec());
            }

            PipelineExpressionBuilder::new(&"test")
                .with_expressions(vec![DataExpression::Conditional(conditional_expr)])
                .build()
                .unwrap()
        }
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

        let pipeline_expr = ConditionalTest {
            branches: vec![(
                "severity_text == \"ERROR\"",
                "project-rename attributes[\"y\"] = attributes[\"x\"]",
            )],
            ..Default::default()
        }
        .as_logs_pipeline();

        let result = exec_logs_pipeline_expr(pipeline_expr, to_logs_data(log_records)).await;

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

        let pipeline_expr = ConditionalTest {
            branches: vec![(
                "severity_text == \"ERROR\"",
                "project-rename attributes[\"y\"] = attributes[\"x\"]",
            )],
            default_branch: Some("project-rename attributes[\"z\"] = attributes[\"x\"]"),
        }
        .as_logs_pipeline();

        let result = exec_logs_pipeline_expr(pipeline_expr, to_logs_data(log_records)).await;

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

        let pipeline_expr = ConditionalTest {
            branches: vec![
                (
                    "severity_text == \"ERROR\"",
                    "project-rename attributes[\"y\"] = attributes[\"x\"]",
                ),
                (
                    "event_name == \"test\"",
                    "project-rename attributes[\"z\"] = attributes[\"x\"]",
                ),
            ],
            default_branch: Some("project-away attributes[\"x\"]"),
        }
        .as_logs_pipeline();

        let result = exec_logs_pipeline_expr(pipeline_expr, to_logs_data(log_records)).await;

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
        // correct results when we use that code bath
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

        let pipeline_expr = ConditionalTest {
            branches: vec![
                (
                    "severity_text == \"INFO\"",
                    "project-rename attributes[\"y\"] = attributes[\"x\"]",
                ),
                (
                    "severity_text == \"ERROR\"",
                    "project-rename attributes[\"z\"] = attributes[\"x\"]",
                ),
                (
                    "severity_text == \"WARN\"",
                    "project-rename attributes[\"a\"] = attributes[\"x\"]",
                ),
            ],
            default_branch: Some("project-away attributes[\"x\"]"),
        }
        .as_logs_pipeline();

        let result = exec_logs_pipeline_expr(pipeline_expr, to_logs_data(log_records)).await;

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
        let pipeline_expr = ConditionalTest {
            branches: vec![
                (
                    "severity_text == \"ERROR\"",
                    "project-rename attributes[\"y\"] = attributes[\"x\"]",
                ),
                (
                    "event_name == \"test\"",
                    "project-rename attributes[\"z\"] = attributes[\"x\"]",
                ),
            ],
            default_branch: Some("project-away attributes[\"x\"]"),
        }
        .as_logs_pipeline();
        let mut pipeline = Pipeline::new(pipeline_expr);

        let input = OtapArrowRecords::Logs(Logs::default());
        let result = pipeline.execute(input.clone()).await.unwrap();
        assert_eq!(result, input);
    }
}
