// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    LogicalExpression, OrLogicalExpression, ScalarExpression, StaticScalarExpression,
};
use datafusion::common::JoinType;
use datafusion::functions::core::expr_ext::FieldAccessor;
use datafusion::functions::core::expr_fn::coalesce;
use datafusion::logical_expr::{
    Expr, LogicalPlan, LogicalPlanBuilder, Operator, and, binary_expr, col, lit, not, or,
};
use datafusion::scalar::ScalarValue;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::consts;

use crate::common::{
    AttributesIdentifier, ColumnAccessor, try_static_scalar_to_any_val_column,
    try_static_scalar_to_literal,
};
use crate::consts::ROW_NUMBER_COL;
use crate::engine::ExecutionContext;
use crate::error::{Error, Result};

pub struct Filter {
    pub(crate) filter_expr: Option<Expr>,
    pub(crate) join: Option<FilteringJoin>,
}

impl Filter {
    pub fn try_from_predicate(
        exec_ctx: &ExecutionContext,
        predicate: &LogicalExpression,
    ) -> Result<Self> {
        match predicate {
            LogicalExpression::And(and_expr) => {
                let left_filter = Self::try_from_predicate(exec_ctx, and_expr.get_left())?;
                let right_filter = Self::try_from_predicate(exec_ctx, and_expr.get_right())?;

                let filter_expr = match (left_filter.filter_expr, right_filter.filter_expr) {
                    (Some(left), Some(right)) => Some(left.and(right)),
                    (None, Some(filter_expr)) | (Some(filter_expr), None) => Some(filter_expr),
                    _ => None,
                };

                let join = match (left_filter.join, right_filter.join) {
                    (Some(left_join), Some(right_join)) => {
                        let mut plan = exec_ctx.root_batch_plan()?;
                        plan = left_join.join_to_plan(plan)?;
                        plan = right_join.join_to_plan(plan)?;

                        Some(FilteringJoin {
                            logical_plan: plan.build()?,
                            join_type: JoinType::LeftSemi,
                            condition: FilteringJoinCondition::MatchingColumnPairs(
                                ROW_NUMBER_COL,
                                ROW_NUMBER_COL,
                            ),
                        })
                    }
                    (None, Some(join)) | (Some(join), None) => Some(join),
                    _ => None,
                };

                Ok(Self { filter_expr, join })
            }

            LogicalExpression::Or(or_expr) => Self::try_from_or_expr(exec_ctx, or_expr),

            LogicalExpression::Not(not_expr) => {
                let not_filter =
                    Self::try_from_predicate(exec_ctx, not_expr.get_inner_expression())?;
                if let Some(not_join) = not_filter.join {
                    let mut plan = exec_ctx.root_batch_plan()?;
                    if let Some(filter_expr) = not_filter.filter_expr {
                        plan = plan.filter(filter_expr)?;
                    }

                    plan = not_join.join_to_plan(plan)?;
                    Ok(Self {
                        filter_expr: None,
                        join: Some(FilteringJoin {
                            logical_plan: plan.build()?,
                            join_type: JoinType::LeftAnti,
                            condition: FilteringJoinCondition::MatchingColumnPairs(
                                ROW_NUMBER_COL,
                                ROW_NUMBER_COL,
                            ),
                        }),
                    })
                } else {
                    Ok(Self {
                        filter_expr: not_filter.filter_expr.map(not),
                        join: None,
                    })
                }
            }

            LogicalExpression::EqualTo(eq_expr) => Self::try_from_binary_expr(
                exec_ctx,
                Operator::Eq,
                eq_expr.get_left(),
                eq_expr.get_right(),
            ),
            expr => Err(Error::NotYetSupportedError {
                message: format!("filtering predicate {:?}", expr),
            }),
        }
    }

    fn try_from_or_expr(
        exec_ctx: &ExecutionContext,
        or_expr: &OrLogicalExpression,
    ) -> Result<Self> {
        // TODO -- there are two things that might be inefficient in the handling of filters
        // with or here that we should optimize.
        //
        // first case
        // - when both sides of the or require a join to evaluate, but the same table is what's
        //   on the right of the join in both branches, instead of joining both tables and
        //   doing a union/distinct on the result, then joining back to the main table, we can
        //   just create an `or` filter on the right table and join that back to the parent
        //
        // second case:
        // - basically what we're doing here is evaluating all filters inside both sides of the
        //   or locally, union/distinct both sides, and join this back to the parent table. But
        //   if the parent table had some filters applied to it, we could eliminate this join
        //   by pushing the filters down to the both sides of the or evaluation. This might come
        //   at the cost of evaluating the filters multiple times however, so need further
        //   investigation.
        //

        let left_filter = Self::try_from_predicate(exec_ctx, or_expr.get_left())?;
        let right_filter = Self::try_from_predicate(exec_ctx, or_expr.get_right())?;

        match (left_filter.join, right_filter.join) {
            (Some(left_join), Some(right_join)) => {
                let mut left_plan = exec_ctx.root_batch_plan()?;
                if let Some(filter_expr) = left_filter.filter_expr {
                    left_plan = left_plan.filter(filter_expr)?;
                }
                left_plan = left_join.join_to_plan(left_plan)?;

                let mut right_plan = exec_ctx.root_batch_plan()?;
                if let Some(filter_expr) = right_filter.filter_expr {
                    right_plan = right_plan.filter(filter_expr)?;
                }
                right_plan = right_join.join_to_plan(right_plan)?;

                Ok(Self {
                    filter_expr: None,
                    join: Some(FilteringJoin {
                        logical_plan: left_plan
                            .union(right_plan.build()?)?
                            .distinct_on(
                                vec![col(ROW_NUMBER_COL)],
                                vec![col(ROW_NUMBER_COL)],
                                None,
                            )?
                            .build()?,
                        join_type: JoinType::LeftSemi,
                        condition: FilteringJoinCondition::MatchingColumnPairs(
                            ROW_NUMBER_COL,
                            ROW_NUMBER_COL,
                        ),
                    }),
                })
            }

            (Some(left_join), None) => {
                let mut left_plan = exec_ctx.root_batch_plan()?;
                if let Some(filter_expr) = left_filter.filter_expr {
                    left_plan = left_plan.filter(filter_expr)?;
                }
                left_plan = left_join.join_to_plan(left_plan)?;

                let mut right_plan = exec_ctx.root_batch_plan()?;
                if let Some(filter_expr) = right_filter.filter_expr {
                    right_plan = right_plan.filter(filter_expr)?;
                }

                Ok(Self {
                    filter_expr: None,
                    join: Some(FilteringJoin {
                        logical_plan: left_plan
                            .union(right_plan.build()?)?
                            .distinct_on(
                                vec![col(ROW_NUMBER_COL)],
                                vec![col(ROW_NUMBER_COL)],
                                None,
                            )?
                            .build()?,
                        join_type: JoinType::LeftSemi,
                        condition: FilteringJoinCondition::MatchingColumnPairs(
                            ROW_NUMBER_COL,
                            ROW_NUMBER_COL,
                        ),
                    }),
                })
            }

            (None, Some(right_join)) => {
                let mut left_plan = exec_ctx.root_batch_plan()?;
                if let Some(filter_expr) = left_filter.filter_expr {
                    left_plan = left_plan.filter(filter_expr)?;
                }

                let mut right_plan = exec_ctx.root_batch_plan()?;
                if let Some(filter_expr) = right_filter.filter_expr {
                    right_plan = right_plan.filter(filter_expr)?;
                }
                right_plan = right_join.join_to_plan(right_plan)?;

                Ok(Self {
                    filter_expr: None,
                    join: Some(FilteringJoin {
                        logical_plan: left_plan
                            .union(right_plan.build()?)?
                            .distinct_on(
                                vec![col(ROW_NUMBER_COL)],
                                vec![col(ROW_NUMBER_COL)],
                                None,
                            )?
                            .build()?,
                        join_type: JoinType::LeftSemi,
                        condition: FilteringJoinCondition::MatchingColumnPairs(
                            ROW_NUMBER_COL,
                            ROW_NUMBER_COL,
                        ),
                    }),
                })
            }

            (None, None) => match (left_filter.filter_expr, right_filter.filter_expr) {
                (Some(left_filter), Some(right_filter)) => Ok(Self {
                    filter_expr: Some(or(left_filter, right_filter)),
                    join: None,
                }),
                (Some(filter_expr), None) | (None, Some(filter_expr)) => Ok(Self {
                    filter_expr: Some(filter_expr),
                    join: None,
                }),
                _ => Ok(Self {
                    filter_expr: None,
                    join: None,
                }),
            },
        }
    }

    fn try_from_binary_expr(
        exec_ctx: &ExecutionContext,
        operator: Operator,
        left: &ScalarExpression,
        right: &ScalarExpression,
    ) -> Result<Self> {
        let left_arg = BinaryArg::try_from(left)?;
        let right_arg = BinaryArg::try_from(right)?;

        match left_arg {
            BinaryArg::Column(left_col) => {
                let left_col_exists = exec_ctx.column_exists(&left_col)?;
                match left_col {
                    ColumnAccessor::ColumnName(col_name) => match right_arg {
                        BinaryArg::Column(right_col) => Err(Error::NotYetSupportedError {
                            message: format!(
                                "column on right side of binary expr. received {:?}",
                                right_col,
                            ),
                        }),
                        BinaryArg::Literal(static_scalar) => {
                            let left = if left_col_exists {
                                col(col_name)
                            } else {
                                lit(ScalarValue::Null)
                            };
                            let right = try_static_scalar_to_literal(&static_scalar)?;
                            Ok(Self {
                                // TODO figure out if `coalesce`` is needed here. this was added to
                                // cover the case where left_col is not existing, in which case we
                                // create an expr like: `lit(Null) <operator> lit(<right>)`.
                                //
                                // without coalesce, this gets optimized to lit(Bool(null)) by
                                // [`datafusion::optimizer::ExprSimplifier`] but if we use this
                                // this in the context of a `not` expr, e.g.
                                // `not(lit(null) <operator> lit(right))`` this also gets optimized
                                // to bool(null) even though it should be `true` always if right is
                                // not null.
                                filter_expr: Some(coalesce(vec![
                                    binary_expr(left, operator, right),
                                    lit(false),
                                ])),
                                join: None,
                            })
                        }
                    },
                    ColumnAccessor::StructCol(struct_col, struct_field) => match right_arg {
                        BinaryArg::Column(right_col) => Err(Error::NotYetSupportedError {
                            message: format!(
                                "column on right side of binary expr. received {:?}",
                                right_col,
                            ),
                        }),
                        BinaryArg::Literal(static_scalar) => {
                            let left = col(struct_col).field(struct_field);
                            let right = try_static_scalar_to_literal(&static_scalar)?;
                            Ok(Self {
                                filter_expr: Some(binary_expr(left, operator, right)),
                                join: None,
                            })
                        }
                    },
                    ColumnAccessor::Attributes(attrs_identifier, attr_key) => match right_arg {
                        BinaryArg::Column(right_col) => Err(Error::NotYetSupportedError {
                            message: format!(
                                "column on right side of binary expr. received {:?}",
                                right_col,
                            ),
                        }),
                        BinaryArg::Literal(static_scalar) => {
                            let attr_val_col_name =
                                try_static_scalar_to_any_val_column(&static_scalar)?;

                            let attrs_payload_type = match attrs_identifier {
                                AttributesIdentifier::Root => {
                                    match exec_ctx.root_batch_payload_type()? {
                                        ArrowPayloadType::Logs => ArrowPayloadType::LogAttrs,
                                        ArrowPayloadType::Spans => ArrowPayloadType::SpanAttrs,
                                        ArrowPayloadType::MultivariateMetrics
                                        | ArrowPayloadType::UnivariateMetrics => {
                                            ArrowPayloadType::MetricAttrs
                                        }
                                        _ => {
                                            unreachable!("invalid root payload type")
                                        }
                                    }
                                }
                                AttributesIdentifier::NonRoot(payload_type) => payload_type,
                            };
                            let attrs_filter =
                                exec_ctx.scan_batch_plan(attrs_payload_type)?.filter(and(
                                    binary_expr(
                                        col(consts::ATTRIBUTE_KEY),
                                        operator,
                                        lit(attr_key),
                                    ),
                                    binary_expr(
                                        col(attr_val_col_name),
                                        operator,
                                        try_static_scalar_to_literal(&static_scalar)?,
                                    ),
                                ))?;

                            let join_condition = match attrs_payload_type {
                                ArrowPayloadType::ResourceAttrs => FilteringJoinCondition::Filter(
                                    col(consts::RESOURCE)
                                        .field(consts::ID)
                                        .eq(col(consts::PARENT_ID)),
                                ),
                                ArrowPayloadType::ScopeAttrs => FilteringJoinCondition::Filter(
                                    col(consts::SCOPE)
                                        .field(consts::ID)
                                        .eq(col(consts::PARENT_ID)),
                                ),
                                _ => FilteringJoinCondition::MatchingColumnPairs(
                                    consts::ID,
                                    consts::PARENT_ID,
                                ),
                            };
                            Ok(Self {
                                filter_expr: None,
                                join: Some(FilteringJoin {
                                    logical_plan: attrs_filter.build()?,
                                    join_type: JoinType::LeftSemi,
                                    condition: join_condition,
                                }),
                            })
                        }
                    },
                }
            }
            _ => Err(Error::NotYetSupportedError {
                message: format!(
                    "expression for left side of binary expr. received {:?}",
                    left,
                ),
            }),
        }
    }
}

#[derive(Debug)]
pub struct FilteringJoin {
    pub(crate) logical_plan: LogicalPlan,
    pub(crate) join_type: JoinType,
    condition: FilteringJoinCondition,
}

#[derive(Debug)]
enum FilteringJoinCondition {
    Filter(Expr),
    MatchingColumnPairs(&'static str, &'static str),
}

impl FilteringJoin {
    pub fn join_to_plan(self, plan_builder: LogicalPlanBuilder) -> Result<LogicalPlanBuilder> {
        Ok(match self.condition {
            FilteringJoinCondition::MatchingColumnPairs(left_col, right_col) => plan_builder.join(
                self.logical_plan,
                self.join_type,
                (vec![left_col], vec![right_col]),
                None,
            )?,

            FilteringJoinCondition::Filter(join_filter_expr) => {
                plan_builder.join_on(self.logical_plan, self.join_type, [join_filter_expr])?
            }
        })
    }
}

enum BinaryArg {
    Column(ColumnAccessor),
    Literal(StaticScalarExpression),
}

impl TryFrom<&ScalarExpression> for BinaryArg {
    type Error = Error;

    fn try_from(scalar_expr: &ScalarExpression) -> Result<Self> {
        let binary_arg = match scalar_expr {
            ScalarExpression::Source(source) => {
                BinaryArg::Column(ColumnAccessor::try_from(source.get_value_accessor())?)
            }
            ScalarExpression::Static(static_expr) => BinaryArg::Literal(static_expr.clone()),
            expr => {
                return Err(Error::NotYetSupportedError {
                    message: format!(
                        "expression type not yet supported as argument to binary operation. received {:?}",
                        expr,
                    ),
                });
            }
        };

        Ok(binary_arg)
    }
}

#[cfg(test)]
mod test {
    use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_rust::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;

    use crate::test::{logs_to_export_req, run_logs_test};

    #[tokio::test]
    async fn filter_simple() {
        let export_req = logs_to_export_req(vec![
            LogRecord {
                event_name: "1".into(),
                severity_text: "WARN".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "2".into(),
                severity_text: "INFO".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "3".into(),
                severity_text: "WARN".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "4".into(),
                severity_text: "DEBUG".into(),
                ..Default::default()
            },
        ]);
        run_logs_test(
            export_req,
            "logs | where severity_text == \"WARN\"",
            vec!["1".into(), "3".into()],
        )
        .await;
    }

    #[tokio::test]
    async fn filter_with_predicate_containing_missing_optional_field() {
        let export_req = logs_to_export_req(vec![
            LogRecord {
                event_name: "1".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "2".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "3".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "4".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "5".into(),
                attributes: vec![KeyValue::new("X2", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
        ]);

        run_logs_test(
            export_req.clone(),
            "logs | where severity_text == \"WARN\"",
            vec![],
        )
        .await;

        run_logs_test(
            export_req,
            "logs | where not(severity_text == \"WARN\")",
            (1..6).map(|i| format!("{i}")).collect(),
        )
        .await;
    }

    #[tokio::test]
    async fn filter_attrs_simple() {
        let export_req = logs_to_export_req(vec![
            LogRecord {
                event_name: "1".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "2".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "3".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "4".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "5".into(),
                attributes: vec![KeyValue::new("X2", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
        ]);
        run_logs_test(
            export_req,
            "logs | where attributes[\"X\"] == \"Y\"",
            vec!["1".into(), "4".into()],
        )
        .await;
    }

    #[tokio::test]
    async fn filter_logical_or() {
        let export_req = logs_to_export_req(vec![
            LogRecord {
                event_name: "1".into(),
                severity_text: "WARN".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "2".into(),
                severity_text: "INFO".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "3".into(),
                severity_text: "WARN".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "4".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "5".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![KeyValue::new("X2", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
        ]);

        // this query can be resolved with a simple "or" logical expr on the root table
        run_logs_test(
            export_req.clone(),
            "logs | where severity_text == \"WARN\" or severity_text == \"INFO\"",
            vec!["1".into(), "2".into(), "3".into()],
        )
        .await;

        // this query can be resolved by a simple "or" logical on the attrs table
        run_logs_test(
            export_req.clone(),
            "logs | where attributes[\"X\"] == \"Y\" or attributes[\"X\"] == \"Y2\"",
            vec!["1".into(), "3".into(), "4".into()],
        )
        .await;

        // this query is more complex, need to filter root, then filter attrs and join to root,
        // then take the distinct union of these two clauses
        run_logs_test(
            export_req,
            "logs | where attributes[\"X\"] == \"Y\" or severity_text == \"INFO\"",
            vec!["1".into(), "2".into(), "4".into()],
        )
        .await;
    }

    #[tokio::test]
    async fn filter_logical_and() {
        let export_req = logs_to_export_req(vec![
            LogRecord {
                event_name: "1".into(),
                severity_text: "WARN".into(),
                attributes: vec![
                    KeyValue::new("X", AnyValue::new_string("Y")),
                    KeyValue::new("X2", AnyValue::new_string("Y2")),
                ],
                ..Default::default()
            },
            LogRecord {
                event_name: "2".into(),
                severity_text: "INFO".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "3".into(),
                severity_text: "WARN".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "4".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "5".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![
                    KeyValue::new("X", AnyValue::new_string("Y")),
                    KeyValue::new("X2", AnyValue::new_string("Y2")),
                ],
                ..Default::default()
            },
        ]);

        run_logs_test(
            export_req.clone(),
            "logs | where severity_text == \"WARN\" and event_name == \"1\"",
            vec!["1".into()],
        )
        .await;

        // when attributes are filtered twice like this, we need filter attributes table twice,
        // then LeftSemi join all the results
        run_logs_test(
            export_req.clone(),
            "logs | where attributes[\"X\"] == \"Y\" and attributes[\"X2\"] == \"Y2\"",
            vec!["1".into(), "5".into()],
        )
        .await;

        run_logs_test(
            export_req,
            "logs | where severity_text == \"DEBUG\" and attributes[\"X\"] == \"Y\" and attributes[\"X2\"] == \"Y2\"",
            vec!["5".into()],
        )
        .await;
    }

    #[tokio::test]
    async fn filter_logical_and_with_or_together() {
        let export_req = logs_to_export_req(vec![
            LogRecord {
                event_name: "1".into(),
                severity_text: "WARN".into(),
                attributes: vec![
                    KeyValue::new("X", AnyValue::new_string("Y")),
                    KeyValue::new("X2", AnyValue::new_string("Y2")),
                ],
                ..Default::default()
            },
            LogRecord {
                event_name: "2".into(),
                severity_text: "INFO".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "3".into(),
                severity_text: "WARN".into(),
                attributes: vec![
                    KeyValue::new("X", AnyValue::new_string("Y2")),
                    KeyValue::new("X2", AnyValue::new_string("Y2")),
                ],
                ..Default::default()
            },
            LogRecord {
                event_name: "4".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "5".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
        ]);

        // this will have to union distinct for the or clause, but it will not have to join the result
        // (1, 4) and ((4, 5) or (1, 3))
        // (1, 4) and (1, 3, 4, 5)
        // (1, 4)
        run_logs_test(
            export_req.clone(),
            "logs | where attributes[\"X\"] == \"Y\" and (severity_text == \"DEBUG\" or attributes[\"X2\"] == \"Y2\")", 
            vec!["1".into(), "4".into()]
        ).await;

        // same as above, just ensuring we handle it correctly when the side requiring the sub
        // queries is on the left
        run_logs_test(
            export_req.clone(),
            "logs | where (severity_text == \"DEBUG\" or attributes[\"X2\"] == \"Y2\") and attributes[\"X\"] == \"Y\"", 
            vec!["1".into(), "4".into()]
        ).await;

        // this query will need to union distinct for each nested or, and then join the results
        // ((1, 4) or (4, 5)) and ((3, 5) or (1, 3))
        // (1, 4, 5) and (1, 3, 5)
        // (1, 5)
        run_logs_test(
            export_req.clone(),
            "logs | where (attributes[\"X\"] == \"Y\" or severity_text == \"DEBUG\") and (attributes[\"X\"] == \"Y2\" or severity_text == \"WARN\")",
            vec!["1".into(), "5".into()],
        )
        .await;
    }

    #[tokio::test]
    async fn filter_logical_not_expr() {
        let export_req = logs_to_export_req(vec![
            LogRecord {
                event_name: "1".into(),
                severity_text: "WARN".into(),
                attributes: vec![
                    KeyValue::new("X", AnyValue::new_string("Y")),
                    KeyValue::new("X2", AnyValue::new_string("Y2")),
                ],
                ..Default::default()
            },
            LogRecord {
                event_name: "2".into(),
                severity_text: "INFO".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "3".into(),
                severity_text: "WARN".into(),
                attributes: vec![
                    KeyValue::new("X", AnyValue::new_string("Y2")),
                    KeyValue::new("X2", AnyValue::new_string("Y2")),
                ],
                ..Default::default()
            },
            LogRecord {
                event_name: "4".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "5".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
        ]);

        run_logs_test(
            export_req.clone(),
            "logs | where not(severity_text == \"WARN\")",
            vec!["2".into(), "4".into(), "5".into()],
        )
        .await;

        run_logs_test(
            export_req.clone(),
            "logs | where not(attributes[\"X\"] == \"Y\")",
            vec!["2".into(), "3".into(), "5".into()],
        )
        .await;
    }

    #[tokio::test]
    async fn filter_inverting_and_or_with_not() {
        let export_req = logs_to_export_req(vec![
            LogRecord {
                event_name: "1".into(),
                severity_text: "WARN".into(),
                attributes: vec![
                    KeyValue::new("X", AnyValue::new_string("Y")),
                    KeyValue::new("X2", AnyValue::new_string("Y2")),
                ],
                ..Default::default()
            },
            LogRecord {
                event_name: "2".into(),
                severity_text: "INFO".into(),
                ..Default::default()
            },
            LogRecord {
                event_name: "3".into(),
                severity_text: "WARN".into(),
                attributes: vec![
                    KeyValue::new("X", AnyValue::new_string("Y2")),
                    KeyValue::new("X2", AnyValue::new_string("Y2")),
                ],
                ..Default::default()
            },
            LogRecord {
                event_name: "4".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                ..Default::default()
            },
            LogRecord {
                event_name: "5".into(),
                severity_text: "DEBUG".into(),
                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                ..Default::default()
            },
        ]);

        run_logs_test(
            export_req.clone(),
            "logs | where not(attributes[\"X\"] == \"Y\" and attributes[\"X2\"] == \"Y2\")",
            ["2", "3", "4", "5"].iter().map(|i| i.to_string()).collect(),
        )
        .await;

        run_logs_test(
            export_req.clone(),
            "logs | where not(severity_text == \"WARN\" or attributes[\"X2\"] == \"Y2\")",
            ["2", "4", "5"].iter().map(|i| i.to_string()).collect(),
        )
        .await;

        run_logs_test(
            export_req.clone(),
            "logs | where not(severity_text == \"WARN\") or attributes[\"X\"] == \"Y2\"",
            ["2", "3", "4", "5"].iter().map(|i| i.to_string()).collect(),
        )
        .await;

        run_logs_test(
            export_req.clone(),
            "logs | where not(severity_text == \"WARN\") and attributes[\"X\"] == \"Y\"",
            ["4"].iter().map(|i| i.to_string()).collect(),
        )
        .await;
    }

    #[tokio::test]
    async fn filter_resource_and_scope_fields() {
        let export_req = ExportLogsServiceRequest {
            resource_logs: vec![
                ResourceLogs {
                    schema_url: "resource_schema1".to_string(),
                    resource: Some(Resource {
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope {
                            name: "scope1".to_string(),
                            ..Default::default()
                        }),
                        log_records: vec![LogRecord {
                            event_name: "1".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                },
                ResourceLogs {
                    schema_url: "resource_schema2".to_string(),
                    resource: Some(Resource {
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope {
                            name: "scope2".to_string(),
                            ..Default::default()
                        }),
                        log_records: vec![LogRecord {
                            event_name: "2".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                },
            ],
        };

        run_logs_test(
            export_req.clone(),
            "logs | where resource.schema_url == \"resource_schema1\"",
            ["1"].iter().map(|i| i.to_string()).collect(),
        )
        .await;

        run_logs_test(
            export_req.clone(),
            "logs | where instrumentation_scope.name == \"scope1\"",
            ["1"].iter().map(|i| i.to_string()).collect(),
        )
        .await;
    }

    #[tokio::test]
    async fn filter_resource_and_scope_attrs() {
        let export_req = ExportLogsServiceRequest {
            resource_logs: vec![
                ResourceLogs {
                    schema_url: "resource_schema1".to_string(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope {
                            name: "scope1".to_string(),
                            attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                            ..Default::default()
                        }),
                        // create some logs with attributes here to ensure the scope & resource
                        // IDs are offset from the log ID. we do this to ensure we join correctly
                        // on resource.id for resource and scope.id for scopes instead of naively
                        // joining on id
                        log_records: vec![
                            LogRecord {
                                event_name: "1".to_string(),
                                attributes: vec![KeyValue::new("A", AnyValue::new_string("B"))],
                                ..Default::default()
                            },
                            LogRecord {
                                event_name: "2".to_string(),
                                attributes: vec![KeyValue::new("A", AnyValue::new_string("B"))],
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }],
                },
                ResourceLogs {
                    schema_url: "resource_schema2".to_string(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![
                        ScopeLogs {
                            scope: Some(InstrumentationScope {
                                name: "scope2".to_string(),
                                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                                ..Default::default()
                            }),
                            log_records: vec![LogRecord {
                                event_name: "3".to_string(),
                                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                        ScopeLogs {
                            scope: Some(InstrumentationScope {
                                name: "scope2".to_string(),
                                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y3"))],
                                ..Default::default()
                            }),
                            log_records: vec![LogRecord {
                                event_name: "4".to_string(),
                                attributes: vec![KeyValue::new("X", AnyValue::new_string("Y2"))],
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                    ],
                },
            ],
        };
        run_logs_test(
            export_req.clone(),
            "logs | where resource.attributes[\"X\"] == \"Y2\"",
            ["3", "4"].iter().map(|i| i.to_string()).collect(),
        )
        .await;

        run_logs_test(
            export_req.clone(),
            "logs | where instrumentation_scope.attributes[\"X\"] == \"Y2\"",
            ["3"].iter().map(|i| i.to_string()).collect(),
        )
        .await;
    }
}
