// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{self, Write};

use arrow::{array::*, buffer::*, datatypes::*};
use otel_arrow_contrib_data_engine_expressions::*;
use thiserror::Error;

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

    pub fn nulls(&self) -> Option<NullBuffer> {
        let key_nulls = self.keys.nulls();

        if let Some(v) = key_nulls.as_ref()
            && v.null_count() == self.keys.len()
        {
            return key_nulls;
        }

        if let Some(value_nulls) = self.values.nulls()
            && value_nulls.null_count() > 0
        {
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
                    && (value_index >= self.values.len() || value_nulls.is_null(value_index))
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
    pub fn new(
        keys: DictionaryKeyArray,
        values: DictionaryValueArray<'a>,
    ) -> Result<Dictionary<'a>, DictionaryError> {
        match &keys {
            DictionaryKeyArray::KeyArray(array) => {
                for (key_index, value_index) in array.iter().enumerate() {
                    if let Some(value_index) = value_index
                        && value_index >= values.len()
                    {
                        return Err(DictionaryError::InvalidKey { key_index });
                    }
                }
            }
            DictionaryKeyArray::BooleanArray { .. } => {
                if values.len() > 2 {
                    return Err(DictionaryError::InvalidBooleanValues);
                }
            }
            DictionaryKeyArray::SingleValue { value_index, .. } => {
                if let Some(value_index) = value_index
                    && *value_index >= values.len()
                {
                    return Err(DictionaryError::InvalidKey { key_index: 0 });
                }
            }
            DictionaryKeyArray::UniqueValues { length, .. } => {
                if *length > values.len() {
                    return Err(DictionaryError::InvalidKey { key_index: *length });
                }
            }
        }
        Ok(Self { keys, values })
    }

    /// # Safety
    ///
    /// Validation that keys point to valid values is NOT performed. Only call
    /// when prior validation has been performed to ensure keys point to valid
    /// values (or null).
    pub unsafe fn new_unvalidated(
        keys: DictionaryKeyArray,
        values: DictionaryValueArray<'a>,
    ) -> Dictionary<'a> {
        Self { keys, values }
    }

    pub fn new_boolean<K: ArrowDictionaryKeyType>(values: BooleanArray) -> Dictionary<'a> {
        Self {
            keys: DictionaryKeyArray::BooleanArray {
                data_type: K::DATA_TYPE,
                values,
            },
            values: DictionaryValueArray::Boolean,
        }
    }

    pub fn new_boolean_with_data_type(
        key_data_type: DataType,
        values: BooleanArray,
    ) -> Result<Dictionary<'a>, DictionaryError> {
        Ok(match key_data_type {
            DataType::Int8 => Self::new_boolean::<Int8Type>(values),
            DataType::Int16 => Self::new_boolean::<Int16Type>(values),
            DataType::Int32 => Self::new_boolean::<Int32Type>(values),
            DataType::Int64 => Self::new_boolean::<Int64Type>(values),

            DataType::UInt8 => Self::new_boolean::<UInt8Type>(values),
            DataType::UInt16 => Self::new_boolean::<UInt16Type>(values),
            DataType::UInt32 => Self::new_boolean::<UInt32Type>(values),
            DataType::UInt64 => Self::new_boolean::<UInt64Type>(values),

            data_type => return Err(DictionaryError::UnsupportedKeyDataType { data_type }),
        })
    }

    pub fn new_unique_values<K: ArrowDictionaryKeyType>(
        values: DictionaryValueArray<'a>,
    ) -> Dictionary<'a> {
        Self {
            keys: DictionaryKeyArray::UniqueValues {
                data_type: K::DATA_TYPE,
                length: values.len(),
            },
            values,
        }
    }

    pub fn new_scalar_with_data_type(
        key_data_type: DataType,
        key_count: usize,
        value: ValueOrRef<'a>,
    ) -> Result<Dictionary<'a>, DictionaryError> {
        Ok(match key_data_type {
            DataType::Int8 => Self::new_scalar::<Int8Type>(key_count, value),
            DataType::Int16 => Self::new_scalar::<Int16Type>(key_count, value),
            DataType::Int32 => Self::new_scalar::<Int32Type>(key_count, value),
            DataType::Int64 => Self::new_scalar::<Int64Type>(key_count, value),

            DataType::UInt8 => Self::new_scalar::<UInt8Type>(key_count, value),
            DataType::UInt16 => Self::new_scalar::<UInt16Type>(key_count, value),
            DataType::UInt32 => Self::new_scalar::<UInt32Type>(key_count, value),
            DataType::UInt64 => Self::new_scalar::<UInt64Type>(key_count, value),

            data_type => return Err(DictionaryError::UnsupportedKeyDataType { data_type }),
        })
    }

    pub fn new_scalar<K: ArrowDictionaryKeyType>(
        key_count: usize,
        value: ValueOrRef<'a>,
    ) -> Dictionary<'a> {
        Self {
            keys: DictionaryKeyArray::SingleValue {
                data_type: K::DATA_TYPE,
                length: key_count,
                value_index: Some(0),
            },
            values: vec![value].into(),
        }
    }

    pub fn new_null_with_data_type(
        count: usize,
        data_type: DataType,
    ) -> Result<Dictionary<'a>, DictionaryError> {
        Ok(match data_type {
            DataType::Int8 => Self::new_null::<Int8Type>(count),
            DataType::Int16 => Self::new_null::<Int16Type>(count),
            DataType::Int32 => Self::new_null::<Int32Type>(count),
            DataType::Int64 => Self::new_null::<Int64Type>(count),

            DataType::UInt8 => Self::new_null::<UInt8Type>(count),
            DataType::UInt16 => Self::new_null::<UInt16Type>(count),
            DataType::UInt32 => Self::new_null::<UInt32Type>(count),
            DataType::UInt64 => Self::new_null::<UInt64Type>(count),

            data_type => return Err(DictionaryError::UnsupportedKeyDataType { data_type }),
        })
    }

    pub fn new_null<K: ArrowDictionaryKeyType>(count: usize) -> Dictionary<'a> {
        Self {
            keys: DictionaryKeyArray::SingleValue {
                data_type: K::DATA_TYPE,
                length: count,
                value_index: None,
            },
            values: vec![].into(),
        }
    }

    pub fn values(&self) -> &DictionaryValueArray<'a> {
        &self.values
    }

    pub fn into_parts(self) -> (DictionaryKeyArray, DictionaryValueArray<'a>) {
        (self.keys, self.values)
    }

    pub fn get_value(&self, key_index: usize) -> Result<ValueOrRef<'a>, DictionaryValueError> {
        if let Some(value_index) = self.get_value_index(key_index) {
            return self.values.get_value_at(value_index);
        }

        Ok(ValueOrRef::Null)
    }
}

macro_rules! impl_from_dictionary_array {
    ($arrow_ty:ty, $variant:ident) => {
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

#[derive(Error, Debug, PartialEq)]
pub enum DictionaryError {
    #[error("Key index '{key_index}' refers to an invalid value")]
    InvalidKey { key_index: usize },

    #[error("Boolean dictionary should have two defined values")]
    InvalidBooleanValues,

    #[error("Data type '{data_type}' is not supported for keys")]
    UnsupportedKeyDataType { data_type: DataType },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Dictionary keys include both null keys and keys that point to a null dictionary value.
    /// Guarantees: Row validity combines key and value validity, and value lookup returns Null for both cases.
    #[test]
    fn dictionary_combines_key_and_value_nulls() {
        let dictionary = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(1), None, Some(0)])),
            DictionaryValueArray::from(vec![ValueOrRef::Integer(10), ValueOrRef::Null]),
        )
        .expect("valid");

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
        )
        .expect("valid");

        let nulls = dictionary.nulls().unwrap();
        assert_eq!(nulls.len(), 3);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_null(1));
        assert!(nulls.is_null(2));
    }

    /// Scenario: Valid, oversized, negative, and null keys reference a value table with no intrinsic nulls.
    /// Guarantees: Invalid references resolve to Null but don't contribute to null row validity
    #[test]
    fn dictionary_handles_invalid_references_without_value_nulls() {
        assert!(
            Dictionary::new(
                DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(1), None])),
                DictionaryValueArray::from(vec![ValueOrRef::Integer(10)]),
            )
            .is_err()
        );

        assert!(
            Dictionary::new(
                DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(-1)])),
                DictionaryValueArray::from(vec![ValueOrRef::Integer(10)]),
            )
            .is_err()
        );

        let dictionary = unsafe {
            Dictionary::new_unvalidated(
                DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(1), Some(-1), None])),
                DictionaryValueArray::from(vec![ValueOrRef::Integer(10)]),
            )
        };

        let nulls = dictionary.nulls().unwrap();
        assert_eq!(nulls.null_count(), 1);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_valid(1));
        assert!(nulls.is_valid(2));
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
        assert!(
            Dictionary::new(
                DictionaryKeyArray::from(Int8Array::from(vec![0, 1, 2])),
                DictionaryValueArray::from(vec![ValueOrRef::Integer(10), ValueOrRef::Null]),
            )
            .is_err()
        );

        let dictionary = unsafe {
            Dictionary::new_unvalidated(
                DictionaryKeyArray::from(Int8Array::from(vec![0, 1, 2])),
                DictionaryValueArray::from(vec![ValueOrRef::Integer(10), ValueOrRef::Null]),
            )
        };

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
        )
        .expect("valid")
        .nulls()
        .expect("valid");
        assert_eq!(repeated_null.null_count(), 2);

        let key_and_value_null = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![Some(0), None])),
            DictionaryValueArray::from(vec![ValueOrRef::Null, ValueOrRef::Integer(10)]),
        )
        .expect("valid")
        .nulls()
        .expect("valid");
        assert_eq!(key_and_value_null.null_count(), 2);

        let includes_non_null = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(1)])),
            DictionaryValueArray::from(vec![ValueOrRef::Null, ValueOrRef::Integer(10)]),
        )
        .expect("valid")
        .nulls()
        .expect("valid");
        assert_eq!(includes_non_null.null_count(), 1);
    }

    /// Scenario: Scalar and null dictionaries are synthesized for a requested Arrow key type.
    /// Guarantees: Every row resolves to the scalar or Null and retains the requested key data type.
    #[test]
    fn scalar_and_null_dictionary_constructors_preserve_shape() {
        let scalar =
            Dictionary::new_scalar_with_data_type(DataType::UInt16, 3, ValueOrRef::Integer(42))
                .expect("valid");
        assert_eq!(scalar.len(), 3);
        assert_eq!(scalar.keys().data_type(), DataType::UInt16);
        assert!(scalar.nulls().is_none());
        assert_eq!(scalar.get_value(2), Ok(ValueOrRef::Integer(42)));
        assert_eq!(scalar.get_value(3), Ok(ValueOrRef::Null));

        let null = Dictionary::new_null_with_data_type(3, DataType::Int32).expect("valid");
        assert_eq!(null.len(), 3);
        assert_eq!(null.keys().data_type(), DataType::Int32);
        assert_eq!(null.nulls().unwrap().null_count(), 3);
        assert_eq!(null.get_value(0), Ok(ValueOrRef::Null));
    }

    /// Scenario: A nullable Boolean array is encoded as a dictionary using a generic Arrow key type.
    /// Guarantees: Boolean values, key nulls, bounds, and the requested key data type are preserved.
    #[test]
    fn boolean_dictionary_constructor_preserves_values_and_shape() {
        let dictionary = Dictionary::new_boolean::<UInt16Type>(BooleanArray::from(vec![
            Some(false),
            Some(true),
            None,
        ]));

        assert_eq!(dictionary.len(), 3);
        assert_eq!(dictionary.keys().data_type(), DataType::UInt16);

        let nulls = dictionary.nulls().expect("nullable Boolean keys");
        assert!(nulls.is_valid(0));
        assert!(nulls.is_valid(1));
        assert!(nulls.is_null(2));

        assert_eq!(dictionary.get_value(0), Ok(ValueOrRef::Boolean(false)));
        assert_eq!(dictionary.get_value(1), Ok(ValueOrRef::Boolean(true)));
        assert_eq!(dictionary.get_value(2), Ok(ValueOrRef::Null));
        assert_eq!(dictionary.get_value(3), Ok(ValueOrRef::Null));
    }

    /// Scenario: Boolean dictionaries are constructed from every supported runtime key data type and one unsupported type.
    /// Guarantees: Supported types are retained and unsupported key types return a typed construction error.
    #[test]
    fn boolean_dictionary_runtime_key_type_is_validated() {
        for data_type in [
            DataType::Int8,
            DataType::Int16,
            DataType::Int32,
            DataType::Int64,
            DataType::UInt8,
            DataType::UInt16,
            DataType::UInt32,
            DataType::UInt64,
        ] {
            let dictionary = Dictionary::new_boolean_with_data_type(
                data_type.clone(),
                BooleanArray::from(vec![false, true]),
            )
            .expect("supported key type");

            assert_eq!(dictionary.keys().data_type(), data_type);
            assert_eq!(dictionary.get_value(0), Ok(ValueOrRef::Boolean(false)));
            assert_eq!(dictionary.get_value(1), Ok(ValueOrRef::Boolean(true)));
        }

        let error = Dictionary::new_boolean_with_data_type(
            DataType::Boolean,
            BooleanArray::from(vec![true]),
        )
        .expect_err("Boolean is not a dictionary key type");
        assert_eq!(
            error,
            DictionaryError::UnsupportedKeyDataType {
                data_type: DataType::Boolean,
            }
        );
    }

    /// Scenario: A primitive Arrow array is represented as a dictionary with one unique key per row.
    /// Guarantees: Row indexes map directly to source values and the first boundary index is rejected.
    #[test]
    fn primitive_array_dictionary_uses_unique_keys() {
        let values = Int32Array::from(vec![5, 8, 13]);
        let dictionary = Dictionary::new_unique_values::<UInt8Type>(values.into());

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
        )
        .expect("valid");

        assert_eq!(
            dictionary.get_value(0),
            Err(DictionaryValueError::IntegerConversionFailure {
                original_value: u64::MAX,
            })
        );
    }
}
