use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    slice::Iter,
};

use data_engine_expressions::*;

use crate::{
    execution_context::ExecutionContext, resolved_value_mut::*,
    scalar_expressions::execute_scalar_expression,
    value_expressions::execute_mutable_value_expression, *,
};

pub fn execute_map_reduce_transform_expression<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    reduce_map_transform_expression: &'a ReduceMapTransformExpression,
) -> Result<(), ExpressionError> {
    match reduce_map_transform_expression {
        ReduceMapTransformExpression::Remove(r) => {
            let reduction = resolve_map_reduction(execution_context, r)?;

            let target = execute_mutable_value_expression(execution_context, r.get_target())?;

            if let Some(ResolvedValueMut::Map(mut m)) = target {
                m.retain(&mut KeyValueMutClosureCallback::new(|k, mut v| {
                    for p in &reduction.key_patterns {
                        if p.get_value().is_match(k) {
                            if execution_context.is_enabled(LogLevel::Verbose) {
                                execution_context.log(LogMessage::new(
                                    LogLevel::Verbose,
                                    r,
                                    format!("Removing key '{k}' due to pattern match"),
                                ));
                            }

                            return false;
                        }
                    }

                    if let Some(inner_reduction) = reduction.keys.get(&MapReductionKey::Key(k)) {
                        let v = v.to_value_mut();
                        let remove = match v {
                            Some(ValueMut::Map(m)) => {
                                if !inner_reduction.keys.is_empty() {
                                    if execution_context.is_enabled(LogLevel::Verbose) {
                                        execution_context.log(LogMessage::new(
                                            LogLevel::Verbose,
                                            r,
                                            format!("Processing sub-keys of '{k}' map because inner reduction rules were specified"),
                                        ));
                                    }
                                    remove_from_map(execution_context, r, m, inner_reduction);
                                    false
                                }
                                else {
                                    true
                                }
                            }
                            Some(ValueMut::Array(a)) => {
                                if !inner_reduction.indices.is_empty() {
                                    if execution_context.is_enabled(LogLevel::Verbose) {
                                        execution_context.log(LogMessage::new(
                                            LogLevel::Verbose,
                                            r,
                                            format!("Processing sub-items of '{k}' array because inner reduction rules were specified"),
                                        ));
                                    }
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
                            if execution_context.is_enabled(LogLevel::Verbose) {
                                execution_context.log(LogMessage::new(
                                    LogLevel::Verbose,
                                    r,
                                    format!("Removing '{k}' due to key match"),
                                ));
                            }
                            return false;
                        }
                    }

                    true
                }));
            } else if execution_context.is_enabled(LogLevel::Warn) {
                execution_context.log(LogMessage::new(
                    LogLevel::Warn,
                    r,
                    "Map reduction target was not a map".into(),
                ));
            }

            Ok(())
        }
        ReduceMapTransformExpression::Retain(r) => {
            let reduction = resolve_map_reduction(execution_context, r)?;

            let target = execute_mutable_value_expression(execution_context, r.get_target())?;

            if let Some(ResolvedValueMut::Map(mut m)) = target {
                m.retain(&mut KeyValueMutClosureCallback::new(|k, mut v| {
                    for p in &reduction.key_patterns {
                        if p.get_value().is_match(k) {
                            return true;
                        }
                    }

                    if let Some(inner_reduction) = reduction.keys.get(&MapReductionKey::Key(k)) {
                        let v = v.to_value_mut();
                        match v {
                            Some(ValueMut::Map(m)) => {
                                if !inner_reduction.keys.is_empty() {
                                    if execution_context.is_enabled(LogLevel::Verbose) {
                                        execution_context.log(LogMessage::new(
                                            LogLevel::Verbose,
                                            r,
                                            format!("Processing sub-keys of '{k}' map because inner reduction rules were specified"),
                                        ));
                                    }
                                    keep_in_map(execution_context, r, m, inner_reduction);
                                }
                            }
                            Some(ValueMut::Array(a)) => {
                                if !inner_reduction.indices.is_empty() {
                                    if execution_context.is_enabled(LogLevel::Verbose) {
                                        execution_context.log(LogMessage::new(
                                            LogLevel::Verbose,
                                            r,
                                            format!("Processing sub-items of '{k}' array because inner reduction rules were specified"),
                                        ));
                                    }
                                    keep_in_array(execution_context, r, a, inner_reduction);
                                }
                            }
                            _ => {}
                        }

                        return true;
                    }

                    if execution_context.is_enabled(LogLevel::Verbose) {
                        execution_context.log(LogMessage::new(
                            LogLevel::Verbose,
                            r,
                            format!("Removing key '{k}' because no rules matched"),
                        ));
                    }

                    false
                }));
            } else if execution_context.is_enabled(LogLevel::Warn) {
                execution_context.log(LogMessage::new(
                    LogLevel::Warn,
                    r,
                    "Map reduction target was not a map".into(),
                ));
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
                let value = execute_scalar_expression(execution_context, s)?;

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
                } else if execution_context.is_enabled(LogLevel::Warn) {
                    execution_context.log(LogMessage::new(
                            LogLevel::Warn,
                            s,
                            format!("Key or pattern specified in map reduction expression with '{value_type:?}' type is not supported"),
                        ));
                }
            }
            MapSelector::ValueAccessor(a) => process_map_reduction_accessor(
                execution_context,
                a.get_selectors().iter(),
                &mut reduction,
            )?,
        }
    }

    Ok(reduction)
}

fn process_map_reduction_accessor<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    mut selectors: Iter<'a, ScalarExpression>,
    current_reduction: &mut MapReduction<'c>,
) -> Result<(), ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    if let Some(selector) = selectors.next() {
        let value = execute_scalar_expression(execution_context, selector)?;

        let value_type = value.get_value_type();

        if value_type == ValueType::String {
            let key = MapReductionKey::Resolved(value.try_resolve_string().unwrap());
            if let Some(t) = current_reduction.keys.get_mut(&key) {
                return process_map_reduction_accessor(execution_context, selectors, t);
            } else {
                let t = current_reduction
                    .keys
                    .entry(key)
                    .or_insert(MapReduction::new());
                return process_map_reduction_accessor(execution_context, selectors, t);
            }
        } else if let Value::Integer(i) = value.to_value() {
            let index = i.get_value();
            if let Some(t) = current_reduction.indices.get_mut(&index) {
                return process_map_reduction_accessor(execution_context, selectors, t);
            } else {
                let t = current_reduction
                    .indices
                    .entry(index)
                    .or_insert(MapReduction::new());
                return process_map_reduction_accessor(execution_context, selectors, t);
            }
        } else if execution_context.is_enabled(LogLevel::Warn) {
            execution_context.log(LogMessage::new(
                    LogLevel::Warn,
                    selector,
                    format!("Value with '{value_type:?}' type specified in map reduction accessor expression is not supported"),
                ));
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
    map.retain(&mut KeyValueMutClosureCallback::new(|k, mut v| {
        if let Some(inner_reduction) = reduction.keys.get(&MapReductionKey::Key(k)) {
            let v = v.to_value_mut();
            let remove = match v {
                Some(ValueMut::Map(m)) => {
                    if !inner_reduction.keys.is_empty() {
                        if execution_context.is_enabled(LogLevel::Verbose) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Verbose,
                                expression,
                                format!("Processing sub-keys of '{k}' map because inner reduction rules were specified"),
                            ));
                        }
                        remove_from_map(execution_context, expression, m, inner_reduction);
                        false
                    }
                    else {
                        true
                    }
                }
                Some(ValueMut::Array(a)) => {
                    if !inner_reduction.indices.is_empty() {
                        if execution_context.is_enabled(LogLevel::Verbose) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Verbose,
                                expression,
                                format!("Processing sub-items of '{k}' array because inner reduction rules were specified"),
                            ));
                        }
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
                if execution_context.is_enabled(LogLevel::Verbose) {
                    execution_context.log(LogMessage::new(
                        LogLevel::Verbose,
                        expression,
                        format!("Removing '{k}' due to key match"),
                    ));
                }
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
            } else if execution_context.is_enabled(LogLevel::Warn) {
                execution_context.log(LogMessage::new(
                    LogLevel::Warn,
                    expression,
                    format!("Duplicate rules for index '{i}' were specified and ignored"),
                ));
            }
        }
    }

    array.retain(&mut IndexValueMutClosureCallback::new(|i, mut v| {
        if let Some(inner_reduction) = elements.get(&i) {
            let v = v.to_value_mut();
            let remove = match v {
                Some(ValueMut::Map(m)) => {
                    if !inner_reduction.keys.is_empty() {
                        if execution_context.is_enabled(LogLevel::Verbose) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Verbose,
                                expression,
                                format!("Processing sub-keys of '{i}' because inner reduction rules were specified"),
                            ));
                        }
                        remove_from_map(execution_context, expression, m, inner_reduction);
                        false
                    }
                    else {
                        true
                    }
                }
                Some(ValueMut::Array(a)) => {
                    if !inner_reduction.indices.is_empty() {
                        if execution_context.is_enabled(LogLevel::Verbose) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Verbose,
                                expression,
                                format!("Processing sub-items of '{i}' because inner reduction rules were specified"),
                            ));
                        }
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
                if execution_context.is_enabled(LogLevel::Verbose) {
                    execution_context.log(LogMessage::new(
                        LogLevel::Verbose,
                        expression,
                        format!("Removing '{i}' due to index match"),
                    ));
                }
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
    map.retain(&mut KeyValueMutClosureCallback::new(|k, mut v| {
        if let Some(inner_reduction) = reduction.keys.get(&MapReductionKey::Key(k)) {
            let v = v.to_value_mut();
            match v {
                Some(ValueMut::Map(m)) => {
                    if !inner_reduction.keys.is_empty() {
                        if execution_context.is_enabled(LogLevel::Verbose) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Verbose,
                                expression,
                                format!("Processing sub-keys of '{k}' map because inner reduction rules were specified"),
                            ));
                        }
                        keep_in_map(execution_context, expression, m, inner_reduction);
                    }
                }
                Some(ValueMut::Array(a)) => {
                    if !inner_reduction.indices.is_empty() {
                        if execution_context.is_enabled(LogLevel::Verbose) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Verbose,
                                expression,
                                format!("Processing sub-items of '{k}' array because inner reduction rules were specified"),
                            ));
                        }
                        keep_in_array(execution_context, expression, a, inner_reduction);
                    }
                }
                _ => { }
            }

            return true;
        }

        if execution_context.is_enabled(LogLevel::Verbose) {
            execution_context.log(LogMessage::new(
                LogLevel::Verbose,
                expression,
                format!("Removing key '{k}' because no rules matched"),
            ));
        }

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
            } else if execution_context.is_enabled(LogLevel::Warn) {
                execution_context.log(LogMessage::new(
                    LogLevel::Warn,
                    expression,
                    format!("Duplicate rules for index '{i}' were specified and ignored"),
                ));
            }
        }
    }

    array.retain(&mut IndexValueMutClosureCallback::new(|i, mut v| {
        if let Some(inner_reduction) = elements.get(&i) {
            let v = v.to_value_mut();
            match v {
                Some(ValueMut::Map(m)) => {
                    if !inner_reduction.keys.is_empty() {
                        if execution_context.is_enabled(LogLevel::Verbose) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Verbose,
                                expression,
                                format!("Processing sub-keys of '{i}' because inner reduction rules were specified"),
                            ));
                        }
                        keep_in_map(execution_context, expression, m, inner_reduction);
                    }
                }
                Some(ValueMut::Array(a)) => {
                    if !inner_reduction.indices.is_empty() {
                        if execution_context.is_enabled(LogLevel::Verbose) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Verbose,
                                expression,
                                format!("Processing sub-items of '{i}' because inner reduction rules were specified"),
                            ));
                        }
                        keep_in_array(execution_context, expression, a, inner_reduction);
                    }
                }
                _ => {}
            }

            return true;
        }

        if execution_context.is_enabled(LogLevel::Verbose) {
            execution_context.log(LogMessage::new(
                LogLevel::Verbose,
                expression,
                format!("Removing '{i}' because no rules matched"),
            ));
        }

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
                        OwnedValue::String(ValueStorage::new("hello".into())),
                    ),
                    (
                        "subkey2".into(),
                        OwnedValue::String(ValueStorage::new("world".into())),
                    ),
                    (
                        "subkey3".into(),
                        OwnedValue::String(ValueStorage::new("goodbye".into())),
                    ),
                ]))),
            )
            .with_key_value(
                "key2".into(),
                OwnedValue::Array(ArrayValueStorage::new(vec![
                    OwnedValue::Integer(ValueStorage::new(1)),
                    OwnedValue::Integer(ValueStorage::new(2)),
                    OwnedValue::Integer(ValueStorage::new(3)),
                ])),
            );

        let run_test = |transform_expression| {
            let pipeline = PipelineExpressionBuilder::new("set")
                .with_expressions(vec![DataExpression::Transform(transform_expression)])
                .build()
                .unwrap();

            let execution_context =
                ExecutionContext::new(LogLevel::Verbose, &pipeline, None, record.clone());

            if let DataExpression::Transform(t) = &pipeline.get_expressions()[0] {
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

        assert_eq!(1, result.len());
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
            "{\"key1\":{\"subkey2\":\"world\"},\"key2\":[2]}",
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
}
