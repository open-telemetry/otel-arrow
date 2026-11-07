// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains utilities for inferring how batch metadata will be accessed from the
//! result of a parsed program.

use otel_arrow_contrib_data_engine_expressions::{
    BinaryMathematicalScalarExpression, CollectionScalarExpression, ConversionScalarExpression,
    ConvertScalarExpression, DataExpression, LogicalExpression, MathScalarExpression,
    ParseScalarExpression, PipelineExpression, PipelineFunction, ScalarExpression,
    TemporalScalarExpression, TextScalarExpression,
};
use otel_arrow_dfe_engine::pdata_context::ContextDeclaration;

pub mod metadata;

/// Implementor can visit the Expression AST resulting from some parsed query
///
/// Currently the only method is for visiting scalar expressions. More trait methods may be added
/// in the future as the need for visiting other types of expressions arises.
pub trait ExprAstVisitor {
    /// Visit a [`ScalarExpression`]
    fn visit_scalar_expr(&mut self, scalar_expr: &ScalarExpression, functions: &[PipelineFunction]);
}

/// Visit the [`PipelineExpression`] AST.
///
/// Note: this function makes no guarantees about order in which the expressions are visited.
pub fn visit_expr_ast<T: ExprAstVisitor>(pipeline_expr: &PipelineExpression, visitor: &mut T) {
    let functions = pipeline_expr.get_functions();
    for expr in pipeline_expr.get_expressions() {
        visit_data_expr(expr, functions, visitor);
    }
}

fn visit_data_expr<T: ExprAstVisitor>(
    data_expr: &DataExpression,
    functions: &[PipelineFunction],
    visitor: &mut T,
) {
    match data_expr {
        DataExpression::Branch(branch) => {
            for branch in branch.get_branches() {
                if let Some(logical_expr) = branch.get_condition() {
                    visit_logical_expr(logical_expr, functions, visitor);
                }
                for expr in branch.get_expressions() {
                    visit_data_expr(expr, functions, visitor);
                }
            }
        }
        _ => {
            todo!()
        }
    }
}

fn visit_logical_expr<T: ExprAstVisitor>(
    logical_expr: &LogicalExpression,
    functions: &[PipelineFunction],
    visitor: &mut T,
) {
    match logical_expr {
        LogicalExpression::And(and) => {
            visit_logical_expr(and.get_left(), functions, visitor);
            visit_logical_expr(and.get_right(), functions, visitor);
        }
        LogicalExpression::Or(or) => {
            visit_logical_expr(or.get_left(), functions, visitor);
            visit_logical_expr(or.get_right(), functions, visitor);
        }
        LogicalExpression::Contains(contains) => {
            visit_scalar_expr(contains.get_haystack(), functions, visitor);
            visit_scalar_expr(contains.get_haystack(), functions, visitor);
        }
        LogicalExpression::EqualTo(eq) => {
            visit_scalar_expr(eq.get_left(), functions, visitor);
            visit_scalar_expr(eq.get_right(), functions, visitor);
        }
        LogicalExpression::GreaterThan(gt) => {
            visit_scalar_expr(gt.get_left(), functions, visitor);
            visit_scalar_expr(gt.get_right(), functions, visitor);
        }
        LogicalExpression::GreaterThanOrEqualTo(gte) => {
            visit_scalar_expr(gte.get_left(), functions, visitor);
            visit_scalar_expr(gte.get_right(), functions, visitor);
        }
        LogicalExpression::Matches(matches) => {
            visit_scalar_expr(matches.get_haystack(), functions, visitor);
            visit_scalar_expr(matches.get_pattern(), functions, visitor);
        }
        LogicalExpression::Not(not) => {
            visit_logical_expr(not.get_inner_expression(), functions, visitor);
        }
        LogicalExpression::Scalar(s) => {
            visit_scalar_expr(s, functions, visitor);
        }
    }
}

fn visit_scalar_expr<T: ExprAstVisitor>(
    scalar_expression: &ScalarExpression,
    functions: &[PipelineFunction],
    visitor: &mut T,
) {
    visitor.visit_scalar_expr(scalar_expression, functions);

    // visit children of scalar expressions
    match scalar_expression {
        ScalarExpression::Argument(_)
        | ScalarExpression::Attached(_)
        | ScalarExpression::Constant(_)
        | ScalarExpression::GetRecordType(_)
        | ScalarExpression::Source(_)
        | ScalarExpression::Temporal(TemporalScalarExpression::Now(_))
        | ScalarExpression::Static(_)
        | ScalarExpression::Variable(_) => {
            // These types of scalar expression do contain children, but not the such as
            // value accessors or static constants, etc. but they are not the types that
            // the visitor supports, so we don't traverse into them.
        }
        ScalarExpression::Collection(c) => match c {
            CollectionScalarExpression::Concat(c) => {
                visit_scalar_expr(c.get_values_expression(), functions, visitor)
            }
            CollectionScalarExpression::List(l) => {
                for expr in l.get_value_expressions() {
                    visit_scalar_expr(expr, functions, visitor);
                }
            }
        },
        ScalarExpression::Case(c) => {
            for (cond, branch) in c.get_expressions_with_conditions() {
                visit_logical_expr(cond, functions, visitor);
                visit_scalar_expr(branch, functions, visitor);
            }
        }
        ScalarExpression::Coalesce(c) => {
            for expr in c.get_expressions() {
                visit_scalar_expr(expr, functions, visitor);
            }
        }
        ScalarExpression::Conditional(c) => {
            visit_logical_expr(c.get_condition(), functions, visitor);
            visit_scalar_expr(c.get_false_expression(), functions, visitor);
            visit_scalar_expr(c.get_true_expression(), functions, visitor);
        }
        ScalarExpression::Convert(c) => match c {
            ConvertScalarExpression::Boolean(inner)
            | ConvertScalarExpression::DateTime(inner)
            | ConvertScalarExpression::Double(inner)
            | ConvertScalarExpression::TimeSpan(inner)
            | ConvertScalarExpression::String(inner)
            | ConvertScalarExpression::Integer(inner) => {
                visit_scalar_expr(inner.get_inner_expression(), functions, visitor);
            }
        },
        ScalarExpression::GetType(t) => {
            visit_scalar_expr(t.get_value(), functions, visitor);
        }
        ScalarExpression::InvokeFunction(f) => {
            println!("TODO") // this is the important one
        }
        ScalarExpression::Length(l) => {
            visit_scalar_expr(l.get_inner_expression(), functions, visitor);
        }
        ScalarExpression::Logical(l) => {
            visit_logical_expr(&l, functions, visitor);
        }
        ScalarExpression::Math(m) => match m {
            MathScalarExpression::Add(b)
            | MathScalarExpression::Bin(b)
            | MathScalarExpression::Divide(b)
            | MathScalarExpression::Modulus(b)
            | MathScalarExpression::Multiply(b)
            | MathScalarExpression::Subtract(b) => {
                visit_scalar_expr(b.get_left_expression(), functions, visitor);
                visit_scalar_expr(b.get_right_expression(), functions, visitor);
            }
            MathScalarExpression::Ceiling(u)
            | MathScalarExpression::Floor(u)
            | MathScalarExpression::Negate(u) => {
                visit_scalar_expr(u.get_value_expression(), functions, visitor);
            }
        },
        ScalarExpression::Parse(p) => match p {
            ParseScalarExpression::Json(j) => {
                visit_scalar_expr(j.get_inner_expression(), functions, visitor);
            }

            ParseScalarExpression::JsonPath(j) => {
                visit_scalar_expr(j.get_inner_expression(), functions, visitor);
            }
            ParseScalarExpression::Regex(r) => {
                visit_scalar_expr(r.get_pattern(), functions, visitor);
            }
        },
        ScalarExpression::Select(s) => {
            visit_scalar_expr(s.get_selectors(), functions, visitor);
            visit_scalar_expr(s.get_value(), functions, visitor);
        }
        ScalarExpression::Slice(s) => {
            visit_scalar_expr(s.get_source(), functions, visitor);
            if let Some(range_len) = s.get_range_length() {
                visit_scalar_expr(range_len, functions, visitor);
            }
            if let Some(range_start) = s.get_range_start() {
                visit_scalar_expr(range_start, functions, visitor);
            }
        }

        ScalarExpression::Text(text) => match text {
            TextScalarExpression::Capture(c) => {
                visit_scalar_expr(c.get_capture_group(), functions, visitor);
                visit_scalar_expr(c.get_haystack(), functions, visitor);
                visit_scalar_expr(c.get_pattern(), functions, visitor);
            }
            TextScalarExpression::Concat(c) => {
                visit_scalar_expr(c.get_values_expression(), functions, visitor);
            }
            TextScalarExpression::Join(c) => {
                visit_scalar_expr(c.get_separator_expression(), functions, visitor);
                visit_scalar_expr(c.get_values_expression(), functions, visitor);
            }
            TextScalarExpression::Replace(r) => {
                visit_scalar_expr(r.get_haystack_expression(), functions, visitor);
                visit_scalar_expr(r.get_needle_expression(), functions, visitor);
                visit_scalar_expr(r.get_replacement_expression(), functions, visitor);
            }
        },
    }
}
