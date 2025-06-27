use std::collections::HashSet;

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{
    Rule, logical_expressions::parse_logical_expression,
    scalar_primitive_expressions::parse_accessor_expression,
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

    let mut keys_to_keep = Some(HashSet::<MapKey>::new());

    let mut map_selection = MapSelectionExpression::new(
        query_location.clone(),
        MutableValueExpression::Source(SourceScalarExpression::new(
            query_location.clone(),
            ValueAccessor::new(),
        )),
    );

    for rule in project_rules {
        let rule_location = to_query_location(&rule);

        match rule.as_rule() {
            Rule::assignment_expression => {
                let assignment_expression = parse_assignment_expression(rule, state)?;

                if let TransformExpression::Set(s) = &assignment_expression {
                    match s.get_destination() {
                        MutableValueExpression::Source(s) => {
                            if let Some(map_key) =
                                get_root_map_key_from_source_scalar_expression(state, s)
                            {
                                if let Some(keys) = &mut keys_to_keep {
                                    keys.insert(map_key);
                                }
                            } else {
                                keys_to_keep = None
                            }

                            let accessor = s.get_value_accessor();

                            if !map_selection.push_value_accessor(accessor) {
                                let location = s.get_query_location();
                                return Err(ParserError::SyntaxError(
                                    location.clone(),
                                    format!(
                                        "The '{}' accessor expression should refer to a map key on the source when used in a project expression",
                                        state.get_query_slice(location).trim()
                                    ),
                                ));
                            }
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
                let accessor_expression = parse_accessor_expression(rule, state)?;

                if let ScalarExpression::Source(s) = &accessor_expression {
                    if let Some(map_key) = get_root_map_key_from_source_scalar_expression(state, s)
                    {
                        if let Some(keys) = &mut keys_to_keep {
                            keys.insert(map_key);
                        }
                    } else {
                        keys_to_keep = None
                    }

                    let accessor = s.get_value_accessor();

                    if !map_selection.push_value_accessor(accessor) {
                        let location = s.get_query_location();
                        return Err(ParserError::SyntaxError(
                            location.clone(),
                            format!(
                                "The '{}' accessor expression should refer to a map key on the source when used in a project expression",
                                state.get_query_slice(location).trim()
                            ),
                        ));
                    }
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

    if let Some(keys) = keys_to_keep {
        expressions.push(TransformExpression::RemoveMapKeys(
            RemoveMapKeysTransformExpression::Retain(MapKeyListExpression::new(
                query_location.clone(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    query_location,
                    ValueAccessor::new(),
                )),
                keys,
            )),
        ));
    } else {
        expressions.push(TransformExpression::ReduceMap(
            ReduceMapTransformExpression::Retain(map_selection),
        ));
    }

    Ok(expressions)
}

pub(crate) fn parse_project_keep_expression(
    project_keep_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<TransformExpression, ParserError> {
    let query_location = to_query_location(&project_keep_expression_rule);

    let project_keep_rules = project_keep_expression_rule.into_inner();

    let mut keys_to_keep = Some(HashSet::<MapKey>::new());

    let mut map_selection = MapSelectionExpression::new(
        query_location.clone(),
        MutableValueExpression::Source(SourceScalarExpression::new(
            query_location.clone(),
            ValueAccessor::new(),
        )),
    );

    for rule in project_keep_rules {
        let rule_location = to_query_location(&rule);

        match rule.as_rule() {
            Rule::identifier_or_pattern_literal => {
                if let Some(selector) =
                    parse_identifier_or_pattern(state, rule_location.clone(), rule.as_str())
                {
                    if let Some(keys) = &mut keys_to_keep {
                        match &selector {
                            MapSelector::KeyPattern(s) => {
                                keys.insert(MapKey::Pattern(s.clone()));
                            }
                            MapSelector::Key(s) => {
                                keys.insert(MapKey::Value(s.clone()));
                            }
                            MapSelector::ValueAccessor(_) => {
                                keys_to_keep = None;
                            }
                        }
                    }

                    map_selection.push_selector(selector);
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
                let accessor_expression = parse_accessor_expression(rule, state)?;

                if let ScalarExpression::Source(s) = &accessor_expression {
                    if let Some(map_key) = get_root_map_key_from_source_scalar_expression(state, s)
                    {
                        if let Some(keys) = &mut keys_to_keep {
                            keys.insert(map_key);
                        }
                    } else {
                        keys_to_keep = None
                    }

                    let accessor = s.get_value_accessor();

                    if !map_selection.push_value_accessor(accessor) {
                        let location = s.get_query_location();
                        return Err(ParserError::SyntaxError(
                            location.clone(),
                            format!(
                                "The '{}' accessor expression should refer to a map key on the source when used in a project-keep expression",
                                state.get_query_slice(location).trim()
                            ),
                        ));
                    }
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

    if let Some(keys) = keys_to_keep {
        return Ok(TransformExpression::RemoveMapKeys(
            RemoveMapKeysTransformExpression::Retain(MapKeyListExpression::new(
                query_location.clone(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    query_location,
                    ValueAccessor::new(),
                )),
                keys,
            )),
        ));
    }

    Ok(TransformExpression::ReduceMap(
        ReduceMapTransformExpression::Retain(map_selection),
    ))
}

pub(crate) fn parse_project_away_expression(
    project_away_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<TransformExpression, ParserError> {
    let query_location = to_query_location(&project_away_expression_rule);

    let project_away_rules = project_away_expression_rule.into_inner();

    let mut keys_to_remove = Some(HashSet::<MapKey>::new());

    let mut map_selection = MapSelectionExpression::new(
        query_location.clone(),
        MutableValueExpression::Source(SourceScalarExpression::new(
            query_location.clone(),
            ValueAccessor::new(),
        )),
    );

    for rule in project_away_rules {
        let rule_location = to_query_location(&rule);

        match rule.as_rule() {
            Rule::identifier_or_pattern_literal => {
                if let Some(selector) =
                    parse_identifier_or_pattern(state, rule_location.clone(), rule.as_str())
                {
                    if let Some(keys) = &mut keys_to_remove {
                        match &selector {
                            MapSelector::KeyPattern(s) => {
                                keys.insert(MapKey::Pattern(s.clone()));
                            }
                            MapSelector::Key(s) => {
                                keys.insert(MapKey::Value(s.clone()));
                            }
                            MapSelector::ValueAccessor(_) => {
                                keys_to_remove = None;
                            }
                        }
                    }

                    map_selection.push_selector(selector);
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
                let accessor_expression = parse_accessor_expression(rule, state)?;

                if let ScalarExpression::Source(s) = &accessor_expression {
                    if let Some(map_key) = get_root_map_key_from_source_scalar_expression(state, s)
                    {
                        if let Some(keys) = &mut keys_to_remove {
                            keys.insert(map_key);
                        }
                    } else {
                        keys_to_remove = None
                    }

                    let accessor = s.get_value_accessor();

                    if !map_selection.push_value_accessor(accessor) {
                        let location = s.get_query_location();
                        return Err(ParserError::SyntaxError(
                            location.clone(),
                            format!(
                                "The '{}' accessor expression should refer to a map key on the source when used in a project-away expression",
                                state.get_query_slice(location).trim()
                            ),
                        ));
                    }
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

    if let Some(keys) = keys_to_remove {
        return Ok(TransformExpression::RemoveMapKeys(
            RemoveMapKeysTransformExpression::Remove(MapKeyListExpression::new(
                query_location.clone(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    query_location,
                    ValueAccessor::new(),
                )),
                keys,
            )),
        ));
    }

    Ok(TransformExpression::ReduceMap(
        ReduceMapTransformExpression::Remove(map_selection),
    ))
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
            Rule::project_keep_expression => expressions.push(DataExpression::Transform(
                parse_project_keep_expression(rule, state)?,
            )),
            Rule::project_away_expression => expressions.push(DataExpression::Transform(
                parse_project_away_expression(rule, state)?,
            )),
            Rule::where_expression => expressions.push(parse_where_expression(rule, state)?),
            _ => panic!("Unexpected rule in tabular_expression: {rule}"),
        }
    }

    Ok(expressions)
}

fn get_root_map_key_from_source_scalar_expression(
    state: &ParserState,
    source_scalar_expression: &SourceScalarExpression,
) -> Option<MapKey> {
    let selectors = source_scalar_expression
        .get_value_accessor()
        .get_selectors();

    if selectors.len() == 1 {
        if let ValueSelector::MapKey(k) = selectors.first().unwrap() {
            return Some(MapKey::Value(k.clone()));
        }
    } else if selectors.len() == 2 {
        // Note: If state has default_source_map_key we allow it
        // to be referenced. For example key2, source.key2,
        // attributes['key2'], and source.attributes['key2'] may
        // all refer to the same thing when
        // default_source_map_key=attributes.
        if let ValueSelector::MapKey(k) = selectors.first().unwrap() {
            let root_key = k.get_value();

            if Some(root_key) == state.get_default_source_map_key() {
                if let ValueSelector::MapKey(k) = selectors.get(1).unwrap() {
                    return Some(MapKey::Value(k.clone()));
                }
            }
        }
    }

    None
}

fn parse_identifier_or_pattern(
    state: &ParserState,
    location: QueryLocation,
    value: &str,
) -> Option<MapSelector> {
    if value.contains("*") {
        Some(MapSelector::KeyPattern(StringScalarExpression::new(
            location, value,
        )))
    } else if state.is_well_defined_identifier(value) {
        None
    } else {
        Some(MapSelector::Key(StringScalarExpression::new(
            location, value,
        )))
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::KqlParser;

    use super::*;

    #[test]
    fn test_parse_extend_expression() {
        let run_test_success = |input: &str, expected: Vec<TransformExpression>| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::extend_expression, input).unwrap();

            let expression = parse_extend_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new().with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::extend_expression, input).unwrap();

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
                    ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                        StringScalarExpression::new(QueryLocation::new_fake(), "new_attribute1"),
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
                        ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                            StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "new_attribute1",
                            ),
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
                        ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                            StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "new_attribute2",
                            ),
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
                        ValueSelector::MapKey(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "body",
                        )),
                        ValueSelector::MapKey(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "nested",
                        )),
                        ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            0,
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
                    .with_default_source_map_key_name("attributes")
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::project_expression, input).unwrap();

            let expression = parse_project_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_default_source_map_key_name("attributes")
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::project_expression, input).unwrap();

            let error = parse_project_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        run_test_success(
            "project key1",
            vec![TransformExpression::RemoveMapKeys(
                RemoveMapKeysTransformExpression::Retain(MapKeyListExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    HashSet::from([MapKey::Value(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    ))]),
                )),
            )],
        );

        run_test_success(
            "project key1, key2",
            vec![TransformExpression::RemoveMapKeys(
                RemoveMapKeysTransformExpression::Retain(MapKeyListExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    HashSet::from([
                        MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        )),
                        MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key2",
                        )),
                    ]),
                )),
            )],
        );

        run_test_success(
            "project key1 = variable",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Variable(
                        VariableScalarExpression::new(
                            QueryLocation::new_fake(),
                            "variable",
                            ValueAccessor::new(),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        )]),
                    )),
                )),
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        HashSet::from([MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        ))]),
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
                            "variable",
                            ValueAccessor::new(),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        )]),
                    )),
                )),
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Attached(
                        AttachedScalarExpression::new(
                            QueryLocation::new_fake(),
                            "resource",
                            ValueAccessor::new_with_selectors(vec![
                                ValueSelector::ScalarExpression(ScalarExpression::Variable(
                                    VariableScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "variable",
                                        ValueAccessor::new(),
                                    ),
                                )),
                            ]),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "attributes",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "key2",
                            )),
                        ]),
                    )),
                )),
                TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                    MapKeyListExpression::new(
                        QueryLocation::new_fake(),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        HashSet::from([
                            MapKey::Value(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "key1",
                            )),
                            MapKey::Value(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "key2",
                            )),
                            MapKey::Value(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "key3",
                            )),
                        ]),
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
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "complex",
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "nested",
                            )),
                            ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                0,
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::ScalarExpression(ScalarExpression::Variable(
                                VariableScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "variable",
                                    ValueAccessor::new(),
                                ),
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
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "name",
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
                                ValueSelector::MapKey(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "body",
                                )),
                                ValueSelector::MapKey(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "name",
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
    }

    #[test]
    fn test_parse_project_keep_expression() {
        let run_test_success = |input: &str, expected: TransformExpression| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_default_source_map_key_name("attributes")
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::project_keep_expression, input).unwrap();

            let expression = parse_project_keep_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_default_source_map_key_name("attributes")
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::project_keep_expression, input).unwrap();

            let error = parse_project_keep_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        run_test_success(
            "project-keep key*",
            TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                MapKeyListExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    HashSet::from([MapKey::Pattern(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key*",
                    ))]),
                ),
            )),
        );

        run_test_success(
            "project-keep *key*value*, *, key1, attributes['key2'], source.attributes['key3']",
            TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Retain(
                MapKeyListExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    HashSet::from([
                        MapKey::Pattern(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "*key*value*",
                        )),
                        MapKey::Pattern(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "*",
                        )),
                        MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        )),
                        MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key2",
                        )),
                        MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key3",
                        )),
                    ]),
                ),
            )),
        );

        run_test_success(
            "project-keep source.body.map['some_attr'], body.nested[0], body[variable]",
            TransformExpression::ReduceMap(ReduceMapTransformExpression::Retain(
                MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "map",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "some_attr",
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "nested",
                            )),
                            ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                0,
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::ScalarExpression(ScalarExpression::Variable(
                                VariableScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "variable",
                                    ValueAccessor::new(),
                                ),
                            )),
                        ])),
                    ],
                ),
            )),
        );

        run_test_failure(
            "project-keep source",
            "To be valid in a project-keep expression 'source' should be an accessor expression which refers to data on the source",
        );

        run_test_failure(
            "project-keep source[0]",
            "The 'source[0]' accessor expression should refer to a map key on the source when used in a project-keep expression",
        );

        run_test_failure(
            "project-keep resource.attributes['key']",
            "To be valid in a project-keep expression 'resource.attributes['key']' should be an accessor expression which refers to the source",
        );
    }

    #[test]
    fn test_parse_project_away_expression() {
        let run_test_success = |input: &str, expected: TransformExpression| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_default_source_map_key_name("attributes")
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::project_away_expression, input).unwrap();

            let expression = parse_project_away_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_default_source_map_key_name("attributes")
                    .with_attached_data_names(&["resource"]),
            );

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::project_away_expression, input).unwrap();

            let error = parse_project_away_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        run_test_success(
            "project-away key*",
            TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Remove(
                MapKeyListExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    HashSet::from([MapKey::Pattern(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key*",
                    ))]),
                ),
            )),
        );

        run_test_success(
            "project-away *key*value*, *, key1, attributes['key2'], source.attributes['key3']",
            TransformExpression::RemoveMapKeys(RemoveMapKeysTransformExpression::Remove(
                MapKeyListExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    HashSet::from([
                        MapKey::Pattern(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "*key*value*",
                        )),
                        MapKey::Pattern(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "*",
                        )),
                        MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        )),
                        MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key2",
                        )),
                        MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key3",
                        )),
                    ]),
                ),
            )),
        );

        run_test_success(
            "project-away source.body.map['some_attr'], body.nested[0], body[variable]",
            TransformExpression::ReduceMap(ReduceMapTransformExpression::Remove(
                MapSelectionExpression::new_with_selectors(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    vec![
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "map",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "some_attr",
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "nested",
                            )),
                            ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                0,
                            )),
                        ])),
                        MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                            ValueSelector::MapKey(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "body",
                            )),
                            ValueSelector::ScalarExpression(ScalarExpression::Variable(
                                VariableScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "variable",
                                    ValueAccessor::new(),
                                ),
                            )),
                        ])),
                    ],
                ),
            )),
        );

        run_test_failure(
            "project-away source",
            "To be valid in a project-away expression 'source' should be an accessor expression which refers to data on the source",
        );

        run_test_failure(
            "project-away source[0]",
            "The 'source[0]' accessor expression should refer to a map key on the source when used in a project-away expression",
        );

        run_test_failure(
            "project-away resource.attributes['key']",
            "To be valid in a project-away expression 'resource.attributes['key']' should be an accessor expression which refers to the source",
        );
    }

    #[test]
    pub fn test_parse_where_expression() {
        let run_test_success = |input: &str, expected: DataExpression| {
            let mut state = ParserState::new(input);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::where_expression, input).unwrap();

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
                                "variable",
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
    fn test_parse_tabular_expression() {
        let run_test = |input: &str, expected: Vec<DataExpression>| {
            let state = ParserState::new(input);

            let mut result = KqlParser::parse(Rule::tabular_expression, input).unwrap();

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
                        ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                            StringScalarExpression::new(QueryLocation::new_fake(), "a"),
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
                        HashSet::from([MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "a",
                        ))]),
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
                        HashSet::from([MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "a",
                        ))]),
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
                        HashSet::from([MapKey::Value(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "a",
                        ))]),
                    ),
                )),
            )],
        );
    }
}
