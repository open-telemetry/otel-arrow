use std::{
    cell::RefMut,
    collections::HashSet,
    fmt::{Display, Write},
};

use data_engine_expressions::*;

use crate::{
    execution_context::ExecutionContext,
    resolved_value_mut::*,
    scalar_expressions::execute_scalar_expression,
    transform::reduce_map_transform_expression::execute_map_reduce_transform_expression,
    value_expressions::{execute_immutable_value_expression, execute_mutable_value_expression},
    *,
};

pub fn execute_transform_expression<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    transform_expression: &'a TransformExpression,
) -> Result<(), ExpressionError> {
    match transform_expression {
        TransformExpression::Set(s) => {
            let source = execute_immutable_value_expression(execution_context, s.get_source())?;

            let destination =
                execute_mutable_value_expression(execution_context, s.get_destination())?;

            match destination {
                Some(d) => match d {
                    ResolvedValueMut::Map(_) => {
                        if execution_context.is_enabled(LogLevel::Warn) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Warn,
                                s,
                                "Cannot set root map".into(),
                            ));
                        }
                    }
                    ResolvedValueMut::MapKey { mut map, key } => {
                        match map.set(key.get_value(), source) {
                            ValueMutWriteResult::NotFound => {
                                if execution_context.is_enabled(LogLevel::Warn) {
                                    execution_context.log(LogMessage::new(
                                        LogLevel::Warn,
                                        s,
                                        format!("Map key '{key}' could not be found on target map"),
                                    ));
                                }
                            }
                            ValueMutWriteResult::Created => {
                                if execution_context.is_enabled(LogLevel::Verbose) {
                                    execution_context.log(LogMessage::new(
                                        LogLevel::Verbose,
                                        s,
                                        format!("Map key '{key}' created on target map"),
                                    ));
                                }
                            }
                            ValueMutWriteResult::Updated(old) => {
                                if execution_context.is_enabled(LogLevel::Verbose) {
                                    execution_context.log(LogMessage::new(
                                                        LogLevel::Verbose,
                                                        s,
                                                        format!("Map key '{key}' updated on target map replacing old value: {}", old.to_value()),
                                                    ));
                                }
                            }
                            ValueMutWriteResult::NotSupported(e) => {
                                if execution_context.is_enabled(LogLevel::Verbose) {
                                    execution_context.log(LogMessage::new(
                                                        LogLevel::Verbose,
                                                        s,
                                                        format!("Map key '{key}' could not be updated on target map: {e}"),
                                                    ));
                                }
                            }
                        }
                    }
                    ResolvedValueMut::ArrayIndex { mut array, index } => {
                        match array.set(index, source) {
                            ValueMutWriteResult::NotFound => {
                                if execution_context.is_enabled(LogLevel::Warn) {
                                    execution_context.log(LogMessage::new(
                                                        LogLevel::Warn,
                                                        s,
                                                        format!("Array index '{index}' could not be found on target array"),
                                                    ));
                                }
                            }
                            ValueMutWriteResult::Created => {
                                if execution_context.is_enabled(LogLevel::Verbose) {
                                    execution_context.log(LogMessage::new(
                                        LogLevel::Verbose,
                                        s,
                                        format!("Array index '{index}' created on target array"),
                                    ));
                                }
                            }
                            ValueMutWriteResult::Updated(old) => {
                                if execution_context.is_enabled(LogLevel::Verbose) {
                                    execution_context.log(LogMessage::new(
                                                        LogLevel::Verbose,
                                                        s,
                                                        format!("Array index '{index}' updated on target array replacing old value: {}", old.to_value()),
                                                    ));
                                }
                            }
                            ValueMutWriteResult::NotSupported(e) => {
                                if execution_context.is_enabled(LogLevel::Verbose) {
                                    execution_context.log(LogMessage::new(
                                                        LogLevel::Verbose,
                                                        s,
                                                        format!("Array index '{index}' could not be updated on target array: {e}"),
                                                    ));
                                }
                            }
                        }
                    }
                },
                None => {
                    if execution_context.is_enabled(LogLevel::Warn) {
                        execution_context.log(LogMessage::new(
                            LogLevel::Warn,
                            s,
                            "Destination could not be resolved".into(),
                        ));
                    }
                }
            }

            Ok(())
        }
        TransformExpression::Remove(r) => {
            let target = execute_mutable_value_expression(execution_context, r.get_target())?;

            match target {
                Some(d) => match d {
                    ResolvedValueMut::Map(_) => {
                        if execution_context.is_enabled(LogLevel::Warn) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Warn,
                                r,
                                "Cannot remove root map".into(),
                            ));
                        }
                    }
                    ResolvedValueMut::MapKey { mut map, key } => {
                        match map.remove(key.get_value()) {
                            ValueMutRemoveResult::NotFound => {
                                if execution_context.is_enabled(LogLevel::Warn) {
                                    execution_context.log(LogMessage::new(
                                        LogLevel::Warn,
                                        r,
                                        format!("Map key '{key}' could not be found on target map"),
                                    ));
                                }
                            }
                            ValueMutRemoveResult::Removed(old) => {
                                if execution_context.is_enabled(LogLevel::Verbose) {
                                    execution_context.log(LogMessage::new(
                                        LogLevel::Verbose,
                                        r,
                                        format!(
                                            "Removed map key '{key}' on target with value: {}",
                                            old.to_value()
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    ResolvedValueMut::ArrayIndex { mut array, index } => {
                        match array.remove(index) {
                            ValueMutRemoveResult::NotFound => {
                                if execution_context.is_enabled(LogLevel::Warn) {
                                    execution_context.log(LogMessage::new(
                                                                LogLevel::Warn,
                                                                r,
                                                                format!("Array index '{index}' could not be found on target array"),
                                                            ));
                                }
                            }
                            ValueMutRemoveResult::Removed(old) => {
                                if execution_context.is_enabled(LogLevel::Verbose) {
                                    execution_context.log(LogMessage::new(
                                                                LogLevel::Verbose,
                                                                r,
                                                                format!("Removed array index '{index}' on target with value: {}", old.to_value()),
                                                            ));
                                }
                            }
                        }
                    }
                },
                None => {
                    if execution_context.is_enabled(LogLevel::Warn) {
                        execution_context.log(LogMessage::new(
                            LogLevel::Warn,
                            r,
                            "Destination could not be resolved".into(),
                        ));
                    }
                }
            }

            Ok(())
        }
        TransformExpression::ReduceMap(r) => {
            execute_map_reduce_transform_expression(execution_context, r)
        }
        TransformExpression::RemoveMapKeys(r) => match r {
            RemoveMapKeysTransformExpression::Remove(m) => {
                let map_keys = resolve_map_keys(execution_context, m)?;

                if execution_context.is_enabled(LogLevel::Verbose) {
                    execution_context.log(LogMessage::new(
                        LogLevel::Verbose,
                        r,
                        format!("Resolved map keys: {map_keys}"),
                    ));
                }

                let target = execute_mutable_value_expression(execution_context, m.get_target())?;

                match resolve_map_destination(target) {
                    Some(mut target_map) => {
                        for key in map_keys.keys {
                            key.to_value().convert_to_string(&mut |k| {
                                match target_map.remove(k) {
                                    ValueMutRemoveResult::NotFound => {
                                        if execution_context.is_enabled(LogLevel::Warn) {
                                            execution_context.log(LogMessage::new(
                                                    LogLevel::Warn,
                                                    r,
                                                    format!("Map key '{key}' could not be found on target map"),
                                                ));
                                        }
                                    }
                                    ValueMutRemoveResult::Removed(old) => {
                                        if execution_context.is_enabled(LogLevel::Verbose) {
                                            execution_context.log(LogMessage::new(
                                                    LogLevel::Verbose,
                                                    r,
                                                    format!(
                                                        "Removed map key '{key}' on target with value: {}",
                                                        old.to_value()
                                                    ),
                                                ));
                                        }
                                    }
                                }
                            });
                        }
                    }
                    None => {
                        if execution_context.is_enabled(LogLevel::Warn) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Warn,
                                r,
                                "Destination map could not be resolved".into(),
                            ));
                        }
                    }
                }

                Ok(())
            }
            RemoveMapKeysTransformExpression::Retain(m) => {
                let map_keys = resolve_map_keys(execution_context, m)?;

                if execution_context.is_enabled(LogLevel::Verbose) {
                    execution_context.log(LogMessage::new(
                        LogLevel::Verbose,
                        r,
                        format!("Resolved map keys: {map_keys}"),
                    ));
                }

                let target = execute_mutable_value_expression(execution_context, m.get_target())?;

                match resolve_map_destination(target) {
                    Some(mut target_map) => {
                        let mut key_map: HashSet<Box<str>> = HashSet::new();
                        for key in map_keys.keys {
                            key.to_value().convert_to_string(&mut |s| {
                                key_map.insert(s.into());
                            });
                        }
                        target_map.retain(&mut KeyValueMutClosureCallback::new(|k, v| {
                            if key_map.contains(k) {
                                return true;
                            }

                            if execution_context.is_enabled(LogLevel::Verbose) {
                                execution_context.log(LogMessage::new(
                                    LogLevel::Verbose,
                                    r,
                                    format!(
                                        "Removing map key '{k}' from target with value: {}",
                                        v.to_value(),
                                    ),
                                ));
                            }
                            false
                        }));
                    }
                    None => {
                        if execution_context.is_enabled(LogLevel::Warn) {
                            execution_context.log(LogMessage::new(
                                LogLevel::Warn,
                                r,
                                "Destination map could not be resolved".into(),
                            ));
                        }
                    }
                }

                Ok(())
            }
        },
    }
}

struct MapKeys<'a> {
    pub keys: Vec<ResolvedValue<'a>>,
}

impl Display for MapKeys<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;

        let mut first = true;
        for key in &self.keys {
            if first {
                first = false;
            } else {
                f.write_str(", ")?;
            }

            key.to_value().fmt(f)?;
        }

        f.write_char(']')
    }
}

fn resolve_map_keys<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    key_list: &'a MapKeyListExpression,
) -> Result<MapKeys<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let mut keys = Vec::new();

    for key_scalar in key_list.get_keys() {
        let value = execute_scalar_expression(execution_context, key_scalar)?;

        keys.push(value);
    }

    Ok(MapKeys { keys })
}

fn resolve_map_destination<'a>(
    resolved_destination: Option<ResolvedValueMut<'a, '_>>,
) -> Option<RefMut<'a, dyn MapValueMut + 'static>> {
    match resolved_destination {
        None => None,
        Some(v) => match v {
            ResolvedValueMut::Map(m) => Some(m),
            ResolvedValueMut::MapKey { map, key } => RefMut::filter_map(map, |v| {
                if let ValueMutGetResult::Found(v) = v.get_mut(key.get_value())
                    && let Some(ValueMut::Map(m)) = v.to_value_mut()
                {
                    Some(m)
                } else {
                    None
                }
            })
            .ok(),
            ResolvedValueMut::ArrayIndex { array, index } => RefMut::filter_map(array, |v| {
                if let ValueMutGetResult::Found(v) = v.get_mut(index)
                    && let Some(ValueMut::Map(m)) = v.to_value_mut()
                {
                    Some(m)
                } else {
                    None
                }
            })
            .ok(),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_execute_set_source_transform_expression() {
        let record = TestRecord::new()
            .with_key_value(
                "key1".into(),
                OwnedValue::String(ValueStorage::new("value1".into())),
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

        // Test updating a key
        let result = run_test(TransformExpression::Set(SetTransformExpression::new(
            QueryLocation::new_fake(),
            ImmutableValueExpression::Scalar(ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            )),
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
        )));

        assert_eq!(
            Value::String(&ValueStorage::new("hello world".into())),
            result.get("key1").unwrap().to_value()
        );

        // Test adding a key
        let result = run_test(TransformExpression::Set(SetTransformExpression::new(
            QueryLocation::new_fake(),
            ImmutableValueExpression::Scalar(ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            )),
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "new_key",
                    )),
                )]),
            )),
        )));

        assert_eq!(
            Value::String(&ValueStorage::new("hello world".into())),
            result.get("new_key").unwrap().to_value()
        );

        // Test updating an index
        let result = run_test(TransformExpression::Set(SetTransformExpression::new(
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
                        StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                    )),
                ]),
            )),
        )));

        if let Some(Value::Array(a)) = result.get("key2").map(|v| v.to_value()) {
            assert_eq!(
                Value::String(&ValueStorage::new("hello world".into())),
                a.get(0).unwrap().to_value()
            );
        } else {
            panic!("Unexpected results")
        }

        // Test updating an index using negative lookup
        let result = run_test(TransformExpression::Set(SetTransformExpression::new(
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
                        StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
                    )),
                ]),
            )),
        )));

        if let Some(Value::Array(a)) = result.get("key2").map(|v| v.to_value()) {
            assert_eq!(
                Value::String(&ValueStorage::new("hello world".into())),
                a.get(2).unwrap().to_value()
            );
        } else {
            panic!("Unexpected results")
        }
    }

    #[test]
    fn test_execute_set_variable_transform_expression() {
        let record = TestRecord::new();

        let run_test = |transform_expression| {
            let pipeline = PipelineExpressionBuilder::new("set")
                .with_expressions(vec![DataExpression::Transform(transform_expression)])
                .build()
                .unwrap();

            let execution_context =
                ExecutionContext::new(LogLevel::Verbose, &pipeline, None, record.clone());

            execution_context.get_variables().borrow_mut().set(
                "var1",
                ResolvedValue::Computed(OwnedValue::Map(MapValueStorage::new(HashMap::from([(
                    "subkey1".into(),
                    OwnedValue::String(ValueStorage::new("hello world".into())),
                )])))),
            );

            if let DataExpression::Transform(t) = &pipeline.get_expressions()[0] {
                execute_transform_expression(&execution_context, t).unwrap();
            } else {
                panic!("Unexpected expression");
            }

            execution_context
                .get_variables()
                .replace(MapValueStorage::new(HashMap::new()))
        };

        // Test setting a variable
        let result = run_test(TransformExpression::Set(SetTransformExpression::new(
            QueryLocation::new_fake(),
            ImmutableValueExpression::Scalar(ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world!",
                )),
            )),
            MutableValueExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "var2"),
                ValueAccessor::new(),
            )),
        )));

        assert_eq!(2, result.len());
        assert_eq!(
            OwnedValue::String(ValueStorage::new("hello world!".into())).to_value(),
            result.get("var2").unwrap().to_value()
        );

        // Test updating a variable
        let result = run_test(TransformExpression::Set(SetTransformExpression::new(
            QueryLocation::new_fake(),
            ImmutableValueExpression::Scalar(ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "goodebye world",
                )),
            )),
            MutableValueExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "var1"),
                ValueAccessor::new(),
            )),
        )));

        assert_eq!(1, result.len());
        assert_eq!(
            OwnedValue::String(ValueStorage::new("goodebye world".into())).to_value(),
            result.get("var1").unwrap().to_value()
        );

        // Test updating a variable with path
        let result = run_test(TransformExpression::Set(SetTransformExpression::new(
            QueryLocation::new_fake(),
            ImmutableValueExpression::Scalar(ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "goodebye world",
                )),
            )),
            MutableValueExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "var1"),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "subkey1",
                    )),
                )]),
            )),
        )));

        assert_eq!(1, result.len());
        assert_eq!(
            "{\"subkey1\":\"goodebye world\"}",
            result.get("var1").unwrap().to_value().to_string()
        );
    }

    #[test]
    fn test_execute_remove_source_transform_expression() {
        let record = TestRecord::new()
            .with_key_value(
                "key1".into(),
                OwnedValue::String(ValueStorage::new("value1".into())),
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
        let result = run_test(TransformExpression::Remove(RemoveTransformExpression::new(
            QueryLocation::new_fake(),
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
        )));

        assert!(!result.contains_key("key1"));

        // Test removing an index
        let result = run_test(TransformExpression::Remove(RemoveTransformExpression::new(
            QueryLocation::new_fake(),
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                    )),
                ]),
            )),
        )));

        if let Some(Value::Array(a)) = result.get("key2").map(|v| v.to_value()) {
            assert_eq!(2, a.len());
        } else {
            panic!("Unexpected results")
        }

        // Test removing an index using negative lookup
        let result = run_test(TransformExpression::Remove(RemoveTransformExpression::new(
            QueryLocation::new_fake(),
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
                    )),
                ]),
            )),
        )));

        if let Some(Value::Array(a)) = result.get("key2").map(|v| v.to_value()) {
            assert_eq!(2, a.len());
        } else {
            panic!("Unexpected results")
        }
    }

    #[test]
    fn test_execute_remove_variable_transform_expression() {
        let record = TestRecord::new();

        let run_test = |transform_expression| {
            let pipeline = PipelineExpressionBuilder::new("set")
                .with_expressions(vec![DataExpression::Transform(transform_expression)])
                .build()
                .unwrap();

            let execution_context =
                ExecutionContext::new(LogLevel::Verbose, &pipeline, None, record.clone());

            execution_context.get_variables().borrow_mut().set(
                "var1",
                ResolvedValue::Computed(OwnedValue::Map(MapValueStorage::new(HashMap::from([(
                    "subkey1".into(),
                    OwnedValue::String(ValueStorage::new("hello world".into())),
                )])))),
            );

            if let DataExpression::Transform(t) = &pipeline.get_expressions()[0] {
                execute_transform_expression(&execution_context, t).unwrap();
            } else {
                panic!("Unexpected expression");
            }

            execution_context
                .get_variables()
                .replace(MapValueStorage::new(HashMap::new()))
        };

        // Test removing a variable
        let result = run_test(TransformExpression::Remove(RemoveTransformExpression::new(
            QueryLocation::new_fake(),
            MutableValueExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "var1"),
                ValueAccessor::new(),
            )),
        )));

        assert_eq!(0, result.len());
    }

    #[test]
    fn test_execute_remove_map_keys_transform_expression() {
        let record = TestRecord::new()
            .with_key_value(
                "key1".into(),
                OwnedValue::String(ValueStorage::new("value1".into())),
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
        let result = run_test(TransformExpression::RemoveMapKeys(
            RemoveMapKeysTransformExpression::Remove(MapKeyListExpression::new(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                ))],
            )),
        ));

        assert_eq!(1, result.len());
        assert!(!result.contains_key("key1"));

        // Test retaining a key
        let result = run_test(TransformExpression::RemoveMapKeys(
            RemoveMapKeysTransformExpression::Retain(MapKeyListExpression::new(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                ))],
            )),
        ));

        assert_eq!(1, result.len());
        assert!(result.contains_key("key1"));
    }
}
