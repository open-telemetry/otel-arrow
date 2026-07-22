// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Planner that converts AST expressions (`ScalarExpression` and `LogicalExpression`)
//! into `ScopedExpr` execution trees.

use std::borrow::Cow;
use std::sync::Arc;

use arrow::datatypes::DataType;
use data_engine_expressions::{
    BinaryMathematicalScalarExpression, BooleanValue, CaptureTextScalarExpression,
    CoalesceScalarExpression, CollectionScalarExpression, CombineScalarExpression, DateTimeValue,
    DoubleValue, Expression, IntegerValue, InvokeFunctionArgument, InvokeFunctionScalarExpression,
    JoinTextScalarExpression, LogicalExpression, MathScalarExpression, PipelineFunction,
    PipelineFunctionImplementation, ReplaceTextScalarExpression, ScalarExpression,
    StaticScalarExpression, StringScalarExpression, StringValue, TextScalarExpression, ValueType,
};
use datafusion::functions::core::coalesce::CoalesceFunc;
use datafusion::functions::core::expr_ext::FieldAccessor;
use datafusion::functions::crypto::{md5, sha256, sha512};
use datafusion::functions::datetime::to_char;
use datafusion::functions::encoding::encode;
use datafusion::functions::math::log10;
use datafusion::functions::string::{
    concat, concat_ws, ends_with, lower, ltrim, replace, rtrim, starts_with, upper, uuid,
};
use datafusion::logical_expr::ScalarUDFImpl;
use datafusion::logical_expr::expr::ScalarFunction;
use datafusion::logical_expr::simplify::{ExprSimplifyResult, SimplifyContext};
use datafusion::logical_expr::{BinaryExpr, Expr, Operator, ScalarUDF, col, lit, not};
use datafusion::prelude::{binary_expr, lit_timestamp_nano};
use otap_df_config::SignalType;
use otap_df_pdata::schema::consts;

#[cfg(feature = "sha1-hash")]
use crate::consts::SHA1_FUNC_NAME;
use crate::consts::{
    ENCODE_FUNC_NAME, ENDS_WITH_FUNC_NAME, FNV_FUNC_NAME, FORMAT_DATETIME_FUNC_NAME, LOG_FUNC_NAME,
    LOWER_CASE_FUNC_NAME, LTRIM_FUNC_NAME, MD5_FUNC_NAME, MURMUR3_FUNC_NAME,
    REGEXP_SUBSTR_FUNC_NAME, RTRIM_FUNC_NAME, SHA256_FUNC_NAME, SHA512_FUNC_NAME,
    STARTS_WITH_FUNC_NAME, UPPER_CASE_FUNC_NAME, UUID_FUNC_NAME, UUIDV7_FUNC_NAME, XXH3_FUNC_NAME,
    XXH128_FUNC_NAME,
};
use crate::error::{Error, Result};
use crate::pipeline::assign::leaf_requires_dict_downcast;
use crate::pipeline::expr::join::is_one_to_many;
use crate::pipeline::expr::types::{
    ExprLogicalType, coerce_arithmetic, nested_struct_field_type, root_field_type,
};
use crate::pipeline::expr::{DataScope, VALUE_COLUMN_NAME, arg_column_name};
use crate::pipeline::expr::{LeafEval, RootParentStruct, ScopedExpr, SignalTypePredicate};
use crate::pipeline::functions::compare::CompareFunc;
use crate::pipeline::functions::expr_fn::contains;
use crate::pipeline::functions::is_type::IsTypeFunc;
#[cfg(feature = "sha1-hash")]
use crate::pipeline::functions::sha1_hash;
use crate::pipeline::functions::{
    arity_range, fnv_hash, murmur3_hash, regexp_substr, substring, uuidv7, xxh3_hash, xxh128_hash,
};
use crate::pipeline::planner::{AttributesIdentifier, ColumnAccessor};
use crate::pipeline::project::{Projection, ProjectionOptions};

/// Planner that converts AST expressions into `ScopedExpr` execution trees.
pub(crate) struct ExprPlanner {
    /// Whether attribute key matching should be case-sensitive. Defaults to `true`.
    /// When `false`, attribute key filtering uses case-insensitive comparison.
    attr_key_case_sensitive: bool,

    /// When `true`, the planner is producing `ScopedExpr` trees for evaluation on an attributes
    /// `RecordBatch` (i.e., inside an `apply attributes { ... }` pipeline).
    plan_for_attributes: bool,
}

/// Intermediate planning result that carries type information alongside the `ScopedExpr`
pub(crate) struct PlannedOp {
    pub expr: ScopedExpr,
    pub expr_type: ExprLogicalType,
    pub requires_dict_downcast: bool,
}

impl ExprPlanner {
    /// Creates a new `ExprPlanner` with case-sensitive attribute key matching
    pub fn new() -> Self {
        Self {
            attr_key_case_sensitive: true,
            plan_for_attributes: false,
        }
    }

    /// Creates a new `ExprPlanner` with the specified attribute key case sensitivity
    pub fn with_attr_key_case_sensitive(attr_key_case_sensitive: bool) -> Self {
        Self {
            attr_key_case_sensitive,
            plan_for_attributes: false,
        }
    }

    /// Creates a new `ExprPlanner` configured for attribute record batch evaluation
    pub fn for_attributes(attr_key_case_sensitive: bool) -> Self {
        Self {
            attr_key_case_sensitive,
            plan_for_attributes: true,
        }
    }

    /// Plan a `ScalarExpression` into a `ScopedExpr` that produces a value
    pub fn plan_scalar(
        &self,
        expr: &ScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<PlannedOp> {
        match expr {
            ScalarExpression::Source(source_scalar_expr) => {
                let value_accessor = source_scalar_expr.get_value_accessor();
                let column_accessor = ColumnAccessor::try_from(value_accessor)?;

                match column_accessor {
                    ColumnAccessor::ColumnName(column_name) => {
                        let field_type = root_field_type(&column_name).ok_or_else(|| {
                            Error::InvalidPipelineError {
                                cause: format!("unknown field {column_name} on record batch"),
                                query_location: Some(
                                    source_scalar_expr.get_query_location().clone(),
                                ),
                            }
                        })?;
                        Ok(PlannedOp {
                            expr: ScopedExpr::Eval {
                                scope: DataScope::Root,
                                eval: LeafEval::new_df_expr(col(column_name), false)?,
                            },
                            expr_type: field_type,
                            requires_dict_downcast: false,
                        })
                    }
                    ColumnAccessor::StructCol(column_name, struct_field_name) => {
                        let field_type = nested_struct_field_type(&struct_field_name)
                            .ok_or_else(|| Error::InvalidPipelineError {
                                cause: format!(
                                    "unknown field {struct_field_name} on {column_name} struct column"
                                ),
                                query_location: Some(
                                    source_scalar_expr.get_query_location().clone(),
                                ),
                            })?;
                        let data_scope = match column_name {
                            consts::RESOURCE => DataScope::RootParent(RootParentStruct::Resource),
                            consts::SCOPE => DataScope::RootParent(RootParentStruct::Scope),
                            _ => DataScope::Root,
                        };
                        Ok(PlannedOp {
                            expr: ScopedExpr::Eval {
                                scope: data_scope,
                                eval: LeafEval::new_df_expr(
                                    col(column_name).field(struct_field_name),
                                    false,
                                )?,
                            },
                            expr_type: field_type,
                            requires_dict_downcast: false,
                        })
                    }
                    ColumnAccessor::Attributes(attrs_id, key) => Ok(PlannedOp {
                        expr: ScopedExpr::Eval {
                            scope: DataScope::Attribute(attrs_id, key),
                            eval: LeafEval::new_df_expr_with_key_case(
                                col(VALUE_COLUMN_NAME),
                                false,
                                self.attr_key_case_sensitive,
                            )?,
                        },
                        expr_type: ExprLogicalType::AnyValue,
                        requires_dict_downcast: false,
                    }),
                    ColumnAccessor::NestedAttribute(_, _, _) => Err(Error::NotYetSupportedError {
                        message: "reading nested serialized attribute paths is not yet supported"
                            .into(),
                    }),
                }
            }

            ScalarExpression::Static(static_scalar_expr) => {
                let (logical_expr, expr_type) = match static_scalar_expr {
                    StaticScalarExpression::Integer(int_expr) => {
                        (lit(int_expr.get_value()), ExprLogicalType::AnyInt)
                    }
                    StaticScalarExpression::Double(double_expr) => {
                        (lit(double_expr.get_value()), ExprLogicalType::Float64)
                    }
                    StaticScalarExpression::Boolean(bool_expr) => {
                        (lit(bool_expr.get_value()), ExprLogicalType::Boolean)
                    }
                    StaticScalarExpression::String(string_expr) => {
                        (lit(string_expr.get_value()), ExprLogicalType::String)
                    }
                    StaticScalarExpression::Null(_) => (Expr::default(), ExprLogicalType::AnyValue),
                    StaticScalarExpression::DateTime(dt_expr) => {
                        let val = dt_expr.get_value().timestamp_nanos_opt().ok_or_else(|| {
                            Error::ExecutionError {
                                cause: format!(
                                    "failed to convert {:?} to nanosecond timestamp",
                                    dt_expr
                                ),
                            }
                        })?;
                        (
                            lit_timestamp_nano(val),
                            ExprLogicalType::TimestampNanosecond,
                        )
                    }
                    _ => {
                        return Err(Error::NotYetSupportedError {
                            message: format!(
                                "static scalar expression type not yet supported: {static_scalar_expr:?}"
                            ),
                        });
                    }
                };

                Ok(PlannedOp {
                    expr: ScopedExpr::Eval {
                        scope: DataScope::StaticScalar,
                        eval: LeafEval::new_df_expr(logical_expr, false)?,
                    },
                    expr_type,
                    requires_dict_downcast: false,
                })
            }

            ScalarExpression::Math(math_scalar_expr) => match math_scalar_expr {
                MathScalarExpression::Add(e) => self.plan_binary_math(e, Operator::Plus, functions),
                MathScalarExpression::Subtract(e) => {
                    self.plan_binary_math(e, Operator::Minus, functions)
                }
                MathScalarExpression::Multiply(e) => {
                    self.plan_binary_math(e, Operator::Multiply, functions)
                }
                MathScalarExpression::Divide(e) => {
                    self.plan_binary_math(e, Operator::Divide, functions)
                }
                MathScalarExpression::Modulus(e) => {
                    self.plan_binary_math(e, Operator::Modulo, functions)
                }
                other => Err(Error::NotYetSupportedError {
                    message: format!("math expression not yet supported {other:?}"),
                }),
            },

            ScalarExpression::Coalesce(coalesce_expr) => {
                self.plan_coalesce_expr(coalesce_expr, functions)
            }

            ScalarExpression::InvokeFunction(invoke_expr) => {
                self.plan_function_invocation(invoke_expr, functions)
            }

            ScalarExpression::Slice(slice_expr) => {
                let start =
                    slice_expr
                        .get_range_start()
                        .ok_or_else(|| Error::InvalidPipelineError {
                            cause: "start index is required for substring".into(),
                            query_location: Some(slice_expr.get_query_location().clone()),
                        })?;

                let mut arg_exprs: Vec<&ScalarExpression> = vec![slice_expr.get_source(), start];
                if let Some(end) = slice_expr.get_range_length() {
                    arg_exprs.push(end);
                }

                let (df_args, scope, eval_scope, dict_downcast) =
                    self.plan_function_args(arg_exprs.into_iter(), functions)?;

                Ok(PlannedOp {
                    expr: self.build_eval_or_join(
                        Expr::ScalarFunction(ScalarFunction::new_udf(substring(), df_args)),
                        scope,
                        eval_scope,
                        dict_downcast,
                    )?,
                    expr_type: ExprLogicalType::String,
                    requires_dict_downcast: dict_downcast,
                })
            }

            ScalarExpression::Text(text_expr) => self.plan_text_expr(text_expr, functions),

            ScalarExpression::Logical(logical_expr) => {
                let expr = self.plan_logical(logical_expr, functions)?;
                Ok(PlannedOp {
                    expr,
                    expr_type: ExprLogicalType::Boolean,
                    requires_dict_downcast: false,
                })
            }

            other_expr => Err(Error::NotYetSupportedError {
                message: format!("expression not yet supported {other_expr:?}"),
            }),
        }
    }

    /// Plan a `LogicalExpression` into a `ScopedExpr` that produces a boolean.
    pub fn plan_logical(
        &self,
        expr: &LogicalExpression,
        functions: &[PipelineFunction],
    ) -> Result<ScopedExpr> {
        match expr {
            LogicalExpression::EqualTo(eq_expr) => {
                // check for signal type check pattern first
                if let Some(type_check_expr) = self.try_plan_as_type_check(
                    eq_expr.get_left(),
                    Operator::Eq,
                    eq_expr.get_right(),
                    functions,
                )? {
                    return Ok(type_check_expr);
                }

                let case_sensitive = !eq_expr.get_case_insensitive();
                self.plan_comparison(
                    eq_expr.get_left(),
                    Operator::Eq,
                    eq_expr.get_right(),
                    case_sensitive,
                    functions,
                )
            }

            LogicalExpression::GreaterThan(gt_expr) => self.plan_comparison(
                gt_expr.get_left(),
                Operator::Gt,
                gt_expr.get_right(),
                true,
                functions,
            ),

            LogicalExpression::GreaterThanOrEqualTo(gte_expr) => self.plan_comparison(
                gte_expr.get_left(),
                Operator::GtEq,
                gte_expr.get_right(),
                true,
                functions,
            ),

            LogicalExpression::And(and_expr) => {
                let left = self.plan_logical(and_expr.get_left(), functions)?;
                let right = self.plan_logical(and_expr.get_right(), functions)?;
                let left_planned = PlannedOp {
                    expr: left,
                    expr_type: ExprLogicalType::Boolean,
                    requires_dict_downcast: false,
                };
                let right_planned = PlannedOp {
                    expr: right,
                    expr_type: ExprLogicalType::Boolean,
                    requires_dict_downcast: false,
                };
                let combined_scope = try_combine_scopes(&left_planned, &right_planned);
                let left = left_planned.expr;
                let right = right_planned.expr;
                if let Some(combined_scope) = combined_scope
                    && !matches!(combined_scope, DataScope::AttributesAll(_))
                {
                    let downcast_dicts =
                        leaf_requires_dict_downcast(&left) || leaf_requires_dict_downcast(&right);
                    let left_expr =
                        left.into_df_eval_expr()
                            .ok_or_else(|| Error::InvalidPipelineError {
                                cause: "invalid input to and".into(),
                                query_location: None,
                            })?;
                    let right_expr =
                        right
                            .into_df_eval_expr()
                            .ok_or_else(|| Error::InvalidPipelineError {
                                cause: "invalid input to and".into(),
                                query_location: None,
                            })?;
                    Ok(ScopedExpr::Eval {
                        scope: combined_scope,
                        eval: LeafEval::new_df_expr(left_expr.and(right_expr), downcast_dicts)?,
                    })
                } else {
                    let mut align_children_to_root = false;
                    if let (
                        DataScope::AttributesAll(left_attrs_id),
                        DataScope::AttributesAll(right_attrs_id),
                    ) = (
                        left.effective_value_scope()?.as_ref(),
                        right.effective_value_scope()?.as_ref(),
                    ) {
                        if left_attrs_id == right_attrs_id {
                            // most performant way to "and" the results of the children exprs
                            // is to create a bitmap of the parent_ids passing each side then
                            // combine the bitmaps
                            return Ok(ScopedExpr::BitmapAnd(Box::new(left), Box::new(right)));
                        } else {
                            // here we're "and"ing the results of filters on attributes, but the
                            // parent_id columns represent different IDs, so we can't "and" the
                            // bitmaps. We set `align_children_to_root` because it's just the most
                            // performant way to line up the results of the filters on each side in
                            // join eval
                            align_children_to_root = true;
                        }
                    }

                    Ok(ScopedExpr::JoinAndEval {
                        children: vec![left, right],
                        eval: LeafEval::new_df_expr(
                            col(arg_column_name(0)).and(col(arg_column_name(1))),
                            false,
                        )?,
                        align_children_to_root,
                        default_null_children: false,
                    })
                }
            }

            LogicalExpression::Or(or_expr) => {
                let left = self.plan_logical(or_expr.get_left(), functions)?;
                let right = self.plan_logical(or_expr.get_right(), functions)?;
                let left_planned = PlannedOp {
                    expr: left,
                    expr_type: ExprLogicalType::Boolean,
                    requires_dict_downcast: false,
                };
                let right_planned = PlannedOp {
                    expr: right,
                    expr_type: ExprLogicalType::Boolean,
                    requires_dict_downcast: false,
                };
                let combined_scope = try_combine_scopes(&left_planned, &right_planned);
                let left = left_planned.expr;
                let right = right_planned.expr;

                if let Some(combined_scope) = combined_scope
                    && !matches!(combined_scope, DataScope::AttributesAll(_))
                {
                    let downcast_dicts =
                        leaf_requires_dict_downcast(&left) || leaf_requires_dict_downcast(&right);
                    let left_expr =
                        left.into_df_eval_expr()
                            .ok_or_else(|| Error::InvalidPipelineError {
                                cause: "invalid input to or".into(),
                                query_location: None,
                            })?;
                    let right_expr =
                        right
                            .into_df_eval_expr()
                            .ok_or_else(|| Error::InvalidPipelineError {
                                cause: "invalid input to or".into(),
                                query_location: None,
                            })?;
                    Ok(ScopedExpr::Eval {
                        scope: combined_scope,
                        eval: LeafEval::new_df_expr(left_expr.or(right_expr), downcast_dicts)?,
                    })
                } else {
                    let mut align_children_to_root = false;
                    if let (
                        DataScope::AttributesAll(left_attrs_id),
                        DataScope::AttributesAll(right_attrs_id),
                    ) = (
                        left.effective_value_scope()?.as_ref(),
                        right.effective_value_scope()?.as_ref(),
                    ) {
                        if left_attrs_id == right_attrs_id {
                            // most performant way to "or" the results of the children exprs
                            // is to create a bitmap of the parent_ids passing each side then
                            // combine the bitmaps
                            return Ok(ScopedExpr::BitmapOr(Box::new(left), Box::new(right)));
                        } else {
                            // here we're "or"ing the results of filters on attributes, but the
                            // parent_id columns represent different IDs, so we can't "or" the
                            // bitmaps. We set `align_children_to_root` because it's just the most
                            // performant way to line up the results of the filters on each side in
                            // join eval
                            align_children_to_root = true;
                        }
                    }

                    Ok(ScopedExpr::JoinAndEval {
                        children: vec![left, right],
                        eval: LeafEval::new_df_expr(
                            col(arg_column_name(0)).or(col(arg_column_name(1))),
                            false,
                        )?,
                        align_children_to_root,
                        default_null_children: false,
                    })
                }
            }

            LogicalExpression::Not(not_expr) => {
                let inner_expr = not_expr.get_inner_expression();

                // The parser will plan Lt/LtEq as Not(GtEq)/Not(Gt) - we look for this pattern and
                // simply plan comparison with the correct operator when its encountered
                match inner_expr {
                    LogicalExpression::GreaterThan(gt_expr) => {
                        return self.plan_comparison(
                            gt_expr.get_left(),
                            Operator::LtEq,
                            gt_expr.get_right(),
                            false,
                            functions,
                        );
                    }
                    LogicalExpression::GreaterThanOrEqualTo(gte_expr) => {
                        return self.plan_comparison(
                            gte_expr.get_left(),
                            Operator::Lt,
                            gte_expr.get_right(),
                            false,
                            functions,
                        );
                    }
                    _ => {}
                }

                let inner = self.plan_logical(inner_expr, functions)?;
                let is_attrs_all_scope = matches!(
                    inner.effective_value_scope()?.as_ref(),
                    DataScope::AttributesAll(_)
                );
                Ok(match inner {
                    ScopedExpr::Eval { scope, eval } => match eval {
                        LeafEval::DatafusionExpr {
                            logical_expr,
                            physical_expr: _,
                            projection,
                            projection_opts,
                            eval_anyval_as_struct,
                            attr_key_case_sensitive,
                            missing_data_passes,
                        } if !is_attrs_all_scope => ScopedExpr::Eval {
                            scope,
                            eval: LeafEval::DatafusionExpr {
                                logical_expr: not(logical_expr),
                                physical_expr: None,
                                projection,
                                projection_opts,
                                eval_anyval_as_struct,
                                attr_key_case_sensitive,
                                missing_data_passes: !missing_data_passes,
                            },
                        },
                        _ => ScopedExpr::BitmapNot(Box::new(ScopedExpr::Eval { scope, eval })),
                    },
                    ScopedExpr::JoinAndEval {
                        children,
                        eval,
                        default_null_children,
                        align_children_to_root,
                    } if !is_attrs_all_scope => match eval {
                        LeafEval::DatafusionExpr {
                            logical_expr,
                            physical_expr: _,
                            projection,
                            projection_opts,
                            eval_anyval_as_struct,
                            attr_key_case_sensitive,
                            missing_data_passes,
                        } => ScopedExpr::JoinAndEval {
                            children,
                            default_null_children,
                            align_children_to_root,
                            eval: LeafEval::DatafusionExpr {
                                logical_expr: not(logical_expr),
                                physical_expr: None,
                                projection,
                                projection_opts,
                                eval_anyval_as_struct,
                                attr_key_case_sensitive,
                                missing_data_passes: !missing_data_passes,
                            },
                        },
                        _ => ScopedExpr::BitmapNot(Box::new(ScopedExpr::JoinAndEval {
                            children,
                            eval,
                            default_null_children,
                            align_children_to_root,
                        })),
                    },
                    _ => ScopedExpr::BitmapNot(Box::new(inner)),
                })
            }

            LogicalExpression::Contains(contains_expr) => {
                self.plan_contains(contains_expr, functions)
            }

            LogicalExpression::Matches(matches_expr) => self.plan_matches(matches_expr, functions),

            LogicalExpression::Scalar(scalar_expr) => match scalar_expr {
                ScalarExpression::Static(StaticScalarExpression::Boolean(bool_val)) => {
                    Ok(ScopedExpr::Eval {
                        scope: DataScope::StaticScalar,
                        eval: LeafEval::new_df_expr(lit(bool_val.get_value()), false)?,
                    })
                }
                other => {
                    let planned = self.plan_scalar(other, functions)?;
                    Ok(planned.expr)
                }
            },
        }
    }

    fn plan_binary_math(
        &self,
        binary_expr: &BinaryMathematicalScalarExpression,
        operator: Operator,
        functions: &[PipelineFunction],
    ) -> Result<PlannedOp> {
        let mut left = self.plan_scalar(binary_expr.get_left_expression(), functions)?;
        let mut right = self.plan_scalar(binary_expr.get_right_expression(), functions)?;

        // extract mutable parts for coercion
        let (left_df_expr, right_df_expr, left_type, right_type) =
            extract_coercion_parts(&mut left, &mut right);

        let expr_type = match (left_df_expr, right_df_expr, left_type, right_type) {
            (Some(le), Some(re), Some(lt), Some(rt)) => coerce_arithmetic(le, lt, re, rt)
                .ok_or_else(|| Error::InvalidPipelineError {
                    cause: format!(
                        "could not coerce types for arithmetic: left type {:?}, right type {:?}",
                        left.expr_type, right.expr_type
                    ),
                    query_location: Some(binary_expr.get_query_location().clone()),
                })?,
            _ => {
                // one side is not an Eval(DatafusionExpr) node (e.g., a JoinAndEval or Bitmap node).
                // cannot coerce in this case, but the types might already be compatible.
                left.expr_type.clone()
            }
        };

        let expr = self.build_binary_expr(left, operator, right, true)?;

        Ok(PlannedOp {
            expr,
            expr_type,
            requires_dict_downcast: true,
        })
    }

    fn plan_coalesce_expr(
        &self,
        coalesce_expr: &CoalesceScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<PlannedOp> {
        let (df_args, args_scope, data_scope, _) =
            self.plan_function_args(coalesce_expr.get_expressions().iter(), functions)?;

        // DataFusion's `coalesce` UDF does not support direct physical evaluation; the optimizer
        // rewrites it via `CoalesceFunc::simplify`. Reuse that implementation here.
        let coalesce_func = CoalesceFunc::new();
        let simplify_result = coalesce_func
            .simplify(df_args, &SimplifyContext::default())
            .map_err(Error::from)?;

        let case_expr = match simplify_result {
            ExprSimplifyResult::Simplified(expr) => expr,
            ExprSimplifyResult::Original(_) => {
                return Err(Error::InvalidPipelineError {
                    cause: "expected coalesce simplify to produce a single expression".into(),
                    query_location: None,
                });
            }
        };

        let mut expr = self.build_eval_or_join(case_expr, args_scope, data_scope, true)?;
        if let ScopedExpr::JoinAndEval {
            ref mut align_children_to_root,
            ..
        } = expr
        {
            *align_children_to_root = true;
        }

        Ok(PlannedOp {
            expr,
            // Like `concat`, mixed attribute columns (often dictionary-encoded) and literals need
            // dictionary downcasting before CASE can build a single array.
            expr_type: ExprLogicalType::AnyValue,
            requires_dict_downcast: true,
        })
    }

    fn plan_function_invocation(
        &self,
        invoke_expr: &InvokeFunctionScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<PlannedOp> {
        let function_id = invoke_expr.get_function_id();
        let function = functions
            .get(function_id)
            .ok_or_else(|| Error::InvalidPipelineError {
                cause: format!("function id {function_id} not found"),
                query_location: Some(invoke_expr.get_query_location().clone()),
            })?;

        let PipelineFunctionImplementation::External(func_name) = function.get_implementation()
        else {
            return Err(Error::NotYetSupportedError {
                message: "Only external functions currently supported in expression".into(),
            });
        };

        let df_udf = DataFusionFunctionDef::from_func_name(func_name).ok_or_else(|| {
            Error::InvalidPipelineError {
                cause: format!("Unknown function '{func_name}"),
                query_location: Some(invoke_expr.get_query_location().clone()),
            }
        })?;

        let invoke_arg_exprs = invoke_expr.get_arguments();
        let num_args = invoke_arg_exprs.len();

        if let Some(arity_range) = arity_range(&df_udf.scalar_udf.signature().type_signature) {
            if !arity_range.contains(&num_args) {
                return Err(Error::InvalidPipelineError {
                    cause: format!(
                        "function '{func_name}' expects {} arguments. Received {num_args}",
                        if arity_range.len() > 1 {
                            format!("{}-{}", arity_range.start, arity_range.end - 1)
                        } else {
                            format!("{}", arity_range.start)
                        }
                    ),
                    query_location: Some(invoke_expr.get_query_location().clone()),
                });
            }
        }

        let (arg_exprs, args_scope, data_scope, source_dict_downcast) = if invoke_arg_exprs
            .is_empty()
        {
            // For zero-arg functions (e.g. `uuid_v4()`, `uuid_v7()`), evaluate against the
            // root batch so that volatile UDFs produce one value per row rather than a single
            // scalar that gets broadcast across rows.
            (
                Vec::new(),
                FunctionArgScope::Combined(DataScope::Root),
                Some(DataScope::Root),
                false,
            )
        } else {
            let scalar_arg_exprs = invoke_arg_exprs
                .iter()
                .map(|arg| match arg {
                    InvokeFunctionArgument::Scalar(scalar_expr) => Ok(scalar_expr),
                    InvokeFunctionArgument::MutableValue(_) => Err(Error::NotYetSupportedError {
                        message:
                            "Mutable value as function argument not yet supported in expression"
                                .into(),
                    }),
                })
                .collect::<Result<Vec<_>>>()?;

            self.plan_function_args(scalar_arg_exprs.into_iter(), functions)?
        };

        let mut logical_expr =
            Expr::ScalarFunction(ScalarFunction::new_udf(df_udf.scalar_udf, arg_exprs));

        if let Some(data_type) = df_udf.cast_result_to {
            logical_expr = datafusion::logical_expr::cast(logical_expr, data_type);
        }

        let dict_downcast = source_dict_downcast || df_udf.requires_dict_downcast;

        Ok(PlannedOp {
            expr: self.build_eval_or_join(logical_expr, args_scope, data_scope, dict_downcast)?,
            expr_type: df_udf.return_type,
            requires_dict_downcast: dict_downcast,
        })
    }

    /// Plan a list of scalar expression arguments and determine scope combinability.
    ///
    /// Returns:
    /// - `Vec<Expr>`: the DataFusion expressions for each argument (either raw or
    ///   col("arg_N") references for multi-join)
    /// - `FunctionArgScope`: whether all args share a scope or require multi-join
    /// - `bool`: dict_downcast
    fn plan_function_args<'a>(
        &self,
        arg_exprs: impl Iterator<Item = &'a ScalarExpression>,
        functions: &[PipelineFunction],
    ) -> Result<(Vec<Expr>, FunctionArgScope, Option<DataScope>, bool)> {
        let planned_args: Vec<PlannedOp> = arg_exprs
            .map(|arg| self.plan_scalar(arg, functions))
            .collect::<Result<Vec<_>>>()?;

        if planned_args.is_empty() {
            return Ok((
                Vec::new(),
                FunctionArgScope::Combined(DataScope::StaticScalar),
                None,
                false,
            ));
        }

        // check if all arguments can be combined into a single scope
        let mut combined_scope: Option<DataScope> = None;
        let mut all_combinable = true;
        let mut dict_downcast = false;

        for arg in &planned_args {
            dict_downcast |= arg.requires_dict_downcast;

            let arg_scope = match arg.expr.eval_scope() {
                Some(scope) => scope,
                None => {
                    all_combinable = false;
                    break;
                }
            };

            combined_scope = match combined_scope.take() {
                None => Some(arg_scope.clone()),
                Some(existing) => {
                    if existing.can_combine(arg_scope) {
                        Some(if !existing.is_scalar() {
                            existing
                        } else {
                            arg_scope.clone()
                        })
                    } else {
                        all_combinable = false;
                        break;
                    }
                }
            };
        }

        if all_combinable {
            let scope = combined_scope.unwrap_or(DataScope::StaticScalar);
            let df_exprs = planned_args
                .into_iter()
                .filter_map(|a| a.expr.into_df_eval_expr())
                .collect();

            Ok((
                df_exprs,
                FunctionArgScope::Combined(scope.clone()),
                Some(scope),
                dict_downcast,
            ))
        } else {
            let arg_col_exprs: Vec<Expr> = (0..planned_args.len())
                .map(|i| col(arg_column_name(i)))
                .collect();
            let children = planned_args.into_iter().map(|a| a.expr).collect();

            Ok((
                arg_col_exprs,
                FunctionArgScope::Join(children),
                None,
                dict_downcast,
            ))
        }
    }

    fn plan_text_expr(
        &self,
        text_expr: &TextScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<PlannedOp> {
        match text_expr {
            TextScalarExpression::Concat(combine_expr) => {
                self.plan_concat_expr(combine_expr, functions)
            }
            TextScalarExpression::Join(join_expr) => self.plan_join_text_expr(join_expr, functions),
            TextScalarExpression::Replace(replace_expr) => {
                self.plan_replace_text_expr(replace_expr, functions)
            }
            TextScalarExpression::Capture(capture_expr) => {
                self.plan_capture_text_expr(capture_expr, functions)
            }
        }
    }

    fn plan_concat_expr(
        &self,
        combine_expr: &CombineScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<PlannedOp> {
        match combine_expr.get_values_expression() {
            ScalarExpression::Collection(CollectionScalarExpression::List(list_expr)) => {
                let (df_args, args_scope, data_scope, _) =
                    self.plan_function_args(list_expr.get_value_expressions().iter(), functions)?;

                Ok(PlannedOp {
                    expr: self.build_eval_or_join(
                        Expr::ScalarFunction(ScalarFunction::new_udf(concat(), df_args)),
                        args_scope,
                        data_scope,
                        true,
                    )?,
                    expr_type: ExprLogicalType::String,
                    requires_dict_downcast: true,
                })
            }
            other => Err(Error::InvalidPipelineError {
                cause: format!(
                    "Unexpected scalar expression for CombineScalarExpression values {other:?}"
                ),
                query_location: Some(combine_expr.get_query_location().clone()),
            }),
        }
    }

    fn plan_join_text_expr(
        &self,
        join_expr: &JoinTextScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<PlannedOp> {
        match join_expr.get_values_expression() {
            ScalarExpression::Collection(CollectionScalarExpression::List(list_expr)) => {
                let (df_args, args_scope, data_scope, _) = self.plan_function_args(
                    [join_expr.get_separator_expression()]
                        .into_iter()
                        .chain(list_expr.get_value_expressions().iter()),
                    functions,
                )?;

                Ok(PlannedOp {
                    expr: self.build_eval_or_join(
                        Expr::ScalarFunction(ScalarFunction::new_udf(concat_ws(), df_args)),
                        args_scope,
                        data_scope,
                        true,
                    )?,
                    expr_type: ExprLogicalType::String,
                    requires_dict_downcast: true,
                })
            }
            other => Err(Error::InvalidPipelineError {
                cause: format!(
                    "Unexpected scalar expression for JoinTextScalarExpression values {other:?}"
                ),
                query_location: Some(join_expr.get_query_location().clone()),
            }),
        }
    }

    fn plan_replace_text_expr(
        &self,
        replace_expr: &ReplaceTextScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<PlannedOp> {
        let (df_args, args_scope, data_scope, _) = self.plan_function_args(
            [
                replace_expr.get_haystack_expression(),
                replace_expr.get_needle_expression(),
                replace_expr.get_replacement_expression(),
            ]
            .into_iter(),
            functions,
        )?;

        Ok(PlannedOp {
            expr: self.build_eval_or_join(
                Expr::ScalarFunction(ScalarFunction::new_udf(replace(), df_args)),
                args_scope,
                data_scope,
                true,
            )?,
            expr_type: ExprLogicalType::String,
            requires_dict_downcast: true,
        })
    }

    fn plan_capture_text_expr(
        &self,
        capture_expr: &CaptureTextScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<PlannedOp> {
        let capture_scalar_expr = match capture_expr.get_pattern() {
            ScalarExpression::Static(StaticScalarExpression::Regex(regexp_expr)) => {
                Cow::Owned(ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(
                        regexp_expr.get_query_location().clone(),
                        regexp_expr.get_value().as_str(),
                    ),
                )))
            }
            other => Cow::Borrowed(other),
        };

        let (mut df_args, args_scope, data_scope, dict_downcast) = self.plan_function_args(
            [
                capture_expr.get_haystack(),
                &capture_scalar_expr,
                capture_expr.get_capture_group(),
            ]
            .into_iter(),
            functions,
        )?;

        Ok(PlannedOp {
            expr: self.build_eval_or_join(
                Expr::ScalarFunction(ScalarFunction::new_udf(
                    regexp_substr(),
                    vec![
                        df_args.remove(0), // source
                        df_args.remove(0), // pattern
                        lit(1),            // start
                        lit(1),            // occurrence
                        Expr::default(),   // flags = literal Null
                        df_args.remove(0), // group
                    ],
                )),
                args_scope,
                data_scope,
                dict_downcast,
            )?,
            expr_type: ExprLogicalType::String,
            requires_dict_downcast: dict_downcast,
        })
    }

    fn plan_comparison(
        &self,
        left_expr: &ScalarExpression,
        mut operator: Operator,
        right_expr: &ScalarExpression,
        case_sensitive: bool,
        functions: &[PipelineFunction],
    ) -> Result<ScopedExpr> {
        let left_is_null = is_null_literal(left_expr);
        let right_is_null = is_null_literal(right_expr);

        // handle null comparisons specially
        if left_is_null || right_is_null {
            if operator != Operator::Eq {
                // TODO in the future we may want to relax this and return static false here
                return Err(Error::InvalidPipelineError {
                    cause: format!(
                        "cannot compare null using operator {operator}. only == is allowed"
                    ),
                    query_location: None,
                });
            }

            if left_is_null && right_is_null {
                // null == null -> always true
                return Ok(ScopedExpr::Eval {
                    scope: DataScope::StaticScalar,
                    eval: LeafEval::new_df_expr(lit(true), false)?,
                });
            }

            // one side is null, the other is a value expression
            let value_expr = if left_is_null { right_expr } else { left_expr };
            if let Some(null_compare) = self.plan_null_comparison(value_expr)? {
                return Ok(null_compare);
            }
        }

        let mut left = self.plan_scalar(left_expr, functions)?;
        let mut right = self.plan_scalar(right_expr, functions)?;
        let either_side_literal = is_literal_eval(&left) || is_literal_eval(&right);

        // Try fused attribute comparison optimization: when one side is an attribute
        // access and the other is a typed literal, skip the expensive key-filter +
        // value-projection materialization step.
        if !self.plan_for_attributes {
            if let Some(fused) = self.try_plan_fused_attr_comparison(
                &mut left,
                operator,
                &mut right,
                case_sensitive,
            )? {
                return Ok(fused);
            }
        }

        // handle body field comparisons — body is an AnyValue struct, so we need to
        // resolve the sub-field based on the other side's type
        resolve_body_field_in_planned_ops(&mut left, &mut right);

        // handle case-insensitive equality
        // TODO once the changes from https://github.com/apache/arrow-rs/pull/9871 are released
        // we can use the `eq_ascii_ignore_case` kernel here instead of having to escape like this
        if !case_sensitive && operator == Operator::Eq {
            operator = Operator::ILikeMatch;
            escape_like_literals(&mut left);
            escape_like_literals(&mut right);
        }

        // apply type coercion (ignore result type -- comparisons always produce bool)
        if let (Some(le), Some(re), Some(lt), Some(rt)) =
            extract_coercion_parts(&mut left, &mut right)
        {
            // TODO using coerce_arithmetic here is a trick to get numeric types to be cast to
            // compatible types for comparison. Eventually we should refactor the signature of
            // the type coercion helpers to make the intention more clear.
            let _ = coerce_arithmetic(le, lt, re, rt);
        }

        let requires_dict_downcast = left.requires_dict_downcast || right.requires_dict_downcast;

        let mut expr = self.build_binary_expr(left, operator, right, requires_dict_downcast)?;
        if !either_side_literal {
            // if we're here, it means both sides of the comparison are not literals. For
            // these cases, we want to use a special comparison evaluation which handles:
            // - when each side resolves to a different type (the normal DF binary expr evaluation
            //   produces an error for this case)
            // - when both sides are null we treat want to treat this as equal whereas datafusion
            //   will produce a `null` which may get coerced into false by ScopedExpr.
            expr = self.compare_using_udf(operator, expr);
        }

        Ok(expr)
    }

    /// Switches from using datafusion's BinaryExpr for comparison to a custom scalar UDF wrapping
    /// [`compare`](crate::pipeline::filter::compare::compare)
    fn compare_using_udf(&self, op: Operator, expr: ScopedExpr) -> ScopedExpr {
        fn transform_leaf(eval: LeafEval) -> LeafEval {
            match eval {
                LeafEval::DatafusionExpr {
                    logical_expr: Expr::BinaryExpr(binary_expr),
                    physical_expr,
                    projection,
                    projection_opts,
                    eval_anyval_as_struct,
                    attr_key_case_sensitive,
                    missing_data_passes,
                } => LeafEval::DatafusionExpr {
                    logical_expr: CompareFunc::new_expr(
                        *binary_expr.left,
                        binary_expr.op,
                        *binary_expr.right,
                    ),
                    physical_expr,
                    projection,
                    projection_opts: ProjectionOptions {
                        // the comparison function here handles all null columns
                        default_null_columns: true,
                        ..projection_opts
                    },
                    eval_anyval_as_struct,
                    attr_key_case_sensitive,
                    missing_data_passes,
                },
                other => other,
            }
        }

        match expr {
            ScopedExpr::Eval { scope, eval } => ScopedExpr::Eval {
                scope,
                eval: transform_leaf(eval),
            },
            ScopedExpr::JoinAndEval { children, eval, .. } => ScopedExpr::JoinAndEval {
                children,
                default_null_children: true,
                align_children_to_root: op == Operator::Eq,
                eval: transform_leaf(eval),
            },
            other => other,
        }
    }

    /// Plan a null comparison: `<value_expr> == null`.
    ///
    /// For column references: produces `col.is_null()`
    /// For struct columns: produces `col(struct).field(name).is_null()`
    /// For attributes: produces `BitmapNot` of a simple key-existence check e.g. if the
    /// attribute key doesn't exist, it "is null"
    fn plan_null_comparison(&self, value_expr: &ScalarExpression) -> Result<Option<ScopedExpr>> {
        // try to resolve the expression as a column accessor
        if let ScalarExpression::Source(source_expr) = value_expr {
            let value_accessor = source_expr.get_value_accessor();
            if let Ok(column_accessor) = ColumnAccessor::try_from(value_accessor) {
                return match column_accessor {
                    ColumnAccessor::ColumnName(col_name) => {
                        let is_null_expr = if col_name == crate::consts::BODY_FIELD_NAME {
                            // body null check — check all sub-fields
                            col(col_name).is_null()
                        } else {
                            col(col_name).is_null()
                        };
                        Ok(Some(ScopedExpr::Eval {
                            scope: DataScope::Root,
                            eval: LeafEval::new_df_expr(is_null_expr, false)?,
                        }))
                    }
                    ColumnAccessor::StructCol(struct_name, field_name) => {
                        Ok(Some(ScopedExpr::Eval {
                            scope: DataScope::Root,
                            eval: LeafEval::new_df_expr(
                                col(struct_name).field(field_name).is_null(),
                                false,
                            )?,
                        }))
                    }
                    ColumnAccessor::Attributes(attrs_id, key) => {
                        // attribute == null means the attribute key does not exist.
                        // This is expressed as NOT(key_exists), where key_exists is
                        // an Eval in the attribute scope that simply checks for row
                        // existence (any matching key row -> exists).
                        //
                        // In the ScopedExpr model: if the key doesn't match any rows,
                        // execute_as_id_mask returns IdMask::None. So:
                        //   key_exists -> IdMask::Some(matching_parent_ids)
                        //   NOT(key_exists) -> IdMask::NotSome(matching_parent_ids)
                        //
                        // We use a trivial "true" predicate in the attribute scope to
                        // check for key existence (key filtering is done at the scope level).
                        Ok(Some(ScopedExpr::BitmapNot(Box::new(ScopedExpr::Eval {
                            scope: DataScope::Attribute(attrs_id, key),
                            eval: LeafEval::new_df_expr_with_key_case(
                                lit(true),
                                false,
                                self.attr_key_case_sensitive,
                            )?,
                        }))))
                    }
                    ColumnAccessor::NestedAttribute(_, _, _) => Ok(None),
                };
            }
        }

        Ok(None)
    }

    /// Try to produce a fused attribute comparison expression that avoids the  key-filter +
    /// value-projection materialization step.
    ///
    /// When one side of a comparison is a **simple** attribute access (`attributes["key"]`)
    /// and the other is a typed literal (string, int, double, bool), we can evaluate the
    /// comparison directly on the raw attributes RecordBatch with a single expression:
    /// `col("key").eq(lit("key")).and(col("<typed_col>").<op>(lit(<value>)))`. This avoids
    /// materializing an intermediate record batch containing only rows with a key match.
    ///
    /// Returns `None` when the pattern doesn't match (falls back to the normal path).
    fn try_plan_fused_attr_comparison(
        &self,
        left: &mut PlannedOp,
        mut operator: Operator,
        right: &mut PlannedOp,
        case_sensitive: bool,
    ) -> Result<Option<ScopedExpr>> {
        // Identify which side is the attribute access and which is the literal.
        let (attrs_op, literal_op, attrs_on_left) =
            match (left.expr.eval_scope(), right.expr.eval_scope()) {
                (Some(DataScope::Attribute(_, _)), Some(DataScope::StaticScalar)) => {
                    (left, right, true)
                }
                (Some(DataScope::StaticScalar), Some(DataScope::Attribute(_, _))) => {
                    (right, left, false)
                }
                _ => return Ok(None),
            };

        // Only apply when the attribute side is a simple `col("value")` reference —
        // not when the attribute value is used in arithmetic, function calls, etc.
        if !is_simple_attr_value_column(attrs_op) {
            return Ok(None);
        }

        if operator == Operator::Eq && !case_sensitive {
            escape_like_literals(literal_op);
            operator = Operator::ILikeMatch;
        }

        // Extract the attribute key and attrs_id
        let (attrs_id, key) = match attrs_op.expr.eval_scope() {
            Some(DataScope::Attribute(id, key)) => (*id, key.clone()),
            _ => return Ok(None),
        };

        // Determine the typed attribute column for the literal's type
        let typed_col = match attr_value_column_for_expr_type(&literal_op.expr_type) {
            Some(col_name) => col_name,
            None => return Ok(None), // type not optimizable, fall back
        };

        // Extract the literal expression
        let literal_expr = match literal_op.expr.as_df_eval_expr_ref() {
            Some(expr) => expr.clone(),
            None => return Ok(None),
        };

        // Build the key filter expression.
        // For case-insensitive key matching, use ilike with escaped LIKE special chars.
        let key_filter = if self.attr_key_case_sensitive {
            col(consts::ATTRIBUTE_KEY).eq(lit(&key))
        } else {
            let escaped_key = if contains_like_pattern(&key) {
                escape_like_pattern(&key)
            } else {
                key.clone()
            };
            col(consts::ATTRIBUTE_KEY).ilike(lit(escaped_key))
        };

        // Build the value comparison expression.
        // If the attribute is on the right (literal on left), flip the operator.
        let effective_operator = if attrs_on_left {
            operator
        } else {
            flip_comparison_operator(operator)
        };

        let value_cmp = Expr::BinaryExpr(BinaryExpr::new(
            Box::new(col(typed_col)),
            effective_operator,
            Box::new(literal_expr),
        ));

        // Fuse: key_filter AND value_comparison
        let fused_expr = key_filter.and(value_cmp);

        Ok(Some(ScopedExpr::Eval {
            scope: DataScope::AttributesAll(attrs_id),
            eval: LeafEval::new_df_expr(fused_expr, false)?,
        }))
    }

    fn plan_contains(
        &self,
        contains_expr: &data_engine_expressions::ContainsLogicalExpression,
        functions: &[PipelineFunction],
    ) -> Result<ScopedExpr> {
        let mut haystack = self.plan_scalar(contains_expr.get_haystack(), functions)?;
        let mut needle = self.plan_scalar(contains_expr.get_needle(), functions)?;

        // Try fused attribute contains optimization: when haystack is attributes["key"]
        // and needle is a string literal.
        if !self.plan_for_attributes {
            if let Some(fused) = self.try_plan_fused_attr_contains(&haystack, &needle)? {
                return Ok(fused);
            }
        }

        // for body column, resolve to body.str for text contains
        let mut has_body_field = is_body_planned_op(&haystack);
        if has_body_field {
            rewrite_body_expr(&mut haystack, consts::ATTRIBUTE_STR);
        }
        if is_body_planned_op(&needle) {
            rewrite_body_expr(&mut needle, consts::ATTRIBUTE_STR);
            has_body_field = true;
        }

        // for attribute-level mode, resolve col("value") to col("str") since
        // contains always operates on string columns
        if self.plan_for_attributes {
            if is_attr_value_column(&haystack) {
                rewrite_attr_value_column(&mut haystack, consts::ATTRIBUTE_STR);
            }
            if is_attr_value_column(&needle) {
                rewrite_attr_value_column(&mut needle, consts::ATTRIBUTE_STR);
            }
        }

        let possible_scope = try_combine_scopes(&haystack, &needle);
        if let Some(scope) = possible_scope {
            // note: currently the InvalidPipelineError shouldn't happen, as the check to combine
            // scopes checks if the expr is a DF Eval, which is the same condition that determines
            // whether into_df_eval_expr will return Some. The error here is just being cautious
            let haystack_expr =
                haystack
                    .expr
                    .into_df_eval_expr()
                    .ok_or_else(|| Error::InvalidPipelineError {
                        cause: "invalid input to match".into(),
                        query_location: None,
                    })?;
            let needle_expr =
                needle
                    .expr
                    .into_df_eval_expr()
                    .ok_or_else(|| Error::InvalidPipelineError {
                        cause: "invalid input to match".into(),
                        query_location: None,
                    })?;

            let dict_downcast = haystack.requires_dict_downcast || needle.requires_dict_downcast;
            let eval = if has_body_field {
                LeafEval::new_df_expr_anyval_as_struct(
                    contains(haystack_expr, needle_expr),
                    dict_downcast,
                )?
            } else {
                LeafEval::new_df_expr_with_key_case(
                    contains(haystack_expr, needle_expr),
                    dict_downcast,
                    self.attr_key_case_sensitive,
                )?
            };
            Ok(ScopedExpr::Eval { scope, eval })
        } else {
            Ok(ScopedExpr::JoinAndEval {
                children: vec![haystack.expr, needle.expr],
                default_null_children: false,
                align_children_to_root: false,
                eval: LeafEval::new_df_expr(
                    contains(col(arg_column_name(0)), col(arg_column_name(1))),
                    true,
                )?,
            })
        }
    }

    /// Try to produce a fused attribute `contains` expression.
    ///
    /// When the haystack is `attributes["key"]` and the needle is a string literal,
    /// produces: `col("key").eq(lit("key")).and(contains(col("str"), lit("needle")))`
    fn try_plan_fused_attr_contains(
        &self,
        haystack: &PlannedOp,
        needle: &PlannedOp,
    ) -> Result<Option<ScopedExpr>> {
        // Haystack must be an attribute access, needle must be a static string literal
        let (attrs_id, key) = match haystack.expr.eval_scope() {
            Some(DataScope::Attribute(id, key)) => (*id, key.clone()),
            _ => return Ok(None),
        };

        if !is_simple_attr_value_column(haystack) {
            return Ok(None);
        }

        if !matches!(needle.expr.eval_scope(), Some(DataScope::StaticScalar)) {
            return Ok(None);
        }

        if !matches!(needle.expr_type, ExprLogicalType::String) {
            return Ok(None);
        }

        let needle_expr = match needle.expr.as_df_eval_expr_ref() {
            Some(expr) => expr.clone(),
            None => return Ok(None),
        };

        let key_filter = if self.attr_key_case_sensitive {
            col(consts::ATTRIBUTE_KEY).eq(lit(&key))
        } else {
            col(consts::ATTRIBUTE_KEY).ilike(lit(escape_like_pattern(&key)))
        };

        let value_contains = contains(col(consts::ATTRIBUTE_STR), needle_expr);

        let fused_expr = key_filter.and(value_contains);

        Ok(Some(ScopedExpr::Eval {
            scope: DataScope::AttributesAll(attrs_id),
            eval: LeafEval::new_df_expr(fused_expr, false)?,
        }))
    }

    fn plan_matches(
        &self,
        matches_expr: &data_engine_expressions::MatchesLogicalExpression,
        functions: &[PipelineFunction],
    ) -> Result<ScopedExpr> {
        let pattern = match matches_expr.get_pattern() {
            ScalarExpression::Static(StaticScalarExpression::Regex(regex)) => {
                lit(regex.get_value().as_str().to_string())
            }
            _ => {
                return Err(Error::InvalidPipelineError {
                    cause: "expected pattern to be a static regex".into(),
                    query_location: Some(matches_expr.get_query_location().clone()),
                });
            }
        };

        let mut haystack = self.plan_scalar(matches_expr.get_haystack(), functions)?;

        // Try fused attribute matches optimization: when haystack is attributes["key"]
        // and pattern is a static regex.
        if !self.plan_for_attributes {
            if let Some(fused) = self.try_plan_fused_attr_matches(&haystack, &pattern)? {
                return Ok(fused);
            }
        }

        // for body column, resolve to body.str for regex matching
        let has_body_field = is_body_planned_op(&haystack);
        if has_body_field {
            rewrite_body_expr(&mut haystack, consts::ATTRIBUTE_STR);
        }

        // for attribute-level mode, resolve col("value") to col("str") since regex
        // matching always operates on string columns
        if self.plan_for_attributes && is_attr_value_column(&haystack) {
            rewrite_attr_value_column(&mut haystack, consts::ATTRIBUTE_STR);
        }
        let scope = haystack
            .expr
            .eval_scope()
            .cloned()
            .unwrap_or(DataScope::Root);
        let haystack_expr =
            haystack
                .expr
                .into_df_eval_expr()
                .ok_or_else(|| Error::InvalidPipelineError {
                    cause: "invalid input to contains".into(),
                    query_location: None,
                })?;

        let eval = if has_body_field {
            LeafEval::new_df_expr_anyval_as_struct(
                binary_expr(haystack_expr, Operator::RegexMatch, pattern),
                haystack.requires_dict_downcast,
            )?
        } else {
            LeafEval::new_df_expr_with_key_case(
                binary_expr(haystack_expr, Operator::RegexMatch, pattern),
                haystack.requires_dict_downcast,
                self.attr_key_case_sensitive,
            )?
        };

        Ok(ScopedExpr::Eval { scope, eval })
    }

    /// Try to produce a fused attribute `matches` (regex) expression.
    ///
    /// When the haystack is `attributes["key"]` and the pattern is a static regex,
    /// produces: `col("key").eq(lit("key")).and(col("str") ~ lit("pattern"))`
    fn try_plan_fused_attr_matches(
        &self,
        haystack: &PlannedOp,
        pattern: &Expr,
    ) -> Result<Option<ScopedExpr>> {
        let (attrs_id, key) = match haystack.expr.eval_scope() {
            Some(DataScope::Attribute(id, key)) => (*id, key.clone()),
            _ => return Ok(None),
        };

        if !is_simple_attr_value_column(haystack) {
            return Ok(None);
        }

        let key_filter = if self.attr_key_case_sensitive {
            col(consts::ATTRIBUTE_KEY).eq(lit(&key))
        } else {
            col(consts::ATTRIBUTE_KEY).ilike(lit(escape_like_pattern(&key)))
        };

        let value_regex = binary_expr(
            col(consts::ATTRIBUTE_STR),
            Operator::RegexMatch,
            pattern.clone(),
        );

        let fused_expr = key_filter.and(value_regex);

        Ok(Some(ScopedExpr::Eval {
            scope: DataScope::AttributesAll(attrs_id),
            eval: LeafEval::new_df_expr(fused_expr, false)?,
        }))
    }

    /// Try to plan as a batch-level type check (e.g., `is Log`).
    fn try_plan_as_type_check(
        &self,
        left_expr: &ScalarExpression,
        operator: Operator,
        right_expr: &ScalarExpression,
        functions: &[PipelineFunction],
    ) -> Result<Option<ScopedExpr>> {
        match (left_expr, operator, right_expr) {
            (
                ScalarExpression::GetRecordType(_),
                Operator::Eq,
                ScalarExpression::Static(StaticScalarExpression::String(typename_expr)),
            ) => {
                let type_name = typename_expr.get_value();
                let signal_type = match type_name {
                    "Log" => SignalType::Logs,
                    "Metric" => SignalType::Metrics,
                    "Span" => SignalType::Traces,
                    _ => {
                        return Err(Error::InvalidPipelineError {
                            cause: format!("Unknown stream type name {type_name}"),
                            query_location: Some(right_expr.get_query_location().clone()),
                        });
                    }
                };

                Ok(Some(ScopedExpr::Eval {
                    scope: DataScope::Root,
                    eval: LeafEval::BatchPredicate(Box::new(SignalTypePredicate::new(signal_type))),
                }))
            }

            (
                ScalarExpression::GetType(get_type_expr),
                Operator::Eq,
                ScalarExpression::Static(StaticScalarExpression::String(typename_expr)),
            ) => {
                let expr_source = self.plan_scalar(get_type_expr.get_value(), functions)?;

                let type_name = typename_expr.get_value();
                let value_type = ValueType::from_str_opt(type_name).ok_or_else(|| {
                    Error::InvalidPipelineError {
                        cause: format!("Unknown type name {type_name}"),
                        query_location: Some(right_expr.get_query_location().clone()),
                    }
                })?;

                // Array, Map, and Null types have no standalone Arrow representation —
                // they only appear as subtypes inside AnyValue struct columns. Handle
                // them via the IsTypeFunc UDF on the AnyValue discriminator.
                if matches!(
                    value_type,
                    ValueType::Array | ValueType::Map | ValueType::Null
                ) {
                    use crate::pipeline::functions::is_type::IsTypeFunc;
                    use otap_df_pdata::otlp::attributes::AttributeValueType;

                    if !matches!(
                        expr_source.expr_type,
                        ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumeric
                    ) {
                        return Ok(Some(ScopedExpr::Eval {
                            scope: DataScope::StaticScalar,
                            eval: LeafEval::new_df_expr(lit(false), false)?,
                        }));
                    }

                    let subtype = match value_type {
                        ValueType::Array => AttributeValueType::Slice,
                        ValueType::Map => AttributeValueType::Map,
                        ValueType::Null => AttributeValueType::Empty,
                        _ => unreachable!(),
                    };

                    let scope = expr_source
                        .expr
                        .eval_scope()
                        .cloned()
                        .unwrap_or(DataScope::Root);
                    let source_expr = expr_source.expr.into_df_eval_expr().ok_or_else(|| {
                        Error::InvalidPipelineError {
                            cause: "invalid input to match".into(),
                            query_location: None,
                        }
                    })?;

                    let is_type_expr = Expr::ScalarFunction(ScalarFunction::new_udf(
                        Arc::new(ScalarUDF::new_from_shared_impl(Arc::new(
                            IsTypeFunc::for_any_value_subtype(subtype),
                        ))),
                        vec![source_expr],
                    ));

                    // Use eval_anyval_as_struct because the IsTypeFunc UDF
                    // operates directly on the AnyValue struct discriminator — no
                    // type-based partitioning needed.
                    return Ok(Some(ScopedExpr::Eval {
                        scope,
                        eval: LeafEval::new_df_expr_anyval_as_struct(is_type_expr, false)?,
                    }));
                }

                let expected_type = match value_type {
                    ValueType::Boolean => ExprLogicalType::Boolean,
                    ValueType::Bytes => ExprLogicalType::Binary,
                    ValueType::DateTime => ExprLogicalType::TimestampNanosecond,
                    ValueType::Double => ExprLogicalType::Float64,
                    ValueType::Integer => ExprLogicalType::AnyInt,
                    ValueType::String => ExprLogicalType::String,
                    ValueType::TimeSpan => ExprLogicalType::DurationNanoSecond,
                    _ => {
                        return Err(Error::NotYetSupportedError {
                            message: format!(
                                "type check logical expression using type {value_type:?} not yet supported"
                            ),
                        });
                    }
                };

                if expr_source.expr_type == expected_type {
                    return Ok(Some(ScopedExpr::Eval {
                        scope: DataScope::StaticScalar,
                        eval: LeafEval::new_df_expr(lit(true), false)?,
                    }));
                }

                if expr_source.expr_type.is_concrete() {
                    return Ok(Some(ScopedExpr::Eval {
                        scope: DataScope::StaticScalar,
                        eval: LeafEval::new_df_expr(lit(false), false)?,
                    }));
                }

                // dynamic type check using IsTypeFunc UDF
                let scope = expr_source
                    .expr
                    .eval_scope()
                    .cloned()
                    .unwrap_or(DataScope::Root);
                let source_expr = expr_source.expr.into_df_eval_expr().ok_or_else(|| {
                    Error::InvalidPipelineError {
                        cause: "invalid input to type check".into(),
                        query_location: None,
                    }
                })?;

                let is_type_expr = Expr::ScalarFunction(ScalarFunction::new_udf(
                    Arc::new(ScalarUDF::new_from_shared_impl(Arc::new(IsTypeFunc::new(
                        expected_type,
                    )))),
                    vec![source_expr],
                ));

                Ok(Some(ScopedExpr::Eval {
                    scope,
                    eval: LeafEval::new_df_expr(is_type_expr, false)?,
                }))
            }

            _ => Ok(None),
        }
    }

    /// Build a binary operation as either a single `Eval` (same-scope) or `JoinAndEval`
    /// (cross-scope).
    fn build_binary_expr(
        &self,
        mut left: PlannedOp,
        operator: Operator,
        mut right: PlannedOp,
        dict_downcast: bool,
    ) -> Result<ScopedExpr> {
        if self.plan_for_attributes {
            resolve_attr_value_column_in_planned_ops(&mut left, &mut right);
        }
        let possible_scope = try_combine_scopes(&left, &right);

        if let Some(scope) = possible_scope {
            let left_expr = left
                .expr
                .into_df_eval_expr()
                .unwrap_or(col(arg_column_name(0)));
            let right_expr = right
                .expr
                .into_df_eval_expr()
                .unwrap_or(col(arg_column_name(1)));

            Ok(ScopedExpr::Eval {
                scope,
                eval: LeafEval::new_df_expr_with_key_case(
                    Expr::BinaryExpr(BinaryExpr::new(
                        Box::new(left_expr),
                        operator,
                        Box::new(right_expr),
                    )),
                    dict_downcast,
                    self.attr_key_case_sensitive,
                )?,
            })
        } else {
            Ok(ScopedExpr::JoinAndEval {
                children: vec![left.expr, right.expr],
                default_null_children: false,
                align_children_to_root: false,
                eval: LeafEval::new_df_expr(
                    Expr::BinaryExpr(BinaryExpr::new(
                        Box::new(col(arg_column_name(0))),
                        operator,
                        Box::new(col(arg_column_name(1))),
                    )),
                    dict_downcast,
                )?,
            })
        }
    }

    /// Build either an `Eval` or `JoinAndEval` node depending on the function arg scope.
    fn build_eval_or_join(
        &self,
        expr: Expr,
        scope: FunctionArgScope,
        _eval_scope: Option<DataScope>,
        dict_downcast: bool,
    ) -> Result<ScopedExpr> {
        match scope {
            FunctionArgScope::Combined(scope) => Ok(ScopedExpr::Eval {
                scope,
                eval: LeafEval::new_df_expr_with_key_case(
                    expr,
                    dict_downcast,
                    self.attr_key_case_sensitive,
                )?,
            }),
            FunctionArgScope::Join(children) => Ok(ScopedExpr::JoinAndEval {
                children,
                default_null_children: false,
                align_children_to_root: false,
                eval: LeafEval::new_df_expr(expr, dict_downcast)?,
            }),
        }
    }
}

/// Result of checking if function arguments share a scope.
enum FunctionArgScope {
    /// All args share a combinable scope.
    Combined(DataScope),
    /// Args require a joining data with different scopes.
    Join(Vec<ScopedExpr>),
}

struct DataFusionFunctionDef {
    scalar_udf: Arc<ScalarUDF>,
    return_type: ExprLogicalType,
    requires_dict_downcast: bool,
    cast_result_to: Option<DataType>,
}

impl DataFusionFunctionDef {
    fn new(
        scalar_udf: Arc<ScalarUDF>,
        return_type: ExprLogicalType,
        requires_dict_downcast: bool,
        cast_result_to: Option<DataType>,
    ) -> Self {
        Self {
            scalar_udf,
            return_type,
            requires_dict_downcast,
            cast_result_to,
        }
    }

    fn from_func_name(func_name: &str) -> Option<Self> {
        // TODO: some functions may produce different result types depending on the input type.
        // In these cases, we may wish to not have a hard-coded return type, and instead attempt
        // to compute the return type from the types of the input expressions.
        // TODO: some of these functions that involve expanding to dictionary, we may wish to
        // implement our own versions that can operate directly on dictionary arrays (or fix this
        // upstream in datafusion_functions)
        Some(match func_name {
            ENCODE_FUNC_NAME => Self::new(encode(), ExprLogicalType::String, false, None),
            ENDS_WITH_FUNC_NAME => Self::new(ends_with(), ExprLogicalType::Boolean, true, None),
            LOG_FUNC_NAME => Self::new(log10(), ExprLogicalType::Float64, true, None),
            LTRIM_FUNC_NAME => Self::new(ltrim(), ExprLogicalType::String, true, None),
            REGEXP_SUBSTR_FUNC_NAME => {
                Self::new(regexp_substr(), ExprLogicalType::String, false, None)
            }
            FORMAT_DATETIME_FUNC_NAME => Self::new(to_char(), ExprLogicalType::String, false, None),
            RTRIM_FUNC_NAME => Self::new(rtrim(), ExprLogicalType::String, true, None),
            SHA256_FUNC_NAME => Self::new(sha256(), ExprLogicalType::Binary, true, None),
            MD5_FUNC_NAME => Self::new(md5(), ExprLogicalType::String, true, Some(DataType::Utf8)),
            FNV_FUNC_NAME => Self::new(fnv_hash(), ExprLogicalType::Int64, true, None),
            MURMUR3_FUNC_NAME => Self::new(murmur3_hash(), ExprLogicalType::Int64, true, None),
            #[cfg(feature = "sha1-hash")]
            SHA1_FUNC_NAME => Self::new(sha1_hash(), ExprLogicalType::Binary, true, None),
            SHA512_FUNC_NAME => Self::new(sha512(), ExprLogicalType::Binary, true, None),
            STARTS_WITH_FUNC_NAME => Self::new(starts_with(), ExprLogicalType::Boolean, true, None),
            XXH3_FUNC_NAME => Self::new(xxh3_hash(), ExprLogicalType::Int64, true, None),
            XXH128_FUNC_NAME => Self::new(xxh128_hash(), ExprLogicalType::Binary, true, None),
            UUID_FUNC_NAME => Self::new(uuid(), ExprLogicalType::String, false, None),
            UUIDV7_FUNC_NAME => Self::new(uuidv7(), ExprLogicalType::String, false, None),
            UPPER_CASE_FUNC_NAME => Self::new(upper(), ExprLogicalType::String, true, None),
            LOWER_CASE_FUNC_NAME => Self::new(lower(), ExprLogicalType::String, true, None),
            _ => return None,
        })
    }
}

impl ScopedExpr {
    /// If this is an `Eval` node, return a reference to its scope.
    pub(crate) fn eval_scope(&self) -> Option<&DataScope> {
        match self {
            Self::Eval { scope, .. } => Some(scope),
            _ => None,
        }
    }

    /// returns the scope of the data that would be produced if this Expr was evaluated to
    /// produce a scoped value
    pub(crate) fn effective_value_scope(&self) -> Result<Cow<'_, DataScope>> {
        match self {
            Self::Eval { scope, .. } => Ok(Cow::Borrowed(scope)),
            Self::JoinAndEval {
                children,
                align_children_to_root,
                ..
            } => {
                if children.is_empty() {
                    return Err(Error::InvalidPipelineError {
                        cause:
                            "Error computing expr scope. invalid join eval passed with no children"
                                .into(),
                        query_location: None,
                    });
                }

                if *align_children_to_root {
                    return Ok(Cow::Owned(DataScope::Root));
                }

                let mut curr_scope = children[0].effective_value_scope()?;

                // compute the data scope of what will be produced when the children are join:
                for child in children.iter().skip(1) {
                    let next_scope = child.effective_value_scope()?;
                    curr_scope = match (curr_scope.as_ref(), next_scope.as_ref()) {
                        (_, DataScope::StaticScalar | DataScope::AttributesAll(_)) => curr_scope,
                        (DataScope::StaticScalar | DataScope::AttributesAll(_), _) => next_scope,
                        (
                            DataScope::Attribute(left_attrs_id, _),
                            DataScope::Attribute(right_attrs_id, _),
                        ) => {
                            if left_attrs_id == right_attrs_id {
                                curr_scope
                            } else if is_one_to_many(left_attrs_id, right_attrs_id) {
                                next_scope
                            } else {
                                curr_scope
                            }
                        }
                        (
                            DataScope::Root | DataScope::RootParent(_),
                            DataScope::Attribute(_, _),
                        ) => curr_scope,
                        (
                            DataScope::Attribute(attr_id, _),
                            DataScope::Root | DataScope::RootParent(_),
                        ) => match attr_id {
                            AttributesIdentifier::Root => curr_scope,
                            AttributesIdentifier::NonRoot(_) => next_scope,
                        },

                        // rest always have root alignment
                        (DataScope::Root, DataScope::Root) => curr_scope,
                        (DataScope::RootParent(_), DataScope::Root) => curr_scope,
                        (DataScope::Root, DataScope::RootParent(_)) => curr_scope,
                        (DataScope::RootParent(_), DataScope::RootParent(_)) => curr_scope,
                    }
                }

                Ok(curr_scope)
            }
            Self::BitmapAnd(_, _) | Self::BitmapOr(_, _) | Self::BitmapNot(_) => {
                Ok(Cow::Owned(DataScope::Root))
            }
        }
    }

    /// Consume an `Eval(DatafusionExpr)` node and return its DataFusion logical expression.
    /// Returns `None` for non-`Eval` or `LeafEval::BatchPredicate` variants.
    pub(crate) fn into_df_eval_expr(self) -> Option<Expr> {
        match self {
            Self::Eval {
                eval: LeafEval::DatafusionExpr { logical_expr, .. },
                ..
            } => Some(logical_expr),
            _ => None,
        }
    }

    /// If this is an `Eval(DatafusionExpr)` node, return a reference to its DataFusion logical
    /// expression. Returns `None` for non-`Eval` or `BatchPredicate` variants.
    pub(crate) fn as_df_eval_expr_ref(&self) -> Option<&Expr> {
        match self {
            Self::Eval {
                eval: LeafEval::DatafusionExpr { logical_expr, .. },
                ..
            } => Some(logical_expr),
            _ => None,
        }
    }
}

fn is_literal_eval(plan: &PlannedOp) -> bool {
    matches!(
        plan.expr,
        ScopedExpr::Eval {
            eval: LeafEval::DatafusionExpr {
                logical_expr: Expr::Literal(_, _),
                ..
            },
            ..
        }
    )
}

/// Try to combine the scopes of two `PlannedOp`s. Returns `Some(scope)` if they're
/// combinable (same scope or one is scalar), `None` if they require a join.
fn try_combine_scopes(left: &PlannedOp, right: &PlannedOp) -> Option<DataScope> {
    let left_scope = left.expr.eval_scope()?;
    let right_scope = right.expr.eval_scope()?;

    if left_scope.can_combine(right_scope) {
        Some(if !left_scope.is_scalar() {
            left_scope.clone()
        } else {
            right_scope.clone()
        })
    } else {
        None
    }
}

/// Extract mutable references to the DataFusion expr and type from two `PlannedOp`s
/// for type coercion. Returns `None` for fields that aren't accessible (non-Eval nodes).
fn extract_coercion_parts<'a>(
    left: &'a mut PlannedOp,
    right: &'a mut PlannedOp,
) -> (
    Option<&'a mut Expr>,
    Option<&'a mut Expr>,
    Option<&'a mut ExprLogicalType>,
    Option<&'a mut ExprLogicalType>,
) {
    let left_expr = match &mut left.expr {
        ScopedExpr::Eval {
            eval: LeafEval::DatafusionExpr { logical_expr, .. },
            ..
        } => Some(logical_expr as &mut Expr),
        _ => None,
    };
    let right_expr = match &mut right.expr {
        ScopedExpr::Eval {
            eval: LeafEval::DatafusionExpr { logical_expr, .. },
            ..
        } => Some(logical_expr as &mut Expr),
        _ => None,
    };
    let left_type = Some(&mut left.expr_type);
    let right_type = Some(&mut right.expr_type);
    (left_expr, right_expr, left_type, right_type)
}

/// If the `PlannedOp` is a string literal `Eval`, escape LIKE special characters in it.
fn escape_like_literals(planned: &mut PlannedOp) {
    if let ScopedExpr::Eval {
        eval:
            LeafEval::DatafusionExpr {
                logical_expr: Expr::Literal(datafusion::scalar::ScalarValue::Utf8(Some(s)), _),
                ..
            },
        ..
    } = &mut planned.expr
    {
        if contains_like_pattern(s) {
            *s = escape_like_pattern(s);
        }
    }
}

/// Check whether a string contains LIKE-pattern special characters (`%`, `_`, `\`).
fn contains_like_pattern(pattern: &str) -> bool {
    memchr::memchr3(b'%', b'_', b'\\', pattern.as_bytes()).is_some()
}

/// Escape LIKE-pattern special characters in a string by prefixing them with `\`.
fn escape_like_pattern(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch == '%' || ch == '_' || ch == '\\' {
            result.push('\\');
        }
        result.push(ch);
    }
    result
}

/// Check if a `ScalarExpression` is a null literal.
fn is_null_literal(expr: &ScalarExpression) -> bool {
    matches!(
        expr,
        ScalarExpression::Static(StaticScalarExpression::Null(_))
    )
}

/// Check if a `PlannedOp` is a body column reference (Eval on root scope with `col("body")`
/// expression and AnyValue type).
fn is_body_planned_op(planned: &PlannedOp) -> bool {
    planned.expr_type == ExprLogicalType::AnyValue
        && matches!(
            &planned.expr,
            ScopedExpr::Eval {
                scope: DataScope::Root,
                eval: LeafEval::DatafusionExpr { logical_expr: Expr::Column(c), .. },
            } if c.name() == crate::consts::BODY_FIELD_NAME
        )
}

/// Check if a `PlannedOp` is a simple attribute value column reference.
///
/// Returns `true` if this is an attribute-scoped `Eval` with a simple `col("value")`
/// expression — the pattern produced by `plan_scalar` for `attributes["key"]`.
///
/// Returns `false` if the attribute value has been used in arithmetic, function calls,
/// or other complex expressions that require the actual typed value column to be
/// materialized.
fn is_simple_attr_value_column(planned: &PlannedOp) -> bool {
    matches!(
        &planned.expr,
        ScopedExpr::Eval {
            scope: DataScope::Attribute(_, _),
            eval: LeafEval::DatafusionExpr { logical_expr: Expr::Column(c), .. },
        } if c.name() == VALUE_COLUMN_NAME
    )
}

/// Check if a `PlannedOp` is a `col("value")` reference (used in attribute-level pipelines
/// where `"value"` is a virtual column that must be resolved to a concrete typed column).
fn is_attr_value_column(planned: &PlannedOp) -> bool {
    matches!(
        &planned.expr,
        ScopedExpr::Eval {
            eval: LeafEval::DatafusionExpr { logical_expr: Expr::Column(c), .. },
            ..
        } if c.name() == crate::consts::VALUE_FIELD_NAME
    )
}

/// When operating inside `apply attributes { ... }`, resolve the virtual `"value"` column
/// to the concrete typed attribute column (`str`, `int`, `double`, `bool`) based on the
/// type of the other operand. For example, `value > 10` becomes `int > 10`.
///
/// This replaces the `AttrValueColumnSelectionOptimizer` from the old filter planning path.
fn resolve_attr_value_column_in_planned_ops(left: &mut PlannedOp, right: &mut PlannedOp) {
    if is_attr_value_column(left) {
        if let Some(col_name) = get_literal_any_val_field(right) {
            rewrite_attr_value_column(left, col_name);
            return;
        }
    }

    if is_attr_value_column(right) {
        if let Some(col_name) = get_literal_any_val_field(left) {
            rewrite_attr_value_column(right, col_name);
        }
    }
}

/// Rewrite a `PlannedOp`'s expression from `col("value")` to `col(field_name)` (e.g.,
/// `col("str")`, `col("int")`).
fn rewrite_attr_value_column(planned: &mut PlannedOp, field_name: &str) {
    if let ScopedExpr::Eval {
        eval:
            LeafEval::DatafusionExpr {
                logical_expr,
                physical_expr,
                projection,
                ..
            },
        ..
    } = &mut planned.expr
    {
        let new_expr = col(field_name);
        *logical_expr = new_expr.clone();
        *physical_expr = None; // reset lazy physical expr
        if let Ok(new_proj) = Projection::try_new(&new_expr) {
            *projection = new_proj;
        }

        // update the expr type based on the column
        planned.expr_type = match field_name {
            consts::ATTRIBUTE_STR => ExprLogicalType::String,
            consts::ATTRIBUTE_INT => ExprLogicalType::Int64,
            consts::ATTRIBUTE_DOUBLE => ExprLogicalType::Float64,
            consts::ATTRIBUTE_BOOL => ExprLogicalType::Boolean,
            _ => ExprLogicalType::AnyValue,
        };
    }
}

/// When one side of a comparison is the `body` column (AnyValue type) and the other is a
/// typed literal, rewrite the body side's DataFusion expression from `col("body")` to
/// `col("body").field("str")` (or the appropriate sub-field based on the literal type).
fn resolve_body_field_in_planned_ops(left: &mut PlannedOp, right: &mut PlannedOp) {
    // body == literal: rewrite body
    if is_body_planned_op(left) {
        if let Some(field_name) = get_literal_any_val_field(right) {
            rewrite_body_expr(left, field_name);
            return;
        }
    }

    // literal == body: rewrite body
    if is_body_planned_op(right) {
        if let Some(field_name) = get_literal_any_val_field(left) {
            rewrite_body_expr(right, field_name);
        }
    }
}

/// If the `PlannedOp` is a static literal, return the AnyValue sub-field name for its type.
fn get_literal_any_val_field(planned: &PlannedOp) -> Option<&'static str> {
    attr_value_column_for_expr_type(&planned.expr_type)
}

/// Map an `ExprLogicalType` to the corresponding attribute value column name.
///
/// Returns the column name in the OTAP attributes RecordBatch schema for this type:
/// - String -> "str"
/// - Int32/Int64/AnyInt -> "int"
/// - Float64 -> "double"
/// - Boolean -> "bool"
fn attr_value_column_for_expr_type(expr_type: &ExprLogicalType) -> Option<&'static str> {
    use otap_df_pdata::schema::consts as pdata_consts;

    match expr_type {
        ExprLogicalType::String => Some(pdata_consts::ATTRIBUTE_STR),
        ExprLogicalType::AnyInt | ExprLogicalType::Int32 | ExprLogicalType::Int64 => {
            Some(pdata_consts::ATTRIBUTE_INT)
        }
        ExprLogicalType::Float64 => Some(pdata_consts::ATTRIBUTE_DOUBLE),
        ExprLogicalType::Boolean => Some(pdata_consts::ATTRIBUTE_BOOL),
        _ => None,
    }
}

/// Flip comparison operators for cases where the operand positions are swapped.
///
/// When we detect `"literal" > attributes["x"]` but want to express it as
/// `attributes["x"] < "literal"`, we need to flip the operator.
fn flip_comparison_operator(op: Operator) -> Operator {
    match op {
        Operator::Gt => Operator::Lt,
        Operator::GtEq => Operator::LtEq,
        Operator::Lt => Operator::Gt,
        Operator::LtEq => Operator::GtEq,
        // Eq, NotEq, etc. are symmetric
        other => other,
    }
}

/// Rewrite a body `PlannedOp`'s expression from `col("body")` to
/// `col("body").field(field_name)`.
fn rewrite_body_expr(planned: &mut PlannedOp, field_name: &str) {
    use datafusion::functions::core::expr_ext::FieldAccessor;

    if let ScopedExpr::Eval {
        eval:
            LeafEval::DatafusionExpr {
                logical_expr,
                physical_expr,
                projection,
                eval_anyval_as_struct,
                ..
            },
        ..
    } = &mut planned.expr
    {
        let new_expr = col(crate::consts::BODY_FIELD_NAME).field(field_name);
        *logical_expr = new_expr.clone();
        *physical_expr = None; // reset lazy physical expr
        *eval_anyval_as_struct = false;
        if let Ok(new_proj) = Projection::try_new(&new_expr) {
            *projection = new_proj;
        }

        // update the expr type based on the field
        planned.expr_type = match field_name {
            consts::ATTRIBUTE_STR => ExprLogicalType::String,
            consts::ATTRIBUTE_INT => ExprLogicalType::Int64,
            consts::ATTRIBUTE_DOUBLE => ExprLogicalType::Float64,
            consts::ATTRIBUTE_BOOL => ExprLogicalType::Boolean,
            _ => ExprLogicalType::AnyValue,
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use data_engine_expressions::{
        EqualToLogicalExpression, GetRecordTypeScalarExpression, GreaterThanLogicalExpression,
        IntegerScalarExpression, NotLogicalExpression, QueryLocation, SourceScalarExpression,
        ValueAccessor,
    };
    use datafusion::common::cast::as_boolean_array;
    use datafusion::logical_expr::ColumnarValue;
    use datafusion::scalar::ScalarValue;
    use otap_df_pdata::otap::filter::IdBitmapPool;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
    use otap_df_pdata::testing::round_trip::{otlp_to_otap, to_logs_data};

    use crate::pipeline::Pipeline;
    use crate::pipeline::expr::{DataScope, ScopedExpr};
    use crate::pipeline::id_mask::IdMask;
    use crate::pipeline::planner::AttributesIdentifier;

    fn ql() -> QueryLocation {
        QueryLocation::new_fake()
    }

    fn make_column_expr(col_name: &str) -> ScalarExpression {
        ScalarExpression::Source(SourceScalarExpression::new(
            ql(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(ql(), col_name)),
            )]),
        ))
    }

    fn make_attr_expr(key: &str) -> ScalarExpression {
        ScalarExpression::Source(SourceScalarExpression::new(
            ql(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(ql(), "attributes"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(ql(), key),
                )),
            ]),
        ))
    }

    fn make_int_literal(v: i64) -> ScalarExpression {
        ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(ql(), v),
        ))
    }

    fn make_string_literal(s: &str) -> ScalarExpression {
        ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
            ql(),
            s,
        )))
    }

    /// Test OTAP batch: 3 log records with severity fields and attributes.
    fn test_otap() -> otap_df_pdata::OtapArrowRecords {
        let logs = to_logs_data(vec![
            LogRecord::build()
                .severity_text("WARN")
                .severity_number(13)
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .event_name("e1")
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .severity_number(17)
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                .event_name("e2")
                .finish(),
            LogRecord::build()
                .severity_text("WARN")
                .severity_number(13)
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .event_name("e3")
                .finish(),
        ]);
        otlp_to_otap(&OtlpProtoMessage::Logs(logs))
    }

    #[test]
    fn test_plan_column_reference() {
        let planner = ExprPlanner::new();
        let expr = make_column_expr("severity_text");
        let planned = planner.plan_scalar(&expr, &[]).unwrap();

        // should produce an Eval with Root scope
        assert!(matches!(
            planned.expr,
            ScopedExpr::Eval {
                scope: DataScope::Root,
                ..
            }
        ));
        assert_eq!(planned.expr_type, ExprLogicalType::String);

        // execute and verify
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut op = planned.expr;
        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        assert_eq!(result.scope, DataScope::Root);

        // should have 3 string values
        match &result.values {
            ColumnarValue::Array(arr) => assert_eq!(arr.len(), 3),
            other => panic!("expected array, got {other:?}"),
        }
    }

    #[test]
    fn test_plan_attribute_access() {
        let planner = ExprPlanner::new();
        let expr = make_attr_expr("x");
        let planned = planner.plan_scalar(&expr, &[]).unwrap();

        assert!(matches!(
            planned.expr,
            ScopedExpr::Eval {
                scope: DataScope::Attribute(AttributesIdentifier::Root, _),
                ..
            }
        ));
        assert_eq!(planned.expr_type, ExprLogicalType::AnyValue);

        // execute and verify
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut op = planned.expr;
        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        assert!(matches!(
            result.scope,
            DataScope::Attribute(AttributesIdentifier::Root, _)
        ));
        // 3 attribute rows (one per log record, each has key "x")
        match &result.values {
            ColumnarValue::Array(arr) => assert_eq!(arr.len(), 3),
            other => panic!("expected array, got {other:?}"),
        }
    }

    #[test]
    fn test_plan_static_literal() {
        let planner = ExprPlanner::new();
        let expr = make_int_literal(42);
        let planned = planner.plan_scalar(&expr, &[]).unwrap();

        assert!(matches!(
            planned.expr,
            ScopedExpr::Eval {
                scope: DataScope::StaticScalar,
                ..
            }
        ));
        assert_eq!(planned.expr_type, ExprLogicalType::AnyInt);

        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut op = planned.expr;
        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        match &result.values {
            ColumnarValue::Scalar(ScalarValue::Int64(Some(42))) => {}
            other => panic!("expected Int64(42), got {other:?}"),
        }
    }

    #[test]
    fn test_plan_same_scope_arithmetic() {
        use data_engine_expressions::{BinaryMathematicalScalarExpression, MathScalarExpression};

        let planner = ExprPlanner::new();
        let left = make_column_expr("severity_number");
        let right = make_int_literal(2);
        let binary = BinaryMathematicalScalarExpression::new(ql(), left, right);
        let expr = ScalarExpression::Math(MathScalarExpression::Add(binary));

        let planned = planner.plan_scalar(&expr, &[]).unwrap();

        // same scope (root + scalar) -> single Eval with Root scope
        assert!(matches!(
            planned.expr,
            ScopedExpr::Eval {
                scope: DataScope::Root,
                ..
            }
        ));

        // execute and verify: severity_numbers are [13, 17, 13], +2 = [15, 19, 15]
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut op = planned.expr;
        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        match &result.values {
            ColumnarValue::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            other => panic!("expected array, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Test 5: Cross-scope arithmetic -> JoinAndEval
    // -----------------------------------------------------------------------

    #[test]
    fn test_plan_cross_scope_arithmetic() {
        use data_engine_expressions::{BinaryMathematicalScalarExpression, MathScalarExpression};

        let planner = ExprPlanner::new();
        let left = make_column_expr("severity_number");
        let right = make_attr_expr("x");
        let binary = BinaryMathematicalScalarExpression::new(ql(), left, right);
        let expr = ScalarExpression::Math(MathScalarExpression::Add(binary));

        let planned = planner.plan_scalar(&expr, &[]).unwrap();

        // different scopes (root + attrs) -> JoinAndEval
        assert!(matches!(planned.expr, ScopedExpr::JoinAndEval { .. }));
    }

    #[test]
    fn test_plan_same_scope_comparison() {
        let planner = ExprPlanner::new();
        let left = make_column_expr("severity_text");
        let right = make_string_literal("WARN");

        let eq_expr = EqualToLogicalExpression::new(ql(), left, right, false);
        let logical = LogicalExpression::EqualTo(eq_expr);

        let op = planner.plan_logical(&logical, &[]).unwrap();

        // same scope (root + scalar) -> single Eval with Root scope
        assert!(matches!(
            op,
            ScopedExpr::Eval {
                scope: DataScope::Root,
                ..
            }
        ));

        // execute and verify: rows 0 and 2 have severity_text="WARN"
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut op = op;
        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();
        let bool_arr = as_boolean_array(match &result.values {
            ColumnarValue::Array(arr) => arr,
            other => panic!("expected array, got {other:?}"),
        })
        .unwrap();
        assert!(bool_arr.value(0));
        assert!(!bool_arr.value(1));
        assert!(bool_arr.value(2));
    }

    #[test]
    fn test_plan_and_two_root_predicates() {
        let planner = ExprPlanner::new();

        // severity_text == "WARN" AND severity_number > 10
        let left_eq = LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            ql(),
            make_column_expr("severity_text"),
            make_string_literal("WARN"),
            false,
        ));
        let right_gt = LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
            ql(),
            make_column_expr("severity_number"),
            make_int_literal(10),
        ));

        let and_expr = data_engine_expressions::AndLogicalExpression::new(ql(), left_eq, right_gt);
        let logical = LogicalExpression::And(and_expr);

        let op = planner.plan_logical(&logical, &[]).unwrap();

        // should be a Eval because they're a combinable source
        assert!(matches!(op, ScopedExpr::Eval { .. }));

        // execute and verify: all 3 rows have severity_number > 10 AND severity_text="WARN"
        // -> rows 0 and 2 pass (row 1 has severity_text="ERROR")
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();
        let mut op = op;
        let result = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        match &result.mask {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(0), "row 0 should pass");
                assert!(!bitmap.contains(1), "row 1 (ERROR) should not pass");
                assert!(bitmap.contains(2), "row 2 should pass");
            }
            other => panic!("expected Some bitmap, got {other:?}"),
        }
    }

    #[test]
    fn test_plan_signal_type_check() {
        let planner = ExprPlanner::new();

        let get_record_type =
            ScalarExpression::GetRecordType(GetRecordTypeScalarExpression::new(ql()));
        let type_name = make_string_literal("Log");
        let eq_expr = EqualToLogicalExpression::new(ql(), get_record_type, type_name, false);
        let logical = LogicalExpression::EqualTo(eq_expr);

        let op = planner.plan_logical(&logical, &[]).unwrap();

        // should produce an Eval(BatchPredicate)
        assert!(matches!(
            op,
            ScopedExpr::Eval {
                eval: LeafEval::BatchPredicate(_),
                ..
            }
        ));

        // on a logs batch -> IdMask::All
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();
        let mut op = op;
        let result = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();
        assert_eq!(result.mask, IdMask::All);
    }

    #[test]
    fn test_plan_scalar_logical() {
        let planner = ExprPlanner::new();

        // Logical(severity_text == "WARN") as a scalar expression
        let inner = LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            ql(),
            make_column_expr("severity_text"),
            make_string_literal("WARN"),
            false,
        ));
        let expr = ScalarExpression::Logical(Box::new(inner));

        let planned = planner.plan_scalar(&expr, &[]).unwrap();
        assert_eq!(planned.expr_type, ExprLogicalType::Boolean);

        // execute and verify it produces a boolean result
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut op = planned.expr;
        let result = op.execute_as_value(&otap, &session_ctx).unwrap().unwrap();

        // the value should be a boolean array
        let bool_arr = as_boolean_array(match &result.values {
            ColumnarValue::Array(arr) => arr,
            other => panic!("expected array, got {other:?}"),
        })
        .unwrap();
        assert!(bool_arr.value(0));
        assert!(!bool_arr.value(1));
        assert!(bool_arr.value(2));
    }

    #[test]
    fn test_plan_not() {
        let planner = ExprPlanner::new();

        let inner = LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            ql(),
            make_column_expr("severity_text"),
            make_string_literal("WARN"),
            false,
        ));
        let not_expr = NotLogicalExpression::new(ql(), inner);
        let logical = LogicalExpression::Not(not_expr);

        let op = planner.plan_logical(&logical, &[]).unwrap();
        // should be eval b/c it's a combinable source
        assert!(matches!(op, ScopedExpr::Eval { .. }));

        // execute: NOT(WARN) should match row 1 (ERROR)
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();
        let mut op = op;
        let result = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        // not(Some({0, 2})) = matches everything except 0 and 2 = matches 1
        match &result.mask {
            IdMask::Some(bitmap) => {
                assert!(!bitmap.contains(0), "0 in negated set");
                assert!(bitmap.contains(1), "1 not in negated set");
                assert!(!bitmap.contains(2), "2 in negated set");
            }
            other => panic!("expected NotSome, got {other:?}"),
        }
    }

    #[test]
    fn test_plan_fused_attr_eq_string() {
        let planner = ExprPlanner::new();

        // attributes["x"] == "a"
        let attr_expr = make_attr_expr("x");
        let literal_expr = make_string_literal("a");
        let eq_expr = EqualToLogicalExpression::new(ql(), attr_expr, literal_expr, false);
        let logical = LogicalExpression::EqualTo(eq_expr);

        let op = planner.plan_logical(&logical, &[]).unwrap();

        // Should produce an Eval with AttributesUnfiltered scope (fused optimization)
        assert!(matches!(
            op,
            ScopedExpr::Eval {
                scope: DataScope::AttributesAll(AttributesIdentifier::Root),
                ..
            }
        ));

        // Execute and verify: rows 0 and 2 have x="a", row 1 has x="b"
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();
        let mut op = op;
        let result = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        match &result.mask {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(0), "row 0 (x=a) should pass");
                assert!(!bitmap.contains(1), "row 1 (x=b) should not pass");
                assert!(bitmap.contains(2), "row 2 (x=a) should pass");
            }
            other => panic!("expected Some bitmap, got {other:?}"),
        }
    }

    #[test]
    fn test_plan_fused_attr_eq_literal_on_left() {
        let planner = ExprPlanner::new();

        // "a" == attributes["x"] (literal on left)
        let literal_expr = make_string_literal("a");
        let attr_expr = make_attr_expr("x");
        let eq_expr = EqualToLogicalExpression::new(ql(), literal_expr, attr_expr, false);
        let logical = LogicalExpression::EqualTo(eq_expr);

        let op = planner.plan_logical(&logical, &[]).unwrap();

        // Should still produce fused optimization
        assert!(matches!(
            op,
            ScopedExpr::Eval {
                scope: DataScope::AttributesAll(AttributesIdentifier::Root),
                ..
            }
        ));

        // Execute and verify
        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();
        let mut op = op;
        let result = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        match &result.mask {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(0), "row 0 should pass");
                assert!(!bitmap.contains(1), "row 1 should not pass");
                assert!(bitmap.contains(2), "row 2 should pass");
            }
            other => panic!("expected Some bitmap, got {other:?}"),
        }
    }

    #[test]
    fn test_plan_fused_attr_gt_integer() {
        use data_engine_expressions::GreaterThanLogicalExpression;

        // Test batch with integer attributes
        let logs = to_logs_data(vec![
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("count", AnyValue::new_int(10))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("count", AnyValue::new_int(5))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("count", AnyValue::new_int(20))])
                .finish(),
        ]);
        let otap = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        let planner = ExprPlanner::new();

        // attributes["count"] > 7
        let attr_expr = make_attr_expr("count");
        let literal_expr = make_int_literal(7);
        let gt_expr = GreaterThanLogicalExpression::new(ql(), attr_expr, literal_expr);
        let logical = LogicalExpression::GreaterThan(gt_expr);

        let op = planner.plan_logical(&logical, &[]).unwrap();

        // Should produce fused optimization
        assert!(matches!(
            op,
            ScopedExpr::Eval {
                scope: DataScope::AttributesAll(AttributesIdentifier::Root),
                ..
            }
        ));

        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();
        let mut op = op;
        let result = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        match &result.mask {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(0), "row 0 (count=10 > 7) should pass");
                assert!(!bitmap.contains(1), "row 1 (count=5 < 7) should not pass");
                assert!(bitmap.contains(2), "row 2 (count=20 > 7) should pass");
            }
            other => panic!("expected Some bitmap, got {other:?}"),
        }
    }

    #[test]
    fn test_plan_fused_attr_two_attrs_combined_with_and() {
        // attributes["x"] == "a" AND attributes["x"] == "a" (same key, same value)
        // Both should use fused paths and then BitmapAnd combines them

        let planner = ExprPlanner::new();

        let left = LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            ql(),
            make_attr_expr("x"),
            make_string_literal("a"),
            false,
        ));
        let right = LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            ql(),
            make_attr_expr("x"),
            make_string_literal("a"),
            false,
        ));

        let and_expr = data_engine_expressions::AndLogicalExpression::new(ql(), left, right);
        let logical = LogicalExpression::And(and_expr);

        let op = planner.plan_logical(&logical, &[]).unwrap();

        // Should be BitmapAnd of two fused evals
        assert!(matches!(op, ScopedExpr::BitmapAnd(_, _)));

        let otap = test_otap();
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();
        let mut op = op;
        let result = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        match &result.mask {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(0), "row 0 should pass");
                assert!(!bitmap.contains(1), "row 1 should not pass");
                assert!(bitmap.contains(2), "row 2 should pass");
            }
            other => panic!("expected Some bitmap, got {other:?}"),
        }
    }

    #[test]
    fn test_plan_no_fused_for_attr_plus_arithmetic() {
        // attributes["x"] + 2 > 5 should NOT use fused path (attribute in arithmetic)
        // But just attributes["x"] > 5 SHOULD use fused path

        use data_engine_expressions::{
            BinaryMathematicalScalarExpression, GreaterThanLogicalExpression, MathScalarExpression,
        };

        let logs = to_logs_data(vec![
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("num", AnyValue::new_int(10))])
                .finish(),
        ]);
        let otap = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        let planner = ExprPlanner::new();

        // attributes["num"] + 2 > 5
        let attr_expr = make_attr_expr("num");
        let two = make_int_literal(2);
        let add_expr = BinaryMathematicalScalarExpression::new(ql(), attr_expr, two);
        let math_expr = ScalarExpression::Math(MathScalarExpression::Add(add_expr));

        let gt_expr = GreaterThanLogicalExpression::new(ql(), math_expr, make_int_literal(5));
        let logical = LogicalExpression::GreaterThan(gt_expr);

        let op = planner.plan_logical(&logical, &[]).unwrap();

        // Should NOT be fused (math on attribute requires value materialization)
        // The left side is JoinAndEval or Eval with Attributes scope
        // but NOT AttributesUnfiltered
        let is_fused = matches!(
            &op,
            ScopedExpr::Eval {
                scope: DataScope::AttributesAll(_),
                ..
            }
        );
        assert!(
            !is_fused,
            "arithmetic on attribute should not use fused path"
        );

        // Still should execute correctly
        let session_ctx = Pipeline::create_session_context();
        let mut pool = IdBitmapPool::new();
        let mut op = op;
        let result = op
            .execute_as_id_mask(&otap, &session_ctx, &mut pool)
            .unwrap();

        // 10 + 2 = 12 > 5 -> should pass
        match &result.mask {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(0), "row 0 should pass");
            }
            IdMask::All => {} // Also acceptable
            other => panic!("expected Some or All, got {other:?}"),
        }
    }
}
