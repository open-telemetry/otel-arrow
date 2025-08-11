use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Display, Write},
    hash::Hash,
    slice::Iter,
};

use data_engine_expressions::*;

use crate::{
    execution_context::*, resolved_value_mut::*, scalars::*,
    value_expressions::execute_mutable_value_expression, *,
};

pub fn execute_map_reduce_transform_expression<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    reduce_map_transform_expression: &'a ReduceMapTransformExpression,
) -> Result<(), ExpressionError> {
    match reduce_map_transform_expression {
        ReduceMapTransformExpression::Remove(r) => {
            let target = r.get_target();

            let reduction = resolve_map_reduction(execution_context, target, r)?;

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                reduce_map_transform_expression,
                || format!("Resolved map reduction: {reduction}"),
            );

            let target = execute_mutable_value_expression(execution_context, target)?;

            if let Some(ResolvedValueMut::Map(mut m)) = target {
                m.retain(&mut KeyValueMutClosureCallback::new(|k, v| {
                    for p in &reduction.key_patterns {
                        if p.get_value().is_match(k) {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                r,
                                || format!("Removing key '{k}' due to pattern match"),
                            );
                            return false;
                        }
                    }

                    if let Some(inner_reduction) = reduction.keys.get(&MapReductionKey::Key(k)) {
                        let v = v.to_static_value_mut();
                        let remove = match v {
                            Some(StaticValueMut::Map(m)) => {
                                if !inner_reduction.key_patterns.is_empty() || !inner_reduction.keys.is_empty() {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Verbose,
                                        r,
                                        || format!("Processing sub-keys of '{k}' map because inner reduction rules were specified"),
                                    );
                                    remove_from_map(execution_context, r, m, inner_reduction);
                                    false
                                }
                                else {
                                    true
                                }
                            }
                            Some(StaticValueMut::Array(a)) => {
                                if !inner_reduction.indices.is_empty() {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Verbose,
                                        r,
                                        || format!("Processing sub-items of '{k}' array because inner reduction rules were specified"),
                                    );
                                    remove_from_array(execution_context, r, a, inner_reduction);
                                    false
                                }
                                else {
                                    true
                                }
                            }
                            _ => true
                        };

                        if remove {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                r,
                                || format!("Removing '{k}' due to key match"),
                            );
                            return false;
                        }
                    }

                    true
                }));
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    r,
                    || "Map reduction target was not a map".into(),
                );
            }

            Ok(())
        }
        ReduceMapTransformExpression::Retain(r) => {
            let target = r.get_target();

            let reduction = resolve_map_reduction(execution_context, target, r)?;

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                reduce_map_transform_expression,
                || format!("Resolved map reduction: {reduction}"),
            );

            let target = execute_mutable_value_expression(execution_context, target)?;

            if let Some(ResolvedValueMut::Map(mut m)) = target {
                m.retain(&mut KeyValueMutClosureCallback::new(|k, v| {
                    for p in &reduction.key_patterns {
                        if p.get_value().is_match(k) {
                            return true;
                        }
                    }

                    if let Some(inner_reduction) = reduction.keys.get(&MapReductionKey::Key(k)) {
                        let v = v.to_static_value_mut();
                        match v {
                            Some(StaticValueMut::Map(m)) => {
                                if !inner_reduction.key_patterns.is_empty() || !inner_reduction.keys.is_empty() {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Verbose,
                                        r,
                                        || format!("Processing sub-keys of '{k}' map because inner reduction rules were specified"),
                                    );
                                    keep_in_map(execution_context, r, m, inner_reduction);
                                }
                            }
                            Some(StaticValueMut::Array(a)) => {
                                if !inner_reduction.indices.is_empty() {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Verbose,
                                        r,
                                        || format!("Processing sub-items of '{k}' array because inner reduction rules were specified"),
                                    );
                                    keep_in_array(execution_context, r, a, inner_reduction);
                                }
                            }
                            _ => {}
                        }

                        return true;
                    }

                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        r,
                        || format!("Removing key '{k}' because no rules matched"),
                    );

                    false
                }));
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    r,
                    || "Map reduction target was not a map".into(),
                );
            }

            Ok(())
        }
    }
}

#[derive(Debug)]
struct MapReduction<'a> {
    keys: HashMap<MapReductionKey<'a>, MapReduction<'a>>,
    key_patterns: Vec<ResolvedRegexValue<'a>>,
    indices: HashMap<i64, MapReduction<'a>>,
}

impl<'a> MapReduction<'a> {
    pub fn new() -> MapReduction<'a> {
        Self {
            keys: HashMap::new(),
            key_patterns: Vec::new(),
            indices: HashMap::new(),
        }
    }
}

impl Display for MapReduction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;

        if !self.keys.is_empty() {
            f.write_str(" keys: [ ")?;

            let mut first = true;
            for (k, r) in &self.keys {
                if first {
                    first = false;
                } else {
                    f.write_str(", ")?;
                }

                f.write_str("{ name: ")?;
                f.write_str(k.get_value())?;
                f.write_str(", reduction: ")?;
                r.fmt(f)?;
                f.write_str(" }")?;
            }

            f.write_str(" ] ")?;
        }

        if !self.key_patterns.is_empty() {
            f.write_str(" key_patterns: [ ")?;

            let mut first = true;
            for r in &self.key_patterns {
                if first {
                    first = false;
                } else {
                    f.write_str(", ")?;
                }

                r.get_value().fmt(f)?;
            }

            f.write_str(" ] ")?;
        }

        if !self.indices.is_empty() {
            f.write_str(" indices: [ ")?;

            let mut first = true;
            for (i, r) in &self.indices {
                if first {
                    first = false;
                } else {
                    f.write_str(", ")?;
                }

                f.write_str("{ index: ")?;
                i.fmt(f)?;
                f.write_str(", reduction: ")?;
                r.fmt(f)?;
                f.write_str(" }")?;
            }

            f.write_str(" ] ")?;
        }

        f.write_char('}')
    }
}

#[derive(Debug)]
enum MapReductionKey<'a> {
    Key(&'a str),
    Resolved(ResolvedStringValue<'a>),
}

impl MapReductionKey<'_> {
    pub fn get_value(&self) -> &str {
        match self {
            MapReductionKey::Key(k) => k,
            MapReductionKey::Resolved(s) => s.get_value(),
        }
    }
}

impl Hash for MapReductionKey<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_value().hash(state);
    }
}

impl PartialEq for MapReductionKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.get_value() == other.get_value()
    }
}

impl Eq for MapReductionKey<'_> {}

fn resolve_map_reduction<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    target: &'a MutableValueExpression,
    map_selection: &'a MapSelectionExpression,
) -> Result<MapReduction<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let mut reduction = MapReduction::new();

    for selector in map_selection.get_selectors() {
        match selector {
            MapSelector::KeyOrKeyPattern(s) => {
                let mut value = execute_scalar_expression(execution_context, s)?;

                if value.copy_if_borrowed_from_target(target) {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        target,
                        || format!("Copied the resolved key or pattern value '{value}' into temporary storage because the value came from the mutable target"));
                }

                let value_type = value.get_value_type();

                if value_type == ValueType::Regex {
                    reduction
                        .key_patterns
                        .push(value.try_resolve_regex().unwrap())
                } else if value_type == ValueType::String {
                    let key = MapReductionKey::Resolved(value.try_resolve_string().unwrap());
                    reduction.keys.entry(key).or_insert_with(MapReduction::new);
                } else if let Value::Integer(i) = value.to_value() {
                    let index = i.get_value();
                    reduction
                        .indices
                        .entry(index)
                        .or_insert_with(MapReduction::new);
                } else {
                    execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            s,
                            || format!("Key or pattern specified in map reduction expression with '{value_type:?}' type is not supported"),
                        );
                }
            }
            MapSelector::ValueAccessor(a) => process_map_reduction_accessor(
                execution_context,
                target,
                a.get_selectors().iter(),
                &mut reduction,
            )?,
        }
    }

    Ok(reduction)
}

fn process_map_reduction_accessor<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    target: &'a MutableValueExpression,
    mut selectors: Iter<'a, ScalarExpression>,
    current_reduction: &mut MapReduction<'c>,
) -> Result<(), ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    if let Some(selector) = selectors.next() {
        let mut value = execute_scalar_expression(execution_context, selector)?;

        if value.copy_if_borrowed_from_target(target) {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                target,
                || format!("Copied the resolved accessor value '{value}' into temporary storage because the value came from the mutable target"));
        }

        let value_type = value.get_value_type();

        if value_type == ValueType::String {
            let key = MapReductionKey::Resolved(value.try_resolve_string().unwrap());
            if let Some(t) = current_reduction.keys.get_mut(&key) {
                return process_map_reduction_accessor(execution_context, target, selectors, t);
            } else {
                let t = current_reduction
                    .keys
                    .entry(key)
                    .or_insert(MapReduction::new());
                return process_map_reduction_accessor(execution_context, target, selectors, t);
            }
        } else if let Value::Integer(i) = value.to_value() {
            let index = i.get_value();
            if let Some(t) = current_reduction.indices.get_mut(&index) {
                return process_map_reduction_accessor(execution_context, target, selectors, t);
            } else {
                let t = current_reduction
                    .indices
                    .entry(index)
                    .or_insert(MapReduction::new());
                return process_map_reduction_accessor(execution_context, target, selectors, t);
            }
        } else if let Value::Regex(_) = value.to_value() {
            current_reduction
                .key_patterns
                .push(value.try_resolve_regex().unwrap());
        } else {
            execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    selector,
                    || format!("Value with '{value_type:?}' type specified in map reduction accessor expression is not supported"),
                );
        }
    }

    Ok(())
}

fn remove_from_map<'a, TRecord: Record + 'static>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    expression: &'a dyn Expression,
    map: &mut dyn MapValueMut,
    reduction: &MapReduction<'_>,
) {
    map.retain(&mut KeyValueMutClosureCallback::new(|k, v| {
        if !reduction.key_patterns.is_empty() {
            for p in &reduction.key_patterns {
                if p.get_value().is_match(k) {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        expression,
                        || format!("Removing '{k}' due to pattern match"),
                    );
                    return false;
                }
            }
        }

        if let Some(inner_reduction) = reduction.keys.get(&MapReductionKey::Key(k)) {
            let v = v.to_static_value_mut();
            let remove = match v {
                Some(StaticValueMut::Map(m)) => {
                    if !inner_reduction.key_patterns.is_empty() || !inner_reduction.keys.is_empty() {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            expression,
                            || format!("Processing sub-keys of '{k}' map because inner reduction rules were specified"),
                        );
                        remove_from_map(execution_context, expression, m, inner_reduction);
                        false
                    }
                    else {
                        true
                    }
                }
                Some(StaticValueMut::Array(a)) => {
                    if !inner_reduction.indices.is_empty() {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            expression,
                            || format!("Processing sub-items of '{k}' array because inner reduction rules were specified"),
                        );
                        remove_from_array(execution_context, expression, a, inner_reduction);
                        false
                    }
                    else {
                        true
                    }
                }
                _ => true
            };

            if remove {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    expression,
                    || format!("Removing '{k}' due to key match"),
                );
                return false;
            }
        }
        true
    }));
}

fn remove_from_array<'a, TRecord: Record + 'static>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    expression: &'a dyn Expression,
    array: &mut dyn ArrayValueMut,
    reduction: &MapReduction<'_>,
) {
    let mut elements = BTreeMap::new();
    let length = array.len() as i64;

    for (i, r) in &reduction.indices {
        let mut index = *i;

        if index < 0 {
            index += length;
        }

        if index >= 0 && index < length {
            let final_index = index as usize;
            if let std::collections::btree_map::Entry::Vacant(e) = elements.entry(final_index) {
                e.insert(r);
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    expression,
                    || format!("Duplicate rules for index '{i}' were specified and ignored"),
                );
            }
        }
    }

    array.retain(&mut IndexValueMutClosureCallback::new(|i, v| {
        if let Some(inner_reduction) = elements.get(&i) {
            let v = v.to_static_value_mut();
            let remove = match v {
                Some(StaticValueMut::Map(m)) => {
                    if !inner_reduction.key_patterns.is_empty() || !inner_reduction.keys.is_empty() {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            expression,
                            || format!("Processing sub-keys of '{i}' because inner reduction rules were specified"),
                        );
                        remove_from_map(execution_context, expression, m, inner_reduction);
                        false
                    }
                    else {
                        true
                    }
                }
                Some(StaticValueMut::Array(a)) => {
                    if !inner_reduction.indices.is_empty() {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            expression,
                            || format!("Processing sub-items of '{i}' because inner reduction rules were specified"),
                        );
                        remove_from_array(execution_context, expression, a, inner_reduction);
                        false
                    }
                    else {
                        true
                    }
                }
                _ => true
            };

            if remove {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    expression,
                    || format!("Removing '{i}' due to index match"),
                );
                return false;
            }
        }

        true
    }));
}

fn keep_in_map<'a, TRecord: Record + 'static>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    expression: &'a dyn Expression,
    map: &mut dyn MapValueMut,
    reduction: &MapReduction<'_>,
) {
    map.retain(&mut KeyValueMutClosureCallback::new(|k, v| {
        if !reduction.key_patterns.is_empty() {
            for p in &reduction.key_patterns {
                if p.get_value().is_match(k) {
                    return true;
                }
            }
        }

        if let Some(inner_reduction) = reduction.keys.get(&MapReductionKey::Key(k)) {
            let v = v.to_static_value_mut();
            match v {
                Some(StaticValueMut::Map(m)) => {
                    if !inner_reduction.key_patterns.is_empty() || !inner_reduction.keys.is_empty() {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            expression,
                            || format!("Processing sub-keys of '{k}' map because inner reduction rules were specified"),
                        );
                        keep_in_map(execution_context, expression, m, inner_reduction);
                    }
                }
                Some(StaticValueMut::Array(a)) => {
                    if !inner_reduction.indices.is_empty() {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            expression,
                            || format!("Processing sub-items of '{k}' array because inner reduction rules were specified"),
                        );
                        keep_in_array(execution_context, expression, a, inner_reduction);
                    }
                }
                _ => { }
            }

            return true;
        }

        execution_context.add_diagnostic_if_enabled(
            RecordSetEngineDiagnosticLevel::Verbose,
            expression,
            || format!("Removing key '{k}' because no rules matched"),
        );

        false
    }));
}

fn keep_in_array<'a, TRecord: Record + 'static>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    expression: &'a dyn Expression,
    array: &mut dyn ArrayValueMut,
    reduction: &MapReduction<'_>,
) {
    let mut elements = BTreeMap::new();
    let length = array.len() as i64;

    for (i, r) in &reduction.indices {
        let mut index = *i;

        if index < 0 {
            index += length;
        }

        if index >= 0 && index < length {
            let final_index = index as usize;
            if let std::collections::btree_map::Entry::Vacant(e) = elements.entry(final_index) {
                e.insert(r);
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    expression,
                    || format!("Duplicate rules for index '{i}' were specified and ignored"),
                );
            }
        }
    }

    array.retain(&mut IndexValueMutClosureCallback::new(|i, v| {
        if let Some(inner_reduction) = elements.get(&i) {
            let v = v.to_static_value_mut();
            match v {
                Some(StaticValueMut::Map(m)) => {
                    if !inner_reduction.key_patterns.is_empty() || !inner_reduction.keys.is_empty() {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            expression,
                            || format!("Processing sub-keys of '{i}' because inner reduction rules were specified"),
                        );
                        keep_in_map(execution_context, expression, m, inner_reduction);
                    }
                }
                Some(StaticValueMut::Array(a)) => {
                    if !inner_reduction.indices.is_empty() {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            expression,
                            || format!("Processing sub-items of '{i}' because inner reduction rules were specified"),
                        );
                        keep_in_array(execution_context, expression, a, inner_reduction);
                    }
                }
                _ => {}
            }

            return true;
        }

        execution_context.add_diagnostic_if_enabled(
            RecordSetEngineDiagnosticLevel::Verbose,
            expression,
            || format!("Removing '{i}' because no rules matched"),
        );

        false
    }));
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use crate::transform::transform_expressions::execute_transform_expression;

    use super::*;

    #[test]
    fn test_execute_reduce_map_transform_expression() {
        let record = TestRecord::new()
            .with_key_value(
                "key1".into(),
                OwnedValue::Map(MapValueStorage::new(HashMap::from([
                    (
                        "subkey1".into(),
                        OwnedValue::String(StringValueStorage::new("hello".into())),
                    ),
                    (
                        "subkey2".into(),
                        OwnedValue::String(StringValueStorage::new("world".into())),
                    ),
                    (
                        "subkey3".into(),
                        OwnedValue::String(StringValueStorage::new("goodbye".into())),
                    ),
                ]))),
            )
            .with_key_value(
                "key2".into(),
                OwnedValue::Array(ArrayValueStorage::new(vec![
                    OwnedValue::Integer(IntegerValueStorage::new(1)),
                    OwnedValue::Integer(IntegerValueStorage::new(2)),
                    OwnedValue::Integer(IntegerValueStorage::new(3)),
                ])),
            )
            .with_key_value(
                "key_to_remove".into(),
                OwnedValue::String(StringValueStorage::new("key1".into())),
            );

        let run_test = |transform_expression| {
            let mut test = TestExecutionContext::new()
                .with_pipeline(
                    PipelineExpressionBuilder::new("set")
                        .with_expressions(vec![DataExpression::Transform(transform_expression)])
                        .build()
                        .unwrap(),
                )
                .with_record(record.clone());

            let execution_context = test.create_execution_context();

            if let DataExpression::Transform(t) =
                &execution_context.get_pipeline().get_expressions()[0]
            {
                execute_transform_expression(&execution_context, t).unwrap();
            } else {
                panic!("Unexpected expression");
            }

            let result = execution_context.consume_into_record();

            println!("{result}");

            result.take_record()
        };

        // Test removing a key
        let result = run_test(TransformExpression::ReduceMap(
            ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![MapSelector::KeyOrKeyPattern(ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                ))],
            )),
        ));

        assert_eq!(2, result.len());
        assert!(!result.contains_key("key1"));

        // Test removing a key using key referencing data on source
        let result = run_test(TransformExpression::ReduceMap(
            ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![MapSelector::KeyOrKeyPattern(ScalarExpression::Source(
                    SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "key_to_remove",
                            )),
                        )]),
                    ),
                ))],
            )),
        ));

        assert_eq!(2, result.len());
        assert!(!result.contains_key("key1"));

        // Test removing a key using accessor referencing data on source
        let result = run_test(TransformExpression::ReduceMap(
            ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![MapSelector::ValueAccessor(
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Source(
                        SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "key_to_remove",
                                )),
                            )]),
                        ),
                    )]),
                )],
            )),
        ));

        assert_eq!(2, result.len());
        assert!(!result.contains_key("key1"));

        // Test removing keys using regex
        let result = run_test(TransformExpression::ReduceMap(
            ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![MapSelector::KeyOrKeyPattern(ScalarExpression::Static(
                    StaticScalarExpression::Regex(RegexScalarExpression::new(
                        QueryLocation::new_fake(),
                        Regex::new(r"key.*").unwrap(),
                    )),
                ))],
            )),
        ));

        assert_eq!(0, result.len());

        // Test removing keys and array items using path
        let result = run_test(TransformExpression::ReduceMap(
            ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "subkey1"),
                        )),
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "subkey3"),
                        )),
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "subkey3"),
                        )), // dupe just to make sure it is ignored
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
                        )),
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )),
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )), // dupe just to make sure it is ignored
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), -3),
                        )), // this should resolve to 0 which is also a dupe
                    ])),
                ],
            )),
        ));

        assert_eq!(
            "{\"key1\":{\"subkey2\":\"world\"},\"key2\":[2],\"key_to_remove\":\"key1\"}",
            Value::Map(&result).to_string()
        );

        // Test keeping keys and array items using path
        let result = run_test(TransformExpression::ReduceMap(
            ReduceMapTransformExpression::Retain(MapSelectionExpression::new_with_selectors(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "subkey1"),
                        )),
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "subkey3"),
                        )),
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "subkey3"),
                        )), // dupe just to make sure it is ignored
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
                        )),
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )),
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )), // dupe just to make sure it is ignored
                    ])),
                    MapSelector::ValueAccessor(ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), -3),
                        )), // this should resolve to 0 which is also a dupe
                    ])),
                ],
            )),
        ));

        assert_eq!(
            "{\"key1\":{\"subkey1\":\"hello\",\"subkey3\":\"goodbye\"},\"key2\":[1,3]}",
            Value::Map(&result).to_string()
        );
    }

    #[test]
    fn test_execute_reduce_map_transform_with_inner_regex_expression() {
        let record = TestRecord::new()
            .with_key_value(
                "key1".into(),
                OwnedValue::Map(MapValueStorage::new(HashMap::from([
                    (
                        "subkey1".into(),
                        OwnedValue::String(StringValueStorage::new("hello".into())),
                    ),
                    (
                        "subkey2".into(),
                        OwnedValue::String(StringValueStorage::new("world".into())),
                    ),
                    (
                        "subkey3".into(),
                        OwnedValue::String(StringValueStorage::new("goodbye".into())),
                    ),
                ]))),
            )
            .with_key_value(
                "key2".into(),
                OwnedValue::Array(ArrayValueStorage::new(vec![
                    OwnedValue::Integer(IntegerValueStorage::new(1)),
                    OwnedValue::Integer(IntegerValueStorage::new(2)),
                    OwnedValue::Integer(IntegerValueStorage::new(3)),
                ])),
            );

        let run_test = |transform_expression| {
            let mut test = TestExecutionContext::new()
                .with_pipeline(
                    PipelineExpressionBuilder::new("set")
                        .with_expressions(vec![DataExpression::Transform(transform_expression)])
                        .build()
                        .unwrap(),
                )
                .with_record(record.clone());

            let execution_context = test.create_execution_context();

            if let DataExpression::Transform(t) =
                &execution_context.get_pipeline().get_expressions()[0]
            {
                execute_transform_expression(&execution_context, t).unwrap();
            } else {
                panic!("Unexpected expression");
            }

            let result = execution_context.consume_into_record();

            println!("{result}");

            result.take_record()
        };

        // Test removing a key
        let result = run_test(TransformExpression::ReduceMap(
            ReduceMapTransformExpression::Remove(MapSelectionExpression::new_with_selectors(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![MapSelector::ValueAccessor(
                    ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Regex(
                            RegexScalarExpression::new(
                                QueryLocation::new_fake(),
                                Regex::new("^sub.*").unwrap(),
                            ),
                        )),
                    ]),
                )],
            )),
        ));

        assert_eq!(2, result.len());
        assert!(result.contains_key("key1"));

        if let Some(Value::Map(m)) = result.get("key1").map(|v| v.to_value()) {
            assert_eq!(0, m.len());
        } else {
            panic!()
        }
    }
}
