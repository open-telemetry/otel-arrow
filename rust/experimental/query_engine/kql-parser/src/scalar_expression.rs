// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::LazyLock;

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::{iterators::Pair, pratt_parser::*};

use crate::{
    Rule, logical_expressions::to_logical_expression, scalar_array_function_expressions::*,
    scalar_conditional_function_expressions::*, scalar_conversion_function_expressions::*,
    scalar_logical_function_expressions::*, scalar_mathematical_function_expressions::*,
    scalar_parse_function_expressions::*, scalar_primitive_expressions::*,
    scalar_string_function_expressions::*, scalar_temporal_function_expressions::*,
};

static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    use Assoc::*;
    use Rule::*;

    // Lowest precedence first
    PrattParser::new()
        // or
        .op(Op::infix(or_token, Left))
        // and
        .op(Op::infix(and_token, Left))
        // == !=
        .op(Op::infix(equals_token, Left)
            | Op::infix(equals_insensitive_token, Left)
            | Op::infix(not_equals_token, Left)
            | Op::infix(not_equals_insensitive_token, Left))
        // <= >= < >
        .op(Op::infix(less_than_or_equal_to_token, Left)
            | Op::infix(greater_than_or_equal_to_token, Left)
            | Op::infix(less_than_token, Left)
            | Op::infix(greater_than_token, Left))
        // contains & has
        .op(Op::infix(not_contains_cs_token, Left)
            | Op::infix(not_contains_token, Left)
            | Op::infix(not_has_cs_token, Left)
            | Op::infix(not_has_token, Left)
            | Op::infix(contains_cs_token, Left)
            | Op::infix(contains_token, Left)
            | Op::infix(has_cs_token, Left)
            | Op::infix(has_token, Left))
        // in
        .op(Op::infix(not_in_insensitive_token, Left)
            | Op::infix(not_in_token, Left)
            | Op::infix(in_insensitive_token, Left)
            | Op::infix(in_token, Left))
        // matches
        .op(Op::infix(matches_regex_token, Left))
        // + -
        .op(Op::infix(plus_token, Left) | Op::infix(minus_token, Left))
        // * / %
        .op(Op::infix(multiply_token, Left)
            | Op::infix(divide_token, Left)
            | Op::infix(modulo_token, Left))

    // ^ ** (right-associative)
    //.op(Op::infix(power, Right))
});

pub(crate) fn parse_scalar_expression(
    scalar_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::scalar_unary_expression => parse_scalar_unary_expression(primary, scope),
            Rule::scalar_expression => parse_scalar_expression(primary, scope),
            Rule::scalar_list_expression => {
                let location = to_query_location(&primary);

                let mut values = Vec::new();

                for rule in primary.into_inner() {
                    let scalar = parse_scalar_expression(rule, scope)?;

                    values.push(scalar);
                }

                Ok(ScalarExpression::Collection(
                    CollectionScalarExpression::List(ListScalarExpression::new(location, values)),
                ))
            }
            _ => panic!("Unexpected rule in scalar_expression: {primary}"),
        })
        .map_infix(|lhs, op, rhs| {
            let location = to_query_location(&op);
            let lhs = lhs?;
            let rhs = rhs?;

            Ok(match op.as_rule() {
                Rule::equals_token => ScalarExpression::Logical(
                    LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                        location, lhs, rhs, false,
                    ))
                    .into(),
                ),

                Rule::equals_insensitive_token => ScalarExpression::Logical(
                    LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                        location, lhs, rhs, true,
                    ))
                    .into(),
                ),

                Rule::not_equals_token => ScalarExpression::Logical(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        location.clone(),
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            location, lhs, rhs, false,
                        )),
                    ))
                    .into(),
                ),

                Rule::not_equals_insensitive_token => ScalarExpression::Logical(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        location.clone(),
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            location, lhs, rhs, true,
                        )),
                    ))
                    .into(),
                ),

                Rule::greater_than_token => ScalarExpression::Logical(
                    LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                        location, lhs, rhs,
                    ))
                    .into(),
                ),

                Rule::greater_than_or_equal_to_token => ScalarExpression::Logical(
                    LogicalExpression::GreaterThanOrEqualTo(
                        GreaterThanOrEqualToLogicalExpression::new(location, lhs, rhs),
                    )
                    .into(),
                ),

                Rule::less_than_token => ScalarExpression::Logical(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        location.clone(),
                        LogicalExpression::GreaterThanOrEqualTo(
                            GreaterThanOrEqualToLogicalExpression::new(location, lhs, rhs),
                        ),
                    ))
                    .into(),
                ),

                Rule::less_than_or_equal_to_token => ScalarExpression::Logical(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        location.clone(),
                        LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                            location, lhs, rhs,
                        )),
                    ))
                    .into(),
                ),

                Rule::matches_regex_token => ScalarExpression::Logical(
                    LogicalExpression::Matches(MatchesLogicalExpression::new(location, lhs, rhs))
                        .into(),
                ),

                Rule::not_contains_cs_token | Rule::not_has_cs_token => ScalarExpression::Logical(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        location.clone(),
                        LogicalExpression::Contains(ContainsLogicalExpression::new(
                            location, lhs, rhs, false,
                        )),
                    ))
                    .into(),
                ),

                Rule::not_contains_token | Rule::not_has_token => ScalarExpression::Logical(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        location.clone(),
                        LogicalExpression::Contains(ContainsLogicalExpression::new(
                            location, lhs, rhs, true,
                        )),
                    ))
                    .into(),
                ),

                Rule::contains_cs_token | Rule::has_cs_token => ScalarExpression::Logical(
                    LogicalExpression::Contains(ContainsLogicalExpression::new(
                        location, lhs, rhs, false,
                    ))
                    .into(),
                ),

                Rule::contains_token | Rule::has_token => ScalarExpression::Logical(
                    LogicalExpression::Contains(ContainsLogicalExpression::new(
                        location, lhs, rhs, true,
                    ))
                    .into(),
                ),

                Rule::not_in_insensitive_token => ScalarExpression::Logical(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        location.clone(),
                        LogicalExpression::Contains(ContainsLogicalExpression::new(
                            location, rhs, lhs, true,
                        )),
                    ))
                    .into(),
                ),

                Rule::not_in_token => ScalarExpression::Logical(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        location.clone(),
                        LogicalExpression::Contains(ContainsLogicalExpression::new(
                            location, rhs, lhs, false,
                        )),
                    ))
                    .into(),
                ),

                Rule::in_insensitive_token => ScalarExpression::Logical(
                    LogicalExpression::Contains(ContainsLogicalExpression::new(
                        location, rhs, lhs, true,
                    ))
                    .into(),
                ),

                Rule::in_token => ScalarExpression::Logical(
                    LogicalExpression::Contains(ContainsLogicalExpression::new(
                        location, rhs, lhs, false,
                    ))
                    .into(),
                ),

                Rule::and_token => ScalarExpression::Logical(
                    LogicalExpression::And(AndLogicalExpression::new(
                        location,
                        to_logical_expression(lhs, scope)?,
                        to_logical_expression(rhs, scope)?,
                    ))
                    .into(),
                ),

                Rule::or_token => ScalarExpression::Logical(
                    LogicalExpression::Or(OrLogicalExpression::new(
                        location,
                        to_logical_expression(lhs, scope)?,
                        to_logical_expression(rhs, scope)?,
                    ))
                    .into(),
                ),

                Rule::multiply_token => ScalarExpression::Math(MathScalarExpression::Multiply(
                    BinaryMathematicalScalarExpression::new(location, lhs, rhs),
                )),

                Rule::divide_token => ScalarExpression::Math(MathScalarExpression::Divide(
                    BinaryMathematicalScalarExpression::new(location, lhs, rhs),
                )),

                Rule::modulo_token => ScalarExpression::Math(MathScalarExpression::Modulus(
                    BinaryMathematicalScalarExpression::new(location, lhs, rhs),
                )),

                Rule::plus_token => ScalarExpression::Math(MathScalarExpression::Add(
                    BinaryMathematicalScalarExpression::new(location, lhs, rhs),
                )),

                Rule::minus_token => ScalarExpression::Math(MathScalarExpression::Subtract(
                    BinaryMathematicalScalarExpression::new(location, lhs, rhs),
                )),

                _ => panic!("Unexpected rule in scalar_expression: {op}"),
            })
        })
        .parse(scalar_expression_rule.into_inner())
}

pub(crate) fn parse_scalar_unary_expression(
    scalar_unary_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let rule = scalar_unary_expression_rule.into_inner().next().unwrap();

    Ok(match rule.as_rule() {
        Rule::type_unary_expressions => {
            ScalarExpression::Static(parse_type_unary_expressions(rule)?)
        }
        Rule::conditional_unary_expressions => parse_conditional_unary_expressions(rule, scope)?,
        Rule::conversion_unary_expressions => parse_conversion_unary_expressions(rule, scope)?,
        Rule::string_unary_expressions => parse_string_unary_expressions(rule, scope)?,
        Rule::parse_unary_expressions => parse_parse_unary_expressions(rule, scope)?,
        Rule::array_unary_expressions => parse_array_unary_expressions(rule, scope)?,
        Rule::math_unary_expressions => parse_math_unary_expressions(rule, scope)?,
        Rule::temporal_unary_expressions => parse_temporal_unary_expressions(rule, scope)?,
        Rule::logical_unary_expressions => parse_logical_unary_expressions(rule, scope)?,
        Rule::accessor_expression => {
            // Note: When used as a scalar expression it is valid for an
            // accessor to fold into a static at the root so
            // allow_root_scalar=true is passed here. Example: iff([logical],
            // [scalar], [scalar]) evaluated as iff([logical],
            // accessor(some_constant1), accessor(some_constant2)) can safely
            // fold to iff([logical], String("constant1"), String("constant2")).
            parse_accessor_expression(rule, scope, true)?
        }
        Rule::scalar_expression => parse_scalar_expression(rule, scope)?,
        _ => panic!("Unexpected rule in scalar_unary_expression: {rule}"),
    })
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
                "tostring(\"hello\")",
                "tostring(42)",
            ],
            &["!"],
        );
    }

    #[test]
    fn test_parse_scalar_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

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

        run_test_success(
            "tostring(42)",
            ScalarExpression::Convert(ConvertScalarExpression::String(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 42),
                    )),
                ),
            )),
        );

        run_test_success(
            "tostring(true)",
            ScalarExpression::Convert(ConvertScalarExpression::String(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                ),
            )),
        );

        run_test_success(
            "tostring(false)",
            ScalarExpression::Convert(ConvertScalarExpression::String(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), false),
                    )),
                ),
            )),
        );

        run_test_success(
            "tostring(bool(null))",
            ScalarExpression::Convert(ConvertScalarExpression::String(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                ),
            )),
        );

        run_test_success(
            "tostring(\"hello\")",
            ScalarExpression::Convert(ConvertScalarExpression::String(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                    )),
                ),
            )),
        );

        run_test_success(
            "tostring(4.44)",
            ScalarExpression::Convert(ConvertScalarExpression::String(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 4.44),
                    )),
                ),
            )),
        );

        run_test_success(
            "tostring(variable)",
            ScalarExpression::Convert(ConvertScalarExpression::String(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "variable",
                            )),
                        )]),
                    )),
                ),
            )),
        );

        run_test_success(
            "tostring((42))",
            ScalarExpression::Convert(ConvertScalarExpression::String(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 42),
                    )),
                ),
            )),
        );

        run_test_success(
            "tostring(datetime(6/9/2025))",
            ScalarExpression::Convert(ConvertScalarExpression::String(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            create_utc(2025, 6, 9, 0, 0, 0, 0),
                        ),
                    )),
                ),
            )),
        );

        // Test toint expressions
        run_test_success(
            "toint(42)",
            ScalarExpression::Convert(ConvertScalarExpression::Integer(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 42),
                    )),
                ),
            )),
        );

        run_test_success(
            "toint(true)",
            ScalarExpression::Convert(ConvertScalarExpression::Integer(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                ),
            )),
        );

        run_test_success(
            "toint(4.44)",
            ScalarExpression::Convert(ConvertScalarExpression::Integer(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 4.44),
                    )),
                ),
            )),
        );

        // Test tobool expressions
        run_test_success(
            "tobool(1)",
            ScalarExpression::Convert(ConvertScalarExpression::Boolean(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                ),
            )),
        );

        run_test_success(
            "tobool(true)",
            ScalarExpression::Convert(ConvertScalarExpression::Boolean(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                ),
            )),
        );

        run_test_success(
            "tobool(4.44)",
            ScalarExpression::Convert(ConvertScalarExpression::Boolean(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 4.44),
                    )),
                ),
            )),
        );

        // Test tofloat expressions
        run_test_success(
            "tofloat(42)",
            ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 42),
                    )),
                ),
            )),
        );

        run_test_success(
            "tofloat(true)",
            ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                ),
            )),
        );

        run_test_success(
            "tofloat(4.44)",
            ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 4.44),
                    )),
                ),
            )),
        );

        // Test tolong expressions
        run_test_success(
            "tolong(42)",
            ScalarExpression::Convert(ConvertScalarExpression::Integer(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 42),
                    )),
                ),
            )),
        );

        run_test_success(
            "tolong(true)",
            ScalarExpression::Convert(ConvertScalarExpression::Integer(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                ),
            )),
        );

        run_test_success(
            "tolong(4.44)",
            ScalarExpression::Convert(ConvertScalarExpression::Integer(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 4.44),
                    )),
                ),
            )),
        );

        // Test toreal expressions
        run_test_success(
            "toreal(42)",
            ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 42),
                    )),
                ),
            )),
        );

        run_test_success(
            "toreal(true)",
            ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                ),
            )),
        );

        run_test_success(
            "toreal(4.44)",
            ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 4.44),
                    )),
                ),
            )),
        );

        // Test todouble expressions
        run_test_success(
            "todouble(42)",
            ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 42),
                    )),
                ),
            )),
        );

        run_test_success(
            "todouble(true)",
            ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                ),
            )),
        );

        run_test_success(
            "todouble(4.44)",
            ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 4.44),
                    )),
                ),
            )),
        );
    }

    #[test]
    fn test_parse_scalar_expression_precedence() {
        let run_test = |input: &str, expected: &str| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let mut expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            println!("{expression:?}");

            let resolved_expression = expression
                .try_resolve_static(&state.get_pipeline().get_resolution_scope())
                .unwrap();

            if let Some(s) = resolved_expression {
                assert_eq!(expected, s.as_ref().to_value().to_string());
            } else {
                panic!()
            }
        };

        run_test("true or false and false", "true");
        run_test("(true or false) and false", "false");
        run_test("1 > 0 + 1", "false");
        run_test("1 > 0 + 1 == false", "true");
        run_test("1 + 0 == true", "true");
        run_test("0!=1 and 1!=0", "true");
    }
}
