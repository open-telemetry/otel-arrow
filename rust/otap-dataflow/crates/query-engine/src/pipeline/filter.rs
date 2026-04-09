// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::functions::expr_fn::contains;
use crate::pipeline::planner::{
    AttributesIdentifier, BinaryArg, ColumnAccessor, try_attrs_value_filter_from_literal,
    try_static_scalar_to_attr_literal, try_static_scalar_to_literal_for_column,
};
use crate::pipeline::project::Projection;
use crate::pipeline::state::ExecutionState;
use arrow::array::{Array, BooleanArray, BooleanBufferBuilder, RecordBatch, UInt16Array};
use arrow::buffer::BooleanBuffer;
use arrow::compute::{and, filter_record_batch, not, or};
use arrow::datatypes::UInt16Type;
use async_trait::async_trait;
use data_engine_expressions::{
    BooleanValue, ContainsLogicalExpression, Expression, LogicalExpression,
    MatchesLogicalExpression, ScalarExpression, StaticScalarExpression, StringScalarExpression,
    StringValue,
};
use datafusion::common::DFSchema;
use datafusion::common::cast::as_boolean_array;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use datafusion::functions::core::expr_ext::FieldAccessor;
use datafusion::logical_expr::{BinaryExpr, Expr, Operator, col, lit};
use datafusion::physical_expr::{PhysicalExprRef, create_physical_expr};
use datafusion::prelude::binary_expr;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::arrays::MaybeDictArrayAccessor;
use otap_df_pdata::otap::filter::{ChildBatchFilterIdHelper, IdBitmap, IdBitmapPool};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

pub mod optimize;

/// A compositional tree structure for combining expressions with boolean operators.
///
/// Represents logical combinations of base values using Not, And, and Or operations,
/// forming a tree that can be evaluated or transformed.
#[derive(Clone, Debug, PartialEq)]
pub enum Composite<T> {
    Base(T),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl<T> Composite<T> {
    /// helper function to create the `Composite::And` variant from passed args
    pub fn and<L, R>(left: L, right: R) -> Self
    where
        L: Into<Self>,
        R: Into<Self>,
    {
        Self::And(Box::new(left.into()), Box::new(right.into()))
    }

    /// helper function to create the `Composite::Or` variant from passed args
    pub fn or<L, R>(left: L, right: R) -> Self
    where
        L: Into<Self>,
        R: Into<Self>,
    {
        Self::Or(Box::new(left.into()), Box::new(right.into()))
    }

    /// helper function to create the `Composite::Not` variant from passed args
    pub fn not<P>(inner: P) -> Self
    where
        P: Into<Self>,
    {
        Self::Not(Box::new(inner.into()))
    }
}

/// A helper trait that can be used to transform a composite logical plan into a composite
/// executable plan.
pub trait ToExec {
    type ExecutablePlan;

    fn to_exec(
        &self,
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Self::ExecutablePlan>;
}

impl<T, B> Composite<T>
where
    T: ToExec<ExecutablePlan = B>,
{
    /// transform the composite logical plan into a composite executable plan by calling `to_exec`
    /// on the base type
    pub fn to_exec(
        &self,
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Composite<B>> {
        match self {
            Self::Base(inner) => {
                let exec = inner.to_exec(session_ctx, otap_batch)?;
                Ok(Composite::Base(exec))
            }
            Self::Not(inner) => {
                let exec = inner.to_exec(session_ctx, otap_batch)?;
                Ok(Composite::not(exec))
            }
            Self::And(left, right) => {
                let left_exec = left.to_exec(session_ctx, otap_batch)?;
                let right_exec = right.to_exec(session_ctx, otap_batch)?;
                Ok(Composite::and(left_exec, right_exec))
            }
            Self::Or(left, right) => {
                let left_exec = left.to_exec(session_ctx, otap_batch)?;
                let right_exec = right.to_exec(session_ctx, otap_batch)?;
                Ok(Composite::or(left_exec, right_exec))
            }
        }
    }
}

/// helper for creating the Composite type from the base type
impl<T> From<T> for Composite<T> {
    fn from(value: T) -> Self {
        Self::Base(value)
    }
}

/// A logical plan for filtering data across root batch's columns and attributes.
///
/// Supports two types of filters that can be applied independently or together:
/// - `source_filter`: Filters on regular columns in the source data
/// - `attribute_filter`: Filters on key-value attribute pairs
///
/// When both types of filters are present, the resulting execution of the plan will be the
/// intersection of the two filters.
///
/// Can be constructed from either a DataFusion `Expr` (for root batch's filters) or an
/// `AttributesFilterPlan` (for attribute filters), and can be composed into boolean expressions
/// using `Composite`.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterPlan {
    /// filters that will be applied to the root record batch
    pub source_filter: Option<Expr>,

    /// filters that will be applied to the attributes record batch in order fo filter the
    /// rows of the root batch
    pub attribute_filter: Option<Composite<AttributesFilterPlan>>,
}

impl From<Expr> for FilterPlan {
    fn from(expr: Expr) -> Self {
        Self {
            source_filter: Some(expr),
            attribute_filter: None,
        }
    }
}

impl From<AttributesFilterPlan> for FilterPlan {
    fn from(attrs_filter: AttributesFilterPlan) -> Self {
        Self {
            source_filter: None,
            attribute_filter: Some(attrs_filter.into()),
        }
    }
}

impl From<Composite<AttributesFilterPlan>> for FilterPlan {
    fn from(attrs_filter: Composite<AttributesFilterPlan>) -> Self {
        Self {
            source_filter: None,
            attribute_filter: Some(attrs_filter),
        }
    }
}

impl FilterPlan {
    /// Try to create a [`FilterPlan`] representing a comparison between the left scalar expression
    /// and right scalar expression using the given binary_op for comparison.
    ///
    /// The scalar expression on either side can represent:
    /// - an attribute (e.g. attributes["x"], resource.attributes["x"], etc.)
    /// - a column (e.g. severity_text, event_name, etc.)
    /// - a column nested within a struct (e.g. resource.schema_url, instrumentation_scope.name, etc.)
    /// - a literal (e.g. "a", 1234, true, etc.)
    ///
    fn try_from_binary_expr(
        left_expr: &ScalarExpression,
        mut binary_op: Operator,
        right_expr: &ScalarExpression,
        case_sensitive: bool,
        attr_keys_case_sensitive: bool,
    ) -> Result<Self> {
        let mut left_arg = BinaryArg::try_from(left_expr)?;
        let mut right_arg = BinaryArg::try_from(right_expr)?;

        // don't allow non equals comparisons for null
        if binary_op != Operator::Eq
            && (matches!(left_arg, BinaryArg::Null) || matches!(right_arg, BinaryArg::Null))
        {
            return Err(Error::InvalidPipelineError {
                cause: format!(
                    "cannot compare null using operator {}. only == is allowed",
                    binary_op
                ),
                query_location: None,
            });
        }

        if !case_sensitive {
            Self::transform_case_insensitive_equals(&mut left_arg, &mut binary_op, &mut right_arg);
        }

        // TODO there are several branches below which are not yet supported
        // - comparing two literals. e.g "a" == "b"
        // - comparing non-literal left with non-literal right. e.g.
        //   - severity_text == event_name
        //   - attributes["x"] == severity_text
        //   - etc.

        match left_arg {
            BinaryArg::Column(left_column) => match left_column {
                ColumnAccessor::ColumnName(left_col_name) => match right_arg {
                    BinaryArg::Literal(right_lit) => {
                        // left = column & right = literal
                        let right_expr =
                            try_static_scalar_to_literal_for_column(&left_col_name, &right_lit)?;
                        Ok(FilterPlan::from(Expr::BinaryExpr(BinaryExpr::new(
                            Box::new(col(left_col_name)),
                            binary_op,
                            Box::new(right_expr),
                        ))))
                    }
                    BinaryArg::Null => {
                        // left = column & right == null
                        Ok(FilterPlan::from(col(left_col_name).is_null()))
                    }
                    _ => Err(Error::NotYetSupportedError {
                        message: "comparing left column with non-literal right in filter.".into(),
                    }),
                },
                ColumnAccessor::StructCol(left_struct_name, left_struct_field) => match right_arg {
                    BinaryArg::Literal(right_lit) => {
                        // left = struct col & right = literal
                        let right_expr = try_static_scalar_to_literal_for_column(
                            &left_struct_field,
                            &right_lit,
                        )?;
                        Ok(FilterPlan::from(Expr::BinaryExpr(BinaryExpr::new(
                            Box::new(col(left_struct_name).field(left_struct_field)),
                            binary_op,
                            Box::new(right_expr),
                        ))))
                    }
                    BinaryArg::Null => {
                        // left = struct col & right = null
                        Ok(FilterPlan::from(
                            col(left_struct_name).field(left_struct_field).is_null(),
                        ))
                    }
                    _ => Err(Error::NotYetSupportedError {
                        message: "comparing left struct column with non-literal right in filter"
                            .into(),
                    }),
                },
                ColumnAccessor::Attributes(attrs_identifier, attrs_key) => {
                    match right_arg {
                        BinaryArg::Literal(right_lit) => {
                            // left = attribute & right = literal
                            Ok(FilterPlan::from(AttributesFilterPlan::new(
                                // col(consts::ATTRIBUTE_KEY).eq(lit(attrs_key))
                                Self::attr_key_equals(&attrs_key, attr_keys_case_sensitive).and(
                                    Expr::BinaryExpr(try_attrs_value_filter_from_literal(
                                        &right_lit, binary_op,
                                    )?),
                                ),
                                attrs_identifier,
                            )))
                        }
                        BinaryArg::Null => {
                            // left = attribute & right = null (e.g. doesn't have attribute)
                            Ok(FilterPlan::from(Composite::not(AttributesFilterPlan::new(
                                Self::attr_key_equals(&attrs_key, attr_keys_case_sensitive),
                                // col(consts::ATTRIBUTE_KEY).eq(lit(attrs_key)),
                                attrs_identifier,
                            ))))
                        }
                        _ => Err(Error::NotYetSupportedError {
                            message: "comparing left attribute with non-literal right in filter"
                                .into(),
                        }),
                    }
                }
            },
            BinaryArg::Literal(left_lit) => match right_arg {
                BinaryArg::Literal(_right_lit) => Err(Error::NotYetSupportedError {
                    message: "comparing literals in filter".into(),
                }),
                BinaryArg::Column(right_column) => match right_column {
                    ColumnAccessor::ColumnName(right_col_name) => {
                        // left = literal & right = column
                        let left_expr =
                            try_static_scalar_to_literal_for_column(&right_col_name, &left_lit)?;
                        Ok(FilterPlan::from(Expr::BinaryExpr(BinaryExpr::new(
                            Box::new(left_expr),
                            binary_op,
                            Box::new(col(right_col_name)),
                        ))))
                    }
                    ColumnAccessor::StructCol(right_struct_name, right_struct_field) => {
                        // left = literal & right = struct col
                        let left_expr = try_static_scalar_to_literal_for_column(
                            &right_struct_field,
                            &left_lit,
                        )?;
                        Ok(FilterPlan::from(Expr::BinaryExpr(BinaryExpr::new(
                            Box::new(left_expr),
                            binary_op,
                            Box::new(col(right_struct_name).field(right_struct_field)),
                        ))))
                    }
                    ColumnAccessor::Attributes(attrs_identifier, attrs_key) => {
                        // left = literal & right = attribute
                        Ok(FilterPlan::from(AttributesFilterPlan::new(
                            // col(consts::ATTRIBUTE_KEY)
                            //     .eq(lit(attrs_key))
                            Self::attr_key_equals(&attrs_key, attr_keys_case_sensitive).and(
                                Expr::BinaryExpr(try_attrs_value_filter_from_literal(
                                    &left_lit, binary_op,
                                )?),
                            ),
                            attrs_identifier,
                        )))
                    }
                },
                BinaryArg::Null => {
                    // literal == null
                    Err(Error::NotYetSupportedError {
                        message: "comparing left literal with right null".into(),
                    })
                }
            },
            BinaryArg::Null => match right_arg {
                BinaryArg::Column(right_column) => match right_column {
                    ColumnAccessor::ColumnName(right_col_name) => {
                        // left = null & right = column
                        Ok(FilterPlan::from(col(right_col_name).is_null()))
                    }
                    ColumnAccessor::StructCol(right_struct_name, right_struct_field) => {
                        // left = null, right = struct column
                        Ok(FilterPlan::from(
                            col(right_struct_name).field(right_struct_field).is_null(),
                        ))
                    }
                    ColumnAccessor::Attributes(attrs_identifier, attrs_key) => {
                        // left = null & right = attribute (e.g. doesn't have attribute)
                        Ok(FilterPlan::from(Composite::not(AttributesFilterPlan::new(
                            Self::attr_key_equals(&attrs_key, attr_keys_case_sensitive),
                            attrs_identifier,
                        ))))
                    }
                },
                BinaryArg::Literal(_lit) => {
                    // null == lit
                    Err(Error::NotYetSupportedError {
                        message: "comparing left null with right literal".into(),
                    })
                }
                BinaryArg::Null => {
                    // null == null
                    Err(Error::NotYetSupportedError {
                        message: "comparing left null with right null".into(),
                    })
                }
            },
        }
    }

    fn try_from_contains_expr(
        contains_expr: &ContainsLogicalExpression,
        attr_keys_case_sensitive: bool,
    ) -> Result<Self> {
        let left_arg = BinaryArg::try_from(contains_expr.get_haystack())?;
        let right_arg = BinaryArg::try_from(contains_expr.get_needle())?;

        match left_arg {
            BinaryArg::Column(left_column) => {
                let (left_expr, attrs) = Self::contains_column_arg(left_column);
                let right_expr = match right_arg {
                    BinaryArg::Literal(right_lit) => try_static_scalar_to_attr_literal(&right_lit)?,
                    _ => {
                        return Err(Error::NotYetSupportedError {
                            message:
                                "text contains predicate comparing column left to non literal right"
                                    .into(),
                        });
                    }
                };

                let contains_expr = contains(left_expr, right_expr);
                Ok(match attrs {
                    None => FilterPlan::from(contains_expr),
                    Some((attrs_identifier, attrs_key)) => {
                        FilterPlan::from(AttributesFilterPlan::new(
                            Self::attr_key_equals(&attrs_key, attr_keys_case_sensitive)
                                .and(contains_expr),
                            attrs_identifier,
                        ))
                    }
                })
            }
            BinaryArg::Literal(left_lit) => {
                let left_expr = try_static_scalar_to_attr_literal(&left_lit)?;
                let (right_expr, attrs) = match right_arg {
                    BinaryArg::Column(right_column) => Self::contains_column_arg(right_column),
                    _ => {
                        return Err(Error::NotYetSupportedError {
                            message: "contains with left literal and right non-column".into(),
                        });
                    }
                };

                let contains_expr = contains(left_expr, right_expr);
                Ok(match attrs {
                    None => FilterPlan::from(contains_expr),
                    Some((attrs_identifier, attrs_key)) => {
                        FilterPlan::from(AttributesFilterPlan::new(
                            Self::attr_key_equals(&attrs_key, attr_keys_case_sensitive)
                                .and(contains_expr),
                            attrs_identifier,
                        ))
                    }
                })
            }
            BinaryArg::Null => Err(Error::NotYetSupportedError {
                message: "contains with left literal null".into(),
            }),
        }
    }

    fn try_from_matches_expr(
        matches_expr: &MatchesLogicalExpression,
        attr_keys_case_sensitive: bool,
    ) -> Result<Self> {
        let left_arg = BinaryArg::try_from(matches_expr.get_haystack())?;
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

        match left_arg {
            BinaryArg::Column(left_column) => Ok(match left_column {
                ColumnAccessor::ColumnName(left_col_name) => FilterPlan::from(binary_expr(
                    col(left_col_name),
                    Operator::RegexMatch,
                    pattern,
                )),
                ColumnAccessor::StructCol(struct_name, struct_field) => {
                    FilterPlan::from(binary_expr(
                        col(struct_name).field(struct_field),
                        Operator::RegexMatch,
                        pattern,
                    ))
                }
                ColumnAccessor::Attributes(attrs_identifier, attr_key) => {
                    FilterPlan::from(AttributesFilterPlan::new(
                        Self::attr_key_equals(&attr_key, attr_keys_case_sensitive).and(
                            binary_expr(col(consts::ATTRIBUTE_STR), Operator::RegexMatch, pattern),
                        ),
                        attrs_identifier,
                    ))
                }
            }),
            BinaryArg::Literal(_) => Err(Error::NotYetSupportedError {
                message: "literal matches regex".into(),
            }),
            BinaryArg::Null => Err(Error::InvalidPipelineError {
                cause: "cannot match null against regex".into(),
                query_location: Some(matches_expr.get_query_location().clone()),
            }),
        }
    }

    /// transform the arguments for a binary filter expression into case-insensitive equals
    /// if the operator and the types would support it
    ///
    /// TODO: Currently this uses ILikeMatch to implement case insensitive equals. We'll eventually
    /// revisit this. There are two small issues with the current implementation:
    /// - In the future, we will probably support filtering where one side is not a static, so
    ///   we won't be able to statically determine that this is the appropriate operator
    /// - If the string literal to which we're comparing contains special characters used in
    ///   SQL Like expressions, such as "%", "_" or "\\", the underlying arrow_string kernel will
    ///   do the comparison using a case insensitive regex match (even if the characters are
    ///   escaped), and this is slower than it just calling eq_ignore_case.
    ///
    /// The better solution in the future is probably to implement our kernel for checking string
    /// equality while ignoring ascii case and embedding it in either a PhysicalExpr or ScalarUDF.
    /// However, this will be a lot easier to implement with some changes to arrow_string crate.
    ///
    fn transform_case_insensitive_equals(
        left_arg: &mut BinaryArg,
        binary_op: &mut Operator,
        right_arg: &mut BinaryArg,
    ) {
        if binary_op != &Operator::Eq {
            return;
        }

        if let BinaryArg::Literal(StaticScalarExpression::String(literal)) = left_arg {
            *binary_op = Operator::ILikeMatch;
            let literal_val = literal.get_value();
            if Self::contains_like_pattern(literal_val) {
                *literal = StringScalarExpression::new(
                    literal.get_query_location().clone(),
                    &Self::escape_like_pattern(literal_val),
                )
            }
        }

        if let BinaryArg::Literal(StaticScalarExpression::String(literal)) = right_arg {
            *binary_op = Operator::ILikeMatch;
            let literal_val = literal.get_value();
            if Self::contains_like_pattern(literal_val) {
                *literal = StringScalarExpression::new(
                    literal.get_query_location().clone(),
                    &Self::escape_like_pattern(literal_val),
                )
            }
        }
    }

    fn contains_like_pattern(pattern: &str) -> bool {
        memchr::memchr3(b'%', b'_', b'\\', pattern.as_bytes()).is_some()
    }

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

    /// Create the expression that checks the name of some attribute.
    ///
    /// If case sensitive, it simply checks the value in thea attribute key column is equal to
    /// the attribute key. When case insensitive, it uses ILikeMatch which will actually get
    /// evaluated using `str.equals_ignore_case` inside `arrow_string::predicate`
    fn attr_key_equals(attrs_key: &str, case_sensitive: bool) -> Expr {
        if case_sensitive {
            col(consts::ATTRIBUTE_KEY).eq(lit(attrs_key))
        } else {
            let rhs_expr = if Self::contains_like_pattern(attrs_key) {
                lit(Self::escape_like_pattern(attrs_key))
            } else {
                lit(attrs_key)
            };
            binary_expr(col(consts::ATTRIBUTE_KEY), Operator::ILikeMatch, rhs_expr)
        }
    }

    /// constructs the Expr that will get passed to the "contains" function for a column
    /// e.g. the first argument in an expression like contains(attributes["x"], "hello")
    ///
    /// The second return value contains which attribute we're passing to contains. This
    /// can be used by the caller to construct the appropriate logical expr for checking
    /// if attribute value contains some text
    fn contains_column_arg(
        column_accessor: ColumnAccessor,
    ) -> (Expr, Option<(AttributesIdentifier, String)>) {
        let mut attrs = None;
        let expr = match column_accessor {
            ColumnAccessor::ColumnName(col_name) => col(col_name),
            ColumnAccessor::StructCol(struct_name, struct_field) => {
                col(struct_name).field(struct_field)
            }
            ColumnAccessor::Attributes(attrs_identifier, attrs_key) => {
                attrs = Some((attrs_identifier, attrs_key));
                // for now we assume that text contains is always applied to the str column
                col(consts::ATTRIBUTE_STR)
            }
        };

        (expr, attrs)
    }
}

impl Composite<FilterPlan> {
    pub fn try_from(
        logical_expr: &LogicalExpression,
        attr_keys_case_sensitive: bool,
    ) -> Result<Self> {
        match logical_expr {
            LogicalExpression::EqualTo(equals_to_expr) => FilterPlan::try_from_binary_expr(
                equals_to_expr.get_left(),
                Operator::Eq,
                equals_to_expr.get_right(),
                !equals_to_expr.get_case_insensitive(),
                attr_keys_case_sensitive,
            )
            .map(|plan| plan.into()),
            LogicalExpression::GreaterThan(gt_expr) => FilterPlan::try_from_binary_expr(
                gt_expr.get_left(),
                Operator::Gt,
                gt_expr.get_right(),
                Default::default(),
                attr_keys_case_sensitive,
            )
            .map(|plan| plan.into()),
            LogicalExpression::GreaterThanOrEqualTo(geq_expr) => FilterPlan::try_from_binary_expr(
                geq_expr.get_left(),
                Operator::GtEq,
                geq_expr.get_right(),
                Default::default(),
                attr_keys_case_sensitive,
            )
            .map(|plan| plan.into()),
            LogicalExpression::And(and_expr) => {
                let left = Self::try_from(and_expr.get_left(), attr_keys_case_sensitive)?;
                let right = Self::try_from(and_expr.get_right(), attr_keys_case_sensitive)?;
                Ok(Self::and(left, right))
            }
            LogicalExpression::Or(or_expr) => {
                let left = Self::try_from(or_expr.get_left(), attr_keys_case_sensitive)?;
                let right = Self::try_from(or_expr.get_right(), attr_keys_case_sensitive)?;
                Ok(Self::or(left, right))
            }
            LogicalExpression::Not(not_expr) => {
                let inner =
                    Self::try_from(not_expr.get_inner_expression(), attr_keys_case_sensitive)?;
                Ok(Self::not(inner))
            }
            LogicalExpression::Contains(contains_expr) => Ok(Self::from(
                FilterPlan::try_from_contains_expr(contains_expr, attr_keys_case_sensitive)?,
            )),
            LogicalExpression::Matches(matches_expr) => Ok(Self::from(
                FilterPlan::try_from_matches_expr(matches_expr, attr_keys_case_sensitive)?,
            )),

            LogicalExpression::Scalar(scalar_expr) => match scalar_expr {
                ScalarExpression::Static(StaticScalarExpression::Boolean(bool)) => {
                    Ok(Self::from(FilterPlan::from(lit(bool.get_value()))))
                }
                // TODO add support for these expressions eventually
                _ => Err(Error::NotYetSupportedError {
                    message: format!("Logical expression not yet supported {logical_expr:?}"),
                }),
            },
        }
    }
}

impl ToExec for FilterPlan {
    type ExecutablePlan = FilterExec;
    fn to_exec(
        &self,
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Self::ExecutablePlan> {
        let physical_expr = self
            .source_filter
            .as_ref()
            .map(|expr| AdaptivePhysicalExprExec::try_new(expr.clone()))
            .transpose()?;

        let attrs_filter = self
            .attribute_filter
            .as_ref()
            .map(|attr_filter| attr_filter.to_exec(session_ctx, otap_batch))
            .transpose()?;

        // compute how to handle missing attributes. If the attrs filter is not(attr exists), then
        // if the id column null for some row (meaning no attributes), or if the ID column is
        // absent entirely (meaning now rows have attributes) then we treat the rows as it passes
        // the attribute filter because
        let missing_attrs_pass = matches!(
            &self.attribute_filter,
            Some(
                Composite::Not(filter)) if matches!(filter.as_ref(),
                Composite::Base(f) if f.checks_existence_only()
            )
        );

        Ok(FilterExec {
            predicate: physical_expr,
            attributes_filter: attrs_filter,
            missing_attrs_pass,
        })
    }
}

/// A logical plan for filtering attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributesFilterPlan {
    /// The filtering expression that  will be applied to the attributes
    ///
    /// Note that the expression should be constructed based on the otap data-model, not using some
    /// abstract notion of an attribute column. This means, say we're filtering for
    /// `attributes["x"] == "y"` that we'd expect a filter expr like
    /// `col("key").eq(lit("x")).and(col("str").eq(lit("y")))`
    pub filter: Expr,

    /// The identifier of which attributes will be considered when filtering.
    pub attrs_identifier: AttributesIdentifier,
}

impl AttributesFilterPlan {
    const fn new(filter: Expr, attrs_identifier: AttributesIdentifier) -> Self {
        Self {
            filter,
            attrs_identifier,
        }
    }

    /// returns true if the expression is only checking for the existence of some attribute versus
    /// checking that the attribute has some given value
    fn checks_existence_only(&self) -> bool {
        // inspect the pattern -- we're looking for col(key).eq(lit("attr name"))
        match &self.filter {
            Expr::BinaryExpr(binary_expr) if binary_expr.op == Operator::Eq => {
                let is_attr_column = matches!(
                    binary_expr.left.as_ref(),
                    Expr::Column(col) if col.name == consts::ATTRIBUTE_KEY
                );
                let is_string_literal = matches!(
                    binary_expr.right.as_ref(),
                    Expr::Literal(ScalarValue::Utf8(_), None)
                );
                is_attr_column && is_string_literal
            }
            _ => false,
        }
    }
}

impl ToExec for AttributesFilterPlan {
    type ExecutablePlan = AttributeFilterExec;

    fn to_exec(
        &self,
        _session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Self::ExecutablePlan> {
        let attrs_payload_type = match self.attrs_identifier {
            AttributesIdentifier::Root => match otap_batch.root_payload_type() {
                ArrowPayloadType::Logs => ArrowPayloadType::LogAttrs,
                ArrowPayloadType::Spans => ArrowPayloadType::SpanAttrs,
                _ => ArrowPayloadType::MetricAttrs,
            },
            AttributesIdentifier::NonRoot(payload_type) => payload_type,
        };

        Ok(AttributeFilterExec {
            filter: AdaptivePhysicalExprExec::try_new(self.filter.clone())?,
            payload_type: attrs_payload_type,
        })
    }
}

impl Composite<AttributesFilterPlan> {
    pub fn attrs_identifier(&self) -> AttributesIdentifier {
        match self {
            Self::Base(filter) => filter.attrs_identifier,
            Self::Not(filter) => filter.attrs_identifier(),

            // All children should be for the same payload type, so we just traverse one side
            // of the tree.
            Self::And(left, _) => left.attrs_identifier(),
            Self::Or(left, _) => left.attrs_identifier(),
        }
    }
}

fn to_physical_exprs(
    expr: &Expr,
    record_batch: &RecordBatch,
    session_ctx: &SessionContext,
) -> Result<PhysicalExprRef> {
    let df_schema = DFSchema::from_unqualified_fields(
        record_batch.schema().fields.clone(),
        Default::default(),
    )?;
    let physical_expr =
        create_physical_expr(expr, &df_schema, session_ctx.state().execution_props())?;

    Ok(physical_expr)
}

pub struct FilterExec {
    pub(crate) predicate: Option<AdaptivePhysicalExprExec>,

    attributes_filter: Option<Composite<AttributeFilterExec>>,

    /// determines how we treat rows that where there are no attributes. if false, this cause the
    /// row not to pass the filter, unless this is true which it should be set it as for filters/
    /// like `attributes["x"] == null`
    missing_attrs_pass: bool,
}

impl From<AdaptivePhysicalExprExec> for FilterExec {
    fn from(predicate: AdaptivePhysicalExprExec) -> Self {
        Self {
            predicate: Some(predicate),
            attributes_filter: None,
            missing_attrs_pass: false,
        }
    }
}

impl FilterExec {
    /// execute the filter expression. returns a selection vector for the passed record batch
    fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
        id_bitmap_pool: &mut IdBitmapPool,
    ) -> Result<BooleanArray> {
        let root_rb = match otap_batch.root_record_batch() {
            Some(rb) => rb,
            None => {
                // The caller should be responsible for checking if the batch is empty (e.g. if
                // there's no root batch), which means nothing could be filtered out so it should
                // not be calling this method. If we encounter this, return an error
                return Err(Error::ExecutionError {
                    cause: "root batch not present in execution".into(),
                });
            }
        };

        // evaluate predicate on the root batch
        let mut selection_vec = self
            .predicate
            .as_mut()
            .map(|predicate| predicate.evaluate_filter(root_rb, session_ctx))
            .transpose()?;

        // also apply any attribute filters
        if let Some(attrs_filter) = &mut self.attributes_filter {
            let id_col =
                match UInt16Type::get_id_col_from_parent(root_rb, attrs_filter.payload_type())? {
                    Some(MaybeDictArrayAccessor::Native(id_col)) => id_col,
                    Some(_) => {
                        // currently based on how `UInt16Type::get_id_col_from_parent` is
                        // implemented, this is actually unreachable, but putting the error
                        // here instead of unreachable! for posterity in case we change the impl
                        // in future for some reason
                        return Err(Error::ExecutionError {
                            cause: "invalid type for ID column on root batch".into(),
                        });
                    }
                    None => {
                        // None of the records have any attributes
                        return Ok(BooleanArray::new(
                            if self.missing_attrs_pass {
                                BooleanBuffer::new_set(root_rb.num_rows())
                            } else {
                                BooleanBuffer::new_unset(root_rb.num_rows())
                            },
                            None,
                        ));
                    }
                };

            let id_mask = attrs_filter.execute(otap_batch, session_ctx, false, id_bitmap_pool)?;
            let mut attrs_selection_vec_builder = BooleanBufferBuilder::new(root_rb.num_rows());

            // we append to the selection vector in contiguous segments rather than doing it 1-by-1
            // for each value, as this is a faster way to build up the BooleanBuffer
            let mut segment_validity = false;
            let mut segment_len = 0usize;

            for index in 0..id_col.len() {
                let row_validity = if id_col.is_valid(index) {
                    id_mask.contains(id_col.value(index) as u32)
                } else {
                    // attribute does not exist
                    self.missing_attrs_pass
                };

                if segment_validity != row_validity {
                    if segment_len > 0 {
                        attrs_selection_vec_builder.append_n(segment_len, segment_validity);
                    }
                    segment_validity = row_validity;
                    segment_len = 0;
                }

                segment_len += 1;
            }

            // append the last segment
            if segment_len > 0 {
                attrs_selection_vec_builder.append_n(segment_len, segment_validity);
            }

            // release the id_mask bitmap back to the pool for reuse
            id_mask.release_to(id_bitmap_pool);

            let attr_selection_vec = BooleanArray::new(attrs_selection_vec_builder.finish(), None);
            selection_vec = Some(match selection_vec {
                // update the result selection_vec to be the intersection of what's already filtered
                // and the attributes filters
                Some(selection_vec) => and(&selection_vec, &attr_selection_vec)?,

                // no predicate was applied to root batch, so we are just filtering by attributes
                None => attr_selection_vec,
            });
        }

        // if for some reason this filter was empty (would be unusual b/c we shouldn't be planning
        // filters like this), we just return a vec indicating that all rows passed the predicate
        let result = selection_vec.unwrap_or(BooleanArray::new(
            BooleanBuffer::new_set(root_rb.num_rows()),
            None,
        ));

        Ok(result)
    }
}

impl Composite<FilterExec> {
    pub(crate) fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
        id_bitmap_pool: &mut IdBitmapPool,
    ) -> Result<BooleanArray> {
        match self {
            Self::Base(filter) => filter.execute(otap_batch, session_ctx, id_bitmap_pool),
            Self::Not(filter) => Ok(not(&filter.execute(
                otap_batch,
                session_ctx,
                id_bitmap_pool,
            )?)?),
            Self::And(left, right) => {
                let left_result = left.execute(otap_batch, session_ctx, id_bitmap_pool)?;

                // short circuit if everything on the left was filtered out. No "true" value
                // in the right selection vector would change the result
                let num_rows = otap_batch
                    .root_record_batch()
                    .map(|batch| batch.num_rows())
                    .unwrap_or_default();
                if left_result.false_count() == num_rows {
                    return Ok(left_result);
                }

                let right_result = right.execute(otap_batch, session_ctx, id_bitmap_pool)?;
                Ok(and(&left_result, &right_result)?)
            }
            Self::Or(left, right) => {
                let left_result = left.execute(otap_batch, session_ctx, id_bitmap_pool)?;

                // short circuit if nothing on the left was filtered out. No "false" value
                // in the right selection vector would change the result
                let num_rows = otap_batch
                    .root_record_batch()
                    .map(|batch| batch.num_rows())
                    .unwrap_or_default();
                if left_result.true_count() == num_rows {
                    return Ok(left_result);
                }

                let right_result = right.execute(otap_batch, session_ctx, id_bitmap_pool)?;
                Ok(or(&left_result, &right_result)?)
            }
        }
    }
}

/// This represents which IDs have been selected by some filter operation.
///
/// For example it can be used as the return type from filtering attributes to represent
/// values from the parent_id column matched some filter that was applied.
#[derive(Debug, PartialEq)]
pub enum IdMask {
    // All IDs are selected
    All,

    /// None of the IDs are selected
    None,

    /// Some of the IDs are selected
    Some(IdBitmap),

    /// Some of the IDs are not selected
    NotSome(IdBitmap),
}

impl IdMask {
    fn contains(&self, id: u32) -> bool {
        match self {
            Self::All => true,
            Self::None => false,
            Self::Some(bitmap) => bitmap.contains(id),
            Self::NotSome(bitmap) => !bitmap.contains(id),
        }
    }

    /// Returns the owned bitmap (if any) to the pool for reuse.
    fn release_to(self, pool: &mut IdBitmapPool) {
        match self {
            Self::Some(bm) | Self::NotSome(bm) => pool.release(bm),
            Self::All | Self::None => {}
        }
    }

    /// Combines two masks with OR logic, returning spare bitmaps to the pool.
    fn combine_or(self, rhs: Self, pool: &mut IdBitmapPool) -> Self {
        match (self, rhs) {
            (Self::All, other) | (other, Self::All) => {
                other.release_to(pool);
                Self::All
            }
            (Self::None, other) | (other, Self::None) => other,

            (Self::Some(mut lhs), Self::Some(rhs)) => {
                lhs.union_with(&rhs);
                pool.release(rhs);
                Self::Some(lhs)
            }

            (Self::Some(lhs), Self::NotSome(mut rhs))
            | (Self::NotSome(mut rhs), Self::Some(lhs)) => {
                // Some(lhs) | NotSome(rhs) = Some(lhs) | !Some(rhs)
                // = everything except what's in rhs but not in lhs
                // = NotSome(rhs - lhs)
                rhs.difference_with(&lhs);
                pool.release(lhs);
                if rhs.is_empty() {
                    pool.release(rhs);
                    Self::All
                } else {
                    Self::NotSome(rhs)
                }
            }

            (Self::NotSome(mut lhs), Self::NotSome(rhs)) => {
                // NotSome(lhs) | NotSome(rhs) = !lhs | !rhs = !(lhs & rhs)
                lhs.intersect_with(&rhs);
                pool.release(rhs);
                Self::NotSome(lhs)
            }
        }
    }

    /// Combines two masks with AND logic, returning spare bitmaps to the pool.
    fn combine_and(self, rhs: Self, pool: &mut IdBitmapPool) -> Self {
        match (self, rhs) {
            (Self::None, other) | (other, Self::None) => {
                other.release_to(pool);
                Self::None
            }
            (Self::All, other) | (other, Self::All) => other,

            (Self::Some(mut lhs), Self::Some(rhs)) => {
                // Some(lhs) & Some(rhs) = intersection
                lhs.intersect_with(&rhs);
                pool.release(rhs);
                Self::Some(lhs)
            }

            (Self::Some(mut lhs), Self::NotSome(rhs))
            | (Self::NotSome(rhs), Self::Some(mut lhs)) => {
                // Some(lhs) & NotSome(rhs) = Some(lhs) & !Some(rhs)
                // = lhs minus rhs
                lhs.difference_with(&rhs);
                pool.release(rhs);
                if lhs.is_empty() {
                    pool.release(lhs);
                    Self::None
                } else {
                    Self::Some(lhs)
                }
            }

            (Self::NotSome(mut lhs), Self::NotSome(rhs)) => {
                // NotSome(lhs) & NotSome(rhs) = !lhs & !rhs = !(lhs | rhs)
                lhs.union_with(&rhs);
                pool.release(rhs);
                Self::NotSome(lhs)
            }
        }
    }
}

pub struct AttributeFilterExec {
    pub filter: AdaptivePhysicalExprExec,
    pub payload_type: ArrowPayloadType,
}

impl AttributeFilterExec {
    /// execute the filter on the attributes. This returns a bitmap of parent_ids that were
    /// selected by the filter. Conversely if invert = `true` it creates a bitmap of parent_ids
    /// not selected by the filter.
    fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
        inverted: bool,
        pool: &mut IdBitmapPool,
    ) -> Result<IdMask> {
        let record_batch = match otap_batch.get(self.payload_type) {
            Some(rb) => rb,
            None => {
                // if there are no attributes, then nothing can match the filter so just return
                // empty ID mask
                return Ok(IdMask::None);
            }
        };

        let selection_vec = self.filter.evaluate_filter(record_batch, session_ctx)?;

        // if no rows passed the filter, return early without mapping the results over the
        // parent_id column
        if selection_vec.false_count() == record_batch.num_rows() {
            return Ok(if inverted { IdMask::All } else { IdMask::None });
        }

        let parent_id_col = get_parent_id_column(record_batch)?;

        // create a bitmap containing the parent_ids that passed the filter predicate
        let mut id_bitmap = pool.acquire();
        id_bitmap.populate(
            parent_id_col
                .iter()
                .enumerate()
                .filter_map(|(index, parent_id)| {
                    selection_vec
                        .value(index)
                        .then(|| {
                            // the parent_id column _should_ be non-nullable, so we could maybe call
                            // `expect` here, but `map` is probably safer just in case there is a null
                            // for some unexpected reason
                            parent_id.map(|i| i as u32)
                        })
                        .flatten()
                }),
        );

        Ok(if inverted {
            IdMask::NotSome(id_bitmap)
        } else {
            IdMask::Some(id_bitmap)
        })
    }
}

impl Composite<AttributeFilterExec> {
    /// Executes the base filter, and combines the the parent_id to using the logical expression
    /// defined by the composite tree. The reason we do here, instead of say combining everything
    /// in `Composite<FilterExec>`, is that this saves us from doing additional conversions between
    /// the parent_id bitmap to a selection vector for the parent record batch.
    fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
        inverted: bool,
        pool: &mut IdBitmapPool,
    ) -> Result<IdMask> {
        match self {
            Self::Base(filter) => filter.execute(otap_batch, session_ctx, inverted, pool),
            Self::Not(filter) => filter.execute(otap_batch, session_ctx, !inverted, pool),
            Self::And(left, right) => {
                let left_result = left.execute(otap_batch, session_ctx, inverted, pool)?;

                // short circuit evaluating the other side if possible. if nothing passed the
                // filter, we don't need to evaluate it because it won't change the result
                if (!inverted && left_result == IdMask::None)
                    || (inverted && left_result == IdMask::All)
                {
                    return Ok(left_result);
                }

                let right_result = right.execute(otap_batch, session_ctx, inverted, pool)?;
                Ok(if inverted {
                    // not (A and B) = (not A) or (not B)
                    left_result.combine_or(right_result, pool)
                } else {
                    left_result.combine_and(right_result, pool)
                })
            }
            Self::Or(left, right) => {
                let left_result = left.execute(otap_batch, session_ctx, inverted, pool)?;
                let right_result = right.execute(otap_batch, session_ctx, inverted, pool)?;
                Ok(if inverted {
                    // not (A or B) = (not A) and (not B)
                    left_result.combine_and(right_result, pool)
                } else {
                    left_result.combine_or(right_result, pool)
                })
            }
        }
    }

    fn payload_type(&self) -> ArrowPayloadType {
        match self {
            Self::Base(filter) => filter.payload_type,
            Self::Not(filter) => filter.payload_type(),

            // All children should be for the same payload type, so we just traverse one side
            // of the tree.
            Self::And(left, _) => left.payload_type(),
            Self::Or(left, _) => left.payload_type(),
        }
    }
}

/// This is responsible for evaluating a  [`PhysicalExpr`](datafusion::physical_expr::PhysicalExpr)
/// while adapting to schema changes that may be encountered between evaluations.
///
/// A given payload type's [`RecordBatch`] might have minor changes the schema between batches
/// - The type of a column may change between Dict<u8, V>, Dict<16, V>, and the native array type
/// - The order of columns may change
///
pub struct AdaptivePhysicalExprExec {
    /// The physical expression that should be evaluated for each batch. This is initialized lazily
    /// when the first non-`None` batch is passed to `evaluate`. This should evaluate to a boolean
    physical_expr: Option<PhysicalExprRef>,

    /// The original logical plan used to produce the [`PhysicalExpr`]
    logical_expr: Expr,

    /// Definition for how the input record batch should be projected so that it's schema is
    /// compatible with what is expected by the physical_expr
    projection: Projection,

    /// Determines the behaviour of the predicate when some columns are missing from the batch.
    ///
    /// When a column is missing, it is implied that all the values are null or default value.
    /// Normally this means that all the rows would fail the predicate, except for a few cases
    /// like `<some_col> == null`
    missing_data_passes: bool,
}

impl AdaptivePhysicalExprExec {
    fn try_new(logical_expr: Expr) -> Result<Self> {
        let projection = Projection::try_new(&logical_expr)?;

        // TODO eventually we may want more sophisticated logic here to handle when the column
        // is a default value. Cases like `dropped_attribute_count == 0`, in this case should
        // also pass the filter if the `dropped_attribute_count` column is missing b/c the column
        // could contain all 0s.
        let missing_data_passes = matches!(logical_expr, Expr::IsNull(_));

        Ok(Self {
            physical_expr: None,
            logical_expr,
            projection,
            missing_data_passes,
        })
    }

    /// Evaluates the [`PhysicalExpr`] for the passed record batch and returns a selection
    /// vector for the rows that pass the predicate.
    pub(crate) fn evaluate_filter(
        &mut self,
        record_batch: &RecordBatch,
        session_ctx: &SessionContext,
    ) -> Result<BooleanArray> {
        let record_batch = match self.projection.project(record_batch)? {
            Some(rb) => rb,
            None => {
                // we weren't able to project the record batch into the schema expected by the
                // physical expr. This means that there were some columns referenced in the logical
                // expr that are missing from the input batch.
                return Ok(BooleanArray::new(
                    if self.missing_data_passes {
                        BooleanBuffer::new_set(record_batch.num_rows())
                    } else {
                        BooleanBuffer::new_unset(record_batch.num_rows())
                    },
                    None,
                ));
            }
        };

        // lazily initialize the physical expr if not already initialized
        if self.physical_expr.is_none() {
            let physical_expr = to_physical_exprs(&self.logical_expr, &record_batch, session_ctx)?;
            self.physical_expr = Some(physical_expr);
        }

        // safety: this is already initialized
        let predicate = self.physical_expr.as_ref().expect("initialized");

        // evaluate the predicate
        let arr = predicate
            .evaluate(&record_batch)?
            .into_array(record_batch.num_rows())?;

        // ensure it actually evaluated to a boolean expression, and if so return that as selection vec
        let boolean_arr = as_boolean_array(&arr)
            .cloned()
            .map_err(|_| Error::ExecutionError {
                cause: format!(
                    "Cannot create selection vector from non-boolean predicates. Found {}",
                    arr.data_type()
                ),
            })?;

        // the underlying compute kernels that will be used for the physical exprs will produce
        // nulls if the incoming value is null. For the expressions we support, we want this to
        // be `false`, so the null buffer must be removed
        let (result_bool_values, null_buffer) = boolean_arr.into_parts();
        let boolean_arr = match null_buffer {
            // no nulls, just return the selection vec as is:
            None => BooleanArray::new(result_bool_values, None),

            // combine nulls into selection vec:
            Some(null_buffer) => {
                // Note: arrow-rs doesn't make any guarantees that the null values in a boolean
                // array are `false` in the values buffer so we can't simply drop the null buffer
                // in the case null values don't pass the filter.
                let mut null_mask = BooleanArray::new(null_buffer.into_inner(), None);
                if self.missing_data_passes {
                    null_mask = not(&null_mask)?;
                }
                and(&BooleanArray::new(result_bool_values, None), &null_mask)?
            }
        };

        Ok(boolean_arr)
    }
}

// filter_otap_batch and filter_child_batch are now provided by
// otap_df_pdata::otap::filter. Re-exported for use by other modules in this
// crate (e.g. conditional.rs).
pub(crate) use otap_df_pdata::otap::filter::filter_otap_batch;

// ChildBatchFilterIdHelper trait and impls are provided by otap_df_pdata::otap::filter.

fn get_parent_id_column(record_batch: &RecordBatch) -> Result<&UInt16Array> {
    // get the parent ID column
    let parent_id_arr = record_batch
        .column_by_name(consts::PARENT_ID)
        .ok_or_else(|| Error::ExecutionError {
            cause: "parent_id column not found on child batch".into(),
        })?;

    let parent_id_ints = parent_id_arr
        .as_any()
        .downcast_ref::<UInt16Array>()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!(
                "unexpected parent_id type for child record batch. Expected u16 found {:?}",
                parent_id_arr.data_type()
            ),
        })?;

    Ok(parent_id_ints)
}

pub struct FilterPipelineStage {
    filter_exec: Composite<FilterExec>,
    id_bitmap_pool: IdBitmapPool,
}

impl FilterPipelineStage {
    pub fn new(filter_exec: Composite<FilterExec>) -> Self {
        Self {
            filter_exec,
            id_bitmap_pool: IdBitmapPool::new(),
        }
    }
}

// filter_otap_batch and filter_child_batch are now in otap_df_pdata::otap::filter.

#[async_trait(?Send)]
impl PipelineStage for FilterPipelineStage {
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        if otap_batch.root_record_batch().is_none() {
            // if batch is empty, no filtering to do
            return Ok(otap_batch);
        }

        let selection_vec =
            self.filter_exec
                .execute(&otap_batch, session_context, &mut self.id_bitmap_pool)?;
        let otap_batch = filter_otap_batch(&selection_vec, &otap_batch, &mut self.id_bitmap_pool)?;

        Ok(otap_batch)
    }

    async fn execute_on_attributes(
        &mut self,
        attrs_record_batch: RecordBatch,
        session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_options: &mut ExecutionState,
    ) -> Result<RecordBatch> {
        let planning_error = || {
            // we shouldn't end up here, unless there was a bug in the planner and it didn't call
            // the correct optimizers on the FilterPlan to turn it into something that can operate
            // directly on the attrs record batch
            Error::InvalidPipelineError {
                cause: "invalid filter plan variant. This pipeline stage was not optimized for attribute filtering".into(),
                query_location: None,
            }
        };

        match &mut self.filter_exec {
            Composite::Base(filter) => {
                let predicate = filter.predicate.as_mut().ok_or_else(planning_error)?;
                let selection_vec =
                    predicate.evaluate_filter(&attrs_record_batch, session_context)?;
                let new_batch = filter_record_batch(&attrs_record_batch, &selection_vec)?;

                Ok(new_batch)
            }
            _ => Err(planning_error()),
        }
    }

    fn supports_exec_on_attributes(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use crate::pipeline::{Pipeline, PipelineOptions};

    use super::*;

    use arrow::array::{
        DictionaryArray, Int32Array, NullBufferBuilder, OffsetBufferBuilder, RecordBatch,
        StringArray, UInt8Array, UInt16Array,
    };
    use arrow::buffer::MutableBuffer;
    use arrow::datatypes::{DataType, Field, Schema};

    /// Test helper to build an IdBitmap from a slice of u32 values.
    fn id_bitmap_from(ids: &[u32]) -> IdBitmap {
        let mut bm = IdBitmap::new();
        for &id in ids {
            bm.insert(id);
        }
        bm
    }
    use data_engine_kql_parser::{KqlParser, Parser};
    use datafusion::physical_plan::PhysicalExpr;
    use otap_df_opl::parser::OplParser;
    use otap_df_pdata::otap::Logs;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otap_df_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
        HistogramDataPoint, Metric, NumberDataPoint, Summary, SummaryDataPoint,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::proto::opentelemetry::trace::v1::span::{Event, Link};
    use otap_df_pdata::proto::opentelemetry::trace::v1::{Span, Status};
    use otap_df_pdata::testing::round_trip::{
        otap_to_otlp, otlp_to_otap, to_logs_data, to_otap_logs, to_otap_metrics, to_otap_traces,
    };

    use crate::pipeline::test::{
        exec_logs_pipeline, otap_to_logs_data, otap_to_metrics_data, otap_to_traces_data,
    };

    async fn test_simple_filter<P: Parser>() {
        let ns_per_second: u64 = 1000 * 1000 * 1000;
        let log_records = vec![
            LogRecord::build()
                .severity_text("TRACE")
                .severity_number(1)
                .time_unix_nano(ns_per_second)
                .event_name("1")
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .severity_number(9)
                .event_name("2")
                .time_unix_nano(2 * ns_per_second)
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .severity_number(17)
                .time_unix_nano(3 * ns_per_second)
                .event_name("3")
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            "logs | where severity_text == \"ERROR\"",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );

        // test same filter where the literal is on the left and column name on the right
        let result = exec_logs_pipeline::<P>(
            "logs | where \"ERROR\" == severity_text",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );

        // test filtering by some other field types (u32, int32, timestamp)
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_number == 17",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_number == 17",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );

        let result = exec_logs_pipeline::<P>(
            "logs | where time_unix_nano > datetime(1970-01-01 00:00:01.1)",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()]
        );

        let result = exec_logs_pipeline::<P>(
            "logs | where datetime(1970-01-01 00:00:01.1) > time_unix_nano",
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()]
        );

        let result =
            exec_logs_pipeline::<P>("logs | where true", to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &log_records
        );

        // assert everything filtered out:
        let result =
            exec_logs_pipeline::<P>("logs | where false", to_logs_data(log_records.clone())).await;
        assert_eq!(result.resource_logs.len(), 0);
    }

    #[tokio::test]
    async fn test_simple_filter_kql_parser() {
        test_simple_filter::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_simple_filter_op_parser() {
        test_simple_filter::<OplParser>().await
    }

    async fn test_simple_attrs_filter<P: Parser>() {
        let otap_batch = to_otap_logs(vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                .event_name("2")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("c"))])
                .event_name("3")
                .finish(),
        ]);

        let expected = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                .event_name("2")
                .finish(),
        ];

        let parser_result = P::parse("logs | where attributes[\"x\"] == \"b\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &expected,
        );

        // test same filter where the literal is on the left and the attribute is on the right
        let parser_result = P::parse("logs | where \"b\" == attributes[\"x\"]").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &expected,
        )
    }

    #[tokio::test]
    async fn test_simple_attrs_filter_kql_parser() {
        test_simple_attrs_filter::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_simple_attrs_filter_opl_parser() {
        test_simple_attrs_filter::<OplParser>().await;
    }

    async fn test_filter_text_contains<P: Parser>(
        q_event_name_contains_error: &str,
        q_1234_contains_event_name: &str,
        q_attrs_username_contains_y: &str,
        q_albert_contains_attrs_username: &str,
    ) {
        let log_records = vec![
            LogRecord::build()
                .event_name("error happen")
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("bert"),
                )])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("username", AnyValue::new_string("tim"))])
                .event_name("the error was caught")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("terry"),
                )])
                .event_name("3")
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            q_event_name_contains_error,
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );

        // check we could specify the column on the right
        let result = exec_logs_pipeline::<P>(
            q_1234_contains_event_name,
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()]
        );

        // also check we can filter by attributes using contains
        let result = exec_logs_pipeline::<P>(
            q_attrs_username_contains_y,
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        // check that we could also specify the column on the right for attributes
        let result = exec_logs_pipeline::<P>(
            q_albert_contains_attrs_username,
            to_logs_data(log_records.clone()),
        )
        .await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_text_contains_kql() {
        test_filter_text_contains::<KqlParser>(
            r#"logs | where event_name contains "error""#,
            r#"logs | where "1234" contains event_name"#,
            r#"logs | where attributes["username"] contains "y""#,
            r#"logs | where "albert" contains attributes["username"]"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_text_contains_opl() {
        test_filter_text_contains::<OplParser>(
            r#"logs | where contains(event_name, "error")"#,
            r#"logs | where contains("1234", event_name)"#,
            r#"logs | where contains(attributes["username"], "y")"#,
            r#"logs | where contains("albert", attributes["username"])"#,
        )
        .await;
    }

    async fn test_filter_text_contains_struct_cols<P: Parser>(q1: &str, q2: &str) {
        let input = LogsData {
            resource_logs: vec![
                ResourceLogs {
                    schema_url: "version1".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("a"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                        ..Default::default()
                    }],
                },
                ResourceLogs {
                    schema_url: "experimental".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("b"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r2.e1").finish()],
                        ..Default::default()
                    }],
                },
            ],
        };

        let result = exec_logs_pipeline::<P>(q1, input.clone()).await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );

        // test same as above, but with literal contains the column value
        let result = exec_logs_pipeline::<P>(q2, input.clone()).await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[1].clone()],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_text_contains_struct_cols_kql() {
        test_filter_text_contains_struct_cols::<KqlParser>(
            r#"logs | where resource.schema_url contains "version""#,
            r#"logs | where "experimental version" contains resource.schema_url"#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_text_contains_struct_cols_opl() {
        test_filter_text_contains_struct_cols::<OplParser>(
            r#"logs | where contains(resource.schema_url, "version")"#,
            r#"logs | where contains("experimental version", resource.schema_url)"#,
        )
        .await;
    }

    async fn test_filter_matches_regex<P: Parser>(q1: &str, q2: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("error happen")
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("bert"),
                )])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("username", AnyValue::new_string("tim"))])
                .event_name("the error was caught")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "username",
                    AnyValue::new_string("terry"),
                )])
                .event_name("3")
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(q1, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()]
        );

        // also check we can filter by attributes using matches/regex
        let result = exec_logs_pipeline::<P>(q2, to_logs_data(log_records.clone())).await;
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_matches_regex_kql() {
        test_filter_matches_regex::<KqlParser>(
            r#"logs | where event_name matches regex "^err.*""#,
            r#"logs | where attributes["username"] matches regex "^t.*""#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_matches_regex_opl() {
        test_filter_matches_regex::<OplParser>(
            r#"logs | where matches(event_name, "^err.*")"#,
            r#"logs | where matches(attributes["username"], "^t.*")"#,
        )
        .await;
    }

    async fn test_filter_text_matches_regex_struct_cols<P: Parser>(q1: &str) {
        let input = LogsData {
            resource_logs: vec![
                ResourceLogs {
                    schema_url: "version1".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("a"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                        ..Default::default()
                    }],
                },
                ResourceLogs {
                    schema_url: "experimental".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("b"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r2.e1").finish()],
                        ..Default::default()
                    }],
                },
            ],
        };

        let result = exec_logs_pipeline::<P>(q1, input.clone()).await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_text_matches_regex_struct_cols_kql() {
        test_filter_text_matches_regex_struct_cols::<KqlParser>(
            r#"logs | where resource.schema_url matches regex "v.*1""#,
        )
        .await;
    }

    #[tokio::test]
    async fn test_filter_text_matches_regex_struct_cols_opl() {
        test_filter_text_matches_regex_struct_cols::<OplParser>(
            r#"logs | where matches(resource.schema_url, "v.*1")"#,
        )
        .await;
    }

    async fn test_filter_by_resources<P: Parser>() {
        let input = LogsData {
            resource_logs: vec![
                ResourceLogs {
                    schema_url: "schema1".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("a"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                        ..Default::default()
                    }],
                },
                ResourceLogs {
                    schema_url: "schema2".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("b"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r2.e1").finish()],
                        ..Default::default()
                    }],
                },
            ],
        };

        // test filter by resource properties
        let result = exec_logs_pipeline::<P>(
            "logs | where resource.schema_url == \"schema1\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );

        // test same as above, but with the literal on the right
        let result = exec_logs_pipeline::<P>(
            "logs | where \"schema2\" == resource.schema_url",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[1].clone()],
            }
        );

        // test filter by resource attributes
        let result = exec_logs_pipeline::<P>(
            "logs | where resource.attributes[\"x\"] == \"a\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_by_resources_kql_parser() {
        test_filter_by_resources::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_by_resources_opl_parser() {
        test_filter_by_resources::<OplParser>().await;
    }

    async fn test_simple_filter_traces<P: Parser>() {
        let spans = vec![
            Span::build()
                .name("span1")
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                .events(vec![
                    Event::build()
                        .name("event1.1")
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .finish(),
                ])
                .links(vec![
                    Link::build()
                        .trace_id(vec![11; 16])
                        .span_id(vec![11; 8])
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .finish(),
                ])
                .finish(),
            Span::build()
                .name("span2")
                .trace_id(vec![2; 16])
                .span_id(vec![2; 8])
                .status(Status::default())
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                .events(vec![
                    Event::build()
                        .name("event2.1")
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Event::build()
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .name("event2.2")
                        .finish(),
                ])
                .links(vec![
                    Link::build()
                        .trace_id(vec![21; 16])
                        .span_id(vec![21; 8])
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Link::build()
                        .trace_id(vec![22; 16])
                        .span_id(vec![22; 8])
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .finish(),
                ])
                .finish(),
            Span::build()
                .name("span3")
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val3"))])
                .events(vec![
                    Event::build()
                        .name("event3.1")
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Event::build().name("event3.2").finish(),
                    Event::build().name("event3.2").finish(),
                ])
                .links(vec![
                    Link::build()
                        .trace_id(vec![31; 16])
                        .span_id(vec![31; 8])
                        .finish(),
                    Link::build()
                        .trace_id(vec![32; 16])
                        .span_id(vec![32; 8])
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Link::build()
                        .trace_id(vec![33; 16])
                        .span_id(vec![33; 8])
                        .finish(),
                ])
                .finish(),
        ];

        let input = to_otap_traces(spans.clone());
        let parser_result = P::parse("traces | where name == \"span2\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        // assert everything got filtered to the right size
        let result_spans = result.get(ArrowPayloadType::Spans).unwrap();
        assert_eq!(result_spans.num_rows(), 1);

        let span_attrs = result.get(ArrowPayloadType::SpanAttrs).unwrap();
        assert_eq!(span_attrs.num_rows(), 1);

        let span_events = result.get(ArrowPayloadType::SpanEvents).unwrap();
        assert_eq!(span_events.num_rows(), 2);

        let span_links = result.get(ArrowPayloadType::SpanLinks).unwrap();
        assert_eq!(span_links.num_rows(), 2);

        let span_link_attrs = result.get(ArrowPayloadType::SpanLinkAttrs).unwrap();
        assert_eq!(span_link_attrs.num_rows(), 3);

        let span_event_attrs = result.get(ArrowPayloadType::SpanEventAttrs).unwrap();
        assert_eq!(span_event_attrs.num_rows(), 3);

        let traces_data = otap_to_traces_data(result);
        assert_eq!(traces_data.resource_spans.len(), 1);
        assert_eq!(traces_data.resource_spans[0].scope_spans.len(), 1);
        pretty_assertions::assert_eq!(
            &traces_data.resource_spans[0].scope_spans[0].spans,
            &[spans[1].clone()]
        )
    }

    #[tokio::test]
    async fn test_simple_filter_traces_kql_parser() {
        test_simple_filter_traces::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_simple_filter_traces_opl_parser() {
        test_simple_filter_traces::<OplParser>().await;
    }

    async fn test_filter_traces_by_attrs<P: Parser>() {
        let spans = vec![
            Span::build()
                .name("span1")
                .trace_id(vec![1; 16])
                .span_id(vec![1; 8])
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                .status(Status::default())
                .finish(),
            Span::build()
                .name("span2")
                .trace_id(vec![2; 16])
                .span_id(vec![2; 8])
                .status(Status::default())
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let input = to_otap_traces(spans.clone());
        let parser_result = P::parse("traces | where attributes[\"key\"] == \"val2\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        let traces_data = otap_to_traces_data(result);
        assert_eq!(traces_data.resource_spans.len(), 1);
        assert_eq!(traces_data.resource_spans[0].scope_spans.len(), 1);
        pretty_assertions::assert_eq!(
            &traces_data.resource_spans[0].scope_spans[0].spans,
            &[spans[1].clone()]
        )
    }

    #[tokio::test]
    async fn test_filter_traces_by_attrs_kql_parser() {
        test_filter_traces_by_attrs::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_traces_by_attrs_opl_parser() {
        test_filter_traces_by_attrs::<OplParser>().await;
    }

    async fn test_simple_filter_metrics<P: Parser>() {
        let metrics = vec![
            Metric::build()
                .name("metric1")
                .data_gauge(Gauge {
                    data_points: vec![
                        NumberDataPoint::build()
                            .time_unix_nano(1000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(100u64)
                                    .trace_id(vec![1; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_gauge(Gauge {
                    data_points: vec![
                        NumberDataPoint::build()
                            .time_unix_nano(2000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(200u64)
                                    .trace_id(vec![2; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric1")
                .data_histogram(Histogram {
                    data_points: vec![
                        HistogramDataPoint::build()
                            .time_unix_nano(3000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(300u64)
                                    .trace_id(vec![1; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                    aggregation_temporality: 0,
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_histogram(Histogram {
                    data_points: vec![
                        HistogramDataPoint::build()
                            .time_unix_nano(4000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(400u64)
                                    .trace_id(vec![2; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                    aggregation_temporality: 0,
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric1")
                .data_exponential_histogram(ExponentialHistogram {
                    data_points: vec![
                        ExponentialHistogramDataPoint::build()
                            .time_unix_nano(5000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(500u64)
                                    .trace_id(vec![1; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val1"),
                                    )])
                                    .finish(),
                            ])
                            .positive(Buckets::default())
                            .negative(Buckets::default())
                            .finish(),
                    ],
                    aggregation_temporality: 0,
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_exponential_histogram(ExponentialHistogram {
                    data_points: vec![
                        ExponentialHistogramDataPoint::build()
                            .time_unix_nano(6000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .exemplars(vec![
                                Exemplar::build()
                                    .time_unix_nano(600u64)
                                    .trace_id(vec![2; 16])
                                    .filtered_attributes(vec![KeyValue::new(
                                        "key",
                                        AnyValue::new_string("val2"),
                                    )])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                    aggregation_temporality: 0,
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric1")
                .data_summary(Summary {
                    data_points: vec![
                        SummaryDataPoint::build()
                            .time_unix_nano(7000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .finish(),
                    ],
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_summary(Summary {
                    data_points: vec![
                        SummaryDataPoint::build()
                            .time_unix_nano(8000u64)
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .finish(),
                    ],
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                .finish(),
        ];

        let input = to_otap_metrics(metrics.clone());
        let parser_result = P::parse("metrics | where name == \"metric1\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        // assert everything got filtered to the right size
        let result_metrics = result.get(ArrowPayloadType::UnivariateMetrics).unwrap();
        assert_eq!(result_metrics.num_rows(), 4);

        let attrs = result.get(ArrowPayloadType::MetricAttrs).unwrap();
        assert_eq!(attrs.num_rows(), 4);

        let number_dps = result.get(ArrowPayloadType::NumberDataPoints).unwrap();
        assert_eq!(number_dps.num_rows(), 1);

        let number_dp_attrs = result.get(ArrowPayloadType::NumberDpAttrs).unwrap();
        assert_eq!(number_dp_attrs.num_rows(), 1);

        let number_dp_exemplars = result.get(ArrowPayloadType::NumberDpExemplars).unwrap();
        assert_eq!(number_dp_exemplars.num_rows(), 1);

        let number_dp_exemplar_attrs = result.get(ArrowPayloadType::NumberDpExemplarAttrs).unwrap();
        assert_eq!(number_dp_exemplar_attrs.num_rows(), 1);

        let hist_dps = result.get(ArrowPayloadType::HistogramDataPoints).unwrap();
        assert_eq!(hist_dps.num_rows(), 1);

        let hist_dp_attrs = result.get(ArrowPayloadType::HistogramDpAttrs).unwrap();
        assert_eq!(hist_dp_attrs.num_rows(), 1);

        let hist_dp_exemplars = result.get(ArrowPayloadType::HistogramDpExemplars).unwrap();
        assert_eq!(hist_dp_exemplars.num_rows(), 1);

        let hist_dp_exemplar_attrs = result
            .get(ArrowPayloadType::HistogramDpExemplarAttrs)
            .unwrap();
        assert_eq!(hist_dp_exemplar_attrs.num_rows(), 1);

        let exp_hist_dps = result
            .get(ArrowPayloadType::ExpHistogramDataPoints)
            .unwrap();
        assert_eq!(exp_hist_dps.num_rows(), 1);

        let exp_hist_dp_attrs = result.get(ArrowPayloadType::ExpHistogramDpAttrs).unwrap();
        assert_eq!(exp_hist_dp_attrs.num_rows(), 1);

        let exp_hist_dp_exemplars = result
            .get(ArrowPayloadType::ExpHistogramDpExemplars)
            .unwrap();
        assert_eq!(exp_hist_dp_exemplars.num_rows(), 1);

        let exp_hist_dp_exemplar_attrs = result
            .get(ArrowPayloadType::ExpHistogramDpExemplarAttrs)
            .unwrap();
        assert_eq!(exp_hist_dp_exemplar_attrs.num_rows(), 1);

        let summary_dps = result.get(ArrowPayloadType::SummaryDataPoints).unwrap();
        assert_eq!(summary_dps.num_rows(), 1);

        let summary_dp_attrs = result.get(ArrowPayloadType::SummaryDpAttrs).unwrap();
        assert_eq!(summary_dp_attrs.num_rows(), 1);

        let metrics_data = otap_to_metrics_data(result);
        assert_eq!(metrics_data.resource_metrics.len(), 1);
        assert_eq!(metrics_data.resource_metrics[0].scope_metrics.len(), 1);
        pretty_assertions::assert_eq!(
            &metrics_data.resource_metrics[0].scope_metrics[0].metrics,
            &[
                metrics[0].clone(),
                metrics[2].clone(),
                metrics[4].clone(),
                metrics[6].clone()
            ]
        )
    }

    #[tokio::test]
    async fn test_simple_filter_metrics_kql_parser() {
        test_simple_filter_metrics::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_simple_filter_metrics_opl_parser() {
        test_simple_filter_metrics::<OplParser>().await;
    }

    async fn test_filter_metrics_by_attrs<P: Parser>() {
        let metrics = vec![
            Metric::build()
                .name("metric1")
                .data_gauge(Gauge {
                    data_points: Vec::default(),
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                .finish(),
            Metric::build()
                .name("metric2")
                .data_gauge(Gauge {
                    data_points: Vec::default(),
                })
                .metadata(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let input = to_otap_metrics(metrics.clone());
        let parser_result = P::parse("metrics | where attributes[\"key\"] == \"val1\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        let metrics_data = otap_to_metrics_data(result);
        assert_eq!(metrics_data.resource_metrics.len(), 1);
        assert_eq!(metrics_data.resource_metrics[0].scope_metrics.len(), 1);
        pretty_assertions::assert_eq!(
            &metrics_data.resource_metrics[0].scope_metrics[0].metrics,
            &[metrics[0].clone(),]
        )
    }

    #[tokio::test]
    async fn test_filter_metrics_by_attrs_kql_parser() {
        test_filter_metrics_by_attrs::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_metrics_by_attrs_opl_parser() {
        test_filter_metrics_by_attrs::<OplParser>().await;
    }

    async fn test_removes_child_record_batch_if_parent_fully_filtered_out<P: Parser>() {
        let spans = vec![
            Span::build()
                .name("span1")
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                .finish(),
            Span::build()
                .name("span2")
                .trace_id(vec![2; 16])
                .span_id(vec![2; 8])
                .status(Status::default())
                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                .events(vec![
                    Event::build()
                        .name("event2.1")
                        .attributes(vec![
                            KeyValue::new("key2", AnyValue::new_string("val2")),
                            KeyValue::new("key3", AnyValue::new_string("val2")),
                        ])
                        .finish(),
                    Event::build()
                        .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val2"))])
                        .name("event2.2")
                        .finish(),
                ])
                .finish(),
        ];

        let input = to_otap_traces(spans.clone());
        let parser_result = P::parse("traces | where name == \"span1\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input).await.unwrap();

        // since we've filtered for span1, which has no events, the event and event attrs batches
        // should no longer be present
        assert!(result.get(ArrowPayloadType::SpanEvents).is_none());
        assert!(result.get(ArrowPayloadType::SpanEventAttrs).is_none())
    }

    #[tokio::test]
    async fn test_removes_child_record_batch_if_parent_fully_filtered_out_kql_parser() {
        test_removes_child_record_batch_if_parent_fully_filtered_out::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_removes_child_record_batch_if_parent_fully_filtered_out_opl_parser() {
        test_removes_child_record_batch_if_parent_fully_filtered_out::<OplParser>().await;
    }

    async fn test_filter_by_scope<P: Parser>() {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name2")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        // test filter by resource properties
        let result = exec_logs_pipeline::<P>(
            "logs | where instrumentation_scope.name == \"name1\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );

        // test same as above, but with the literal on the right
        let result = exec_logs_pipeline::<P>(
            "logs | where \"name2\" == instrumentation_scope.name",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[1].clone()],
                    ..Default::default()
                }],
            }
        );

        // test filter by resource attributes
        let result = exec_logs_pipeline::<P>(
            "logs | where instrumentation_scope.attributes[\"x\"] == \"a\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_by_scope_kql_parser() {
        test_filter_by_scope::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_by_scope_opl_parser() {
        test_filter_by_scope::<OplParser>().await;
    }

    async fn test_filter_with_and<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];
        let otap_batch = to_otap_logs(log_records.clone());

        // check simple filter "and" properties
        let parser_result =
            P::parse("logs | where severity_text == \"ERROR\" and event_name == \"2\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );

        // check simple filter "and" with mixed attributes and properties
        let parser_result =
            P::parse("logs | where severity_text == \"ERROR\" and attributes[\"x\"] == \"c\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        // check simple filter "and" two attributes
        let parser_result =
            P::parse("logs | where attributes[\"y\"] == \"d\" and attributes[\"x\"] == \"a\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_and_kql_parser() {
        test_filter_with_and::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_with_and_opl_parser() {
        test_filter_with_and::<OplParser>().await;
    }

    async fn test_filter_with_or<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];
        let otap_batch = to_otap_logs(log_records.clone());

        // check simple filter "or" with properties predicates
        let parser_result =
            P::parse("logs | where severity_text == \"INFO\" or severity_text == \"ERROR\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );

        // check simple filter "or" with mixed attributes/properties predicates
        let parser_result =
            P::parse("logs | where severity_text == \"ERROR\" or attributes[\"x\"] == \"c\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check simple filter "or" two attributes predicates
        let parser_result =
            P::parse("logs | where attributes[\"x\"] == \"a\" or attributes[\"y\"] == \"e\"")
                .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_or_kql_parser() {
        test_filter_with_or::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_with_or_opl_parser() {
        test_filter_with_or::<OplParser>().await;
    }

    async fn test_filter_with_not<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // check simple filter "not" with properties predicate
        let result = exec_logs_pipeline::<P>(
            "logs | where not(severity_text == \"INFO\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check simple filter "not" with attributes predicate
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"b\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_not_kql_parser() {
        test_filter_with_not::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_with_not_opl_parser() {
        test_filter_with_not::<OplParser>().await;
    }

    async fn test_filter_not_and<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // check simple inverted "and" filter with properties predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(severity_text == \"INFO\" and event_name == \"1\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check simple inverted "and" filter with attributes predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"b\" and attributes[\"y\"] == \"e\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );

        // check simple inverted "and" filter with mixed attributes & properties predicates
        // check simple inverted "and" filter with attributes predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"c\" and severity_text == \"DEBUG\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_not_and_kql_parser() {
        test_filter_not_and::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_not_and_opl_parser() {
        test_filter_not_and::<OplParser>().await;
    }

    async fn test_filter_not_or<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // check simple inverted "or" filter with properties predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(severity_text == \"INFO\" or event_name == \"2\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        // check simple inverted "or" filter with attributes predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"b\" or attributes[\"y\"] == \"f\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );

        // check simple inverted "or" filter with mixed attributes & properties predicates
        let result = exec_logs_pipeline::<P>(
            "logs | where not(attributes[\"x\"] == \"c\" or severity_text == \"INFO\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_not_or_kql_parser() {
        test_filter_not_or::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_not_or_opl_parser() {
        test_filter_not_or::<OplParser>().await;
    }

    async fn test_filter_with_nulls<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .finish(),
            LogRecord::build()
                .event_name("3")
                // severity_text == null
                .finish(),
        ];

        // check simple filter to ensure we filter out the value with null in the column
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_text == \"ERROR\"",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );

        // test a few scenarios where if we had null in the selection vector (which we
        // shouldn't have), they would not pass:
        let result = exec_logs_pipeline::<P>(
            "logs | where not(severity_text == \"ERROR\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );
        let result = exec_logs_pipeline::<P>(
            "logs | where severity_text == \"ERROR\" or event_name == \"3\"",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_nulls_kql_parser() {
        test_filter_with_nulls::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_with_nulls_opl_parser() {
        test_filter_with_nulls::<OplParser>().await;
    }

    async fn run_filter_numeric_comparison_binary_operators_test<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("z", AnyValue::new_int(1)),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("z", AnyValue::new_int(2)),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("z", AnyValue::new_int(3)),
                ])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            "logs | where attributes[\"z\"] > 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        let result = exec_logs_pipeline::<P>(
            "logs | where attributes[\"z\"] >= 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        let result = exec_logs_pipeline::<P>(
            "logs | where attributes[\"z\"] < 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );

        let result = exec_logs_pipeline::<P>(
            "logs | where attributes[\"z\"] <= 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_numeric_comparison_binary_operators_kql_parser() {
        run_filter_numeric_comparison_binary_operators_test::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_numeric_comparison_binary_operators_opl_parser() {
        run_filter_numeric_comparison_binary_operators_test::<OplParser>().await;
    }

    async fn test_filter_nomatch<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        let parser_result = P::parse("logs | where event_name == \"5\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        // assert it's equal to empty batch because there were no matches
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // assert we have the correct behaviour when filtering by attributes as well
        let parser_result = KqlParser::parse("logs | where attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()))
    }

    #[tokio::test]
    async fn test_filter_nomatch_kql_parser() {
        test_filter_nomatch::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_nomatch_opl_parser() {
        test_filter_nomatch::<OplParser>().await;
    }

    async fn test_empty_batch<P: Parser>() {
        let input = OtapArrowRecords::Logs(Logs::default());
        let parser_result = P::parse("logs | where event_name == \"5\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(input.clone()).await.unwrap();
        assert_eq!(result, input);
    }

    #[tokio::test]
    async fn test_empty_batch_kql_parser() {
        test_empty_batch::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_empty_batch_opl_parser() {
        test_empty_batch::<OplParser>().await;
    }

    async fn test_filter_no_attrs<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("INFO")
                .finish(),
        ];

        // check that if there are no attributes to filter by then, we get the empty batch
        let parser_result = P::parse("logs | where attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // check that the same result happens when filtering by resource and scope attrs
        let parser_result =
            P::parse("logs | where resource.attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // check that the same result happens when filtering by resource and scope attrs
        let parser_result =
            P::parse("logs | where instrumentation_scope.attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // check that inverting the filters above basically just return the original record batch
        for inverted_attrs_filter in [
            "logs | where not(attributes[\"a\"] == \"1234\")",
            "logs | where not(resource.attributes[\"a\"] == \"1234\")",
            "logs | where not(instrumentation_scope.attributes[\"a\"] == \"1234\")",
        ] {
            let parser_result = P::parse(inverted_attrs_filter).unwrap();
            let mut pipeline = Pipeline::new(parser_result.pipeline);
            let input = to_otap_logs(log_records.clone());
            let result = pipeline.execute(input.clone()).await.unwrap();
            assert_eq!(result, input);
        }
    }

    #[tokio::test]
    async fn test_filter_no_attrs_kql_parser() {
        test_filter_no_attrs::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_filter_no_attrs_opl_parser() {
        test_filter_no_attrs::<OplParser>().await;
    }

    async fn test_filter_property_is_null<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where severity_text == {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );

        // check it's supported if null literal on the left and column on the right
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == severity_text"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_property_is_null_kql_parser() {
        test_filter_property_is_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_property_is_null_opl_parser() {
        test_filter_property_is_null::<OplParser>("null").await;
    }

    async fn run_filter_property_is_not_null<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // severity_text != <null>
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where severity_text != {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );

        // <null> != severity_text
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} != severity_text"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_property_is_not_null_kql_parser() {
        run_filter_property_is_not_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_property_is_not_null_opl_parser() {
        run_filter_property_is_not_null::<OplParser>("null").await;
    }

    async fn run_filter_property_is_null_missing_column<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // just double check this gets encoded as something w/out the column we're using
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        assert!(logs_rb.column_by_name(consts::SEVERITY_TEXT).is_none());

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where severity_text == {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[
                log_records[0].clone(),
                log_records[1].clone(),
                log_records[2].clone()
            ],
        );

        // check it's supported if null literal on the left and column on the right
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == severity_text"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[
                log_records[0].clone(),
                log_records[1].clone(),
                log_records[2].clone()
            ],
        );
    }

    #[tokio::test]
    async fn test_filter_property_is_null_missing_column_kql_parser() {
        run_filter_property_is_null_missing_column::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_property_is_null_missing_column_opl_parser() {
        run_filter_property_is_null_missing_column::<OplParser>("null").await;
    }

    async fn run_filter_property_is_not_null_missing_column<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // just double check this gets encoded as something w/out the column we're using
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        assert!(logs_rb.column_by_name(consts::SEVERITY_TEXT).is_none());

        let parser_result = P::parse(&format!("logs | where severity_text != {null_lit}")).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        // assert it's equal to empty batch because there were no matches
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // assert we do the right thing where the null is on the left and value on the right
        let parser_result = P::parse(&format!("logs | where {null_lit} != severity_text")).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()))
    }

    #[tokio::test]
    async fn test_filter_property_is_not_null_missing_column_kql_parser() {
        run_filter_property_is_not_null_missing_column::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_property_is_not_null_missing_column_opl_parser() {
        run_filter_property_is_not_null_missing_column::<OplParser>("null").await;
    }

    async fn run_filter_struct_property_is_null<P: Parser>(null_lit: &str) {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        // test filter by scope properties
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where instrumentation_scope.name == {null_lit}"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[1].clone()],
                    ..Default::default()
                }],
            }
        );

        // test filter by scope properties, this time the null is on the left
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == instrumentation_scope.name"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[1].clone()],
                    ..Default::default()
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_null_kql_parser() {
        run_filter_struct_property_is_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_null_opl_parser() {
        run_filter_struct_property_is_null::<OplParser>("null").await;
    }

    async fn run_filter_struct_property_is_null_missing_column<P: Parser>(null_lit: &str) {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        // test filter by scope properties
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where instrumentation_scope.name == {null_lit}"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone(), scope_logs[1].clone()],
                    ..Default::default()
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_null_missing_column_kql_parser() {
        run_filter_struct_property_is_null_missing_column::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_null_missing_column_opl_parser() {
        run_filter_struct_property_is_null_missing_column::<OplParser>("null").await;
    }

    async fn run_struct_property_is_not_null<P: Parser>(null_lit: &str) {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        // test filter by scope properties
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where instrumentation_scope.name != {null_lit}"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );

        // test filter by scope properties, this time the null is on the left
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} != instrumentation_scope.name"),
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_struct_property_is_not_null_kql_parser() {
        run_struct_property_is_not_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_struct_property_is_not_null_opl_parser() {
        run_struct_property_is_not_null::<OplParser>("null").await;
    }

    async fn run_filter_struct_property_is_not_null_missing_column<P: Parser>(null_lit: &str) {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        let parser_result = P::parse(&format!(
            "logs | where instrumentation_scope.name != {null_lit}"
        ))
        .unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(otlp_to_otap(&OtlpProtoMessage::Logs(input)))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()))
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_not_null_missing_column_kql_parser() {
        run_filter_struct_property_is_not_null_missing_column::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_struct_property_is_not_null_missing_column_opl_parser() {
        run_filter_struct_property_is_not_null_missing_column::<OplParser>("null").await;
    }

    async fn run_filter_attribute_is_null<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build().event_name("2").finish(),
            LogRecord::build()
                .event_name("3")
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("b"))])
                .finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where attributes[\"x\"] == {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check the same thing works if we put null on the left
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == attributes[\"x\"]"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_attribute_is_null_kql_parser() {
        run_filter_attribute_is_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_attribute_is_null_opl_parser() {
        run_filter_attribute_is_null::<OplParser>("null").await;
    }

    async fn run_filter_attribute_is_null_no_attrs<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build().event_name("1").finish(),
            LogRecord::build().event_name("2").finish(),
            LogRecord::build().event_name("3").finish(),
        ];

        // double check that when we encode this as OTLP that the attributes
        // record batch is not present
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_none());

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where attributes[\"x\"] == {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &log_records.clone()
        );

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} == attributes[\"x\"]"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &log_records.clone()
        );
    }

    #[tokio::test]
    async fn test_filter_attribute_is_null_no_attrs_kql_parser() {
        run_filter_attribute_is_null_no_attrs::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_attribute_is_null_no_attrs_opl_parser() {
        run_filter_attribute_is_null_no_attrs::<OplParser>("null").await;
    }

    async fn run_filter_attribute_is_not_null<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("b"))])
                .finish(),
            LogRecord::build().event_name("3").finish(),
        ];

        let result = exec_logs_pipeline::<P>(
            &format!("logs | where attributes[\"x\"] != {null_lit}"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );

        // check the same thing works if we put null on the left
        let result = exec_logs_pipeline::<P>(
            &format!("logs | where {null_lit} != attributes[\"x\"]"),
            to_logs_data(log_records.clone()),
        )
        .await;

        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_attribute_is_not_null_kql_parser() {
        run_filter_attribute_is_not_null::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_attribute_is_not_null_opl_parser() {
        run_filter_attribute_is_not_null::<OplParser>("null").await;
    }

    async fn run_filter_attribute_is_not_null_no_attrs<P: Parser>(null_lit: &str) {
        let log_records = vec![
            LogRecord::build().event_name("1").finish(),
            LogRecord::build().event_name("2").finish(),
            LogRecord::build().event_name("3").finish(),
        ];

        // double check that when we encode this as OTLP that the attributes
        // record batch is not present
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_none());

        let parser_result =
            P::parse(&format!("logs | where attributes[\"x\"] != {null_lit}")).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        // assert it's equal to empty batch because there were no matches
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // assert we do the right thing where the null is on the left and value on the right
        let parser_result =
            P::parse(&format!("logs | where {null_lit} != attributes[\"x\"]")).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline
            .execute(to_otap_logs(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()))
    }

    #[tokio::test]
    async fn test_filter_attribute_is_not_null_no_attrs_kql_parser() {
        run_filter_attribute_is_not_null_no_attrs::<KqlParser>("string(null)").await;
    }

    #[tokio::test]
    async fn test_filter_attribute_is_not_null_no_attrs_opl_parser() {
        run_filter_attribute_is_not_null_no_attrs::<OplParser>("null").await;
    }

    async fn test_optional_attrs_existence_changes<P: Parser>() {
        // what happens if some optional attributes are present one batch, then not present in the
        // next, then present in the next, etc.

        let query = "logs | where attributes[\"a\"] == \"1234\"";
        let parser_result = P::parse(query).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);

        // no attrs to start
        let batch1 = to_otap_logs(vec![LogRecord::build().event_name("a").finish()]);
        let result = pipeline.execute(batch1).await.unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // now process a batch with some attrs
        let log_records = vec![
            LogRecord::build().finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("a", AnyValue::new_string("1234"))])
                .finish(),
        ];
        let batch2 = to_otap_logs(log_records.clone());
        let result = pipeline.execute(batch2).await.unwrap();
        let expected = to_otap_logs(log_records[1..2].to_vec());
        assert_eq!(result, expected);

        // handle another record batch with missing attributes
        let batch3 = to_otap_logs(vec![LogRecord::build().event_name("a").finish()]);
        let result = pipeline.execute(batch3).await.unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));
    }

    #[tokio::test]
    async fn test_filter_all_match() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // assert the behaviour is correct when nothing is filtered out
        let otap_input = to_otap_logs(log_records);
        let parser_result = KqlParser::parse("logs | where severity_text == \"INFO\"").unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_input.clone()).await.unwrap();

        assert_eq!(result, otap_input)
    }

    #[tokio::test]
    async fn test_optional_attrs_existence_changes_kql_parser() {
        test_optional_attrs_existence_changes::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_optional_attrs_existence_changes_opl_parser() {
        test_optional_attrs_existence_changes::<OplParser>().await;
    }

    #[test]
    fn test_id_mask_contains() {
        let all = IdMask::All;
        let none = IdMask::None;
        let some = IdMask::Some(id_bitmap_from(&[1, 2, 3]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[1, 2, 3]));

        assert!(all.contains(5));
        assert!(!none.contains(5));
        assert!(some.contains(2));
        assert!(!some.contains(5));
        assert!(!not_some.contains(2));
        assert!(not_some.contains(5));
    }

    #[test]
    fn test_id_mask_bitor_basic() {
        let mut pool = IdBitmapPool::new();
        let some1 = IdMask::Some(id_bitmap_from(&[1, 2]));
        let some2 = IdMask::Some(id_bitmap_from(&[2, 3]));

        match some1.combine_or(some2, &mut pool) {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(1));
                assert!(bitmap.contains(2));
                assert!(bitmap.contains(3));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_id_mask_bitor_with_all_none() {
        let mut pool = IdBitmapPool::new();

        assert!(matches!(
            IdMask::All.combine_or(IdMask::Some(id_bitmap_from(&[1, 2])), &mut pool),
            IdMask::All
        ));
        assert!(matches!(
            IdMask::Some(id_bitmap_from(&[1, 2])).combine_or(IdMask::All, &mut pool),
            IdMask::All
        ));
        assert!(matches!(
            IdMask::None.combine_or(IdMask::Some(id_bitmap_from(&[1, 2])), &mut pool),
            IdMask::Some(_)
        ));
        assert!(matches!(
            IdMask::Some(id_bitmap_from(&[1, 2])).combine_or(IdMask::None, &mut pool),
            IdMask::Some(_)
        ));
    }

    #[test]
    fn test_id_mask_bitor_some_notsome() {
        let mut pool = IdBitmapPool::new();
        let some = IdMask::Some(id_bitmap_from(&[1, 2, 3]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[2, 3, 4]));

        // Some([1,2,3]) | NotSome([2,3,4]) = NotSome([4])
        // Because we select 1,2,3 plus everything except 2,3,4
        // Result: everything except 4
        match some.combine_or(not_some, &mut pool) {
            IdMask::NotSome(bitmap) => {
                assert!(bitmap.contains(4));
                assert!(!bitmap.contains(1));
                assert!(!bitmap.contains(2));
            }
            _ => panic!("Expected NotSome variant"),
        }
    }

    #[test]
    fn test_bitor_some_notsome_becomes_all() {
        let mut pool = IdBitmapPool::new();
        // For this to become All, we need the NotSome set to be a subset of Some
        let some = IdMask::Some(id_bitmap_from(&[1, 2, 3, 4, 5]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[2, 3]));

        // Some([1,2,3,4,5]) | NotSome([2,3])
        // = [1,2,3,4,5] plus everything except [2,3]
        // = everything (because [2,3] - [1,2,3,4,5] = empty)
        assert!(matches!(some.combine_or(not_some, &mut pool), IdMask::All));
    }

    #[test]
    fn test_id_mask_bitor_notsome_notsome() {
        let mut pool = IdBitmapPool::new();
        let not_some1 = IdMask::NotSome(id_bitmap_from(&[1, 2]));
        let not_some2 = IdMask::NotSome(id_bitmap_from(&[2, 3]));

        // NotSome([1,2]) | NotSome([2,3]) = NotSome([2])
        match not_some1.combine_or(not_some2, &mut pool) {
            IdMask::NotSome(bitmap) => {
                assert!(bitmap.contains(2));
                assert!(!bitmap.contains(1));
                assert!(!bitmap.contains(3));
            }
            _ => panic!("Expected NotSome variant"),
        }
    }

    #[test]
    fn test_id_mask_bitand_basic() {
        let mut pool = IdBitmapPool::new();
        let some1 = IdMask::Some(id_bitmap_from(&[1, 2, 3]));
        let some2 = IdMask::Some(id_bitmap_from(&[2, 3, 4]));

        match some1.combine_and(some2, &mut pool) {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(2));
                assert!(bitmap.contains(3));
                assert!(!bitmap.contains(1));
                assert!(!bitmap.contains(4));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_id_mask_bitand_with_all_none() {
        let mut pool = IdBitmapPool::new();

        assert!(matches!(
            IdMask::None.combine_and(IdMask::Some(id_bitmap_from(&[1, 2])), &mut pool),
            IdMask::None
        ));
        assert!(matches!(
            IdMask::Some(id_bitmap_from(&[1, 2])).combine_and(IdMask::None, &mut pool),
            IdMask::None
        ));
        assert!(matches!(
            IdMask::All.combine_and(IdMask::Some(id_bitmap_from(&[1, 2])), &mut pool),
            IdMask::Some(_)
        ));
        assert!(matches!(
            IdMask::Some(id_bitmap_from(&[1, 2])).combine_and(IdMask::All, &mut pool),
            IdMask::Some(_)
        ));
    }

    #[test]
    fn test_id_mask_bitand_some_notsome() {
        let mut pool = IdBitmapPool::new();
        let some = IdMask::Some(id_bitmap_from(&[1, 2, 3, 4]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[3, 4, 5]));

        // Some([1,2,3,4]) & NotSome([3,4,5]) = Some([1,2])
        match some.combine_and(not_some, &mut pool) {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(1));
                assert!(bitmap.contains(2));
                assert!(!bitmap.contains(3));
                assert!(!bitmap.contains(4));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_id_mask_bitand_some_notsome_becomes_none() {
        let mut pool = IdBitmapPool::new();
        let some = IdMask::Some(id_bitmap_from(&[1, 2]));
        let not_some = IdMask::NotSome(id_bitmap_from(&[3, 4]));

        // Some([1,2]) & NotSome([3,4]) = Some([1,2])
        // (since [1,2] are not in [3,4])
        match some.combine_and(not_some, &mut pool) {
            IdMask::Some(bitmap) => {
                assert!(bitmap.contains(1));
                assert!(bitmap.contains(2));
            }
            _ => panic!("Expected Some variant"),
        }

        // But Some([1,2]) & NotSome([1,2,3]) = None
        let some2 = IdMask::Some(id_bitmap_from(&[1, 2]));
        let not_some2 = IdMask::NotSome(id_bitmap_from(&[1, 2, 3]));
        assert!(matches!(
            some2.combine_and(not_some2, &mut pool),
            IdMask::None
        ));
    }

    #[test]
    fn test_id_mask_bitand_notsome_notsome() {
        let mut pool = IdBitmapPool::new();
        let not_some1 = IdMask::NotSome(id_bitmap_from(&[1, 2]));
        let not_some2 = IdMask::NotSome(id_bitmap_from(&[2, 3]));

        // NotSome([1,2]) & NotSome([2,3]) = NotSome([1,2,3])
        match not_some1.combine_and(not_some2, &mut pool) {
            IdMask::NotSome(bitmap) => {
                assert!(bitmap.contains(1));
                assert!(bitmap.contains(2));
                assert!(bitmap.contains(3));
            }
            _ => panic!("Expected NotSome variant"),
        }
    }

    #[tokio::test]
    async fn test_composite_attributes_filter() {
        // Currently our plans don't construct the Composite<AttributeFilterExec> and until we have
        // that planning in place, we are just invoking it manually to ensure it actually works.

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());

        let attrs_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("parent_id", DataType::UInt16, false),
                Field::new("key", DataType::Utf8, false),
                Field::new("str", DataType::Utf8, true),
                Field::new("type", DataType::UInt8, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 0, 1, 1, 2, 2])),
                Arc::new(StringArray::from_iter_values([
                    "x", "y", "x", "y", "x", "y",
                ])),
                Arc::new(StringArray::from_iter_values([
                    "a", "d", "b", "e", "c", "f",
                ])),
                Arc::new(UInt8Array::from_iter_values([1, 1, 1, 1, 1, 1])),
            ],
        )
        .unwrap();

        otap_batch
            .set(ArrowPayloadType::LogAttrs, attrs_rb)
            .unwrap();

        let session_ctx = Pipeline::create_session_context();

        let filter_x_eq_a = AttributesFilterPlan {
            filter: col("key").eq(lit("x")).and(col("str").eq(lit("a"))),
            attrs_identifier: AttributesIdentifier::Root,
        };

        let filter_y_eq_d = AttributesFilterPlan {
            filter: col("key").eq(lit("y")).and(col("str").eq(lit("d"))),
            attrs_identifier: AttributesIdentifier::Root,
        };

        let filter_x_eq_b = AttributesFilterPlan {
            filter: col("key").eq(lit("x")).and(col("str").eq(lit("b"))),
            attrs_identifier: AttributesIdentifier::Root,
        };

        let mut pool = IdBitmapPool::new();

        // test simple filter
        let mut filter_exec: Composite<AttributeFilterExec> =
            Composite::<AttributesFilterPlan>::from(filter_x_eq_a.clone())
                .to_exec(&session_ctx, &otap_batch)
                .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false, &mut pool)
                .unwrap(),
            IdMask::Some(id_bitmap_from(&[0]))
        );

        // test simple not filter
        filter_exec = Composite::Not(Box::new(filter_x_eq_a.clone().into()))
            .to_exec(&session_ctx, &otap_batch)
            .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false, &mut pool)
                .unwrap(),
            IdMask::NotSome(id_bitmap_from(&[0]))
        );

        // test "and" filter
        filter_exec = Composite::And(
            Box::new(filter_x_eq_a.clone().into()),
            Box::new(filter_y_eq_d.clone().into()),
        )
        .to_exec(&session_ctx, &otap_batch)
        .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false, &mut pool)
                .unwrap(),
            IdMask::Some(id_bitmap_from(&[0]))
        );

        // test inverted "and" filter
        filter_exec = Composite::Not(Box::new(Composite::And(
            Box::new(filter_x_eq_a.clone().into()),
            Box::new(filter_y_eq_d.clone().into()),
        )))
        .to_exec(&session_ctx, &otap_batch)
        .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false, &mut pool)
                .unwrap(),
            IdMask::NotSome(id_bitmap_from(&[0]))
        );

        // test "or" filter
        filter_exec = Composite::Or(
            Box::new(filter_x_eq_a.clone().into()),
            Box::new(filter_x_eq_b.clone().into()),
        )
        .to_exec(&session_ctx, &otap_batch)
        .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false, &mut pool)
                .unwrap(),
            IdMask::Some(id_bitmap_from(&[0, 1]))
        );

        // test inverted "or" filter
        filter_exec = Composite::Not(Box::new(Composite::Or(
            Box::new(filter_x_eq_a.clone().into()),
            Box::new(filter_x_eq_b.clone().into()),
        )))
        .to_exec(&session_ctx, &otap_batch)
        .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false, &mut pool)
                .unwrap(),
            IdMask::NotSome(id_bitmap_from(&[0, 1]))
        );
    }

    #[tokio::test]
    async fn test_adaptive_physical_expr_type_change() {
        let logical_expr = col(consts::SEVERITY_TEXT).eq(lit("WARN"));
        let mut phys_expr = AdaptivePhysicalExprExec::try_new(logical_expr).unwrap();

        let session_ctx = Pipeline::create_session_context();

        // start of with column of type Dict<u8, utf8>
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            consts::SEVERITY_TEXT,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        )]));
        let rb1 = RecordBatch::try_new(
            schema1.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values([0, 1, 2, 1]),
                Arc::new(StringArray::from_iter_values(["DEBUG", "INFO", "WARN"])),
            ))],
        )
        .unwrap();

        let result = phys_expr.evaluate_filter(&rb1, &session_ctx).unwrap();
        assert_eq!(result, BooleanArray::from_iter([false, false, true, false]));

        // next batch has some column, with type utf8
        let schema2 = Arc::new(Schema::new(vec![Field::new(
            consts::SEVERITY_TEXT,
            DataType::Utf8,
            false,
        )]));

        let rb2 = RecordBatch::try_new(
            schema2.clone(),
            vec![Arc::new(StringArray::from_iter_values([
                "WARN", "INFO", "WARN", "ERROR", "DEBUG",
            ]))],
        )
        .unwrap();
        let result = phys_expr.evaluate_filter(&rb2, &session_ctx).unwrap();
        assert_eq!(
            result,
            BooleanArray::from_iter([true, false, true, false, false])
        );

        // next batch has some column with type Dict<u16, utf8>
        let schema3 = Arc::new(Schema::new(vec![Field::new(
            consts::SEVERITY_TEXT,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        )]));
        let rb3 = RecordBatch::try_new(
            schema3.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values([0, 1, 1]),
                Arc::new(StringArray::from_iter_values(["DEBUG", "WARN"])),
            ))],
        )
        .unwrap();
        let result = phys_expr.evaluate_filter(&rb3, &session_ctx).unwrap();
        assert_eq!(result, BooleanArray::from_iter([false, true, true]));
    }

    #[tokio::test]
    async fn test_adaptive_physical_expr_optional_column() {
        let logical_expr = col(consts::SEVERITY_TEXT).eq(lit("WARN"));
        let mut phys_expr = AdaptivePhysicalExprExec::try_new(logical_expr).unwrap();
        let session_ctx = Pipeline::create_session_context();

        // send an initial batch with the column missing to ensure we handle this correctly
        let schema0 = Arc::new(Schema::new(vec![Field::new(
            consts::EVENT_NAME,
            DataType::Utf8,
            false,
        )]));
        let rb0 = RecordBatch::try_new(
            schema0.clone(),
            vec![Arc::new(StringArray::from_iter_values([
                "a", "b", "a", "d",
            ]))],
        )
        .unwrap();
        let result = phys_expr.evaluate_filter(&rb0, &session_ctx).unwrap();
        assert_eq!(
            result,
            BooleanArray::from_iter([false, false, false, false])
        );

        // next batch has the column we expect
        let schema1 = Arc::new(Schema::new(vec![
            Field::new(consts::EVENT_NAME, DataType::Utf8, false),
            Field::new(consts::SEVERITY_TEXT, DataType::Utf8, false),
        ]));

        let rb1 = RecordBatch::try_new(
            schema1.clone(),
            vec![
                Arc::new(StringArray::from_iter_values(["a", "b", "a", "b", "a"])),
                Arc::new(StringArray::from_iter_values([
                    "WARN", "INFO", "WARN", "ERROR", "DEBUG",
                ])),
            ],
        )
        .unwrap();
        let result = phys_expr.evaluate_filter(&rb1, &session_ctx).unwrap();
        assert_eq!(
            result,
            BooleanArray::from_iter([true, false, true, false, false])
        );

        // next batch, the optional column is omitted
        let schema2 = Arc::new(Schema::new(vec![Field::new(
            consts::EVENT_NAME,
            DataType::Utf8,
            false,
        )]));
        let rb2 = RecordBatch::try_new(
            schema2.clone(),
            vec![Arc::new(StringArray::from_iter_values(["a", "b", "a"]))],
        )
        .unwrap();
        let result = phys_expr.evaluate_filter(&rb2, &session_ctx).unwrap();
        assert_eq!(result, BooleanArray::from_iter([false, false, false]));
    }

    #[tokio::test]
    async fn test_adaptive_physical_expr_columns_order_change() {
        let logical_expr = col(consts::SEVERITY_TEXT).eq(lit("WARN"));
        let mut phys_expr = AdaptivePhysicalExprExec::try_new(logical_expr).unwrap();
        let session_ctx = Pipeline::create_session_context();

        // process an initial batch with the columns in some given order
        let schema0 = Arc::new(Schema::new(vec![
            Field::new(consts::EVENT_NAME, DataType::Utf8, false),
            Field::new(consts::SEVERITY_TEXT, DataType::Utf8, false),
        ]));
        let rb0 = RecordBatch::try_new(
            schema0.clone(),
            vec![
                Arc::new(StringArray::from_iter_values(["a", "b", "a", "d"])),
                Arc::new(StringArray::from_iter_values([
                    "WARN", "INFO", "ERROR", "DEBUG",
                ])),
            ],
        )
        .unwrap();
        let result = phys_expr.evaluate_filter(&rb0, &session_ctx).unwrap();
        assert_eq!(result, BooleanArray::from_iter([true, false, false, false]));

        // next batch we switch the column order .. now severity_text is column 0
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            consts::SEVERITY_TEXT,
            DataType::Utf8,
            false,
        )]));

        let rb1 = RecordBatch::try_new(
            schema1.clone(),
            vec![Arc::new(StringArray::from_iter_values([
                "WARN", "INFO", "WARN", "ERROR", "DEBUG",
            ]))],
        )
        .unwrap();
        let result = phys_expr.evaluate_filter(&rb1, &session_ctx).unwrap();
        assert_eq!(
            result,
            BooleanArray::from_iter([true, false, true, false, false])
        );

        // next batch, we've changed the order again. now severity_text is column 2
        let schema2 = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false),
            Field::new(consts::SEVERITY_NUMBER, DataType::Int32, false),
            Field::new(consts::SEVERITY_TEXT, DataType::Utf8, false),
        ]));
        let rb2 = RecordBatch::try_new(
            schema2.clone(),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 1, 2])),
                Arc::new(Int32Array::from_iter_values([5, 5, 5])),
                Arc::new(StringArray::from_iter_values(["WARN", "WARN", "ERROR"])),
            ],
        )
        .unwrap();
        let result = phys_expr.evaluate_filter(&rb2, &session_ctx).unwrap();
        assert_eq!(result, BooleanArray::from_iter([true, true, false]));
    }

    #[tokio::test]
    async fn test_adaptive_physical_expr_columns_null_values() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            consts::SEVERITY_TEXT,
            DataType::Utf8,
            true,
        )]));

        // Note - this array is constructed in a specific way that ensures that when doing
        // something like `eq(arr, &StringArray::new_scalar("WARN"))`, we produce a values
        // buffer that has `true` in a null position. This is used to test  that we use the
        // correct logic when combining the null buffer with the values buffer.
        let mut offsets = OffsetBufferBuilder::new(4);
        offsets.push_length(4);
        offsets.push_length(4);
        offsets.push_length(4);
        offsets.push_length(4);
        let mut values = MutableBuffer::new(16);
        values.extend_from_slice(b"WARN");
        values.extend_from_slice(b"INFO");
        values.extend_from_slice(b"WARN");
        values.extend_from_slice(b"INFO");
        let mut nulls = NullBufferBuilder::new(4);
        nulls.append(true);
        nulls.append(false);
        nulls.append(false);
        nulls.append(true);
        let string_arr = StringArray::new(offsets.finish(), values.into(), nulls.finish());

        let input = RecordBatch::try_new(schema.clone(), vec![Arc::new(string_arr)]).unwrap();

        let session_ctx = Pipeline::create_session_context();

        // in this expression, we want to produce a selection vec that considers null to be false
        let logical_expr = col(consts::SEVERITY_TEXT).eq(lit("WARN"));
        let mut phys_expr = AdaptivePhysicalExprExec::try_new(logical_expr).unwrap();
        let result = phys_expr.evaluate_filter(&input, &session_ctx).unwrap();
        assert_eq!(result, BooleanArray::from_iter([true, false, false, false]));

        // some smoke tests for other null handling scenarios
        let logical_expr = col(consts::SEVERITY_TEXT).is_null();
        let mut phys_expr = AdaptivePhysicalExprExec::try_new(logical_expr).unwrap();
        let result = phys_expr.evaluate_filter(&input, &session_ctx).unwrap();
        assert_eq!(result, BooleanArray::from_iter([false, true, true, false]));

        let logical_expr = col(consts::SEVERITY_TEXT).is_not_null();
        let mut phys_expr = AdaptivePhysicalExprExec::try_new(logical_expr).unwrap();
        let result = phys_expr.evaluate_filter(&input, &session_ctx).unwrap();
        assert_eq!(result, BooleanArray::from_iter([true, false, false, true]));
    }

    /// A physical Expr that should not get called. This can be used to test that some
    /// code which was supposed optimize away the invocation of this expression does
    /// what it is supposed to
    #[derive(Debug, Eq, Hash, PartialEq)]
    struct PanickingPhysicalExpr {}

    impl std::fmt::Display for PanickingPhysicalExpr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PanickingPhysicalExpr(for test)")
        }
    }

    impl PhysicalExpr for PanickingPhysicalExpr {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn evaluate(
            &self,
            _batch: &RecordBatch,
        ) -> datafusion::error::Result<datafusion::logical_expr::ColumnarValue> {
            panic!("this shouldn't get called")
        }

        fn children(&self) -> Vec<&Arc<dyn PhysicalExpr>> {
            Vec::new()
        }

        fn with_new_children(
            self: Arc<Self>,
            _children: Vec<Arc<dyn PhysicalExpr>>,
        ) -> datafusion::error::Result<Arc<dyn PhysicalExpr>> {
            Ok(Arc::new(Self {}))
        }

        fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PanickingPhysicalExpr(for test)")
        }
    }

    #[test]
    fn test_composite_filter_exec_and_takes_short_circuit() {
        let mut filter_exec = Composite::and(
            FilterExec::from(
                AdaptivePhysicalExprExec::try_new(col("severity_text").eq(lit("y"))).unwrap(),
            ),
            FilterExec::from(AdaptivePhysicalExprExec {
                logical_expr: lit("should panic"), // placeholder b/c physical is already planned
                physical_expr: Some(Arc::new(PanickingPhysicalExpr {})),
                projection: Projection::from(vec!["severity_text".into()]),
                missing_data_passes: false,
            }),
        );

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "severity_text",
                DataType::Utf8,
                false,
            )])),
            vec![Arc::new(StringArray::from_iter_values(["a", "b", "c"]))],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        otap_batch
            .set(ArrowPayloadType::Logs, input.clone())
            .unwrap();

        let session_ctx = Pipeline::create_session_context();

        let mut pool = IdBitmapPool::new();
        let result = filter_exec
            .execute(&otap_batch, &session_ctx, &mut pool)
            .unwrap();
        assert_eq!(result.false_count(), input.num_rows());
    }

    #[test]
    fn test_composite_filter_exec_or_takes_short_circuit() {
        let mut filter_exec = Composite::or(
            FilterExec::from(
                AdaptivePhysicalExprExec::try_new(col("severity_text").eq(lit("a"))).unwrap(),
            ),
            FilterExec::from(AdaptivePhysicalExprExec {
                logical_expr: lit("should panic"), // placeholder b/c physical is already planned
                physical_expr: Some(Arc::new(PanickingPhysicalExpr {})),
                projection: Projection::from(vec!["severity_text".into()]),
                missing_data_passes: false,
            }),
        );

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "severity_text",
                DataType::Utf8,
                false,
            )])),
            vec![Arc::new(StringArray::from_iter_values(["a", "a", "a"]))],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        otap_batch
            .set(ArrowPayloadType::Logs, input.clone())
            .unwrap();

        let session_ctx = Pipeline::create_session_context();

        let mut pool = IdBitmapPool::new();
        let result = filter_exec
            .execute(&otap_batch, &session_ctx, &mut pool)
            .unwrap();
        assert_eq!(result.true_count(), input.num_rows());
    }

    #[test]
    fn test_composite_attr_exec_and_takes_short_circuit() {
        let mut attr_exec = Composite::and(
            AttributeFilterExec {
                payload_type: ArrowPayloadType::LogAttrs,
                filter: AdaptivePhysicalExprExec::try_new(col("key").eq(lit("y"))).unwrap(),
            },
            AttributeFilterExec {
                payload_type: ArrowPayloadType::LogAttrs,
                filter: AdaptivePhysicalExprExec {
                    logical_expr: lit("should panic"), // placeholder b/c physical is already planned
                    physical_expr: Some(Arc::new(PanickingPhysicalExpr {})),
                    projection: Projection::from(vec!["key".into()]),
                    missing_data_passes: false,
                },
            },
        );

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("parent_id", DataType::UInt16, false),
                Field::new("key", DataType::Utf8, false),
                Field::new("type", DataType::UInt8, false),
            ])),
            vec![
                Arc::new(UInt16Array::from(vec![0u16, 1, 2])),
                Arc::new(StringArray::from_iter_values(["a", "b", "c"])),
                Arc::new(UInt8Array::from_iter_values([1, 1, 1])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        otap_batch
            .set(ArrowPayloadType::LogAttrs, input.clone())
            .unwrap();
        let session_ctx = Pipeline::create_session_context();

        let mut pool = IdBitmapPool::new();
        let result = attr_exec
            .execute(&otap_batch, &session_ctx, false, &mut pool)
            .unwrap();
        assert_eq!(result, IdMask::None);

        // check we handle the inverted case as well
        let result = attr_exec
            .execute(&otap_batch, &session_ctx, true, &mut pool)
            .unwrap();
        assert_eq!(result, IdMask::All);
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_key_match() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key2", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("KEY1", AnyValue::new_string("val1"))])
                .finish(),
        ];

        let query = "logs | where attributes[\"key1\"] == \"val1\"";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_key_match_escape_special_chars() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key%1_1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new(
                    "keyabcd1x1",
                    AnyValue::new_string("val1"),
                )])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("KEY%1_1", AnyValue::new_string("val1"))])
                .finish(),
        ];

        let query = "logs | where attributes[\"key%1_1\"] == \"val1\"";
        let pipeline_expr = KqlParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_key_match_record_has_same_key_different_case()
     {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("key1", AnyValue::new_string("val1")),
                    KeyValue::new("KEY1", AnyValue::new_string("val2")),
                ])
                .finish(),
        ];

        let query = "logs | where attributes[\"key1\"] == \"val1\"";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        // test that since at least one of the attributes passes predicate, we get the result
        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()]
        );

        // test the negation as well: since one of the attributes having "key1" is equal to
        // "val1", we filter out the record
        let query = "logs | where attributes[\"key1\"] != \"val2\"";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let result = pipeline.execute(input).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };
        assert!(&result.resource_logs.is_empty());
    }

    async fn test_filter_by_attributes_case_insensitive_equals<P: Parser>() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("VAL1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let query = "logs | where attributes[\"key1\"] =~ \"val1\"";
        let pipeline_expr = P::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );

        // check it also works w/ the literal on the left
        let query = "logs | where \"val1\" =~ attributes[\"key1\"]";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_equals_opl_parser() {
        test_filter_by_attributes_case_insensitive_equals::<OplParser>().await
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_equals_kql_parser() {
        test_filter_by_attributes_case_insensitive_equals::<KqlParser>().await
    }

    #[tokio::test]
    async fn test_filter_by_attributes_case_insensitive_equals_escapes_special_chars() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val%1_1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("VAL%1_1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("valA1B1"))])
                .finish(),
        ];

        let query = "logs | where attributes[\"key1\"] =~ \"val%1_1\"";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );

        // check it also escapes correctly when the literal is on the left
        let query = "logs | where  \"val%1_1\" =~ attributes[\"key1\"]";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_contains_case_insensitive_key_match() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("KEY1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let query = "logs | where contains(attributes[\"key1\"], \"1\")";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }

    #[tokio::test]
    async fn test_filter_matches_case_insensitive_key_match() {
        let log_records = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("KEY1", AnyValue::new_string("val1"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("key1", AnyValue::new_string("val2"))])
                .finish(),
        ];

        let query = "logs | where matches(attributes[\"key1\"], \".*1\")";
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new_with_options(
            pipeline_expr,
            PipelineOptions {
                filter_attribute_keys_case_sensitive: false,
            },
        );
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(to_logs_data(log_records.clone())));
        let result = pipeline.execute(input.clone()).await.unwrap();

        let OtlpProtoMessage::Logs(result) = otap_to_otlp(&result) else {
            panic!("invalid variant {:?}", result)
        };

        assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()]
        );
    }
}
