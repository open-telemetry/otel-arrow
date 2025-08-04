use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{
    Rule, logical_expressions::parse_logical_expression,
    scalar_conditional_function_expressions::*, scalar_primitive_expressions::*,
};

pub(crate) fn parse_scalar_expression(
    scalar_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let scalar_rule = scalar_expression_rule.into_inner().next().unwrap();

    let scalar = match scalar_rule.as_rule() {
        Rule::null_literal => ScalarExpression::Static(parse_standard_null_literal(scalar_rule)),
        Rule::real_expression => ScalarExpression::Static(parse_real_expression(scalar_rule)?),
        Rule::datetime_expression => {
            ScalarExpression::Static(parse_datetime_expression(scalar_rule)?)
        }
        Rule::conditional_expression => parse_conditional_expression(scalar_rule, state)?,
        Rule::strlen_expression => parse_strlen_expression(scalar_rule, state)?,
        Rule::replace_string_expression => parse_replace_string_expression(scalar_rule, state)?,
        Rule::case_expression => parse_case_expression(scalar_rule, state)?,
        Rule::true_literal | Rule::false_literal => {
            ScalarExpression::Static(parse_standard_bool_literal(scalar_rule))
        }
        Rule::double_literal => {
            ScalarExpression::Static(parse_standard_double_literal(scalar_rule, None)?)
        }
        Rule::integer_literal => {
            ScalarExpression::Static(parse_standard_integer_literal(scalar_rule)?)
        }
        Rule::string_literal => ScalarExpression::Static(parse_string_literal(scalar_rule)),
        Rule::accessor_expression => {
            // Note: When used as a scalar expression it is valid for an
            // accessor to fold into a static at the root so
            // allow_root_scalar=true is passed here. Example: iff([logical],
            // [scalar], [scalar]) evaluated as iff([logical],
            // accessor(some_constant1), accessor(some_constant2)) can safely
            // fold to iff([logical], String("constant1"), String("constant2")).
            parse_accessor_expression(scalar_rule, state, true)?
        }
        Rule::logical_expression => {
            let l = parse_logical_expression(scalar_rule, state)?;

            if let LogicalExpression::Scalar(s) = l {
                s
            } else {
                ScalarExpression::Logical(l.into())
            }
        }
        Rule::scalar_expression => parse_scalar_expression(scalar_rule, state)?,
        _ => panic!("Unexpected rule in scalar_expression: {scalar_rule}"),
    };

    Ok(scalar)
}

pub(crate) fn parse_strlen_expression(
    strlen_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&strlen_expression_rule);

    let mut strlen_rules = strlen_expression_rule.into_inner();
    let inner_expression = strlen_rules.next().unwrap();

    Ok(ScalarExpression::Length(LengthScalarExpression::new(
        query_location,
        parse_scalar_expression(inner_expression, state)?,
    )))
}

pub(crate) fn parse_replace_string_expression(
    replace_string_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&replace_string_expression_rule);

    let mut replace_string_rules = replace_string_expression_rule.into_inner();
    let text_expression = replace_string_rules.next().unwrap();
    let lookup_expression = replace_string_rules.next().unwrap();
    let replacement_expression = replace_string_rules.next().unwrap();

    Ok(ScalarExpression::ReplaceString(
        ReplaceStringScalarExpression::new(
            query_location,
            parse_scalar_expression(text_expression, state)?,
            parse_scalar_expression(lookup_expression, state)?,
            parse_scalar_expression(replacement_expression, state)?,
        ),
    ))
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::{KqlPestParser, date_utils::create_utc};

    use super::*;

    #[test]
    fn test_pest_parse_scalar_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::scalar_expression,
            &[
                "1",
                "1e1",
                "real(1)",
                "datetime(6/9/2025)",
                "true",
                "false",
                "(true == true)",
                "\"hello world\"",
                "variable",
                "(1)",
                "iff(true, 0, 1)",
                "case(true, 1, false)",
                "case(true, 1, false, 2, 0)",
                "bool(null)",
                "int(null)",
                "long(null)",
                "real(null)",
                "datetime(null)",
                "time(null)",
                "timespan(null)",
                "guid(null)",
                "dynamic(null)",
                "strlen(\"hello\")",
                "replace_string(\"text\", \"old\", \"new\")",
            ],
            &["!"],
        );
    }

    #[test]
    fn test_parse_scalar_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "1",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );

        run_test_success(
            "(1)",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );

        run_test_success(
            "1e1",
            ScalarExpression::Static(StaticScalarExpression::Double(DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1e1,
            ))),
        );

        run_test_success(
            "real(1)",
            ScalarExpression::Static(StaticScalarExpression::Double(DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.0,
            ))),
        );

        run_test_success(
            "datetime(6/9/2025)",
            ScalarExpression::Static(StaticScalarExpression::DateTime(
                DateTimeScalarExpression::new(
                    QueryLocation::new_fake(),
                    create_utc(2025, 6, 9, 0, 0, 0, 0),
                ),
            )),
        );

        run_test_success(
            "true",
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
        );

        run_test_success(
            "false",
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            )),
        );

        run_test_success(
            "(true == true)",
            ScalarExpression::Logical(
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
                ))
                .into(),
            ),
        );

        run_test_success(
            "\"hello world\"",
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
        );

        run_test_success(
            "identifier",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "identifier",
                    )),
                )]),
            )),
        );

        run_test_success(
            "bool(null)",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "datetime(null)",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "dynamic(null)",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "guid(null)",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "int(null)",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "long(null)",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "real(null)",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "double(null)",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "timespan(null)",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "strlen(\"hello\")",
            ScalarExpression::Length(LengthScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                )),
            )),
        );

        run_test_success(
            "replace_string(\"A magic trick can turn a cat into a dog\", \"cat\", \"hamster\")",
            ScalarExpression::ReplaceString(ReplaceStringScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "A magic trick can turn a cat into a dog",
                    ),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "cat"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hamster"),
                )),
            )),
        );

        run_test_success(
            "case(true, 1, 0)",
            ScalarExpression::Case(CaseScalarExpression::new(
                QueryLocation::new_fake(),
                vec![(
                    LogicalExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            true,
                        )),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                )],
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            )),
        );
    }
}
