// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module provides adaptive array builders for Arrow arrays.
//!
//! Often for OTel-Arrow, we have columns that are optional on the schema. For example, the Int
//! column may not be present in a record batch representing a list of attributes only be present
//! if there are some int type values in the list.
//!
//! There are also cases where we want to dynamically use dictionary encoding with the smallest index
//! the cardinality of the allows.
//!
//! This module contains adaptive array builders that can dynamically create either no array (for
//! an all-null or all default-value columns) , an array that may be a dictionary, of an array or
//! native types. It will handle converting between different builders dynamically  based on the
//! data which is appended.

use arrow::array::{
    ArrayRef, ArrowPrimitiveType, BinaryBuilder, BinaryDictionaryBuilder, FixedSizeBinaryBuilder,
    FixedSizeBinaryDictionaryBuilder, PrimitiveBuilder, PrimitiveDictionaryBuilder, StringBuilder,
    StringDictionaryBuilder,
};
use arrow::datatypes::{
    DurationNanosecondType, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type,
    TimestampNanosecondType, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow::error::ArrowError;

use crate::arrays::NullableArrayAccessor;
use crate::encode::record::array::dictionary::{
    CheckedDictionaryAppendSlice, DictionaryArrayAppendSlice, DictionaryArrayAppendStr,
    DictionaryBuilder,
};
use crate::encode::record::array::prefix::ArrayPrefixBuilder;

use dictionary::{
    AdaptiveDictionaryBuilder, CheckedDictionaryArrayAppend, ConvertToNativeHelper,
    DictionaryArrayAppend, DictionaryBuilderError, DictionaryOptions, UpdateDictionaryIndexInto,
    checked,
};

pub mod binary;
pub mod boolean;
pub mod dictionary;
pub mod fixed_size_binary;
pub mod prefix;
pub mod primitive;
pub mod string;

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

/// this trait can be implemented by types that can receive a value to append as a type of str.
///
/// This is mainly useful to avoid copies when calling types that implement ArrayAppend with a
/// Native type of String, if the caller already has a str.
pub trait ArrayAppendStr {
    /// Append a value of type str to the builder
    fn append_str(&mut self, value: &str);
}

/// this trait can be implemented by types that can receive a value to append as a type of &[T].
///
/// This is mainly useful to avoid copies when calling types that implement ArrayAppend with a
/// Native type of Vec<u8>, if the caller already has a byte slice of &[u8]
pub trait ArrayAppendSlice {
    type Native;

    /// append a slice of T to the builder. Note that this does not append an individual
    /// element for each value in the slice, it appends the slice as a single row
    fn append_slice(&mut self, val: &[Self::Native]);
}

/// Checked variant of ArrayAppendSlice for values that can return an error
pub trait CheckedArrayAppendSlice {
    type Native;

    /// append a slice of T to the builder. Note that this does not append an individual
    /// element for each value in the slice, it appends the slice as a single row
    fn append_slice(&mut self, val: &[Self::Native]) -> Result<(), ArrowError>;
}

/// Used by the builder to identify the default value of the array that is being built. By default
/// the adaptive array builder will not produce an array if all the values are either null or
/// default value.
///
/// Having a trait that can return the default value makes it os the parent array builder can
/// produce an instance of the value lazily, only when needed.
pub trait DefaultValueProvider<T: Clone + PartialEq, TArgs> {
    /// Produce a copy of the default value. This takes the arguments which can be used to compute
    /// the value if necessary.
    fn default_value(args: TArgs) -> T;
}

/// This enum is a container that abstracts array builder which is either
/// dictionary or native. It converts from the dictionary builder to the
/// native builder when the dictionary builder overflows.
enum InnerBuilder<NativeBuilder, DictBuilderU8, DictBuilderU16> {
    Uninitialized(ArrayPrefixBuilder),
    Native(NativeBuilder),
    Dictionary(AdaptiveDictionaryBuilder<DictBuilderU8, DictBuilderU16>),
}

pub struct ArrayOptions {
    pub dictionary_options: Option<DictionaryOptions>,

    /// Whether this is an "optional" array. For an optional array, if all the values are null or
    /// the default value, we will not build the array
    pub optional: bool,

    /// Whether or not default values are optional. If this is true, the array won't be produced
    /// if all the value are either null or default.
    pub default_values_optional: bool,
}

impl Default for ArrayOptions {
    fn default() -> Self {
        Self {
            dictionary_options: None,
            optional: true,
            default_values_optional: true,
        }
    }
}

pub struct AdaptiveArrayBuilder<T: Clone + PartialEq, TArgs, TN, TD8, TD16> {
    dictionary_options: Option<DictionaryOptions>,
    inner: InnerBuilder<TN, TD8, TD16>,

    // these are the args that will be used to create the underlying builder. In most cases this
    // will be NoArgs, but there are some cases where Array builder's constructors require args,
    // for example `FixedSizeBinary` requires the byte_width
    inner_args: TArgs,

    // the default value for the type being built. If this is set to Some(T), when appending a
    // value to the uninitialized state, we'll check if the value is equal to the default value
    // and if so, avoid initializing the builder. This behaviour can be controlled by the
    // constructor option `default_values_optional`
    default_value: Option<T>,
}

impl<T, TN, TD8, TD16> AdaptiveArrayBuilder<T, NoArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq + Default,
    TN: ArrayBuilderConstructor<Args = NoArgs> + ArrayAppendNulls + DefaultValueProvider<T, NoArgs>,
    TD8: ArrayBuilderConstructor<Args = NoArgs> + ArrayAppendNulls,
    TD16: ArrayBuilderConstructor<Args = NoArgs> + ArrayAppendNulls,
{
    /// Creates a new instance of the adaptive array builder.
    pub fn new(options: ArrayOptions) -> Self {
        Self::new_with_args(options, ())
    }
}

impl<T, TArgs, TN, TD8, TD16> AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
    TArgs: Clone,
    TN: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls + DefaultValueProvider<T, TArgs>,
    TD8: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls,
    TD16: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls,
{
    pub fn new_with_args(options: ArrayOptions, args: TArgs) -> Self {
        let inner = if options.optional {
            InnerBuilder::Uninitialized(ArrayPrefixBuilder::new())
        } else {
            Self::initialized_inner(args.clone(), &options.dictionary_options)
        };

        let default_value = options
            .default_values_optional
            .then(|| TN::default_value(args.clone()));

        Self {
            dictionary_options: options.dictionary_options,
            inner,
            inner_args: args,
            default_value,
        }
    }
}

impl<T, TArgs, TN, TD8, TD16> AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
    TArgs: Clone,
    TN: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls,
    TD8: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls,
    TD16: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls,
{
    /// check if the value is equal to the default value. returns false if default_value is `None``
    fn is_default_value<T2>(default_value: &Option<T>, value: &T2) -> bool
    where
        T: PartialEq<T2>,
    {
        if let Some(default_val) = default_value {
            if default_val == value {
                // prefix.append_value();
                return true;
            }
        }

        false
    }

    /// Creates the initial the builder, which may either be a builder for the dict, if dictionary
    /// options is `Some`, otherwise it will construct the native builder variant
    fn initialized_inner(
        args: TArgs,
        dictionary_options: &Option<DictionaryOptions>,
    ) -> InnerBuilder<TN, TD8, TD16> {
        match dictionary_options.as_ref() {
            Some(dictionary_options) => {
                InnerBuilder::Dictionary(AdaptiveDictionaryBuilder::new(dictionary_options, args))
            }
            None => InnerBuilder::Native(TN::new(args)),
        }
    }

    /// initialize self.inner if it's not already initialized returning the prefix builder
    #[must_use]
    fn initialize_inner(&mut self) -> Option<ArrayPrefixBuilder> {
        if matches!(self.inner, InnerBuilder::Uninitialized(_)) {
            let prefix = match std::mem::replace(
                &mut self.inner,
                Self::initialized_inner(self.inner_args.clone(), &self.dictionary_options),
            ) {
                InnerBuilder::Uninitialized(prefix) => prefix,
                _ => {
                    // safety: shouldn't happen because we have a check above
                    panic!("unexpected state initializing the inner builder")
                }
            };

            Some(prefix)
        } else {
            None
        }
    }
}

impl<T, TArgs, TN, TD8, TD16> ArrayAppendNulls for AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
    TN: ArrayAppendNulls,
    TD8: ArrayAppendNulls,
    TD16: ArrayAppendNulls,
{
    fn append_null(&mut self) {
        match &mut self.inner {
            InnerBuilder::Native(builder) => builder.append_null(),
            InnerBuilder::Dictionary(builder) => builder.append_null(),
            InnerBuilder::Uninitialized(prefix) => prefix.append_null(),
        }
    }

    fn append_nulls(&mut self, n: usize) {
        match &mut self.inner {
            InnerBuilder::Native(builder) => builder.append_nulls(n),
            InnerBuilder::Dictionary(builder) => builder.append_nulls(n),
            InnerBuilder::Uninitialized(prefix) => prefix.append_nulls(n),
        }
    }
}

impl<T, TArgs, TN, TD8, TD16> AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
    TArgs: Clone,
    TN: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls + ArrayAppend<Native = T>,
    TD8: ArrayBuilderConstructor<Args = TArgs>
        + ArrayAppendNulls
        + ConvertToNativeHelper
        + DictionaryArrayAppend<Native = T>
        + DictionaryBuilder<UInt8Type>
        + UpdateDictionaryIndexInto<TD16>,
    <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    TD16: ArrayBuilderConstructor<Args = TArgs>
        + ArrayAppendNulls
        + ConvertToNativeHelper
        + DictionaryArrayAppend<Native = T>
        + DictionaryBuilder<UInt16Type>,
    <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
{
    // generic function to handle doing an append and upgrading the array. There are some types
    // that have an optimized append mechanism, for example to avoid cloning str and slice so
    // we try to use the optimized append functions here while having one generic function that
    // contains logic for how/when to upgrade the inner builder
    fn handle_append<FIsDefault, FNative, FDict, FRetry>(
        &mut self,
        is_default: FIsDefault,
        default_value: &Option<T>,
        mut append_native_fn: FNative,
        mut append_dict_fn: FDict,
        retry_append: FRetry,
    ) where
        FIsDefault: FnOnce() -> bool,
        FNative: FnMut(&mut TN),
        FDict: FnMut(
            &mut AdaptiveDictionaryBuilder<TD8, TD16>,
        ) -> Result<usize, DictionaryBuilderError>,
        FRetry: FnOnce(&mut Self),
    {
        match &mut self.inner {
            InnerBuilder::Native(native_builder) => {
                append_native_fn(native_builder);
            }
            InnerBuilder::Dictionary(dictionary_builder) => {
                match append_dict_fn(dictionary_builder) {
                    Ok(_) => {}
                    Err(DictionaryBuilderError::DictOverflow {}) => {
                        let mut native = TN::new(self.inner_args.clone());
                        dictionary_builder.to_native(&mut native);
                        self.inner = InnerBuilder::Native(native);
                        retry_append(self);
                    }
                }
            }
            InnerBuilder::Uninitialized(prefix) => {
                if is_default() {
                    prefix.append_value();
                } else {
                    let mut prefix = self.initialize_inner().expect("can get prefix");
                    prefix.init_builder(self, default_value.clone());
                    retry_append(self);
                }
            }
        }
    }
}

impl<T, TArgs, TN, TD8, TD16> AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
    TArgs: Clone,
    TN: ArrayBuilderConstructor<Args = TArgs> + ArrayAppendNulls + CheckedArrayAppend<Native = T>,
    TD8: ArrayBuilderConstructor<Args = TArgs>
        + ArrayAppendNulls
        + ConvertToNativeHelper
        + CheckedDictionaryArrayAppend<Native = T>
        + DictionaryBuilder<UInt8Type>
        + UpdateDictionaryIndexInto<TD16>,
    <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    TD16: ArrayBuilderConstructor<Args = TArgs>
        + ArrayAppendNulls
        + ConvertToNativeHelper
        + CheckedDictionaryArrayAppend<Native = T>
        + DictionaryBuilder<UInt16Type>,
    <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
{
    fn handle_append_checked<FIsDefault, FNative, FDict, FRetry>(
        &mut self,
        is_default: FIsDefault,
        default_value: &Option<T>,
        mut append_native_fn: FNative,
        mut append_dict_fn: FDict,
        retry_append: FRetry,
    ) -> Result<(), ArrowError>
    where
        FIsDefault: FnOnce() -> bool,
        FNative: FnMut(&mut TN) -> Result<(), ArrowError>,
        FDict: FnMut(
            &mut AdaptiveDictionaryBuilder<TD8, TD16>,
        ) -> Result<usize, checked::DictionaryBuilderError>,
        FRetry: FnOnce(&mut Self) -> Result<(), ArrowError>,
    {
        match &mut self.inner {
            InnerBuilder::Native(native_builder) => append_native_fn(native_builder),
            InnerBuilder::Dictionary(dictionary_builder) => {
                match append_dict_fn(dictionary_builder) {
                    Ok(_) => {
                        // append succeeded
                        Ok(())
                    }

                    Err(checked::DictionaryBuilderError::DictOverflow {}) => {
                        let mut native = TN::new(self.inner_args.clone());
                        dictionary_builder.to_native_checked(&mut native)?;
                        self.inner = InnerBuilder::Native(native);
                        retry_append(self)
                    }
                    Err(checked::DictionaryBuilderError::CheckedBuilderError {
                        source: arrow_error,
                    }) => Err(arrow_error),
                }
            }
            InnerBuilder::Uninitialized(prefix) => {
                if is_default() {
                    prefix.append_value();
                    Ok(())
                } else {
                    // safety: initialize_inner will return the prefix if the inner variant is
                    // `Uninitialized` which we've checked in the match here
                    let mut prefix = self.initialize_inner().expect("can get prefix");
                    prefix.init_builder_checked(self, default_value.clone())?;
                    retry_append(self)
                }
            }
        }
    }
}

impl<T, TArgs, TN, TD8, TD16> ArrayAppend for AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
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
        // temporarily move the value of default_value to avoid borrowing self as mut and non-mut
        // at the same time
        let default_value = std::mem::take(&mut self.default_value);
        let is_default = || Self::is_default_value(&default_value, value);
        self.handle_append(
            is_default,
            &default_value,
            |native| native.append_value(value),
            |dict| dict.append_value(value),
            |me| me.append_value(value),
        );

        // restore the default value
        self.default_value = default_value;
    }
}

impl<TArgs, TN, TD8, TD16> ArrayAppendStr for AdaptiveArrayBuilder<String, TArgs, TN, TD8, TD16>
where
    TArgs: Clone,
    TN: ArrayAppendStr
        + ArrayAppend<Native = String>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>,
    TD8: DictionaryArrayAppendStr
        + DictionaryArrayAppend<Native = String>
        + DictionaryBuilder<UInt8Type>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper
        + UpdateDictionaryIndexInto<TD16>,
    <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = String> + 'static,
    TD16: DictionaryArrayAppendStr
        + DictionaryArrayAppend<Native = String>
        + ArrayAppendNulls
        + DictionaryBuilder<UInt16Type>
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper,
    <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = String> + 'static,
{
    fn append_str(&mut self, value: &str) {
        // temporarily move the value of default_value to avoid borrowing self as mut and non-mut
        // at the same time
        let default_value = std::mem::take(&mut self.default_value);
        let is_default = || Self::is_default_value(&default_value, &value);
        self.handle_append(
            is_default,
            &default_value,
            |native| native.append_str(value),
            |dict| dict.append_str(value),
            |me| me.append_str(value),
        );

        // restore value of default value
        self.default_value = default_value
    }
}

impl<T, TArgs, TN, TD8, TD16> ArrayAppendSlice
    for AdaptiveArrayBuilder<Vec<T>, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
    TArgs: Clone,
    TN: ArrayAppendSlice<Native = T>
        + ArrayAppend<Native = Vec<T>>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>,
    TD8: DictionaryArrayAppendSlice<Native = T>
        + DictionaryArrayAppend<Native = Vec<T>>
        + DictionaryBuilder<UInt8Type>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper
        + UpdateDictionaryIndexInto<TD16>,
    <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = Vec<T>> + 'static,
    TD16: DictionaryArrayAppendSlice<Native = T>
        + DictionaryArrayAppend<Native = Vec<T>>
        + ArrayAppendNulls
        + DictionaryBuilder<UInt16Type>
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper,
    <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = Vec<T>> + 'static,
{
    type Native = T;

    fn append_slice(&mut self, value: &[Self::Native]) {
        // temporarily move the value of default_value to avoid borrowing self as mut and non-mut
        // at the same time
        let default_value = std::mem::take(&mut self.default_value);
        let is_default = || Self::is_default_value(&default_value, &value);
        self.handle_append(
            is_default,
            &default_value,
            |native| native.append_slice(value),
            |dict| dict.append_slice(value),
            |me| me.append_slice(value),
        );
        // restore value of default value
        self.default_value = default_value
    }
}

impl<T, TArgs, TN, TD8, TD16> CheckedArrayAppend for AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
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
        // temporarily move the value of default_value to avoid borrowing self as mut and non-mut
        // at the same time
        let default_value = std::mem::take(&mut self.default_value);
        let is_default = || Self::is_default_value(&default_value, value);
        let result = self.handle_append_checked(
            is_default,
            &default_value,
            |native| native.append_value(value),
            |dict| dict.append_value_checked(value),
            |me| me.append_value(value),
        );

        // restore the default value
        self.default_value = default_value;

        result
    }
}

impl<T, TArgs, TN, TD8, TD16> CheckedArrayAppendSlice
    for AdaptiveArrayBuilder<Vec<T>, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
    TArgs: Clone,
    TN: CheckedArrayAppendSlice<Native = T>
        + CheckedArrayAppend<Native = Vec<T>>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>,
    TD8: CheckedDictionaryAppendSlice<Native = T>
        + CheckedDictionaryArrayAppend<Native = Vec<T>>
        + DictionaryBuilder<UInt8Type>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper
        + UpdateDictionaryIndexInto<TD16>,
    <TD8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = Vec<T>> + 'static,
    TD16: CheckedDictionaryAppendSlice<Native = T>
        + CheckedDictionaryArrayAppend<Native = Vec<T>>
        + DictionaryBuilder<UInt16Type>
        + ArrayAppendNulls
        + ArrayBuilderConstructor<Args = TArgs>
        + ConvertToNativeHelper,
    <TD16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = Vec<T>> + 'static,
{
    type Native = T;

    /// Try to append a value to the underlying builder. This method may return an error if
    /// the value is not valid
    fn append_slice(&mut self, value: &[Self::Native]) -> Result<(), ArrowError> {
        // temporarily move the value of default_value to avoid borrowing self as mut and non-mut
        // at the same time
        let default_value = std::mem::take(&mut self.default_value);
        let is_default = || Self::is_default_value(&default_value, &value);
        let result = self.handle_append_checked(
            is_default,
            &default_value,
            |native| native.append_slice(value),
            |dict| dict.append_slice_checked(value),
            |me| me.append_slice(value),
        );

        // restore the default value
        self.default_value = default_value;

        result
    }
}

impl<T, TArgs, TN, TD8, TD16> AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>
where
    T: Clone + PartialEq,
    TN: ArrayBuilder,
    TD8: DictionaryBuilder<UInt8Type>,
    TD16: DictionaryBuilder<UInt16Type>,
{
    pub fn finish(&mut self) -> Option<ArrayRef> {
        match &mut self.inner {
            InnerBuilder::Native(builder) => Some(builder.finish()),
            InnerBuilder::Dictionary(builder) => Some(builder.finish()),
            InnerBuilder::Uninitialized(_) => None,
        }
    }
}

// Arg type for an array constructor that takes no arguments.
pub(crate) type NoArgs = ();

pub type StringArrayBuilder = AdaptiveArrayBuilder<
    String,
    NoArgs,
    StringBuilder,
    StringDictionaryBuilder<UInt8Type>,
    StringDictionaryBuilder<UInt16Type>,
>;

pub type BinaryArrayBuilder = AdaptiveArrayBuilder<
    Vec<u8>,
    NoArgs,
    BinaryBuilder,
    BinaryDictionaryBuilder<UInt8Type>,
    BinaryDictionaryBuilder<UInt16Type>,
>;

pub type FixedSizeBinaryArrayBuilder = AdaptiveArrayBuilder<
    Vec<u8>,
    i32,
    FixedSizeBinaryBuilder,
    FixedSizeBinaryDictionaryBuilder<UInt8Type>,
    FixedSizeBinaryDictionaryBuilder<UInt16Type>,
>;

pub type PrimitiveArrayBuilder<T> = AdaptiveArrayBuilder<
    <T as ArrowPrimitiveType>::Native,
    NoArgs,
    PrimitiveBuilder<T>,
    PrimitiveDictionaryBuilder<UInt8Type, T>,
    PrimitiveDictionaryBuilder<UInt16Type, T>,
>;

// aliases for adaptive primitive array builders
#[allow(dead_code)]
pub type Float32ArrayBuilder = PrimitiveArrayBuilder<Float32Type>;
pub type Float64ArrayBuilder = PrimitiveArrayBuilder<Float64Type>;
pub type UInt8ArrayBuilder = PrimitiveArrayBuilder<UInt8Type>;
pub type UInt16ArrayBuilder = PrimitiveArrayBuilder<UInt16Type>;
pub type UInt32ArrayBuilder = PrimitiveArrayBuilder<UInt32Type>;
#[allow(dead_code)]
pub type UInt64ArrayBuilder = PrimitiveArrayBuilder<UInt64Type>;
#[allow(dead_code)]
pub type Int8ArrayBuilder = PrimitiveArrayBuilder<Int8Type>;
#[allow(dead_code)]
pub type Int16ArrayBuilder = PrimitiveArrayBuilder<Int16Type>;
pub type Int32ArrayBuilder = PrimitiveArrayBuilder<Int32Type>;
pub type Int64ArrayBuilder = PrimitiveArrayBuilder<Int64Type>;
pub type TimestampNanosecondArrayBuilder = PrimitiveArrayBuilder<TimestampNanosecondType>;
#[allow(dead_code)]
pub type DurationNanosecondArrayBuilder = PrimitiveArrayBuilder<DurationNanosecondType>;

#[cfg(test)]
pub mod test {
    use super::*;

    use arrow::array::{Array, DictionaryArray, FixedSizeBinaryArray, UInt8Array};
    use arrow::datatypes::{DataType, TimeUnit};

    fn test_array_builder_generic<T, TArgs, TN, TD8, TD16>(
        array_builder_factory: &impl Fn(ArrayOptions) -> AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>,
        expected_data_type: DataType,
    ) where
        T: Clone + PartialEq + std::fmt::Debug,
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
            optional: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
        });
        let result = builder.finish();
        assert!(result.is_none());

        // expect that if it is non-nullable, we always get an empty array instead of 'None'
        let mut builder = array_builder_factory(ArrayOptions {
            optional: false,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
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
            optional: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
        });
        builder.append_null();
        builder.append_nulls(2);
        let result = builder.finish();
        assert!(result.is_none());
    }

    fn test_array_append_generic<T, TN, TD8, TD16>(
        array_builder_factory: impl Fn(ArrayOptions) -> AdaptiveArrayBuilder<T, NoArgs, TN, TD8, TD16>,
        values: Vec<T>,
        expected_data_type: DataType,
    ) where
        T: Clone + PartialEq + std::fmt::Debug + Default,
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
            optional: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
        });

        // expect that for empty array, we get a None value because the builder is nullable
        let result = builder.finish();
        assert!(result.is_none());

        // expect that if we append a bunch of null/default values that we get none array because
        // the builder is building an optional array
        builder.append_value(&T::default());
        builder.append_value(&T::default());
        builder.append_null();
        let result = builder.finish();
        assert!(result.is_none());

        let mut builder = array_builder_factory(ArrayOptions {
            optional: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
        });

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
            optional: false,
            ..Default::default()
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
            optional: false,
            ..Default::default()
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
            optional: true,
            ..Default::default()
        });
        builder.append_null();
        builder.append_nulls(2);
        builder.append_value(&T::default());
        builder.append_value(&values[0]);
        let result = builder.finish().unwrap();
        let dict_array = result
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        let dict_keys = dict_array.keys();
        assert_eq!(
            dict_keys,
            &UInt8Array::from_iter(vec![None, None, None, Some(0), Some(1)])
        );
    }

    #[test]
    fn test_array_builder() {
        test_array_append_generic(UInt8ArrayBuilder::new, vec![2, 1], DataType::UInt8);
        test_array_append_generic(UInt16ArrayBuilder::new, vec![2, 1], DataType::UInt16);
        test_array_append_generic(UInt32ArrayBuilder::new, vec![2, 1], DataType::UInt32);
        test_array_append_generic(UInt64ArrayBuilder::new, vec![2, 1], DataType::UInt64);
        test_array_append_generic(Int8ArrayBuilder::new, vec![2, 1], DataType::Int8);
        test_array_append_generic(Int16ArrayBuilder::new, vec![2, 1], DataType::Int16);
        test_array_append_generic(Int32ArrayBuilder::new, vec![2, 1], DataType::Int32);
        test_array_append_generic(Int64ArrayBuilder::new, vec![2, 1], DataType::Int64);
        test_array_append_generic(Float32ArrayBuilder::new, vec![2.0, 1.0], DataType::Float32);
        test_array_append_generic(Float64ArrayBuilder::new, vec![2.0, 1.1], DataType::Float64);
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
            vec![2, 1],
            DataType::Timestamp(TimeUnit::Nanosecond, None),
        );
        test_array_append_generic(
            DurationNanosecondArrayBuilder::new,
            vec![2, 1],
            DataType::Duration(TimeUnit::Nanosecond),
        );
    }

    #[test]
    fn test_default_values_not_optional() {
        // assert that by default, we don't create an optional builder that's only default values
        let mut builder = UInt32ArrayBuilder::new(ArrayOptions {
            optional: true,
            dictionary_options: None,
            ..Default::default()
        });
        builder.append_value(&0);
        assert!(builder.finish().is_none());

        // assert that by passing default_value = None, we do create the array
        let mut builder = UInt32ArrayBuilder::new(ArrayOptions {
            optional: true,
            dictionary_options: None,
            default_values_optional: false,
        });
        builder.append_value(&0);
        assert!(builder.finish().is_some());
    }

    #[test]
    fn test_string_array_builder_append_str() {
        let mut builder = StringArrayBuilder::new(ArrayOptions {
            optional: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
        });

        // Append using append_str
        builder.append_str("foo");
        builder.append_str("bar");
        builder.append_null();
        builder.append_str("foo");
        builder.append_str("baz");
        builder.append_nulls(2);

        let result = builder.finish().unwrap();
        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        );
        assert_eq!(result.len(), 7);

        let dict_array = result
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        let dict_keys = dict_array.keys();
        assert_eq!(
            dict_keys,
            &UInt8Array::from_iter(vec![Some(0), Some(1), None, Some(0), Some(2), None, None])
        );
        let dict_values = dict_array
            .values()
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .unwrap();
        assert_eq!(dict_values.value(0), "foo");
        assert_eq!(dict_values.value(1), "bar");
        assert_eq!(dict_values.value(2), "baz");

        // This test checks that when dictionary overflows, we fallback to native builder
        let mut builder = StringArrayBuilder::new(ArrayOptions {
            optional: false,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 1,
                min_cardinality: 1,
            }),
            ..Default::default()
        });

        builder.append_str("");
        builder.append_str("a");
        builder.append_null();
        builder.append_nulls(2);
        builder.append_str("b"); // triggers overflow

        let result = builder.finish().unwrap();
        assert_eq!(result.len(), 6);

        let array = result
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .unwrap();
        assert_eq!(array.value(0), "");
        assert_eq!(array.value(1), "a");
        assert!(!array.is_valid(2));
        assert!(!array.is_valid(3));
        assert!(!array.is_valid(4));
        assert_eq!(array.value(5), "b");

        // ensure that we don't produce an optional array that is full of default values
        let mut builder = StringArrayBuilder::new(ArrayOptions {
            optional: true,
            dictionary_options: None,
            ..Default::default()
        });
        builder.append_str("");
        builder.append_str("");
        builder.append_str("");
        assert!(builder.finish().is_none());
    }

    #[test]
    fn test_binary_array_builder_append_slice() {
        let mut builder = BinaryArrayBuilder::new(ArrayOptions {
            optional: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
        });

        // Append using append_slice
        builder.append_slice(b"foo");
        builder.append_slice(b"bar");
        builder.append_null();
        builder.append_slice(b"foo");
        builder.append_slice(b"baz");
        builder.append_nulls(2);

        let result = builder.finish().unwrap();
        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Binary))
        );
        assert_eq!(result.len(), 7);

        let dict_array = result
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        let dict_keys = dict_array.keys();
        assert_eq!(
            dict_keys,
            &UInt8Array::from_iter(vec![Some(0), Some(1), None, Some(0), Some(2), None, None])
        );
        let dict_values = dict_array
            .values()
            .as_any()
            .downcast_ref::<arrow::array::BinaryArray>()
            .unwrap();
        assert_eq!(dict_values.value(0), b"foo");
        assert_eq!(dict_values.value(1), b"bar");
        assert_eq!(dict_values.value(2), b"baz");

        // This test checks that when dictionary overflows, we fallback to native builder
        let mut builder = BinaryArrayBuilder::new(ArrayOptions {
            optional: false,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 1,
                min_cardinality: 1,
            }),
            ..Default::default()
        });

        builder.append_slice(b"");
        builder.append_slice(b"a");
        builder.append_null();
        builder.append_nulls(2);
        builder.append_slice(b"b"); // triggers overflow

        let result = builder.finish().unwrap();
        assert_eq!(result.len(), 6);

        let array = result
            .as_any()
            .downcast_ref::<arrow::array::BinaryArray>()
            .unwrap();
        assert_eq!(array.value(0), b"");
        assert_eq!(array.value(1), b"a");
        assert!(!array.is_valid(2));
        assert!(!array.is_valid(3));
        assert!(!array.is_valid(4));
        assert_eq!(array.value(5), b"b");

        // ensure that we don't produce an optional array that is full of default values
        let mut builder = BinaryArrayBuilder::new(ArrayOptions {
            optional: true,
            dictionary_options: None,
            ..Default::default()
        });
        builder.append_slice(b"");
        builder.append_slice(b"");
        builder.append_slice(b"");
        assert!(builder.finish().is_none());
    }

    #[test]
    fn test_checked_array_builder_append_slice() {
        fn checked_binary_array_builder_factory(opts: ArrayOptions) -> FixedSizeBinaryArrayBuilder {
            // FixedSizeBinaryArrayBuilder expects byte_width as the second arg
            FixedSizeBinaryArrayBuilder::new_with_args(opts, 1)
        }

        // Valid values: single byte slices
        let valid_values = [b"a".to_vec(), b"b".to_vec()];
        // Invalid values: slices with length != 1
        let invalid_values = [b"aa".to_vec(), b"bb".to_vec()];

        // Test with dictionary options
        let mut builder = checked_binary_array_builder_factory(ArrayOptions {
            optional: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
        });

        // Append valid slices
        assert!(builder.append_slice(&valid_values[0]).is_ok());
        assert!(builder.append_slice(&valid_values[1]).is_ok());
        builder.append_null();
        assert!(builder.append_slice(&valid_values[0]).is_ok());
        assert!(builder.append_slice(&valid_values[1]).is_ok());
        builder.append_nulls(2);

        let result = builder.finish().unwrap();
        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(
                Box::new(DataType::UInt8),
                Box::new(DataType::FixedSizeBinary(1))
            )
        );
        assert_eq!(result.len(), 7);

        let dict_array = result
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        let dict_keys = dict_array.keys();
        assert_eq!(
            dict_keys,
            &UInt8Array::from_iter(vec![Some(0), Some(1), None, Some(0), Some(1), None, None])
        );
        let dict_values = dict_array
            .values()
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .unwrap();
        assert_eq!(dict_values.value(0), b"a");
        assert_eq!(dict_values.value(1), b"b");

        // Test fallback to native builder on dictionary overflow
        let mut builder = checked_binary_array_builder_factory(ArrayOptions {
            optional: false,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 1,
                min_cardinality: 1,
            }),
            ..Default::default()
        });

        assert!(builder.append_slice(&valid_values[0]).is_ok());
        builder.append_null();
        builder.append_nulls(2);
        assert!(builder.append_slice(&valid_values[1]).is_ok()); // triggers overflow

        let result = builder.finish().unwrap();
        assert_eq!(result.len(), 5);

        let array = result
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .unwrap();
        assert_eq!(array.value(0), b"a");
        assert!(!array.is_valid(1));
        assert!(!array.is_valid(2));
        assert!(!array.is_valid(3));
        assert_eq!(array.value(4), b"b");

        // Test invalid value is rejected
        let mut builder = checked_binary_array_builder_factory(ArrayOptions {
            optional: false,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
        });
        let result = builder.append_slice(&invalid_values[0]);
        assert!(matches!(result, Err(ArrowError::InvalidArgumentError(_))));
    }

    fn test_checked_array_builder_generic<T, TArgs, TN, TD8, TD16>(
        array_builder_factory: impl Fn(ArrayOptions) -> AdaptiveArrayBuilder<T, TArgs, TN, TD8, TD16>,
        values: Vec<T>,
        invalid_values: Vec<T>,
        expected_data_type: DataType,
        default_value: T,
    ) where
        T: Clone + PartialEq + std::fmt::Debug,
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

        // ensure we will wont produce a builder if the array has many prepended nulls and
        // default values
        let mut builder = array_builder_factory(ArrayOptions {
            dictionary_options: None,
            optional: true,
            ..Default::default()
        });
        assert!(builder.append_value(&default_value.clone()).is_ok());
        builder.append_null();
        let result = builder.finish();
        assert!(result.is_none());

        let mut builder = array_builder_factory(ArrayOptions {
            optional: true,
            dictionary_options: Some(DictionaryOptions {
                max_cardinality: 4,
                min_cardinality: 4,
            }),
            ..Default::default()
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
            optional: false,
            ..Default::default()
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
            optional: false,
            ..Default::default()
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
            optional: false,
            ..Default::default()
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
            vec![0],
        );
    }
}
