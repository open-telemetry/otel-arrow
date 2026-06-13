// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// TODO: Should this (or some portion of this) be made pub in the oss repo for ease of exporter
// development?

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, BinaryArray, BooleanArray, DictionaryArray,
    FixedSizeBinaryArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array,
    Int64Array, PrimitiveArray, RecordBatch, StringArray, StructArray, TimestampNanosecondArray,
    UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, DataType, Fields, TimeUnit, UInt8Type, UInt16Type,
};
use otap_df_pdata::error::{Error, Result};
use paste::paste;

pub trait NullableArrayAccessor {
    type Native;

    fn value_at(&self, idx: usize) -> Option<Self::Native>;

    fn value_at_or_default(&self, idx: usize) -> Self::Native
    where
        Self::Native: Default,
    {
        self.value_at(idx).unwrap_or_default()
    }
}

impl<T> NullableArrayAccessor for &T
where
    T: NullableArrayAccessor,
{
    type Native = T::Native;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        (*self).value_at(idx)
    }
}

impl<T> NullableArrayAccessor for Option<T>
where
    T: NullableArrayAccessor,
{
    type Native = T::Native;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        self.as_ref().and_then(|r| r.value_at(idx))
    }
}

impl<T> NullableArrayAccessor for PrimitiveArray<T>
where
    T: ArrowPrimitiveType,
{
    type Native = T::Native;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        if self.is_valid(idx) {
            Some(self.value(idx))
        } else {
            None
        }
    }
}

impl NullableArrayAccessor for BooleanArray {
    type Native = bool;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        if self.is_valid(idx) {
            Some(self.value(idx))
        } else {
            None
        }
    }
}

impl NullableArrayAccessor for BinaryArray {
    type Native = Vec<u8>;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        if self.is_valid(idx) {
            Some(self.value(idx).to_vec())
        } else {
            None
        }
    }
}

impl NullableArrayAccessor for FixedSizeBinaryArray {
    type Native = Vec<u8>;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        if self.is_valid(idx) {
            Some(self.value(idx).to_vec())
        } else {
            None
        }
    }
}

impl NullableArrayAccessor for StringArray {
    type Native = String;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        if self.is_valid(idx) {
            Some(self.value(idx).to_string())
        } else {
            None
        }
    }
}

macro_rules! impl_downcast {
    ($suffix:ident, $data_type:expr, $array_type:ident) => {
        paste!{
            #[allow(dead_code)]
            pub fn [<get_ $suffix _array_opt> ]<'a>(rb: &'a RecordBatch, name: &str) -> Result<Option<&'a $array_type>> {
                use arrow::datatypes::DataType::*;
                rb.column_by_name(name)
                    .map(|arr|{
                        arr.as_any()
                            .downcast_ref::<$array_type>()
                            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                                name: name.into(),
                                expect: $data_type,
                                actual: arr.data_type().clone(),
                            })
                }).transpose()
            }

            #[allow(dead_code)]
              pub fn [<get_ $suffix _array> ]<'a>(rb: &'a RecordBatch, name: &str) -> Result<&'a $array_type> {
                use arrow::datatypes::DataType::*;
                let arr = get_required_array(rb, name)?;

                 arr.as_any()
                            .downcast_ref::<$array_type>()
                            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                                name: name.into(),
                                expect: $data_type,
                                actual: arr.data_type().clone(),
                            })
            }
        }
    };
}

impl_downcast!(u8, UInt8, UInt8Array);
impl_downcast!(u16, UInt16, UInt16Array);
impl_downcast!(u32, UInt32, UInt32Array);
impl_downcast!(u64, UInt64, UInt64Array);
impl_downcast!(i8, Int8, Int8Array);
impl_downcast!(i16, Int16, Int16Array);
impl_downcast!(i32, Int32, Int32Array);
impl_downcast!(i64, Int64, Int64Array);
impl_downcast!(bool, Boolean, BooleanArray);

impl_downcast!(f32, Float32, Float32Array);
impl_downcast!(f64, Float64, Float64Array);

impl_downcast!(string, Utf8, StringArray);
impl_downcast!(binary, Binary, BinaryArray);

impl_downcast!(
    timestamp_nanosecond,
    Timestamp(TimeUnit::Nanosecond, None),
    TimestampNanosecondArray
);

/// Get reference to array that the caller requires to be in the record batch.
/// If the column is not in the record batch, returns `ColumnNotFound` error
pub fn get_required_array<'a>(
    record_batch: &'a RecordBatch,
    column_name: &str,
) -> Result<&'a ArrayRef> {
    record_batch
        .column_by_name(column_name)
        .ok_or_else(|| Error::ColumnNotFound {
            name: column_name.into(),
        })
}

/// Get reference to array that may be in the record batch.
/// If the column is not in the record batch, returns None
pub fn get_array_op<'a>(record_batch: &'a RecordBatch, column_name: &str) -> Option<&'a ArrayRef> {
    record_batch.column_by_name(column_name)
}

/// Get reference to a struct array that the caller requires to be in the record batch.
/// If the column is not in the record batch, returns `ColumnNotFound` error
/// if the column is not a struct array, returns `ColumnDataTypeMismatch` error
#[allow(dead_code)]
pub fn get_required_struct_array<'a>(
    record_batch: &'a RecordBatch,
    column_name: &str,
) -> Result<&'a StructArray> {
    let struct_arr =
        record_batch
            .column_by_name(column_name)
            .ok_or_else(|| Error::ColumnNotFound {
                name: column_name.into(),
            })?;
    struct_arr
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| Error::ColumnDataTypeMismatch {
            name: column_name.into(),
            actual: struct_arr.data_type().clone(),
            expect: DataType::Struct(Fields::empty()),
        })
}

/// Get reference to array that the caller requires to be in the struct array
/// If the column is not in the struct array, returns `ColumnNotFound` error
#[allow(dead_code)]
pub fn get_required_array_from_struct_array<'a>(
    struct_arr: &'a StructArray,
    column_name: &str,
) -> Result<&'a ArrayRef> {
    struct_arr
        .column_by_name(column_name)
        .ok_or_else(|| Error::ColumnNotFound {
            name: column_name.into(),
        })
}

/// Get reference to array that the caller requires to be in a struct array
/// in a record batch.
/// If the column is not in the struct array/record batch, returns `ColumnNotFound` error
#[allow(dead_code)]
pub fn get_required_array_from_struct_array_from_record_batch<'a>(
    record_batch: &'a RecordBatch,
    record_batch_column_name: &str,
    struct_array_column_name: &str,
) -> Result<&'a ArrayRef> {
    let struct_arr = record_batch
        .column_by_name(record_batch_column_name)
        .ok_or_else(|| Error::ColumnNotFound {
            name: record_batch_column_name.into(),
        })?;

    let struct_arr = struct_arr
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| Error::ColumnDataTypeMismatch {
            name: record_batch_column_name.into(),
            actual: struct_arr.data_type().clone(),
            expect: DataType::Struct(Fields::empty()),
        })?;
    struct_arr
        .column_by_name(struct_array_column_name)
        .ok_or_else(|| Error::ColumnNotFound {
            name: struct_array_column_name.into(),
        })
}

#[allow(dead_code)]
trait NullableInt64ArrayAccessor {
    fn i64_at(&self, idx: usize) -> Result<Option<i64>>;
}

impl NullableInt64ArrayAccessor for &Int64Array {
    fn i64_at(&self, idx: usize) -> Result<Option<i64>> {
        Ok(self.value_at(idx))
    }
}

impl<T> NullableInt64ArrayAccessor for &DictionaryArray<T>
where
    T: ArrowDictionaryKeyType,
{
    fn i64_at(&self, idx: usize) -> Result<Option<i64>> {
        let Some(idx) = self.keys().value_at(idx) else {
            return Ok(None);
        };
        let x = self
            .values()
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("Int64 array");
        let value_idx = idx.to_usize().expect("log");
        Ok(x.value_at(value_idx))
    }
}

#[allow(dead_code)]
trait NullableF64ArrayAccessor {
    fn f64_at(&self, idx: usize) -> Result<Option<f64>>;
}

impl NullableF64ArrayAccessor for &Float64Array {
    fn f64_at(&self, idx: usize) -> Result<Option<f64>> {
        Ok(self.value_at(idx))
    }
}

impl<T> NullableF64ArrayAccessor for &DictionaryArray<T>
where
    T: ArrowDictionaryKeyType,
{
    fn f64_at(&self, idx: usize) -> Result<Option<f64>> {
        let Some(idx) = self.keys().value_at(idx) else {
            return Ok(None);
        };

        let value_idx = idx.to_usize().expect("Invalid value index type");
        let x = self
            .values()
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("Float64 array");
        Ok(x.value_at(value_idx))
    }
}

/// Wrapper around various arrays that may return a byte slice. Note that
/// this delegates to the underlying NullableArrayAccessor implementation
/// for the Arrow array which copies the bytes when value_at is called
pub enum ByteArrayAccessor<'a> {
    Binary(MaybeDictArrayAccessor<'a, BinaryArray>),
    FixedSizeBinary(MaybeDictArrayAccessor<'a, FixedSizeBinaryArray>),
}

impl<'a> ByteArrayAccessor<'a> {
    #[allow(dead_code)]
    pub fn try_new_for_column(record_batch: &'a RecordBatch, column_name: &str) -> Result<Self> {
        Self::try_new(get_required_array(record_batch, column_name)?)
    }

    pub fn try_new(arr: &'a ArrayRef) -> Result<Self> {
        match arr.data_type() {
            DataType::Binary => {
                MaybeDictArrayAccessor::<BinaryArray>::try_new(arr).map(Self::Binary)
            }
            DataType::FixedSizeBinary(dims) => {
                MaybeDictArrayAccessor::<FixedSizeBinaryArray>::try_new(arr, *dims)
                    .map(Self::FixedSizeBinary)
            }
            DataType::Dictionary(_, val) => match **val {
                DataType::Binary => {
                    MaybeDictArrayAccessor::<BinaryArray>::try_new(arr).map(Self::Binary)
                }
                DataType::FixedSizeBinary(dims) => {
                    MaybeDictArrayAccessor::<FixedSizeBinaryArray>::try_new(arr, dims)
                        .map(Self::FixedSizeBinary)
                }
                _ => Err(Error::UnsupportedDictionaryValueType {
                    expect_oneof: vec![DataType::Binary, DataType::FixedSizeBinary(-1)],
                    actual: (**val).clone(),
                }),
            },
            _ => Err(Error::InvalidListArray {
                expect_oneof: vec![
                    DataType::Binary,
                    DataType::FixedSizeBinary(-1),
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Binary)),
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::FixedSizeBinary(-1)),
                    ),
                    DataType::Dictionary(
                        Box::new(DataType::UInt16),
                        Box::new(DataType::FixedSizeBinary(-1)),
                    ),
                ],
                actual: arr.data_type().clone(),
            }),
        }
    }
}

impl NullableArrayAccessor for ByteArrayAccessor<'_> {
    type Native = Vec<u8>;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        match self {
            Self::Binary(b) => b.value_at(idx),
            Self::FixedSizeBinary(b) => b.value_at(idx),
        }
    }
}

impl<'a> ByteArrayAccessor<'a> {
    pub fn slice_at(&self, idx: usize) -> Option<&[u8]> {
        match self {
            Self::Binary(b) => b.slice_at(idx),
            Self::FixedSizeBinary(b) => b.slice_at(idx),
        }
    }
}

/// Wrapper around an array that might be a dictionary or it might just be an unencoded
/// array of the base type
pub enum MaybeDictArrayAccessor<'a, V> {
    Native(&'a V),
    Dictionary8(DictionaryArrayAccessor<'a, UInt8Type, V>),
    Dictionary16(DictionaryArrayAccessor<'a, UInt16Type, V>),
}

impl<'a, T> NullableArrayAccessor for MaybeDictArrayAccessor<'a, T>
where
    T: Array + NullableArrayAccessor + 'static,
{
    type Native = T::Native;

    fn value_at(
        &self,
        idx: usize,
    ) -> Option<<MaybeDictArrayAccessor<'a, T> as NullableArrayAccessor>::Native> {
        match self {
            Self::Native(s) => s.value_at(idx),
            Self::Dictionary8(d) => d.value_at(idx),
            Self::Dictionary16(d) => d.value_at(idx),
        }
    }
}

impl<'a, T> MaybeDictArrayAccessor<'a, T>
where
    T: Array + NullableArrayAccessor + 'static,
{
    /// Inspects the given array to determine whether it can be treated as an array
    /// of the specified data type. The array must either:
    /// - Directly have the expected data type, or
    /// - Be a dictionary array whose value type matches the expected data type.
    ///
    /// Returns a wrapped native array if the type matches.
    /// Returns an error if the array type can't be treated as this datatype
    fn try_new_with_datatype(data_type: DataType, arr: &'a ArrayRef) -> Result<Self> {
        // if the type isn't a dictionary, we treat it as an unencoded array
        if *arr.data_type() == data_type {
            return Ok(Self::Native(
                arr.as_any()
                    .downcast_ref::<T>()
                    .expect("array can be downcast to it's native datatype"),
            ));
        }

        // determine if the type is a dictionary where the value is the desired datatype
        if let DataType::Dictionary(key, v) = arr.data_type() {
            if **v != data_type {
                return Err(Error::UnsupportedDictionaryValueType {
                    expect_oneof: vec![data_type],
                    actual: (**v).clone(),
                });
            }

            let result = match **key {
                DataType::UInt8 => Self::Dictionary8(DictionaryArrayAccessor::new(
                    arr.as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                        .expect("array can be downcast to DictionaryArray<UInt8Type"),
                )?),
                DataType::UInt16 => Self::Dictionary16(DictionaryArrayAccessor::new(
                    arr.as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .expect("array can be downcast to DictionaryArray<UInt16Type>"),
                )?),
                _ => {
                    return Err(Error::UnsupportedDictionaryKeyType {
                        expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                        actual: (**key).clone(),
                    });
                }
            };

            return Ok(result);
        }

        Err(Error::InvalidListArray {
            expect_oneof: vec![
                data_type.clone(),
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(data_type.clone())),
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(data_type.clone())),
            ],
            actual: arr.data_type().clone(),
        })
    }

    #[allow(dead_code)]
    pub fn is_valid(&self, index: usize) -> bool {
        match self {
            Self::Dictionary16(d) => d.is_valid(index),
            Self::Dictionary8(d) => d.is_valid(index),
            Self::Native(d) => d.is_valid(index),
        }
    }
}

#[allow(dead_code)]
impl<'a, V> MaybeDictArrayAccessor<'a, PrimitiveArray<V>>
where
    V: ArrowPrimitiveType,
{
    pub fn try_new(arr: &'a ArrayRef) -> Result<Self> {
        Self::try_new_with_datatype(V::DATA_TYPE, arr)
    }

    pub fn try_new_for_column(record_batch: &'a RecordBatch, column_name: &str) -> Result<Self> {
        Self::try_new(get_required_array(record_batch, column_name)?)
    }

    pub fn null_count(&self) -> usize {
        match self {
            Self::Dictionary16(d) => d.null_count(),
            Self::Dictionary8(d) => d.null_count(),
            Self::Native(n) => n.null_count(),
        }
    }
}

impl<'a> MaybeDictArrayAccessor<'a, BinaryArray> {
    pub fn try_new(arr: &'a ArrayRef) -> Result<Self> {
        Self::try_new_with_datatype(BinaryArray::DATA_TYPE, arr)
    }

    pub fn slice_at(&self, idx: usize) -> Option<&[u8]> {
        match self {
            Self::Dictionary16(dict) => dict.slice_at(idx),
            Self::Dictionary8(dict) => dict.slice_at(idx),
            Self::Native(bin_arr) => {
                if bin_arr.is_valid(idx) {
                    Some(bin_arr.value(idx))
                } else {
                    None
                }
            }
        }
    }
}

impl<'a> MaybeDictArrayAccessor<'a, FixedSizeBinaryArray> {
    pub fn try_new(arr: &'a ArrayRef, dims: i32) -> Result<Self> {
        Self::try_new_with_datatype(DataType::FixedSizeBinary(dims), arr)
    }

    pub fn slice_at(&self, idx: usize) -> Option<&[u8]> {
        match self {
            Self::Dictionary16(dict) => dict.slice_at(idx),
            Self::Dictionary8(dict) => dict.slice_at(idx),
            Self::Native(fsb_arr) => {
                if fsb_arr.is_valid(idx) {
                    Some(fsb_arr.value(idx))
                } else {
                    None
                }
            }
        }
    }
}

impl<'a> MaybeDictArrayAccessor<'a, StringArray> {
    pub fn try_new(arr: &'a ArrayRef) -> Result<Self> {
        Self::try_new_with_datatype(StringArray::DATA_TYPE, arr)
    }

    pub fn try_new_for_column(record_batch: &'a RecordBatch, column_name: &str) -> Result<Self> {
        Self::try_new(get_required_array(record_batch, column_name)?)
    }

    pub fn str_at(&self, idx: usize) -> Option<&str> {
        match self {
            Self::Dictionary16(dict) => dict.str_at(idx),
            Self::Dictionary8(dict) => dict.str_at(idx),
            Self::Native(str_arr) => {
                if str_arr.is_valid(idx) {
                    Some(str_arr.value(idx))
                } else {
                    None
                }
            }
        }
    }
}

#[allow(dead_code)]
pub type UInt32ArrayAccessor<'a> = MaybeDictArrayAccessor<'a, UInt32Array>;
pub type Int32ArrayAccessor<'a> = MaybeDictArrayAccessor<'a, Int32Array>;
pub type Int64ArrayAccessor<'a> = MaybeDictArrayAccessor<'a, Int64Array>;
pub type StringArrayAccessor<'a> = MaybeDictArrayAccessor<'a, StringArray>;

pub struct DictionaryArrayAccessor<'a, K, V>
where
    K: ArrowDictionaryKeyType,
{
    inner: &'a DictionaryArray<K>,
    value: &'a V,
}

#[allow(dead_code)]
impl<'a, K, V> DictionaryArrayAccessor<'a, K, V>
where
    K: ArrowDictionaryKeyType,
    V: Array + NullableArrayAccessor + 'static,
{
    pub fn new(dict: &'a DictionaryArray<K>) -> Result<Self> {
        let value =
            dict.values()
                .as_any()
                .downcast_ref::<V>()
                .ok_or_else(|| Error::InvalidListArray {
                    expect_oneof: Vec::new(),
                    actual: dict.values().data_type().clone(),
                })?;
        Ok(Self { inner: dict, value })
    }

    pub fn value_at(&self, idx: usize) -> Option<V::Native> {
        if self.inner.is_valid(idx) {
            let offset = self
                .inner
                .key(idx)
                .expect("dictionary should be valid at index");
            self.value.value_at(offset)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn null_count(&self) -> usize {
        self.inner.null_count()
    }

    pub fn is_valid(&self, index: usize) -> bool {
        self.inner.is_valid(index)
    }
}

impl<'a, K> DictionaryArrayAccessor<'a, K, BinaryArray>
where
    K: ArrowDictionaryKeyType,
{
    pub fn slice_at(&self, idx: usize) -> Option<&[u8]> {
        if self.inner.is_valid(idx) {
            let offset = self
                .inner
                .key(idx)
                .expect("dictionary should be valid at index");
            Some(self.value.value(offset))
        } else {
            None
        }
    }
}

impl<'a, K> DictionaryArrayAccessor<'a, K, FixedSizeBinaryArray>
where
    K: ArrowDictionaryKeyType,
{
    pub fn slice_at(&self, idx: usize) -> Option<&[u8]> {
        if self.inner.is_valid(idx) {
            let offset = self
                .inner
                .key(idx)
                .expect("dictionary should be valid at index");
            Some(self.value.value(offset))
        } else {
            None
        }
    }
}

impl<'a, K> DictionaryArrayAccessor<'a, K, StringArray>
where
    K: ArrowDictionaryKeyType,
{
    pub fn str_at(&self, idx: usize) -> Option<&str> {
        if self.inner.is_valid(idx) {
            let offset = self
                .inner
                .key(idx)
                .expect("dictionary should be valid at index");
            Some(self.value.value(offset))
        } else {
            None
        }
    }
}

/// Helper for accessing columns of a struct array
///
/// Methods return various errors into this crate's Error type if
/// if callers requirements for the struct columns are not met (for
/// example `ColumnDataTypeMismatch`)
pub struct StructColumnAccessor<'a> {
    inner: &'a StructArray,
}

impl<'a> StructColumnAccessor<'a> {
    pub fn new(arr: &'a StructArray) -> Self {
        Self { inner: arr }
    }

    pub fn get_inner_array_op(&self, column_name: &str) -> Option<&'a ArrayRef> {
        self.inner.column_by_name(column_name)
    }

    pub fn primitive_column<T: ArrowPrimitiveType + 'static>(
        &self,
        column_name: &str,
    ) -> Result<&'a PrimitiveArray<T>> {
        self.primitive_column_op(column_name)?
            .ok_or_else(|| Error::ColumnNotFound {
                name: column_name.to_string(),
            })
    }

    pub fn primitive_column_op<T: ArrowPrimitiveType + 'static>(
        &self,
        column_name: &str,
    ) -> Result<Option<&'a PrimitiveArray<T>>> {
        self.inner
            .column_by_name(column_name)
            .map(|arr| {
                arr.as_any()
                    .downcast_ref::<PrimitiveArray<T>>()
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: column_name.to_string(),
                        expect: T::DATA_TYPE,
                        actual: arr.data_type().clone(),
                    })
            })
            .transpose()
    }

    pub fn bool_column_op(&self, column_name: &str) -> Result<Option<&'a BooleanArray>> {
        self.inner
            .column_by_name(column_name)
            .map(|arr| {
                arr.as_any()
                    .downcast_ref()
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: column_name.to_string(),
                        expect: DataType::Boolean,
                        actual: arr.data_type().clone(),
                    })
            })
            .transpose()
    }

    pub fn string_column_op(&self, column_name: &str) -> Result<Option<StringArrayAccessor<'a>>> {
        self.inner
            .column_by_name(column_name)
            .map(StringArrayAccessor::try_new)
            .transpose()
    }

    pub fn byte_array_column_op(&self, column_name: &str) -> Result<Option<ByteArrayAccessor<'a>>> {
        self.inner
            .column_by_name(column_name)
            .map(ByteArrayAccessor::try_new)
            .transpose()
    }

    #[allow(dead_code)]
    pub fn int32_column_op(&self, column_name: &str) -> Result<Option<Int32ArrayAccessor<'a>>> {
        self.inner
            .column_by_name(column_name)
            .map(Int32ArrayAccessor::try_new)
            .transpose()
    }

    pub fn int64_column_op(&self, column_name: &str) -> Result<Option<Int64ArrayAccessor<'a>>> {
        self.inner
            .column_by_name(column_name)
            .map(Int64ArrayAccessor::try_new)
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{
        ArrayRef, BinaryArray, BooleanArray, DictionaryArray, Float32Array, Float64Array,
        Int8Array, Int16Array, Int32Array, Int64Array, RecordBatch, StringArray, StructArray,
        UInt8Array, UInt16Array, UInt32Array, UInt64Array,
    };
    use arrow::datatypes::{DataType, Field, Int32Type, Schema, UInt8Type, UInt16Type};
    use std::sync::Arc;

    // ---- NullableArrayAccessor trait tests ----

    #[test]
    fn primitive_array_valid_values() {
        let arr = Int32Array::from(vec![Some(1), Some(2), Some(3)]);
        assert_eq!(arr.value_at(0), Some(1));
        assert_eq!(arr.value_at(1), Some(2));
        assert_eq!(arr.value_at(2), Some(3));
    }

    #[test]
    fn primitive_array_null_values() {
        let arr = Int32Array::from(vec![Some(1), None, Some(3)]);
        assert_eq!(arr.value_at(0), Some(1));
        assert_eq!(arr.value_at(1), None);
        assert_eq!(arr.value_at(2), Some(3));
    }

    #[test]
    fn primitive_array_value_at_or_default() {
        let arr = Int32Array::from(vec![Some(42), None]);
        assert_eq!(arr.value_at_or_default(0), 42);
        assert_eq!(arr.value_at_or_default(1), 0); // i32 default
    }

    #[test]
    fn boolean_array_valid_values() {
        let arr = BooleanArray::from(vec![Some(true), Some(false), None]);
        assert_eq!(arr.value_at(0), Some(true));
        assert_eq!(arr.value_at(1), Some(false));
        assert_eq!(arr.value_at(2), None);
    }

    #[test]
    fn binary_array_valid_values() {
        let arr = BinaryArray::from(vec![Some(b"hello".as_ref()), None, Some(b"world".as_ref())]);
        assert_eq!(arr.value_at(0), Some(b"hello".to_vec()));
        assert_eq!(arr.value_at(1), None);
        assert_eq!(arr.value_at(2), Some(b"world".to_vec()));
    }

    #[test]
    fn fixed_size_binary_array_valid_values() {
        use arrow::array::FixedSizeBinaryBuilder;

        let mut builder = FixedSizeBinaryBuilder::with_capacity(3, 4);
        builder.append_value([1u8, 2, 3, 4]).unwrap();
        builder.append_null();
        builder.append_value([5u8, 6, 7, 8]).unwrap();
        let arr = builder.finish();
        assert_eq!(arr.value_at(0), Some(vec![1, 2, 3, 4]));
        assert_eq!(arr.value_at(1), None);
        assert_eq!(arr.value_at(2), Some(vec![5, 6, 7, 8]));
    }

    #[test]
    fn string_array_valid_values() {
        let arr = StringArray::from(vec![Some("hello"), None, Some("world")]);
        assert_eq!(arr.value_at(0), Some("hello".to_string()));
        assert_eq!(arr.value_at(1), None);
        assert_eq!(arr.value_at(2), Some("world".to_string()));
    }

    #[test]
    fn ref_accessor_delegates_to_inner() {
        let arr = Int32Array::from(vec![Some(42), None]);
        let r = &arr;
        assert_eq!(r.value_at(0), Some(42));
        assert_eq!(r.value_at(1), None);
    }

    #[test]
    fn option_accessor_with_some() {
        let arr = Int32Array::from(vec![Some(10), None]);
        let opt: Option<&Int32Array> = Some(&arr);
        assert_eq!(opt.value_at(0), Some(10));
        assert_eq!(opt.value_at(1), None);
    }

    #[test]
    fn option_accessor_with_none() {
        let opt: Option<Int32Array> = None;
        assert_eq!(opt.value_at(0), None);
    }

    // ---- RecordBatch helper function tests ----

    fn sample_record_batch() -> RecordBatch {
        let schema = Schema::new(vec![
            Field::new("ints", DataType::Int32, false),
            Field::new("strings", DataType::Utf8, true),
            Field::new("bools", DataType::Boolean, true),
        ]);
        RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec![Some("a"), None, Some("c")])),
                Arc::new(BooleanArray::from(vec![Some(true), Some(false), None])),
            ],
        )
        .unwrap()
    }

    #[test]
    fn get_required_array_found() {
        let rb = sample_record_batch();
        let arr = get_required_array(&rb, "ints");
        assert!(arr.is_ok());
        assert_eq!(arr.unwrap().len(), 3);
    }

    #[test]
    fn get_required_array_not_found() {
        let rb = sample_record_batch();
        let err = get_required_array(&rb, "nonexistent").unwrap_err();
        assert!(matches!(err, Error::ColumnNotFound { .. }));
    }

    #[test]
    fn get_array_op_found() {
        let rb = sample_record_batch();
        assert!(get_array_op(&rb, "ints").is_some());
    }

    #[test]
    fn get_array_op_not_found() {
        let rb = sample_record_batch();
        assert!(get_array_op(&rb, "nonexistent").is_none());
    }

    // ---- Macro-generated downcast tests ----

    #[test]
    fn get_i32_array_success() {
        let rb = sample_record_batch();
        let arr = get_i32_array(&rb, "ints").unwrap();
        assert_eq!(arr.value(0), 1);
        assert_eq!(arr.value(2), 3);
    }

    #[test]
    fn get_i32_array_type_mismatch() {
        let rb = sample_record_batch();
        let err = get_i32_array(&rb, "strings").unwrap_err();
        assert!(matches!(err, Error::ColumnDataTypeMismatch { .. }));
    }

    #[test]
    fn get_i32_array_opt_found() {
        let rb = sample_record_batch();
        let arr = get_i32_array_opt(&rb, "ints").unwrap();
        assert!(arr.is_some());
        assert_eq!(arr.unwrap().value(0), 1);
    }

    #[test]
    fn get_i32_array_opt_not_found() {
        let rb = sample_record_batch();
        let arr = get_i32_array_opt(&rb, "nonexistent").unwrap();
        assert!(arr.is_none());
    }

    #[test]
    fn get_string_array_success() {
        let rb = sample_record_batch();
        let arr = get_string_array(&rb, "strings").unwrap();
        assert_eq!(arr.value(0), "a");
    }

    #[test]
    fn get_bool_array_success() {
        let rb = sample_record_batch();
        let arr = get_bool_array(&rb, "bools").unwrap();
        assert!(arr.value(0));
        assert!(!arr.value(1));
    }

    #[test]
    fn get_bool_array_type_mismatch() {
        let rb = sample_record_batch();
        let err = get_bool_array(&rb, "ints").unwrap_err();
        assert!(matches!(err, Error::ColumnDataTypeMismatch { .. }));
    }

    fn numeric_record_batch() -> RecordBatch {
        let schema = Schema::new(vec![
            Field::new("u8s", DataType::UInt8, false),
            Field::new("u16s", DataType::UInt16, false),
            Field::new("u32s", DataType::UInt32, false),
            Field::new("u64s", DataType::UInt64, false),
            Field::new("i8s", DataType::Int8, false),
            Field::new("i16s", DataType::Int16, false),
            Field::new("i64s", DataType::Int64, false),
            Field::new("f32s", DataType::Float32, false),
            Field::new("f64s", DataType::Float64, false),
        ]);
        RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(UInt8Array::from(vec![1u8])),
                Arc::new(UInt16Array::from(vec![2u16])),
                Arc::new(UInt32Array::from(vec![3u32])),
                Arc::new(UInt64Array::from(vec![4u64])),
                Arc::new(Int8Array::from(vec![5i8])),
                Arc::new(Int16Array::from(vec![6i16])),
                Arc::new(Int64Array::from(vec![7i64])),
                Arc::new(Float32Array::from(vec![8.0f32])),
                Arc::new(Float64Array::from(vec![9.0f64])),
            ],
        )
        .unwrap()
    }

    #[test]
    fn get_u8_array_success() {
        let rb = numeric_record_batch();
        assert_eq!(get_u8_array(&rb, "u8s").unwrap().value(0), 1);
    }

    #[test]
    fn get_u16_array_success() {
        let rb = numeric_record_batch();
        assert_eq!(get_u16_array(&rb, "u16s").unwrap().value(0), 2);
    }

    #[test]
    fn get_u32_array_success() {
        let rb = numeric_record_batch();
        assert_eq!(get_u32_array(&rb, "u32s").unwrap().value(0), 3);
    }

    #[test]
    fn get_u64_array_success() {
        let rb = numeric_record_batch();
        assert_eq!(get_u64_array(&rb, "u64s").unwrap().value(0), 4);
    }

    #[test]
    fn get_i8_array_success() {
        let rb = numeric_record_batch();
        assert_eq!(get_i8_array(&rb, "i8s").unwrap().value(0), 5);
    }

    #[test]
    fn get_i16_array_success() {
        let rb = numeric_record_batch();
        assert_eq!(get_i16_array(&rb, "i16s").unwrap().value(0), 6);
    }

    #[test]
    fn get_i64_array_success() {
        let rb = numeric_record_batch();
        assert_eq!(get_i64_array(&rb, "i64s").unwrap().value(0), 7);
    }

    #[test]
    fn get_f32_array_success() {
        let rb = numeric_record_batch();
        assert_eq!(get_f32_array(&rb, "f32s").unwrap().value(0), 8.0);
    }

    #[test]
    fn get_f64_array_success() {
        let rb = numeric_record_batch();
        assert_eq!(get_f64_array(&rb, "f64s").unwrap().value(0), 9.0);
    }

    // ---- StructArray accessor tests ----

    fn struct_record_batch() -> RecordBatch {
        let inner_fields = vec![
            Field::new("x", DataType::Int32, false),
            Field::new("y", DataType::Utf8, true),
        ];
        let struct_arr = StructArray::from(vec![
            (
                Arc::new(Field::new("x", DataType::Int32, false)),
                Arc::new(Int32Array::from(vec![10, 20])) as ArrayRef,
            ),
            (
                Arc::new(Field::new("y", DataType::Utf8, true)),
                Arc::new(StringArray::from(vec![Some("hello"), None])) as ArrayRef,
            ),
        ]);

        let schema = Schema::new(vec![
            Field::new("my_struct", DataType::Struct(inner_fields.into()), false),
            Field::new("plain", DataType::Int32, false),
        ]);
        RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(struct_arr),
                Arc::new(Int32Array::from(vec![100, 200])),
            ],
        )
        .unwrap()
    }

    #[test]
    fn get_required_struct_array_success() {
        let rb = struct_record_batch();
        let sa = get_required_struct_array(&rb, "my_struct").unwrap();
        assert_eq!(sa.len(), 2);
    }

    #[test]
    fn get_required_struct_array_not_found() {
        let rb = struct_record_batch();
        let err = get_required_struct_array(&rb, "nonexistent").unwrap_err();
        assert!(matches!(err, Error::ColumnNotFound { .. }));
    }

    #[test]
    fn get_required_struct_array_type_mismatch() {
        let rb = struct_record_batch();
        let err = get_required_struct_array(&rb, "plain").unwrap_err();
        assert!(matches!(err, Error::ColumnDataTypeMismatch { .. }));
    }

    #[test]
    fn get_required_array_from_struct_array_success() {
        let rb = struct_record_batch();
        let sa = get_required_struct_array(&rb, "my_struct").unwrap();
        let arr = get_required_array_from_struct_array(sa, "x").unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn get_required_array_from_struct_array_not_found() {
        let rb = struct_record_batch();
        let sa = get_required_struct_array(&rb, "my_struct").unwrap();
        let err = get_required_array_from_struct_array(sa, "z").unwrap_err();
        assert!(matches!(err, Error::ColumnNotFound { .. }));
    }

    #[test]
    fn get_required_array_from_struct_array_from_record_batch_success() {
        let rb = struct_record_batch();
        let arr =
            get_required_array_from_struct_array_from_record_batch(&rb, "my_struct", "x").unwrap();
        let ints = arr.as_any().downcast_ref::<Int32Array>().unwrap();
        assert_eq!(ints.value(0), 10);
        assert_eq!(ints.value(1), 20);
    }

    #[test]
    fn get_required_array_from_struct_array_from_record_batch_not_found_column() {
        let rb = struct_record_batch();
        let err = get_required_array_from_struct_array_from_record_batch(&rb, "nonexistent", "x")
            .unwrap_err();
        assert!(matches!(err, Error::ColumnNotFound { .. }));
    }

    #[test]
    fn get_required_array_from_struct_array_from_record_batch_not_struct() {
        let rb = struct_record_batch();
        let err =
            get_required_array_from_struct_array_from_record_batch(&rb, "plain", "x").unwrap_err();
        assert!(matches!(err, Error::ColumnDataTypeMismatch { .. }));
    }

    #[test]
    fn get_required_array_from_struct_array_from_record_batch_inner_not_found() {
        let rb = struct_record_batch();
        let err =
            get_required_array_from_struct_array_from_record_batch(&rb, "my_struct", "nonexistent")
                .unwrap_err();
        assert!(matches!(err, Error::ColumnNotFound { .. }));
    }

    // ---- ByteArrayAccessor tests ----

    #[test]
    fn byte_array_accessor_binary() {
        let arr: ArrayRef = Arc::new(BinaryArray::from(vec![
            Some(b"ab".as_ref()),
            None,
            Some(b"cd".as_ref()),
        ]));
        let accessor = ByteArrayAccessor::try_new(&arr).unwrap();
        assert_eq!(accessor.value_at(0), Some(b"ab".to_vec()));
        assert_eq!(accessor.value_at(1), None);
        assert_eq!(accessor.value_at(2), Some(b"cd".to_vec()));
        assert_eq!(accessor.slice_at(0), Some(b"ab".as_ref()));
        assert_eq!(accessor.slice_at(1), None);
    }

    #[test]
    fn byte_array_accessor_fixed_size_binary() {
        use arrow::array::FixedSizeBinaryBuilder;

        let mut builder = FixedSizeBinaryBuilder::with_capacity(3, 2);
        builder.append_value([1u8, 2]).unwrap();
        builder.append_null();
        builder.append_value([3u8, 4]).unwrap();
        let arr: ArrayRef = Arc::new(builder.finish());
        let accessor = ByteArrayAccessor::try_new(&arr).unwrap();
        assert_eq!(accessor.value_at(0), Some(vec![1, 2]));
        assert_eq!(accessor.value_at(1), None);
        assert_eq!(accessor.slice_at(2), Some(vec![3u8, 4].as_ref()));
    }

    #[test]
    fn byte_array_accessor_unsupported_type() {
        let arr: ArrayRef = Arc::new(Int32Array::from(vec![1, 2]));
        let result = ByteArrayAccessor::try_new(&arr);
        assert!(result.is_err());
        let Err(err) = result else { unreachable!() };
        assert!(matches!(err, Error::InvalidListArray { .. }));
    }

    #[test]
    fn byte_array_accessor_dict_binary() {
        use arrow::array::BinaryDictionaryBuilder;

        let mut builder = BinaryDictionaryBuilder::<UInt8Type>::new();
        builder.append_value(b"hello");
        builder.append_value(b"hello"); // duplicate
        builder.append_value(b"world");
        let dict_arr: ArrayRef = Arc::new(builder.finish());

        let accessor = ByteArrayAccessor::try_new(&dict_arr).unwrap();
        assert_eq!(accessor.value_at(0), Some(b"hello".to_vec()));
        assert_eq!(accessor.value_at(1), Some(b"hello".to_vec()));
        assert_eq!(accessor.value_at(2), Some(b"world".to_vec()));
        assert_eq!(accessor.slice_at(0), Some(b"hello".as_ref()));
    }

    #[test]
    fn byte_array_accessor_dict_fixed_size_binary() {
        use arrow::array::FixedSizeBinaryBuilder;

        // Build a FixedSizeBinaryArray with 2-byte values as the dictionary values
        let mut fsb_builder = FixedSizeBinaryBuilder::with_capacity(2, 2);
        fsb_builder.append_value([0xAA, 0xBB]).unwrap();
        fsb_builder.append_value([0xCC, 0xDD]).unwrap();
        let values = fsb_builder.finish();

        // Keys: row 0 -> value 0, row 1 -> value 1, row 2 -> value 0 (repeat)
        let keys = UInt8Array::from(vec![0u8, 1, 0]);
        let dict = DictionaryArray::<UInt8Type>::try_new(keys, Arc::new(values)).unwrap();
        let arr: ArrayRef = Arc::new(dict);

        let accessor = ByteArrayAccessor::try_new(&arr).unwrap();
        assert_eq!(accessor.value_at(0), Some(vec![0xAA, 0xBB]));
        assert_eq!(accessor.value_at(1), Some(vec![0xCC, 0xDD]));
        assert_eq!(accessor.value_at(2), Some(vec![0xAA, 0xBB]));
        assert_eq!(accessor.slice_at(0), Some([0xAA, 0xBB].as_ref()));
        assert_eq!(accessor.slice_at(1), Some([0xCC, 0xDD].as_ref()));
    }

    #[test]
    fn byte_array_accessor_dict_unsupported_value_type() {
        // Create a dict array with String values instead of Binary
        let dict: DictionaryArray<UInt8Type> = vec!["a", "b"].into_iter().collect();
        let arr: ArrayRef = Arc::new(dict);
        let result = ByteArrayAccessor::try_new(&arr);
        assert!(result.is_err());
        let Err(err) = result else { unreachable!() };
        assert!(matches!(err, Error::UnsupportedDictionaryValueType { .. }));
    }

    // ---- MaybeDictArrayAccessor tests ----

    #[test]
    fn maybe_dict_native_string_array() {
        let arr: ArrayRef = Arc::new(StringArray::from(vec![Some("a"), None, Some("c")]));
        let accessor = StringArrayAccessor::try_new(&arr).unwrap();
        assert_eq!(accessor.value_at(0), Some("a".to_string()));
        assert_eq!(accessor.value_at(1), None);
        assert_eq!(accessor.value_at(2), Some("c".to_string()));
    }

    #[test]
    fn maybe_dict_string_str_at() {
        let arr: ArrayRef = Arc::new(StringArray::from(vec![Some("hello"), None]));
        let accessor = StringArrayAccessor::try_new(&arr).unwrap();
        assert_eq!(accessor.str_at(0), Some("hello"));
        assert_eq!(accessor.str_at(1), None);
    }

    #[test]
    fn maybe_dict_dict16_string_accessor() {
        let dict: DictionaryArray<UInt16Type> = vec!["a", "a", "b", "c"].into_iter().collect();
        let arr = Arc::new(dict) as ArrayRef;
        let accessor = StringArrayAccessor::try_new(&arr).unwrap();
        assert_eq!(accessor.value_at(0), Some("a".to_string()));
        assert_eq!(accessor.value_at(1), Some("a".to_string()));
        assert_eq!(accessor.value_at(2), Some("b".to_string()));
        assert_eq!(accessor.value_at(3), Some("c".to_string()));
        // str_at on dict
        assert_eq!(accessor.str_at(0), Some("a"));
    }

    #[test]
    fn maybe_dict_dict8_string_accessor() {
        let dict: DictionaryArray<UInt8Type> = vec!["x", "y"].into_iter().collect();
        let arr = Arc::new(dict) as ArrayRef;
        let accessor = StringArrayAccessor::try_new(&arr).unwrap();
        assert_eq!(accessor.value_at(0), Some("x".to_string()));
        assert_eq!(accessor.value_at(1), Some("y".to_string()));
    }

    #[test]
    fn maybe_dict_is_valid() {
        let arr: ArrayRef = Arc::new(StringArray::from(vec![Some("a"), None]));
        let accessor = StringArrayAccessor::try_new(&arr).unwrap();
        assert!(accessor.is_valid(0));
        assert!(!accessor.is_valid(1));
    }

    #[test]
    fn maybe_dict_unsupported_type() {
        let arr: ArrayRef = Arc::new(Int32Array::from(vec![1, 2]));
        let result = StringArrayAccessor::try_new(&arr);
        assert!(result.is_err());
        let Err(err) = result else { unreachable!() };
        assert!(matches!(err, Error::InvalidListArray { .. }));
    }

    #[test]
    fn maybe_dict_dict_wrong_value_type() {
        // Dictionary with Int32 values, but asking for String
        use arrow::array::PrimitiveDictionaryBuilder;
        let mut builder = PrimitiveDictionaryBuilder::<UInt8Type, Int32Type>::new();
        builder.append_value(42);
        let dict_arr: ArrayRef = Arc::new(builder.finish());
        let result = StringArrayAccessor::try_new(&dict_arr);
        assert!(result.is_err());
        let Err(err) = result else { unreachable!() };
        assert!(matches!(err, Error::UnsupportedDictionaryValueType { .. }));
    }

    // ---- StructColumnAccessor tests ----

    fn test_struct_array() -> StructArray {
        StructArray::from(vec![
            (
                Arc::new(Field::new("num", DataType::Int32, false)),
                Arc::new(Int32Array::from(vec![10, 20, 30])) as ArrayRef,
            ),
            (
                Arc::new(Field::new("text", DataType::Utf8, true)),
                Arc::new(StringArray::from(vec![Some("a"), None, Some("c")])) as ArrayRef,
            ),
            (
                Arc::new(Field::new("flag", DataType::Boolean, true)),
                Arc::new(BooleanArray::from(vec![Some(true), Some(false), None])) as ArrayRef,
            ),
            (
                Arc::new(Field::new("big", DataType::Int64, false)),
                Arc::new(Int64Array::from(vec![100, 200, 300])) as ArrayRef,
            ),
        ])
    }

    #[test]
    fn struct_column_accessor_get_inner_array_op_found() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        assert!(acc.get_inner_array_op("num").is_some());
    }

    #[test]
    fn struct_column_accessor_get_inner_array_op_not_found() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        assert!(acc.get_inner_array_op("nonexistent").is_none());
    }

    #[test]
    fn struct_column_accessor_primitive_column_success() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let col = acc.primitive_column::<Int32Type>("num").unwrap();
        assert_eq!(col.value(0), 10);
        assert_eq!(col.value(2), 30);
    }

    #[test]
    fn struct_column_accessor_primitive_column_not_found() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let err = acc.primitive_column::<Int32Type>("missing").unwrap_err();
        assert!(matches!(err, Error::ColumnNotFound { .. }));
    }

    #[test]
    fn struct_column_accessor_primitive_column_op_found() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let col = acc.primitive_column_op::<Int32Type>("num").unwrap();
        assert!(col.is_some());
    }

    #[test]
    fn struct_column_accessor_primitive_column_op_not_found() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let col = acc.primitive_column_op::<Int32Type>("missing").unwrap();
        assert!(col.is_none());
    }

    #[test]
    fn struct_column_accessor_primitive_column_op_type_mismatch() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        // "text" is Utf8, not Int32
        let err = acc.primitive_column_op::<Int32Type>("text").unwrap_err();
        assert!(matches!(err, Error::ColumnDataTypeMismatch { .. }));
    }

    #[test]
    fn struct_column_accessor_bool_column_op() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let col = acc.bool_column_op("flag").unwrap().unwrap();
        assert!(col.value(0));
        assert!(!col.value(1));
    }

    #[test]
    fn struct_column_accessor_bool_column_op_not_found() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let col = acc.bool_column_op("missing").unwrap();
        assert!(col.is_none());
    }

    #[test]
    fn struct_column_accessor_bool_column_op_type_mismatch() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let err = acc.bool_column_op("num").unwrap_err();
        assert!(matches!(err, Error::ColumnDataTypeMismatch { .. }));
    }

    #[test]
    fn struct_column_accessor_string_column_op() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let col = acc.string_column_op("text").unwrap().unwrap();
        assert_eq!(col.value_at(0), Some("a".to_string()));
        assert_eq!(col.value_at(1), None);
    }

    #[test]
    fn struct_column_accessor_string_column_op_not_found() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let col = acc.string_column_op("missing").unwrap();
        assert!(col.is_none());
    }

    #[test]
    fn struct_column_accessor_int64_column_op() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let col = acc.int64_column_op("big").unwrap().unwrap();
        assert_eq!(col.value_at(0), Some(100i64));
    }

    #[test]
    fn struct_column_accessor_int64_column_op_not_found() {
        let sa = test_struct_array();
        let acc = StructColumnAccessor::new(&sa);
        let col = acc.int64_column_op("missing").unwrap();
        assert!(col.is_none());
    }

    // ---- DictionaryArrayAccessor tests ----

    #[test]
    fn dictionary_array_accessor_len_and_null_count() {
        let dict: DictionaryArray<UInt16Type> = vec!["a", "b", "c"].into_iter().collect();
        let accessor = DictionaryArrayAccessor::<UInt16Type, StringArray>::new(&dict).unwrap();
        assert_eq!(accessor.len(), 3);
        assert_eq!(accessor.null_count(), 0);
    }

    #[test]
    fn dictionary_array_accessor_is_valid() {
        let dict: DictionaryArray<UInt16Type> = vec!["a", "b"].into_iter().collect();
        let accessor = DictionaryArrayAccessor::<UInt16Type, StringArray>::new(&dict).unwrap();
        assert!(accessor.is_valid(0));
        assert!(accessor.is_valid(1));
    }

    #[test]
    fn dictionary_array_accessor_null_key_returns_none() {
        use arrow::array::StringDictionaryBuilder;

        let mut builder = StringDictionaryBuilder::<UInt16Type>::new();
        builder.append_value("hello");
        builder.append_null();
        builder.append_value("world");
        let dict = builder.finish();

        let accessor = DictionaryArrayAccessor::<UInt16Type, StringArray>::new(&dict).unwrap();
        assert_eq!(accessor.value_at(0), Some("hello".to_string()));
        assert_eq!(accessor.value_at(1), None);
        assert_eq!(accessor.value_at(2), Some("world".to_string()));
        assert!(accessor.is_valid(0));
        assert!(!accessor.is_valid(1));
        assert!(accessor.is_valid(2));
        assert_eq!(accessor.len(), 3);
        assert_eq!(accessor.null_count(), 1);
    }

    #[test]
    fn maybe_dict_unsupported_dictionary_key_type() {
        // DictionaryArray with UInt32 keys -- only UInt8 and UInt16 are supported
        use arrow::datatypes::UInt32Type;

        let keys = UInt32Array::from(vec![0u32, 1]);
        let values = StringArray::from(vec!["a", "b"]);
        let dict = DictionaryArray::<UInt32Type>::try_new(keys, Arc::new(values)).unwrap();
        let arr: ArrayRef = Arc::new(dict);

        let result = StringArrayAccessor::try_new(&arr);
        assert!(result.is_err());
        let Err(err) = result else { unreachable!() };
        assert!(matches!(err, Error::UnsupportedDictionaryKeyType { .. }));
    }

    // ---- ByteArrayAccessor try_new_for_column tests ----

    #[test]
    fn byte_array_accessor_try_new_for_column_success() {
        let schema = Schema::new(vec![Field::new("data", DataType::Binary, true)]);
        let rb = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(BinaryArray::from(vec![
                Some(b"ab".as_ref()),
                Some(b"cd".as_ref()),
            ]))],
        )
        .unwrap();
        let accessor = ByteArrayAccessor::try_new_for_column(&rb, "data").unwrap();
        assert_eq!(accessor.value_at(0), Some(b"ab".to_vec()));
    }

    #[test]
    fn byte_array_accessor_try_new_for_column_not_found() {
        let schema = Schema::new(vec![Field::new("data", DataType::Binary, true)]);
        let rb = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(BinaryArray::from(vec![Some(b"ab".as_ref())]))],
        )
        .unwrap();
        let result = ByteArrayAccessor::try_new_for_column(&rb, "nonexistent");
        assert!(result.is_err());
        let Err(err) = result else { unreachable!() };
        assert!(matches!(err, Error::ColumnNotFound { .. }));
    }

    // ---- NullableInt64ArrayAccessor / NullableF64ArrayAccessor tests ----

    #[test]
    fn nullable_int64_direct() {
        let arr = Int64Array::from(vec![Some(42i64), None]);
        let r: &Int64Array = &arr;
        assert_eq!(r.i64_at(0).unwrap(), Some(42));
        assert_eq!(r.i64_at(1).unwrap(), None);
    }

    #[test]
    fn nullable_int64_dictionary() {
        use arrow::array::PrimitiveDictionaryBuilder;
        use arrow::datatypes::Int64Type;

        let mut builder = PrimitiveDictionaryBuilder::<UInt8Type, Int64Type>::new();
        builder.append_value(100);
        builder.append_value(200);
        let dict = builder.finish();
        let r: &DictionaryArray<UInt8Type> = &dict;
        assert_eq!(r.i64_at(0).unwrap(), Some(100));
        assert_eq!(r.i64_at(1).unwrap(), Some(200));
    }

    #[test]
    fn nullable_f64_direct() {
        let arr = Float64Array::from(vec![Some(3.14), None]);
        let r: &Float64Array = &arr;
        assert_eq!(r.f64_at(0).unwrap(), Some(3.14));
        assert_eq!(r.f64_at(1).unwrap(), None);
    }

    #[test]
    fn nullable_f64_dictionary() {
        use arrow::array::PrimitiveDictionaryBuilder;
        use arrow::datatypes::Float64Type;

        let mut builder = PrimitiveDictionaryBuilder::<UInt8Type, Float64Type>::new();
        builder.append_value(1.5);
        builder.append_value(2.5);
        let dict = builder.finish();
        let r: &DictionaryArray<UInt8Type> = &dict;
        assert_eq!(r.f64_at(0).unwrap(), Some(1.5));
        assert_eq!(r.f64_at(1).unwrap(), Some(2.5));
    }

    // ---- Original test preserved ----

    #[test]
    fn test_dictionary_accessor() {
        let expected: DictionaryArray<UInt16Type> = vec!["a", "a", "b", "c"].into_iter().collect();
        let dict = Arc::new(expected) as ArrayRef;
        let accessor = StringArrayAccessor::try_new(&dict).unwrap();
        assert_eq!("a", accessor.value_at(0).unwrap());
        assert_eq!("a", accessor.value_at(1).unwrap());
        assert_eq!("b", accessor.value_at(2).unwrap());
        assert_eq!("c", accessor.value_at(3).unwrap());
    }
}
