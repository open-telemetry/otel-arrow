// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;

use arrow::array::BooleanArray;
use arrow::buffer::BooleanBuffer;
use arrow::datatypes::{DataType, TimeUnit};
use datafusion::common::exec_err;
use datafusion::error::Result;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
};
use datafusion::scalar::ScalarValue;

use crate::pipeline::expr::types::ExprLogicalType;

/// This function implements the runtime check that some function is the expected logical type.
///
/// The intention is that if the arg is a [`ColumnarValue`] of some concrete type, and we expect
/// some logical type, this will return Scalar boolean value indicating if the concrete type is
/// a representation of the logical type.
///
/// For example, the logical type [`ExprLogicalTypeUtf8`] may be represented by concrete types
/// of `DataType::Utf8`, but also a dictionary where the keys are `DataType::Utf8`.
///
/// When the logical type is something like [`ExprLogicalType::AnyInt`], any integer data type
/// will return true (`UInt[8-64]`, `Int[8-64]`) as well as any dictionary data type where the
/// dictionary values are also integers.
///
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct IsTypeFunc {
    expected_type: ExprLogicalType,
    signature: Signature,
}

impl IsTypeFunc {
    pub fn new(expected_type: ExprLogicalType) -> Self {
        Self {
            expected_type,
            signature: Signature::any(1, Volatility::Immutable),
        }
    }
}

impl ScalarUDFImpl for IsTypeFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "is_type"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Boolean)
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let args = args.args;
        if args.len() != 1 {
            return exec_err!(
                "invalid number of args. {} expected 1 arg, found {}",
                self.name(),
                args.len()
            );
        }

        let arg_data_type = args[0].data_type();

        let type_matches = match self.expected_type {
            ExprLogicalType::Boolean => arg_data_type == DataType::Boolean,
            ExprLogicalType::Float64 => is_logically_type(&arg_data_type, &DataType::Float64),
            ExprLogicalType::DurationNanoSecond => {
                is_logically_type(&arg_data_type, &DataType::Duration(TimeUnit::Nanosecond))
            }
            ExprLogicalType::TimestampNanosecond => is_logically_type(
                &arg_data_type,
                &DataType::Timestamp(TimeUnit::Nanosecond, None),
            ),
            ExprLogicalType::String => is_logically_type(&arg_data_type, &DataType::Utf8),
            ExprLogicalType::Binary => is_logically_type(&arg_data_type, &DataType::Binary),
            ExprLogicalType::FixedSizeBinary(size) => {
                is_logically_type(&arg_data_type, &DataType::FixedSizeBinary(size as i32))
            }
            ExprLogicalType::Int32 => is_logically_type(&arg_data_type, &DataType::Int32),
            ExprLogicalType::Int64 => is_logically_type(&arg_data_type, &DataType::Int64),
            ExprLogicalType::UInt8 => is_logically_type(&arg_data_type, &DataType::UInt8),
            ExprLogicalType::UInt32 => is_logically_type(&arg_data_type, &DataType::UInt32),
            ExprLogicalType::AnyInt => {
                if arg_data_type.is_integer() {
                    true
                } else if let DataType::Dictionary(_, v) = &arg_data_type {
                    v.as_ref().is_integer()
                } else {
                    false
                }
            }
            _ => {
                todo!()
            }
        };

        // TODO ugly - and I _think_ we should have the liberty to return a scalar here ....
        let result = match &args[0] {
            ColumnarValue::Scalar(_) => {
                ColumnarValue::Scalar(ScalarValue::Boolean(Some(type_matches)))
            }
            ColumnarValue::Array(arr) => {
                ColumnarValue::Array(std::sync::Arc::new(BooleanArray::new(
                    if type_matches {
                        BooleanBuffer::new_set(arr.len())
                    } else {
                        BooleanBuffer::new_unset(arr.len())
                    },
                    None,
                )))
            }
        };

        Ok(result)
    }

    fn documentation(&self) -> Option<&Documentation> {
        None
    }
}

fn is_logically_type(source: &DataType, expected: &DataType) -> bool {
    if source == expected {
        true
    } else if let DataType::Dictionary(_, v) = source {
        v.as_ref() == expected
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{ArrayRef, Int32Array, StringArray, UInt8Array};
    use arrow::datatypes::{DataType, Field, TimeUnit};
    use datafusion::config::ConfigOptions;
    use datafusion::logical_expr::{ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl};
    use datafusion::scalar::ScalarValue;
    use std::sync::Arc;

    // ─── helpers ──────────────────────────────────────────────────────

    /// Invoke `IsTypeFunc` with a single scalar `ColumnarValue` and return the boolean result.
    fn invoke_scalar(expected_type: ExprLogicalType, arg: ColumnarValue) -> Result<bool> {
        let func = IsTypeFunc::new(expected_type);
        let result = func.invoke_with_args(ScalarFunctionArgs {
            args: vec![arg],
            arg_fields: Vec::new(),
            number_rows: 1,
            return_field: Arc::new(Field::new("", DataType::Boolean, false)),
            config_options: Arc::new(ConfigOptions::default()),
        })?;
        match result {
            ColumnarValue::Scalar(ScalarValue::Boolean(Some(v))) => Ok(v),
            other => panic!("expected Scalar(Boolean(Some(_))), got {:?}", other),
        }
    }

    /// Invoke with a single-element array column instead of a scalar.
    fn invoke_array(expected_type: ExprLogicalType, array: ArrayRef) -> Result<bool> {
        invoke_scalar(expected_type, ColumnarValue::Array(array))
    }

    // ─── ScalarUDFImpl trait basics ──────────────────────────────────

    #[test]
    fn test_name() {
        let func = IsTypeFunc::new(ExprLogicalType::Boolean);
        assert_eq!(func.name(), "is_type");
    }

    #[test]
    fn test_return_type_is_boolean() {
        let func = IsTypeFunc::new(ExprLogicalType::Boolean);
        let ret = func.return_type(&[DataType::Utf8]).unwrap();
        assert_eq!(ret, DataType::Boolean);
    }

    #[test]
    fn test_signature_accepts_any_single_arg() {
        let func = IsTypeFunc::new(ExprLogicalType::Boolean);
        let sig = func.signature();
        assert_eq!(sig.volatility, Volatility::Immutable);
        // Signature::any(1, _) accepts exactly 1 argument of any type
    }

    #[test]
    fn test_documentation_is_none() {
        let func = IsTypeFunc::new(ExprLogicalType::Boolean);
        assert!(func.documentation().is_none());
    }

    // ─── wrong arg count ─────────────────────────────────────────────

    #[test]
    fn test_zero_args_returns_error() {
        let func = IsTypeFunc::new(ExprLogicalType::Boolean);
        let result = func.invoke_with_args(ScalarFunctionArgs {
            args: vec![],
            arg_fields: vec![],
            number_rows: 1,
            return_field: Arc::new(Field::new("", DataType::Boolean, false)),
            config_options: Arc::new(ConfigOptions::default()),
        });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("expected 1 arg"), "error: {msg}");
    }

    #[test]
    fn test_two_args_returns_error() {
        let func = IsTypeFunc::new(ExprLogicalType::Boolean);
        let result = func.invoke_with_args(ScalarFunctionArgs {
            args: vec![
                ColumnarValue::Scalar(ScalarValue::Boolean(Some(true))),
                ColumnarValue::Scalar(ScalarValue::Boolean(Some(false))),
            ],
            arg_fields: Vec::new(),
            number_rows: 1,
            return_field: Arc::new(Field::new("", DataType::Boolean, false)),
            config_options: Arc::new(ConfigOptions::default()),
        });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("found 2"), "error: {msg}");
    }

    // ─── Boolean ─────────────────────────────────────────────────────

    #[test]
    fn test_boolean_matches_boolean() {
        let v = invoke_scalar(
            ExprLogicalType::Boolean,
            ColumnarValue::Scalar(ScalarValue::Boolean(Some(true))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_boolean_rejects_int32() {
        let v = invoke_scalar(
            ExprLogicalType::Boolean,
            ColumnarValue::Scalar(ScalarValue::Int32(Some(1))),
        )
        .unwrap();
        assert!(!v);
    }

    // ─── Float64 ─────────────────────────────────────────────────────

    #[test]
    fn test_float64_matches_float64() {
        let v = invoke_scalar(
            ExprLogicalType::Float64,
            ColumnarValue::Scalar(ScalarValue::Float64(Some(1.0))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_float64_rejects_utf8() {
        let v = invoke_scalar(
            ExprLogicalType::Float64,
            ColumnarValue::Scalar(ScalarValue::Utf8(Some("hello".into()))),
        )
        .unwrap();
        assert!(!v);
    }

    #[test]
    fn test_float64_matches_dict_float64() {
        // Dictionary with Float64 values should match
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int32),
            Box::new(ScalarValue::Float64(Some(5.14))),
        );
        let v = invoke_scalar(ExprLogicalType::Float64, ColumnarValue::Scalar(dict)).unwrap();
        assert!(v);
    }

    #[test]
    fn test_float64_rejects_dict_utf8() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int32),
            Box::new(ScalarValue::Utf8(Some("hello".into()))),
        );
        let v = invoke_scalar(ExprLogicalType::Float64, ColumnarValue::Scalar(dict)).unwrap();
        assert!(!v);
    }

    // ─── String (Utf8) ───────────────────────────────────────────────

    #[test]
    fn test_string_matches_utf8() {
        let v = invoke_scalar(
            ExprLogicalType::String,
            ColumnarValue::Scalar(ScalarValue::Utf8(Some("test".into()))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_string_rejects_int64() {
        let v = invoke_scalar(
            ExprLogicalType::String,
            ColumnarValue::Scalar(ScalarValue::Int64(Some(42))),
        )
        .unwrap();
        assert!(!v);
    }

    #[test]
    fn test_string_matches_dict_utf8() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int32),
            Box::new(ScalarValue::Utf8(Some("hello".into()))),
        );
        let v = invoke_scalar(ExprLogicalType::String, ColumnarValue::Scalar(dict)).unwrap();
        assert!(v);
    }

    // ─── Int32 ───────────────────────────────────────────────────────

    #[test]
    fn test_int32_matches_int32() {
        let v = invoke_scalar(
            ExprLogicalType::Int32,
            ColumnarValue::Scalar(ScalarValue::Int32(Some(42))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_int32_rejects_int64() {
        let v = invoke_scalar(
            ExprLogicalType::Int32,
            ColumnarValue::Scalar(ScalarValue::Int64(Some(42))),
        )
        .unwrap();
        assert!(!v);
    }

    #[test]
    fn test_int32_matches_dict_int32() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int8),
            Box::new(ScalarValue::Int32(Some(7))),
        );
        let v = invoke_scalar(ExprLogicalType::Int32, ColumnarValue::Scalar(dict)).unwrap();
        assert!(v);
    }

    // ─── Int64 ───────────────────────────────────────────────────────

    #[test]
    fn test_int64_matches_int64() {
        let v = invoke_scalar(
            ExprLogicalType::Int64,
            ColumnarValue::Scalar(ScalarValue::Int64(Some(100))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_int64_rejects_int32() {
        let v = invoke_scalar(
            ExprLogicalType::Int64,
            ColumnarValue::Scalar(ScalarValue::Int32(Some(100))),
        )
        .unwrap();
        assert!(!v);
    }

    // ─── UInt8 ───────────────────────────────────────────────────────

    #[test]
    fn test_uint8_matches_uint8() {
        let v = invoke_scalar(
            ExprLogicalType::UInt8,
            ColumnarValue::Scalar(ScalarValue::UInt8(Some(255))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_uint8_rejects_uint32() {
        let v = invoke_scalar(
            ExprLogicalType::UInt8,
            ColumnarValue::Scalar(ScalarValue::UInt32(Some(255))),
        )
        .unwrap();
        assert!(!v);
    }

    // ─── UInt32 ──────────────────────────────────────────────────────

    #[test]
    fn test_uint32_matches_uint32() {
        let v = invoke_scalar(
            ExprLogicalType::UInt32,
            ColumnarValue::Scalar(ScalarValue::UInt32(Some(99))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_uint32_rejects_uint8() {
        let v = invoke_scalar(
            ExprLogicalType::UInt32,
            ColumnarValue::Scalar(ScalarValue::UInt8(Some(99))),
        )
        .unwrap();
        assert!(!v);
    }

    // ─── Binary ──────────────────────────────────────────────────────

    #[test]
    fn test_binary_matches_binary() {
        let v = invoke_scalar(
            ExprLogicalType::Binary,
            ColumnarValue::Scalar(ScalarValue::Binary(Some(vec![0x01, 0x02]))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_binary_rejects_utf8() {
        let v = invoke_scalar(
            ExprLogicalType::Binary,
            ColumnarValue::Scalar(ScalarValue::Utf8(Some("bytes".into()))),
        )
        .unwrap();
        assert!(!v);
    }

    #[test]
    fn test_binary_matches_dict_binary() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int32),
            Box::new(ScalarValue::Binary(Some(vec![0xAB]))),
        );
        let v = invoke_scalar(ExprLogicalType::Binary, ColumnarValue::Scalar(dict)).unwrap();
        assert!(v);
    }

    // ─── FixedSizeBinary ─────────────────────────────────────────────

    #[test]
    fn test_fixed_size_binary_matches() {
        let v = invoke_scalar(
            ExprLogicalType::FixedSizeBinary(4),
            ColumnarValue::Scalar(ScalarValue::FixedSizeBinary(4, Some(vec![1, 2, 3, 4]))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_fixed_size_binary_wrong_size_rejects() {
        let v = invoke_scalar(
            ExprLogicalType::FixedSizeBinary(4),
            ColumnarValue::Scalar(ScalarValue::FixedSizeBinary(
                8,
                Some(vec![1, 2, 3, 4, 5, 6, 7, 8]),
            )),
        )
        .unwrap();
        assert!(!v);
    }

    #[test]
    fn test_fixed_size_binary_rejects_regular_binary() {
        let v = invoke_scalar(
            ExprLogicalType::FixedSizeBinary(4),
            ColumnarValue::Scalar(ScalarValue::Binary(Some(vec![1, 2, 3, 4]))),
        )
        .unwrap();
        assert!(!v);
    }

    // ─── DurationNanosecond ──────────────────────────────────────────

    #[test]
    fn test_duration_nanosecond_matches() {
        let v = invoke_scalar(
            ExprLogicalType::DurationNanoSecond,
            ColumnarValue::Scalar(ScalarValue::DurationNanosecond(Some(1_000_000))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_duration_nanosecond_rejects_int64() {
        let v = invoke_scalar(
            ExprLogicalType::DurationNanoSecond,
            ColumnarValue::Scalar(ScalarValue::Int64(Some(1_000_000))),
        )
        .unwrap();
        assert!(!v);
    }

    // ─── TimestampNanosecond ─────────────────────────────────────────

    #[test]
    fn test_timestamp_nanosecond_matches() {
        let v = invoke_scalar(
            ExprLogicalType::TimestampNanosecond,
            ColumnarValue::Scalar(ScalarValue::TimestampNanosecond(Some(1_000_000), None)),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_timestamp_nanosecond_rejects_duration() {
        let v = invoke_scalar(
            ExprLogicalType::TimestampNanosecond,
            ColumnarValue::Scalar(ScalarValue::DurationNanosecond(Some(1_000_000))),
        )
        .unwrap();
        assert!(!v);
    }

    // ─── AnyInt ──────────────────────────────────────────────────────

    #[test]
    fn test_any_int_matches_int32() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::Int32(Some(1))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_any_int_matches_int64() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::Int64(Some(1))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_any_int_matches_uint8() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::UInt8(Some(1))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_any_int_matches_uint16() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::UInt16(Some(1))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_any_int_matches_uint32() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::UInt32(Some(1))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_any_int_matches_uint64() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::UInt64(Some(1))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_any_int_matches_int8() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::Int8(Some(1))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_any_int_matches_int16() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::Int16(Some(1))),
        )
        .unwrap();
        assert!(v);
    }

    #[test]
    fn test_any_int_rejects_float64() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::Float64(Some(1.0))),
        )
        .unwrap();
        assert!(!v);
    }

    #[test]
    fn test_any_int_rejects_utf8() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::Utf8(Some("42".into()))),
        )
        .unwrap();
        assert!(!v);
    }

    #[test]
    fn test_any_int_rejects_boolean() {
        let v = invoke_scalar(
            ExprLogicalType::AnyInt,
            ColumnarValue::Scalar(ScalarValue::Boolean(Some(true))),
        )
        .unwrap();
        assert!(!v);
    }

    // ─── AnyInt + Dictionary ─────────────────────────────────────────

    #[test]
    fn test_any_int_with_dict_int32() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int8),
            Box::new(ScalarValue::Int32(Some(42))),
        );
        let v = invoke_scalar(ExprLogicalType::AnyInt, ColumnarValue::Scalar(dict)).unwrap();
        assert!(
            v,
            "Dictionary with Int32 values should be considered an integer"
        );
    }

    #[test]
    fn test_any_int_with_dict_int64() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int8),
            Box::new(ScalarValue::Int64(Some(99))),
        );
        let v = invoke_scalar(ExprLogicalType::AnyInt, ColumnarValue::Scalar(dict)).unwrap();
        assert!(
            v,
            "Dictionary with Int64 values should be considered an integer"
        );
    }

    #[test]
    fn test_any_int_with_dict_uint8() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int32),
            Box::new(ScalarValue::UInt8(Some(7))),
        );
        let v = invoke_scalar(ExprLogicalType::AnyInt, ColumnarValue::Scalar(dict)).unwrap();
        assert!(
            v,
            "Dictionary with UInt8 values should be considered an integer"
        );
    }

    #[test]
    fn test_any_int_with_dict_uint64() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int32),
            Box::new(ScalarValue::UInt64(Some(1000))),
        );
        let v = invoke_scalar(ExprLogicalType::AnyInt, ColumnarValue::Scalar(dict)).unwrap();
        assert!(
            v,
            "Dictionary with UInt64 values should be considered an integer"
        );
    }

    #[test]
    fn test_any_int_with_dict_float64_rejects() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int32),
            Box::new(ScalarValue::Float64(Some(5.14))),
        );
        let v = invoke_scalar(ExprLogicalType::AnyInt, ColumnarValue::Scalar(dict)).unwrap();
        assert!(!v, "Dictionary with Float64 values is not an integer");
    }

    #[test]
    fn test_any_int_with_dict_utf8_rejects() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int32),
            Box::new(ScalarValue::Utf8(Some("nope".into()))),
        );
        let v = invoke_scalar(ExprLogicalType::AnyInt, ColumnarValue::Scalar(dict)).unwrap();
        assert!(!v, "Dictionary with Utf8 values is not an integer");
    }
    // ─── Array inputs (ColumnarValue::Array) ─────────────────────────

    #[test]
    fn test_string_matches_utf8_array() {
        let arr: ArrayRef = Arc::new(StringArray::from(vec!["hello", "world"]));
        let v = invoke_array(ExprLogicalType::String, arr).unwrap();
        assert!(v);
    }

    #[test]
    fn test_int32_matches_int32_array() {
        let arr: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let v = invoke_array(ExprLogicalType::Int32, arr).unwrap();
        assert!(v);
    }

    #[test]
    fn test_float64_rejects_int32_array() {
        let arr: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let v = invoke_array(ExprLogicalType::Float64, arr).unwrap();
        assert!(!v);
    }

    #[test]
    fn test_any_int_matches_uint8_array() {
        let arr: ArrayRef = Arc::new(UInt8Array::from(vec![1, 2, 3]));
        let v = invoke_array(ExprLogicalType::AnyInt, arr).unwrap();
        assert!(v);
    }

    // ─── is_logically_type (unit tests for the helper) ──────────────

    #[test]
    fn test_is_logically_type_exact_match() {
        assert!(is_logically_type(&DataType::Utf8, &DataType::Utf8));
        assert!(is_logically_type(&DataType::Int32, &DataType::Int32));
        assert!(is_logically_type(&DataType::Float64, &DataType::Float64));
        assert!(is_logically_type(
            &DataType::Duration(TimeUnit::Nanosecond),
            &DataType::Duration(TimeUnit::Nanosecond),
        ));
    }

    #[test]
    fn test_is_logically_type_mismatch() {
        assert!(!is_logically_type(&DataType::Utf8, &DataType::Int32));
        assert!(!is_logically_type(&DataType::Float64, &DataType::Utf8));
        assert!(!is_logically_type(
            &DataType::Duration(TimeUnit::Nanosecond),
            &DataType::Duration(TimeUnit::Microsecond),
        ));
    }

    #[test]
    fn test_is_logically_type_dictionary_match() {
        let dict_type = DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8));
        assert!(is_logically_type(&dict_type, &DataType::Utf8));
    }

    #[test]
    fn test_is_logically_type_dictionary_mismatch() {
        let dict_type = DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Int64));
        assert!(!is_logically_type(&dict_type, &DataType::Utf8));
    }

    #[test]
    fn test_is_logically_type_dictionary_with_various_key_types() {
        // The key type shouldn't matter - only the value type
        for key_type in [
            DataType::Int8,
            DataType::Int16,
            DataType::Int32,
            DataType::Int64,
        ] {
            let dict_type = DataType::Dictionary(Box::new(key_type), Box::new(DataType::Float64));
            assert!(
                is_logically_type(&dict_type, &DataType::Float64),
                "Dictionary with any key type should match on value type"
            );
        }
    }

    // ─── IsTypeFunc: PartialEq and Debug ─────────────────────────────

    #[test]
    fn test_is_type_func_debug() {
        let func = IsTypeFunc::new(ExprLogicalType::Boolean);
        let debug_str = format!("{:?}", func);
        assert!(debug_str.contains("IsTypeFunc"));
        assert!(debug_str.contains("Boolean"));
    }

    #[test]
    fn test_is_type_func_as_any() {
        let func = IsTypeFunc::new(ExprLogicalType::Boolean);
        let any_ref = func.as_any();
        assert!(any_ref.downcast_ref::<IsTypeFunc>().is_some());
    }

    #[test]
    fn test_is_type_func_as_any_matches_original() {
        let func = IsTypeFunc::new(ExprLogicalType::Int64);
        let any_ref = func.as_any();
        let downcasted = any_ref.downcast_ref::<IsTypeFunc>().unwrap();
        assert_eq!(downcasted, &func);
    }
}
