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

/// MurmurHash3 32-bit (seed=0). Returns the hash as i64 for parity with OTTL.
fn murmur3_32(data: &[u8]) -> i64 {
    const C1: u32 = 0xcc9e2d51;
    const C2: u32 = 0x1b873593;

    let mut h: u32 = 0;
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let mut k = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        k = k.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
        h ^= k;
        h = h.rotate_left(13).wrapping_mul(5).wrapping_add(0xe6546b64);
    }

    let mut k: u32 = 0;
    match remainder.len() {
        3 => {
            k ^= u32::from(remainder[2]) << 16;
            k ^= u32::from(remainder[1]) << 8;
            k ^= u32::from(remainder[0]);
            k = k.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
            h ^= k;
        }
        2 => {
            k ^= u32::from(remainder[1]) << 8;
            k ^= u32::from(remainder[0]);
            k = k.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
            h ^= k;
        }
        1 => {
            k ^= u32::from(remainder[0]);
            k = k.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
            h ^= k;
        }
        _ => {}
    }

    h ^= data.len() as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;

    i64::from(h)
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Murmur3HashFunc {
    signature: Signature,
}

impl Default for Murmur3HashFunc {
    fn default() -> Self {
        Self::new()
    }
}

impl Murmur3HashFunc {
    pub fn new() -> Self {
        Self {
            signature: Signature::any(1, Volatility::Immutable),
        }
    }
}

impl ScalarUDFImpl for Murmur3HashFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "murmur3"
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
            Ok(v.as_deref().map(|s| murmur3_32(s.as_bytes())))
        }
        ScalarValue::Utf8View(v) => Ok(v.as_deref().map(|s| murmur3_32(s.as_bytes()))),
        ScalarValue::Binary(v) | ScalarValue::LargeBinary(v) => {
            Ok(v.as_ref().map(|b| murmur3_32(b)))
        }
        other => exec_err!("murmur3: unsupported scalar type {:?}", other.data_type()),
    }
}

fn hash_array(arr: &dyn Array) -> Result<Int64Array> {
    match arr.data_type() {
        DataType::Utf8 => {
            let arr = arr.as_any().downcast_ref::<StringArray>().expect("Utf8");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|s| murmur3_32(s.as_bytes()))),
            ))
        }
        DataType::LargeUtf8 => {
            let arr = arr
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .expect("LargeUtf8");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|s| murmur3_32(s.as_bytes()))),
            ))
        }
        DataType::Utf8View => {
            let arr = arr
                .as_any()
                .downcast_ref::<StringViewArray>()
                .expect("Utf8View");
            Ok(Int64Array::from_iter(
                arr.iter().map(|v| v.map(|s| murmur3_32(s.as_bytes()))),
            ))
        }
        DataType::Binary => {
            let arr = arr.as_any().downcast_ref::<BinaryArray>().expect("Binary");
            Ok(Int64Array::from_iter(arr.iter().map(|v| v.map(murmur3_32))))
        }
        DataType::LargeBinary => {
            let arr = arr
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("LargeBinary");
            Ok(Int64Array::from_iter(arr.iter().map(|v| v.map(murmur3_32))))
        }
        other => exec_err!("murmur3: unsupported array type {:?}", other),
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
        let func = Murmur3HashFunc::new();
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
    fn test_murmur3_empty_string() {
        // murmur3 of empty input with seed=0 is 0
        let v = invoke(ColumnarValue::Scalar(ScalarValue::Utf8(
            Some(String::new()),
        )))
        .unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn test_murmur3_string() {
        let v = invoke(ColumnarValue::Scalar(ScalarValue::Utf8(Some(
            "hello".to_string(),
        ))))
        .unwrap();
        // pre-computed MurmurHash3 32-bit of "hello" with seed=0
        assert_eq!(v, 613_153_351_i64);
    }

    #[test]
    fn test_murmur3_binary_matches_string() {
        let v = invoke(ColumnarValue::Scalar(ScalarValue::Binary(Some(
            b"hello".to_vec(),
        ))))
        .unwrap();
        assert_eq!(v, 613_153_351_i64);
    }

    #[test]
    fn test_murmur3_null_returns_none() {
        let func = Murmur3HashFunc::new();
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
    fn test_murmur3_array() {
        use arrow::array::StringArray;
        let arr = StringArray::from(vec![Some("hello"), Some("world"), None]);
        let func = Murmur3HashFunc::new();
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
        assert_eq!(arr.value(0), 613_153_351_i64);
        assert!(!arr.is_null(0));
        assert!(!arr.is_null(1));
        assert!(arr.is_null(2));
    }
}
