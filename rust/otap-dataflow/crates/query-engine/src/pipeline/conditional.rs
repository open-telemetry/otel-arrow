// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// TODO module docs

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::prelude::SessionContext;
use otap_df_pdata::OtapArrowRecords;

use crate::error::Result;
use crate::pipeline::filter::{Composite, FilterExec};
use crate::pipeline::{BoxedPipelineStage, PipelineStage};

/// TODO comments on what its' doing
pub struct ConditionalPipelineStage {
    /// TODO comments
    branches: Vec<ConditionalPipelineStageBranch>,
    /// TODO comments
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

/// TODO comments
pub struct ConditionalPipelineStageBranch {
    /// TODO comments
    predicate: Composite<FilterExec>,
    /// TODO comments
    pipeline_stages: Vec<BoxedPipelineStage>,
}

impl ConditionalPipelineStageBranch {
    pub fn new(predicate: Composite<FilterExec>, pipeline_stages: Vec<BoxedPipelineStage>) -> Self {
        Self {
            predicate,
            pipeline_stages,
        }
    }
}

#[async_trait(?Send)]
impl PipelineStage for ConditionalPipelineStage {
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
    ) -> Result<OtapArrowRecords> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use data_engine_expressions::{
        ConditionalExpression, ConditionalExpressionBranch, DataExpression, LogicalExpression,
        PipelineExpression, PipelineExpressionBuilder, QueryLocation,
    };
    use data_engine_kql_parser::{KqlParser, Parser};
    use otap_df_pdata::proto::opentelemetry::{
        common::v1::{AnyValue, KeyValue},
        logs::v1::LogRecord,
    };

    use crate::pipeline::test::{exec_logs_pipeline_expr, to_logs_data};

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

                branch_exprs.push(ConditionalExpressionBranch::new(
                    QueryLocation::new_fake(),
                    condition,
                    exprs_pipeline.get_expressions().to_vec(),
                ));
            }

            let mut conditional_expr = ConditionalExpression::new(QueryLocation::new_fake());
            for branch in branch_exprs {
                conditional_expr = conditional_expr.with_branch(branch);
            }

            PipelineExpressionBuilder::new(&"test")
                .with_expressions(vec![DataExpression::Conditional(conditional_expr)])
                .build()
                .unwrap()
        }
    }

    #[tokio::test]
    async fn test_conditional_simple() {
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
                "project-rename attributes[\"x\"] = attributes[\"y\"]",
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
}
