// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{self, Write};

use arrow::{array::*, buffer::*, datatypes::*};
use otel_arrow_contrib_data_engine_expressions::*;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Dictionary<'a> {
    keys: DictionaryKeyArray,
    values: DictionaryValueArray<'a>,
}

impl Dictionary<'_> {
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn is_null(&self) -> bool {
        if self.keys.is_null() || self.values.is_null() {
            return true;
        }

        for key_index in 0..self.len() {
            if let Some(value_index) = self.keys.get_value_index_for_key_index(key_index)
                && !self.values.is_null_at(value_index)
            {
                return false;
            }
        }

        true
    }

    pub fn nulls(&self) -> Option<NullBuffer> {
        let key_nulls = self.keys.nulls();

        if let Some(v) = key_nulls.as_ref()
            && v.null_count() == self.keys.len()
        {
            return key_nulls;
        }

        let value_nulls = self.values.nulls();
        let has_value_nulls = value_nulls
            .as_ref()
            .is_some_and(|nulls| nulls.null_count() > 0);
        let has_invalid_value_indices =
            !has_value_nulls && self.keys.has_value_index_out_of_bounds(self.values.len());

        if has_value_nulls || has_invalid_value_indices {
            let key_length = self.keys.len();

            let mut builder: BooleanBufferBuilder = key_nulls.map_or_else(
                || {
                    let mut builder = BooleanBufferBuilder::new(key_length);
                    builder.append_n(key_length, true);
                    builder
                },
                |v| {
                    let mut builder = BooleanBufferBuilder::new(v.len());
                    builder.append_buffer(v.inner());
                    builder
                },
            );

            for key_index in 0..key_length {
                if let Some(value_index) = self.keys.get_value_index_for_key_index(key_index)
                    && (value_index >= self.values.len()
                        || value_nulls
                            .as_ref()
                            .is_some_and(|nulls| nulls.is_null(value_index)))
                {
                    builder.set_bit(key_index, false);
                }
            }

            return Some(NullBuffer::new(builder.into()));
        }

        key_nulls
    }

    pub fn keys(&self) -> &DictionaryKeyArray {
        &self.keys
    }

    pub fn get_value_index(&self, key_index: usize) -> Option<usize> {
        self.keys.get_value_index_for_key_index(key_index)
    }

    pub fn diagnostic_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{keys:[")?;
        for key in 0..self.keys.len() {
            if key > 0 {
                f.write_char(',')?;
            }
            write!(f, "{:?}", self.get_value_index(key))?;
        }
        f.write_str("],values:[")?;
        for value in 0..self.values.len() {
            if value > 0 {
                f.write_char(',')?;
            }
            self.values
                .get_value_at(value)
                .unwrap_or(ValueOrRef::Null)
                .to_value()
                .diagnostic_fmt(f)?;
        }
        f.write_str("]}")
    }
}

impl<'a> Dictionary<'a> {
    pub fn new(keys: DictionaryKeyArray, values: DictionaryValueArray<'a>) -> Dictionary<'a> {
        Self { keys, values }
    }

    pub fn from_array<K: ArrowDictionaryKeyType, V: ArrowPrimitiveType>(
        values: &PrimitiveArray<V>,
    ) -> Dictionary<'a> {
        Self {
            keys: DictionaryKeyArray::UniqueValues {
                data_type: K::DATA_TYPE,
                length: values.len(),
            },
            values: (values as &dyn Array).into(),
        }
    }

    pub fn new_scalar_with_data_type(
        key_data_type: DataType,
        key_count: usize,
        value: ValueOrRef<'a>,
    ) -> Dictionary<'a> {
        match key_data_type {
            DataType::Int8 => Self::new_scalar::<Int8Type>(key_count, value),
            DataType::Int16 => Self::new_scalar::<Int16Type>(key_count, value),
            DataType::Int32 => Self::new_scalar::<Int32Type>(key_count, value),
            DataType::Int64 => Self::new_scalar::<Int64Type>(key_count, value),

            DataType::UInt8 => Self::new_scalar::<UInt8Type>(key_count, value),
            DataType::UInt16 => Self::new_scalar::<UInt16Type>(key_count, value),
            DataType::UInt32 => Self::new_scalar::<UInt32Type>(key_count, value),
            DataType::UInt64 => Self::new_scalar::<UInt64Type>(key_count, value),

            d => panic!("Unexpected dictionary key type '{d}' encountered"),
        }
    }

    pub fn new_scalar<K: ArrowDictionaryKeyType>(
        key_count: usize,
        value: ValueOrRef<'a>,
    ) -> Dictionary<'a> {
        Dictionary::new(
            DictionaryKeyArray::SingleValue {
                data_type: K::DATA_TYPE,
                length: key_count,
                value_index: Some(0),
            },
            vec![value].into(),
        )
    }

    pub fn new_null_with_data_type(count: usize, data_type: DataType) -> Dictionary<'a> {
        match data_type {
            DataType::Int8 => Self::new_null::<Int8Type>(count),
            DataType::Int16 => Self::new_null::<Int16Type>(count),
            DataType::Int32 => Self::new_null::<Int32Type>(count),
            DataType::Int64 => Self::new_null::<Int64Type>(count),

            DataType::UInt8 => Self::new_null::<UInt8Type>(count),
            DataType::UInt16 => Self::new_null::<UInt16Type>(count),
            DataType::UInt32 => Self::new_null::<UInt32Type>(count),
            DataType::UInt64 => Self::new_null::<UInt64Type>(count),

            d => panic!("Unexpected dictionary key type '{d}' encountered"),
        }
    }

    pub fn new_null<K: ArrowDictionaryKeyType>(count: usize) -> Dictionary<'a> {
        Dictionary::new(
            DictionaryKeyArray::SingleValue {
                data_type: K::DATA_TYPE,
                length: count,
                value_index: None,
            },
            vec![].into(),
        )
    }

    pub fn values(&self) -> &DictionaryValueArray<'a> {
        &self.values
    }

    pub fn into_parts(self) -> (DictionaryKeyArray, DictionaryValueArray<'a>) {
        (self.keys, self.values)
    }

    pub fn get_value(&self, key_index: usize) -> Result<ValueOrRef<'a>, ValueError> {
        if let Some(value_index) = self.get_value_index(key_index) {
            return self.values.get_value_at(value_index);
        }

        Ok(ValueOrRef::Null)
    }
}

macro_rules! impl_from_dictionary_array {
    ($arrow_ty:ty, $variant:ident) => {
        impl<'a> From<&DictionaryArray<$arrow_ty>> for Dictionary<'a> {
            fn from(value: &DictionaryArray<$arrow_ty>) -> Self {
                Dictionary {
                    keys: value.keys().into(),
                    values: (value.values() as &dyn Array).into(),
                }
            }
        }

        impl<'a, 'b, V> From<TypedDictionaryArray<'b, $arrow_ty, V>> for Dictionary<'a>
        where
            DictionaryValueArray<'a>: From<&'b V>,
        {
            fn from(value: TypedDictionaryArray<'b, $arrow_ty, V>) -> Self {
                Dictionary {
                    keys: value.keys().into(),
                    values: value.values().into(),
                }
            }
        }
    };
}

impl_from_dictionary_array!(Int8Type, Int8);
impl_from_dictionary_array!(Int16Type, Int16);
impl_from_dictionary_array!(Int32Type, Int32);
impl_from_dictionary_array!(Int64Type, Int64);
impl_from_dictionary_array!(UInt8Type, UInt8);
impl_from_dictionary_array!(UInt16Type, UInt16);
impl_from_dictionary_array!(UInt32Type, UInt32);
impl_from_dictionary_array!(UInt64Type, UInt64);

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    /// Scenario: Dictionary keys include both null keys and keys that point to a null dictionary value.
    /// Guarantees: Row validity combines key and value validity, and value lookup returns Null for both cases.
    #[test]
    fn dictionary_combines_key_and_value_nulls() {
        let dictionary = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(1), None, Some(0)])),
            DictionaryValueArray::from(vec![ValueOrRef::Integer(10), ValueOrRef::Null]),
        );

        let nulls = dictionary.nulls().unwrap();
        assert_eq!(nulls.null_count(), 2);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_null(1));
        assert!(nulls.is_null(2));
        assert!(nulls.is_valid(3));
        assert_eq!(dictionary.get_value(0), Ok(ValueOrRef::Integer(10)));
        assert_eq!(dictionary.get_value(1), Ok(ValueOrRef::Null));
        assert_eq!(dictionary.get_value(2), Ok(ValueOrRef::Null));
        assert_eq!(dictionary.get_value(4), Ok(ValueOrRef::Null));
    }

    /// Scenario: A non-byte-aligned slice of nullable keys references one valid and one null dictionary value.
    /// Guarantees: Combined row validity preserves the sliced key bitmap offset and value-level nulls.
    #[test]
    fn dictionary_nulls_preserve_sliced_key_validity_offset() {
        let keys = Int8Array::from(vec![None, Some(0), None, Some(1)]).slice(1, 3);
        let dictionary = Dictionary::new(
            DictionaryKeyArray::from(&keys),
            DictionaryValueArray::from(vec![ValueOrRef::Integer(10), ValueOrRef::Null]),
        );

        let nulls = dictionary.nulls().unwrap();
        assert_eq!(nulls.len(), 3);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_null(1));
        assert!(nulls.is_null(2));
    }

    /// Scenario: Valid, oversized, negative, and null keys reference a value table with no intrinsic nulls.
    /// Guarantees: Invalid references resolve to Null and contribute null row validity without requiring constructor validation.
    #[test]
    fn dictionary_handles_invalid_references_without_value_nulls() {
        let dictionary = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(1), Some(-1), None])),
            DictionaryValueArray::from(vec![ValueOrRef::Integer(10)]),
        );

        let nulls = dictionary.nulls().unwrap();
        assert_eq!(nulls.null_count(), 3);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_null(1));
        assert!(nulls.is_null(2));
        assert!(nulls.is_null(3));
        assert_eq!(dictionary.get_value(0), Ok(ValueOrRef::Integer(10)));
        assert_eq!(dictionary.get_value(1), Ok(ValueOrRef::Null));
        assert_eq!(dictionary.get_value(2), Ok(ValueOrRef::Null));
        assert_eq!(dictionary.get_value(3), Ok(ValueOrRef::Null));
    }

    /// Scenario: Dictionary keys reference a valid value, a null value, and an index beyond a nullable value table.
    /// Guarantees: Value nulls and invalid references are combined into one effective row-validity buffer.
    #[test]
    fn dictionary_combines_value_nulls_and_invalid_references() {
        let dictionary = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![0, 1, 2])),
            DictionaryValueArray::from(vec![ValueOrRef::Integer(10), ValueOrRef::Null]),
        );

        let nulls = dictionary.nulls().unwrap();
        assert_eq!(nulls.null_count(), 2);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_null(1));
        assert!(nulls.is_null(2));
    }

    /// Scenario: Dictionary rows reference only a null value while an unused non-null value remains in the value table.
    /// Guarantees: Nullness is computed from effective rows for repeated, mixed-null, non-null, and empty dictionaries.
    #[test]
    fn dictionary_is_null_uses_effective_row_validity() {
        let repeated_null = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(0)])),
            DictionaryValueArray::from(vec![ValueOrRef::Null, ValueOrRef::Integer(10)]),
        );
        assert!(repeated_null.is_null());

        let key_and_value_null = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![Some(0), None])),
            DictionaryValueArray::from(vec![ValueOrRef::Null, ValueOrRef::Integer(10)]),
        );
        assert!(key_and_value_null.is_null());

        let includes_non_null = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(1)])),
            DictionaryValueArray::from(vec![ValueOrRef::Null, ValueOrRef::Integer(10)]),
        );
        assert!(!includes_non_null.is_null());

        let empty = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(Vec::<i8>::new())),
            DictionaryValueArray::from(Vec::new()),
        );
        assert!(empty.is_null());
    }

    /// Scenario: A real Arrow dictionary contains false, true, repeated, null-valued, and null-key Boolean rows.
    /// Guarantees: Arrow-backed Boolean values convert without panicking and preserve both values and null semantics.
    #[test]
    fn arrow_boolean_dictionary_values_are_supported() {
        let arrow = DictionaryArray::<Int8Type>::try_new(
            Int8Array::from(vec![Some(0), Some(1), Some(0), Some(2), None]),
            Arc::new(BooleanArray::from(vec![Some(false), Some(true), None])),
        )
        .unwrap();
        let dictionary = Dictionary::from(&arrow);

        assert_eq!(dictionary.get_value(0), Ok(ValueOrRef::Boolean(false)));
        assert_eq!(dictionary.get_value(1), Ok(ValueOrRef::Boolean(true)));
        assert_eq!(dictionary.get_value(2), Ok(ValueOrRef::Boolean(false)));
        assert_eq!(dictionary.get_value(3), Ok(ValueOrRef::Null));
        assert_eq!(dictionary.get_value(4), Ok(ValueOrRef::Null));

        let nulls = dictionary.nulls().unwrap();
        assert_eq!(nulls.null_count(), 2);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_valid(1));
        assert!(nulls.is_valid(2));
        assert!(nulls.is_null(3));
        assert!(nulls.is_null(4));
    }

    /// Scenario: Scalar and null dictionaries are synthesized for a requested Arrow key type.
    /// Guarantees: Every row resolves to the scalar or Null and retains the requested key data type.
    #[test]
    fn scalar_and_null_dictionary_constructors_preserve_shape() {
        let scalar =
            Dictionary::new_scalar_with_data_type(DataType::UInt16, 3, ValueOrRef::Integer(42));
        assert_eq!(scalar.len(), 3);
        assert_eq!(scalar.keys().data_type(), DataType::UInt16);
        assert!(!scalar.is_null());
        assert_eq!(scalar.get_value(2), Ok(ValueOrRef::Integer(42)));
        assert_eq!(scalar.get_value(3), Ok(ValueOrRef::Null));

        let null = Dictionary::new_null_with_data_type(3, DataType::Int32);
        assert_eq!(null.len(), 3);
        assert_eq!(null.keys().data_type(), DataType::Int32);
        assert!(null.is_null());
        assert_eq!(null.nulls().unwrap().null_count(), 3);
        assert_eq!(null.get_value(0), Ok(ValueOrRef::Null));
    }

    /// Scenario: A primitive Arrow array is represented as a dictionary with one unique key per row.
    /// Guarantees: Row indexes map directly to source values and the first boundary index is rejected.
    #[test]
    fn primitive_array_dictionary_uses_unique_keys() {
        let values = Int32Array::from(vec![5, 8, 13]);
        let dictionary = Dictionary::from_array::<UInt8Type, Int32Type>(&values);

        assert_eq!(dictionary.len(), 3);
        assert_eq!(dictionary.get_value_index(2), Some(2));
        assert_eq!(dictionary.get_value(2), Ok(ValueOrRef::Integer(13)));
        assert_eq!(dictionary.get_value_index(3), None);
        assert_eq!(dictionary.get_value(3), Ok(ValueOrRef::Null));
    }

    /// Scenario: A dictionary key resolves to an Arrow UInt64 value that cannot fit in the engine integer type.
    /// Guarantees: Dictionary lookup propagates the typed value-conversion error instead of replacing it with Null.
    #[test]
    fn dictionary_lookup_propagates_value_conversion_errors() {
        let dictionary = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![0])),
            DictionaryValueArray::from(&UInt64Array::from(vec![u64::MAX])),
        );

        assert_eq!(
            dictionary.get_value(0),
            Err(ValueError::IntegerConversionFailure {
                original_value: u64::MAX,
            })
        );
    }
}
