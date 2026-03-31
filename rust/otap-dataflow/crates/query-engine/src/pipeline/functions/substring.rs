// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayIter, ArrayRef, AsArray, DictionaryArray, Int64Array, StringArray, StringBuilder,
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

/// Replacement for datafusion's [`SubstrFunc`](datafusion::functions::unicode::substr::SubstrFunc)
/// with a few behaviour changes:
/// - can operate directly on Dictionary values if the passed source is dictionary encoded
/// - returns a `StringArray` instead of a `StringViewArray`. We do this because, OTAP data model
///   does not support string views.
/// - the substring start index is 0-based, whereas datafusion's implementation is 1-based (like in
///   PostgresSQL)
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
            match (source, start, len) {
                (Some(source), Some(start), Some(len)) => {
                    let (start, end) = get_true_start_end(
                        source,
                        // + 1 offset b/c get_true_start_end is indexed starting from 1-based offset
                        // (like in PostgreSQL):
                        start + 1,
                        Some(len as u64),
                        enable_ascii_fast_path,
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
            match (source, start) {
                (Some(source), Some(start)) => {
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

struct SubstrIndexArgIter {
    index: usize,
    source: SubstringIndexArgIterSource,
}

impl SubstrIndexArgIter {
    fn try_new(val: &ColumnarValue) -> Result<Self> {
        Ok(Self {
            index: 0,
            source: val.try_into()?,
        })
    }
}

impl Iterator for SubstrIndexArgIter {
    type Item = Option<i64>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.source {
            SubstringIndexArgIterSource::Scalar(val) => Some(Some(*val)),
            SubstringIndexArgIterSource::Array(arr) => {
                if self.index < arr.len() {
                    let val = arr.is_valid(self.index).then(|| arr.value(self.index));
                    self.index += 1;
                    Some(val)
                } else {
                    None
                }
            }
        }
    }
}

enum SubstringIndexArgIterSource {
    Scalar(i64),
    Array(Int64Array),
}

impl TryFrom<&ColumnarValue> for SubstringIndexArgIterSource {
    type Error = DataFusionError;

    fn try_from(value: &ColumnarValue) -> Result<Self> {
        match value {
            ColumnarValue::Scalar(s) => match s.cast_to(&DataType::Int64) {
                Ok(ScalarValue::Int64(i)) => Ok(Self::Scalar(i.unwrap_or_default())),
                Err(e) => Err(e),
                _ => unreachable!("expected cast to produce i64 or error"),
            },
            ColumnarValue::Array(arr) => {
                let index_arr = arr.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                    DataFusionError::Execution(format!(
                        "Expected range index type int64, found {:?}",
                        arr.data_type()
                    ))
                })?;
                Ok(Self::Array(index_arr.clone()))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use arrow::{
        array::PrimitiveArray,
        datatypes::{ArrowNativeType, Field},
    };
    use datafusion::config::ConfigOptions;

    use super::*;

    fn invoke_substring_expr(
        input: ColumnarValue,
        start: ColumnarValue,
        len: Option<ColumnarValue>,
    ) -> Result<ColumnarValue> {
        let func = SubstringFunc::new();

        let number_rows = match &input {
            ColumnarValue::Array(arr) => arr.len(),
            _ => 1,
        };

        let mut args = vec![input, start];
        if let Some(len) = len {
            args.push(len)
        };

        func.invoke_with_args(ScalarFunctionArgs {
            args,
            number_rows,
            // fields below are placeholders as they're not used by the function impl
            arg_fields: Vec::new(),
            return_field: Arc::new(Field::new("", DataType::Utf8, true)),
            config_options: Arc::new(ConfigOptions::default()),
        })
    }

    #[test]
    fn test_substring_with_scalar_start() {
        let source = ColumnarValue::Array(Arc::new(StringArray::from_iter_values([
            "arrow",
            "substring",
            "func",
        ])));

        let start = ColumnarValue::Scalar(ScalarValue::Int64(Some(3)));

        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["ow", "string", "c"]));

        match invoke_substring_expr(source, start, None) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_with_scalar_start_and_length() {
        let source = ColumnarValue::Array(Arc::new(StringArray::from_iter_values([
            "arrow",
            "substring",
            "func",
        ])));

        let start = ColumnarValue::Scalar(ScalarValue::Int64(Some(0)));
        let end = ColumnarValue::Scalar(ScalarValue::Int64(Some(3)));

        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["arr", "sub", "fun"]));

        match invoke_substring_expr(source, start, Some(end)) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_with_arrray_start() {
        let source = ColumnarValue::Array(Arc::new(StringArray::from_iter_values([
            "arrow",
            "substring",
            "func",
        ])));

        let start = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([0, 1, 3])));

        let expected: ArrayRef =
            Arc::new(StringArray::from_iter_values(["arrow", "ubstring", "c"]));

        match invoke_substring_expr(source, start, None) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_with_array_start_and_array_len() {
        let source = ColumnarValue::Array(Arc::new(StringArray::from_iter_values([
            "arrow",
            "substring",
            "func",
        ])));

        let start = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([0, 1, 0])));
        let len = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([1, 3, 2])));

        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["a", "ubs", "fu"]));

        match invoke_substring_expr(source, start, Some(len)) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_with_out_of_bounds_indices() {
        let source = ColumnarValue::Array(Arc::new(StringArray::from_iter_values([
            "arrow",
            "substring",
        ])));

        let start = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([20, 1])));
        let len = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([1, 20])));

        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["", "ubstring"]));

        match invoke_substring_expr(source, start, Some(len)) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_with_start_array_shorter_than_source() {
        let source = ColumnarValue::Array(Arc::new(StringArray::from_iter_values([
            "arrow",
            "substring",
            "func",
        ])));

        let start = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([1, 2])));
        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["rrow", "bstring"]));
        match invoke_substring_expr(source, start, None) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_with_start_array_too_longer_than_source() {
        let source = ColumnarValue::Array(Arc::new(StringArray::from_iter_values([
            "arrow",
            "substring",
            "func",
        ])));

        let start = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([1, 2, 3, 4])));
        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["rrow", "bstring", "c"]));

        match invoke_substring_expr(source, start, None) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_with_length_array_shorter_than_source() {
        let source = ColumnarValue::Array(Arc::new(StringArray::from_iter_values([
            "arrow",
            "substring",
            "func",
        ])));

        let start = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([1, 1, 1])));
        let len = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([1, 2])));
        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["r", "ub"]));
        match invoke_substring_expr(source, start, Some(len)) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_with_length_array_too_longer_than_source() {
        let source = ColumnarValue::Array(Arc::new(StringArray::from_iter_values([
            "arrow",
            "substring",
            "func",
        ])));

        let start = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([1, 1, 1])));
        let len = ColumnarValue::Array(Arc::new(Int64Array::from_iter_values([1, 2, 3, 4])));
        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["r", "ub", "unc"]));

        match invoke_substring_expr(source, start, Some(len)) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    fn create_dict_arr<K: ArrowDictionaryKeyType>(
        dict_keys: Vec<usize>,
        dict_values: StringArray,
    ) -> DictionaryArray<K> {
        let dict_keys = dict_keys
            .into_iter()
            .map(|k| K::Native::from_usize(k).unwrap());
        DictionaryArray::new(
            PrimitiveArray::<K>::from_iter_values(dict_keys),
            Arc::new(dict_values),
        )
    }

    fn test_substring_dict_values_start_scalar_generic<K: ArrowDictionaryKeyType>() {
        let dict_keys = vec![0, 0, 1];
        let dict_values = StringArray::from_iter_values(["arrow", "func"]);
        let source = ColumnarValue::Array(Arc::new(create_dict_arr::<K>(dict_keys, dict_values)));
        let start = ColumnarValue::Scalar(ScalarValue::Int64(Some(1)));

        let expected = Arc::new(create_dict_arr::<K>(
            vec![0, 0, 1],
            StringArray::from_iter_values(["rrow", "unc"]),
        ));

        match invoke_substring_expr(source, start, None) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_dict_values_start_scalar() {
        test_substring_dict_values_start_scalar_generic::<Int8Type>();
        test_substring_dict_values_start_scalar_generic::<Int16Type>();
        test_substring_dict_values_start_scalar_generic::<Int32Type>();
        test_substring_dict_values_start_scalar_generic::<Int64Type>();
        test_substring_dict_values_start_scalar_generic::<UInt8Type>();
        test_substring_dict_values_start_scalar_generic::<UInt16Type>();
        test_substring_dict_values_start_scalar_generic::<UInt32Type>();
        test_substring_dict_values_start_scalar_generic::<UInt64Type>();
    }

    fn test_substring_dict_values_start_and_len_scalar_generic<K: ArrowDictionaryKeyType>() {
        let dict_keys = vec![0, 0, 1];
        let dict_values = StringArray::from_iter_values(["arrow", "func"]);
        let source = ColumnarValue::Array(Arc::new(create_dict_arr::<K>(dict_keys, dict_values)));
        let start = ColumnarValue::Scalar(ScalarValue::Int64(Some(1)));
        let len = ColumnarValue::Scalar(ScalarValue::Int64(Some(2)));

        let expected = Arc::new(create_dict_arr::<K>(
            vec![0, 0, 1],
            StringArray::from_iter_values(["rr", "un"]),
        ));

        match invoke_substring_expr(source, start, Some(len)) {
            Ok(ColumnarValue::Array(arr)) => assert_eq!(arr.as_ref(), expected.as_ref()),
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn test_substring_dict_values_start_and_len_scalar() {
        test_substring_dict_values_start_and_len_scalar_generic::<Int8Type>();
        test_substring_dict_values_start_and_len_scalar_generic::<Int16Type>();
        test_substring_dict_values_start_and_len_scalar_generic::<Int32Type>();
        test_substring_dict_values_start_and_len_scalar_generic::<Int64Type>();
        test_substring_dict_values_start_and_len_scalar_generic::<UInt8Type>();
        test_substring_dict_values_start_and_len_scalar_generic::<UInt16Type>();
        test_substring_dict_values_start_and_len_scalar_generic::<UInt32Type>();
        test_substring_dict_values_start_and_len_scalar_generic::<UInt64Type>();
    }

    // TODO test when input is a scalar
}
