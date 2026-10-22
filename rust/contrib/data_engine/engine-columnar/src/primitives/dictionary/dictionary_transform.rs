// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use arrow::{array::*, datatypes::*};

use crate::*;

impl<'a> Dictionary<'a> {
    pub(crate) fn transform_into_any<FTransform>(
        self,
        mut transform: FTransform,
    ) -> Result<Dictionary<'a>, DictionaryValueError>
    where
        FTransform: FnMut(&ValueOrRef<'a>) -> ValueOrRef<'a>,
    {
        let (keys, values) = self.into_parts();

        match keys {
            DictionaryKeyArray::KeyArray(key_array) => {
                transform_primitive_array_into_any(key_array, values, transform)
            }
            DictionaryKeyArray::BooleanArray {
                data_type,
                values: key_array,
            } => match data_type {
                DictionaryKeyType::Int8 => transform_boolean_array_into_any_typed::<Int8Type, _>(
                    data_type, key_array, values, transform,
                ),
                DictionaryKeyType::Int16 => transform_boolean_array_into_any_typed::<Int16Type, _>(
                    data_type, key_array, values, transform,
                ),
                DictionaryKeyType::Int32 => transform_boolean_array_into_any_typed::<Int32Type, _>(
                    data_type, key_array, values, transform,
                ),
                DictionaryKeyType::Int64 => transform_boolean_array_into_any_typed::<Int64Type, _>(
                    data_type, key_array, values, transform,
                ),

                DictionaryKeyType::UInt8 => transform_boolean_array_into_any_typed::<UInt8Type, _>(
                    data_type, key_array, values, transform,
                ),
                DictionaryKeyType::UInt16 => {
                    transform_boolean_array_into_any_typed::<UInt16Type, _>(
                        data_type, key_array, values, transform,
                    )
                }
                DictionaryKeyType::UInt32 => {
                    transform_boolean_array_into_any_typed::<UInt32Type, _>(
                        data_type, key_array, values, transform,
                    )
                }
                DictionaryKeyType::UInt64 => {
                    transform_boolean_array_into_any_typed::<UInt64Type, _>(
                        data_type, key_array, values, transform,
                    )
                }
            },
            DictionaryKeyArray::UniqueValues { data_type, length } => match data_type {
                DictionaryKeyType::Int8 => transform_unique_values_into_any_typed::<Int8Type, _>(
                    data_type, length, values, transform,
                ),
                DictionaryKeyType::Int16 => transform_unique_values_into_any_typed::<Int16Type, _>(
                    data_type, length, values, transform,
                ),
                DictionaryKeyType::Int32 => transform_unique_values_into_any_typed::<Int32Type, _>(
                    data_type, length, values, transform,
                ),
                DictionaryKeyType::Int64 => transform_unique_values_into_any_typed::<Int64Type, _>(
                    data_type, length, values, transform,
                ),

                DictionaryKeyType::UInt8 => transform_unique_values_into_any_typed::<UInt8Type, _>(
                    data_type, length, values, transform,
                ),
                DictionaryKeyType::UInt16 => {
                    transform_unique_values_into_any_typed::<UInt16Type, _>(
                        data_type, length, values, transform,
                    )
                }
                DictionaryKeyType::UInt32 => {
                    transform_unique_values_into_any_typed::<UInt32Type, _>(
                        data_type, length, values, transform,
                    )
                }
                DictionaryKeyType::UInt64 => {
                    transform_unique_values_into_any_typed::<UInt64Type, _>(
                        data_type, length, values, transform,
                    )
                }
            },
            DictionaryKeyArray::SingleValue {
                ref data_type,
                length,
                value_index,
            } => {
                let value = match value_index {
                    None => ValueOrRef::Null,
                    Some(value_index) => values.get_value_at(value_index)?,
                };

                Ok(match transform(&value) {
                    ValueOrRef::Null => Dictionary::new_null(data_type.clone(), length),
                    v => Dictionary::new_scalar(data_type.clone(), length, v),
                })
            }
        }
    }
}

fn transform_primitive_array_into_any<'a, FTransform>(
    keys: DictionaryArrowKeyArray,
    values: DictionaryValueArray<'a>,
    transform: FTransform,
) -> Result<Dictionary<'a>, DictionaryValueError>
where
    FTransform: FnMut(&ValueOrRef<'a>) -> ValueOrRef<'a>,
{
    let key_data_type = keys.data_type();

    match keys {
        DictionaryArrowKeyArray::Int8(keys) => {
            transform_primitive_array_into_any_typed(key_data_type, keys, values, transform)
        }
        DictionaryArrowKeyArray::Int16(keys) => {
            transform_primitive_array_into_any_typed(key_data_type, keys, values, transform)
        }
        DictionaryArrowKeyArray::Int32(keys) => {
            transform_primitive_array_into_any_typed(key_data_type, keys, values, transform)
        }
        DictionaryArrowKeyArray::Int64(keys) => {
            transform_primitive_array_into_any_typed(key_data_type, keys, values, transform)
        }

        DictionaryArrowKeyArray::UInt8(keys) => {
            transform_primitive_array_into_any_typed(key_data_type, keys, values, transform)
        }
        DictionaryArrowKeyArray::UInt16(keys) => {
            transform_primitive_array_into_any_typed(key_data_type, keys, values, transform)
        }
        DictionaryArrowKeyArray::UInt32(keys) => {
            transform_primitive_array_into_any_typed(key_data_type, keys, values, transform)
        }
        DictionaryArrowKeyArray::UInt64(keys) => {
            transform_primitive_array_into_any_typed(key_data_type, keys, values, transform)
        }
    }
}

fn transform_primitive_array_into_any_typed<'a, K: ArrowDictionaryKeyType, FTransform>(
    key_data_type: DictionaryKeyType,
    keys: PrimitiveArray<K>,
    values: DictionaryValueArray<'a>,
    mut transform: FTransform,
) -> Result<Dictionary<'a>, DictionaryValueError>
where
    FTransform: FnMut(&ValueOrRef<'a>) -> ValueOrRef<'a>,
    DictionaryKeyArray: From<PrimitiveArray<K>>,
{
    let key_length = keys.len();

    let mut key_builder = DictionaryKeyArrayBuilder::<K>::new(key_length);
    let mut key_writer = key_builder.get_writer();

    let (mut transformed_values, value_index_lookup) = values.transform_into_set(&mut transform)?;

    let mut null_index = None;

    for (key_index, value_index) in keys.into_iter().enumerate() {
        if let Some(value_index) = value_index.map(<K as ArrowPrimitiveType>::Native::as_usize) {
            if let Some(value_index_lookup) = value_index_lookup.as_ref() {
                if let Some(Some(transformed_value_index)) = value_index_lookup.get(&value_index) {
                    unsafe {
                        key_writer.set_value_index_unchecked(key_index, *transformed_value_index)
                    };
                    continue;
                }
            } else {
                unsafe { key_writer.set_value_index_unchecked(key_index, value_index) };
                continue;
            }
        }

        let (has_value_index, value_index) = match null_index {
            Some(v) => v,
            None => {
                let v = match transform(&ValueOrRef::Null) {
                    ValueOrRef::Null => (
                        false,
                        <K as ArrowPrimitiveType>::Native::from_usize(0).unwrap(),
                    ),
                    v => {
                        let (index, _) = transformed_values.insert_full(v);
                        (
                            true,
                            <K as ArrowPrimitiveType>::Native::from_usize(index).unwrap(),
                        )
                    }
                };
                null_index = Some(v);
                v
            }
        };

        if has_value_index {
            unsafe { key_writer.set_value_index_typed_unchecked(key_index, value_index) };
            continue;
        }

        unsafe { key_writer.set_null_unchecked(key_index) };
    }

    if transformed_values.is_empty() {
        return Ok(Dictionary::new_null(key_data_type, key_length));
    } else if transformed_values.len() == 1 && !key_builder.has_nulls() {
        return Ok(Dictionary::new_scalar(
            key_data_type,
            key_length,
            transformed_values.into_iter().next().expect("has value"),
        ));
    }

    Ok(unsafe {
        Dictionary::new_unvalidated(key_builder.finish().into(), transformed_values.into())
    })
}

fn transform_boolean_array_into_any_typed<'a, K: ArrowDictionaryKeyType, FTransform>(
    key_data_type: DictionaryKeyType,
    keys: BooleanArray,
    values: DictionaryValueArray<'a>,
    mut transform: FTransform,
) -> Result<Dictionary<'a>, DictionaryValueError>
where
    FTransform: FnMut(&ValueOrRef<'a>) -> ValueOrRef<'a>,
    DictionaryKeyArray: From<PrimitiveArray<K>>,
{
    let key_length = keys.len();

    let mut key_builder = DictionaryKeyArrayBuilder::<K>::new(key_length);
    let mut key_writer = key_builder.get_writer();

    let (mut transformed_values, value_index_lookup) = values.transform_into_set(&mut transform)?;

    let mut null_index = None;

    for (key_index, value_index) in keys.into_iter().enumerate() {
        if let Some(value_index) = value_index.map(|v| if v { 1 } else { 0 }) {
            if let Some(value_index_lookup) = value_index_lookup.as_ref() {
                if let Some(Some(transformed_value_index)) = value_index_lookup.get(&value_index) {
                    unsafe {
                        key_writer.set_value_index_unchecked(key_index, *transformed_value_index)
                    };
                    continue;
                }
            } else {
                unsafe { key_writer.set_value_index_unchecked(key_index, value_index) };
                continue;
            }
        }

        let (has_value_index, value_index) = match null_index {
            Some(v) => v,
            None => {
                let v = match transform(&ValueOrRef::Null) {
                    ValueOrRef::Null => (
                        false,
                        <K as ArrowPrimitiveType>::Native::from_usize(0).unwrap(),
                    ),
                    v => {
                        let (index, _) = transformed_values.insert_full(v);
                        (
                            true,
                            <K as ArrowPrimitiveType>::Native::from_usize(index).unwrap(),
                        )
                    }
                };
                null_index = Some(v);
                v
            }
        };

        if has_value_index {
            unsafe { key_writer.set_value_index_typed_unchecked(key_index, value_index) };
            continue;
        }

        unsafe { key_writer.set_null_unchecked(key_index) };
    }

    if transformed_values.is_empty() {
        return Ok(Dictionary::new_null(key_data_type, key_length));
    } else if transformed_values.len() == 1 && !key_builder.has_nulls() {
        return Ok(Dictionary::new_scalar(
            key_data_type,
            key_length,
            transformed_values.into_iter().next().expect("has value"),
        ));
    }

    Ok(unsafe {
        Dictionary::new_unvalidated(key_builder.finish().into(), transformed_values.into())
    })
}

fn transform_unique_values_into_any_typed<'a, K: ArrowDictionaryKeyType, FTransform>(
    key_data_type: DictionaryKeyType,
    key_length: usize,
    values: DictionaryValueArray<'a>,
    mut transform: FTransform,
) -> Result<Dictionary<'a>, DictionaryValueError>
where
    FTransform: FnMut(&ValueOrRef<'a>) -> ValueOrRef<'a>,
    DictionaryKeyArray: From<PrimitiveArray<K>>,
{
    let mut key_builder = DictionaryKeyArrayBuilder::<K>::new(key_length);
    let mut key_writer = key_builder.get_writer();

    let (mut transformed_values, value_index_lookup) = values.transform_into_set(&mut transform)?;

    let mut null_index = None;

    for key_index in 0..key_length {
        if let Some(value_index_lookup) = value_index_lookup.as_ref() {
            if let Some(Some(transformed_value_index)) = value_index_lookup.get(&key_index) {
                unsafe {
                    key_writer.set_value_index_unchecked(key_index, *transformed_value_index)
                };
                continue;
            }
        } else {
            unsafe { key_writer.set_value_index_unchecked(key_index, key_index) };
            continue;
        }

        let (has_value_index, value_index) = match null_index {
            Some(v) => v,
            None => {
                let v = match transform(&ValueOrRef::Null) {
                    ValueOrRef::Null => (
                        false,
                        <K as ArrowPrimitiveType>::Native::from_usize(0).unwrap(),
                    ),
                    v => {
                        let (index, _) = transformed_values.insert_full(v);
                        (
                            true,
                            <K as ArrowPrimitiveType>::Native::from_usize(index).unwrap(),
                        )
                    }
                };
                null_index = Some(v);
                v
            }
        };

        if has_value_index {
            unsafe { key_writer.set_value_index_typed_unchecked(key_index, value_index) };
            continue;
        }

        unsafe { key_writer.set_null_unchecked(key_index) };
    }

    if transformed_values.is_empty() {
        return Ok(Dictionary::new_null(key_data_type, key_length));
    } else if transformed_values.len() == 1 && !key_builder.has_nulls() {
        return Ok(Dictionary::new_scalar(
            key_data_type,
            key_length,
            transformed_values.into_iter().next().expect("has value"),
        ));
    }

    Ok(unsafe {
        Dictionary::new_unvalidated(key_builder.finish().into(), transformed_values.into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn primitive_keys(length: usize) -> Vec<DictionaryKeyArray> {
        macro_rules! keys {
            ($array:ty) => {
                DictionaryKeyArray::from(
                    <$array>::from(vec![Some(0), Some(1), Some(0), None, None]).slice(0, length),
                )
            };
        }

        vec![
            keys!(Int8Array),
            keys!(Int16Array),
            keys!(Int32Array),
            keys!(Int64Array),
            keys!(UInt8Array),
            keys!(UInt16Array),
            keys!(UInt32Array),
            keys!(UInt64Array),
        ]
    }

    fn dictionaries() -> Vec<Dictionary<'static>> {
        primitive_keys(5)
            .into_iter()
            .flat_map(|keys| {
                let data_type = keys.data_type();
                [
                    Dictionary::new(keys, DictionaryValueArray::Boolean).expect("valid"),
                    Dictionary::new_boolean(
                        data_type.clone(),
                        BooleanArray::from(vec![Some(false), Some(true), Some(false), None, None]),
                    ),
                    Dictionary::new_unique_values(
                        data_type,
                        vec![
                            ValueOrRef::Boolean(false),
                            ValueOrRef::Boolean(true),
                            ValueOrRef::Boolean(false),
                            ValueOrRef::Null,
                            ValueOrRef::Null,
                        ]
                        .into(),
                    ),
                ]
            })
            .collect()
    }

    fn assert_rows(dictionary: &Dictionary<'_>, expected: &[ValueOrRef<'_>]) {
        assert_eq!(dictionary.len(), expected.len());
        for (index, value) in expected.iter().enumerate() {
            assert_eq!(
                dictionary.get_value(index).as_ref(),
                Ok(value),
                "row {index}"
            );
            assert_eq!(
                dictionary.nulls().is_some_and(|nulls| nulls.is_null(index)),
                matches!(value, ValueOrRef::Null),
                "row {index} validity"
            );
        }
    }

    /// Scenario: Every integer key type uses primitive, Boolean, or unique-value keys with repeated and null rows.
    /// Guarantees: Heterogeneous transformed values retain row order, key type, nulls, and shared value indexes.
    #[test]
    fn transform_into_any_preserves_rows_and_key_types() {
        let text = ValueOrRef::String(StringValueOrRef::new_ref("true"));
        for dictionary in dictionaries() {
            let data_type = dictionary.keys().data_type();
            let transformed = dictionary
                .transform_into_any(|value| match value {
                    ValueOrRef::Boolean(false) => ValueOrRef::Integer(10),
                    ValueOrRef::Boolean(true) => text.clone(),
                    ValueOrRef::Null => ValueOrRef::Null,
                    other => panic!("unexpected value: {other:?}"),
                })
                .expect("transform succeeds");

            assert_eq!(transformed.keys().data_type(), data_type);
            assert_rows(
                &transformed,
                &[
                    ValueOrRef::Integer(10),
                    text.clone(),
                    ValueOrRef::Integer(10),
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                ],
            );
            assert_eq!(transformed.values().len(), 2);
            assert_eq!(transformed.get_value_index(0), Some(0));
            assert_eq!(transformed.get_value_index(1), Some(1));
            assert_eq!(transformed.get_value_index(2), Some(0));
            assert_eq!(transformed.get_value_index(3), None);
            assert_eq!(transformed.get_value_index(4), None);
        }
    }

    /// Scenario: Null rows transform into either a new value or one already produced by a non-null value.
    /// Guarantees: Nulls become valid keys and replacement values are inserted once or reuse an existing index.
    #[test]
    fn transform_into_any_replaces_nulls_and_deduplicates_replacements() {
        for replacement in [10, 30] {
            for dictionary in dictionaries() {
                let data_type = dictionary.keys().data_type();
                let transformed = dictionary
                    .transform_into_any(|value| match value {
                        ValueOrRef::Boolean(false) => ValueOrRef::Integer(10),
                        ValueOrRef::Boolean(true) => ValueOrRef::Integer(20),
                        ValueOrRef::Null => ValueOrRef::Integer(replacement),
                        other => panic!("unexpected value: {other:?}"),
                    })
                    .expect("transform succeeds");

                assert_eq!(transformed.keys().data_type(), data_type);
                assert_rows(
                    &transformed,
                    &[10, 20, 10, replacement, replacement].map(ValueOrRef::Integer),
                );
                assert_eq!(
                    transformed.values().len(),
                    if replacement == 10 { 2 } else { 3 }
                );
                assert_eq!(
                    transformed.get_value_index(3),
                    transformed.get_value_index(4)
                );
                if replacement == 10 {
                    assert_eq!(
                        transformed.get_value_index(0),
                        transformed.get_value_index(3)
                    );
                }
            }
        }
    }

    /// Scenario: One transform maps false to Null, true to an integer, and original nulls to a replacement.
    /// Guarantees: Produced nulls use the null replacement and share its value index with original nulls.
    #[test]
    fn transform_into_any_replaces_produced_and_original_nulls() {
        for dictionary in dictionaries() {
            let data_type = dictionary.keys().data_type();
            let transformed = dictionary
                .transform_into_any(|value| match value {
                    ValueOrRef::Boolean(false) => ValueOrRef::Null,
                    ValueOrRef::Boolean(true) => ValueOrRef::Integer(20),
                    ValueOrRef::Null => ValueOrRef::Integer(99),
                    other => panic!("unexpected value: {other:?}"),
                })
                .expect("transform succeeds");

            assert_eq!(transformed.keys().data_type(), data_type);
            assert_rows(&transformed, &[99, 20, 99, 99, 99].map(ValueOrRef::Integer));
            assert_eq!(transformed.values().len(), 2);
            assert_eq!(transformed.get_value_index(0), Some(1));
            for index in [2, 3, 4] {
                assert_eq!(transformed.get_value_index(index), Some(1));
            }
        }
    }

    /// Scenario: Nullable primitive and Boolean keys are sliced at offset three for seventeen rows.
    /// Guarantees: Transformation preserves sliced values and validity across multiple bitmap byte boundaries.
    #[test]
    fn transform_into_any_preserves_sliced_key_values_and_validity() {
        let input = vec![
            Some(true),
            None,
            Some(false),
            Some(false),
            Some(true),
            None,
            Some(false),
            None,
            Some(true),
            Some(true),
            None,
            Some(false),
            Some(true),
            None,
            Some(false),
            Some(true),
            None,
            Some(false),
            None,
            Some(true),
            None,
            Some(false),
        ];
        macro_rules! sliced_keys {
            ($array:ty) => {
                DictionaryKeyArray::from(
                    <$array>::from_iter(
                        input
                            .iter()
                            .map(|value| value.map(|value| if value { 1 } else { 0 })),
                    )
                    .slice(3, 17),
                )
            };
        }
        let expected = [
            Some(10),
            Some(20),
            None,
            Some(10),
            None,
            Some(20),
            Some(20),
            None,
            Some(10),
            Some(20),
            None,
            Some(10),
            Some(20),
            None,
            Some(10),
            None,
            Some(20),
        ]
        .map(|value| value.map_or(ValueOrRef::Null, ValueOrRef::Integer));

        for keys in [
            sliced_keys!(Int8Array),
            sliced_keys!(Int16Array),
            sliced_keys!(Int32Array),
            sliced_keys!(Int64Array),
            sliced_keys!(UInt8Array),
            sliced_keys!(UInt16Array),
            sliced_keys!(UInt32Array),
            sliced_keys!(UInt64Array),
        ] {
            let data_type = keys.data_type();
            for dictionary in [
                Dictionary::new(keys, DictionaryValueArray::Boolean).expect("valid"),
                Dictionary::new_boolean(
                    data_type.clone(),
                    BooleanArray::from(input.clone()).slice(3, 17),
                ),
            ] {
                let transformed = dictionary
                    .transform_into_any(|value| match value {
                        ValueOrRef::Boolean(false) => ValueOrRef::Integer(10),
                        ValueOrRef::Boolean(true) => ValueOrRef::Integer(20),
                        ValueOrRef::Null => ValueOrRef::Null,
                        other => panic!("unexpected value: {other:?}"),
                    })
                    .expect("transform succeeds");

                assert_eq!(transformed.keys().data_type(), data_type);
                assert_rows(&transformed, &expected);
                assert_eq!(transformed.values().len(), 2);
            }
        }
    }

    /// Scenario: Multiple values merge, a value is removed, or every row becomes null or the same scalar.
    /// Guarantees: Remapped keys preserve nullable rows and only all-valid single-valued output becomes a scalar.
    #[test]
    fn transform_into_any_normalizes_null_and_scalar_outputs() {
        for dictionary in dictionaries() {
            let data_type = dictionary.keys().data_type();
            let nullable = dictionary
                .clone()
                .transform_into_any(|value| match value {
                    ValueOrRef::Null => ValueOrRef::Null,
                    _ => ValueOrRef::Integer(7),
                })
                .expect("transform succeeds");
            assert_rows(
                &nullable,
                &[
                    ValueOrRef::Integer(7),
                    ValueOrRef::Integer(7),
                    ValueOrRef::Integer(7),
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                ],
            );
            assert_eq!(nullable.values().len(), 1);
            assert!(matches!(nullable.keys(), DictionaryKeyArray::KeyArray(_)));

            let removed = dictionary
                .clone()
                .transform_into_any(|value| match value {
                    ValueOrRef::Boolean(true) => ValueOrRef::Integer(20),
                    _ => ValueOrRef::Null,
                })
                .expect("transform succeeds");
            assert_rows(
                &removed,
                &[
                    ValueOrRef::Null,
                    ValueOrRef::Integer(20),
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                ],
            );
            assert_eq!(removed.values().len(), 1);

            let scalar = dictionary
                .clone()
                .transform_into_any(|_| ValueOrRef::Integer(7))
                .expect("transform succeeds");
            assert_eq!(
                scalar,
                Dictionary::new_scalar(data_type.clone(), 5, ValueOrRef::Integer(7))
            );

            let null = dictionary
                .transform_into_any(|_| ValueOrRef::Null)
                .expect("transform succeeds");
            assert_eq!(null, Dictionary::new_null(data_type, 5));
        }
    }

    /// Scenario: Primitive keys reference Arrow, Vec, and Set values containing duplicates or an intrinsic null.
    /// Guarantees: Each value storage variant remaps duplicate indexes and preserves value-level and key-level nulls.
    #[test]
    fn transform_into_any_supports_value_storage_variants() {
        let values = vec![
            ValueOrRef::Integer(10),
            ValueOrRef::Integer(20),
            ValueOrRef::Integer(10),
            ValueOrRef::Null,
        ];
        let set: ValueOrRefSet<'_> = values.iter().cloned().collect();
        for values in [
            DictionaryValueArray::from(&Int32Array::from(vec![Some(10), Some(20), Some(10), None])),
            values.into(),
            set.into(),
        ] {
            let last_index = values.len() - 1;
            let dictionary = Dictionary::new(
                UInt64Array::from(vec![
                    Some(1),
                    Some(0),
                    Some(last_index as u64),
                    None,
                    Some(0),
                ])
                .into(),
                values,
            )
            .expect("valid");
            let transformed = dictionary
                .transform_into_any(Clone::clone)
                .expect("transform succeeds");
            assert_rows(
                &transformed,
                &[
                    ValueOrRef::Integer(20),
                    ValueOrRef::Integer(10),
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                    ValueOrRef::Integer(10),
                ],
            );
            assert_eq!(transformed.values().len(), 2);
            assert_eq!(transformed.get_value_index(0), Some(1));
            assert_eq!(transformed.get_value_index(1), Some(0));
        }
    }

    /// Scenario: Repeated primitive and Boolean keys contain multiple null keys and a stateful transform.
    /// Guarantees: The callback runs once per dictionary value and once for null keys, not once per row.
    #[test]
    fn transform_into_any_caches_null_key_transformation() {
        for dictionary in dictionaries().into_iter().filter(|dictionary| {
            !matches!(dictionary.keys(), DictionaryKeyArray::UniqueValues { .. })
        }) {
            let mut calls = Vec::new();
            let transformed = dictionary
                .transform_into_any(|value| {
                    calls.push(value.clone());
                    ValueOrRef::Integer(calls.len() as i64)
                })
                .expect("transform succeeds");
            assert_eq!(
                calls,
                vec![
                    ValueOrRef::Boolean(false),
                    ValueOrRef::Boolean(true),
                    ValueOrRef::Null
                ]
            );
            assert_rows(&transformed, &[1, 2, 1, 3, 3].map(ValueOrRef::Integer));
        }
    }

    /// Scenario: Scalar and null dictionaries of every key type transform to either a scalar or null.
    /// Guarantees: The callback receives the logical scalar once and output preserves the key type and row count.
    #[test]
    fn transform_into_any_transforms_single_values() {
        for keys in primitive_keys(5) {
            let data_type = keys.data_type();
            for value in [ValueOrRef::Integer(42), ValueOrRef::Null] {
                let dictionary = if matches!(value, ValueOrRef::Null) {
                    Dictionary::new_null(data_type.clone(), 4)
                } else {
                    Dictionary::new_scalar(data_type.clone(), 4, value.clone())
                };
                for replacement in [ValueOrRef::Boolean(true), ValueOrRef::Null] {
                    let mut calls = 0;
                    let transformed = dictionary
                        .clone()
                        .transform_into_any(|input| {
                            calls += 1;
                            assert_eq!(input, &value);
                            replacement.clone()
                        })
                        .expect("transform succeeds");
                    assert_eq!(calls, 1);
                    let expected = if matches!(replacement, ValueOrRef::Null) {
                        Dictionary::new_null(data_type.clone(), 4)
                    } else {
                        Dictionary::new_scalar(data_type.clone(), 4, replacement)
                    };
                    assert_eq!(transformed, expected);
                }
            }
        }
    }

    /// Scenario: Primitive, Boolean, unique-value, scalar, and null dictionaries have zero rows.
    /// Guarantees: Transformation preserves empty length and key type without introducing rows.
    #[test]
    fn transform_into_any_preserves_empty_dictionaries() {
        for keys in primitive_keys(0) {
            let data_type = keys.data_type();
            for dictionary in [
                Dictionary::new(keys, vec![].into()).expect("valid"),
                Dictionary::new_boolean(data_type.clone(), BooleanArray::from(Vec::<bool>::new())),
                Dictionary::new_unique_values(data_type.clone(), vec![].into()),
                Dictionary::new_scalar(data_type.clone(), 0, ValueOrRef::Integer(7)),
                Dictionary::new_null(data_type.clone(), 0),
            ] {
                let transformed = dictionary
                    .transform_into_any(Clone::clone)
                    .expect("transform succeeds");
                assert!(transformed.is_empty());
                assert_eq!(transformed.keys().data_type(), data_type);
            }
        }
    }

    /// Scenario: Primitive, unique-value, and scalar keys reference an Arrow integer that cannot fit in i64.
    /// Guarantees: Value conversion failures propagate unchanged without invoking the callback.
    #[test]
    fn transform_into_any_propagates_conversion_errors() {
        let values = DictionaryValueArray::from(&UInt64Array::from(vec![u64::MAX]));
        for keys in [
            DictionaryKeyArray::from(Int8Array::from(vec![0])),
            DictionaryKeyArray::UniqueValues {
                data_type: DictionaryKeyType::Int8,
                length: 1,
            },
            DictionaryKeyArray::SingleValue {
                data_type: DictionaryKeyType::Int8,
                length: 3,
                value_index: Some(0),
            },
        ] {
            let dictionary = Dictionary::new(keys, values.clone()).expect("valid keys");
            assert_eq!(
                dictionary.transform_into_any(|_| panic!("unconvertible value")),
                Err(DictionaryValueError::IntegerConversionFailure {
                    original_value: u64::MAX
                })
            );
        }
    }
}
