// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    DataExpression, DiscardDataExpression, LogicalExpression, NotLogicalExpression,
};
use data_engine_parser_abstractions::{ParserError, ParserState, to_query_location};
use pest::iterators::Pair;

use crate::parser::expression::parse_expression;
use crate::parser::{Rule, invalid_child_rule_error};

pub(crate) fn parse_operator_call(
    rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    for rule in rule.into_inner() {
        match rule.as_rule() {
            Rule::where_operator_call => parse_where_operator_call(rule, state)?,
            invalid_rule => {
                let query_location = to_query_location(&rule);
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::operator_call,
                    invalid_rule,
                ));
            }
        };
    }

    Ok(())
}

pub(crate) fn parse_where_operator_call(
    operator_call_rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    let operator_call_query_location = to_query_location(&operator_call_rule);
    if let Some(rule) = operator_call_rule.into_inner().next() {
        match rule.as_rule() {
            Rule::expression => {
                let rule_query_location = to_query_location(&rule);
                let predicate = parse_expression(rule)?;
                let discard_expr = DiscardDataExpression::new(operator_call_query_location)
                    .with_predicate(LogicalExpression::Not(NotLogicalExpression::new(
                        rule_query_location,
                        predicate,
                    )));
                state.push_expression(DataExpression::Discard(discard_expr));
            }

            invalid_rule => {
                let query_location = to_query_location(&rule);
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::operator_call,
                    invalid_rule,
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use data_engine_expressions::{
        DataExpression, DiscardDataExpression, EqualToLogicalExpression, LogicalExpression,
        NotLogicalExpression, QueryLocation, ScalarExpression, SourceScalarExpression,
        StaticScalarExpression, StringScalarExpression, ValueAccessor,
    };
    use data_engine_parser_abstractions::{ParserOptions, ParserState};
    use pest::Parser as _;
    use pretty_assertions::assert_eq;

    use crate::parser::Rule;
    use crate::parser::operator::parse_operator_call;
    use crate::parser::pest::OplPestParser;

    #[test]
    fn test_parse_where_operator_call() {
        let query = "where value == \"x\"";
        let mut state = ParserState::new_with_options(query, ParserOptions::default());
        let parse_result = OplPestParser::parse(Rule::operator_call, query).unwrap();
        assert_eq!(parse_result.len(), 1);
        let rule = parse_result.into_iter().next().unwrap();
        parse_operator_call(rule, &mut state).unwrap();

        let result = state.build().unwrap();
        let expressions = result.get_expressions();
        assert_eq!(expressions.len(), 1);
        let expected = DataExpression::Discard(
            DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                LogicalExpression::Not(NotLogicalExpression::new(
                    QueryLocation::new_fake(),
                    LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "value",
                                )),
                            )]),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "x"),
                        )),
                        true,
                    )),
                )),
            ),
        );

        assert_eq!(&expressions[0], &expected);
    }
}
