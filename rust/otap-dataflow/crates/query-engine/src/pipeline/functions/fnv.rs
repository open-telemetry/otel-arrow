// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BinaryArray, Int64Array, LargeBinaryArray, LargeStringArray, StringArray,
    StringViewArray,
};
use arrow::datatypes::DataType;
use datafusion::common::exec_err;
use datafusion::error::Result;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
};
use datafusion::scalar::ScalarValue;

const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
const FNV_PRIME: u64 = 1099511628211;

fn fnv1a_64(data: &[u8]) -> i64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash as i64
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FnvHashFunc {
    signature: Signature,
}

impl Default for FnvHashFunc {
    fn default() -> Self {
        Self::new()
    }
}

impl FnvHashFunc {
    pub fn new() -> Self {
        Self {
            signature: Signature::any(1, Volatility::Immutable),
        }
    }
}

impl ScalarUDFImpl for FnvHashFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "fnv"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Int64)
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

        match &args[0] {
            ColumnarValue::Scalar(scalar) => {
                let hash = hash_scalar(scalar)?;
                Ok(ColumnarValue::Scalar(ScalarValue::Int64(hash)))
            }
            ColumnarValue::Array(arr) => {
                let hashes = hash_array(arr.as_ref())?;
                Ok(ColumnarValue::Array(Arc::new(hashes) as ArrayRef))
            }
        }
    }

    fn documentation(&self) -> Option<&Documentation> {
        None
    }
}

/// computes the FNV-1a 64-bit hash of a scalar value
/// returns None if the input is null, or an error if the input type is not supported.
fn hash_scalar(scalar: &ScalarValue) -> Result<Option<i64>> {
    match scalar {
        ScalarValue::Utf8(v) | ScalarValue::LargeUtf8(v) => {
            Ok(v.as_deref().map(|s| fnv1a_64(s.as_bytes())))
        }
        ScalarValue::Utf8View(v) => Ok(v.as_deref().map(|s| fnv1a_64(s.as_bytes()))),
        ScalarValue::Binary(v) | ScalarValue::LargeBinary(v) => Ok(v.as_ref().map(|b| fnv1a_64(b))),
        other => exec_err!("fnv: unsupported scalar type {:?}", other.data_type()),
    }
}

/// computes the FNV-1a 64-bit hash of each element in an array
/// null elements produce null output values.
/// supports [DataType::Utf8], [DataType::LargeUtf8], and [DataType::Binary] input arrays.
fn hash_array(arr: &dyn Array) -> Result<Int64Array> {
    match arr.data_type() {
        DataType::Utf8 => {
            let arr = arr.as_any().downcast_ref::<StringArray>().expect("Utf8");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|s| fnv1a_64(s.as_bytes()))),
            ))
        }
        DataType::LargeUtf8 => {
            let arr = arr
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .expect("LargeUtf8");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|s| fnv1a_64(s.as_bytes()))),
            ))
        }
        DataType::Utf8View => {
            let arr = arr
                .as_any()
                .downcast_ref::<StringViewArray>()
                .expect("Utf8View");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|s| fnv1a_64(s.as_bytes()))),
            ))
        }
        DataType::Binary => {
            let arr = arr.as_any().downcast_ref::<BinaryArray>().expect("Binary");
            Ok(Int64Array::from_iter(arr.iter().map(|v| v.map(fnv1a_64))))
        }
        DataType::LargeBinary => {
            let arr = arr
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("LargeBinary");
            Ok(Int64Array::from_iter(arr.iter().map(|v| v.map(fnv1a_64))))
        }
        other => exec_err!("fnv: unsupported array type {:?}", other),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::datatypes::Field;
    use datafusion::common::config::ConfigOptions;
    use datafusion::logical_expr::{ColumnarValue, ScalarFunctionArgs};
    use datafusion::scalar::ScalarValue;
    use std::sync::Arc;

    fn invoke(arg: ColumnarValue) -> Result<i64> {
        let func = FnvHashFunc::new();
        let result = func.invoke_with_args(ScalarFunctionArgs {
            args: vec![arg],
            arg_fields: vec![],
            number_rows: 1,
            return_field: Field::new("", DataType::Int64, true).into(),
            config_options: Arc::new(ConfigOptions::default()),
        })?;
        match result {
            ColumnarValue::Scalar(ScalarValue::Int64(Some(v))) => Ok(v),
            other => panic!("expected Scalar(Int64(Some(_))), got {:?}", other),
        }
    }

    #[test]
    fn test_fnv_empty_string() {
        // FNV-1a 64-bit of empty input is the offset basis itself
        let v = invoke(ColumnarValue::Scalar(ScalarValue::Utf8(
            Some(String::new()),
        )))
        .unwrap();
        assert_eq!(v, FNV_OFFSET_BASIS as i64);
    }

    #[test]
    fn test_fnv_string() {
        let v = invoke(ColumnarValue::Scalar(ScalarValue::Utf8(Some(
            "hello".to_string(),
        ))))
        .unwrap();
        assert_eq!(v, -6615550055289275125_i64);
    }

    #[test]
    fn test_fnv_binary() {
        let v = invoke(ColumnarValue::Scalar(ScalarValue::Binary(Some(
            b"hello".to_vec(),
        ))))
        .unwrap();
        assert_eq!(v, -6615550055289275125_i64);
    }

    #[test]
    fn test_fnv_null_returns_none() {
        let func = FnvHashFunc::new();
        let result = func
            .invoke_with_args(ScalarFunctionArgs {
                args: vec![ColumnarValue::Scalar(ScalarValue::Utf8(None))],
                arg_fields: vec![],
                number_rows: 1,
                return_field: Field::new("", DataType::Int64, true).into(),
                config_options: Arc::new(ConfigOptions::default()),
            })
            .unwrap();
        match result {
            ColumnarValue::Scalar(ScalarValue::Int64(None)) => {}
            other => panic!("expected Scalar(Int64(None)), got {:?}", other),
        }
    }

    #[test]
    fn test_fnv_array() {
        use arrow::array::StringArray;
        let arr = StringArray::from(vec![Some("hello"), Some("world"), None]);
        let func = FnvHashFunc::new();
        let result = func
            .invoke_with_args(ScalarFunctionArgs {
                args: vec![ColumnarValue::Array(Arc::new(arr))],
                arg_fields: vec![],
                number_rows: 3,
                return_field: Field::new("", DataType::Int64, true).into(),
                config_options: Arc::new(ConfigOptions::default()),
            })
            .unwrap();
        let ColumnarValue::Array(arr) = result else {
            panic!("expected array");
        };
        let arr = arr.as_any().downcast_ref::<Int64Array>().expect("Int64");
        assert_eq!(arr.value(0), -6615550055289275125_i64);
        assert!(!arr.is_null(0));
        assert!(!arr.is_null(1));
        assert!(arr.is_null(2));
    }
}
