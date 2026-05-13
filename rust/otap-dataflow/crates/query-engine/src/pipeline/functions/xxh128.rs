// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BinaryArray, BinaryBuilder, LargeBinaryArray, LargeStringArray, StringArray,
    StringViewArray,
};
use arrow::datatypes::DataType;
use datafusion::common::exec_err;
use datafusion::error::Result;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
};
use datafusion::scalar::ScalarValue;
use xxhash_rust::xxh3::xxh3_128;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Xxh128Func {
    signature: Signature,
}

impl Default for Xxh128Func {
    fn default() -> Self {
        Self::new()
    }
}

impl Xxh128Func {
    pub fn new() -> Self {
        Self {
            signature: Signature::any(1, Volatility::Immutable),
        }
    }
}

impl ScalarUDFImpl for Xxh128Func {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "xxh128"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Binary)
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
                Ok(ColumnarValue::Scalar(ScalarValue::Binary(hash)))
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

fn to_bytes(hash: u128) -> Vec<u8> {
    hash.to_be_bytes().to_vec()
}

fn hash_scalar(scalar: &ScalarValue) -> Result<Option<Vec<u8>>> {
    match scalar {
        ScalarValue::Utf8(v) | ScalarValue::LargeUtf8(v) => {
            Ok(v.as_deref().map(|s| to_bytes(xxh3_128(s.as_bytes()))))
        }
        ScalarValue::Utf8View(v) => Ok(v.as_deref().map(|s| to_bytes(xxh3_128(s.as_bytes())))),
        ScalarValue::Binary(v) | ScalarValue::LargeBinary(v) => {
            Ok(v.as_ref().map(|b| to_bytes(xxh3_128(b))))
        }
        other => exec_err!("xxh128: unsupported scalar type {:?}", other.data_type()),
    }
}

fn hash_array(arr: &dyn Array) -> Result<BinaryArray> {
    let len = arr.len();
    let mut builder = BinaryBuilder::with_capacity(len, len * 16);

    match arr.data_type() {
        DataType::Utf8 => {
            let arr = arr.as_any().downcast_ref::<StringArray>().expect("Utf8");
            for v in arr.iter() {
                match v {
                    Some(s) => builder.append_value(to_bytes(xxh3_128(s.as_bytes()))),
                    None => builder.append_null(),
                }
            }
        }
        DataType::LargeUtf8 => {
            let arr = arr
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .expect("LargeUtf8");
            for v in arr.iter() {
                match v {
                    Some(s) => builder.append_value(to_bytes(xxh3_128(s.as_bytes()))),
                    None => builder.append_null(),
                }
            }
        }
        DataType::Utf8View => {
            let arr = arr
                .as_any()
                .downcast_ref::<StringViewArray>()
                .expect("Utf8View");
            for v in arr.iter() {
                match v {
                    Some(s) => builder.append_value(to_bytes(xxh3_128(s.as_bytes()))),
                    None => builder.append_null(),
                }
            }
        }
        DataType::Binary => {
            let arr = arr.as_any().downcast_ref::<BinaryArray>().expect("Binary");
            for v in arr.iter() {
                match v {
                    Some(b) => builder.append_value(to_bytes(xxh3_128(b))),
                    None => builder.append_null(),
                }
            }
        }
        DataType::LargeBinary => {
            let arr = arr
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("LargeBinary");
            for v in arr.iter() {
                match v {
                    Some(b) => builder.append_value(to_bytes(xxh3_128(b))),
                    None => builder.append_null(),
                }
            }
        }
        other => return exec_err!("xxh128: unsupported array type {:?}", other),
    }

    Ok(builder.finish())
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
        Xxh128Func::new().invoke_with_args(ScalarFunctionArgs {
            args: vec![arg],
            arg_fields: vec![],
            number_rows: 1,
            return_field: Field::new("", DataType::Binary, true).into(),
            config_options: Arc::new(ConfigOptions::default()),
        })
    }

    #[test]
    fn test_xxh128_string() {
        let result = invoke(ColumnarValue::Scalar(ScalarValue::Utf8(Some(
            "hello".to_string(),
        ))))
        .unwrap();
        match result {
            ColumnarValue::Scalar(ScalarValue::Binary(Some(v))) => {
                assert_eq!(v, to_bytes(xxh3_128(b"hello")));
                assert_eq!(v.len(), 16);
            }
            other => panic!("expected Scalar(Binary(Some(_))), got {:?}", other),
        }
    }

    #[test]
    fn test_xxh128_binary_matches_string() {
        let result = invoke(ColumnarValue::Scalar(ScalarValue::Binary(Some(
            b"hello".to_vec(),
        ))))
        .unwrap();
        match result {
            ColumnarValue::Scalar(ScalarValue::Binary(Some(v))) => {
                assert_eq!(v, to_bytes(xxh3_128(b"hello")));
            }
            other => panic!("expected Scalar(Binary(Some(_))), got {:?}", other),
        }
    }

    #[test]
    fn test_xxh128_null() {
        let result = invoke(ColumnarValue::Scalar(ScalarValue::Utf8(None))).unwrap();
        match result {
            ColumnarValue::Scalar(ScalarValue::Binary(None)) => {}
            other => panic!("expected Scalar(Binary(None)), got {:?}", other),
        }
    }
}
