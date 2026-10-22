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
