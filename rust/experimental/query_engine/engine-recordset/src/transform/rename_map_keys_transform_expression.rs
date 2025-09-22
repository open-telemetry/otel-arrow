// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeMap, HashMap},
    ops::DerefMut,
    slice::Iter,
};

use data_engine_expressions::*;

use crate::{
    execution_context::*, resolved_value_mut::*, scalars::*,
    transform::reduce_map_transform_expression::MapReductionKey,
    value_expressions::execute_mutable_value_expression, *,
};

pub fn execute_rename_map_keys_transform_expression<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    rename_map_keys_transform_expression: &'a RenameMapKeysTransformExpression,
) -> Result<(), ExpressionError> {
    let target = rename_map_keys_transform_expression.get_target();

    let map_keys = rename_map_keys_transform_expression.get_keys();

    let resolved_map_keys = resolve_map_keys(execution_context, target, map_keys)?;

    execution_context.add_diagnostic_if_enabled(
        RecordSetEngineDiagnosticLevel::Verbose,
        rename_map_keys_transform_expression,
        || format!("Resolved source map keys: {:?}", resolved_map_keys.0),
    );

    execution_context.add_diagnostic_if_enabled(
        RecordSetEngineDiagnosticLevel::Verbose,
        rename_map_keys_transform_expression,
        || format!("Resolved destination map keys: {:?}", resolved_map_keys.1),
    );

    let target = execute_mutable_value_expression(execution_context, target)?;

    if let Some(ResolvedValueMut::Map(mut m)) = target {
        let mut values = HashMap::with_capacity(map_keys.len());

        remove(
            execution_context,
            rename_map_keys_transform_expression,
            &mut values,
            StaticValueMut::Map(m.deref_mut()),
            &resolved_map_keys.0,
        );

        set(
            execution_context,
            rename_map_keys_transform_expression,
            &mut values,
            StaticValueMut::Map(m.deref_mut()),
            &resolved_map_keys.1,
        );
    } else {
        execution_context.add_diagnostic_if_enabled(
            RecordSetEngineDiagnosticLevel::Warn,
            rename_map_keys_transform_expression,
            || "Map reduction target was not a map".into(),
        );
    }

    Ok(())
}

fn resolve_map_keys<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    target: &'a MutableValueExpression,
    map_keys: &'a [MapKeyRenameSelector],
) -> Result<(MapRename<'c>, MapRename<'c>), ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let mut source = MapRename::new();
    let mut destination = MapRename::new();

    for (i, selector) in map_keys.iter().enumerate() {
        process_map_key_accessor(
            execution_context,
            target,
            selector.get_source().get_selectors().iter(),
            &mut source,
            i,
        )?;

        process_map_key_accessor(
            execution_context,
            target,
            selector.get_destination().get_selectors().iter(),
            &mut destination,
            i,
        )?;
    }

    Ok((source, destination))
}

fn process_map_key_accessor<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    target: &'a MutableValueExpression,
    mut selectors: Iter<'a, ScalarExpression>,
    current_reduction: &mut MapRename<'c>,
    index: usize,
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
                return process_map_key_accessor(execution_context, target, selectors, t, index);
            } else {
                let t = current_reduction
                    .keys
                    .entry(key)
                    .or_insert(MapRename::new());
                return process_map_key_accessor(execution_context, target, selectors, t, index);
            }
        } else if let Value::Integer(i) = value.to_value() {
            let selector_index = i.get_value();
            if let Some(t) = current_reduction.indices.get_mut(&selector_index) {
                return process_map_key_accessor(execution_context, target, selectors, t, index);
            } else {
                let t = current_reduction
                    .indices
                    .entry(selector_index)
                    .or_insert(MapRename::new());
                return process_map_key_accessor(execution_context, target, selectors, t, index);
            }
        } else {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Warn,
                selector,
                || format!("Value with '{value_type:?}' type specified in map reduction accessor expression is not supported"),
            );
        }
    }

    current_reduction.index = Some(index);

    Ok(())
}

fn remove<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    expression: &'a dyn Expression,
    values: &mut HashMap<usize, OwnedValue>,
    root: StaticValueMut,
    rename: &MapRename,
) {
    if let StaticValueMut::Map(m) = root {
        for (key, rename) in &rename.keys {
            let key = key.get_value();

            if !rename.indices.is_empty() || !rename.keys.is_empty() {
                if let ValueMutGetResult::Found(v) = m.get_mut(key)
                    && let Some(v) = v.to_static_value_mut()
                {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        expression,
                        || format!("Processing sub-items for key '{key}' on target map because inner rules were specified"),
                    );
                    remove(execution_context, expression, values, v, rename);
                } else {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        expression,
                        || format!("Mutable value could not be found for key '{key}' on target map inner rules will be ignored"),
                    );
                }
            }

            if let Some(i) = rename.index {
                if let ValueMutRemoveResult::Removed(v) = m.remove(key) {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        expression,
                        || {
                            format!(
                                "Removed map key '{key}' on target with value: {}",
                                v.to_value()
                            )
                        },
                    );
                    values.insert(i, v);
                } else {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        expression,
                        || format!("Map key '{key}' could not be found on target map"),
                    );
                }
            }
        }
    } else if let StaticValueMut::Array(a) = root {
        let mut elements = BTreeMap::new();
        let length = a.len() as i64;

        for (i, r) in &rename.indices {
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

        for (index, rename) in elements.iter().rev() {
            if !rename.indices.is_empty() || !rename.keys.is_empty() {
                if let ValueMutGetResult::Found(v) = a.get_mut(*index)
                    && let Some(v) = v.to_static_value_mut()
                {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        expression,
                        || format!("Processing sub-items for index '{index}' on target array because inner rules were specified"),
                    );
                    remove(execution_context, expression, values, v, rename);
                } else {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        expression,
                        || format!("Mutable value could not be found for index '{index}' on target array inner rules will be ignored"),
                    );
                }
            }

            if let Some(i) = rename.index {
                if let ValueMutRemoveResult::Removed(v) = a.remove(*index) {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        expression,
                        || {
                            format!(
                                "Removed array index '{index}' on target with value: {}",
                                v.to_value()
                            )
                        },
                    );
                    values.insert(i, v);
                } else {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        expression,
                        || format!("Array index '{index}' could not be found on target map"),
                    );
                }
            }
        }
    } else {
        execution_context.add_diagnostic_if_enabled(
            RecordSetEngineDiagnosticLevel::Warn,
            expression,
            || {
                format!(
                    "Cannot remove data from a '{:?}' value",
                    root.get_value_type()
                )
            },
        );
    }
}

fn set<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    expression: &'a dyn Expression,
    values: &mut HashMap<usize, OwnedValue>,
    root: StaticValueMut,
    rename: &MapRename,
) {
    if let StaticValueMut::Map(m) = root {
        for (key, rename) in &rename.keys {
            let key = key.get_value();

            if let Some(i) = rename.index {
                if let Some(v) = values.remove(&i) {
                    match m.set(key, ResolvedValue::Computed(v)) {
                        ValueMutWriteResult::NotFound => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Warn,
                                expression,
                                || format!("Map key '{key}' could not be found on target map"),
                            );
                        }
                        ValueMutWriteResult::Created => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                expression,
                                || format!("Map key '{key}' created on target map"),
                            );
                        }
                        ValueMutWriteResult::Updated(old) => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                expression,
                                || format!("Map key '{key}' updated on target map replacing old value: {}", old.to_value()),
                            );
                        }
                        ValueMutWriteResult::NotSupported(e) => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                expression,
                                || {
                                    format!(
                                        "Map key '{key}' could not be updated on target map: {e}"
                                    )
                                },
                            );
                        }
                    }
                }
            } else if !rename.indices.is_empty() || !rename.keys.is_empty() {
                if let ValueMutGetResult::Found(v) = m.get_mut(key)
                    && let Some(v) = v.to_static_value_mut()
                {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        expression,
                        || format!("Processing sub-items for key '{key}' on target map because inner rules were specified"),
                    );
                    set(execution_context, expression, values, v, rename);
                } else {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        expression,
                        || format!("Mutable value could not be found for key '{key}' on target map inner rules will be ignored"),
                    );
                }
            }
        }
    } else if let StaticValueMut::Array(a) = root {
        let mut elements = BTreeMap::new();
        let length = a.len() as i64;

        for (i, r) in &rename.indices {
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

        for (index, rename) in elements.iter().rev() {
            if let Some(i) = rename.index {
                if let Some(v) = values.remove(&i) {
                    match a.set(*index, ResolvedValue::Computed(v)) {
                        ValueMutWriteResult::NotFound => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Warn,
                                expression,
                                || {
                                    format!(
                                        "Array index '{index}' could not be found on target array"
                                    )
                                },
                            );
                        }
                        ValueMutWriteResult::Created => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                expression,
                                || format!("Array index '{index}' created on target array"),
                            );
                        }
                        ValueMutWriteResult::Updated(old) => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                expression,
                                || format!("Array index '{index}' updated on target array replacing old value: {}", old.to_value()),
                            );
                        }
                        ValueMutWriteResult::NotSupported(e) => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                expression,
                                || format!("Array index '{index}' could not be updated on target array: {e}"),
                            );
                        }
                    }
                }
            } else if !rename.indices.is_empty() || !rename.keys.is_empty() {
                if let ValueMutGetResult::Found(v) = a.get_mut(*index)
                    && let Some(v) = v.to_static_value_mut()
                {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        expression,
                        || format!("Processing sub-items for index '{index}' on target array because inner rules were specified"),
                    );
                    set(execution_context, expression, values, v, rename);
                } else {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        expression,
                        || format!("Mutable value could not be found for index '{index}' on target array inner rules will be ignored"),
                    );
                }
            }
        }
    } else {
        execution_context.add_diagnostic_if_enabled(
            RecordSetEngineDiagnosticLevel::Warn,
            expression,
            || format!("Cannot set data on a '{:?}' value", root.get_value_type()),
        );
    }
}

#[derive(Debug)]
struct MapRename<'a> {
    keys: HashMap<MapReductionKey<'a>, MapRename<'a>>,
    indices: HashMap<i64, MapRename<'a>>,
    index: Option<usize>,
}

impl<'a> MapRename<'a> {
    pub fn new() -> MapRename<'a> {
        Self {
            keys: HashMap::new(),
            indices: HashMap::new(),
            index: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::transform::transform_expressions::execute_transform_expression;

    use super::*;

    #[test]
    fn test_execute_rename_map_keys_transform_expression() {
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

        // Note: Simple rename test.
        let record = run_test(TransformExpression::RenameMapKeys(
            RenameMapKeysTransformExpression::new(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![MapKeyRenameSelector::new(
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        )),
                    )]),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "new_key1",
                        )),
                    )]),
                )],
            ),
        ));

        assert_eq!(2, record.len());
        assert_eq!(
            "{\"subkey1\":\"hello\",\"subkey2\":\"world\",\"subkey3\":\"goodbye\"}",
            record.get("new_key1").unwrap().to_value().to_string()
        );
        assert!(record.contains_key("key2"));

        // Note: In this test subkey1 is pulled out of the map on key1 and set
        // as new_key2. The remaining items from key1 become new_key1.
        let record = run_test(TransformExpression::RenameMapKeys(
            RenameMapKeysTransformExpression::new(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![
                    MapKeyRenameSelector::new(
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "key1",
                            )),
                        )]),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "new_key1",
                            )),
                        )]),
                    ),
                    MapKeyRenameSelector::new(
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "subkey1"),
                            )),
                        ]),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "new_key2",
                            )),
                        )]),
                    ),
                ],
            ),
        ));

        assert_eq!(3, record.len());
        assert_eq!(
            "hello",
            record.get("new_key2").unwrap().to_value().to_string()
        );
        assert_eq!(
            "{\"subkey2\":\"world\",\"subkey3\":\"goodbye\"}",
            record.get("new_key1").unwrap().to_value().to_string()
        );
        assert!(record.contains_key("key2"));

        // Note: In this test index 1 & 2 are pulled out of the array on key2 and set
        // as new keys and then the remaining items from key2 become new_key2.
        let record = run_test(TransformExpression::RenameMapKeys(
            RenameMapKeysTransformExpression::new(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![
                    MapKeyRenameSelector::new(
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                            )),
                        ]),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "array_index_2",
                            )),
                        )]),
                    ),
                    MapKeyRenameSelector::new(
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                            )),
                        ]),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "array_index_1",
                            )),
                        )]),
                    ),
                    MapKeyRenameSelector::new(
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "key2",
                            )),
                        )]),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "new_key2",
                            )),
                        )]),
                    ),
                ],
            ),
        ));

        assert_eq!(4, record.len());
        assert_eq!(
            "2",
            record.get("array_index_1").unwrap().to_value().to_string()
        );
        assert_eq!(
            "3",
            record.get("array_index_2").unwrap().to_value().to_string()
        );
        assert_eq!(
            "[1]",
            record.get("new_key2").unwrap().to_value().to_string()
        );
        assert!(record.contains_key("key1"));

        // Note: In this test the map at key1 is moved into the array at key2 in
        // the first index.
        let record = run_test(TransformExpression::RenameMapKeys(
            RenameMapKeysTransformExpression::new(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                vec![MapKeyRenameSelector::new(
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        )),
                    )]),
                    ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )),
                    ]),
                )],
            ),
        ));

        assert_eq!(1, record.len());
        assert_eq!(
            "[{\"subkey1\":\"hello\",\"subkey2\":\"world\",\"subkey3\":\"goodbye\"},2,3]",
            record.get("key2").unwrap().to_value().to_string()
        );
    }
}
