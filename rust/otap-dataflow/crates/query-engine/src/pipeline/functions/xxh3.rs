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
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Xxh3Func {
    signature: Signature,
}

impl Default for Xxh3Func {
    fn default() -> Self {
        Self::new()
    }
}

impl Xxh3Func {
    pub fn new() -> Self {
        Self {
            signature: Signature::any(1, Volatility::Immutable),
        }
    }
}

impl ScalarUDFImpl for Xxh3Func {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "xxh3"
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

fn hash_scalar(scalar: &ScalarValue) -> Result<Option<i64>> {
    match scalar {
        ScalarValue::Utf8(v) | ScalarValue::LargeUtf8(v) => {
            Ok(v.as_deref().map(|s| xxh3_64(s.as_bytes()) as i64))
        }
        ScalarValue::Utf8View(v) => Ok(v.as_deref().map(|s| xxh3_64(s.as_bytes()) as i64)),
        ScalarValue::Binary(v) | ScalarValue::LargeBinary(v) => {
            Ok(v.as_ref().map(|b| xxh3_64(b) as i64))
        }
        other => exec_err!("xxh3: unsupported scalar type {:?}", other.data_type()),
    }
}

fn hash_array(arr: &dyn Array) -> Result<Int64Array> {
    match arr.data_type() {
        DataType::Utf8 => {
            let arr = arr.as_any().downcast_ref::<StringArray>().expect("Utf8");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|s| xxh3_64(s.as_bytes()) as i64)),
            ))
        }
        DataType::LargeUtf8 => {
            let arr = arr
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .expect("LargeUtf8");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|s| xxh3_64(s.as_bytes()) as i64)),
            ))
        }
        DataType::Utf8View => {
            let arr = arr
                .as_any()
                .downcast_ref::<StringViewArray>()
                .expect("Utf8View");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|s| xxh3_64(s.as_bytes()) as i64)),
            ))
        }
        DataType::Binary => {
            let arr = arr.as_any().downcast_ref::<BinaryArray>().expect("Binary");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|b| xxh3_64(b) as i64)),
            ))
        }
        DataType::LargeBinary => {
            let arr = arr
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("LargeBinary");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|b| xxh3_64(b) as i64)),
            ))
        }
        other => exec_err!("xxh3: unsupported array type {:?}", other),
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

    fn invoke(arg: ColumnarValue) -> Result<ColumnarValue> {
        Xxh3Func::new().invoke_with_args(ScalarFunctionArgs {
            args: vec![arg],
            arg_fields: vec![],
            number_rows: 1,
            return_field: Field::new("", DataType::Int64, true).into(),
            config_options: Arc::new(ConfigOptions::default()),
        })
    }

    #[test]
    fn test_xxh3_string() {
        let result = invoke(ColumnarValue::Scalar(ScalarValue::Utf8(Some(
            "hello".to_string(),
        ))))
        .unwrap();
        match result {
            ColumnarValue::Scalar(ScalarValue::Int64(Some(v))) => {
                assert_eq!(v, xxh3_64(b"hello") as i64);
            }
            other => panic!("expected Scalar(Int64(Some(_))), got {:?}", other),
        }
    }

    #[test]
    fn test_xxh3_binary_matches_string() {
        let result = invoke(ColumnarValue::Scalar(ScalarValue::Binary(Some(
            b"hello".to_vec(),
        ))))
        .unwrap();
        match result {
            ColumnarValue::Scalar(ScalarValue::Int64(Some(v))) => {
                assert_eq!(v, xxh3_64(b"hello") as i64);
            }
            other => panic!("expected Scalar(Int64(Some(_))), got {:?}", other),
        }
    }

    #[test]
    fn test_xxh3_null() {
        let result = invoke(ColumnarValue::Scalar(ScalarValue::Utf8(None))).unwrap();
        match result {
            ColumnarValue::Scalar(ScalarValue::Int64(None)) => {}
            other => panic!("expected Scalar(Int64(None)), got {:?}", other),
        }
    }
}
