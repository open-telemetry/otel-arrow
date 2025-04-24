// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error;
use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, BinaryArray, BooleanArray, DictionaryArray, Float32Array,
    Float64Array, Int16Array, Int32Array, Int64Array, Int8Array, PrimitiveArray, RecordBatch,
    StringArray, TimestampNanosecondArray, UInt16Array, UInt32Array, UInt64Array, UInt8Array,
};
use arrow::datatypes::{ArrowDictionaryKeyType, TimeUnit};
use arrow::datatypes::{ArrowNativeType, DataType, UInt16Type, UInt8Type};
use paste::paste;
use snafu::{ensure, OptionExt};

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
            pub fn [<get_ $suffix _array_opt> ]<'a>(rb: &'a RecordBatch, name: &str) -> error::Result<Option<&'a $array_type>> {
                use arrow::datatypes::DataType::*;
                rb.column_by_name(name)
                    .map(|arr|{
                        arr.as_any()
                            .downcast_ref::<$array_type>()
                            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                                name,
                                expect: $data_type,
                                actual: arr.data_type().clone(),
                            })
                }).transpose()
            }

              pub fn [<get_ $suffix _array> ]<'a>(rb: &'a RecordBatch, name: &str) -> error::Result<&'a $array_type> {
                use arrow::datatypes::DataType::*;
                let arr = rb.column_by_name(name).context(error::ColumnNotFoundSnafu {
            name,
        })?;

                 arr.as_any()
                            .downcast_ref::<$array_type>()
                            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                                name,
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

trait NullableInt64ArrayAccessor {
    fn i64_at(&self, idx: usize) -> error::Result<Option<i64>>;
}

impl NullableInt64ArrayAccessor for &Int64Array {
    fn i64_at(&self, idx: usize) -> error::Result<Option<i64>> {
        Ok(self.value_at(idx))
    }
}

impl<T> NullableInt64ArrayAccessor for &DictionaryArray<T>
where
    T: ArrowDictionaryKeyType,
{
    fn i64_at(&self, idx: usize) -> error::Result<Option<i64>> {
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

trait NullableF64ArrayAccessor {
    fn f64_at(&self, idx: usize) -> error::Result<Option<f64>>;
}

impl NullableF64ArrayAccessor for &Float64Array {
    fn f64_at(&self, idx: usize) -> error::Result<Option<f64>> {
        Ok(self.value_at(idx))
    }
}

impl<T> NullableF64ArrayAccessor for &DictionaryArray<T>
where
    T: ArrowDictionaryKeyType,
{
    fn f64_at(&self, idx: usize) -> error::Result<Option<f64>> {
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

pub type DictionaryStringArrayAccessor<'a, K> = DictionaryArrayAccessor<'a, K, StringArray>;

/// [StringArrayAccessor] allows to access string values from [StringArray]s and [DictionaryArray]s.
pub enum StringArrayAccessor<'a> {
    /// Plain StringArray
    String(&'a StringArray),
    /// DictionaryArray with UInt8 keys and String values.
    Dictionary8(DictionaryStringArrayAccessor<'a, UInt8Type>),
    /// DictionaryArray with UInt16 keys and String values.
    Dictionary16(DictionaryStringArrayAccessor<'a, UInt16Type>),
}

impl NullableArrayAccessor for StringArrayAccessor<'_> {
    type Native = String;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        match self {
            StringArrayAccessor::String(s) => s.value_at(idx),
            StringArrayAccessor::Dictionary8(d) => d.value_at(idx),
            StringArrayAccessor::Dictionary16(d) => d.value_at(idx),
        }
    }
}

impl<'a> StringArrayAccessor<'a> {
    pub fn new(a: &'a ArrayRef) -> error::Result<Self> {
        let result = match a.data_type() {
            DataType::Utf8 => {
                // safety: we've checked array data type
                Self::String(a.as_any().downcast_ref::<StringArray>().unwrap())
            }
            DataType::Dictionary(key, v) => {
                ensure!(
                    **v == DataType::Utf8,
                    error::UnsupportedStringColumnTypeSnafu {
                        data_type: (**v).clone()
                    }
                );
                match **key {
                    DataType::UInt8 => Self::Dictionary8(DictionaryArrayAccessor::new(
                        // safety: we've checked the key type
                        a.as_any()
                            .downcast_ref::<DictionaryArray<UInt8Type>>()
                            .unwrap(),
                    )),
                    DataType::UInt16 => Self::Dictionary16(DictionaryArrayAccessor::new(
                        // safety: we've checked the key type
                        a.as_any()
                            .downcast_ref::<DictionaryArray<UInt16Type>>()
                            .unwrap(),
                    )),
                    _ => {
                        return error::UnsupportedStringDictKeyTypeSnafu {
                            data_type: a.data_type().clone(),
                        }
                        .fail()
                    }
                }
            }
            _ => {
                return error::UnsupportedStringColumnTypeSnafu {
                    data_type: a.data_type().clone(),
                }
                .fail()
            }
        };
        Ok(result)
    }
}

pub struct DictionaryArrayAccessor<'a, K, V>
where
    K: ArrowDictionaryKeyType,
{
    inner: &'a DictionaryArray<K>,
    value: &'a V,
}

impl<'a, K, V> DictionaryArrayAccessor<'a, K, V>
where
    K: ArrowDictionaryKeyType,
    V: Array + NullableArrayAccessor + 'static,
{
    pub fn new(a: &'a DictionaryArray<K>) -> Self {
        let dict = a.as_any().downcast_ref::<DictionaryArray<K>>().unwrap();
        let value = dict.values().as_any().downcast_ref::<V>().unwrap();
        Self { inner: dict, value }
    }

    pub fn value_at(&self, idx: usize) -> Option<V::Native> {
        let offset = self.inner.key(idx).unwrap();
        self.value.value_at(offset)
    }
}

#[cfg(test)]
mod tests {
    use crate::arrays::{NullableArrayAccessor, StringArrayAccessor};
    use arrow::array::{ArrayRef, DictionaryArray};
    use arrow::datatypes::UInt16Type;
    use std::sync::Arc;

    #[test]
    fn test_dictionary_accessor() {
        let expected: DictionaryArray<UInt16Type> = vec!["a", "a", "b", "c"].into_iter().collect();
        let dict = Arc::new(expected) as ArrayRef;
        let accessor = StringArrayAccessor::new(&dict).unwrap();
        assert_eq!("a", accessor.value_at(0).unwrap());
        assert_eq!("a", accessor.value_at(1).unwrap());
        assert_eq!("b", accessor.value_at(2).unwrap());
        assert_eq!("c", accessor.value_at(3).unwrap());
    }
}
