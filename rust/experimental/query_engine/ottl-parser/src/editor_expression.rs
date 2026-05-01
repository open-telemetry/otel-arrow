// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    DataExpression, MutableValueExpression, ScalarExpression, SetTransformExpression,
    TransformExpression,
};
use data_engine_parser_abstractions::{ParserError, ParserState, to_query_location};
use pest::iterators::Pair;

use crate::{ottl_parser::Rule, scalar_expression::parse_scalar_expression};

/// Parse an editor expression.
///
/// For more info on editor expressions, see:
/// https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/v0.151.0/pkg/ottl/LANGUAGE.md#editors
pub(crate) fn parse_editor_expression(
    rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();

    if inner_rules.len() != 2 {
        return Err(ParserError::SyntaxError(
            query_location,
            format!(
                "{:?} expected two inner rule, found {}",
                Rule::editor_expression,
                inner_rules.len()
            ),
        ));
    }

    // Parse the name of the editor. For complete list of editors, see:
    // https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/v0.151.0/pkg/ottl/ottlfuncs#editors

    let editor_name = inner_rules.next().expect("there are two rules");
    let arg_list = inner_rules.next().expect("there are two rules");

    match editor_name.as_str() {
        "set" => {
            let (destination, source) = parse_set_expr_args(arg_list, state)?;
            state.push_expression(DataExpression::Transform(TransformExpression::Set(
                SetTransformExpression::new(query_location, source, destination),
            )));
        }
        other => {
            return Err(ParserError::SyntaxNotSupported(
                query_location,
                format!("editor function '{other}' not supported"),
            ));
        }
    }

    Ok(())
}

/// parse the destination and source for "set" editor expression
fn parse_set_expr_args(
    args: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(MutableValueExpression, ScalarExpression), ParserError> {
    let query_location = to_query_location(&args);
    let mut inner_rules = args.into_inner();
    if inner_rules.len() != 2 {
        return Err(ParserError::SyntaxError(
            query_location,
            format!(
                "set editor expression expected two args rule, found {}",
                inner_rules.len()
            ),
        ));
    }

    let destination = inner_rules.next().expect("there are two rules");
    let dest_query_location = to_query_location(&destination);
    let destination = match parse_scalar_expression(destination, state)? {
        ScalarExpression::Source(source_scalar_expr) => {
            MutableValueExpression::Source(source_scalar_expr)
        }
        other => {
            return Err(ParserError::SyntaxError(
                dest_query_location,
                format!("expected source scalar expression, found {other:?}"),
            ));
        }
    };

    let source = parse_scalar_expression(inner_rules.next().expect("there are rwo rules"), state)?;

    Ok((destination, source))
}
