// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayIter, ArrayRef, AsArray, DictionaryArray, Int64Array, MutableArrayData,
    PrimitiveArray, StringArray, StringBuilder,
};
use arrow::datatypes::{
    ArrowDictionaryKeyType, DataType, Int8Type, Int16Type, Int32Type, Int64Type, UInt8Type,
    UInt16Type, UInt32Type, UInt64Type,
};
use datafusion::common::exec_err;
use datafusion::error::{DataFusionError, Result};
use datafusion::functions::unicode::substr::get_true_start_end;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ScalarFunctionArgs, ScalarUDFImpl, Signature,
};
use datafusion::scalar::ScalarValue;

/// TODO comments
#[derive(Default, Debug, PartialEq, Eq, Hash)]
pub struct SubstringFunc {
    inner: datafusion::functions::unicode::substr::SubstrFunc,
}

impl SubstringFunc {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ScalarUDFImpl for SubstringFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "substring"
    }

    fn signature(&self) -> &Signature {
        self.inner.signature()
    }

    fn return_type(&self, data_types: &[DataType]) -> Result<DataType> {
        if let Some(DataType::Dictionary(k, _)) = data_types.get(0) {
            Ok(DataType::Dictionary(
                Box::new(k.as_ref().clone()),
                Box::new(DataType::Utf8),
            ))
        } else {
            Ok(DataType::Utf8)
        }
    }

    fn invoke_with_args(&self, scalar_func_args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let args = scalar_func_args.args;
        if args.len() < 2 || args.len() > 3 {
            return exec_err!(
                "invalid number of args. received {} expected 2 or 3",
                args.len()
            );
        }

        let mut args_iter = args.into_iter();
        let source_arg = args_iter.next().expect("there are at least two args");
        let start_arg = args_iter.next().expect("there are at least two args");
        let len_arg = args_iter.next();

        let source_arr = match source_arg {
            ColumnarValue::Array(arr) => arr,
            ColumnarValue::Scalar(s) => s.to_array()?,
        };

        substr(source_arr, start_arg, len_arg).map(ColumnarValue::Array)
    }

    fn documentation(&self) -> Option<&Documentation> {
        None
    }
}

fn substr(
    source_arr: ArrayRef,
    start_arg: ColumnarValue,
    len_arg: Option<ColumnarValue>,
) -> Result<ArrayRef> {
    match source_arr.data_type() {
        DataType::Dictionary(k, _) => match k.as_ref() {
            DataType::UInt8 => dict_substr::<UInt8Type>(source_arr, start_arg, len_arg),
            DataType::UInt16 => dict_substr::<UInt16Type>(source_arr, start_arg, len_arg),
            DataType::UInt32 => dict_substr::<UInt32Type>(source_arr, start_arg, len_arg),
            DataType::UInt64 => dict_substr::<UInt64Type>(source_arr, start_arg, len_arg),
            DataType::Int8 => dict_substr::<Int8Type>(source_arr, start_arg, len_arg),
            DataType::Int16 => dict_substr::<Int16Type>(source_arr, start_arg, len_arg),
            DataType::Int32 => dict_substr::<Int32Type>(source_arr, start_arg, len_arg),
            DataType::Int64 => dict_substr::<Int64Type>(source_arr, start_arg, len_arg),
            _ => unreachable!("unexpected dict key type"),
        },
        DataType::Utf8 => {
            let string_arr = source_arr.as_string();
            string_substr(string_arr, start_arg, len_arg)
        }
        other => {
            exec_err!(
                "Unsupported data type {other:?} for function substring,\
                    expected Utf8 or Dict<_, Utf8>."
            )
        }
    }
}

fn dict_substr<K: ArrowDictionaryKeyType>(
    source_arr: ArrayRef,
    start_arg: ColumnarValue,
    len_arg: Option<ColumnarValue>,
) -> Result<ArrayRef> {
    let dict_arr = source_arr
        .as_any()
        .downcast_ref::<DictionaryArray<K>>()
        .expect("expected caller to check type");

    let new_vals = substr(Arc::clone(dict_arr.values()), start_arg, len_arg)?;

    Ok(Arc::new(DictionaryArray::new(
        dict_arr.keys().clone(),
        new_vals,
    )))
}

fn string_substr(
    string_arr: &StringArray,
    start_arg: ColumnarValue,
    len_arg: Option<ColumnarValue>,
) -> Result<ArrayRef> {
    let enable_ascii_fast_path = string_arr.is_ascii();

    let source_iter = ArrayIter::new(string_arr);
    let start_iter = SubstrIndexArgIter::try_new(&start_arg)?;
    let mut result_builder = StringBuilder::new();

    if let Some(len_arg) = len_arg {
        let len_iter = SubstrIndexArgIter::try_new(&len_arg)?;
        for ((source, start), len) in source_iter.zip(start_iter).zip(len_iter) {
            match source {
                Some(source) => {
                    println!("input = source {source}, start: {start}, len: {len}");
                    let (start, end) = get_true_start_end(
                        source,
                        // + 1 offset b/c get_true_start_end is indexed starting from 1-based offset
                        // (like in PostgreSQL):
                        start + 1,
                        Some(len as u64),
                        enable_ascii_fast_path,
                    );
                    println!(
                        "output = source {}, start: {start}, end: {end}",
                        &source[start..end]
                    );
                    result_builder.append_value(&source[start..end])
                }
                _ => {
                    result_builder.append_null();
                }
            }
        }
    } else {
        for (source, start) in source_iter.zip(start_iter) {
            match source {
                Some(source) => {
                    let (start, end) = get_true_start_end(
                        source,
                        // + 1 offset b/c get_true_start_end is indexed starting from 1-based offset
                        // (like in PostgreSQL):
                        start + 1,
                        None,
                        enable_ascii_fast_path,
                    );
                    result_builder.append_value(&source[start..end])
                }
                _ => {
                    result_builder.append_null();
                }
            }
        }
    }

    let result = result_builder.finish();
    Ok(Arc::new(result))
}

struct SubstrIndexArgIter<'a> {
    source: SubstringIndexArgIterSource<'a>,
}

impl<'a> SubstrIndexArgIter<'a> {
    fn try_new(val: &'a ColumnarValue) -> Result<Self> {
        Ok(Self {
            source: val.try_into()?,
        })
    }
}

impl<'a> Iterator for SubstrIndexArgIter<'a> {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source {
            SubstringIndexArgIterSource::Scalar(val) => Some(val),
            SubstringIndexArgIterSource::Array(arr) => {
                todo!()
            }
        }
    }
}

enum SubstringIndexArgIterSource<'a> {
    Scalar(i64),
    Array(&'a Int64Array),
}

impl<'a> TryFrom<&ColumnarValue> for SubstringIndexArgIterSource<'a> {
    type Error = DataFusionError;

    fn try_from(value: &ColumnarValue) -> Result<Self> {
        match value {
            ColumnarValue::Scalar(s) => match s.cast_to(&DataType::Int64) {
                Ok(ScalarValue::Int64(i)) => Ok(Self::Scalar(i.unwrap_or_default())),
                Err(e) => Err(e),
                _ => unreachable!("expected cast to produce i64 or error"),
            },
            ColumnarValue::Array(a) => {
                todo!()
            }
        }
    }
}
