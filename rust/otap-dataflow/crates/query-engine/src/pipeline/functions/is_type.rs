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
use crate::pipeline::project::anyval::is_any_value_data_type;

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
            ExprLogicalType::AnyInt => is_any_int(&arg_data_type),
            ExprLogicalType::AnyValueNumeric => {
                is_logically_type(&arg_data_type, &DataType::Float64) | is_any_int(&arg_data_type)
            }
            ExprLogicalType::AnyValue => is_any_value_data_type(&arg_data_type),
        };

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

fn is_any_int(source: &DataType) -> bool {
    if source.is_integer() {
        true
    } else if let DataType::Dictionary(_, v) = &source {
        v.as_ref().is_integer()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use arrow::array::{ArrayRef, AsArray, StringArray, StructArray, UInt8Array};
    use arrow::datatypes::{DataType, Field};
    use datafusion::config::ConfigOptions;
    use datafusion::logical_expr::{ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl};
    use datafusion::scalar::ScalarValue;
    use otap_df_pdata::otlp::attributes::AttributeValueType;
    use otap_df_pdata::schema::consts;

    /// helper to invoke is_type scalar UDF
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

    fn invoke_array(expected_type: ExprLogicalType, arg: ArrayRef) -> Result<BooleanArray> {
        let func = IsTypeFunc::new(expected_type);
        let result = func.invoke_with_args(ScalarFunctionArgs {
            args: vec![ColumnarValue::Array(arg)],
            arg_fields: Vec::new(),
            number_rows: 1,
            return_field: Arc::new(Field::new("", DataType::Boolean, false)),
            config_options: Arc::new(ConfigOptions::default()),
        })?;
        match result {
            ColumnarValue::Array(v) => Ok(v.as_boolean().clone()),
            other => panic!("expected Scalar(Boolean(Some(_))), got {:?}", other),
        }
    }

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
    fn test_boolean_rejects_invalid_type() {
        let v = invoke_scalar(
            ExprLogicalType::Boolean,
            ColumnarValue::Scalar(ScalarValue::Int32(Some(1))),
        )
        .unwrap();
        assert!(!v);
    }

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
    fn test_float64_rejects_dict_invalid_type() {
        let dict = ScalarValue::Dictionary(
            Box::new(DataType::Int32),
            Box::new(ScalarValue::Utf8(Some("hello".into()))),
        );
        let v = invoke_scalar(ExprLogicalType::Float64, ColumnarValue::Scalar(dict)).unwrap();
        assert!(!v);
    }

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
    fn test_string_rejects_invalid_type() {
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
    fn test_int32_rejects_invalid_type() {
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
    fn test_binary_matches_binary() {
        let v = invoke_scalar(
            ExprLogicalType::Binary,
            ColumnarValue::Scalar(ScalarValue::Binary(Some(vec![0x01, 0x02]))),
        )
        .unwrap();
        assert!(v);
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

    #[test]
    fn test_any_int_matches_all_integer_types() {
        let int_scalars = vec![
            ScalarValue::Int8(Some(1)),
            ScalarValue::Int16(Some(1)),
            ScalarValue::Int32(Some(1)),
            ScalarValue::Int64(Some(1)),
            ScalarValue::UInt8(Some(1)),
            ScalarValue::UInt16(Some(1)),
            ScalarValue::UInt32(Some(1)),
            ScalarValue::UInt64(Some(1)),
        ];

        for scalar in int_scalars {
            let dt = scalar.data_type();
            let v = invoke_scalar(ExprLogicalType::AnyInt, ColumnarValue::Scalar(scalar)).unwrap();
            assert!(v, "AnyInt should match {dt}");
        }
    }

    #[test]
    fn test_any_int_with_dict_integer_values() {
        let int_dict_scalars = vec![
            ScalarValue::Int8(Some(1)),
            ScalarValue::Int16(Some(1)),
            ScalarValue::Int32(Some(42)),
            ScalarValue::Int64(Some(99)),
            ScalarValue::UInt8(Some(7)),
            ScalarValue::UInt16(Some(1)),
            ScalarValue::UInt32(Some(1)),
            ScalarValue::UInt64(Some(1000)),
        ];

        for value in int_dict_scalars {
            let dt = value.data_type();
            let dict = ScalarValue::Dictionary(Box::new(DataType::Int32), Box::new(value));
            let v = invoke_scalar(ExprLogicalType::AnyInt, ColumnarValue::Scalar(dict)).unwrap();
            assert!(
                v,
                "Dictionary with {dt} values should be considered an integer"
            );
        }
    }

    #[test]
    fn test_any_int_with_dict_non_integer_values_rejects() {
        let non_int_dict_scalars = vec![
            ScalarValue::Float64(Some(5.14)),
            ScalarValue::Utf8(Some("nope".into())),
            ScalarValue::Boolean(Some(true)),
        ];

        for value in non_int_dict_scalars {
            let dt = value.data_type();
            let dict = ScalarValue::Dictionary(Box::new(DataType::Int32), Box::new(value));
            let v = invoke_scalar(ExprLogicalType::AnyInt, ColumnarValue::Scalar(dict)).unwrap();
            assert!(
                !v,
                "Dictionary with {dt} values should not be considered an integer"
            );
        }
    }

    #[test]
    fn test_any_numeric() {
        let numeric_data_types = [
            ScalarValue::Float64(Some(5.14)),
            ScalarValue::Int64(Some(514)),
        ];
        for value in numeric_data_types {
            let v = invoke_scalar(
                ExprLogicalType::AnyValueNumeric,
                ColumnarValue::Scalar(value),
            )
            .unwrap();
            assert!(v,);
        }
    }

    #[test]
    fn test_any_value_struct() {
        let v = invoke_array(
            ExprLogicalType::AnyValue,
            Arc::new(StructArray::new(
                vec![
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                ]
                .into(),
                vec![
                    Arc::new(UInt8Array::from_iter_values(
                        [AttributeValueType::Str as u8],
                    )),
                    Arc::new(StringArray::from_iter_values(["hello"])),
                ],
                None,
            )),
        )
        .unwrap();
        assert!(v.value(0))
    }
}
