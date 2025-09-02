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
    shared_expressions::parse_source_assignment_expression,
};

pub(crate) fn parse_extend_expression(
    extend_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<Vec<TransformExpression>, ParserError> {
    let extend_rules = extend_expression_rule.into_inner();

    let mut set_expressions = Vec::new();

    for rule in extend_rules {
        match rule.as_rule() {
            Rule::assignment_expression => {
                let (query_location, source, destination) =
                    parse_source_assignment_expression(rule, scope)?;

                set_expressions.push(TransformExpression::Set(SetTransformExpression::new(
                    query_location,
                    source,
                    MutableValueExpression::Source(destination),
                )));
            }
            _ => panic!("Unexpected rule in extend_expression: {rule}"),
        }
    }

    Ok(set_expressions)
}

pub(crate) fn parse_project_expression(
    project_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<Vec<TransformExpression>, ParserError> {
    let query_location = to_query_location(&project_expression_rule);

    let project_rules = project_expression_rule.into_inner();

    let mut expressions = Vec::new();

    let mut reduction = MapReductionState::new();

    for rule in project_rules {
        let rule_location = to_query_location(&rule);

        match rule.as_rule() {
            Rule::assignment_expression => {
                let (query_location, source, destination) =
                    parse_source_assignment_expression(rule, scope)?;

                process_map_selection_source_scalar_expression(
                    "project",
                    scope,
                    &destination,
                    &mut reduction,
                )?;

                expressions.push(TransformExpression::Set(SetTransformExpression::new(
                    query_location,
                    source,
                    MutableValueExpression::Source(destination),
                )));
            }
            Rule::accessor_expression => {
                let accessor_expression = parse_accessor_expression(rule, scope, true)?;

                if let ScalarExpression::Source(s) = &accessor_expression {
                    process_map_selection_source_scalar_expression(
                        "project",
                        scope,
                        s,
                        &mut reduction,
                    )?;
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location.clone(),
                        format!(
                            "To be valid in a project expression '{}' should be an assignment expression or an accessor expression which refers to the source",
                            scope.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            _ => panic!("Unexpected rule in project_expression: {rule}"),
        }
    }

    push_map_transformation_expression(
        "project",
        scope,
        &mut expressions,
        &query_location,
        true,
        reduction,
    )?;

    Ok(expressions)
}

pub(crate) fn parse_project_keep_expression(
    project_keep_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
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
                    parse_identifier_or_pattern_literal(scope, rule_location.clone(), rule)?
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
                            scope.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            Rule::accessor_expression => {
                let accessor_expression = parse_accessor_expression(rule, scope, true)?;

                if let ScalarExpression::Source(s) = &accessor_expression {
                    process_map_selection_source_scalar_expression(
                        "project-keep",
                        scope,
                        s,
                        &mut reduction,
                    )?;
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location.clone(),
                        format!(
                            "To be valid in a project-keep expression '{}' should be an accessor expression which refers to the source",
                            scope.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            _ => panic!("Unexpected rule in project_keep_expression: {rule}"),
        }
    }

    push_map_transformation_expression(
        "project-keep",
        scope,
        &mut expressions,
        &query_location,
        true,
        reduction,
    )?;

    Ok(expressions)
}

pub(crate) fn parse_project_away_expression(
    project_away_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
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
                    parse_identifier_or_pattern_literal(scope, rule_location.clone(), rule)?
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
                            scope.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            Rule::accessor_expression => {
                let accessor_expression = parse_accessor_expression(rule, scope, true)?;

                if let ScalarExpression::Source(s) = &accessor_expression {
                    process_map_selection_source_scalar_expression(
                        "project-away",
                        scope,
                        s,
                        &mut reduction,
                    )?;
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location.clone(),
                        format!(
                            "To be valid in a project-away expression '{}' should be an accessor expression which refers to the source",
                            scope.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            _ => panic!("Unexpected rule in project_away_expression: {rule}"),
        }
    }

    push_map_transformation_expression(
        "project-away",
        scope,
        &mut expressions,
        &query_location,
        false,
        reduction,
    )?;

    Ok(expressions)
}

pub(crate) fn parse_where_expression(
    where_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<DataExpression, ParserError> {
    let query_location = to_query_location(&where_expression_rule);

    let where_rule = where_expression_rule.into_inner().next().unwrap();

    let predicate = match where_rule.as_rule() {
        Rule::logical_expression => parse_logical_expression(where_rule, scope)?,
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

pub(crate) fn parse_summarize_expression(
    summarize_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<DataExpression, ParserError> {
    let query_location = to_query_location(&summarize_expression_rule);

    let mut aggregation_expressions: HashMap<Box<str>, AggregationExpression> = HashMap::new();
    let mut group_by_expressions: HashMap<Box<str>, ScalarExpression> = HashMap::new();
    let mut post_expressions = Vec::new();

    for summarize_rule in summarize_expression_rule.into_inner() {
        match summarize_rule.as_rule() {
            Rule::aggregate_assignment_expression => {
                let (key, aggregate) =
                    parse_aggregate_assignment_expression(summarize_rule, scope)?;

                aggregation_expressions.insert(key, aggregate);
            }
            Rule::group_by_expression => {
                let mut group_by = summarize_rule.into_inner();

                let group_by_first_rule = group_by.next().unwrap();
                let group_by_first_rule_location = to_query_location(&group_by_first_rule);

                match group_by_first_rule.as_rule() {
                    Rule::identifier_literal => {
                        let scalar = parse_scalar_expression(group_by.next().unwrap(), scope)?;

                        group_by_expressions
                            .insert(group_by_first_rule.as_str().trim().into(), scalar);
                    }
                    Rule::accessor_expression => {
                        let mut accessor =
                            parse_accessor_expression(group_by_first_rule, scope, true)?;

                        // Note: The call here into try_resolve_static is to
                        // make sure all eligible selectors on the accessor are
                        // folded into static values.
                        accessor
                            .try_resolve_static(&scope.get_pipeline().get_resolution_scope())
                            .map_err(|e| ParserError::from(&e))?;

                        match &accessor {
                            ScalarExpression::Source(s) => {
                                group_by_expressions.insert(
                                    parse_group_by_accessor(
                                        &group_by_first_rule_location,
                                        s.get_value_accessor(),
                                        scope,
                                    )?,
                                    accessor,
                                );
                            }
                            ScalarExpression::Attached(a) => {
                                group_by_expressions.insert(
                                    parse_group_by_accessor(
                                        &group_by_first_rule_location,
                                        a.get_value_accessor(),
                                        scope,
                                    )?,
                                    accessor,
                                );
                            }
                            ScalarExpression::Variable(v) => {
                                group_by_expressions.insert(
                                    parse_group_by_accessor(
                                        &group_by_first_rule_location,
                                        v.get_value_accessor(),
                                        scope,
                                    )?,
                                    accessor,
                                );
                            }
                            _ => {
                                return Err(ParserError::SyntaxError(
                                    group_by_first_rule_location,
                                    "Could not determine the source and/or name for summary group-by expression. Try using assignment syntax instead (Name = [expression]).".into(),
                                ));
                            }
                        }
                    }
                    _ => panic!("Unexpected rule in group_by_expression: {group_by_first_rule}"),
                }
            }
            _ => {
                let scope = scope.create_scope(ParserOptions::new());

                for e in parse_tabular_expression_rule(summarize_rule, &scope)? {
                    post_expressions.push(e);
                }
            }
        }
    }

    if group_by_expressions.is_empty() && aggregation_expressions.is_empty() {
        return Err(ParserError::SyntaxError(
            query_location,
            "Invalid summarize operator: missing both aggregates and group-by expressions".into(),
        ));
    } else {
        let mut summary = SummaryDataExpression::new(
            query_location,
            group_by_expressions,
            aggregation_expressions,
        );

        if !post_expressions.is_empty() {
            for e in post_expressions {
                summary.push_post_expression(e);
            }
        }

        return Ok(DataExpression::Summary(summary));
    }

    fn parse_group_by_accessor(
        query_location: &QueryLocation,
        value_accessor: &ValueAccessor,
        scope: &dyn ParserScope,
    ) -> Result<Box<str>, ParserError> {
        fn try_read_string(
            scalar: &ScalarExpression,
            value: &StaticScalarExpression,
        ) -> Result<Box<str>, ParserError> {
            match value.to_value() {
                Value::String(s) => {
                    Ok(s.get_value().into())
                }
                _ => {
                    Err(ParserError::SyntaxError(
                        scalar.get_query_location().clone(),
                        "Could not determine the name for summary group-by expression. Try using assignment syntax instead (Name = [expression]).".into(),
                    ))
                }
            }
        }

        let selectors = value_accessor.get_selectors();
        if selectors.is_empty() {
            Err(ParserError::SyntaxError(
                query_location.clone(),
                "Cannot refer to a root map directly in a group-by expression".into(),
            ))
        } else {
            let last = selectors.last().unwrap();
            match scope.scalar_as_static(last) {
                Some(v) => try_read_string(last, v),
                None => {
                    Err(ParserError::SyntaxError(
                        last.get_query_location().clone(),
                        "Could not determine the name for summary group-by expression. Try using assignment syntax instead (Name = [expression]).".into(),
                    ))
                }
            }
        }
    }
}

pub(crate) fn parse_tabular_expression(
    tabular_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<Vec<DataExpression>, ParserError> {
    let mut rules = tabular_expression_rule.into_inner();

    // Note: This is the identifier. In a query like logs | extend a=b the
    // indentifier is "logs" which is not currently used for anything.
    let _ = rules.next().unwrap();

    let mut expressions = Vec::new();

    for rule in rules {
        for e in parse_tabular_expression_rule(rule, scope)? {
            expressions.push(e);
        }
    }

    Ok(expressions)
}

pub(crate) fn parse_tabular_expression_rule(
    tabular_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<Vec<DataExpression>, ParserError> {
    let mut expressions = Vec::new();

    match tabular_expression_rule.as_rule() {
        Rule::extend_expression => {
            let extend_expressions = parse_extend_expression(tabular_expression_rule, scope)?;

            for e in extend_expressions {
                expressions.push(DataExpression::Transform(e));
            }
        }
        Rule::project_expression => {
            let project_expressions = parse_project_expression(tabular_expression_rule, scope)?;

            for e in project_expressions {
                expressions.push(DataExpression::Transform(e));
            }
        }
        Rule::project_keep_expression => {
            let project_keep_expressions =
                parse_project_keep_expression(tabular_expression_rule, scope)?;

            for e in project_keep_expressions {
                expressions.push(DataExpression::Transform(e));
            }
        }
        Rule::project_away_expression => {
            let project_away_expressions =
                parse_project_away_expression(tabular_expression_rule, scope)?;

            for e in project_away_expressions {
                expressions.push(DataExpression::Transform(e));
            }
        }
        Rule::where_expression => {
            expressions.push(parse_where_expression(tabular_expression_rule, scope)?)
        }
        Rule::summarize_expression => {
            expressions.push(parse_summarize_expression(tabular_expression_rule, scope)?)
        }
        _ => panic!("Unexpected rule in tabular_expression: {tabular_expression_rule}"),
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
    scope: &dyn ParserScope,
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
    } else if scope.is_well_defined_identifier(&value) {
        Ok(None)
    } else if let Some(schema) = scope.get_source_schema() {
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
    scope: &dyn ParserScope,
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
                scope.get_query_slice(location).trim()
            ),
        ));
    }

    let destination_selectors = destination_accessor.get_selectors();

    if destination_selectors.len() == 2 {
        // Note: If scope has source keys defined look for selectors targeting maps off the root.
        if let ScalarExpression::Static(StaticScalarExpression::String(root)) =
            destination_selectors.first().unwrap()
        {
            let root_key = root.get_value();

            if Some(Some(Some(ValueType::Map)))
                == scope
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
    scope: &dyn ParserScope,
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
            if let Some(schema) = scope.get_source_schema() {
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
                        scope.get_query_slice(location).trim()
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

        run_test_success(
            "extend new_attribute1 = 1",
            vec![TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
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
                ScalarExpression::Constant(ConstantScalarExpression::Reference(
                    ReferenceConstantScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueType::String,
                        0,
                    ),
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
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
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
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
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
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
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

        // Note: variable on the left of assignment in extend context should set
        // Attributes['variable'] and not write to Variables['variable'].
        run_test_success(
            "extend variable = variable",
            vec![TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                    ValueAccessor::new(),
                )),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "variable",
                        )),
                    )]),
                )),
            ))],
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
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
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

        // Note: variable on the left of assignment in project context should set
        // Attributes['variable'] and not write to Variables['variable'].
        run_test_success(
            "project variable = variable",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
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
                                StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
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
                            StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
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
                    ScalarExpression::Constant(ConstantScalarExpression::Reference(
                        ReferenceConstantScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueType::String,
                            0,
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
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                        ValueAccessor::new(),
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
                    ScalarExpression::Attached(AttachedScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Variable(
                            VariableScalarExpression::new(
                                QueryLocation::new_fake(),
                                StringScalarExpression::new(QueryLocation::new_fake(), "variable"),
                                ValueAccessor::new(),
                            ),
                        )]),
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
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
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
            "project variable",
            "To be valid in a project expression 'variable' should be an assignment expression or an accessor expression which refers to the source",
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
    pub fn test_parse_summarize_expression() {
        let run_test_success = |input: &str, expected: SummaryDataExpression| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_source_map_schema(
                        ParserMapSchema::new()
                            .with_key_definition("a", ParserMapKeySchema::Any)
                            .with_key_definition("c", ParserMapKeySchema::Any)
                            .with_key_definition("Attributes", ParserMapKeySchema::Map)
                            .set_default_map_key("Attributes"),
                    )
                    .with_attached_data_names(&["resource"]),
            );

            state.push_constant(
                "const",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "const_str",
                )),
            );

            let mut result = KqlPestParser::parse(Rule::summarize_expression, input).unwrap();

            let expression = parse_summarize_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(DataExpression::Summary(expected), expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new_with_options(
                input,
                ParserOptions::new()
                    .with_source_map_schema(
                        ParserMapSchema::new()
                            .with_key_definition("a", ParserMapKeySchema::Any)
                            .with_key_definition("c", ParserMapKeySchema::Any)
                            .with_key_definition("Attributes", ParserMapKeySchema::Map)
                            .set_default_map_key("Attributes"),
                    )
                    .with_attached_data_names(&["resource"]),
            );

            state.push_constant(
                "const",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "const_str",
                )),
            );

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
            )
            .with_post_expressions(vec![DataExpression::Transform(
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "v",
                            )),
                        )]),
                    )),
                )),
            )]),
        );

        run_test_success(
            "summarize by Level",
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "Level".into(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "Attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "Level"),
                            )),
                        ]),
                    )),
                )]),
                HashMap::new(),
            ),
        );

        run_test_success(
            "summarize by Attributes[const]",
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "const_str".into(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "Attributes",
                                ),
                            )),
                            ScalarExpression::Constant(ConstantScalarExpression::Copy(
                                CopyConstantScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    0,
                                    StaticScalarExpression::String(StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "const_str",
                                    )),
                                ),
                            )),
                        ]),
                    )),
                )]),
                HashMap::new(),
            ),
        );

        run_test_success(
            "summarize by Attributes['something']['else']",
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "else".into(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "Attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "something"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "else"),
                            )),
                        ]),
                    )),
                )]),
                HashMap::new(),
            ),
        );

        run_test_success(
            "summarize by resource.Attributes['service.name']",
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "service.name".into(),
                    ScalarExpression::Attached(AttachedScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "Attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "service.name",
                                ),
                            )),
                        ]),
                    )),
                )]),
                HashMap::new(),
            ),
        );

        run_test_failure(
            "summarize | extend v = 1",
            "Invalid summarize operator: missing both aggregates and group-by expressions",
        );

        run_test_failure(
            "summarize by resource",
            "Cannot refer to a root map directly in a group-by expression",
        );

        run_test_failure(
            "summarize by Attributes[tostring(now())]",
            "Could not determine the name for summary group-by expression. Try using assignment syntax instead (Name = [expression]).",
        );

        run_test_failure(
            "summarize by Attributes['array'][0]",
            "Could not determine the name for summary group-by expression. Try using assignment syntax instead (Name = [expression]).",
        );

        run_test_failure(
            "summarize by const",
            "Could not determine the source and/or name for summary group-by expression. Try using assignment syntax instead (Name = [expression]).",
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
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
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
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
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
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
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
