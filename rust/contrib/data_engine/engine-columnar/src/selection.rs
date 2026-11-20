// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use ahash::RandomState;
use arrow::{array::PrimitiveArray, datatypes::*};
use indexmap::IndexSet;
use otel_arrow_contrib_data_engine_expressions::*;

use crate::{
    execution_context::ExecutionContext, resolved_value::*, scalars::execute_scalar_expression, *,
};

pub fn select_from_record_table<'a, 'pipeline, TRecords: ColumnarRecords<'pipeline>>(
    execution_context: &ExecutionContext<'a, 'pipeline, TRecords>,
    root: &'a dyn RecordTable<'pipeline>,
    selectors: &'pipeline [ScalarExpression],
) -> ResolvedScalarValue<'pipeline, 'a> {
    let current = ResolvedScalarValue::Table(root);

    select(execution_context, current, selectors)
}

pub fn select<'a, 'pipeline, TRecords: ColumnarRecords<'pipeline>>(
    execution_context: &ExecutionContext<'a, 'pipeline, TRecords>,
    mut current: ResolvedScalarValue<'pipeline, 'a>,
    selectors: &'pipeline [ScalarExpression],
) -> ResolvedScalarValue<'pipeline, 'a> {
    for selector in selectors {
        let next = execute_scalar_expression(execution_context, selector).map_into_with_state(
            current,
            |current, s| match s {
                ValueOrRef::String(key) => match current {
                    ResolvedScalarValue::Table(t) => match t.get_values(key.get_value()) {
                        Some(RecordTableValue::Table(t)) => Some(ResolvedScalarValue::Table(t)),
                        Some(RecordTableValue::Dictionary(d)) => Some(ResolvedScalarValue::Dictionary(d)),
                        None => None,
                    }
                    ResolvedScalarValue::Dictionary(d) => {
                        Some(ResolvedScalarValue::Dictionary(d.transform_into_any(&execution_context.get_diagnostic_receiver().bind(selector), |v| {
                            if let ValueOrRef::Map(m) = v {
                                match m {
                                    MapValueOrRef::Ref(m) => m.get(key.get_value()).map_or(ValueOrRef::Null, |v| v.to_value().into()),
                                    MapValueOrRef::Owned(m) => m.get_values().get(key.get_value()).map_or(ValueOrRef::Null, |v| v.clone())
                                }
                            } else {
                                execution_context.add_diagnostic_if_enabled(
                                    ColumnarEngineDiagnosticLevel::Warn,
                                    selector,
                                    || format!("Could not search for map key '{}' specified in accessor expression because current node is a '{}' value", key.get_value(), v.get_value_type()),
                                );
                                ValueOrRef::Null
                            }
                        })))
                    }
                    ResolvedScalarValue::Single(s) => match s {
                        ValueOrRef::Map(m) => {
                            Some(ResolvedScalarValue::Single(match m {
                                MapValueOrRef::Ref(m) => m.get(key.get_value()).map_or(ValueOrRef::Null, |v| v.to_value().into()),
                                MapValueOrRef::Owned(m) => m.get_values().get(key.get_value()).map_or(ValueOrRef::Null, |v| v.clone())
                            }))
                        }
                        v => {
                            execution_context.add_diagnostic_if_enabled(
                                ColumnarEngineDiagnosticLevel::Warn,
                                selector,
                                || format!("Could not search for map key '{}' specified in accessor expression because current node is a '{}' value", key.get_value(), v.get_value_type()),
                            );
                            None
                        }
                    }
                },
                ValueOrRef::Integer(index) => match current {
                    ResolvedScalarValue::Dictionary(d) => {
                        Some(ResolvedScalarValue::Dictionary(d.transform_into_any(&execution_context.get_diagnostic_receiver().bind(selector), |v| {
                            if let ValueOrRef::Array(a) = v {
                                let len = a.len();
                                let mut index = index.get_value();
                                if index < 0 {
                                    index += len as i64;
                                }
                                if index < 0 || index >= len as i64 {
                                    execution_context.add_diagnostic_if_enabled(
                                        ColumnarEngineDiagnosticLevel::Warn,
                                        selector,
                                        || format!("Array index '{index}' specified in accessor expression is invalid"),
                                    );
                                    ValueOrRef::Null
                                } else {
                                    a.get(index as usize)
                                }
                            } else {
                                execution_context.add_diagnostic_if_enabled(
                                    ColumnarEngineDiagnosticLevel::Warn,
                                    selector,
                                    || format!("Could not search for array index '{}' specified in accessor expression because current node is a '{}' value", index.get_value(), v.get_value_type()),
                                );
                                ValueOrRef::Null
                            }
                        })))
                    }
                    ResolvedScalarValue::Single(s) => match s {
                        ValueOrRef::Array(a) => {
                            let len = a.len();
                            let mut index = index.get_value();
                            if index < 0 {
                                index += len as i64;
                            }
                            if index < 0 || index >= len as i64 {
                                execution_context.add_diagnostic_if_enabled(
                                    ColumnarEngineDiagnosticLevel::Warn,
                                    selector,
                                    || format!("Array index '{index}' specified in accessor expression is invalid"),
                                );
                                None
                            } else {
                                Some(ResolvedScalarValue::Single(a.get(index as usize)))
                            }
                        }
                        v => {
                            execution_context.add_diagnostic_if_enabled(
                                ColumnarEngineDiagnosticLevel::Warn,
                                selector,
                                || format!("Could not search for array index '{}' specified in accessor expression because current node is a '{}' value", index.get_value(), v.get_value_type()),
                            );
                            None
                        }
                    }
                    ResolvedScalarValue::Table(_) => {
                        execution_context.add_diagnostic_if_enabled(
                            ColumnarEngineDiagnosticLevel::Warn,
                            selector,
                            || format!("Could not search for array index '{}' specified in accessor expression because current node is a 'Map' value", index.get_value()),
                        );
                        None
                    }
                }
                v => {
                    execution_context.add_diagnostic_if_enabled(
                        ColumnarEngineDiagnosticLevel::Warn,
                        selector,
                        || format!("Unexpected scalar expression with '{}' value type encountered in accessor expression", v.get_value_type()),
                    );
                    None
                }
            },
            |current, dictionary| {
                // Every row can select a distinct value even when the selector repeats one value.
                let result_key_type = widen_result_key_type(
                    dictionary.keys().data_type(),
                    dictionary.len().saturating_sub(1),
                );
                Some(match result_key_type {
                    DictionaryKeyType::UInt8 => select_using_dictionary::<UInt8Type, TRecords>(execution_context, current, selector, dictionary),
                    DictionaryKeyType::UInt16 => select_using_dictionary::<UInt16Type, TRecords>(execution_context, current, selector, dictionary),
                    DictionaryKeyType::UInt32 => select_using_dictionary::<UInt32Type, TRecords>(execution_context, current, selector, dictionary),
                    DictionaryKeyType::UInt64 => select_using_dictionary::<UInt64Type, TRecords>(execution_context, current, selector, dictionary),

                    DictionaryKeyType::Int8 => select_using_dictionary::<Int8Type, TRecords>(execution_context, current, selector, dictionary),
                    DictionaryKeyType::Int16 => select_using_dictionary::<Int16Type, TRecords>(execution_context, current, selector, dictionary),
                    DictionaryKeyType::Int32 => select_using_dictionary::<Int32Type, TRecords>(execution_context, current, selector, dictionary),
                    DictionaryKeyType::Int64 => select_using_dictionary::<Int64Type, TRecords>(execution_context, current, selector, dictionary),
                })
            },
            |_, _| {
                execution_context.add_diagnostic_if_enabled(
                    ColumnarEngineDiagnosticLevel::Warn,
                    selector,
                    || "Unexpected scalar expression with Map value type encountered in accessor expression".into(),
                );
                None
            }
        );

        match next {
            None => {
                current = ResolvedScalarValue::new_null();
                break;
            }
            Some(v) => current = v,
        }
    }

    current
}

fn widen_result_key_type(
    mut key_type: DictionaryKeyType,
    max_value_index: usize,
) -> DictionaryKeyType {
    loop {
        key_type = match key_type {
            DictionaryKeyType::Int8 if i8::try_from(max_value_index).is_err() => {
                DictionaryKeyType::Int16
            }
            DictionaryKeyType::Int16 if i16::try_from(max_value_index).is_err() => {
                DictionaryKeyType::Int32
            }
            DictionaryKeyType::Int32 if i32::try_from(max_value_index).is_err() => {
                DictionaryKeyType::Int64
            }
            DictionaryKeyType::Int64 if i64::try_from(max_value_index).is_err() => {
                DictionaryKeyType::UInt64
            }
            DictionaryKeyType::UInt8 if u8::try_from(max_value_index).is_err() => {
                DictionaryKeyType::UInt16
            }
            DictionaryKeyType::UInt16 if u16::try_from(max_value_index).is_err() => {
                DictionaryKeyType::UInt32
            }
            DictionaryKeyType::UInt32 if u32::try_from(max_value_index).is_err() => {
                DictionaryKeyType::UInt64
            }
            _ => return key_type,
        };
    }
}

fn select_using_dictionary<
    'a,
    'pipeline,
    K: ArrowDictionaryKeyType,
    TRecords: ColumnarRecords<'pipeline>,
>(
    execution_context: &ExecutionContext<'a, 'pipeline, TRecords>,
    mut current: ResolvedScalarValue<'pipeline, 'a>,
    selector_expression: &'pipeline dyn Expression,
    selector: Dictionary<'pipeline>,
) -> ResolvedScalarValue<'pipeline, 'a>
where
    DictionaryKeyArray: From<PrimitiveArray<K>>,
{
    let key_count = selector.len();

    let mut key_builder = DictionaryKeyArrayBuilder::<K>::new(key_count);
    let mut key_writer = key_builder.get_writer();

    let mut values = IndexSet::with_hasher(RandomState::new());

    let mut conversion_failures = 0;
    let mut key_overflows = 0;

    if let ResolvedScalarValue::Single(value) = current {
        current = ResolvedScalarValue::Dictionary(
            value.into_dictionary(selector.keys().data_type(), selector.len()),
        );
    }

    for key_index in 0..key_count {
        match selector
            .get_value(key_index)
            .unwrap_or_else(|_| {
                conversion_failures += 1;
                ValueOrRef::Null
            })
            .to_value()
        {
            Value::String(key) => {
                let value = match current {
                    ResolvedScalarValue::Table(t) => match t.get_values(key.get_value()) {
                        Some(RecordTableValue::Dictionary(d)) => {
                            d.get_value(key_index).unwrap_or_else(|_| {
                                conversion_failures += 1;
                                ValueOrRef::Null
                            })
                        }
                        Some(RecordTableValue::Table(_)) => {
                            todo!("Table returning a table for a key is not currently supported")
                        }
                        None => ValueOrRef::Null,
                    },
                    ResolvedScalarValue::Dictionary(ref d) => {
                        match d.get_value(key_index).unwrap_or_else(|_| {
                            conversion_failures += 1;
                            ValueOrRef::Null
                        }) {
                            ValueOrRef::Map(m) => match m {
                                MapValueOrRef::Ref(m) => m
                                    .get(key.get_value())
                                    .map_or(ValueOrRef::Null, |v| v.to_value().into()),
                                MapValueOrRef::Owned(m) => m
                                    .get_values()
                                    .get(key.get_value())
                                    .map_or(ValueOrRef::Null, |v| v.clone()),
                            },
                            v => {
                                execution_context.add_diagnostic_if_enabled(
                                    ColumnarEngineDiagnosticLevel::Warn,
                                    selector_expression,
                                    || format!("Could not search for map key '{}' specified in accessor expression because current node is a '{}' value", key.get_value(), v.get_value_type()),
                                );
                                ValueOrRef::Null
                            }
                        }
                    }
                    ResolvedScalarValue::Single(_) => {
                        unreachable!("Single should be converted to dictionary by this point")
                    }
                };
                if let ValueOrRef::Null = value {
                    unsafe { key_writer.set_null_unchecked(key_index) }
                } else {
                    let (index, _) = values.insert_full(value);
                    if !unsafe {
                        key_writer
                            .set_value_index_unchecked(key_index, index)
                            .is_ok()
                    } {
                        key_overflows += 1;
                        unsafe { key_writer.set_null_unchecked(key_index) }
                    }
                }
            }
            Value::Integer(index) => {
                let value = match current {
                    ResolvedScalarValue::Table(_) => {
                        execution_context.add_diagnostic_if_enabled(
                            ColumnarEngineDiagnosticLevel::Warn,
                            selector_expression,
                            || format!("Could not search for array index '{}' specified in accessor expression because current node is a 'Map' value", index.get_value()),
                        );
                        ValueOrRef::Null
                    }
                    ResolvedScalarValue::Dictionary(ref d) => {
                        match d.get_value(key_index).unwrap_or_else(|_| {
                            conversion_failures += 1;
                            ValueOrRef::Null
                        }) {
                            ValueOrRef::Array(a) => {
                                let len = a.len();
                                let mut index = index.get_value();
                                if index < 0 {
                                    index += len as i64;
                                }
                                if index < 0 || index >= len as i64 {
                                    execution_context.add_diagnostic_if_enabled(
                                        ColumnarEngineDiagnosticLevel::Warn,
                                        selector_expression,
                                        || format!("Array index '{index}' specified in accessor expression is invalid"),
                                    );
                                    ValueOrRef::Null
                                } else {
                                    a.get(index as usize)
                                }
                            }
                            v => {
                                execution_context.add_diagnostic_if_enabled(
                                    ColumnarEngineDiagnosticLevel::Warn,
                                    selector_expression,
                                    || format!("Could not search for array index '{}' specified in accessor expression because current node is a '{}' value", index.get_value(), v.get_value_type()),
                                );
                                ValueOrRef::Null
                            }
                        }
                    }
                    ResolvedScalarValue::Single(_) => {
                        unreachable!("Single should be converted to dictionary by this point")
                    }
                };
                if let ValueOrRef::Null = value {
                    unsafe { key_writer.set_null_unchecked(key_index) };
                } else {
                    let (index, _) = values.insert_full(value);
                    if !unsafe {
                        key_writer
                            .set_value_index_unchecked(key_index, index)
                            .is_ok()
                    } {
                        key_overflows += 1;
                        unsafe { key_writer.set_null_unchecked(key_index) }
                    }
                }
            }
            Value::Null => {
                unsafe { key_writer.set_null_unchecked(key_index) };
            }
            v => {
                execution_context.add_diagnostic_if_enabled(
                    ColumnarEngineDiagnosticLevel::Warn,
                    selector_expression,
                    || format!("Unexpected scalar expression with '{}' value type encountered in accessor expression", v.get_value_type()),
                );
                unsafe { key_writer.set_null_unchecked(key_index) };
            }
        }
    }

    if conversion_failures > 0 {
        execution_context.add_diagnostic_if_enabled(
            ColumnarEngineDiagnosticLevel::Warn,
            selector_expression,
            || format!("Encountered {conversion_failures} value(s) which could not be converted into accessor types"));
    }
    if key_overflows > 0 {
        execution_context.add_diagnostic_if_enabled(
            ColumnarEngineDiagnosticLevel::Warn,
            selector_expression,
            || format!("Encountered {key_overflows} value(s) which could not be stored due to key overflow"));
    }

    unsafe {
        ResolvedScalarValue::Dictionary(Dictionary::new_unvalidated(
            key_builder.finish().into(),
            values.into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Result indexes require preserving or widening signed and unsigned dictionary keys.
    /// Guarantees: Sufficient types never narrow, and widening covers large indexes without truncation.
    #[test]
    fn test_widen_result_key_type_preserves_and_promotes_widths() {
        for key_type in [
            DictionaryKeyType::Int8,
            DictionaryKeyType::Int16,
            DictionaryKeyType::Int32,
            DictionaryKeyType::Int64,
            DictionaryKeyType::UInt8,
            DictionaryKeyType::UInt16,
            DictionaryKeyType::UInt32,
            DictionaryKeyType::UInt64,
        ] {
            assert_eq!(key_type, widen_result_key_type(key_type.clone(), 0));
        }

        for (key_type, max_value_index, expected) in [
            (DictionaryKeyType::Int8, 128, DictionaryKeyType::Int16),
            (DictionaryKeyType::Int8, 32_768, DictionaryKeyType::Int32),
            (DictionaryKeyType::Int16, 32_767, DictionaryKeyType::Int16),
            (DictionaryKeyType::Int16, 32_768, DictionaryKeyType::Int32),
            (DictionaryKeyType::UInt8, 256, DictionaryKeyType::UInt16),
            (DictionaryKeyType::UInt8, 65_536, DictionaryKeyType::UInt32),
            (DictionaryKeyType::UInt16, 65_535, DictionaryKeyType::UInt16),
            (DictionaryKeyType::UInt16, 65_536, DictionaryKeyType::UInt32),
        ] {
            assert_eq!(expected, widen_result_key_type(key_type, max_value_index));
        }

        for (key_type, max_value_index, expected) in [
            (
                DictionaryKeyType::Int32,
                (1u64 << 31) - 1,
                DictionaryKeyType::Int32,
            ),
            (
                DictionaryKeyType::Int32,
                1u64 << 31,
                DictionaryKeyType::Int64,
            ),
            (
                DictionaryKeyType::UInt32,
                (1u64 << 32) - 1,
                DictionaryKeyType::UInt32,
            ),
            (
                DictionaryKeyType::UInt32,
                1u64 << 32,
                DictionaryKeyType::UInt64,
            ),
            (
                DictionaryKeyType::UInt8,
                1u64 << 32,
                DictionaryKeyType::UInt64,
            ),
            (
                DictionaryKeyType::Int64,
                (1u64 << 63) - 1,
                DictionaryKeyType::Int64,
            ),
            (
                DictionaryKeyType::Int64,
                1u64 << 63,
                DictionaryKeyType::UInt64,
            ),
            (
                DictionaryKeyType::Int8,
                1u64 << 63,
                DictionaryKeyType::UInt64,
            ),
            (
                DictionaryKeyType::UInt64,
                u64::MAX,
                DictionaryKeyType::UInt64,
            ),
        ] {
            if let Ok(max_value_index) = usize::try_from(max_value_index) {
                assert_eq!(expected, widen_result_key_type(key_type, max_value_index));
            }
        }
    }
}
