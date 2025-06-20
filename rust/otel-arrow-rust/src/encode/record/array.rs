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

use arrow::array::{
    ArrayRef, BinaryBuilder, BinaryDictionaryBuilder, FixedSizeBinaryBuilder,
    FixedSizeBinaryDictionaryBuilder, PrimitiveBuilder, PrimitiveDictionaryBuilder, StringBuilder,
    StringDictionaryBuilder,
};
use arrow::datatypes::{
    DurationNanosecondType, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type,
    TimestampNanosecondType, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow::error::ArrowError;

use crate::arrays::NullableArrayAccessor;
use crate::encode::record::array::dictionary::DictionaryBuilder;

use dictionary::{
    AdaptiveDictionaryBuilder, CheckedDictionaryArrayAppend, ConvertToNativeHelper,
    DictionaryArrayAppend, DictionaryBuilderError, DictionaryOptions, UpdateDictionaryIndexInto,
    checked,
};

pub mod binary;
pub mod boolean;
pub mod dictionary;
pub mod fixed_size_binary;
pub mod primitive;
pub mod string;
pub mod structs;

/// This is the base trait that array builders should implement to build the array.
///
/// Generally this will be used for arrow array builders that implement this method
/// by wrapping the result of the finish method in an Arc to produce an array ref.
pub trait ArrayBuilder {
    fn finish(&mut self) -> ArrayRef;
}

/// This is a helper trait that allows the adaptive builders to construct new
/// instances of the builder dynamically
pub trait ArrayBuilderConstructor {
    type Args;

    fn new(args: Self::Args) -> Self;

    // TODO, at some point we may consider optionally adding a
    // with_capacity function here that could be used to create
    // a builder with pre-allocated buffers
}

/// this trait implementation called by adaptive array builders on the base array builders to
/// append values to the underlying builder.
///
/// If the underlying builder can return an error from append (e.g. if some values for the
/// Native are not valid), then `CheckedArrayAppend` should be implemented instead.
pub trait ArrayAppend: ArrayAppendNulls {
    type Native;

    fn append_value(&mut self, value: &Self::Native);
}

/// this trait implementation called by adaptive array builders on the base array builders to
/// append values to the underlying builder in cases where the underlying append call can fail.
///
/// Some underlying builders may have `append_` methods that return results. For example,
/// FixedSizeBinary's builders can return an error if a byte array of the wrong length is passed.
///
/// In this case, underlying builders should implement this trait and callers can use it when
/// there's uncertainty that the value they've passed is valid.
pub trait CheckedArrayAppend: ArrayAppendNulls {
    type Native;

    fn append_value(&mut self, value: &Self::Native) -> Result<(), ArrowError>;
}

pub trait ArrayAppendNulls {
    /// Append a null value to the builder
    fn append_null(&mut self);

    /// Append `n` nulls to the builder
    fn append_nulls(&mut self, n: usize);
}

/// This enum is a container that abstracts array builder which is either
/// dictionary or native. It converts from the dictionary builder to the
/// native builder when the dictionary builder overflows.
enum MaybeDictionaryBuilder<NativeBuilder, DictBuilderU8, DictBuilderU16> {
    Native(NativeBuilder),
    Dictionary(AdaptiveDictionaryBuilder<DictBuilderU8, DictBuilderU16>),
}

impl<TN, TD8, TD16> ArrayBuilder for MaybeDictionaryBuilder<TN, TD8, TD16>
where
    TN: ArrayBuilder,
    TD8: DictionaryBuilder<UInt8Type>,
    TD16: DictionaryBuilder<UInt16Type>,
{
    fn finish(&mut self) -> ArrayRef {
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

pub struct AdaptiveArrayBuilder<TArgs, TN, TD8, TD16> {
    dictionary_options: Option<DictionaryOptions>,
    nullable: bool,
    inner: Option<MaybeDictionaryBuilder<TN, TD8, TD16>>,

    // the number of nulls that have been appended to the builder before the first value. This is
    // used as a counter until the underlying builder possibly gets initialized, then we prepend
    // this many nulls
    nulls_prefix: usize,

    // these are the args that will be used to create the underlying builder. In most cases this
    // will be NoArgs, but there are some cases where Array builder's constructors require args,
    // for example `FixedSizeBinary` requires the byte_width
    inner_args: TArgs,
}

impl<TN, TD8, TD16> AdaptiveArrayBuilder<NoArgs, TN, TD8, TD16>
where
    TN: ArrayBuilderConstructor<Args = NoArgs> + ArrayAppendNulls,
    TD8: ArrayBuilderConstructor<Args = NoArgs> + ArrayAppendNulls,
    TD16: ArrayBuilderConstructor<Args = NoArgs> + ArrayAppendNulls,
{
    pub fn new(options: ArrayOptions) -> Self {
        Self::new_with_args(options, ())
    }
}

impl<TArgs, TN, TD8, TD16> AdaptiveArrayBuilder<TArgs, TN, TD8, TD16>
where
    TArgs: Clone,
    TN: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls,
    TD8: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls,
    TD16: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls,
{
    pub fn new_with_args(options: ArrayOptions, args: TArgs) -> Self {
        let inner = if options.nullable {
            None
        } else {
            Some(Self::initial_builder(
                args.clone(),
                &options.dictionary_options,
            ))
        };

        Self {
            dictionary_options: options.dictionary_options,
            nullable: options.nullable,
            nulls_prefix: 0,
            inner,
            inner_args: args,
        }
    }

    // Creates the initial the builder, which may either be a builder for the dict, if dictionary
    // options is `Some`, otherwise it will construct the native builder variant
    fn initial_builder(
        args: TArgs,
        dictionary_options: &Option<DictionaryOptions>,
    ) -> MaybeDictionaryBuilder<TN, TD8, TD16> {
        match dictionary_options.as_ref() {
            Some(dictionary_options) => MaybeDictionaryBuilder::Dictionary(
                AdaptiveDictionaryBuilder::new(dictionary_options, args),
            ),
            None => MaybeDictionaryBuilder::Native(TN::new(args)),
        }
    }

    // initialize the inner builder if it is not already initialized.
    fn initialize_inner(&mut self) {
        if self.inner.is_none() {
            // TODO -- when we handle nulls here we need to keep track of how many
            // nulls have been appended before the first value, and prefix this
            // newly initialized array with that number of nulls
            // https://github.com/open-telemetry/otel-arrow/issues/534
            self.inner = Some(Self::initial_builder(
                self.inner_args.clone(),
                &self.dictionary_options,
            ));
            if self.nulls_prefix > 0 {
                self.append_nulls(self.nulls_prefix);
            }
        }
    }
}

impl<TArgs, TN, TD8, TD16> ArrayAppendNulls for AdaptiveArrayBuilder<TArgs, TN, TD8, TD16>
where
    TN: ArrayAppendNulls,
    TD8: ArrayAppendNulls,
    TD16: ArrayAppendNulls,
{
    fn append_null(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            match inner {
                MaybeDictionaryBuilder::Dictionary(builder) => builder.append_null(),
                MaybeDictionaryBuilder::Native(builder) => builder.append_null(),
            }
        } else {
            self.nulls_prefix += 1;
        }
    }

    fn append_nulls(&mut self, n: usize) {
        if let Some(inner) = self.inner.as_mut() {
            match inner {
                MaybeDictionaryBuilder::Dictionary(builder) => builder.append_nulls(n),
                MaybeDictionaryBuilder::Native(builder) => builder.append_nulls(n),
            }
        } else {
            self.nulls_prefix += n;
        }
    }
}

impl<T, TArgs, TN, TD8, TD16> ArrayAppend for AdaptiveArrayBuilder<TArgs, TN, TD8, TD16>
where
    TArgs: Clone,
    TN: ArrayAppend<Native = T> + ArrayAppendNulls + ArrayBuilderConstructor<Args = TArgs>,
    TD8: DictionaryArrayAppend<Native = T>
        + DictionaryBuilder<UInt8Type>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper
        + UpdateDictionaryIndexInto<TD16>,
    <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    TD16: DictionaryArrayAppend<Native = T>
        + ArrayAppendNulls
        + DictionaryBuilder<UInt16Type>
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper,
    <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
{
    type Native = T;

    /// Append a value to the underlying builder
    fn append_value(&mut self, value: &T) {
        self.initialize_inner();
        let inner = self
            .inner
            .as_mut()
            .expect("inner should now be initialized");
        match inner {
            MaybeDictionaryBuilder::Native(native_builder) => {
                native_builder.append_value(value);
            }
            MaybeDictionaryBuilder::Dictionary(dictionary_builder) => {
                match dictionary_builder.append_value(value) {
                    Ok(_) => {}
                    Err(DictionaryBuilderError::DictOverflow {}) => {
                        let mut native = TN::new(self.inner_args.clone());
                        dictionary_builder.to_native(&mut native);
                        self.inner = Some(MaybeDictionaryBuilder::Native(native));
                        self.append_value(value);
                    }
                }
            }
        }
    }
}

impl<T, TArgs, TN, TD8, TD16> CheckedArrayAppend for AdaptiveArrayBuilder<TArgs, TN, TD8, TD16>
where
    TArgs: Clone,
    TN: CheckedArrayAppend<Native = T> + ArrayAppendNulls + ArrayBuilderConstructor<Args = TArgs>,
    TD8: CheckedDictionaryArrayAppend<Native = T>
        + DictionaryBuilder<UInt8Type>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper
        + UpdateDictionaryIndexInto<TD16>,
    <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    TD16: CheckedDictionaryArrayAppend<Native = T>
        + DictionaryBuilder<UInt16Type>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper,
    <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
{
    type Native = T;

    /// Try to append a value to the underlying builder. This method may return an error if
    /// the value is not valid.
    fn append_value(&mut self, value: &T) -> Result<(), ArrowError> {
        self.initialize_inner();
        let inner = self
            .inner
            .as_mut()
            .expect("inner should now be initialized");
        match inner {
            MaybeDictionaryBuilder::Native(native_builder) => native_builder.append_value(value),
            MaybeDictionaryBuilder::Dictionary(dictionary_builder) => {
                match dictionary_builder.append_value_checked(value) {
                    Ok(_) => {
                        // append succeeded
                        Ok(())
                    }

                    Err(checked::DictionaryBuilderError::DictOverflow {}) => {
                        let mut native = TN::new(self.inner_args.clone());
                        dictionary_builder.to_native_checked(&mut native)?;
                        self.inner = Some(MaybeDictionaryBuilder::Native(native));
                        self.append_value(value)
                    }
                    Err(checked::DictionaryBuilderError::CheckedBuilderError {
                        source: arrow_error,
                    }) => Err(arrow_error),
                }
            }
        }
    }
}

impl<TArgs, TN, TD8, TD16> AdaptiveArrayBuilder<TArgs, TN, TD8, TD16>
where
    TN: ArrayBuilder,
    TD8: DictionaryBuilder<UInt8Type>,
    TD16: DictionaryBuilder<UInt16Type>,
{
    fn finish(&mut self) -> Option<ArrayRef> {
        self.inner.as_mut().map(|builder| builder.finish())
    }
}

// Arg type for an array constructor that takes no arguments.
pub(crate) type NoArgs = ();

pub type StringArrayBuilder = AdaptiveArrayBuilder<
    NoArgs,
    StringBuilder,
    StringDictionaryBuilder<UInt8Type>,
    StringDictionaryBuilder<UInt16Type>,
>;

pub type BinaryArrayBuilder = AdaptiveArrayBuilder<
    NoArgs,
    BinaryBuilder,
    BinaryDictionaryBuilder<UInt8Type>,
    BinaryDictionaryBuilder<UInt16Type>,
>;

pub type FixedSizeBinaryArrayBuilder = AdaptiveArrayBuilder<
    i32,
    FixedSizeBinaryBuilder,
    FixedSizeBinaryDictionaryBuilder<UInt8Type>,
    FixedSizeBinaryDictionaryBuilder<UInt16Type>,
>;

pub type PrimitiveArrayBuilder<T> = AdaptiveArrayBuilder<
    NoArgs,
    PrimitiveBuilder<T>,
    PrimitiveDictionaryBuilder<UInt8Type, T>,
    PrimitiveDictionaryBuilder<UInt16Type, T>,
>;

// aliases for adaptive primitive array builders
pub type Float32ArrayBuilder = PrimitiveArrayBuilder<Float32Type>;
pub type Float64ArrayBuilder = PrimitiveArrayBuilder<Float64Type>;
pub type UInt8ArrayBuilder = PrimitiveArrayBuilder<UInt8Type>;
pub type UInt16ArrayBuilder = PrimitiveArrayBuilder<UInt16Type>;
pub type UInt32ArrayBuilder = PrimitiveArrayBuilder<UInt32Type>;
pub type UInt64ArrayBuilder = PrimitiveArrayBuilder<UInt64Type>;
pub type Int8ArrayBuilder = PrimitiveArrayBuilder<Int8Type>;
pub type Int16ArrayBuilder = PrimitiveArrayBuilder<Int16Type>;
pub type Int32ArrayBuilder = PrimitiveArrayBuilder<Int32Type>;
pub type Int64ArrayBuilder = PrimitiveArrayBuilder<Int64Type>;
pub type TimestampNanosecondArrayBuilder = PrimitiveArrayBuilder<TimestampNanosecondType>;
pub type DurationNanosecondArrayBuilder = PrimitiveArrayBuilder<DurationNanosecondType>;

#[cfg(test)]
pub mod test {
    use super::*;

    use arrow::array::{DictionaryArray, UInt8Array};
    use arrow::datatypes::{DataType, TimeUnit};

    fn test_array_builder_generic<T, TArgs, TN, TD8, TD16>(
        array_builder_factory: &impl Fn(ArrayOptions) -> AdaptiveArrayBuilder<TArgs, TN, TD8, TD16>,
        expected_data_type: DataType,
    ) where
        T: PartialEq + std::fmt::Debug,
        TN: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls + ArrayBuilder,
        TD8: DictionaryBuilder<UInt8Type>
            + ArrayAppendNulls
            + ArrayBuilderConstructor<Args = TArgs>
            + ConvertToNativeHelper
            + UpdateDictionaryIndexInto<TD16>,
        <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T>,
        TD16: DictionaryBuilder<UInt16Type>
            + ArrayAppendNulls
            + ArrayBuilderConstructor<Args = TArgs>
            + ConvertToNativeHelper,
        <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T>,
    {
        // tests some common behaviours of checked & unchecked array builders:

        // expect that for empty array, we get a None value because the builder is nullable
        let mut builder = array_builder_factory(ArrayOptions {
            nullable: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
        });
        let result = builder.finish();
        assert!(result.is_none());

        // expect that if it is non-nullable, we always get an empty array instead of 'None'
        let mut builder = array_builder_factory(ArrayOptions {
            nullable: false,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
        });
        let result = builder.finish().unwrap();
        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(
                Box::new(DataType::UInt8),
                Box::new(expected_data_type.clone())
            )
        );
        assert_eq!(result.len(), 0);

        // expect that for an all null array, we get None if the array is marked as nullable
        let mut builder = array_builder_factory(ArrayOptions {
            nullable: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
        });
        builder.append_null();
        builder.append_nulls(2);
        let result = builder.finish();
        assert!(result.is_none());
    }

    fn test_array_append_generic<T, TN, TD8, TD16>(
        array_builder_factory: impl Fn(ArrayOptions) -> AdaptiveArrayBuilder<NoArgs, TN, TD8, TD16>,
        values: Vec<T>,
        expected_data_type: DataType,
    ) where
        T: PartialEq + std::fmt::Debug,
        TN: ArrayAppend<Native = T>
            + ArrayAppendNulls
            + ArrayBuilderConstructor<Args = NoArgs>
            + ArrayBuilder,
        TD8: DictionaryArrayAppend<Native = T>
            + DictionaryBuilder<UInt8Type>
            + ArrayAppendNulls
            + ArrayBuilderConstructor<Args = NoArgs>
            + ConvertToNativeHelper
            + UpdateDictionaryIndexInto<TD16>,
        <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
        TD16: DictionaryArrayAppend<Native = T>
            + DictionaryBuilder<UInt16Type>
            + ArrayAppendNulls
            + ArrayBuilderConstructor<Args = NoArgs>
            + ConvertToNativeHelper,
        <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    {
        test_array_builder_generic(&array_builder_factory, expected_data_type.clone());

        let mut builder = array_builder_factory(ArrayOptions {
            nullable: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
        });

        // expect that for empty array, we get a None value because the builder is nullable
        let result = builder.finish();
        assert!(result.is_none());

        // expect that when we add values, we get a dictionary
        builder.append_value(&values[0]);
        builder.append_value(&values[0]);
        builder.append_value(&values[1]);
        builder.append_null();
        builder.append_value(&values[0]);
        builder.append_nulls(2);
        builder.append_value(&values[1]);

        let result = builder.finish().unwrap();
        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(
                Box::new(DataType::UInt8),
                Box::new(expected_data_type.clone())
            )
        );
        assert_eq!(result.len(), 8);

        let dict_array = result
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        let dict_keys = dict_array.keys();
        assert_eq!(
            dict_keys,
            &UInt8Array::from_iter(vec![
                Some(0),
                Some(0),
                Some(1),
                None,
                Some(0),
                None,
                None,
                Some(1)
            ])
        );
        let dict_values = dict_array
            .values()
            .as_any()
            .downcast_ref::<<TD8 as ConvertToNativeHelper>::Accessor>()
            .unwrap();
        assert_eq!(dict_values.value_at(0).unwrap(), values[0]);
        assert_eq!(dict_values.value_at(1).unwrap(), values[1]);

        // expect that if dictionary options is 'None', we just get the native array
        let mut builder = array_builder_factory(ArrayOptions {
            dictionary_options: None,
            nullable: false,
        });
        builder.append_value(&values[0]);
        builder.append_value(&values[1]);
        builder.append_null();
        builder.append_value(&values[1]);
        builder.append_nulls(2);
        builder.append_value(&values[1]);
        let result = builder.finish().unwrap();
        assert_eq!(result.len(), 7);
        let array = result
            .as_any()
            .downcast_ref::<<TD8 as ConvertToNativeHelper>::Accessor>()
            .unwrap();
        assert_eq!(array.value_at(0).unwrap(), values[0]);
        assert_eq!(array.value_at(1).unwrap(), values[1]);
        assert!(array.value_at(2).is_none());
        assert_eq!(array.value_at(3).unwrap(), values[1]);
        assert!(array.value_at(4).is_none());
        assert!(array.value_at(5).is_none());
        assert_eq!(array.value_at(6).unwrap(), values[1]);

        // expect that when dictionary overflow happens, we get the native builder
        let mut builder = array_builder_factory(ArrayOptions {
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 1,
                min_cardinality: 1,
            }),
            nullable: false,
        });
        builder.append_value(&values[0]);
        builder.append_null();
        builder.append_nulls(2);
        builder.append_value(&values[1]);
        let result = builder.finish().unwrap();
        assert_eq!(result.len(), 5);
        let array = result
            .as_any()
            .downcast_ref::<<TD8 as ConvertToNativeHelper>::Accessor>()
            .unwrap();
        assert_eq!(array.value_at(0).unwrap(), values[0]);
        assert!(array.value_at(1).is_none());
        assert!(array.value_at(2).is_none());
        assert!(array.value_at(3).is_none());
        assert_eq!(array.value_at(4).unwrap(), values[1]);

        // check that for nullable arrays we properly prepend nulls
        let mut builder = array_builder_factory(ArrayOptions {
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 5,
                min_cardinality: 5,
            }),
            nullable: true,
        });
        builder.append_null();
        builder.append_nulls(2);
        builder.append_value(&values[0]);
        let result = builder.finish().unwrap();
        let dict_array = result
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        let dict_keys = dict_array.keys();
        assert_eq!(
            dict_keys,
            &UInt8Array::from_iter(vec![None, None, None, Some(0),])
        );
    }

    #[test]
    fn test_array_builder() {
        test_array_append_generic(UInt8ArrayBuilder::new, vec![0, 1], DataType::UInt8);
        test_array_append_generic(UInt16ArrayBuilder::new, vec![0, 1], DataType::UInt16);
        test_array_append_generic(UInt32ArrayBuilder::new, vec![0, 1], DataType::UInt32);
        test_array_append_generic(UInt64ArrayBuilder::new, vec![0, 1], DataType::UInt64);
        test_array_append_generic(Int8ArrayBuilder::new, vec![0, 1], DataType::Int8);
        test_array_append_generic(Int16ArrayBuilder::new, vec![0, 1], DataType::Int16);
        test_array_append_generic(Int32ArrayBuilder::new, vec![0, 1], DataType::Int32);
        test_array_append_generic(Int64ArrayBuilder::new, vec![0, 1], DataType::Int64);
        test_array_append_generic(Float32ArrayBuilder::new, vec![0.0, 1.0], DataType::Float32);
        test_array_append_generic(Float64ArrayBuilder::new, vec![0.0, 1.1], DataType::Float64);
        test_array_append_generic(
            StringArrayBuilder::new,
            vec!["a".to_string(), "b".to_string()],
            DataType::Utf8,
        );
        test_array_append_generic(
            BinaryArrayBuilder::new,
            vec![b"a".to_vec(), b"b".to_vec()],
            DataType::Binary,
        );
        test_array_append_generic(
            TimestampNanosecondArrayBuilder::new,
            vec![0, 1],
            DataType::Timestamp(TimeUnit::Nanosecond, None),
        );
        test_array_append_generic(
            DurationNanosecondArrayBuilder::new,
            vec![0, 1],
            DataType::Duration(TimeUnit::Nanosecond),
        );
    }

    fn test_checked_array_builder_generic<T, TArgs, TN, TD8, TD16>(
        array_builder_factory: impl Fn(ArrayOptions) -> AdaptiveArrayBuilder<TArgs, TN, TD8, TD16>,
        values: Vec<T>,
        invalid_values: Vec<T>,
        expected_data_type: DataType,
    ) where
        T: PartialEq + std::fmt::Debug,
        TArgs: Clone,
        TN: CheckedArrayAppend<Native = T> + ArrayBuilderConstructor<Args = TArgs> + ArrayBuilder,
        TD8: CheckedDictionaryArrayAppend<Native = T>
            + DictionaryBuilder<UInt8Type>
            + ArrayBuilderConstructor<Args = TArgs>
            + ConvertToNativeHelper
            + UpdateDictionaryIndexInto<TD16>,
        <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
        TD16: CheckedDictionaryArrayAppend<Native = T>
            + DictionaryBuilder<UInt16Type>
            + ArrayBuilderConstructor<Args = TArgs>
            + ConvertToNativeHelper,
        <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    {
        test_array_builder_generic(&array_builder_factory, expected_data_type.clone());

        let mut builder = array_builder_factory(ArrayOptions {
            nullable: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
        });

        assert!(builder.append_value(&values[0]).is_ok());
        assert!(builder.append_value(&values[0]).is_ok());
        assert!(builder.append_value(&values[1]).is_ok());
        builder.append_null();
        assert!(builder.append_value(&values[0]).is_ok());
        builder.append_nulls(2);
        assert!(builder.append_value(&values[1]).is_ok());

        let result = builder.finish().unwrap();
        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(
                Box::new(DataType::UInt8),
                Box::new(expected_data_type.clone())
            )
        );
        assert_eq!(result.len(), 8);

        let dict_array = result
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        let dict_keys = dict_array.keys();
        assert_eq!(
            dict_keys,
            &UInt8Array::from_iter(vec![
                Some(0),
                Some(0),
                Some(1),
                None,
                Some(0),
                None,
                None,
                Some(1)
            ])
        );
        let dict_values = dict_array
            .values()
            .as_any()
            .downcast_ref::<<TD8 as ConvertToNativeHelper>::Accessor>()
            .unwrap();
        assert_eq!(dict_values.value_at(0).unwrap(), values[0]);
        assert_eq!(dict_values.value_at(1).unwrap(), values[1]);

        // expect that if dictionary options is 'None', we just get the native array
        let mut builder = array_builder_factory(ArrayOptions {
            dictionary_options: None,
            nullable: false,
        });
        assert!(builder.append_value(&values[0]).is_ok());
        assert!(builder.append_value(&values[1]).is_ok());
        builder.append_null();
        assert!(builder.append_value(&values[0]).is_ok());
        builder.append_nulls(2);
        assert!(builder.append_value(&values[0]).is_ok());
        let result = builder.finish().unwrap();
        assert_eq!(result.len(), 7);
        let array = result
            .as_any()
            .downcast_ref::<<TD8 as ConvertToNativeHelper>::Accessor>()
            .unwrap();
        assert_eq!(array.value_at(0).unwrap(), values[0]);
        assert_eq!(array.value_at(1).unwrap(), values[1]);
        assert!(array.value_at(2).is_none());
        assert_eq!(array.value_at(3).unwrap(), values[0]);
        assert!(array.value_at(4).is_none());
        assert!(array.value_at(5).is_none());
        assert_eq!(array.value_at(6).unwrap(), values[0]);

        // expect that when dictionary overflow happens, we get the native builder
        let mut builder = array_builder_factory(ArrayOptions {
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 1,
                min_cardinality: 1,
            }),
            nullable: false,
        });
        assert!(builder.append_value(&values[0]).is_ok());
        builder.append_null();
        builder.append_nulls(2);
        assert!(builder.append_value(&values[1]).is_ok());
        let result = builder.finish().unwrap();
        assert_eq!(result.len(), 5);
        let array = result
            .as_any()
            .downcast_ref::<<TD8 as ConvertToNativeHelper>::Accessor>()
            .unwrap();
        assert_eq!(array.value_at(0).unwrap(), values[0]);
        assert!(array.value_at(1).is_none());
        assert!(array.value_at(2).is_none());
        assert!(array.value_at(3).is_none());
        assert_eq!(array.value_at(4).unwrap(), values[1]);

        // expect that invalid values are rejected by the dictionary builder:
        let mut builder = array_builder_factory(ArrayOptions {
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 1,
                min_cardinality: 1,
            }),
            nullable: false,
        });
        let result = builder.append_value(&invalid_values[0]);
        let err = result.unwrap_err();
        assert!(matches!(err, ArrowError::InvalidArgumentError(_)))
    }

    #[test]
    fn test_checked_array_builder() {
        test_checked_array_builder_generic(
            |opts| FixedSizeBinaryArrayBuilder::new_with_args(opts, 1),
            vec![b"a".to_vec(), b"b".to_vec()],
            vec![b"aa".to_vec(), b"bb".to_vec()],
            DataType::FixedSizeBinary(1),
        );
    }
}
