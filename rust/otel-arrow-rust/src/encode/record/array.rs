// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module provides adaptive array builders for Arrow arrays.
//!
//! Often for OTel-Arrow, we have columns that are optional on the schema. For example, the Boolean
//! column may not be present in a record batch representing a list of attributes only be present
//! if there are some boolean type values in the list.
//!
//! There are also cases where we want to dynamically use dictionary encoding with the smallest index
//! the cardinality of the allows.
//!
//! This module contains adaptive array builders that can dynamically create either no array (for
//! an all-null) column, an array that may be a dictionary, of an array or native types. It will
//! handle converting between different builders dynamically  based on the data which is appended.

use arrow::array::{ArrayRef, StringBuilder, StringDictionaryBuilder};
use arrow::datatypes::{DataType, UInt8Type, UInt16Type};

use crate::arrays::NullableArrayAccessor;

use dictionary::{
    AdaptiveDictionaryBuilder, ConvertToNativeHelper, DictionaryArrayBuilder,
    DictionaryBuilderError, DictionaryOptions, UpdateDictionaryIndexInto,
};

pub mod dictionary;
pub mod string;

/// This is the base trait that array builders should implement.
pub trait ArrayBuilder {
    type Native;

    fn append_value(&mut self, value: &Self::Native);

    fn finish(&mut self) -> ArrayWithType;
}

pub struct ArrayWithType {
    pub array: ArrayRef,
    pub data_type: DataType,
}

/// This is a helper trait that allows the adaptive builders to construct new
/// instances of the builder dynamically
pub trait ArrayBuilderConstructor {
    fn new() -> Self;

    // TODO, at some point we may consider optionally adding a
    // with_capacity function here that could be used to create
    // a builder with pre-allocated buffers
}

/// This enum is a container that abstracts array builder which is either
/// dictionary or native. It converts from the dictionary builder to the
/// native builder when the dictionary builder overflows.
enum MaybeDictionaryBuilder<
    NativeBuilder: ArrayBuilder + ArrayBuilderConstructor,
    DictBuilderU8: DictionaryArrayBuilder<UInt8Type> + ArrayBuilderConstructor,
    DictBuilderU16: DictionaryArrayBuilder<UInt16Type> + ArrayBuilderConstructor,
> {
    Native(NativeBuilder),
    Dictionary(AdaptiveDictionaryBuilder<DictBuilderU8, DictBuilderU16>),
}

impl<T, TN, TD8, TD16> ArrayBuilder for MaybeDictionaryBuilder<TN, TD8, TD16>
where
    TN: ArrayBuilder<Native = T> + ArrayBuilderConstructor,
    TD8: DictionaryArrayBuilder<UInt8Type, Native = T>
        + ArrayBuilderConstructor
        + ConvertToNativeHelper,
    <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    TD16: DictionaryArrayBuilder<UInt16Type, Native = T>
        + ArrayBuilderConstructor
        + ConvertToNativeHelper,
    <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    TD8: UpdateDictionaryIndexInto<TD16>,
{
    type Native = T;

    fn append_value(
        &mut self,
        value: &<MaybeDictionaryBuilder<TN, TD8, TD16> as ArrayBuilder>::Native,
    ) {
        match self {
            Self::Native(array_builder) => array_builder.append_value(value),
            Self::Dictionary(dict_array_builder) => match dict_array_builder.append_value(value) {
                // we've overflowed the dictionary, so we must convert to the native builder type
                Err(DictionaryBuilderError::DictOverflow {}) => {
                    let mut native = TN::new();
                    dict_array_builder.to_native(&mut native);
                    native.append_value(value);
                    *self = Self::Native(native);
                }
                _ => {
                    // do nothing here, as the append was successful
                }
            },
        }
    }

    fn finish(&mut self) -> ArrayWithType {
        match self {
            Self::Dictionary(dict_array_builder) => dict_array_builder.finish(),
            Self::Native(array_builder) => array_builder.finish(),
        }
    }
}

#[derive(Default)]
pub struct ArrayOptions {
    pub dictionary_options: Option<DictionaryOptions>,
    pub nullable: bool,
}

pub struct AdaptiveArrayBuilder<
    TN: ArrayBuilder + ArrayBuilderConstructor,
    TD8: DictionaryArrayBuilder<UInt8Type> + ArrayBuilderConstructor,
    TD16: DictionaryArrayBuilder<UInt16Type> + ArrayBuilderConstructor,
> {
    dictionary_options: Option<DictionaryOptions>,
    inner: Option<MaybeDictionaryBuilder<TN, TD8, TD16>>,
}

impl<T, TN, TD8, TD16> AdaptiveArrayBuilder<TN, TD8, TD16>
where
    TN: ArrayBuilder<Native = T> + ArrayBuilderConstructor,
    TD8: DictionaryArrayBuilder<UInt8Type, Native = T>
        + ArrayBuilderConstructor
        + ConvertToNativeHelper,
    <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    TD16: DictionaryArrayBuilder<UInt16Type, Native = T>
        + ArrayBuilderConstructor
        + ConvertToNativeHelper,
    <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    TD8: UpdateDictionaryIndexInto<TD16>,
{
    pub fn new(options: ArrayOptions) -> Self {
        let inner = if options.nullable {
            None
        } else {
            Some(Self::initial_builder(&options.dictionary_options))
        };

        Self {
            dictionary_options: options.dictionary_options,
            inner,
        }
    }

    // Initializes the builder, which may either be a builder for the, if dictionary
    // options is `Some`, otherwise it will construct the native builder builder variant
    fn initial_builder(
        dictionary_options: &Option<DictionaryOptions>,
    ) -> MaybeDictionaryBuilder<TN, TD8, TD16> {
        match dictionary_options.as_ref() {
            Some(dictionary_options) => MaybeDictionaryBuilder::Dictionary(
                AdaptiveDictionaryBuilder::new(dictionary_options),
            ),
            None => MaybeDictionaryBuilder::Native(TN::new()),
        }
    }

    fn append_value(&mut self, value: &T) {
        if self.inner.is_none() {
            // TODO -- when we handle nulls here we need to keep track of how many
            // nulls have been appended before the first value, and prefix this
            // newly initialized array with that number of nulls
            // https://github.com/open-telemetry/otel-arrow/issues/534
            self.inner = Some(Self::initial_builder(&self.dictionary_options));
        }

        let inner = self
            .inner
            .as_mut()
            .expect("inner should now be initialized");
        inner.append_value(value)
    }

    fn finish(&mut self) -> Option<ArrayWithType> {
        self.inner.as_mut().map(|builder| builder.finish())
    }
}

pub type StringArrayBuilder = AdaptiveArrayBuilder<
    StringBuilder,
    StringDictionaryBuilder<UInt8Type>,
    StringDictionaryBuilder<UInt16Type>,
>;

#[cfg(test)]
pub mod test {
    use super::*;

    use std::sync::Arc;

    use arrow::array::{StringArray, UInt8Array, UInt8DictionaryArray};
    use arrow::datatypes::DataType;

    #[test]
    fn test_array_builder() {
        let mut builder = StringArrayBuilder::new(ArrayOptions {
            nullable: true,
            dictionary_options: Some(DictionaryOptions {
                min_cardinality: 4,
                max_cardinality: 4,
            }),
        });

        // expect that for empty array, we get a None value because the builder is nullable
        let result = builder.finish();
        assert!(result.is_none());

        // expect that when we add values, we get a dictionary
        builder.append_value(&"a".to_string());
        builder.append_value(&"a".to_string());
        builder.append_value(&"b".to_string());

        let result = builder.finish().unwrap();
        assert_eq!(
            result.data_type,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        );

        let mut expected_dict_values = StringBuilder::new();
        expected_dict_values.append_value("a");
        expected_dict_values.append_value("b");
        let expected_dict_keys = UInt8Array::from_iter_values(vec![0, 0, 1]);
        let expected =
            UInt8DictionaryArray::new(expected_dict_keys, Arc::new(expected_dict_values.finish()));

        assert_eq!(
            result
                .array
                .as_any()
                .downcast_ref::<UInt8DictionaryArray>()
                .unwrap(),
            &expected
        );
    }

    #[test]
    fn test_array_builder_non_nullable_empty() {
        let mut builder = StringArrayBuilder::new(ArrayOptions {
            nullable: false,
            dictionary_options: Some(DictionaryOptions {
                min_cardinality: 4,
                max_cardinality: 4,
            }),
        });

        // check that since the type we're building is not nullable, we get an empty array
        let result = builder.finish().unwrap();
        assert_eq!(
            result.data_type,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        );
        assert_eq!(result.array.len(), 0);
    }

    #[test]
    fn test_array_builder_dict_overflow() {
        let mut builder = StringArrayBuilder::new(ArrayOptions {
            nullable: false,
            dictionary_options: Some(DictionaryOptions {
                min_cardinality: 4,
                max_cardinality: 4,
            }),
        });

        // expect that when we add values, we get a dictionary
        builder.append_value(&"a".to_string());
        builder.append_value(&"b".to_string());
        builder.append_value(&"c".to_string());
        builder.append_value(&"d".to_string());
        builder.append_value(&"e".to_string());

        let result = builder.finish().unwrap();
        assert_eq!(result.data_type, DataType::Utf8);

        let mut expected_values = StringBuilder::new();
        expected_values.append_value("a");
        expected_values.append_value("b");
        expected_values.append_value("c");
        expected_values.append_value("d");
        expected_values.append_value("e");

        assert_eq!(
            result.array.as_any().downcast_ref::<StringArray>().unwrap(),
            &expected_values.finish()
        )
    }
}
