// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, BooleanArray, LargeStringArray, StringArray, StringViewArray};
use arrow::compute::contains as arrow_contains;
use arrow::datatypes::DataType;
use datafusion::common::exec_err;
use datafusion::error::Result;
use datafusion::functions::string::contains::ContainsFunc;
use datafusion::logical_expr::binary::{binary_to_string_coercion, string_coercion};
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ScalarFunctionArgs, ScalarUDFImpl, Signature,
};
use datafusion::scalar::ScalarValue;

/// This is a stand-in replacement for DataFusion's [`ContainsFunc`] which adds supports for
/// dictionary encoded arrays.
///
// TODO some of this code is lifted & adapted from [`datafusion::functions::string::contains`] and
// eventually we could contribute it back upstream to add support for dictionary arrays, then use
// contains function directly.
#[derive(Default, Debug, PartialEq, Eq, Hash)]
pub struct ExtendedContainsFunc {
    inner: ContainsFunc,
}

impl ExtendedContainsFunc {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ScalarUDFImpl for ExtendedContainsFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "contains"
    }

    fn signature(&self) -> &Signature {
        self.inner.signature()
    }

    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Boolean)
    }

    fn invoke_with_args(&self, scalar_func_args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let args = scalar_func_args.args;
        if args.len() != 2 {
            return exec_err!("invalid number of args. received {} expected 2", args.len());
        }

        // take the args
        // safety: we've already checked the length of the args vec
        let mut iter = args.into_iter();
        let left = iter.next().expect("there are two args");
        let right = iter.next().expect("there are two args");

        let is_scalar =
            matches!(left, ColumnarValue::Scalar(_)) && matches!(right, ColumnarValue::Scalar(_));
        let result = contains(left, right).map(|arr| Arc::new(arr) as ArrayRef);

        if is_scalar {
            let result = result.and_then(|arr| ScalarValue::try_from_array(&arr, 0));
            result.map(ColumnarValue::Scalar)
        } else {
            result.map(ColumnarValue::Array)
        }
    }

    fn documentation(&self) -> Option<&Documentation> {
        self.inner.documentation()
    }
}

/// returns the type of values in the column. For native arrays and scalar column, this is just the
/// data type, and for dictionary encoded arrays it is the type of the dictionary values.
fn logical_data_type(val: &ColumnarValue) -> DataType {
    match val {
        ColumnarValue::Array(arr) => match arr.data_type() {
            DataType::Dictionary(_, v) => v.as_ref(),
            dt => dt,
        }
        .clone(),
        ColumnarValue::Scalar(s) => s.data_type(),
    }
}

/// convert the array into being logically the coercion data type. If the array is dictionary
/// encoded, this means casting the values. Otherwise we just cast the array.
fn coerce(coercion_data_type: &DataType, array: &ArrayRef) -> Result<ArrayRef> {
    if array.data_type() == coercion_data_type {
        // array is already the coercion data type, nothing to do
        Ok(Arc::clone(array))
    } else {
        match array.data_type() {
            DataType::Dictionary(k, v) => {
                if v.as_ref() == coercion_data_type {
                    // dict values is already the coercion data type, nothing to do
                    Ok(Arc::clone(array))
                } else {
                    // cast the dict values
                    Ok(arrow::compute::kernels::cast::cast(
                        array,
                        &DataType::Dictionary(k.clone(), Box::new(coercion_data_type.clone())),
                    )?)
                }
            }

            // array is not dict, so just cast the array
            _ => Ok(arrow::compute::kernels::cast::cast(
                array,
                coercion_data_type,
            )?),
        }
    }
}

/// try to cast the scalar to the expected data type for contains check, then take the string
/// value from it if the cast was successful
fn coerce_scalar_to_string(coercion_data_type: &DataType, scalar: ScalarValue) -> Result<String> {
    let coerced_scalar = if scalar.data_type() == *coercion_data_type {
        scalar
    } else {
        scalar.cast_to(coercion_data_type)?
    };

    let scalar_str = match coerced_scalar {
        ScalarValue::Utf8View(Some(val)) => val,
        ScalarValue::Utf8(Some(val)) => val,
        ScalarValue::LargeUtf8(Some(val)) => val,
        _ => {
            return exec_err!(
                "unexpected result of coercion of scalar {:?}",
                coerced_scalar
            );
        }
    };

    Ok(scalar_str)
}

// this is lifted somewhat verbatim from [`datafusion::functions::string::contains`] which is why
// we should eventually contribute this back upstream
fn contains(left: ColumnarValue, right: ColumnarValue) -> Result<BooleanArray> {
    // select the datatype we'll use when calling contains. arrow's `contains` kernel expects both
    // left and right to be the same logical type, so we need to coerce to a common data type.
    // datafusion's coercion rules are used to make the type selection
    let left_data_type = logical_data_type(&left);
    let right_data_type = logical_data_type(&right);
    let coercion_data_type = string_coercion(&left_data_type, &right_data_type)
        .or_else(|| binary_to_string_coercion(&left_data_type, &right_data_type));

    if let Some(coercion_data_type) = coercion_data_type {
        match (left, right) {
            (ColumnarValue::Array(left), ColumnarValue::Array(right)) => match coercion_data_type {
                DataType::Utf8View | DataType::Utf8 | DataType::LargeUtf8 => {
                    let src_str = coerce(&coercion_data_type, &left)?;
                    let match_str = coerce(&coercion_data_type, &right)?;
                    Ok(arrow_contains(&src_str, &match_str)?)
                }
                other => {
                    exec_err!("Unsupported data type {other:?} for function `contains`.")
                }
            },
            (ColumnarValue::Array(left), ColumnarValue::Scalar(scalar)) => {
                let src_str = coerce(&coercion_data_type, &left)?;
                let scalar_str = coerce_scalar_to_string(&coercion_data_type, scalar)?;

                match coercion_data_type {
                    DataType::Utf8View => {
                        let match_str = StringViewArray::new_scalar(scalar_str);
                        Ok(arrow_contains(&src_str, &match_str)?)
                    }
                    DataType::Utf8 => {
                        let match_str = StringArray::new_scalar(scalar_str);
                        Ok(arrow_contains(&src_str, &match_str)?)
                    }
                    DataType::LargeUtf8 => {
                        let match_str = LargeStringArray::new_scalar(scalar_str);
                        Ok(arrow_contains(&src_str, &match_str)?)
                    }
                    other => {
                        exec_err!("Unsupported data type {other:?} for function `contains`.")
                    }
                }
            }
            (ColumnarValue::Scalar(left), ColumnarValue::Array(right)) => {
                let left_str = coerce_scalar_to_string(&coercion_data_type, left)?;
                let match_str = coerce(&coercion_data_type, &right)?;
                match coercion_data_type {
                    DataType::Utf8View => {
                        let src_str = StringViewArray::new_scalar(left_str);
                        Ok(arrow_contains(&src_str, &match_str)?)
                    }
                    DataType::Utf8 => {
                        let stc_str = StringArray::new_scalar(left_str);
                        Ok(arrow_contains(&stc_str, &match_str)?)
                    }
                    DataType::LargeUtf8 => {
                        let stc_str = LargeStringArray::new_scalar(left_str);
                        Ok(arrow_contains(&stc_str, &match_str)?)
                    }
                    other => {
                        exec_err!("Unsupported data type {other:?} for function `contains`.")
                    }
                }
            }
            (ColumnarValue::Scalar(left), ColumnarValue::Scalar(right)) => {
                let left_str = coerce_scalar_to_string(&coercion_data_type, left)?;
                let right_str = coerce_scalar_to_string(&coercion_data_type, right)?;

                match coercion_data_type {
                    DataType::Utf8View => {
                        let src_str = StringViewArray::new_scalar(left_str);
                        let match_str = StringViewArray::new_scalar(right_str);
                        Ok(arrow_contains(&src_str, &match_str)?)
                    }
                    DataType::Utf8 => {
                        let stc_str = StringArray::new_scalar(left_str);
                        let match_str = StringArray::new_scalar(right_str);
                        Ok(arrow_contains(&stc_str, &match_str)?)
                    }
                    DataType::LargeUtf8 => {
                        let stc_str = LargeStringArray::new_scalar(left_str);
                        let match_str = LargeStringArray::new_scalar(right_str);
                        Ok(arrow_contains(&stc_str, &match_str)?)
                    }
                    other => {
                        exec_err!("Unsupported data type {other:?} for function `contains`.")
                    }
                }
            }
        }
    } else {
        exec_err!(
            "Unsupported data type {}, {:?} for function `contains`.",
            left_data_type,
            right_data_type
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{
        BooleanArray, DictionaryArray, LargeStringArray, StringArray, StringViewArray, UInt8Array,
    };
    use arrow::datatypes::{DataType, Field};
    use datafusion::common::ScalarValue;
    use datafusion::common::config::ConfigOptions;
    use datafusion::logical_expr::{ColumnarValue, Expr, ScalarFunctionArgs, ScalarUDFImpl};
    use std::sync::Arc;

    fn invoke_contains_udf(
        left: ColumnarValue,
        right: ColumnarValue,
        number_rows: usize,
    ) -> ColumnarValue {
        let udf = ExtendedContainsFunc::default();
        let arg_fields = vec![
            Field::new("a", DataType::Utf8, true).into(),
            Field::new("a", DataType::Utf8, true).into(),
        ];

        let args = ScalarFunctionArgs {
            args: vec![left, right],
            arg_fields,
            number_rows,
            return_field: Field::new("f", DataType::Boolean, true).into(),
            config_options: Arc::new(ConfigOptions::default()),
        };

        udf.invoke_with_args(args).unwrap()
    }

    #[test]
    fn test_contains_udf_scalar() {
        let array = ColumnarValue::Array(Arc::new(StringArray::from(vec![
            Some("xxx?()"),
            Some("yyy?()"),
        ])));
        let scalar = ColumnarValue::Scalar(ScalarValue::Utf8(Some("x?(".to_string())));
        let result = invoke_contains_udf(array, scalar, 2);
        let expect =
            ColumnarValue::Array(Arc::new(BooleanArray::from(vec![Some(true), Some(false)])));
        assert_eq!(
            *result.into_array(2).unwrap(),
            *expect.into_array(2).unwrap()
        );
    }

    #[test]
    fn test_contains_udf_scalar_other_types() {
        let array = ColumnarValue::Array(Arc::new(LargeStringArray::from(vec![
            Some("xxx?()"),
            Some("yyy?()"),
        ])));
        let scalar = ColumnarValue::Scalar(ScalarValue::LargeUtf8(Some("x?(".to_string())));
        let result1 = invoke_contains_udf(array, scalar, 2);

        let array = ColumnarValue::Array(Arc::new(StringViewArray::from(vec![
            Some("xxx?()"),
            Some("yyy?()"),
        ])));
        let scalar = ColumnarValue::Scalar(ScalarValue::Utf8View(Some("x?(".to_string())));
        let result2 = invoke_contains_udf(array, scalar, 2);

        let expect =
            ColumnarValue::Array(Arc::new(BooleanArray::from(vec![Some(true), Some(false)])));
        let expect_arr = expect.into_array(2).unwrap();

        assert_eq!(*result1.into_array(2).unwrap(), expect_arr);
        assert_eq!(*result2.into_array(2).unwrap(), expect_arr);
    }

    #[test]
    fn test_column_contains_column() {
        let array = ColumnarValue::Array(Arc::new(LargeStringArray::from(vec![
            Some("albert"),
            Some("arrow"),
            Some("parquet"),
        ])));
        let right = ColumnarValue::Array(Arc::new(LargeStringArray::from(vec![
            Some("a"),
            Some("b"),
            Some("c"),
        ])));
        let result1 = invoke_contains_udf(array, right, 3);

        let expect = ColumnarValue::Array(Arc::new(BooleanArray::from(vec![
            Some(true),
            Some(false),
            Some(false),
        ])));
        assert_eq!(
            *result1.into_array(3).unwrap(),
            *expect.into_array(3).unwrap()
        );
    }

    #[test]
    fn test_contains_udf_dict_scalar() {
        let array = ColumnarValue::Array(Arc::new(DictionaryArray::new(
            UInt8Array::from_iter_values([0, 1, 1, 0, 2]),
            Arc::new(StringArray::from_iter_values([
                "arrow", "parquet", "barquet",
            ])),
        )));
        let scalar = ColumnarValue::Scalar(ScalarValue::Utf8(Some("que".to_string())));
        let result = invoke_contains_udf(array, scalar, 5);
        let expect = ColumnarValue::Array(Arc::new(BooleanArray::from(vec![
            Some(false),
            Some(true),
            Some(true),
            Some(false),
            Some(true),
        ])));
        assert_eq!(
            *result.into_array(2).unwrap(),
            *expect.into_array(2).unwrap()
        );
    }

    #[test]
    fn test_contains_udf_scalar_scalar() {
        let array = ColumnarValue::Scalar(ScalarValue::Utf8(Some("arrow".to_string())));
        let scalar = ColumnarValue::Scalar(ScalarValue::Utf8(Some("b".to_string())));
        let result = invoke_contains_udf(array, scalar, 1);
        let expect = ColumnarValue::Scalar(ScalarValue::Boolean(Some(false)));
        assert_eq!(
            *result.into_array(1).unwrap(),
            *expect.into_array(1).unwrap()
        );
    }

    #[test]
    fn test_contains_udf_scalar_arr() {
        let scalar = ColumnarValue::Scalar(ScalarValue::Utf8(Some("terry".to_string())));
        let array = ColumnarValue::Array(Arc::new(StringArray::from_iter_values(["a", "e", "i"])));
        let result = invoke_contains_udf(scalar, array, 3);
        let expect = ColumnarValue::Array(Arc::new(BooleanArray::from(vec![
            Some(false),
            Some(true),
            Some(false),
        ])));
        assert_eq!(
            *result.into_array(3).unwrap(),
            *expect.into_array(3).unwrap()
        );
    }

    #[test]
    fn test_contains_udf_dict_array() {
        let array = ColumnarValue::Array(Arc::new(DictionaryArray::new(
            UInt8Array::from_iter_values([0, 1, 1, 0, 2]),
            Arc::new(StringArray::from_iter_values([
                "arrow", "parquet", "barquet",
            ])),
        )));
        let right = ColumnarValue::Array(Arc::new(StringArray::from_iter_values(vec![
            "a", "b", "c", "d", "e",
        ])));
        let result = invoke_contains_udf(array, right, 5);
        let expect = ColumnarValue::Array(Arc::new(BooleanArray::from(vec![
            Some(true),
            Some(false),
            Some(false),
            Some(false),
            Some(true),
        ])));
        assert_eq!(
            *result.into_array(5).unwrap(),
            *expect.into_array(5).unwrap()
        );
    }

    #[test]
    fn test_contains_udf_dict_contains_dict() {
        let array = ColumnarValue::Array(Arc::new(DictionaryArray::new(
            UInt8Array::from_iter_values([0, 1, 1, 0, 2]),
            Arc::new(StringArray::from_iter_values([
                "arrow", "parquet", "barquet",
            ])),
        )));
        let right = ColumnarValue::Array(Arc::new(DictionaryArray::new(
            UInt8Array::from_iter_values([0, 1, 1, 0, 0]),
            Arc::new(StringArray::from_iter_values(["a", "t"])),
        )));
        let result = invoke_contains_udf(array, right, 5);
        let expect = ColumnarValue::Array(Arc::new(BooleanArray::from(vec![
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
        ])));
        assert_eq!(
            *result.into_array(5).unwrap(),
            *expect.into_array(5).unwrap()
        );
    }

    #[test]
    fn test_contains_api() {
        use crate::pipeline::functions::expr_fn::contains;
        let expr = contains(
            Expr::Literal(
                ScalarValue::Utf8(Some("the quick brown fox".to_string())),
                None,
            ),
            Expr::Literal(ScalarValue::Utf8(Some("row".to_string())), None),
        );

        assert_eq!(
            expr.to_string(),
            "contains(Utf8(\"the quick brown fox\"), Utf8(\"row\"))"
        );
    }
}
