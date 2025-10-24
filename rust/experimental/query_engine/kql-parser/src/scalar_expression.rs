// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, sync::LazyLock};

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
        Rule::get_type_expression => {
            let location = to_query_location(&rule);

            let scalar = parse_scalar_expression(rule.into_inner().next().unwrap(), scope)?;

            let type_map_id = match scope.get_constant_id("@type_map") {
                Some(id) => id.0,
                None => {
                    let mut type_map: HashMap<Box<str>, StaticScalarExpression> = HashMap::new();
                    for value_type in ValueType::get_value_types() {
                        let item = match value_type {
                            ValueType::Array => ("Array", "array"),
                            ValueType::Boolean => ("Boolean", "bool"),
                            ValueType::DateTime => ("DateTime", "datetime"),
                            ValueType::Double => ("Double", "real"),
                            ValueType::Integer => ("Integer", "long"),
                            ValueType::Map => ("Map", "dictionary"),
                            ValueType::Null => ("Null", "null"),
                            ValueType::Regex => ("Regex", "regex"),
                            ValueType::String => ("String", "string"),
                            ValueType::TimeSpan => ("TimeSpan", "timespan"),
                        };
                        type_map.insert(
                            item.0.into(),
                            StaticScalarExpression::String(StringScalarExpression::new(
                                location.clone(),
                                item.1,
                            )),
                        );
                    }
                    scope.push_constant(
                        "@type_map",
                        StaticScalarExpression::Map(MapScalarExpression::new(
                            location.clone(),
                            type_map,
                        )),
                    )
                }
            };

            let get_type =
                ScalarExpression::GetType(GetTypeScalarExpression::new(location.clone(), scalar));

            // Note: We register a lookup into the type map to translate expression ValueType into KQL type
            ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
                location,
                ValueType::Map,
                type_map_id,
                ValueAccessor::new_with_selectors(vec![get_type]),
            ))
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

pub(crate) fn try_resolve_identifier(
    scalar_expression: &ScalarExpression,
    scope: &dyn ParserScope,
) -> Result<Option<Vec<Box<str>>>, ParserError> {
    let r = match scalar_expression {
        ScalarExpression::Attached(a) => {
            parse_identifier_from_accessor(a.get_name().get_value(), a.get_value_accessor(), scope)
        }
        ScalarExpression::Constant(c) => {
            let name = scope
                .get_constant_name(c.get_constant_id())
                .expect("Constant not found")
                .0;

            parse_identifier_from_accessor(name.as_ref(), c.get_value_accessor(), scope)
        }
        ScalarExpression::Source(s) => {
            parse_identifier_from_accessor("source", s.get_value_accessor(), scope)
        }
        ScalarExpression::Variable(v) => {
            parse_identifier_from_accessor(v.get_name().get_value(), v.get_value_accessor(), scope)
        }
        ScalarExpression::Math(m) => match m {
            MathScalarExpression::Add(_) => Ok(None),
            MathScalarExpression::Bin(b) => try_resolve_identifier(b.get_left_expression(), scope),
            MathScalarExpression::Ceiling(u) => {
                try_resolve_identifier(u.get_value_expression(), scope)
            }
            MathScalarExpression::Divide(_) => Ok(None),
            MathScalarExpression::Floor(u) => {
                try_resolve_identifier(u.get_value_expression(), scope)
            }
            MathScalarExpression::Modulus(_) => Ok(None),
            MathScalarExpression::Multiply(_) => Ok(None),
            MathScalarExpression::Negate(_) => Ok(None),
            MathScalarExpression::Subtract(_) => Ok(None),
        },
        ScalarExpression::Convert(c) => match c {
            ConvertScalarExpression::Boolean(c) => {
                try_resolve_identifier(c.get_inner_expression(), scope)
            }
            ConvertScalarExpression::DateTime(c) => {
                try_resolve_identifier(c.get_inner_expression(), scope)
            }
            ConvertScalarExpression::Double(c) => {
                try_resolve_identifier(c.get_inner_expression(), scope)
            }
            ConvertScalarExpression::Integer(c) => {
                try_resolve_identifier(c.get_inner_expression(), scope)
            }
            ConvertScalarExpression::String(c) => {
                try_resolve_identifier(c.get_inner_expression(), scope)
            }
            ConvertScalarExpression::TimeSpan(c) => {
                try_resolve_identifier(c.get_inner_expression(), scope)
            }
        },
        ScalarExpression::Case(_) => Ok(None),
        ScalarExpression::Coalesce(_) => Ok(None),
        ScalarExpression::Collection(_) => Ok(None),
        ScalarExpression::Conditional(_) => Ok(None),
        ScalarExpression::Temporal(_) => Ok(None),
        ScalarExpression::Length(l) => {
            if let Some(mut i) = try_resolve_identifier(l.get_inner_expression(), scope)? {
                i.insert(0, "len".into());
                return Ok(Some(i));
            }

            Ok(None)
        }
        ScalarExpression::Logical(_) => Ok(None),
        ScalarExpression::Parse(_) => Ok(None),
        ScalarExpression::Slice(_) => Ok(None),
        ScalarExpression::Static(_) => Ok(None),
        ScalarExpression::Text(_) => Ok(None),
        ScalarExpression::GetType(g) => {
            if let Some(mut i) = try_resolve_identifier(g.get_value(), scope)? {
                i.insert(0, "type".into());
                return Ok(Some(i));
            }

            Ok(None)
        }
    };

    if let Ok(Some(mut identifier)) = r {
        // Note: The identifier path may contain source.[default_map_key]. We always
        // remove "source" and then remove the default_map_key if the mode is
        // enabled.
        if let Some("source") = identifier.first().map(|v| v.as_ref()) {
            identifier.remove(0);

            if let Some(schema) = scope.get_source_schema()
                && let Some((key, _)) = schema.get_default_map()
                && let Some(first) = identifier.first().map(|v| v.as_ref())
                && key == first
            {
                identifier.remove(0);
            }
        }
        return Ok(Some(identifier));
    }

    r
}

fn parse_identifier_from_accessor(
    root: &str,
    value_accessor: &ValueAccessor,
    scope: &dyn ParserScope,
) -> Result<Option<Vec<Box<str>>>, ParserError> {
    let mut identifier: Vec<Box<str>> = vec![root.into()];
    for selector in value_accessor.get_selectors() {
        match selector {
            ScalarExpression::Static(s) => match s.to_value() {
                Value::String(s) => {
                    identifier.push(s.get_value().into());
                }
                Value::Integer(i) => {
                    identifier.push(format!("{}", i.get_value()).into());
                }
                _ => {
                    return Ok(None);
                }
            },
            ScalarExpression::Constant(c) => {
                let name = scope
                    .get_constant_name(c.get_constant_id())
                    .expect("Constant not found")
                    .0;

                if let Some(mut i) =
                    parse_identifier_from_accessor(name.as_ref(), c.get_value_accessor(), scope)?
                {
                    identifier.append(&mut i);
                } else {
                    return Ok(None);
                }
            }
            _ => {
                return Ok(None);
            }
        }
    }
    Ok(Some(identifier))
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

            let pipeline = state.get_pipeline();

            let resolved_expression = expression
                .try_resolve_static(&pipeline.get_resolution_scope())
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

    #[test]
    fn test_parse_get_type_scalar_expression() {
        let run_test_success = |input: &str, expected: &str| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let mut scalar = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            let pipeline = state.get_pipeline();

            let actual = scalar
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();

            assert_eq!(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    expected
                ))
                .to_value(),
                actual.unwrap().as_ref().to_value()
            );
        };

        run_test_success("gettype(dynamic([0, 1, 2]))", "array");

        run_test_success("gettype(true)", "bool");

        run_test_success("gettype(datetime(10/21/2025))", "datetime");

        run_test_success("gettype(real(1.18))", "real");

        run_test_success("gettype(18)", "long");

        run_test_success("gettype(dynamic({'key1':1}))", "dictionary");

        run_test_success("gettype(int(null))", "null");

        run_test_success("gettype(parse_regex('.*'))", "regex");

        run_test_success("gettype('hello world')", "string");

        run_test_success("gettype(1m)", "timespan");
    }
}
