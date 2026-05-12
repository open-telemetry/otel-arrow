// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::array::GenericStringBuilder;
use arrow::datatypes::DataType;
use datafusion::common::{Result, exec_err};
use datafusion::logical_expr::{
    ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
};
use uuid::Uuid;

/// Custom scalar UDF that returns a fresh [UUID v7][rfc-9562] string per row.
///
/// DataFusion provides a built-in `uuid()` function for v4 UUIDs, but no v7 equivalent.
/// This implementation mirrors the shape of DataFusion's `UuidFunc` and produces one
/// UUID per row in the input batch using [`Uuid::now_v7`], so the output is unique per
/// row and time-ordered.
///
/// [rfc-9562]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct UuidV7Func {
    signature: Signature,
}

impl Default for UuidV7Func {
    fn default() -> Self {
        Self::new()
    }
}

impl UuidV7Func {
    pub fn new() -> Self {
        Self {
            signature: Signature::nullary(Volatility::Volatile),
        }
    }
}

impl ScalarUDFImpl for UuidV7Func {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "uuidv7"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Ok(DataType::Utf8)
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        if !args.args.is_empty() {
            return exec_err!(
                "{} function does not accept arguments, received {}",
                self.name(),
                args.args.len()
            );
        }

        let mut builder =
            GenericStringBuilder::<i32>::with_capacity(args.number_rows, args.number_rows * 36);
        let mut buffer = [0u8; 36];
        for _ in 0..args.number_rows {
            let uuid = Uuid::now_v7();
            let fmt = uuid::fmt::Hyphenated::from_uuid(uuid);
            builder.append_value(fmt.encode_lower(&mut buffer));
        }

        Ok(ColumnarValue::Array(Arc::new(builder.finish())))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{Array, StringArray};
    use arrow::datatypes::Field;
    use datafusion::common::config::ConfigOptions;
    use datafusion::logical_expr::ScalarUDFImpl;
    use std::collections::HashSet;

    fn invoke_uuidv7_udf(number_rows: usize) -> ColumnarValue {
        let udf = UuidV7Func::default();
        let args = ScalarFunctionArgs {
            args: vec![],
            arg_fields: vec![],
            number_rows,
            return_field: Field::new("f", DataType::Utf8, true).into(),
            config_options: Arc::new(ConfigOptions::default()),
        };
        udf.invoke_with_args(args).unwrap()
    }

    #[test]
    fn test_uuidv7_returns_distinct_values_per_row() {
        let result = invoke_uuidv7_udf(5);
        let arr = result.into_array(5).unwrap();
        let str_arr = arr.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(str_arr.len(), 5);

        let unique: HashSet<&str> = str_arr.iter().map(|v| v.unwrap()).collect();
        assert_eq!(unique.len(), 5, "expected 5 distinct UUIDs, got {unique:?}");

        for value in str_arr.iter().flatten() {
            let parsed = Uuid::parse_str(value).expect("UUID parse");
            assert_eq!(
                parsed.get_version_num(),
                7,
                "expected v7 UUID, got {value} ({:?})",
                parsed.get_version()
            );
        }
    }

    #[test]
    fn test_uuidv7_zero_rows() {
        let result = invoke_uuidv7_udf(0);
        let arr = result.into_array(0).unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_uuidv7_rejects_arguments() {
        let udf = UuidV7Func::default();
        let args = ScalarFunctionArgs {
            args: vec![ColumnarValue::Scalar(
                datafusion::scalar::ScalarValue::Utf8(Some("nope".into())),
            )],
            arg_fields: vec![Field::new("a", DataType::Utf8, true).into()],
            number_rows: 1,
            return_field: Field::new("f", DataType::Utf8, true).into(),
            config_options: Arc::new(ConfigOptions::default()),
        };
        assert!(udf.invoke_with_args(args).is_err());
    }
}
