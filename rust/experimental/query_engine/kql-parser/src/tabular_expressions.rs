// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;
use regex::Regex;

use crate::{
    Rule,
    aggregate_expressions::parse_aggregate_assignment_expression,
    logical_expressions::parse_logical_expression,
    scalar_expression::parse_scalar_expression,
    scalar_primitive_expressions::{parse_accessor_expression, parse_string_literal},
    shared_expressions::parse_assignment_expression,
};

pub(crate) fn parse_extend_expression(
    extend_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<Vec<TransformExpression>, ParserError> {
    let extend_rules = extend_expression_rule.into_inner();

    let mut set_expressions = Vec::new();

    for rule in extend_rules {
        match rule.as_rule() {
            Rule::assignment_expression => {
                let assignment_expression = parse_assignment_expression(rule, state)?;

                if let TransformExpression::Set(s) = &assignment_expression {
                    match s.get_destination() {
                        MutableValueExpression::Source(_) => {}
                        MutableValueExpression::Variable(v) => {
                            let location = v.get_query_location();

                            return Err(ParserError::SyntaxError(
                                location.clone(),
                                format!(
                                    "'{}' destination accessor must refer to source to be used in an extend expression",
                                    state.get_query_slice(location).trim()
                                ),
                            ));
                        }
                    }
                    set_expressions.push(assignment_expression);
                } else {
                    panic!(
                        "Unexpected transformation in extend_expression: {assignment_expression:?}"
                    )
                }
            }
            _ => panic!("Unexpected rule in extend_expression: {rule}"),
        }
    }

    Ok(set_expressions)
}

pub(crate) fn parse_project_expression(
    project_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<Vec<TransformExpression>, ParserError> {
    let query_location = to_query_location(&project_expression_rule);

    let project_rules = project_expression_rule.into_inner();

    let mut expressions = Vec::new();

    let mut reduction = MapReductionState::new();

    for rule in project_rules {
        let rule_location = to_query_location(&rule);

        match rule.as_rule() {
            Rule::assignment_expression => {
                let assignment_expression = parse_assignment_expression(rule, state)?;

                if let TransformExpression::Set(s) = &assignment_expression {
                    match s.get_destination() {
                        MutableValueExpression::Source(s) => {
                            process_map_selection_source_scalar_expression(
                                "project",
                                state,
                                s,
                                &mut reduction,
                            )?;
                        }
                        MutableValueExpression::Variable(v) => {
                            let location = v.get_query_location();

                            return Err(ParserError::SyntaxError(
                                location.clone(),
                                format!(
                                    "'{}' destination accessor must refer to source to be used in a project expression",
                                    state.get_query_slice(location).trim()
                                ),
                            ));
                        }
                    }
                    expressions.push(assignment_expression);
                } else {
                    panic!(
                        "Unexpected transformation in project_expression: {assignment_expression:?}"
                    )
                }
            }
            Rule::accessor_expression => {
                let accessor_expression = parse_accessor_expression(rule, state, true)?;

                if let ScalarExpression::Source(s) = &accessor_expression {
                    process_map_selection_source_scalar_expression(
                        "project",
                        state,
                        s,
                        &mut reduction,
                    )?;
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location.clone(),
                        format!(
                            "To be valid in a project expression '{}' should be an assignment expression or an accessor expression which refers to the source",
                            state.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            _ => panic!("Unexpected rule in project_expression: {rule}"),
        }
    }

    push_map_transformation_expression(
        "project",
        state,
        &mut expressions,
        &query_location,
        true,
        reduction,
    )?;

    Ok(expressions)
}

pub(crate) fn parse_project_keep_expression(
    project_keep_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<Vec<TransformExpression>, ParserError> {
    let query_location = to_query_location(&project_keep_expression_rule);

    let project_keep_rules = project_keep_expression_rule.into_inner();

    let mut expressions = Vec::new();

    let mut reduction = MapReductionState::new();

    for rule in project_keep_rules {
        let rule_location = to_query_location(&rule);

        match rule.as_rule() {
            Rule::identifier_or_pattern_literal => {
                if let Some(identifier_or_pattern) =
                    parse_identifier_or_pattern_literal(state, rule_location.clone(), rule)?
                {
                    match identifier_or_pattern {
                        IdentifierOrPattern::Identifier(s) => reduction.keys.push(s),
                        IdentifierOrPattern::Map { name, key } => reduction
                            .maps
                            .entry(Into::<Box<str>>::into(name.get_value()))
                            .or_default()
                            .push((name, key)),
                        IdentifierOrPattern::Pattern(r) => reduction.patterns.push(r),
                    }
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location.clone(),
                        format!(
                            "To be valid in a project-keep expression '{}' should be an accessor expression which refers to data on the source",
                            state.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            Rule::accessor_expression => {
                let accessor_expression = parse_accessor_expression(rule, state, true)?;

                if let ScalarExpression::Source(s) = &accessor_expression {
                    process_map_selection_source_scalar_expression(
                        "project-keep",
                        state,
                        s,
                        &mut reduction,
                    )?;
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location.clone(),
                        format!(
                            "To be valid in a project-keep expression '{}' should be an accessor expression which refers to the source",
                            state.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            _ => panic!("Unexpected rule in project_keep_expression: {rule}"),
        }
    }

    push_map_transformation_expression(
        "project-keep",
        state,
        &mut expressions,
        &query_location,
        true,
        reduction,
    )?;

    Ok(expressions)
}

pub(crate) fn parse_project_away_expression(
    project_away_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<Vec<TransformExpression>, ParserError> {
    let query_location = to_query_location(&project_away_expression_rule);

    let project_away_rules = project_away_expression_rule.into_inner();

    let mut expressions = Vec::new();

    let mut reduction = MapReductionState::new();

    for rule in project_away_rules {
        let rule_location = to_query_location(&rule);

        match rule.as_rule() {
            Rule::identifier_or_pattern_literal => {
                if let Some(identifier_or_pattern) =
                    parse_identifier_or_pattern_literal(state, rule_location.clone(), rule)?
                {
                    match identifier_or_pattern {
                        IdentifierOrPattern::Identifier(s) => reduction.keys.push(s),
                        IdentifierOrPattern::Map { name, key } => reduction
                            .maps
                            .entry(Into::<Box<str>>::into(name.get_value()))
                            .or_default()
                            .push((name, key)),
                        IdentifierOrPattern::Pattern(r) => reduction.patterns.push(r),
                    }
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location.clone(),
                        format!(
                            "To be valid in a project-away expression '{}' should be an accessor expression which refers to data on the source",
                            state.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            Rule::accessor_expression => {
                let accessor_expression = parse_accessor_expression(rule, state, true)?;

                if let ScalarExpression::Source(s) = &accessor_expression {
                    process_map_selection_source_scalar_expression(
                        "project-away",
                        state,
                        s,
                        &mut reduction,
                    )?;
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location.clone(),
                        format!(
                            "To be valid in a project-away expression '{}' should be an accessor expression which refers to the source",
                            state.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            _ => panic!("Unexpected rule in project_away_expression: {rule}"),
        }
    }

    push_map_transformation_expression(
        "project-away",
        state,
        &mut expressions,
        &query_location,
        false,
        reduction,
    )?;

    Ok(expressions)
}

pub(crate) fn parse_where_expression(
    where_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<DataExpression, ParserError> {
    let query_location = to_query_location(&where_expression_rule);

    let where_rule = where_expression_rule.into_inner().next().unwrap();

    let predicate = match where_rule.as_rule() {
        Rule::logical_expression => parse_logical_expression(where_rule, state)?,
        _ => panic!("Unexpected rule in where_expression: {where_rule}"),
    };

    // Note: KQL "where" describes data to retain. Query engine "discard"
    // describes data to remove. When mapping KQL "where" onto "discard" we use
    // a "Not" expression to flip the KQL logic to match.
    Ok(DataExpression::Discard(
        DiscardDataExpression::new(query_location.clone()).with_predicate(LogicalExpression::Not(
            NotLogicalExpression::new(query_location, predicate),
        )),
    ))
}

pub(crate) fn parse_parse_expression(
    parse_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<Vec<TransformExpression>, ParserError> {
    let query_location = to_query_location(&parse_expression_rule);

    let mut _kind = "simple"; // default - unused for now
    let mut _flags: Option<String> = None; // unused for now
    let mut input_expression: Option<ScalarExpression> = None;
    let mut pattern_elements = Vec::new();

    for rule in parse_expression_rule.into_inner() {
        match rule.as_rule() {
            Rule::parse_kind => {
                if let Some(kind_rule) = rule.into_inner().next() {
                    _kind = match kind_rule.as_str() {
                        "simple" | "regex" | "relaxed" => kind_rule.as_str(),
                        _ => {
                            return Err(ParserError::SyntaxError(
                                to_query_location(&kind_rule),
                                format!("Unsupported parse kind: {}", kind_rule.as_str()),
                            ));
                        }
                    };
                }
            }
            Rule::string_literal => {
                // This could be the flags parameter - extract the string value
                let rule_location = to_query_location(&rule);
                if let StaticScalarExpression::String(s) = parse_string_literal(rule) {
                    _flags = Some(s.get_value().to_string());
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location,
                        "Expected string literal for flags".to_string(),
                    ));
                }
            }
            Rule::scalar_expression => {
                input_expression = Some(parse_scalar_expression(rule, state)?);
            }
            Rule::parse_pattern => {
                for pattern_element_rule in rule.into_inner() {
                    pattern_elements.push(parse_parse_pattern_element(pattern_element_rule)?);
                }
            }
            _ => panic!("Unexpected rule in parse_expression: {rule}"),
        }
    }

    let input_expression = input_expression.ok_or_else(|| {
        ParserError::SyntaxError(
            query_location.clone(),
            "Missing input expression".to_string(),
        )
    })?;

    if pattern_elements.is_empty() {
        return Err(ParserError::SyntaxError(
            query_location,
            "Parse pattern cannot be empty".to_string(),
        ));
    }

    // Convert the pattern elements into proper parse expressions using ParseExtractScalarExpression
    let mut set_expressions = Vec::new();

    // Build the regex pattern from the pattern elements
    let mut regex_pattern = String::new();
    let mut capture_groups = Vec::new();

    for element in &pattern_elements {
        match element {
            ParsePatternElement::Wildcard => {
                regex_pattern.push_str(".*?");
            }
            ParsePatternElement::Literal(text) => {
                // Escape regex special characters in literal text
                regex_pattern.push_str(&regex::escape(text));
            }
            ParsePatternElement::Column { name, column_type } => {
                // Create a named capture group for this column
                // Use .* instead of .*? to capture greedily until the next literal or end of string
                regex_pattern.push_str(&format!("(?P<{}>.*)", name));
                capture_groups.push((name.clone(), column_type.clone()));
            }
        }
    }

    // Create text extract expressions for each column
    for (group_name, column_type) in capture_groups {
        let extract_expression = ScalarExpression::Text(TextScalarExpression::Extract(
            ExtractTextScalarExpression::new(
                query_location.clone(),
                input_expression.clone(), // The input string to parse
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(query_location.clone(), &regex_pattern),
                )), // The regex pattern with all capture groups
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(query_location.clone(), &group_name),
                )), // The specific capture group name to extract
            ),
        ));

        // Apply conversion based on column type
        let parse_expression = match column_type.as_str() {
            "int" | "long" => ScalarExpression::Convert(ConvertScalarExpression::Integer(
                ConversionScalarExpression::new(query_location.clone(), extract_expression),
            )),
            "real" | "double" => ScalarExpression::Convert(ConvertScalarExpression::Double(
                ConversionScalarExpression::new(query_location.clone(), extract_expression),
            )),
            "bool" | "boolean" => ScalarExpression::Convert(ConvertScalarExpression::Boolean(
                ConversionScalarExpression::new(query_location.clone(), extract_expression),
            )),
            "datetime" => ScalarExpression::Convert(ConvertScalarExpression::DateTime(
                ConversionScalarExpression::new(query_location.clone(), extract_expression),
            )),
            "timespan" => ScalarExpression::Convert(ConvertScalarExpression::TimeSpan(
                ConversionScalarExpression::new(query_location.clone(), extract_expression),
            )),
            "string" => extract_expression, // No conversion needed for strings
            _ => extract_expression,        // Default to no conversion
        };

        // Create the set expression that uses the parse extract expression
        // Use schema information to determine where to place the new column
        let destination = if let Some(schema) = state.get_source_schema() {
            if schema.get_schema_for_key(&group_name).is_some() {
                // If the column is defined in the schema, use it directly
                MutableValueExpression::Source(SourceScalarExpression::new(
                    query_location.clone(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            query_location.clone(),
                            &group_name,
                        )),
                    )]),
                ))
            } else if let Some(default_map_key) = schema.get_default_map_key() {
                // If not defined in schema, place it in the default map
                MutableValueExpression::Source(SourceScalarExpression::new(
                    query_location.clone(),
                    ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(query_location.clone(), default_map_key),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(query_location.clone(), &group_name),
                        )),
                    ]),
                ))
            } else {
                // No schema or default map, place directly on source
                MutableValueExpression::Source(SourceScalarExpression::new(
                    query_location.clone(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            query_location.clone(),
                            &group_name,
                        )),
                    )]),
                ))
            }
        } else {
            // No schema available, place directly on source
            MutableValueExpression::Source(SourceScalarExpression::new(
                query_location.clone(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        query_location.clone(),
                        &group_name,
                    )),
                )]),
            ))
        };

        let set_expression = TransformExpression::Set(SetTransformExpression::new(
            query_location.clone(),
            ImmutableValueExpression::Scalar(parse_expression),
            destination,
        ));

        set_expressions.push(set_expression);
    }

    Ok(set_expressions)
}

fn parse_parse_pattern_element(
    pattern_element_rule: Pair<Rule>,
) -> Result<ParsePatternElement, ParserError> {
    let rule_location = to_query_location(&pattern_element_rule);

    match pattern_element_rule.as_rule() {
        Rule::parse_pattern_element => {
            if let Some(inner_rule) = pattern_element_rule.into_inner().next() {
                match inner_rule.as_rule() {
                    Rule::string_literal => {
                        if let StaticScalarExpression::String(s) = parse_string_literal(inner_rule)
                        {
                            let value = s.get_value();
                            if value == "*" {
                                Ok(ParsePatternElement::Wildcard)
                            } else {
                                Ok(ParsePatternElement::Literal(value.to_string()))
                            }
                        } else {
                            Err(ParserError::SyntaxError(
                                rule_location,
                                "Expected string literal".to_string(),
                            ))
                        }
                    }
                    Rule::parse_column_spec => {
                        let mut parts = inner_rule.into_inner();
                        let name_rule = parts.next().unwrap();
                        let type_rule = parts.next(); // Type is now optional

                        Ok(ParsePatternElement::Column {
                            name: name_rule.as_str().to_string(),
                            column_type: type_rule
                                .map(|r| r.as_str().to_string())
                                .unwrap_or_else(|| "string".to_string()), // Default to string when not specified
                        })
                    }
                    _ => Err(ParserError::SyntaxError(
                        rule_location,
                        format!(
                            "Unexpected inner parse pattern element: {:?} {}",
                            inner_rule.as_rule(),
                            inner_rule.as_str()
                        ),
                    )),
                }
            } else {
                Err(ParserError::SyntaxError(
                    rule_location,
                    "Empty parse pattern element".to_string(),
                ))
            }
        }
        _ => Err(ParserError::SyntaxError(
            rule_location,
            format!(
                "Expected parse_pattern_element rule, got {:?}",
                pattern_element_rule.as_rule()
            ),
        )),
    }
}

#[derive(Debug, Clone)]
enum ParsePatternElement {
    Wildcard,
    Literal(String),
    Column { name: String, column_type: String },
}

pub(crate) fn parse_summarize_expression(
    summarize_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<DataExpression, ParserError> {
    let query_location = to_query_location(&summarize_expression_rule);

    let mut aggregation_expressions: HashMap<Box<str>, AggregationExpression> = HashMap::new();
    let mut group_by_expressions: HashMap<Box<str>, ScalarExpression> = HashMap::new();

    for summarize_rule in summarize_expression_rule.into_inner() {
        match summarize_rule.as_rule() {
            Rule::aggregate_assignment_expression => {
                let (key, aggregate) =
                    parse_aggregate_assignment_expression(summarize_rule, state)?;

                aggregation_expressions.insert(key, aggregate);
            }
            Rule::group_by_expression => {
                let mut group_by = summarize_rule.into_inner();

                let identifier_rule = group_by.next().unwrap();
                let identifier = identifier_rule.as_str();

                let scalar = if let Some(scalar_rule) = group_by.next() {
                    parse_scalar_expression(scalar_rule, state)?
                } else {
                    ScalarExpression::Source(SourceScalarExpression::new(
                        to_query_location(&identifier_rule),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                to_query_location(&identifier_rule),
                                identifier,
                            )),
                        )]),
                    ))
                };

                group_by_expressions.insert(identifier.into(), scalar);
            }
            _ => {
                // todo: Support applying tabular expressions to summary
                parse_tabular_expression(summarize_rule, state)?;
            }
        }
    }

    if group_by_expressions.is_empty() && aggregation_expressions.is_empty() {
        Err(ParserError::SyntaxError(
            query_location,
            "Invalid summarize operator: missing both aggregates and group-by expressions".into(),
        ))
    } else {
        Ok(DataExpression::Summary(SummaryDataExpression::new(
            query_location,
            group_by_expressions,
            aggregation_expressions,
        )))
    }
}

pub(crate) fn parse_tabular_expression(
    tabular_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<Vec<DataExpression>, ParserError> {
    let mut rules = tabular_expression_rule.into_inner();

    // Note: This is the identifier. In a query like logs | extend a=b the
    // indentifier is "logs" which is not currently used for anything.
    let _ = rules.next().unwrap();

    let mut expressions = Vec::new();

    for rule in rules {
        match rule.as_rule() {
            Rule::extend_expression => {
                let extend_expressions = parse_extend_expression(rule, state)?;

                for e in extend_expressions {
                    expressions.push(DataExpression::Transform(e));
                }
            }
            Rule::project_expression => {
                let project_expressions = parse_project_expression(rule, state)?;

                for e in project_expressions {
                    expressions.push(DataExpression::Transform(e));
                }
            }
            Rule::project_keep_expression => {
                let project_keep_expressions = parse_project_keep_expression(rule, state)?;

                for e in project_keep_expressions {
                    expressions.push(DataExpression::Transform(e));
                }
            }
            Rule::project_away_expression => {
                let project_away_expressions = parse_project_away_expression(rule, state)?;

                for e in project_away_expressions {
                    expressions.push(DataExpression::Transform(e));
                }
            }
            Rule::where_expression => expressions.push(parse_where_expression(rule, state)?),
            Rule::parse_expression => {
                let parse_expressions = parse_parse_expression(rule, state)?;

                for e in parse_expressions {
                    expressions.push(DataExpression::Transform(e));
                }
            }
            Rule::summarize_expression => {
                expressions.push(parse_summarize_expression(rule, state)?)
            }
            _ => panic!("Unexpected rule in tabular_expression: {rule}"),
        }
    }

    Ok(expressions)
}

fn get_root_map_key_from_source_scalar_expression(
    source_scalar_expression: &SourceScalarExpression,
) -> Option<StringScalarExpression> {
    let selectors = source_scalar_expression
        .get_value_accessor()
        .get_selectors();

    if selectors.len() == 1
        && let ScalarExpression::Static(StaticScalarExpression::String(key)) =
            selectors.first().unwrap()
    {
        return Some(key.clone());
    }

    None
}

enum IdentifierOrPattern {
    Identifier(StringScalarExpression),
    Map {
        name: StringScalarExpression,
        key: ScalarExpression,
    },
    Pattern(RegexScalarExpression),
}

fn parse_identifier_or_pattern_literal(
    state: &ParserState,
    location: QueryLocation,
    identifier_or_pattern_literal: Pair<Rule>,
) -> Result<Option<IdentifierOrPattern>, ParserError> {
    let raw = identifier_or_pattern_literal.as_str();

    let value: Box<str> = match identifier_or_pattern_literal.into_inner().next() {
        Some(r) => match r.as_rule() {
            Rule::string_literal => match parse_string_literal(r) {
                StaticScalarExpression::String(v) => v.get_value().into(),
                _ => panic!("Unexpected type returned from parse_string_literal"),
            },
            _ => panic!("Unexpected rule in identifier_or_pattern_literal: {r}"),
        },
        None => raw.into(),
    };

    if value.contains("*") {
        let mut pattern = regex::escape(&value).replace("\\*", ".*");
        pattern.insert(0, '^');
        let regex = Regex::new(&pattern);
        if regex.is_err() {
            return Err(ParserError::SyntaxError(
                location,
                format!(
                    "The '{value}' string value could not be parsed into a Regex: {}",
                    regex.unwrap_err()
                ),
            ));
        }
        Ok(Some(IdentifierOrPattern::Pattern(
            RegexScalarExpression::new(location, regex.unwrap()),
        )))
    } else if state.is_well_defined_identifier(&value) {
        Ok(None)
    } else if let Some(schema) = state.get_source_schema() {
        if schema.get_schema_for_key(&value).is_some() {
            Ok(Some(IdentifierOrPattern::Identifier(
                StringScalarExpression::new(location, &value),
            )))
        } else if let Some(d) = schema.get_default_map_key() {
            Ok(Some(IdentifierOrPattern::Map {
                name: StringScalarExpression::new(location.clone(), d),
                key: ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(location, &value),
                )),
            }))
        } else {
            Err(ParserError::QueryLanguageDiagnostic {
                location,
                diagnostic_id: "KS109",
                message: format!(
                    "The name '{value}' does not refer to any known column, table, variable or function",
                ),
            })
        }
    } else {
        Ok(Some(IdentifierOrPattern::Identifier(
            StringScalarExpression::new(location, &value),
        )))
    }
}

fn process_map_selection_source_scalar_expression(
    expression_name: &str,
    state: &ParserState,
    source: &SourceScalarExpression,
    reduction: &mut MapReductionState,
) -> Result<(), ParserError> {
    if let Some(map_key) = get_root_map_key_from_source_scalar_expression(source) {
        reduction.keys.push(map_key);
        return Ok(());
    }

    let destination_accessor = source.get_value_accessor();

    if !destination_accessor.has_selectors() {
        let location = source.get_query_location();
        return Err(ParserError::SyntaxError(
            location.clone(),
            format!(
                "The '{}' accessor expression should refer to a map key on the source when used in a {expression_name} expression",
                state.get_query_slice(location).trim()
            ),
        ));
    }

    let destination_selectors = destination_accessor.get_selectors();

    if destination_selectors.len() == 2 {
        // Note: If state has source keys defined look for selectors targeting maps off the root.
        if let ScalarExpression::Static(StaticScalarExpression::String(root)) =
            destination_selectors.first().unwrap()
        {
            let root_key = root.get_value();

            if Some(Some(Some(ValueType::Map)))
                == state
                    .get_source_schema()
                    .map(|v| v.get_schema_for_key(root_key).map(|v| v.get_value_type()))
            {
                let keys = reduction.maps.entry(root_key.into()).or_default();
                keys.push((root.clone(), destination_selectors.get(1).unwrap().clone()));
                return Ok(());
            }
        }
    }

    reduction.selectors.push(source.clone());

    Ok(())
}

fn foreach_source_schema_key<F>(schema: &ParserMapSchema, mut action: F)
where
    F: FnMut(&str),
{
    if cfg!(test) {
        // Note: When building tests we sort the key list so that it is
        // deterministice.
        let mut source_keys: Vec<&Box<str>> = schema.get_schema_for_keys().keys().collect();
        source_keys.sort();
        for k in source_keys {
            (action)(k)
        }
    } else {
        let source_keys = schema.get_schema_for_keys().keys();
        for k in source_keys {
            (action)(k)
        }
    }
}

struct MapReductionState {
    pub keys: Vec<StringScalarExpression>,
    pub patterns: Vec<RegexScalarExpression>,
    pub maps: HashMap<Box<str>, Vec<(StringScalarExpression, ScalarExpression)>>,
    pub selectors: Vec<SourceScalarExpression>,
}

impl MapReductionState {
    pub fn new() -> MapReductionState {
        Self {
            keys: Vec::new(),
            patterns: Vec::new(),
            maps: HashMap::new(),
            selectors: Vec::new(),
        }
    }
}

fn push_map_transformation_expression(
    expression_name: &str,
    state: &ParserState,
    expressions: &mut Vec<TransformExpression>,
    query_location: &QueryLocation,
    retain: bool,
    mut reduction: MapReductionState,
) -> Result<(), ParserError> {
    if reduction.patterns.is_empty() && reduction.selectors.is_empty() {
        // Note: If there are no patterns and no selectors this means we have a
        // simple reduction. Instead of using ReduceMap we use the simpler
        // RemoveMapKeys which is faster uses less resources to execute.

        if !reduction.maps.is_empty() {
            // maps contains the list of top-level maps on the source we want to
            // modify. Take an expression like "project key1, attributes.key2"
            // when using default_source_map_key=attributes. What we want to do
            // is remove everything except key1 and key2 from attributes. To do
            // this we issue a RemoveMapKeys expression targeting
            // source.attributes.
            for (_, v) in reduction.maps {
                let mut first_root = None;
                let mut map_keys = Vec::new();
                for (root, key) in v {
                    if first_root.is_none() {
                        first_root = Some(root);
                    }

                    map_keys.push(key);
                }

                let first_root = first_root.expect("First root was not set");

                let map_key_list = MapKeyListExpression::new(
                    query_location.clone(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        query_location.clone(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(first_root.clone()),
                        )]),
                    )),
                    map_keys,
                );

                if retain {
                    expressions.push(TransformExpression::RemoveMapKeys(
                        RemoveMapKeysTransformExpression::Retain(map_key_list),
                    ));
                } else {
                    expressions.push(TransformExpression::RemoveMapKeys(
                        RemoveMapKeysTransformExpression::Remove(map_key_list),
                    ));
                }

                if retain && !reduction.keys.is_empty() {
                    // In retain mode if we have other keys on the source we are
                    // also going to issue a RemoveMapKeys for source itself. We
                    // need to make sure the map is preserved so we add it to
                    // the root list.
                    reduction.keys.push(first_root);
                }
            }
        }

        if !reduction.keys.is_empty() {
            // keys contains the top-level keys we want to preserve. We issue a
            // RemoveMapKeys expression for the source with the list of keys.
            let map_key_list = MapKeyListExpression::new(
                query_location.clone(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    query_location.clone(),
                    ValueAccessor::new(),
                )),
                reduction
                    .keys
                    .drain(..)
                    .map(|k| ScalarExpression::Static(StaticScalarExpression::String(k)))
                    .collect(),
            );

            if retain {
                expressions.push(TransformExpression::RemoveMapKeys(
                    RemoveMapKeysTransformExpression::Retain(map_key_list),
                ));
            } else {
                expressions.push(TransformExpression::RemoveMapKeys(
                    RemoveMapKeysTransformExpression::Remove(map_key_list),
                ));
            }
        }
    } else {
        let mut map_selection = MapSelectionExpression::new(
            query_location.clone(),
            MutableValueExpression::Source(SourceScalarExpression::new(
                query_location.clone(),
                ValueAccessor::new(),
            )),
        );

        if !reduction.patterns.is_empty() {
            if let Some(schema) = state.get_source_schema() {
                let default_source_map = schema.get_default_map_key();
                let mut default_source_map_matched_regex = false;

                // Note: If we have schema we can apply the regex patterns ahead
                // of time.
                foreach_source_schema_key(schema, |k| {
                    for p in &reduction.patterns {
                        if p.get_value().is_match(k) {
                            if let Some(d) = default_source_map
                                && k == d
                            {
                                default_source_map_matched_regex = true;
                                if retain {
                                    continue;
                                }
                            }
                            reduction
                                .keys
                                .push(StringScalarExpression::new(query_location.clone(), k));
                            break;
                        }
                    }
                });

                if let Some(d) = default_source_map {
                    if retain || !default_source_map_matched_regex {
                        // Note: We add selectors for the default_source_map_key
                        // which will run the regex for its keys. When we are in
                        // remove mode if we already added the
                        // default_source_map_key we can skip this because the
                        // whole map will be removed.
                        for p in reduction.patterns {
                            reduction.selectors.push(SourceScalarExpression::new(
                                query_location.clone(),
                                ValueAccessor::new_with_selectors(vec![
                                    ScalarExpression::Static(StaticScalarExpression::String(
                                        StringScalarExpression::new(query_location.clone(), d),
                                    )),
                                    ScalarExpression::Static(StaticScalarExpression::Regex(p)),
                                ]),
                            ));
                        }
                    }
                }
            } else {
                for p in reduction.patterns {
                    map_selection.push_key_or_key_pattern(ScalarExpression::Static(
                        StaticScalarExpression::Regex(p),
                    ));
                }
            }
        }

        let mut processed_keys: HashSet<Box<str>> = HashSet::new();

        for k in reduction.keys {
            if !retain {
                processed_keys.insert(k.get_value().into());
            }
            let result = map_selection.push_key_or_key_pattern(ScalarExpression::Static(
                StaticScalarExpression::String(k),
            ));
            debug_assert!(result);
        }

        for s in reduction.selectors {
            if !map_selection.push_value_accessor(s.get_value_accessor().clone()) {
                let location = s.get_query_location();
                return Err(ParserError::SyntaxError(
                    location.clone(),
                    format!(
                        "The '{}' accessor expression should refer to a map key on the source when used in a {expression_name} expression",
                        state.get_query_slice(location).trim()
                    ),
                ));
            }
        }

        for (_, v) in reduction.maps {
            for (map, key) in v {
                if !retain && processed_keys.contains(map.get_value()) {
                    // Note: If we already removed the key we can ignore any
                    // selectors.
                    continue;
                }
                map_selection.push_value_accessor(ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(map)),
                    key,
                ]));
            }
        }

        if retain {
            expressions.push(TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Retain(map_selection),
            ));
        } else {
            expressions.push(TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Remove(map_selection),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_parse_extend_expression() {
        let run_test_success = |input: &str, expected: Vec<TransformExpression>| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");
            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let mut result = KqlPestParser::parse(Rule::extend_expression, input).unwrap();

            let expression = parse_extend_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlPestParser::parse(Rule::extend_expression, input).unwrap();

            let error = parse_extend_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        run_test_success(
            "extend new_attribute1 = 1",
            vec![TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ImmutableValueExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                )),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "new_attribute1",
                        )),
                    )]),
                )),
            ))],
        );

        // Note: In an extend operation "const_str" as the destination is not
        // resolved to its value. It is treated literally as the identifier to
        // set on the source. The source "const_str" does get evaluated to
        // "hello world".
        run_test_success(
            "extend const_str = const_str",
            vec![TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ImmutableValueExpression::Scalar(ScalarExpression::Constant(
                    ConstantScalarExpression::Reference(ReferenceConstantScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueType::String,
                        0,
                    )),
                )),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "const_str",
                        )),
                    )]),
                )),
            ))],
        );

        run_test_success(
            "extend new_attribute1 = 1, new_attribute2 = 2",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            1,
                        )),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "new_attribute1",
                            )),
                        )]),
                    )),
                )),
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            2,
                        )),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "new_attribute2",
                            )),
                        )]),
                    )),
                )),
            ],
        );

        run_test_success(
            "extend body.nested[0] = 1",
            vec![TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ImmutableValueExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                )),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "nested"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )),
                    ]),
                )),
            ))],
        );

        run_test_failure(
            "extend variable.key = 1",
            "'variable.key' destination accessor must refer to source to be used in an extend expression",
        );
    }

    #[test]
    fn test_parse_project_expression() {
        let run_test_success = |input: &str, expected: Vec<TransformExpression>| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_source_map_schema(
                        ParserMapSchema::new()
                            .set_default_map_key("attributes")
                            .with_key_definition("body", ParserMapKeySchema::Any),
                    )
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");
            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let mut result = KqlPestParser::parse(Rule::project_expression, input).unwrap();

            let expression = parse_project_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_source_map_schema(
                        ParserMapSchema::new()
                            .set_default_map_key("attributes")
                            .with_key_definition("body", ParserMapKeySchema::Any),
                    )
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");
            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let mut result = KqlPestParser::parse(Rule::project_expression, input).unwrap();

            let error = parse_project_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        // This test only modifies a single key which is resolved as
        // attributes['key1']. We use a RemoveMapKeys expression targeting
        // "source.attributes" to do this.
        run_test_success(
            "project key1",
            vec![TransformExpression::RemoveMapKeys(
                RemoveMapKeysTransformExpression::Retain(MapKeyListExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "attributes",
                            )),
                        )]),
                    )),
                    vec![ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                    ))],
                )),
            )],
        );

        // This test modifies "source.body" and "source.attributes" for "key1"
        // and "key2". What we do is issue a RemoveMapKeys for the source to
        // retain "body" and "attributes" and issue a RemoveMapKeys for
        // source.attributes which retains "key1" and "key2".
        run_test_success(
            "project key1, body, attributes['key2']",
            vec![
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                )),
                            )]),
                        )),
                        vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                            )),
                        ],
                    ),
                )),
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                        ],
                    ),
                )),
            ],
        );

        run_test_success(
            "project key1 = variable",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Variable(
                        VariableScalarExpression::new(
                            QueryLocation::new_fake(),
                            StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                            ValueAccessor::new(),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                            )),
                        ]),
                    )),
                )),
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                )),
                            )]),
                        )),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        ))],
                    ),
                )),
            ],
        );

        // Note: In an project operation "const_str" as the destination is not
        // resolved to its value. It is treated literally as the identifier to
        // set on the source. The source "const_str" does get evaluated to
        // "hello world".
        run_test_success(
            "project const_str = const_str",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Constant(
                        ConstantScalarExpression::Reference(
                            ReferenceConstantScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueType::String,
                                0,
                            ),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "const_str"),
                            )),
                        ]),
                    )),
                )),
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                )),
                            )]),
                        )),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "const_str"),
                        ))],
                    ),
                )),
            ],
        );

        run_test_success(
            "project key1 = variable, attributes['key2'] = resource[variable], source.attributes['key3']",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Variable(
                        VariableScalarExpression::new(
                            QueryLocation::new_fake(),
                            StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                            ValueAccessor::new(),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                            )),
                        ]),
                    )),
                )),
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Attached(
                        AttachedScalarExpression::new(
                            QueryLocation::new_fake(),
                            StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Variable(
                                VariableScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "variable",
                                    ),
                                    ValueAccessor::new(),
                                ),
                            )]),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                            )),
                        ]),
                    )),
                )),
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                )),
                            )]),
                        )),
                        vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key3"),
                            )),
                        ],
                    ),
                )),
            ],
        );

        run_test_success(
            "project body['complex'], source.body.nested[0], body[variable]",
            vec![TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Retain(MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "complex"),
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "nested"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Variable(VariableScalarExpression::new(
                                QueryLocation::new_fake(),
                                StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                                ValueAccessor::new(),
                            )),
                        ])),
                    ],
                )),
            )],
        );

        run_test_success(
            "project body['name'] = 'hello world'",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "hello world",
                        )),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "name"),
                            )),
                        ]),
                    )),
                )),
                TransformExpression::ReduceMap(ReduceMapTransformExpression::Retain(
                    MapSelectionExpression::new_with_selectors(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        vec![MapSelector::ValueAccessor(
                            ValueAccessor::new_with_selectors(vec![
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                                )),
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "name"),
                                )),
                            ]),
                        )],
                    ),
                )),
            ],
        );

        run_test_failure(
            "project source[0]",
            "The 'source[0]' accessor expression should refer to a map key on the source when used in a project expression",
        );

        run_test_failure(
            "project variable = 1",
            "'variable' destination accessor must refer to source to be used in a project expression",
        );

        run_test_failure(
            "project resource.attributes['key']",
            "To be valid in a project expression 'resource.attributes['key']' should be an assignment expression or an accessor expression which refers to the source",
        );

        run_test_failure(
            "project source",
            "The 'source' accessor expression should refer to a map key on the source when used in a project expression",
        );

        // Note: This is technically supported in KQL. What will happen in KQL
        // is a name is automatically generated. So this runs the same as
        // project const_str = const_str. We don't currently support generating
        // names though so this is an error. But with an easy workaround.
        run_test_failure(
            "project const_str",
            "To be valid in a project expression 'const_str' should be an assignment expression or an accessor expression which refers to the source",
        );
    }

    #[test]
    fn test_parse_project_keep_expression() {
        let run_test_success = |input: &str, expected: Vec<TransformExpression>| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_source_map_schema(
                        ParserMapSchema::new()
                            .set_default_map_key("attributes")
                            .with_key_definition("body", ParserMapKeySchema::Any),
                    )
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");
            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let mut result = KqlPestParser::parse(Rule::project_keep_expression, input).unwrap();

            let expression = parse_project_keep_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_source_map_schema(
                        ParserMapSchema::new()
                            .set_default_map_key("attributes")
                            .with_key_definition("body", ParserMapKeySchema::Any),
                    )
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");
            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let mut result = KqlPestParser::parse(Rule::project_keep_expression, input).unwrap();

            let error = parse_project_keep_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        // Simple form where RemoveMapKeys is used to retain a single key from attributes.
        run_test_success(
            "project-keep key1",
            vec![TransformExpression::RemoveMapKeys(
                RemoveMapKeysTransformExpression::Retain(MapKeyListExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "attributes",
                            )),
                        )]),
                    )),
                    vec![ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                    ))],
                )),
            )],
        );

        // In this test we use the simple form of RemoveMapKeys but we make two
        // calls. One to retain body & attributes on the source. And then a
        // second call to preserve only key1 on attributes.
        run_test_success(
            "project-keep body, key1",
            vec![
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                )),
                            )]),
                        )),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        ))],
                    ),
                )),
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                        ],
                    ),
                )),
            ],
        );

        // Note: In this test body is not present in the ReduceMap because it
        // doesn't match the regex.
        run_test_success(
            "project-keep ['namespace.*']",
            vec![TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Retain(MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![MapSelector::ValueAccessor(
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Regex(
                                RegexScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    Regex::new("^namespace\\..*").unwrap(),
                                ),
                            )),
                        ]),
                    )],
                )),
            )],
        );

        // Note: In this test body is present in the ReduceMap because it
        // matches the regex.
        run_test_success(
            "project-keep b*",
            vec![TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Retain(MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![
                        MapSelector::KeyOrKeyPattern(ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                        )),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Regex(
                                RegexScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    Regex::new("^b.*").unwrap(),
                                ),
                            )),
                        ])),
                    ],
                )),
            )],
        );

        // This is a more complex form of the previous tests just to make sure
        // multiple regex expressions work.
        run_test_success(
            "project-keep *key*value*, *, key1, attributes['key2']",
            vec![TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Retain(MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![
                        MapSelector::KeyOrKeyPattern(ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                        )),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Regex(
                                RegexScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    Regex::new("^.*key.*value.*").unwrap(),
                                ),
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Regex(
                                RegexScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    Regex::new("^.*").unwrap(),
                                ),
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                            )),
                        ])),
                    ],
                )),
            )],
        );

        // Test a more complex reduction on the body.
        run_test_success(
            "project-keep source.body.map['some_attr'], body.nested[0], body[variable]",
            vec![TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Retain(MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "map"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "some_attr"),
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "nested"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Variable(VariableScalarExpression::new(
                                QueryLocation::new_fake(),
                                StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                                ValueAccessor::new(),
                            )),
                        ])),
                    ],
                )),
            )],
        );

        run_test_failure(
            "project-keep source[0]",
            "The 'source[0]' accessor expression should refer to a map key on the source when used in a project-keep expression",
        );

        run_test_failure(
            "project-keep resource.attributes['key']",
            "To be valid in a project-keep expression 'resource.attributes['key']' should be an accessor expression which refers to the source",
        );

        /* ************** */
        // Note: The following four tests are technically all valid in KQL. What
        // will happen is "source", "resource", "variable", and "const_str" will
        // just be treated as column names. In query engine however we support a
        // much richer set of access operations using accessor expressions. This
        // is currently treated as an error to prevent mistakes.
        run_test_failure(
            "project-keep source",
            "To be valid in a project-keep expression 'source' should be an accessor expression which refers to data on the source",
        );

        run_test_failure(
            "project-keep resource",
            "To be valid in a project-keep expression 'resource' should be an accessor expression which refers to data on the source",
        );

        run_test_failure(
            "project-keep variable",
            "To be valid in a project-keep expression 'variable' should be an accessor expression which refers to data on the source",
        );

        run_test_failure(
            "project-keep const_str",
            "To be valid in a project-keep expression 'const_str' should be an accessor expression which refers to data on the source",
        );
        /* ************** */
    }

    #[test]
    fn test_parse_project_away_expression() {
        let run_test_success = |input: &str, expected: Vec<TransformExpression>| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_source_map_schema(
                        ParserMapSchema::new()
                            .set_default_map_key("attributes")
                            .with_key_definition("body", ParserMapKeySchema::Any),
                    )
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");
            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let mut result = KqlPestParser::parse(Rule::project_away_expression, input).unwrap();

            let expression = parse_project_away_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_source_map_schema(
                        ParserMapSchema::new()
                            .set_default_map_key("attributes")
                            .with_key_definition("body", ParserMapKeySchema::Any),
                    )
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");
            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let mut result = KqlPestParser::parse(Rule::project_away_expression, input).unwrap();

            let error = parse_project_away_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        // Simple form where RemoveMapKeys is used to remove a single key from attributes.
        run_test_success(
            "project-away key1",
            vec![TransformExpression::RemoveMapKeys(
                RemoveMapKeysTransformExpression::Remove(MapKeyListExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "attributes",
                            )),
                        )]),
                    )),
                    vec![ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                    ))],
                )),
            )],
        );

        // In this test we use the simple form of RemoveMapKeys but we make two
        // calls. One to remove body on the source. And then a
        // second call to remove key1 on attributes.
        run_test_success(
            "project-away body, key1",
            vec![
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Remove(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                )),
                            )]),
                        )),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        ))],
                    ),
                )),
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Remove(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                        ))],
                    ),
                )),
            ],
        );

        // Note: In this test body is not present in the ReduceMap because it
        // doesn't match the regex.
        run_test_success(
            "project-away ['namespace.*']",
            vec![TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![MapSelector::ValueAccessor(
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Regex(
                                RegexScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    Regex::new("^namespace\\..*").unwrap(),
                                ),
                            )),
                        ]),
                    )],
                )),
            )],
        );

        // Note: In this test body is present in the ReduceMap because it
        // matches the regex.
        run_test_success(
            "project-away b*",
            vec![TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![
                        MapSelector::KeyOrKeyPattern(ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                        )),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Regex(
                                RegexScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    Regex::new("^b.*").unwrap(),
                                ),
                            )),
                        ])),
                    ],
                )),
            )],
        );

        // In this test we understand attributes is removed by the regex so
        // everything else gets dropped.
        run_test_success(
            "project-away *key*value*, *, key1, attributes['key2']",
            vec![TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![
                        MapSelector::KeyOrKeyPattern(ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "attributes",
                            )),
                        )),
                        MapSelector::KeyOrKeyPattern(ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                        )),
                    ],
                )),
            )],
        );

        // Test a more complex reduction on the body.
        run_test_success(
            "project-away source.body.map['some_attr'], body.nested[0], body[variable]",
            vec![TransformExpression::ReduceMap(
                ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "map"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "some_attr"),
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "nested"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "body"),
                            )),
                            ScalarExpression::Variable(VariableScalarExpression::new(
                                QueryLocation::new_fake(),
                                StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                                ValueAccessor::new(),
                            )),
                        ])),
                    ],
                )),
            )],
        );

        run_test_failure(
            "project-away source[0]",
            "The 'source[0]' accessor expression should refer to a map key on the source when used in a project-away expression",
        );

        run_test_failure(
            "project-away resource.attributes['key']",
            "To be valid in a project-away expression 'resource.attributes['key']' should be an accessor expression which refers to the source",
        );

        /* ************** */
        // Note: The following four tests are technically all valid in KQL. What
        // will happen is "source", "resource", "variable", and "const_str" will
        // just be treated as column names. In query engine however we support a
        // much richer set of access operations using accessor expressions. This
        // is currently treated as an error to prevent mistakes.
        run_test_failure(
            "project-away source",
            "To be valid in a project-away expression 'source' should be an accessor expression which refers to data on the source",
        );

        run_test_failure(
            "project-away resource",
            "To be valid in a project-away expression 'resource' should be an accessor expression which refers to data on the source",
        );

        run_test_failure(
            "project-away variable",
            "To be valid in a project-away expression 'variable' should be an accessor expression which refers to data on the source",
        );

        run_test_failure(
            "project-away const_str",
            "To be valid in a project-away expression 'const_str' should be an accessor expression which refers to data on the source",
        );
        /* ************** */
    }

    #[test]
    pub fn test_parse_where_expression() {
        let run_test_success = |input: &str, expected: DataExpression| {
            let mut state = ParserState::new(input);

            state.push_variable_name("variable");

            let mut result = KqlPestParser::parse(Rule::where_expression, input).unwrap();

            let expression = parse_where_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "where variable",
            DataExpression::Discard(
                DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        QueryLocation::new_fake(),
                        LogicalExpression::Scalar(ScalarExpression::Variable(
                            VariableScalarExpression::new(
                                QueryLocation::new_fake(),
                                StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                                ValueAccessor::new(),
                            ),
                        )),
                    )),
                ),
            ),
        );

        run_test_success(
            "where 1 > 0",
            DataExpression::Discard(
                DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        QueryLocation::new_fake(),
                        LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                            )),
                        )),
                    )),
                ),
            ),
        );
    }

    #[test]
    pub fn test_parse_parse_expression() {
        // Test pest grammar parsing
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::parse_expression,
            &[
                "parse EventText with \"Event:\" resourceName:string",
                "parse kind=simple EventText with \"*\" resourceName:string \"*\"",
                "parse kind=regex EventText with \"Event: (.*?)\" resourceName:string",
                "parse kind=relaxed EventText with \"Event:\" resourceName:string \", totalSlices=\" totalSlices:long",
                "parse EventText with \"*\" col1:string \"*\" col2:string",
                "parse EventText with \"name=\" name:string \", count=\" count:long \", value=\" value:double \", active=\" active:bool \", time=\" time:datetime",
                "parse kind=regex flags=\"i\" EventText with \"(.*?)\" col:string",
                // Optional type tests
                "parse EventText with \"data:\" field",
                "parse EventText with \"*\" col1 \"*\" col2:int",
                "parse EventText with \"name:\" userName \", id:\" userId:long",
            ],
            &[
                "parse",                       // missing input expression
                "parse EventText",             // missing 'with' keyword
                "parse with \"*\" col:string", // missing input expression
            ],
        );

        // Helper to test successful parsing and validate column extraction
        let run_test = |input: &str, expected_columns: Vec<&str>| {
            let state = ParserState::new(input);
            let mut result = KqlPestParser::parse(Rule::parse_expression, input).unwrap();
            let expressions = parse_parse_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(
                expected_columns.len(),
                expressions.len(),
                "Wrong number of expressions for: {}",
                input
            );

            for (i, expression) in expressions.iter().enumerate() {
                if let TransformExpression::Set(set_expr) = expression {
                    // Validate destination column name
                    if let MutableValueExpression::Source(source_expr) = set_expr.get_destination()
                    {
                        let selectors = source_expr.get_value_accessor().get_selectors();
                        if let ScalarExpression::Static(StaticScalarExpression::String(
                            string_expr,
                        )) = &selectors[0]
                        {
                            assert_eq!(string_expr.get_value(), expected_columns[i]);
                        } else {
                            panic!("Expected string selector for column name");
                        }
                    } else {
                        panic!("Expected source expression as destination");
                    }

                    // Validate source is Text or Convert(Text)
                    match set_expr.get_source() {
                        ImmutableValueExpression::Scalar(ScalarExpression::Text(_)) => {
                            // String type - no conversion needed
                        }
                        ImmutableValueExpression::Scalar(ScalarExpression::Convert(
                            convert_expr,
                        )) => {
                            // Non-string type - should have conversion with Text inner expression
                            let has_text_inner = match convert_expr {
                                ConvertScalarExpression::Integer(c) => {
                                    matches!(c.get_inner_expression(), ScalarExpression::Text(_))
                                }
                                ConvertScalarExpression::Double(c) => {
                                    matches!(c.get_inner_expression(), ScalarExpression::Text(_))
                                }
                                ConvertScalarExpression::Boolean(c) => {
                                    matches!(c.get_inner_expression(), ScalarExpression::Text(_))
                                }
                                ConvertScalarExpression::DateTime(c) => {
                                    matches!(c.get_inner_expression(), ScalarExpression::Text(_))
                                }
                                ConvertScalarExpression::TimeSpan(c) => {
                                    matches!(c.get_inner_expression(), ScalarExpression::Text(_))
                                }
                                ConvertScalarExpression::String(c) => {
                                    matches!(c.get_inner_expression(), ScalarExpression::Text(_))
                                }
                            };
                            assert!(has_text_inner, "Expected Text expression inside conversion");
                        }
                        _ => panic!("Expected Text or Convert expression as source"),
                    }
                } else {
                    panic!("Expected Set transform expression");
                }
            }
        };

        // Test different parse modes
        run_test(
            "parse EventText with \"Event:\" resourceName:string",
            vec!["resourceName"],
        );
        run_test(
            "parse kind=simple EventText with \"*\" resourceName:string \"*\"",
            vec!["resourceName"],
        );
        run_test(
            "parse kind=regex EventText with \"Event: (.*?)\" resourceName:string",
            vec!["resourceName"],
        );
        run_test(
            "parse kind=relaxed EventText with \"Event:\" resourceName:string \", totalSlices=\" totalSlices:long",
            vec!["resourceName", "totalSlices"],
        );

        // Test multiple columns and wildcards
        run_test(
            "parse EventText with \"*\" col1:string \"*\" col2:string",
            vec!["col1", "col2"],
        );
        run_test(
            "parse EventText with \"*\" col1:string \"*\" col2:string \"*\" col3:string \"*\"",
            vec!["col1", "col2", "col3"],
        );

        // Test various column types
        run_test(
            "parse EventText with \"name=\" name:string \", count=\" count:long \", value=\" value:double \", active=\" active:bool \", time=\" time:datetime",
            vec!["name", "count", "value", "active", "time"],
        );

        // Test regex flags
        run_test(
            "parse kind=regex flags=\"i\" EventText with \"(.*?)\" col:string",
            vec!["col"],
        );
        run_test(
            "parse kind=regex flags=\"Ui\" EventText with \"(.*?)\" col:string",
            vec!["col"],
        );

        // Test complex patterns
        run_test(
            "parse EventText with \"resourceName=\" resourceName:string \", totalSlices=\" totalSlices:long \", sliceNumber=\" sliceNumber:long",
            vec!["resourceName", "totalSlices", "sliceNumber"],
        );

        // Test specific type conversions with one detailed example
        let input = "parse Severity with \"sevnum:\" SeverityNumber:int";
        let state = ParserState::new(input);
        let mut result = KqlPestParser::parse(Rule::parse_expression, input).unwrap();
        let expressions = parse_parse_expression(result.next().unwrap(), &state).unwrap();

        assert_eq!(1, expressions.len());
        if let TransformExpression::Set(set_expr) = &expressions[0] {
            // Verify integer conversion with correct pattern and group name
            if let ImmutableValueExpression::Scalar(ScalarExpression::Convert(
                ConvertScalarExpression::Integer(conversion),
            )) = set_expr.get_source()
            {
                if let ScalarExpression::Text(TextScalarExpression::Extract(extract)) =
                    conversion.get_inner_expression()
                {
                    if let ScalarExpression::Static(StaticScalarExpression::String(pattern)) =
                        extract.get_pattern_expression()
                    {
                        assert_eq!(pattern.get_value(), "sevnum:(?P<SeverityNumber>.*)");
                    }
                    if let ScalarExpression::Static(StaticScalarExpression::String(group)) =
                        extract.get_capture_name_or_index()
                    {
                        assert_eq!(group.get_value(), "SeverityNumber");
                    }
                }
            }
        }

        // Test optional column types (should default to string when not specified)
        run_test("parse EventText with \"data:\" field", vec!["field"]);
        run_test("parse EventText with \"*\" col1 \"*\" col2:int", vec!["col1", "col2"]);
        run_test("parse EventText with \"name:\" userName \", id:\" userId:long", vec!["userName", "userId"]);

        // Test that unsupported column types still default to string (no conversion)
        run_test("parse EventText with \"data:\" field:unknown", vec!["field"]);

        // Test grammar errors
        for invalid in ["parse", "parse EventText", "parse with \"*\" col:string"] {
            assert!(
                KqlPestParser::parse(Rule::parse_expression, invalid).is_err(),
                "Expected parse error for: {}",
                invalid
            );
        }
    }

    #[test]
    pub fn test_parse_summarize_expression() {
        let run_test_success = |input: &str, expected: SummaryDataExpression| {
            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::summarize_expression, input).unwrap();

            let expression = parse_summarize_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(DataExpression::Summary(expected), expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::summarize_expression, input).unwrap();

            let e = parse_summarize_expression(result.next().unwrap(), &state).unwrap_err();

            assert!(matches!(e, ParserError::SyntaxError(_, msg) if msg == expected))
        };

        run_test_success(
            "summarize c = count()",
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
                HashMap::from([(
                    "c".into(),
                    AggregationExpression::new(
                        QueryLocation::new_fake(),
                        AggregationFunction::Count,
                        None,
                    ),
                )]),
            ),
        );

        run_test_success(
            "summarize c = count(), d = count()",
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
                HashMap::from([
                    (
                        "c".into(),
                        AggregationExpression::new(
                            QueryLocation::new_fake(),
                            AggregationFunction::Count,
                            None,
                        ),
                    ),
                    (
                        "d".into(),
                        AggregationExpression::new(
                            QueryLocation::new_fake(),
                            AggregationFunction::Count,
                            None,
                        ),
                    ),
                ]),
            ),
        );

        run_test_success(
            "summarize by c",
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "c".into(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "c",
                            )),
                        )]),
                    )),
                )]),
                HashMap::new(),
            ),
        );

        run_test_success(
            "summarize by c, d = a",
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    (
                        "c".into(),
                        ScalarExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "c",
                                )),
                            )]),
                        )),
                    ),
                    (
                        "d".into(),
                        ScalarExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "a",
                                )),
                            )]),
                        )),
                    ),
                ]),
                HashMap::new(),
            ),
        );

        run_test_success(
            "summarize c = count() by a | extend v = 1",
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "a".into(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "a",
                            )),
                        )]),
                    )),
                )]),
                HashMap::from([(
                    "c".into(),
                    AggregationExpression::new(
                        QueryLocation::new_fake(),
                        AggregationFunction::Count,
                        None,
                    ),
                )]),
            ),
        );

        run_test_failure(
            "summarize | extend v = 1",
            "Invalid summarize operator: missing both aggregates and group-by expressions",
        );
    }

    #[test]
    fn test_parse_tabular_expression() {
        let run_test = |input: &str, expected: Vec<DataExpression>| {
            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::tabular_expression, input).unwrap();

            let expression = parse_tabular_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test(
            "source | where true | extend a = 1 | project-keep a",
            vec![
                DataExpression::Discard(
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        LogicalExpression::Not(NotLogicalExpression::new(
                            QueryLocation::new_fake(),
                            LogicalExpression::Scalar(ScalarExpression::Static(
                                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    true,
                                )),
                            )),
                        )),
                    ),
                ),
                DataExpression::Transform(TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            1,
                        )),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "a",
                            )),
                        )]),
                    )),
                ))),
                DataExpression::Transform(TransformExpression::RemoveMapKeys(
                    RemoveMapKeysTransformExpression::Retain(MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                        ))],
                    )),
                )),
            ],
        );

        run_test(
            "things | project-away a",
            vec![DataExpression::Transform(
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Remove(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                        ))],
                    ),
                )),
            )],
        );

        run_test(
            "source | project a",
            vec![DataExpression::Transform(
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                        ))],
                    ),
                )),
            )],
        );

        run_test(
            "source | summarize c = count()",
            vec![DataExpression::Summary(SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
                HashMap::from([(
                    "c".into(),
                    AggregationExpression::new(
                        QueryLocation::new_fake(),
                        AggregationFunction::Count,
                        None,
                    ),
                )]),
            ))],
        );

        // Test whitespace and newline handling before tabular expressions
        run_test(
            "source |\n  extend a = 1",
            vec![DataExpression::Transform(TransformExpression::Set(
                SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            1,
                        )),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "a",
                            )),
                        )]),
                    )),
                ),
            ))],
        );

        run_test(
            "source |\n\t\twhere true",
            vec![DataExpression::Discard(
                DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        QueryLocation::new_fake(),
                        LogicalExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                QueryLocation::new_fake(),
                                true,
                            )),
                        )),
                    )),
                ),
            )],
        );

        run_test(
            "source | \n  extend a = 1 |\n\t project a",
            vec![
                DataExpression::Transform(TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            1,
                        )),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "a",
                            )),
                        )]),
                    )),
                ))),
                DataExpression::Transform(TransformExpression::RemoveMapKeys(
                    RemoveMapKeysTransformExpression::Retain(MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        vec![ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                        ))],
                    )),
                )),
            ],
        );
    }
}
