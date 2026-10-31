// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use arrow::{array::*, datatypes::*};

use crate::*;

impl<'a> Dictionary<'a> {
    /// Transforms values, reusing keys and validity buffers when indexes and null keys are unchanged.
    /// Empty value sets and non-null single-value outputs are normalized to null or scalar dictionaries.
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

    let (mut transformed_values, value_index_lookup) = values.transform_into_set(&mut transform)?;

    let mut null_value = None;
    if value_index_lookup.is_none() {
        if keys.null_count() > 0 {
            null_value = Some(transform(&ValueOrRef::Null));
        }
        if null_value
            .as_ref()
            .is_none_or(|value| matches!(value, ValueOrRef::Null))
        {
            return Ok(dictionary_with_unchanged_keys(
                keys.into(),
                transformed_values,
            ));
        }
    }

    let mut key_builder = DictionaryKeyArrayBuilder::<K>::new(key_length);
    let mut key_writer = key_builder.get_writer();
    let mut null_index = None;

    for (key_index, value_index) in keys.into_iter().enumerate() {
        if let Some(value_index) = value_index.map(<K as ArrowPrimitiveType>::Native::as_usize) {
            if let Some(value_index_lookup) = value_index_lookup.as_ref() {
                if let Some(Some(transformed_value_index)) = value_index_lookup.get(&value_index) {
                    unsafe {
                        key_writer.set_value_index_unchecked(key_index, *transformed_value_index)?
                    };
                } else {
                    unsafe { key_writer.set_null_unchecked(key_index) };
                }
                continue;
            } else {
                unsafe { key_writer.set_value_index_unchecked(key_index, value_index)? };
                continue;
            }
        }

        let (has_value_index, value_index) = match null_index {
            Some(v) => v,
            None => {
                let v = match null_value
                    .take()
                    .unwrap_or_else(|| transform(&ValueOrRef::Null))
                {
                    ValueOrRef::Null => (
                        false,
                        <K as ArrowPrimitiveType>::Native::from_usize(0).unwrap(),
                    ),
                    v => {
                        let (index, _) = transformed_values.insert_full(v);
                        (
                            true,
                            DictionaryKeyArrayWriter::<'_, K>::get_typed_value_index(index)?,
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

    let (mut transformed_values, value_index_lookup) = values.transform_into_set(&mut transform)?;

    let mut null_value = None;
    if value_index_lookup.is_none() {
        if keys.null_count() > 0 {
            null_value = Some(transform(&ValueOrRef::Null));
        }
        if null_value
            .as_ref()
            .is_none_or(|value| matches!(value, ValueOrRef::Null))
        {
            return Ok(dictionary_with_unchanged_keys(
                DictionaryKeyArray::BooleanArray {
                    data_type: key_data_type,
                    values: keys,
                },
                transformed_values,
            ));
        }
    }

    let mut key_builder = DictionaryKeyArrayBuilder::<K>::new(key_length);
    let mut key_writer = key_builder.get_writer();
    let mut null_index = None;

    for (key_index, value_index) in keys.into_iter().enumerate() {
        if let Some(value_index) = value_index.map(|v| if v { 1 } else { 0 }) {
            if let Some(value_index_lookup) = value_index_lookup.as_ref() {
                if let Some(Some(transformed_value_index)) = value_index_lookup.get(&value_index) {
                    unsafe {
                        key_writer.set_value_index_unchecked(key_index, *transformed_value_index)?
                    };
                } else {
                    unsafe { key_writer.set_null_unchecked(key_index) };
                }
                continue;
            } else {
                unsafe { key_writer.set_value_index_unchecked(key_index, value_index)? };
                continue;
            }
        }

        let (has_value_index, value_index) = match null_index {
            Some(v) => v,
            None => {
                let v = match null_value
                    .take()
                    .unwrap_or_else(|| transform(&ValueOrRef::Null))
                {
                    ValueOrRef::Null => (
                        false,
                        <K as ArrowPrimitiveType>::Native::from_usize(0).unwrap(),
                    ),
                    v => {
                        let (index, _) = transformed_values.insert_full(v);
                        (
                            true,
                            DictionaryKeyArrayWriter::<'_, K>::get_typed_value_index(index)?,
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
    transform: FTransform,
) -> Result<Dictionary<'a>, DictionaryValueError>
where
    FTransform: FnMut(&ValueOrRef<'a>) -> ValueOrRef<'a>,
    DictionaryKeyArray: From<PrimitiveArray<K>>,
{
    let (transformed_values, value_index_lookup) = values.transform_into_set(transform)?;

    let value_index_lookup = match value_index_lookup {
        None => {
            // Note: Reaching here means nothing was remapped so we can skip
            // building a new key array.
            return Ok(dictionary_with_unchanged_keys(
                DictionaryKeyArray::UniqueValues {
                    data_type: key_data_type,
                    length: key_length,
                },
                transformed_values,
            ));
        }
        Some(lookup) => lookup,
    };

    let mut key_builder = DictionaryKeyArrayBuilder::<K>::new(key_length);
    let mut key_writer = key_builder.get_writer();

    for key_index in 0..key_length {
        if let Some(Some(transformed_value_index)) = value_index_lookup.get(&key_index) {
            unsafe { key_writer.set_value_index_unchecked(key_index, *transformed_value_index)? };
        } else {
            unsafe { key_writer.set_null_unchecked(key_index) };
        }
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

fn dictionary_with_unchanged_keys<'a>(
    keys: DictionaryKeyArray,
    values: ValueOrRefSet<'a>,
) -> Dictionary<'a> {
    let key_data_type = keys.data_type();
    let key_length = keys.len();
    match values.len() {
        0 => Dictionary::new_null(key_data_type, key_length),
        1 if keys.nulls().is_none_or(|nulls| nulls.null_count() == 0) => Dictionary::new_scalar(
            key_data_type,
            key_length,
            values.into_iter().next().expect("has value"),
        ),
        // Callers preserve value indexes and leave null keys unchanged.
        _ => unsafe { Dictionary::new_unvalidated(keys, values.into()) },
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use half::f16;

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
        let nulls = dictionary.nulls();
        for (index, value) in expected.iter().enumerate() {
            assert_eq!(
                dictionary.get_value(index).as_ref(),
                Ok(value),
                "row {index}"
            );
            assert_eq!(
                nulls.as_ref().is_some_and(|nulls| nulls.is_null(index)),
                matches!(value, ValueOrRef::Null),
                "row {index} validity"
            );
        }
    }

    fn assert_key_buffers_reused(original: &DictionaryKeyArray, actual: &DictionaryKeyArray) {
        let array_data = |keys: &DictionaryKeyArray| match keys {
            DictionaryKeyArray::KeyArray(keys) => keys.as_array().to_data(),
            DictionaryKeyArray::BooleanArray { values, .. } => values.to_data(),
            other => panic!("expected array keys: {other:?}"),
        };
        assert_eq!(original.data_type(), actual.data_type());
        let original = array_data(original);
        let actual = array_data(actual);
        assert_eq!(original.data_type(), actual.data_type());
        assert_eq!(original.len(), actual.len());
        assert_eq!(original.offset(), actual.offset());
        assert_eq!(original.buffers().len(), actual.buffers().len());
        for (original, actual) in original.buffers().iter().zip(actual.buffers()) {
            assert_eq!(original.as_ptr(), actual.as_ptr());
            assert_eq!(original.len(), actual.len());
        }
        match (original.nulls(), actual.nulls()) {
            (Some(original), Some(actual)) => {
                assert_eq!(
                    original.inner().values().as_ptr(),
                    actual.inner().values().as_ptr()
                );
                assert_eq!(original.offset(), actual.offset());
                assert_eq!(original.len(), actual.len());
            }
            (None, None) => {}
            other => panic!("validity buffers differ: {other:?}"),
        }
    }

    /// Scenario: Primitive and Boolean keys have either no null rows or null rows that remain null after an injective transform.
    /// Guarantees: Every key type reuses its key and validity buffers, and the callback runs once per value and at most once for null keys.
    #[test]
    fn transform_into_any_reuses_unchanged_key_buffers() {
        for length in [3, 5] {
            for keys in primitive_keys(length) {
                let data_type = keys.data_type();
                for dictionary in [
                    Dictionary::new(keys, DictionaryValueArray::Boolean).expect("valid"),
                    Dictionary::new_boolean(
                        data_type,
                        BooleanArray::from(vec![Some(false), Some(true), Some(false), None, None])
                            .slice(0, length),
                    ),
                ] {
                    let original_keys = dictionary.keys().clone();
                    let mut calls = Vec::new();
                    let transformed = dictionary
                        .transform_into_any(|value| {
                            calls.push(value.clone());
                            match value {
                                ValueOrRef::Boolean(false) => ValueOrRef::Integer(10),
                                ValueOrRef::Boolean(true) => ValueOrRef::Integer(20),
                                ValueOrRef::Null => ValueOrRef::Null,
                                other => panic!("unexpected value: {other:?}"),
                            }
                        })
                        .expect("transform succeeds");
                    assert_key_buffers_reused(&original_keys, transformed.keys());
                    let expected = [
                        ValueOrRef::Integer(10),
                        ValueOrRef::Integer(20),
                        ValueOrRef::Integer(10),
                        ValueOrRef::Null,
                        ValueOrRef::Null,
                    ];
                    assert_rows(&transformed, &expected[..length]);
                    let mut expected_calls =
                        vec![ValueOrRef::Boolean(false), ValueOrRef::Boolean(true)];
                    if length == 5 {
                        expected_calls.push(ValueOrRef::Null);
                    }
                    assert_eq!(calls, expected_calls);
                }
            }
        }
    }

    /// Scenario: Primitive keys need no remapping and reference either no values or one non-null value.
    /// Guarantees: The buffer-reuse fast path retains null and scalar normalization instead of returning redundant key arrays.
    #[test]
    fn transform_into_any_normalizes_unchanged_key_outputs() {
        let null = Dictionary::new(UInt8Array::from(vec![None, None]).into(), vec![].into())
            .expect("valid null keys")
            .transform_into_any(Clone::clone)
            .expect("transform succeeds");
        assert_eq!(null, Dictionary::new_null(DictionaryKeyType::UInt8, 2));

        let scalar = Dictionary::new(
            UInt8Array::from(vec![0, 0]).into(),
            vec![ValueOrRef::Integer(10)].into(),
        )
        .expect("valid scalar keys")
        .transform_into_any(Clone::clone)
        .expect("transform succeeds");
        assert_eq!(
            scalar,
            Dictionary::new_scalar(DictionaryKeyType::UInt8, 2, ValueOrRef::Integer(10))
        );
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
    /// Guarantees: Produced nulls stay null while only original nulls receive the replacement value.
    #[test]
    fn transform_into_any_preserves_produced_nulls_and_replaces_original_nulls() {
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
            assert_rows(
                &transformed,
                &[
                    ValueOrRef::Null,
                    ValueOrRef::Integer(20),
                    ValueOrRef::Null,
                    ValueOrRef::Integer(99),
                    ValueOrRef::Integer(99),
                ],
            );
            assert_eq!(transformed.values().len(), 2);
            for index in [0, 2] {
                assert_eq!(transformed.get_value_index(index), None);
            }
            for index in [3, 4] {
                assert_eq!(transformed.get_value_index(index), Some(1));
            }
        }
    }

    /// Scenario: Equivalent scalar, primitive, Boolean, and unique-value rows transform false to Null and Null to 99.
    /// Guarantees: Every representation agrees and produced nulls never cause an extra callback invocation.
    #[test]
    fn transform_into_any_matches_scalar_null_semantics() {
        fn transform(value: &ValueOrRef<'static>) -> ValueOrRef<'static> {
            match value {
                ValueOrRef::Boolean(false) => ValueOrRef::Null,
                ValueOrRef::Boolean(true) => ValueOrRef::Integer(20),
                ValueOrRef::Null => ValueOrRef::Integer(99),
                other => panic!("unexpected value: {other:?}"),
            }
        }

        for keys in primitive_keys(3) {
            let data_type = keys.data_type();
            for input in [Some(false), Some(true), None] {
                let value = input.map_or(ValueOrRef::Null, ValueOrRef::Boolean);
                let scalar = Dictionary::new_scalar(data_type.clone(), 3, value.clone())
                    .transform_into_any(transform)
                    .expect("transform succeeds");
                assert_rows(&scalar, &vec![transform(&value); 3]);

                for dictionary in [
                    Dictionary::new(keys.clone(), vec![value.clone(); 2].into()).expect("valid"),
                    Dictionary::new_boolean(data_type.clone(), BooleanArray::from(vec![input; 3])),
                    Dictionary::new_unique_values(data_type.clone(), vec![value.clone(); 3].into()),
                ] {
                    let expected_null_calls = if input.is_none() {
                        match dictionary.keys() {
                            DictionaryKeyArray::BooleanArray { .. } => 1,
                            _ => dictionary.values().len(),
                        }
                    } else {
                        0
                    };
                    let mut null_calls = 0;
                    let transformed = dictionary
                        .transform_into_any(|value| {
                            if matches!(value, ValueOrRef::Null) {
                                null_calls += 1;
                            }
                            transform(value)
                        })
                        .expect("transform succeeds");
                    assert_eq!(transformed.keys().data_type(), data_type);
                    assert_rows(&transformed, &vec![transform(&value); 3]);
                    for index in 0..3 {
                        assert_eq!(transformed.get_value(index), scalar.get_value(index));
                    }
                    assert_eq!(null_calls, expected_null_calls);
                }
            }
        }
    }

    /// Scenario: Nullable primitive and Boolean keys are sliced at offset three for seventeen rows.
    /// Guarantees: Transformation preserves sliced values and reuses key and validity buffers across multiple bitmap byte boundaries.
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
                let original_keys = dictionary.keys().clone();
                let transformed = dictionary
                    .transform_into_any(|value| match value {
                        ValueOrRef::Boolean(false) => ValueOrRef::Integer(10),
                        ValueOrRef::Boolean(true) => ValueOrRef::Integer(20),
                        ValueOrRef::Null => ValueOrRef::Null,
                        other => panic!("unexpected value: {other:?}"),
                    })
                    .expect("transform succeeds");

                assert_eq!(transformed.keys().data_type(), data_type);
                assert_key_buffers_reused(&original_keys, transformed.keys());
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

    /// Scenario: Null replacement adds or reuses a value at the capacity of signed and unsigned integer keys.
    /// Guarantees: The largest index succeeds, existing values and nulls remain usable at capacity, and overflow returns an error.
    #[test]
    fn transform_into_any_checks_null_replacement_key_capacity() {
        fn check<K: ArrowDictionaryKeyType>(capacity: usize)
        where
            DictionaryKeyArray: From<PrimitiveArray<K>>,
        {
            for value_count in [capacity - 1, capacity] {
                let keys = PrimitiveArray::<K>::from_iter([
                    Some(K::Native::from_usize(value_count - 1).expect("index fits")),
                    None,
                    None,
                ]);
                let values = (0..value_count)
                    .map(|index| ValueOrRef::Integer(index as i64))
                    .collect::<Vec<_>>();
                let dictionary = Dictionary::new(keys.into(), values.into()).expect("valid");
                let data_type = dictionary.keys().data_type();

                let unchanged = dictionary
                    .clone()
                    .transform_into_any(Clone::clone)
                    .expect("nulls do not need another value");
                assert_rows(
                    &unchanged,
                    &[
                        ValueOrRef::Integer((value_count - 1) as i64),
                        ValueOrRef::Null,
                        ValueOrRef::Null,
                    ],
                );
                assert_eq!(unchanged.keys().data_type(), data_type);

                let reused = dictionary
                    .clone()
                    .transform_into_any(|value| match value {
                        ValueOrRef::Null => ValueOrRef::Integer(0),
                        value => value.clone(),
                    })
                    .expect("existing replacement fits");
                assert_rows(
                    &reused,
                    &[(value_count - 1) as i64, 0, 0].map(ValueOrRef::Integer),
                );
                assert_eq!(reused.values().len(), value_count);
                assert_eq!(reused.get_value_index(1), Some(0));
                assert_eq!(reused.get_value_index(2), Some(0));

                let added = dictionary.transform_into_any(|value| match value {
                    ValueOrRef::Null => ValueOrRef::Integer(-1),
                    value => value.clone(),
                });
                if value_count == capacity {
                    assert_eq!(
                        added,
                        Err(DictionaryValueError::KeyOverflow {
                            value_index: capacity,
                            key_data_type: K::DATA_TYPE,
                        })
                    );
                } else {
                    let added = added.expect("last representable index fits");
                    assert_rows(
                        &added,
                        &[(value_count - 1) as i64, -1, -1].map(ValueOrRef::Integer),
                    );
                    assert_eq!(added.keys().data_type(), data_type);
                    assert_eq!(added.values().len(), capacity);
                    assert_eq!(added.get_value_index(1), Some(capacity - 1));
                    assert_eq!(added.get_value_index(2), Some(capacity - 1));
                }
            }
        }

        check::<Int8Type>(128);
        check::<UInt8Type>(256);
        check::<Int16Type>(32_768);
        check::<UInt16Type>(65_536);
    }

    /// Scenario: Unique-value columns of every key type transform 300 distinct values without changing indexes.
    /// Guarantees: Identity and injective transforms preserve all rows and implicit keys even beyond the tagged key width.
    #[test]
    fn transform_into_any_preserves_implicit_unique_value_keys() {
        for data_type in [
            DictionaryKeyType::Int8,
            DictionaryKeyType::UInt8,
            DictionaryKeyType::Int16,
            DictionaryKeyType::UInt16,
            DictionaryKeyType::Int32,
            DictionaryKeyType::UInt32,
            DictionaryKeyType::Int64,
            DictionaryKeyType::UInt64,
        ] {
            for offset in [0, 1000] {
                let values = (0..300).map(ValueOrRef::Integer).collect::<Vec<_>>();
                let transformed = Dictionary::new_unique_values(data_type.clone(), values.into())
                    .transform_into_any(|value| match value {
                        ValueOrRef::Integer(value) => ValueOrRef::Integer(value + offset),
                        other => panic!("unexpected value: {other:?}"),
                    })
                    .expect("implicit indexes are not limited by the key type");

                assert_eq!(
                    transformed.keys(),
                    &DictionaryKeyArray::UniqueValues {
                        data_type: data_type.clone(),
                        length: 300,
                    }
                );
                let expected = (0..300)
                    .map(|index| ValueOrRef::Integer(index + offset))
                    .collect::<Vec<_>>();
                assert_rows(&transformed, &expected);
            }
        }
    }

    /// Scenario: Unique-value keys address only a prefix of a larger value array without needing remapping.
    /// Guarantees: Preserving implicit indexes retains the original row count rather than the value count.
    #[test]
    fn transform_into_any_preserves_implicit_unique_value_key_length() {
        let keys = DictionaryKeyArray::UniqueValues {
            data_type: DictionaryKeyType::UInt8,
            length: 2,
        };
        let values = vec![1, 2, 3]
            .into_iter()
            .map(ValueOrRef::Integer)
            .collect::<Vec<_>>();
        let transformed = Dictionary::new(keys.clone(), values.into())
            .expect("valid prefix")
            .transform_into_any(Clone::clone)
            .expect("implicit indexes are preserved");

        assert_eq!(transformed.keys(), &keys);
        assert_rows(
            &transformed,
            &[ValueOrRef::Integer(1), ValueOrRef::Integer(2)],
        );
    }

    /// Scenario: Unique-value rows exceed the tagged key capacity with implicit indexes, duplicates, or nulls.
    /// Guarantees: Implicit indexes remain unrestricted; remapped indexes must fit, and produced nulls use no capacity.
    #[test]
    fn transform_into_any_checks_unique_value_key_capacity() {
        for (data_type, arrow_data_type, capacity) in [
            (DictionaryKeyType::Int8, DataType::Int8, 128),
            (DictionaryKeyType::UInt8, DataType::UInt8, 256),
            (DictionaryKeyType::Int16, DataType::Int16, 32_768),
            (DictionaryKeyType::UInt16, DataType::UInt16, 65_536),
        ] {
            let values = (0..=capacity)
                .map(|index| ValueOrRef::Integer(index as i64))
                .collect::<Vec<_>>();
            for length in [capacity - 1, capacity, capacity + 1] {
                let transformed = Dictionary::new_unique_values(
                    data_type.clone(),
                    values[..length].to_vec().into(),
                )
                .transform_into_any(Clone::clone)
                .expect("implicit indexes are not limited by the key type");
                assert_eq!(
                    transformed.keys(),
                    &DictionaryKeyArray::UniqueValues {
                        data_type: data_type.clone(),
                        length,
                    }
                );
                assert_rows(&transformed, &values[..length]);
            }

            let collapsed = Dictionary::new_unique_values(data_type.clone(), values.clone().into())
                .transform_into_any(|value| match value {
                    ValueOrRef::Integer(value) => ValueOrRef::Integer(value % 2),
                    other => panic!("unexpected value: {other:?}"),
                })
                .expect("remapped indexes fit even when the row count exceeds capacity");
            let expected = (0..=capacity)
                .map(|index| ValueOrRef::Integer((index % 2) as i64))
                .collect::<Vec<_>>();
            assert_rows(&collapsed, &expected);
            assert_eq!(collapsed.keys().data_type(), data_type);
            assert_eq!(collapsed.values().len(), 2);

            let mut duplicates = vec![ValueOrRef::Integer(0)];
            duplicates.extend(values.iter().cloned());
            let mut produced_null = values[..capacity].to_vec();
            produced_null.push(ValueOrRef::Boolean(false));

            let mut original_null = values[..capacity].to_vec();
            original_null.push(ValueOrRef::Null);

            let replaced = Dictionary::new_unique_values(data_type.clone(), original_null.into())
                .transform_into_any(|value| match value {
                    ValueOrRef::Null => ValueOrRef::Integer(-1),
                    value => value.clone(),
                })
                .expect("distinct null replacement preserves implicit indexes");
            let mut expected = values[..capacity].to_vec();
            expected.push(ValueOrRef::Integer(-1));
            assert_rows(&replaced, &expected);
            assert_eq!(
                replaced.keys(),
                &DictionaryKeyArray::UniqueValues {
                    data_type: data_type.clone(),
                    length: capacity + 1,
                }
            );

            let dictionary = Dictionary::new_unique_values(data_type.clone(), duplicates.into());
            assert_eq!(
                dictionary.transform_into_any(Clone::clone),
                Err(DictionaryValueError::KeyOverflow {
                    value_index: capacity,
                    key_data_type: arrow_data_type.clone(),
                })
            );

            let transformed =
                Dictionary::new_unique_values(data_type.clone(), produced_null.into())
                    .transform_into_any(|value| match value {
                        ValueOrRef::Boolean(false) => ValueOrRef::Null,
                        ValueOrRef::Null => panic!("produced null must not be transformed"),
                        value => value.clone(),
                    })
                    .expect("produced null does not need another value index");
            assert_eq!(transformed.len(), capacity + 1);
            for index in 0..capacity {
                assert_eq!(
                    transformed.get_value(index),
                    Ok(ValueOrRef::Integer(index as i64))
                );
            }
            assert_eq!(transformed.get_value(capacity), Ok(ValueOrRef::Null));
            let nulls = transformed.nulls().expect("produced null is tracked");
            assert_eq!(nulls.null_count(), 1);
            assert!(nulls.is_null(capacity));
            assert_eq!(transformed.keys().data_type(), data_type);
            assert_eq!(transformed.values().len(), capacity);
        }
    }

    /// Scenario: Every Arrow value type transforms nullable values with repeated inputs and merging outputs.
    /// Guarantees: Callbacks receive exact converted values, duplicates share indexes, and remapped rows preserve values and validity.
    #[test]
    fn transform_into_any_supports_all_arrow_value_types() {
        let mut cases = Vec::new();
        macro_rules! case {
            ($array:ty, $first:expr, $second:expr, $expected_first:expr, $expected_second:expr) => {{
                let first = $expected_first;
                let second = $expected_second;
                cases.push((
                    DictionaryValueArray::from(&<$array>::from(vec![
                        Some($first),
                        None,
                        Some($second),
                        Some($first),
                    ])),
                    vec![first.clone(), ValueOrRef::Null, second, first],
                ));
            }};
        }
        macro_rules! integer_case {
            ($array:ty, $first:expr) => {
                case!(
                    $array,
                    $first,
                    9,
                    ValueOrRef::Integer($first),
                    ValueOrRef::Integer(9)
                );
            };
        }
        integer_case!(Int8Array, -7);
        integer_case!(Int16Array, -7);
        integer_case!(Int32Array, -7);
        integer_case!(Int64Array, -7);
        integer_case!(UInt8Array, 7);
        integer_case!(UInt16Array, 7);
        integer_case!(UInt32Array, 7);
        integer_case!(UInt64Array, 7);
        case!(
            Float16Array,
            f16::from_f32(-1.5),
            f16::from_f32(2.25),
            ValueOrRef::Double(-1.5),
            ValueOrRef::Double(2.25)
        );
        case!(
            Float32Array,
            -1.5,
            2.25,
            ValueOrRef::Double(-1.5),
            ValueOrRef::Double(2.25)
        );
        case!(
            Float64Array,
            -1.5,
            2.25,
            ValueOrRef::Double(-1.5),
            ValueOrRef::Double(2.25)
        );
        case!(
            StringArray,
            "\u{e9}\u{65e5}",
            "",
            ValueOrRef::String(StringValueOrRef::new_ref("\u{e9}\u{65e5}")),
            ValueOrRef::String(StringValueOrRef::new_ref(""))
        );
        case!(
            LargeStringArray,
            "\u{e9}\u{65e5}",
            "",
            ValueOrRef::String(StringValueOrRef::new_ref("\u{e9}\u{65e5}")),
            ValueOrRef::String(StringValueOrRef::new_ref(""))
        );
        let before_epoch = ValueOrRef::DateTime(
            Utc.timestamp_opt(-2, 0)
                .single()
                .expect("valid timestamp")
                .into(),
        );
        let after_epoch = ValueOrRef::DateTime(
            Utc.timestamp_opt(3, 0)
                .single()
                .expect("valid timestamp")
                .into(),
        );
        case!(
            TimestampSecondArray,
            -2,
            3,
            before_epoch.clone(),
            after_epoch.clone()
        );
        case!(
            TimestampMillisecondArray,
            -2_000,
            3_000,
            before_epoch.clone(),
            after_epoch.clone()
        );
        case!(
            TimestampMicrosecondArray,
            -2_000_000,
            3_000_000,
            before_epoch.clone(),
            after_epoch.clone()
        );
        case!(
            TimestampNanosecondArray,
            -2_000_000_000,
            3_000_000_000,
            before_epoch,
            after_epoch
        );
        let mut binary = FixedSizeBinaryBuilder::new(3);
        binary.append_value(b"\0\x80\xff").expect("valid width");
        binary.append_null();
        binary.append_value(b"abc").expect("valid width");
        binary.append_value(b"\0\x80\xff").expect("valid width");
        let first = ValueOrRef::Array(ArrayValueOrRef::from(
            (*b"\0\x80\xff").map(|byte| ValueOrRef::Integer(i64::from(byte))),
        ));
        let second = ValueOrRef::Array(ArrayValueOrRef::from(
            (*b"abc").map(|byte| ValueOrRef::Integer(i64::from(byte))),
        ));
        cases.push((
            DictionaryValueArray::from(&binary.finish()),
            vec![first.clone(), ValueOrRef::Null, second, first],
        ));

        for (values, expected_inputs) in cases {
            let dictionary = Dictionary::new(UInt32Array::from(vec![2, 0, 3, 1, 0]).into(), values)
                .expect("valid keys");
            for merge in [false, true] {
                let mut inputs = Vec::new();
                let transformed = dictionary
                    .clone()
                    .transform_into_any(|value| {
                        inputs.push(value.clone());
                        if merge {
                            ValueOrRef::Integer(if matches!(value, ValueOrRef::Null) {
                                99
                            } else {
                                42
                            })
                        } else {
                            value.clone()
                        }
                    })
                    .expect("transform succeeds");
                assert_eq!(inputs, expected_inputs);
                assert_eq!(transformed.keys().data_type(), DictionaryKeyType::UInt32);
                let expected_rows = if merge {
                    [42, 42, 42, 99, 42].map(ValueOrRef::Integer)
                } else {
                    [
                        expected_inputs[2].clone(),
                        expected_inputs[0].clone(),
                        expected_inputs[0].clone(),
                        ValueOrRef::Null,
                        expected_inputs[0].clone(),
                    ]
                };
                assert_rows(&transformed, &expected_rows);
                assert_eq!(transformed.values().len(), 2);
                let expected_indexes = if merge {
                    [Some(0), Some(0), Some(0), Some(1), Some(0)]
                } else {
                    [Some(1), Some(0), Some(0), None, Some(0)]
                };
                for (row, expected) in expected_indexes.into_iter().enumerate() {
                    assert_eq!(transformed.get_value_index(row), expected);
                }
            }
        }
    }

    /// Scenario: Timestamp arrays in second, millisecond, and microsecond units fail conversion after a valid value.
    /// Guarantees: Primitive, unique-value, and scalar transforms return the exact error without calling back for invalid or later values.
    #[test]
    fn transform_into_any_propagates_timestamp_conversion_errors() {
        for invalid in [i64::MIN, i64::MAX] {
            let input = vec![0, invalid, 1];
            for (values, time_unit) in [
                (
                    DictionaryValueArray::from(&TimestampSecondArray::from(input.clone())),
                    TimeUnit::Second,
                ),
                (
                    DictionaryValueArray::from(&TimestampMillisecondArray::from(input.clone())),
                    TimeUnit::Millisecond,
                ),
                (
                    DictionaryValueArray::from(&TimestampMicrosecondArray::from(input.clone())),
                    TimeUnit::Microsecond,
                ),
            ] {
                for keys in [
                    DictionaryKeyArray::from(Int8Array::from(vec![0, 1, 2])),
                    DictionaryKeyArray::UniqueValues {
                        data_type: DictionaryKeyType::Int8,
                        length: 3,
                    },
                    DictionaryKeyArray::SingleValue {
                        data_type: DictionaryKeyType::Int8,
                        length: 3,
                        value_index: Some(1),
                    },
                ] {
                    let scalar = matches!(keys, DictionaryKeyArray::SingleValue { .. });
                    let dictionary = Dictionary::new(keys, values.clone()).expect("valid keys");
                    let mut inputs = Vec::new();
                    assert_eq!(
                        dictionary.transform_into_any(|value| {
                            inputs.push(value.clone());
                            value.clone()
                        }),
                        Err(DictionaryValueError::TimestampConversionFailure {
                            original_value: invalid,
                            time_unit,
                        })
                    );
                    let expected_inputs = if scalar {
                        vec![]
                    } else {
                        vec![ValueOrRef::DateTime(
                            Utc.timestamp_opt(0, 0)
                                .single()
                                .expect("valid timestamp")
                                .into(),
                        )]
                    };
                    assert_eq!(inputs, expected_inputs);
                }
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
