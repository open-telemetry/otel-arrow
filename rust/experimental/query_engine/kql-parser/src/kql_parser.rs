use std::collections::HashSet;

use data_engine_expressions::*;
use pest::iterators::Pair;

use pest_derive::Parser;

use crate::Error;

#[derive(Parser)]
#[grammar = "kql.pest"]
#[allow(dead_code)]
pub(crate) struct KqlParser;

#[allow(dead_code)]
pub(crate) struct KqlParserState {
    default_source_map_key: Option<Box<str>>,
    attached_data_names: HashSet<Box<str>>,
    variable_names: HashSet<Box<str>>,
}

impl KqlParserState {
    #[allow(dead_code)]
    pub fn new() -> KqlParserState {
        Self {
            default_source_map_key: None,
            attached_data_names: HashSet::new(),
            variable_names: HashSet::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_default_source_map_key_name(mut self, name: &str) -> KqlParserState {
        if !name.is_empty() {
            self.default_source_map_key = Some(name.into());
        }

        self
    }

    #[allow(dead_code)]
    pub fn with_attached_data_names(mut self, names: &[&str]) -> KqlParserState {
        for name in names {
            self.attached_data_names.insert((*name).into());
        }

        self
    }

    #[allow(dead_code)]
    pub fn push_variable_name(&mut self, name: &str) {
        self.variable_names.insert(name.into());
    }
}

/// The goal of this code is to unescape string literal values as they come in
/// when parsed from pest:
/// * `'some \\' string'` -> `some ' string`
/// * `\"some \\\" string\"` -> `some \" string`
#[allow(dead_code)]
pub(crate) fn parse_string_literal(string_literal_rule: Pair<Rule>) -> String {
    let raw_string = string_literal_rule.as_str();
    let mut chars = raw_string.chars();
    let mut s = String::with_capacity(raw_string.len());
    let mut position = 1;
    let mut last_char = '\0';

    let mut c = chars.next();
    loop {
        debug_assert!(c.is_some());

        let mut current_char = c.unwrap();
        let mut skip_push = false;

        if position == 1 || current_char == '\\' {
            skip_push = true;
        } else if last_char == '\\' {
            match current_char {
                '"' => current_char = '"',
                '\'' => current_char = '\'',
                '\\' => current_char = '\\',
                'n' => current_char = '\n',
                'r' => current_char = '\r',
                't' => current_char = '\t',
                _ => panic!("Unexpected escape character"),
            }
        }

        last_char = current_char;
        position += 1;

        c = chars.next();
        if c.is_none() {
            break;
        }

        if !skip_push {
            s.push(current_char);
        }
    }

    s
}

#[allow(dead_code)]
pub(crate) fn parse_integer_literal(integer_literal_rule: Pair<Rule>) -> Result<i64, Error> {
    let r = integer_literal_rule.as_str().parse::<i64>();
    if r.is_err() {
        return Err(Error::SyntaxError(
            to_query_location(&integer_literal_rule),
            "Integer value could not be parsed".into(),
        ));
    }

    Ok(r.unwrap())
}

/// The goal of this code is to translate a KQL dynamic access
/// (https://learn.microsoft.com/kusto/query/scalar-data-types/dynamic)
/// expression into a valid query scalar expression.
///
/// * `source.attributes.array[0]` -> `Source(MapKey("attributes"),
///   MapKey("array"), ArrayIndex(0))`
/// * `scope.name` -> `Attached("scope", MapKey("name"))`
/// * `var_name` -> `Variable("var_name")`
/// * `source.attributes[-var_name]` -> `Source(MapKey("attributes"),
///   Scalar(Negate(Variable("var_name"))))`
///
/// Special cases:
///
/// * If the root identifier is not `source` or something contained in either
///   attached names nor variables names we assume the user wants some default
///   behavior. This is controlled by `default_source_map_key` on
///   [`KqlParserState`].
///
///   `unknown` -> `Source(MapKey("attributes"), MapKey("unknown"))`
#[allow(dead_code)]
pub(crate) fn parse_accessor_expression(
    accessor_expression_rule: Pair<Rule>,
    state: &KqlParserState,
) -> Result<ScalarExpression, Error> {
    let query_location = to_query_location(&accessor_expression_rule);

    println!("{:?}", query_location);

    let mut accessor_rules = accessor_expression_rule.into_inner();

    let root_accessor_identity = accessor_rules.next().unwrap().as_str();

    let mut value_accessor = ValueAccessor::new();

    let mut negate_location = None;

    loop {
        let accessor = accessor_rules.next();
        if accessor.is_none() {
            break;
        }

        let pair = accessor.unwrap();
        match pair.as_rule() {
            Rule::integer_literal => {
                let location = to_query_location(&pair);

                let i = parse_integer_literal(pair)?;
                if i < i32::MIN as i64 || i > i32::MAX as i64 {
                    return Err(Error::SyntaxError(
                        location,
                        "Integer value for array index is too large to fit into a 32bit value"
                            .into(),
                    ));
                }
                value_accessor.push_selector(ValueSelector::ArrayIndex(
                    IntegerScalarExpression::new(location, i),
                ));
            }
            Rule::string_literal => {
                value_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
                    to_query_location(&pair),
                    parse_string_literal(pair).as_str(),
                )));
            }
            Rule::identifier_literal => {
                value_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
                    to_query_location(&pair),
                    pair.as_str(),
                )));
            }
            Rule::accessor_expression => {
                let expression = parse_accessor_expression(pair, state)?;

                if negate_location.is_some() {
                    value_accessor.push_selector(ValueSelector::ScalarExpression(
                        ScalarExpression::Negate(NegateScalarExpression::new(
                            negate_location.unwrap(),
                            expression,
                        )),
                    ));
                    negate_location = None;
                } else {
                    value_accessor.push_selector(ValueSelector::ScalarExpression(expression));
                }
            }
            Rule::minus_token => {
                negate_location = Some(to_query_location(&pair));
            }
            _ => panic!("Unexpected rule in accessor_expression: {}", pair),
        }
    }

    if root_accessor_identity == "source" {
        Ok(ScalarExpression::Source(SourceScalarExpression::new(
            query_location,
            value_accessor,
        )))
    } else if state.attached_data_names.contains(root_accessor_identity) {
        return Ok(ScalarExpression::Attached(AttachedScalarExpression::new(
            query_location,
            root_accessor_identity,
            value_accessor,
        )));
    } else if state.variable_names.contains(root_accessor_identity) {
        return Ok(ScalarExpression::Variable(VariableScalarExpression::new(
            query_location,
            root_accessor_identity,
            value_accessor,
        )));
    } else {
        value_accessor.insert_selector(
            0,
            ValueSelector::MapKey(StringScalarExpression::new(
                query_location.clone(),
                root_accessor_identity,
            )),
        );

        if state.default_source_map_key.is_some() {
            value_accessor.insert_selector(
                0,
                ValueSelector::MapKey(StringScalarExpression::new(
                    query_location.clone(),
                    state.default_source_map_key.as_ref().unwrap(),
                )),
            );
        }

        return Ok(ScalarExpression::Source(SourceScalarExpression::new(
            query_location,
            value_accessor,
        )));
    }
}

pub(crate) fn to_query_location(rule: &Pair<Rule>) -> QueryLocation {
    let s = rule.as_span();
    let (line_number, column_number) = rule.line_col();
    QueryLocation::new(s.start(), s.end(), line_number, column_number)
}

#[cfg(test)]
mod pest_tests {
    use std::mem::discriminant;

    use super::*;
    use pest::{Parser, iterators::Pairs};

    #[test]
    fn test_integer_literal() {
        assert!(KqlParser::parse(Rule::integer_literal, "123").is_ok());
        assert!(KqlParser::parse(Rule::integer_literal, "-123").is_ok());
        assert!(KqlParser::parse(Rule::integer_literal, ".53").is_err());
        assert!(KqlParser::parse(Rule::integer_literal, "abc").is_err());
    }

    #[test]
    fn test_string_literal() {
        assert!(KqlParser::parse(Rule::string_literal, "\"hello\"").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, "\"he\\\"llo\"").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, "'hello'").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, "'he\"llo'").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, "'he\\'llo'").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, r#""hello"#).is_err());
        assert!(KqlParser::parse(Rule::string_literal, r#"hello""#).is_err());
        assert!(KqlParser::parse(Rule::string_literal, r#""""#).is_ok());
    }

    #[test]
    fn test_identifier_literal() {
        assert!(KqlParser::parse(Rule::identifier_literal, "Abc").is_ok());
        assert!(KqlParser::parse(Rule::identifier_literal, "abc_123").is_ok());
        assert!(KqlParser::parse(Rule::identifier_literal, "_abc").is_ok());
    }

    #[test]
    fn test_accessor_expression() {
        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "Abc").unwrap(),
            &[
                (Rule::accessor_expression, "Abc"),
                (Rule::identifier_literal, "Abc"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "abc_123").unwrap(),
            &[
                (Rule::accessor_expression, "abc_123"),
                (Rule::identifier_literal, "abc_123"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "_abc").unwrap(),
            &[
                (Rule::accessor_expression, "_abc"),
                (Rule::identifier_literal, "_abc"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "array[0]").unwrap(),
            &[
                (Rule::accessor_expression, "array[0]"),
                (Rule::identifier_literal, "array"),
                (Rule::integer_literal, "0"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "array[-1]").unwrap(),
            &[
                (Rule::accessor_expression, "array[-1]"),
                (Rule::identifier_literal, "array"),
                (Rule::integer_literal, "-1"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "abc.name").unwrap(),
            &[
                (Rule::accessor_expression, "abc.name"),
                (Rule::identifier_literal, "abc"),
                (Rule::identifier_literal, "name"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "abc.name1.name2").unwrap(),
            &[
                (Rule::accessor_expression, "abc.name1.name2"),
                (Rule::identifier_literal, "abc"),
                (Rule::identifier_literal, "name1"),
                (Rule::identifier_literal, "name2"),
            ],
        );

        validate_rule(
            KqlParser::parse(
                Rule::accessor_expression,
                "abc['~name-!'].name1[0][-sub].name2",
            )
            .unwrap(),
            &[
                (
                    Rule::accessor_expression,
                    "abc['~name-!'].name1[0][-sub].name2",
                ),
                (Rule::identifier_literal, "abc"),
                (Rule::string_literal, "'~name-!'"),
                (Rule::identifier_literal, "name1"),
                (Rule::integer_literal, "0"),
                (Rule::minus_token, "-"),
                (Rule::accessor_expression, "sub"),
                (Rule::identifier_literal, "sub"),
                (Rule::identifier_literal, "name2"),
            ],
        );

        assert!(KqlParser::parse(Rule::accessor_expression, "123").is_err());
        assert!(KqlParser::parse(Rule::accessor_expression, "+name").is_err());
        assert!(KqlParser::parse(Rule::accessor_expression, "-name").is_err());
        assert!(KqlParser::parse(Rule::accessor_expression, "~name").is_err());
        assert!(KqlParser::parse(Rule::accessor_expression, ".name").is_err());
    }

    fn validate_rule(parsed: Pairs<'_, Rule>, expected: &[(Rule, &str)]) {
        let flat = parsed.flatten();

        assert_eq!(flat.len(), expected.len());

        for (index, rule) in flat.enumerate() {
            let expected = expected.get(index).unwrap();

            assert!(discriminant(&rule.as_rule()) == discriminant(&expected.0));
            assert_eq!(rule.as_str(), expected.1);
        }
    }
}

#[cfg(test)]
mod parse_tests {
    use core::panic;

    use super::*;
    use pest::Parser;

    #[test]
    fn test_parse_positive_integer_literal() {
        let mut result = KqlParser::parse(Rule::integer_literal, "1").unwrap();

        let i = parse_integer_literal(result.next().unwrap());

        assert!(i.is_ok());
        assert_eq!(1, i.unwrap());
    }

    #[test]
    fn test_parse_negative_integer_literal() {
        let mut result = KqlParser::parse(Rule::integer_literal, "-1").unwrap();

        let i = parse_integer_literal(result.next().unwrap());

        assert!(i.is_ok());
        assert_eq!(-1, i.unwrap());
    }

    #[test]
    fn test_parse_invalid_integer_literal() {
        let input = format!("{}", i64::MAX as i128 + 1);
        let result = KqlParser::parse(Rule::integer_literal, input.as_str());

        assert!(result.is_ok());

        let mut pairs = result.unwrap();
        let i = parse_integer_literal(pairs.next().unwrap());

        assert!(i.is_err());

        if let Error::SyntaxError(_, _) = i.unwrap_err() {
        } else {
            panic!("Expected SyntaxError");
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let run_test = |input: &str, expected: &str| {
            let mut result = KqlParser::parse(Rule::string_literal, input).unwrap();

            let actual = parse_string_literal(result.next().unwrap());

            assert_eq!(expected, actual);
        };

        run_test("\"Hello world\"", "Hello world");
        run_test("\"Hello \\\" world\"", "Hello \" world");
        run_test("'Hello world'", "Hello world");
        run_test("'Hello \" world'", "Hello \" world");
        run_test("'Hello \\' world'", "Hello ' world");
    }

    #[test]
    fn test_parse_accessor_expression_from_source() {
        let mut result =
            KqlParser::parse(Rule::accessor_expression, "source.subkey['array'][0]").unwrap();

        let expression =
            parse_accessor_expression(result.next().unwrap(), &KqlParserState::new()).unwrap();

        if let ScalarExpression::Source(path) = expression {
            assert_eq!(
                &[
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(7, 13, 1, 8),
                        "subkey"
                    )),
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(14, 21, 1, 15),
                        "array"
                    )),
                    ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                        QueryLocation::new(23, 24, 1, 24),
                        0
                    ))
                ]
                .to_vec(),
                path.get_selectors()
            );
        } else {
            panic!("Expected SourceScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_implicit_source() {
        let mut result =
            KqlParser::parse(Rule::accessor_expression, "subkey[var][-neg_attr]").unwrap();

        let mut state = KqlParserState::new();

        state.push_variable_name("var");

        let expression = parse_accessor_expression(result.next().unwrap(), &state).unwrap();

        let mut v = ValueAccessor::new();
        v.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new(13, 21, 1, 14),
            "neg_attr",
        )));

        if let ScalarExpression::Source(s) = expression {
            assert_eq!(
                &[
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(0, 22, 1, 1),
                        "subkey"
                    )),
                    ValueSelector::ScalarExpression(ScalarExpression::Variable(
                        VariableScalarExpression::new(
                            QueryLocation::new(7, 10, 1, 8),
                            "var",
                            ValueAccessor::new()
                        )
                    )),
                    ValueSelector::ScalarExpression(ScalarExpression::Negate(
                        NegateScalarExpression::new(
                            QueryLocation::new(12, 13, 1, 13),
                            ScalarExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new(13, 21, 1, 14),
                                v
                            ))
                        )
                    ))
                ]
                .to_vec(),
                s.get_selectors()
            );
        } else {
            panic!("Expected SourceScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_implicit_source_and_default_map() {
        let mut result = KqlParser::parse(Rule::accessor_expression, "subkey").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &KqlParserState::new().with_default_source_map_key_name("attributes"),
        )
        .unwrap();

        let mut v = ValueAccessor::new();
        v.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new(13, 21, 1, 14),
            "neg_attr",
        )));

        if let ScalarExpression::Source(s) = expression {
            assert_eq!(
                &[
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(0, 6, 1, 1),
                        "attributes"
                    )),
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(0, 6, 1, 1),
                        "subkey"
                    ))
                ]
                .to_vec(),
                s.get_selectors()
            );
        } else {
            panic!("Expected SourceScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_from_attached() {
        let mut result =
            KqlParser::parse(Rule::accessor_expression, "resource['~at\\'tr~']").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &KqlParserState::new().with_attached_data_names(&["resource"]),
        )
        .unwrap();

        if let ScalarExpression::Attached(a) = expression {
            assert_eq!("resource", a.get_name());
            assert_eq!(
                &[ValueSelector::MapKey(StringScalarExpression::new(
                    QueryLocation::new(9, 19, 1, 10),
                    "~at'tr~"
                ))]
                .to_vec(),
                a.get_selectors()
            );
        } else {
            panic!("Expected AttachedScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_from_variable() {
        let mut result = KqlParser::parse(Rule::accessor_expression, "a[-1]").unwrap();

        let mut state = KqlParserState::new();

        state.push_variable_name("a");

        let expression = parse_accessor_expression(result.next().unwrap(), &state).unwrap();

        if let ScalarExpression::Variable(v) = expression {
            assert_eq!("a", v.get_name());
            assert_eq!(
                &[ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                    QueryLocation::new(2, 4, 1, 3),
                    -1
                ))]
                .to_vec(),
                v.get_selectors()
            );
        } else {
            panic!("Expected VariableScalarExpression");
        }
    }
}
