use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{
    Rule, logical_expressions::parse_logical_expression,
    scalar_conditional_function_expressions::*, scalar_conversion_function_expressions::*,
    scalar_primitive_expressions::*, scalar_string_function_expressions::*,
};

pub(crate) fn parse_scalar_expression(
    scalar_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let scalar_rule = scalar_expression_rule.into_inner().next().unwrap();

    let scalar = match scalar_rule.as_rule() {
        Rule::arithmetic_expression => parse_arithmetic_expression(scalar_rule, state)?,
        Rule::null_literal => ScalarExpression::Static(parse_standard_null_literal(scalar_rule)),
        Rule::real_expression => ScalarExpression::Static(parse_real_expression(scalar_rule)?),
        Rule::datetime_expression => {
            ScalarExpression::Static(parse_datetime_expression(scalar_rule)?)
        }
        Rule::conditional_expression => parse_conditional_expression(scalar_rule, state)?,
        Rule::tostring_expression => parse_tostring_expression(scalar_rule, state)?,
        Rule::toint_expression => parse_toint_expression(scalar_rule, state)?,
        Rule::tobool_expression => parse_tobool_expression(scalar_rule, state)?,
        Rule::tofloat_expression => parse_tofloat_expression(scalar_rule, state)?,
        Rule::tolong_expression => parse_tolong_expression(scalar_rule, state)?,
        Rule::toreal_expression => parse_toreal_expression(scalar_rule, state)?,
        Rule::todouble_expression => parse_todouble_expression(scalar_rule, state)?,
        Rule::todatetime_expression => parse_todatetime_expression(scalar_rule, state)?,
        Rule::strlen_expression => parse_strlen_expression(scalar_rule, state)?,
        Rule::replace_string_expression => parse_replace_string_expression(scalar_rule, state)?,
        Rule::substring_expression => parse_substring_expression(scalar_rule, state)?,
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

fn parse_arithmetic_expression(
    arithmetic_expr_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&arithmetic_expr_rule);
    let mut inner_rules = arithmetic_expr_rule.into_inner();

    // Check for unary operator
    let first = inner_rules.next().unwrap();
    let (has_unary_minus, mut current_expr) = match first.as_rule() {
        Rule::minus_token => {
            let factor = inner_rules.next().unwrap();
            (true, parse_arithmetic_factor(factor, state)?)
        }
        Rule::plus_token => {
            let factor = inner_rules.next().unwrap();
            (false, parse_arithmetic_factor(factor, state)?)
        }
        Rule::arithmetic_factor => (false, parse_arithmetic_factor(first, state)?),
        _ => panic!("Unexpected rule in arithmetic_expression: {first}"),
    };

    // Apply unary minus if present
    if has_unary_minus {
        current_expr = ScalarExpression::Math(MathScalarExpression::Multiply(
            BinaryMathmaticalScalarExpression::new(
                query_location.clone(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(query_location.clone(), -1),
                )),
                current_expr,
            ),
        ));
    }

    // Process remaining addition/subtraction operations
    while let Some(op_rule) = inner_rules.next() {
        let right = parse_arithmetic_factor(inner_rules.next().unwrap(), state)?;

        current_expr = match op_rule.as_rule() {
            Rule::plus_token => ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathmaticalScalarExpression::new(query_location.clone(), current_expr, right),
            )),
            Rule::minus_token => ScalarExpression::Math(MathScalarExpression::Subtract(
                BinaryMathmaticalScalarExpression::new(query_location.clone(), current_expr, right),
            )),
            _ => panic!("Unexpected operator in arithmetic_expression: {op_rule}"),
        };
    }

    Ok(current_expr)
}

fn parse_arithmetic_factor(
    factor_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&factor_rule);
    let mut inner_rules = factor_rule.into_inner();
    let mut current_expr = parse_arithmetic_atom(inner_rules.next().unwrap(), state)?;

    // Process multiplication/division/modulo operations
    while let Some(op_rule) = inner_rules.next() {
        let right = parse_arithmetic_atom(inner_rules.next().unwrap(), state)?;

        current_expr = match op_rule.as_rule() {
            Rule::multiply_token => ScalarExpression::Math(MathScalarExpression::Multiply(
                BinaryMathmaticalScalarExpression::new(query_location.clone(), current_expr, right),
            )),
            Rule::divide_token => ScalarExpression::Math(MathScalarExpression::Divide(
                BinaryMathmaticalScalarExpression::new(query_location.clone(), current_expr, right),
            )),
            Rule::modulo_token => ScalarExpression::Math(MathScalarExpression::Modulus(
                BinaryMathmaticalScalarExpression::new(query_location.clone(), current_expr, right),
            )),
            _ => panic!("Unexpected operator in arithmetic_factor: {op_rule}"),
        };
    }

    Ok(current_expr)
}

fn parse_arithmetic_atom(
    atom_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    // Reuse the existing parse_scalar_expression for all base cases
    parse_scalar_expression(atom_rule, state)
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
    fn test_parse_arithmetic_expressions() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);
            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();
            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        // Test simple addition
        run_test_success(
            "(5 + 3)",
            ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                    )),
                ),
            )),
        );

        // Test simple subtraction
        run_test_success(
            "(10 - 4)",
            ScalarExpression::Math(MathScalarExpression::Subtract(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                    )),
                ),
            )),
        );

        // Test simple multiplication
        run_test_success(
            "(6 * 7)",
            ScalarExpression::Math(MathScalarExpression::Multiply(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 6),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 7),
                    )),
                ),
            )),
        );

        // Test simple division
        run_test_success(
            "(20 / 4)",
            ScalarExpression::Math(MathScalarExpression::Divide(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 20),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                    )),
                ),
            )),
        );

        // Test simple modulo
        run_test_success(
            "(10 % 3)",
            ScalarExpression::Math(MathScalarExpression::Modulus(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                    )),
                ),
            )),
        );

        // Test operator precedence: multiplication before addition
        run_test_success(
            "(2 + 3 * 4)",
            ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                    )),
                    ScalarExpression::Math(MathScalarExpression::Multiply(
                        BinaryMathmaticalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                            )),
                        ),
                    )),
                ),
            )),
        );

        // Test parentheses override precedence
        run_test_success(
            "((2 + 3) * 4)",
            ScalarExpression::Math(MathScalarExpression::Multiply(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Math(MathScalarExpression::Add(
                        BinaryMathmaticalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                            )),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                    )),
                ),
            )),
        );

        // Test unary minus
        run_test_success(
            "(-5)",
            ScalarExpression::Math(MathScalarExpression::Multiply(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                    )),
                ),
            )),
        );

        // Test unary plus (should have no effect)
        run_test_success(
            "(+5)",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
            )),
        );

        // Test complex expression with multiple operators
        run_test_success(
            "(10 + 20 / 4 - 3 * 2)",
            ScalarExpression::Math(MathScalarExpression::Subtract(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Math(MathScalarExpression::Add(
                        BinaryMathmaticalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                            )),
                            ScalarExpression::Math(MathScalarExpression::Divide(
                                BinaryMathmaticalScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    ScalarExpression::Static(StaticScalarExpression::Integer(
                                        IntegerScalarExpression::new(QueryLocation::new_fake(), 20),
                                    )),
                                    ScalarExpression::Static(StaticScalarExpression::Integer(
                                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                                    )),
                                ),
                            )),
                        ),
                    )),
                    ScalarExpression::Math(MathScalarExpression::Multiply(
                        BinaryMathmaticalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                            )),
                        ),
                    )),
                ),
            )),
        );

        // Test arithmetic with doubles
        run_test_success(
            "(3.14 + 2.86)",
            ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 3.14),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 2.86),
                    )),
                ),
            )),
        );

        // Test arithmetic with variables
        run_test_success(
            "(x + y)",
            ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "x",
                            )),
                        )]),
                    )),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "y",
                            )),
                        )]),
                    )),
                ),
            )),
        );

        // Test nested parentheses
        run_test_success(
            "((2 + 3) * (4 + 5))",
            ScalarExpression::Math(MathScalarExpression::Multiply(
                BinaryMathmaticalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Math(MathScalarExpression::Add(
                        BinaryMathmaticalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                            )),
                        ),
                    )),
                    ScalarExpression::Math(MathScalarExpression::Add(
                        BinaryMathmaticalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                            )),
                        ),
                    )),
                ),
            )),
        );
    }

    #[test]
    fn test_pest_parse_arithmetic_expressions() {
        // Add arithmetic expressions to the pest parser tests
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::scalar_expression,
            &[
                // Basic arithmetic
                "(1 + 2)",
                "(10 - 5)",
                "(3 * 4)",
                "(20 / 4)",
                "(10 % 3)",
                // With doubles
                "(3.14 + 2.86)",
                "(10.5 * 2.0)",
                // With variables
                "(x + y)",
                "(a * b + c)",
                // Operator precedence
                "(2 + 3 * 4)",
                "(10 - 6 / 2)",
                "(5 + 10 % 3)",
                // Parentheses
                "((2 + 3) * 4)",
                "(5 * (10 - 8))",
                "(((1 + 2) * 3) + 4)",
                // Unary operators
                "(-5)",
                "(+10)",
                "(-x)",
                "(+y)",
                "(-(5 + 3))",
                // Complex expressions
                "(a + b * c - d / e % f)",
                "((a + b) * (c - d) / (e + f))",
                "(1 + 2 * 3 - 4 / 2 + 5 % 3)",
            ],
            &[
                // Invalid expressions
                "(+ +)", "(5 +)", "(* 3)", "(10 /)", "(% 5)",
            ],
        );
    }

    #[test]
    fn test_arithmetic_constant_evaluation() {
        let evaluate_constant = |input: &str| -> Option<i64> {
            println!("Testing: {input}");

            let state = ParserState::new(input);
            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();
            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            // Create a dummy pipeline for static resolution
            let pipeline = PipelineExpression::default();

            if let Ok(Some(resolved)) = expression.try_resolve_static(&pipeline) {
                match resolved.to_value() {
                    Value::Integer(i) => Some(i.get_value()),
                    _ => None,
                }
            } else {
                None
            }
        };
        // Test basic arithmetic
        assert_eq!(evaluate_constant("(2 + 3)"), Some(5));
        assert_eq!(evaluate_constant("(10 - 4)"), Some(6));
        assert_eq!(evaluate_constant("(6 * 7)"), Some(42));
        assert_eq!(evaluate_constant("(10 % 3)"), Some(1));
        assert_eq!(evaluate_constant("(10 / 2)"), Some(5));

        // Test operator precedence
        assert_eq!(evaluate_constant("(2 + 3 * 4)"), Some(14)); // not 20
        assert_eq!(evaluate_constant("((2 + 3) * 4)"), Some(20));
        assert_eq!(evaluate_constant("(10 - 6 * 2)"), Some(-2));
        assert_eq!(evaluate_constant("((2 + 3) * (4 + 5))"), Some(45));
        assert_eq!(evaluate_constant("(1 + 2 * 3 - 4 / 2 + 5 % 3)"), Some(7));
        assert_eq!(evaluate_constant("(1 + 2 * 3 - 5 / 2 + 5 % 3)"), Some(7));
        assert_eq!(evaluate_constant("(10 / 3)"), Some(3));

        // Test unary operators
        assert_eq!(evaluate_constant("(-5)"), Some(-5));
        assert_eq!(evaluate_constant("(-5 + 3)"), Some(-2));
        assert_eq!(evaluate_constant("(-(5 + 3))"), Some(-8));
    }

    #[test]
    fn test_arithmetic_double_evaluation() {
        let evaluate_constant_double = |input: &str| -> Option<f64> {
            println!("Testing: {input}");

            let state = ParserState::new(input);
            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();
            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            let pipeline = PipelineExpression::default();

            if let Ok(Some(resolved)) = expression.try_resolve_static(&pipeline) {
                match resolved.to_value() {
                    Value::Double(d) => Some(d.get_value()),
                    _ => None,
                }
            } else {
                None
            }
        };

        // Division always returns double
        assert_eq!(evaluate_constant_double("(10.0 / 3)"), Some(10.0 / 3.0));
        assert_eq!(
            evaluate_constant_double("(1 + 2 * 3 - 5.0 / 2 + 5 % 3)"),
            Some(6.5)
        );

        // Double arithmetic
        assert_eq!(evaluate_constant_double("(3.14 + 2.86)"), Some(6.0));
        assert_eq!(evaluate_constant_double("(10.5 * 2.0)"), Some(21.0));
    }
}
