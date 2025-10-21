// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::compute::concat_batches;
use arrow::ipc::RecordBatch;
use data_engine_expressions::{
    BooleanValue, DataExpression, DoubleValue, IntegerValue, LogicalExpression, PipelineExpression,
    ScalarExpression, SourceScalarExpression, StaticScalarExpression, StringValue, ValueAccessor,
};
use datafusion::catalog::MemTable;
use datafusion::common::{Column, JoinType};
use datafusion::datasource::provider_as_source;
use datafusion::functions::unicode::right;
use datafusion::functions_window::expr_fn::row_number;
use datafusion::logical_expr::expr::{WindowFunction, WindowFunctionParams};
use datafusion::logical_expr::{self, LogicalPlan, LogicalPlanBuilder, Operator};
use datafusion::logical_expr::{Expr, logical_plan};

use datafusion::prelude::{SessionContext, col};

use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::consts;

use crate::error::{Error, Result};
use crate::out_port::{OutPort, OutPortProvider};

const ROW_NUMBER_COL: &str = "_row_number";

struct ExecutionContext {
    curr_batch: OtapArrowRecords,
    session_ctx: SessionContext,
}

pub struct OtapBatchEngine {}

impl OtapBatchEngine {
    pub async fn process(
        &mut self,
        pipeline: &PipelineExpression,
        otap_batch: &OtapArrowRecords,
    ) -> Result<OtapArrowRecords> {
        let mut exec_ctx = ExecutionContext {
            curr_batch: otap_batch.clone(),
            session_ctx: SessionContext::new(),
        };

        for data_expr in pipeline.get_expressions() {
            self.process_data_expr(data_expr, &mut exec_ctx).await?;
        }

        Ok(exec_ctx.curr_batch)
    }

    async fn process_data_expr(
        &mut self,
        data_expr: &DataExpression,
        exec_ctx: &mut ExecutionContext,
    ) -> Result<()> {
        match data_expr {
            DataExpression::Discard(discard) => {
                if let Some(predicate) = discard.get_predicate() {
                    match predicate {
                        // we do opposite of the discard predicate. e.g. keep what would be discarded
                        LogicalExpression::Not(not_expr) => {
                            filter_batch(exec_ctx, not_expr.get_inner_expression()).await?;
                        }
                        _ => todo!("handle invalid discard predciate"),
                    }
                }
            }
            _ => {
                todo!()
            }
        }
        Ok(())
    }
}

fn scan_root_batch(exec_ctx: &ExecutionContext) -> Result<LogicalPlanBuilder> {
    let plan = match exec_ctx.curr_batch {
        OtapArrowRecords::Logs(_) => scan_batch(exec_ctx, ArrowPayloadType::Logs),
        _ => {
            todo!("handle other root batches");
        }
    }?;

    // add a row number column
    let plan = plan
        .window(vec![row_number().alias(ROW_NUMBER_COL)])
        .unwrap();

    Ok(plan)
}

fn scan_batch(
    exec_ctx: &ExecutionContext,
    payload_type: ArrowPayloadType,
) -> Result<LogicalPlanBuilder> {
    if let Some(rb) = exec_ctx.curr_batch.get(payload_type) {
        let table_provider = MemTable::try_new(rb.schema(), vec![vec![rb.clone()]]).unwrap();
        let table_source = provider_as_source(Arc::new(table_provider));
        let logical_plan = LogicalPlanBuilder::scan(
            // TODO could avoid allocation here?
            format!("{:?}", payload_type).to_ascii_lowercase(),
            table_source,
            None,
        )
        .unwrap();

        Ok(logical_plan)
    } else {
        todo!("handle payload type missing");
    }
}

enum ColumnAccessor {
    ColumnName(String),
    Attributes(String),
}

impl TryFrom<&ValueAccessor> for ColumnAccessor {
    type Error = Error;

    fn try_from(accessor: &ValueAccessor) -> Result<Self> {
        let selectors = accessor.get_selectors();

        // TODO the parsing here is kind of goofy
        match &selectors[0] {
            ScalarExpression::Static(StaticScalarExpression::String(column)) => {
                let column_name = column.get_value();
                match column_name {
                    // TODO parsing here is kind of goofy
                    "attributes" => match &selectors[1] {
                        ScalarExpression::Static(StaticScalarExpression::String(attr_key)) => {
                            Ok(Self::Attributes(attr_key.get_value().to_string()))
                        }
                        _ => {
                            todo!("handle invalid attribute key")
                        }
                    },
                    "resource" => {
                        todo!("handle resource access");
                    }
                    "instrumentation_scope" => {
                        todo!("handle instrumentation scope");
                    }
                    value => Ok(Self::ColumnName(value.to_string())),
                }
            }
            _ => {
                todo!("handle invalid attr expression")
            }
        }
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
            _ => {
                todo!("handle invalid scalar expr");
            }
        };

        Ok(binary_arg)
    }
}

// TODO delete this
fn handle_binary_arg(scalar_expr: &ScalarExpression) -> Result<BinaryArg> {
    let binary_arg = match scalar_expr {
        ScalarExpression::Source(source) => {
            BinaryArg::Column(ColumnAccessor::try_from(source.get_value_accessor())?)
        }
        ScalarExpression::Static(static_expr) => BinaryArg::Literal(static_expr.clone()),
        _ => {
            todo!("handle invalid scalar expr");
        }
    };

    Ok(binary_arg)
}

fn try_static_scalar_to_literal(static_scalar: &StaticScalarExpression) -> Result<Expr> {
    let lit_expr = match static_scalar {
        StaticScalarExpression::String(str_val) => logical_expr::lit(str_val.get_value()),
        StaticScalarExpression::Boolean(bool_val) => logical_expr::lit(bool_val.get_value()),
        StaticScalarExpression::Integer(int_val) => logical_expr::lit(int_val.get_value()),
        StaticScalarExpression::Double(float_val) => logical_expr::lit(float_val.get_value()),
        _ => {
            todo!("handle other value types")
        }
    };

    Ok(lit_expr)
}

fn try_static_scalar_to_any_val_column(
    static_scalar: &StaticScalarExpression,
) -> Result<&'static str> {
    let col_name = match static_scalar {
        StaticScalarExpression::Boolean(_) => consts::ATTRIBUTE_BOOL,
        StaticScalarExpression::Double(_) => consts::ATTRIBUTE_DOUBLE,
        StaticScalarExpression::Integer(_) => consts::ATTRIBUTE_INT,
        StaticScalarExpression::String(_) => consts::ATTRIBUTE_STR,
        _ => {
            todo!("handle other attribute columns for binary expr")
        }
    };

    Ok(col_name)
}

#[derive(Clone, Debug)]
enum FilterSubQuery {
    // TODO dumb that these are vec, should be left/right tulues
    LeftSemiJoin(Vec<FilterBuilder>),
    UnionDistinct(Vec<FilterBuilder>),
}

#[derive(Clone, Debug, Default)]
struct FilterBuilder {
    root_batch_exprs: Vec<Expr>,
    attr_batch_exprs: Vec<Expr>,
    sub_query: Option<FilterSubQuery>,
}

fn handle_binary_filter_expr(
    filter_builder: &mut FilterBuilder,
    operator: Operator,
    left: &ScalarExpression,
    right: &ScalarExpression,
) -> Result<()> {
    let left_arg = handle_binary_arg(left)?;
    let right_arg = handle_binary_arg(right)?;

    // TODO support yoda people who put the column name on the right
    // - "WARN" == "severity_text"
    // TODO support people who wanna match columns together (e.g. neither left or right are literals)
    match left_arg {
        BinaryArg::Column(left_col) => match left_col {
            ColumnAccessor::ColumnName(col_name) => match right_arg {
                BinaryArg::Column(_) => {
                    todo!("handle binary right column")
                }
                BinaryArg::Literal(static_scalar_expr) => {
                    let col = logical_expr::col(col_name);
                    let filter_expr = logical_expr::binary_expr(
                        col,
                        operator,
                        try_static_scalar_to_literal(&static_scalar_expr)?,
                    );
                    filter_builder.root_batch_exprs.push(filter_expr);
                }
            },
            ColumnAccessor::Attributes(attr_key) => match right_arg {
                BinaryArg::Column(_) => {
                    todo!("handle column on right");
                }
                BinaryArg::Literal(static_scalar_expr) => {
                    let attr_val_column_name = match static_scalar_expr {
                        StaticScalarExpression::Boolean(_) => consts::ATTRIBUTE_BOOL,
                        StaticScalarExpression::Double(_) => consts::ATTRIBUTE_DOUBLE,
                        StaticScalarExpression::Integer(_) => consts::ATTRIBUTE_INT,
                        StaticScalarExpression::String(_) => consts::ATTRIBUTE_STR,
                        _ => {
                            todo!("handle other attribute columns for binary expr")
                        }
                    };

                    let filter_expr = logical_expr::and(
                        logical_expr::binary_expr(
                            logical_expr::col(consts::ATTRIBUTE_KEY),
                            operator,
                            logical_expr::lit(attr_key),
                        ),
                        logical_expr::binary_expr(
                            logical_expr::col(attr_val_column_name),
                            operator,
                            try_static_scalar_to_literal(&static_scalar_expr)?,
                        ),
                    );
                    filter_builder.attr_batch_exprs.push(filter_expr);
                }
            },
        },
        _ => {
            todo!("handle non column left");
        }
    };

    Ok(())
}

fn handle_filter_predicate(
    filter_builder: &mut FilterBuilder,
    predicate: &LogicalExpression,
) -> Result<()> {
    match predicate {
        LogicalExpression::And(and_expr) => {
            let mut left_builder = FilterBuilder::default();
            handle_filter_predicate(&mut left_builder, and_expr.get_left())?;

            let mut right_builder = FilterBuilder::default();
            handle_filter_predicate(&mut right_builder, and_expr.get_right())?;

            match (&left_builder.sub_query, &right_builder.sub_query) {
                // (x1 or x2) and (y1 or y2)
                //
                // (select x1 union distinct x2) Join::LeftSemi (y1 union distinct y2)
                (
                    Some(FilterSubQuery::UnionDistinct(_)),
                    Some(FilterSubQuery::UnionDistinct(_)),
                ) => {
                    filter_builder.sub_query = Some(FilterSubQuery::LeftSemiJoin(vec![
                        left_builder,
                        right_builder,
                    ]));
                }

                // x1 and (y1 or y1)
                //
                // select x1 and (y1 union distinct y2)
                (_, Some(FilterSubQuery::UnionDistinct(union_sub_queries))) => {
                    filter_builder
                        .root_batch_exprs
                        .append(&mut left_builder.root_batch_exprs);
                    filter_builder
                        .attr_batch_exprs
                        .append(&mut left_builder.attr_batch_exprs);

                    // TODO once again it'd be nice to avoid the clone
                    filter_builder.sub_query =
                        Some(FilterSubQuery::UnionDistinct(union_sub_queries.to_vec()));
                }

                // (x1 or x2) and y1
                //
                // select (x1 union distinct x2) and y1
                (Some(FilterSubQuery::UnionDistinct(union_sub_queries)), _) => {
                    filter_builder
                        .root_batch_exprs
                        .append(&mut right_builder.root_batch_exprs);
                    filter_builder
                        .attr_batch_exprs
                        .append(&mut right_builder.attr_batch_exprs);

                    // TODO once again it'd be nice to avoid the clone
                    filter_builder.sub_query =
                        Some(FilterSubQuery::UnionDistinct(union_sub_queries.to_vec()));
                }

                // select x1 and y1
                (None, None) => {
                    // all the fields can just be and-ed together

                    // TODO just make this an append method ....
                    filter_builder
                        .root_batch_exprs
                        .append(&mut left_builder.root_batch_exprs);
                    filter_builder
                        .attr_batch_exprs
                        .append(&mut left_builder.attr_batch_exprs);

                    filter_builder
                        .root_batch_exprs
                        .append(&mut right_builder.root_batch_exprs);
                    filter_builder
                        .attr_batch_exprs
                        .append(&mut right_builder.attr_batch_exprs);
                }
                _ => {
                    todo!("handle other join clauses")
                }
            }
        }
        LogicalExpression::Or(or_expr) => {
            // TODO it'd be nice to avoid the allocations here ...
            let mut left_builder = FilterBuilder::default();
            let left = or_expr.get_left();
            handle_filter_predicate(&mut left_builder, left)?;

            let mut right_builder = FilterBuilder::default();
            let right = or_expr.get_right();
            handle_filter_predicate(&mut right_builder, right)?;

            // TODO need to check we're not also joining resource_attrs, scope_attrs
            let no_joins = right_builder.attr_batch_exprs.is_empty()
                && left_builder.attr_batch_exprs.is_empty();

            // TODO there's an optimization that can be made here for filters like
            // attributes["X"] == "Y" or attributes["X"] == "Z"
            // rather than doing a subquery, we can do a single join against the
            // root table while filtering attrs table for either of these attributes

            if no_joins {
                let mut lefts = left_builder.root_batch_exprs[0].clone();
                for next_left in left_builder.root_batch_exprs.iter().skip(1) {
                    lefts = logical_expr::and(lefts, next_left.clone());
                }

                let mut rights = right_builder.root_batch_exprs[0].clone();
                for next_right in right_builder.root_batch_exprs.iter().skip(1) {
                    rights = logical_expr::and(rights, next_right.clone());
                }
                let filter_expr = logical_expr::or(lefts, rights);
                filter_builder.root_batch_exprs.push(filter_expr);
            } else {
                filter_builder.sub_query = Some(FilterSubQuery::UnionDistinct(vec![
                    left_builder,
                    right_builder,
                ]))
            }
        }
        LogicalExpression::EqualTo(eq_expr) => {
            // println!("handling EQ expr {:#?}", eq_expr);
            let left = eq_expr.get_left();
            let right = eq_expr.get_right();
            handle_binary_filter_expr(filter_builder, Operator::Eq, left, right)?;
        }
        _ => {
            todo!("handle unsupported predicate")
        }
    };

    Ok(())
}

struct FilteringJoin {
    logical_plan: LogicalPlan,
    join_type: JoinType,
    left_col: &'static str,
    right_col: &'static str,
}

impl FilteringJoin {
    fn join_to_plan(self, plan_builder: LogicalPlanBuilder) -> LogicalPlanBuilder {
        plan_builder
            .join(
                self.logical_plan,
                self.join_type,
                (vec![self.left_col], vec![self.right_col]),
                None,
            )
            .unwrap()
    }
}

struct Filter {
    filter_expr: Option<Expr>,
    join: Option<FilteringJoin>,
}

impl Filter {
    fn try_from_predicate(
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
                        let join_both = scan_root_batch(exec_ctx)?
                            .join(
                                left_join.logical_plan,
                                JoinType::LeftSemi,
                                (vec![left_join.left_col], vec![left_join.right_col]),
                                None,
                            )
                            .unwrap()
                            .join(
                                right_join.logical_plan,
                                JoinType::LeftSemi,
                                (vec![right_join.left_col], vec![right_join.right_col]),
                                None,
                            )
                            .unwrap();

                        Some(FilteringJoin {
                            logical_plan: join_both.build().unwrap(),
                            join_type: JoinType::LeftSemi,
                            left_col: ROW_NUMBER_COL,
                            right_col: ROW_NUMBER_COL,
                        })
                    }
                    (None, Some(join)) | (Some(join), None) => Some(join),
                    _ => None,
                };

                Ok(Self { filter_expr, join })
            }

            LogicalExpression::Or(or_expr) => {
                let left_filter = Self::try_from_predicate(exec_ctx, or_expr.get_left())?;
                let right_filter = Self::try_from_predicate(exec_ctx, or_expr.get_right())?;

                match (left_filter.join, right_filter.join) {
                    (Some(left_join), Some(right_join)) => {
                        let mut left_plan = scan_root_batch(exec_ctx)?;
                        if let Some(filter_expr) = left_filter.filter_expr {
                            left_plan = left_plan.filter(filter_expr).unwrap();
                        }
                        left_plan = left_join.join_to_plan(left_plan);

                        let mut right_plan = scan_root_batch(exec_ctx)?;
                        if let Some(filter_expr) = right_filter.filter_expr {
                            right_plan = right_plan.filter(filter_expr).unwrap();
                        }
                        right_plan = right_join.join_to_plan(right_plan);

                        Ok(Self {
                            filter_expr: None,
                            join: Some(FilteringJoin {
                                logical_plan: left_plan
                                    .union_distinct(right_plan.build().unwrap())
                                    .unwrap()
                                    .build()
                                    .unwrap(),
                                join_type: JoinType::LeftSemi,
                                left_col: ROW_NUMBER_COL,
                                right_col: ROW_NUMBER_COL,
                            }),
                        })
                    }

                    (Some(left_join), None) => {
                        let mut left_plan = scan_root_batch(exec_ctx)?;
                        if let Some(filter_expr) = left_filter.filter_expr {
                            left_plan = left_plan.filter(filter_expr).unwrap();
                        }
                        left_plan = left_join.join_to_plan(left_plan);

                        let mut right_plan = scan_root_batch(exec_ctx)?;
                        if let Some(filter_expr) = right_filter.filter_expr {
                            right_plan = right_plan.filter(filter_expr).unwrap();
                        } else {
                            todo!("would this be invalid?")
                        }

                        Ok(Self {
                            filter_expr: None,
                            join: Some(FilteringJoin {
                                logical_plan: left_plan
                                    .union_distinct(right_plan.build().unwrap())
                                    .unwrap()
                                    .build()
                                    .unwrap(),
                                join_type: JoinType::LeftSemi,
                                left_col: ROW_NUMBER_COL,
                                right_col: ROW_NUMBER_COL,
                            }),
                        })
                    }

                    (None, Some(right_join)) => {
                        let mut left_plan = scan_root_batch(exec_ctx)?;
                        if let Some(filter_expr) = left_filter.filter_expr {
                            left_plan = left_plan.filter(filter_expr).unwrap();
                        } else {
                            todo!("would this be invalid?")
                        }

                        let mut right_plan = scan_root_batch(exec_ctx)?;
                        if let Some(filter_expr) = right_filter.filter_expr {
                            right_plan = right_plan.filter(filter_expr).unwrap();
                        }
                        right_plan = right_join.join_to_plan(right_plan);

                        Ok(Self {
                            filter_expr: None,
                            join: Some(FilteringJoin {
                                logical_plan: left_plan
                                    .union_distinct(right_plan.build().unwrap())
                                    .unwrap()
                                    .build()
                                    .unwrap(),
                                join_type: JoinType::LeftSemi,
                                left_col: ROW_NUMBER_COL,
                                right_col: ROW_NUMBER_COL,
                            }),
                        })
                    }

                    (None, None) => match (left_filter.filter_expr, right_filter.filter_expr) {
                        (Some(left_filter), Some(right_filter)) => Ok(Self {
                            filter_expr: Some(logical_expr::or(left_filter, right_filter)),
                            join: None,
                        }),
                        _ => {
                            todo!("How does this happen?")
                        }
                    },
                    _ => {
                        todo!()
                    }
                }
            }

            LogicalExpression::Not(not_expr) => {
                let not_filter =
                    Self::try_from_predicate(exec_ctx, not_expr.get_inner_expression())?;
                if let Some(not_join) = not_filter.join {
                    let mut plan = scan_root_batch(exec_ctx)?;
                    if let Some(filter_expr) = not_filter.filter_expr {
                        plan = plan.filter(filter_expr).unwrap();
                    }

                    plan = not_join.join_to_plan(plan);
                    Ok(Self {
                        filter_expr: None,
                        join: Some(FilteringJoin {
                            logical_plan: plan.build().unwrap(),
                            join_type: JoinType::LeftAnti,
                            left_col: ROW_NUMBER_COL,
                            right_col: ROW_NUMBER_COL,
                        }),
                    })
                } else {
                    if let Some(expr) = not_filter.filter_expr {
                        Ok(Self {
                            filter_expr: Some(logical_expr::not(expr)),
                            join: None,
                        })
                    } else {
                        todo!("invalid?")
                    }
                }
            }

            LogicalExpression::EqualTo(eq_expr) => Self::try_from_binary_expr(
                exec_ctx,
                Operator::Eq,
                eq_expr.get_left(),
                eq_expr.get_right(),
            ),
            _ => {
                todo!("return error")
            }
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
            BinaryArg::Column(left_col) => match left_col {
                ColumnAccessor::ColumnName(col_name) => match right_arg {
                    BinaryArg::Column(_) => {
                        todo!("handle column right arg in binary expr")
                    }
                    BinaryArg::Literal(static_scalar) => {
                        let left = logical_expr::col(col_name);
                        let right = try_static_scalar_to_literal(&static_scalar)?;
                        Ok(Self {
                            filter_expr: Some(logical_expr::binary_expr(left, operator, right)),
                            join: None,
                        })
                    }
                },
                ColumnAccessor::Attributes(attr_key) => match right_arg {
                    BinaryArg::Column(_) => {
                        todo!("handle column right arg in binary expr")
                    }
                    BinaryArg::Literal(static_scalar) => {
                        let attr_val_col_name =
                            try_static_scalar_to_any_val_column(&static_scalar)?;

                        // TODO -- not have payload type hard-coded
                        let attrs_filter = scan_batch(exec_ctx, ArrowPayloadType::LogAttrs)?
                            .filter(logical_expr::and(
                                logical_expr::binary_expr(
                                    logical_expr::col(consts::ATTRIBUTE_KEY),
                                    operator,
                                    logical_expr::lit(attr_key),
                                ),
                                logical_expr::binary_expr(
                                    logical_expr::col(attr_val_col_name),
                                    operator,
                                    try_static_scalar_to_literal(&static_scalar)?,
                                ),
                            ))
                            .unwrap();

                        Ok(Self {
                            filter_expr: None,
                            join: Some(FilteringJoin {
                                logical_plan: attrs_filter.build().unwrap(),
                                join_type: JoinType::LeftSemi,
                                left_col: consts::ID,
                                right_col: consts::PARENT_ID,
                            }),
                        })
                    }
                },
            },
            _ => {
                todo!("handle non column left arg in binary expr");
            }
        }
    }
}

fn append_filter_steps(
    exec_ctx: &mut ExecutionContext,
    mut root_logical_plan: LogicalPlanBuilder,
    filter_builder: &FilterBuilder,
) -> Result<LogicalPlanBuilder> {
    for filter_expr in &filter_builder.root_batch_exprs {
        root_logical_plan = root_logical_plan.filter(filter_expr.clone()).unwrap();
    }

    // TODO check the perf here, but it might be better to join this with itself vs. with the root
    // plan multiple times?
    if !filter_builder.attr_batch_exprs.is_empty() {
        for filter_expr in &filter_builder.attr_batch_exprs {
            root_logical_plan = root_logical_plan
                .join(
                    scan_batch(&exec_ctx, ArrowPayloadType::LogAttrs)?
                        .filter(filter_expr.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                    JoinType::LeftSemi,
                    (vec![consts::ID], vec![consts::PARENT_ID]),
                    None,
                )
                .unwrap();
        }
    }

    match &filter_builder.sub_query {
        Some(FilterSubQuery::LeftSemiJoin(sub_queries)) => {
            let left_builder = &sub_queries[0];
            let left = append_filter_steps(exec_ctx, root_logical_plan.clone(), left_builder)?;

            let right_builder = &sub_queries[1];
            let right = append_filter_steps(exec_ctx, root_logical_plan.clone(), right_builder)?;

            root_logical_plan = left
                .join(
                    right.build().unwrap(),
                    JoinType::LeftSemi,
                    (vec![consts::ID], vec![consts::ID]),
                    None,
                )
                .unwrap();
        }

        // TODO -- double check if this is the most efficient way to do this.
        // e.g. there's 2 options here: x1 and (y1 or y2)
        //
        // op 1: select (y1 where x1) union distinct (y2 where x1)
        // op 2: select (y1 union y2) where x1
        //
        Some(FilterSubQuery::UnionDistinct(sub_queries)) => {
            let left_builder = &sub_queries[0];
            let left = append_filter_steps(exec_ctx, root_logical_plan.clone(), left_builder)?;

            let right_builder = &sub_queries[1];
            let right = append_filter_steps(exec_ctx, root_logical_plan.clone(), right_builder)?;

            root_logical_plan = left.union_distinct(right.build().unwrap()).unwrap();
        }
        None => {
            // nothing to do
        }
    }

    return Ok(root_logical_plan);
}

async fn filter_batch(
    exec_ctx: &mut ExecutionContext,
    predicate: &LogicalExpression,
) -> Result<()> {
    // let mut filter_builder = FilterBuilder::default();
    // handle_filter_predicate(&mut filter_builder, predicate)?;
    // let root_logical_plan =
    //     append_filter_steps(exec_ctx, scan_root_batch(&exec_ctx)?, &filter_builder)?;

    let filter = Filter::try_from_predicate(&exec_ctx, predicate)?;
    let mut root_logical_plan = scan_root_batch(&exec_ctx)?;

    if let Some(expr) = filter.filter_expr {
        root_logical_plan = root_logical_plan.filter(expr).unwrap();
    }

    if let Some(join) = filter.join {
        root_logical_plan = join.join_to_plan(root_logical_plan);
    }

    let logical_plan = root_logical_plan.build().unwrap();
    println!("logical plan:\n{}", logical_plan);

    let batches = exec_ctx
        .session_ctx
        .execute_logical_plan(logical_plan)
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    // TODO handle batches empty
    // TODO not sure concat_batches is necessary here as there should only be one batch
    let mut result = concat_batches(batches[0].schema_ref(), &batches).unwrap();

    // remove the ROW_ID col
    if let Ok(col_idx) = result.schema_ref().index_of(ROW_NUMBER_COL) {
        _ = result.remove_column(col_idx);
    }

    // TODO how do we know it's logs here?
    exec_ctx.curr_batch.set(ArrowPayloadType::Logs, result);

    filter_attrs_for_root(exec_ctx, ArrowPayloadType::LogAttrs).await?;
    filter_attrs_for_root(exec_ctx, ArrowPayloadType::ResourceAttrs).await?;
    filter_attrs_for_root(exec_ctx, ArrowPayloadType::ScopeAttrs).await?;

    Ok(())
}

async fn filter_attrs_for_root(
    exec_ctx: &mut ExecutionContext,
    payload_type: ArrowPayloadType,
) -> Result<()> {
    if exec_ctx.curr_batch.get(payload_type).is_some() {
        let attrs_table_scan = scan_batch(exec_ctx, payload_type)?;
        let root_table_scan = scan_root_batch(exec_ctx)?;

        let logical_plan = attrs_table_scan
            .join(
                root_table_scan.build().unwrap(),
                JoinType::LeftSemi,
                (vec![consts::PARENT_ID], vec![consts::ID]),
                None,
            )
            .unwrap()
            .build()
            .unwrap();

        let batches = exec_ctx
            .session_ctx
            .execute_logical_plan(logical_plan)
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();
        // TODO handle batches empty
        let result = concat_batches(batches[0].schema_ref(), &batches).unwrap();

        exec_ctx.curr_batch.set(payload_type, result);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use arrow::array::{DictionaryArray, StringArray, UInt8Array};
    use arrow::datatypes::UInt8Type;
    use data_engine_kql_parser::{
        KqlParser, Parser, ParserMapKeySchema, ParserMapSchema, ParserOptions,
    };
    use datafusion::prelude::lit;
    use datafusion::sql::sqlparser::keywords::ROW;
    use otap_df_otap::encoder::encode_logs_otap_batch;
    use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
    use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use prost::Message;

    use crate::out_port::MapOutPortProvider;

    use super::*;

    async fn run_logs_test(
        record: ExportLogsServiceRequest,
        kql_expr: &str,

        // TODO is this kind of a hokey way to validate?
        expected_event_name: Vec<String>,
    ) {
        println!("\n>>>>\nHADLING QUERY:\n  {}\n>>>>", kql_expr);

        let mut bytes = vec![];
        record.encode(&mut bytes).unwrap();
        let logs_view = RawLogsData::new(&bytes);
        let otap_batch = encode_logs_otap_batch(&logs_view).unwrap();
        let mut engine = OtapBatchEngine {};
        let parser_options = ParserOptions::new();
        let pipeline_expr = KqlParser::parse_with_options(kql_expr, parser_options).unwrap();
        let result = engine.process(&pipeline_expr, &otap_batch).await.unwrap();
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();
        let event_name_col = logs_rb.column_by_name(consts::EVENT_NAME).unwrap();

        let tmp = event_name_col
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        let tmp2 = tmp.downcast_dict::<StringArray>().unwrap();
        let mut event_names = tmp2
            .into_iter()
            .map(|v| v.unwrap().to_string())
            .collect::<Vec<_>>();
        event_names.sort();

        assert_eq!(expected_event_name, event_names)
    }

    fn logs_to_export_req(log_records: Vec<LogRecord>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    #[tokio::test]
    async fn test_filter_simple() {
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
        // (TODO -- make the optimization this comment describes)
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

    #[ignore]
    #[tokio::test]
    async fn smoke_test() {
        let record = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![
                        LogRecord {
                            severity_text: "INFO".to_string(),
                            attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                            event_name: "1".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "WARN".to_string(),
                            attributes: vec![
                                KeyValue::new("X", AnyValue::new_string("Y")),
                                KeyValue::new("error", AnyValue::new_string("error happen")),
                            ],
                            event_name: "2".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "WARN".to_string(),
                            event_name: "3".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "ERROR".to_string(),
                            event_name: "4".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            attributes: vec![
                                KeyValue::new("error", AnyValue::new_string("error happen")),
                                KeyValue::new("attr_int", AnyValue::new_int(1)),
                            ],
                            severity_text: "DEBUG".to_string(),
                            event_name: "5".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let mut bytes = vec![];
        record.encode(&mut bytes).unwrap();
        let logs_view = RawLogsData::new(&bytes);
        let otap_batch = encode_logs_otap_batch(&logs_view).unwrap();

        let mut engine = OtapBatchEngine {};

        let mut parser_options = ParserOptions::new();

        // TODO what is the point of this?
        // let mut logs_schema = ParserMapSchema::new()
        //     .with_key_definition("severity_text", ParserMapKeySchema::Integer)
        //     .with_key_definition("attributes", ParserMapKeySchema::Map(None));
        // parser_options = parser_options.with_source_map_schema(logs_schema);

        // let kql_expr = "logs | where log.severity_text == \"WARN\"";
        // let kql_expr = "logs | where log.attributes[\"X\"] == \"Y\"";
        // let kql_expr = "logs | where log.severity_text == \"INFO\" or log.severity_text == \"ERROR\"";
        // let kql_expr = "logs | where log.severity_text == \"ERROR\" or log.attributes[\"error\"] == \"error happen\"";
        // let kql_expr = "logs | where log.attributes[\"attr_int\"] == 1 and log.attributes[\"error\"] == \"error happen\" and log.severity_text == \"DEBUG\"";
        let kql_expr = "logs | where (log.attributes[\"attr_int\"] == 1 or log.severity_text == \"WARN\") and (log.severity_text == \"DEBUG\" or log.attributes[\"error\"] == \"error happen\")";

        // TODO next:
        // - logs | where logs.attributes["X"] == "Y" and (log.severity_text == \"ERROR\" or log.severity_text == "DEBUG")
        // - logs | where severity_text == "WARN" and (log.severity_text == \"ERROR\" or log.attributes[\"error\"] == \"error happen\"")
        // - logs | where (log.severity_text == ERROR or log.attributes["error"] == "error happen") and (log.severity_text == "INFO" or log.attributes["info"] == true)

        // - filter with an AND expression
        // - filter by resource.name
        // - filter by resource.attributes

        let pipeline_expr = KqlParser::parse_with_options(kql_expr, parser_options).unwrap();

        let result = engine.process(&pipeline_expr, &otap_batch).await.unwrap();
        arrow::util::pretty::print_batches(&[result.get(ArrowPayloadType::Logs).unwrap().clone()])
            .unwrap();
    }

    #[tokio::test]
    async fn test_fussin() {
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

        let mut bytes = vec![];
        export_req.encode(&mut bytes).unwrap();

        let logs_view = RawLogsData::new(&bytes);
        let otap_batch = encode_logs_otap_batch(&logs_view).unwrap();

        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();

        arrow::util::pretty::print_batches(&[logs_rb.clone()]).unwrap();

        let exec_ctx = ExecutionContext {
            curr_batch: otap_batch,
            session_ctx: SessionContext::new(),
        };

        let ctx = SessionContext::new();
        let plan = scan_root_batch(&exec_ctx)
            .unwrap()
            .filter(col("severity_text").eq(lit("WARN")))
            .unwrap();

        let result = ctx
            .execute_logical_plan(plan.clone().build().unwrap())
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();

        println!("result1 :");
        arrow::util::pretty::print_batches(&result).unwrap();

        let plan2 = scan_root_batch(&exec_ctx)
            .unwrap()
            .filter(col("event_name").eq(lit("3")))
            .unwrap();

        let result = ctx
            .execute_logical_plan(plan2.clone().build().unwrap())
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();

        println!("result2 :");
        arrow::util::pretty::print_batches(&result).unwrap();

        let jp = plan
            .clone()
            .join(
                plan2.clone().build().unwrap(),
                JoinType::LeftSemi,
                (vec!["id"], vec!["id"]),
                None,
            )
            .unwrap();

        let result = ctx
            .execute_logical_plan(jp.clone().build().unwrap())
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();

        println!("result3 :");
        arrow::util::pretty::print_batches(&result).unwrap();

        let jp = plan
            .join(
                plan2.build().unwrap(),
                JoinType::LeftSemi,
                (vec![ROW_NUMBER_COL], vec![ROW_NUMBER_COL]),
                None,
            )
            .unwrap();

        let result = ctx
            .execute_logical_plan(jp.clone().build().unwrap())
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();

        println!("result4 :");
        arrow::util::pretty::print_batches(&result).unwrap();
    }
}
