// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, AsArray, DictionaryArray, PrimitiveArray, StringArray, StringArrayType,
    StringBuilder, as_dictionary_array, make_array,
};
use arrow::buffer::{BooleanBuffer, NullBuffer};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, DataType, Field, FieldRef, Int8Type, Int16Type,
    Int32Type, Int64Type, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow::util::bit_util;
use datafusion::common::types::{NativeType, logical_int64, logical_string, logical_uint16};
use datafusion::error::Result;
use datafusion::functions::regex::compile_regex;
use datafusion::logical_expr::{
    Coercion, ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl,
    Signature, TypeSignature, TypeSignatureClass, Volatility,
};
use datafusion::scalar::ScalarValue;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RegexpCaptureFunc {
    signature: Signature,
}

impl Default for RegexpCaptureFunc {
    fn default() -> Self {
        Self::new()
    }
}

impl RegexpCaptureFunc {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![TypeSignature::Coercible(vec![
                    Coercion::new_exact(TypeSignatureClass::Native(logical_string())),
                    // TODO - if we don't end up supporting something dynamic on the other side
                    // then we should maybe just make this 2nd arg a straight up str?
                    Coercion::new_exact(TypeSignatureClass::Native(logical_string())),
                    Coercion::new_implicit(
                        // TODO - should we allow that there are more than u16::Max capture groups?
                        TypeSignatureClass::Native(logical_uint16()),
                        vec![
                            // TODO are there others
                            TypeSignatureClass::Native(logical_int64()),
                        ],
                        NativeType::UInt16,
                    ),
                ])],
                Volatility::Immutable,
            )
            .with_parameter_names(vec![
                "str".to_string(),
                "pattern".to_string(),
                "capture_group".to_string(),
            ])
            .expect("valid parameter names"),
        }
    }
}

impl ScalarUDFImpl for RegexpCaptureFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "regexp_capture"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    // TODO - not supposed to call this .... there's another one where we can figure out the
    // return type from the args
    fn return_type(&self, arg_types: &[DataType]) -> Result<DataType> {
        // TODO - return an error because this shouldn't be called
        todo!()
    }

    fn return_field_from_args(&self, args: ReturnFieldArgs) -> Result<FieldRef> {
        // TODO len check on args & scalar arguments
        // TODO comment on why this is like this
        let return_type =
            if args.scalar_arguments[1].is_some() && args.scalar_arguments[2].is_some() {
                args.arg_fields[0].data_type().clone()
            } else {
                DataType::Utf8
            };

        Ok(Arc::new(Field::new(self.name(), return_type, true)))
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        if args.args.len() != 3 {
            todo!("return error")
        }

        let str_arr = &args.args[0];
        let regexes = &args.args[1];
        let groups = &args.args[2];
        let len = match str_arr {
            ColumnarValue::Array(arr) => arr.len(),
            ColumnarValue::Scalar(scalar) => 1,
        };
        let str_arr = str_arr.to_array(len)?;

        let result = match str_arr.data_type() {
            DataType::Utf8 => {
                let str_arr = str_arr.as_string::<i32>();
                regex_capture(str_arr.iter(), regexes, groups)
            }
            DataType::Dictionary(k, v) => {
                if !matches!(v.as_ref(), &DataType::Utf8) {
                    todo!("invalid type")
                }

                let dict_vals = get_dict_values(&str_arr, k.as_ref())?;
                match (regexes, groups) {
                    (ColumnarValue::Scalar(_), ColumnarValue::Scalar(_)) => {
                        // fast path, can just match the regexes against the dicts values and
                        // convert them to matched patterns
                        let dict_vals_str = dict_vals.as_string::<i32>();
                        let new_vals = regex_capture(dict_vals_str.iter(), regexes, groups)?;
                        replace_dict_values(&str_arr, k.as_ref(), new_vals)
                    }
                    _ => {
                        match k.as_ref() {
                            DataType::UInt8 => {
                                let dict_arr = as_dictionary_array::<UInt8Type>(&str_arr);

                                // safety: we've checked the value type is utf8 the
                                let strs = dict_arr
                                    .downcast_dict::<StringArray>()
                                    .expect("dict vals are strings");
                                regex_capture(strs.into_iter(), regexes, groups)
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                }
            }
            _ => {
                todo!()
            }
        }?;

        // TODO - maybe coerce back to scalar ..
        Ok(ColumnarValue::Array(result))
    }

    fn documentation(&self) -> Option<&Documentation> {
        None
    }
}

fn get_dict_values(dict_arr: &ArrayRef, dict_key_type: &DataType) -> Result<ArrayRef> {
    let dict_vals = match dict_key_type {
        DataType::UInt8 => as_dictionary_array::<UInt8Type>(&dict_arr).values(),
        DataType::UInt16 => as_dictionary_array::<UInt16Type>(&dict_arr).values(),
        DataType::UInt32 => as_dictionary_array::<UInt32Type>(&dict_arr).values(),
        DataType::UInt64 => as_dictionary_array::<UInt64Type>(&dict_arr).values(),
        DataType::Int8 => as_dictionary_array::<Int8Type>(&dict_arr).values(),
        DataType::Int16 => as_dictionary_array::<Int16Type>(&dict_arr).values(),
        DataType::Int32 => as_dictionary_array::<Int32Type>(&dict_arr).values(),
        DataType::Int64 => as_dictionary_array::<Int64Type>(&dict_arr).values(),
        _ => {
            todo!()
        }
    };
    Ok(Arc::clone(dict_vals))
}

fn replace_dict_values(
    dict_arr: &ArrayRef,
    dict_key_type: &DataType,
    new_values: ArrayRef,
) -> Result<ArrayRef> {
    match dict_key_type {
        DataType::UInt8 => {
            replace_dict_values_generic(as_dictionary_array::<UInt8Type>(&dict_arr), new_values)
        }
        DataType::UInt16 => {
            replace_dict_values_generic(as_dictionary_array::<UInt16Type>(&dict_arr), new_values)
        }
        DataType::UInt32 => {
            replace_dict_values_generic(as_dictionary_array::<UInt32Type>(&dict_arr), new_values)
        }
        DataType::UInt64 => {
            replace_dict_values_generic(as_dictionary_array::<UInt64Type>(&dict_arr), new_values)
        }
        DataType::Int8 => {
            replace_dict_values_generic(as_dictionary_array::<Int8Type>(&dict_arr), new_values)
        }
        DataType::Int16 => {
            replace_dict_values_generic(as_dictionary_array::<Int16Type>(&dict_arr), new_values)
        }
        DataType::Int32 => {
            replace_dict_values_generic(as_dictionary_array::<Int32Type>(&dict_arr), new_values)
        }
        DataType::Int64 => {
            replace_dict_values_generic(as_dictionary_array::<Int64Type>(&dict_arr), new_values)
        }
        _ => {
            todo!()
        }
    }
}

fn replace_dict_values_generic<K: ArrowDictionaryKeyType>(
    dict_arr: &DictionaryArray<K>,
    new_values: ArrayRef,
) -> Result<ArrayRef> {
    match new_values.nulls() {
        Some(val_nulls) => {
            let mut new_nulls = match dict_arr.keys().nulls() {
                Some(key_nulls) => key_nulls.buffer().to_vec(),
                None => vec![0xFF; bit_util::ceil(dict_arr.len(), 8)],
            };

            let dict_keys = dict_arr.keys();
            for i in 0..dict_arr.len() {
                let key = dict_keys.values()[i].as_usize();
                if val_nulls.is_null(key) {
                    bit_util::unset_bit(&mut new_nulls, i);
                }
            }

            let new_data = new_values.to_data().into_builder().nulls(None).build()?;

            Ok(Arc::new(DictionaryArray::new(
                PrimitiveArray::<K>::new(
                    dict_arr.keys().values().clone(),
                    Some(NullBuffer::new(BooleanBuffer::new(
                        new_nulls.into(),
                        0,
                        dict_arr.len(),
                    ))),
                ),
                make_array(new_data),
            )))
        }
        None => Ok(Arc::new(DictionaryArray::new(
            dict_arr.keys().clone(),
            new_values,
        ))),
    }
}

fn regex_capture<'a, I>(
    values: I,
    regexes: &ColumnarValue,
    groups: &ColumnarValue,
) -> Result<ArrayRef>
where
    I: Iterator<Item = Option<&'a str>>,
{
    match regexes {
        ColumnarValue::Array(regex_arr) => match regex_arr.data_type() {
            DataType::Utf8 => {
                let regex_arr_str = regex_arr.as_string::<i32>();
                regex_capture_with_regex_iter(values, regex_arr_str.iter(), groups)
            }
            _ => {
                todo!()
            }
        },
        ColumnarValue::Scalar(regex_scalar) => {
            let regex_scalar_str = regex_scalar.cast_to(&DataType::Utf8)?;
            let ScalarValue::Utf8(str_regex) = regex_scalar_str else {
                unreachable!("we've casted this to Utf8")
            };
            regex_capture_with_regex_iter(values, std::iter::repeat(str_regex.as_deref()), groups)
        }
    }
}

fn regex_capture_with_regex_iter<'a, 'b, I1, I2>(
    values: I1,
    regexes: I2,
    groups: &ColumnarValue,
) -> Result<ArrayRef>
where
    I1: Iterator<Item = Option<&'a str>>,
    I2: Iterator<Item = Option<&'b str>>,
{
    match groups {
        ColumnarValue::Array(arr) => {
            todo!()
        }
        ColumnarValue::Scalar(scalar) => {
            // TODO maybe like check if it's negative (at least to know what happens)
            let u64_scalar = scalar.cast_to(&DataType::UInt64)?;
            let ScalarValue::UInt64(group) = u64_scalar else {
                unreachable!("we've casted this to UInt64")
            };
            let group = group.map(|i| i as usize);
            regex_capture_with_group_iter(values, regexes, std::iter::repeat(group))
        }
    }
}

fn regex_capture_with_group_iter<'a, 'b, I1, I2, I3>(
    values: I1,
    regexes: I2,
    groups: I3,
) -> Result<ArrayRef>
where
    I1: Iterator<Item = Option<&'a str>>,
    I2: Iterator<Item = Option<&'b str>>,
    I3: Iterator<Item = Option<usize>>,
{
    let mut result_builder = StringBuilder::new();
    let mut null_run = 0;

    for ((value, regex), group) in values.zip(regexes).zip(groups) {
        match (value, regex, group) {
            (Some(value), Some(regex), Some(group)) => {
                let regex = compile_regex(regex, None)?;
                if let Some(capture) = regex.captures(value).map(|c| c.get(group)).flatten() {
                    if null_run > 0 {
                        result_builder.append_nulls(null_run);
                    }

                    result_builder.append_value(&value[capture.start()..capture.end()]);
                    null_run = 0;
                } else {
                    null_run += 1
                }
            }
            _ => {
                null_run += 1;
            }
        }
    }
    if null_run > 0 {
        result_builder.append_nulls(null_run);
    }

    Ok(Arc::new(result_builder.finish()))
}

#[cfg(test)]
mod test {
    use arrow::array::{Array, ArrayRef, DictionaryArray, RecordBatch, StringArray, UInt8Array};
    use arrow::datatypes::{Field, Schema};
    use datafusion::common::DFSchema;
    use datafusion::logical_expr::ColumnarValue;
    use datafusion::logical_expr::{Expr, col, expr::ScalarFunction, lit};
    use datafusion::physical_expr::create_physical_expr;
    use std::sync::Arc;

    use crate::pipeline::Pipeline;
    use crate::pipeline::functions::regexp_capture;

    // TODO test cases:
    // - negative group
    // - zero group
    // - uncoercable data types

    #[tokio::test]
    async fn test_utf8_arr_values_scalar_pattern_scalar_group() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), lit(1u16)],
        ));

        let input_col = StringArray::from_iter_values(["hello world", "hello otap", "arrow"]);

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "test_col",
                input_col.data_type().clone(),
                true,
            )])),
            vec![Arc::new(input_col)],
        )
        .unwrap();

        let df_schema =
            DFSchema::from_unqualified_fields(input.schema().fields.clone(), Default::default())
                .unwrap();

        let physical_expr =
            create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                .unwrap();

        let result = physical_expr.evaluate(&input).unwrap();

        let expected: ArrayRef =
            Arc::new(StringArray::from_iter([Some("world"), Some("otap"), None]));
        match result {
            ColumnarValue::Array(arr) => {
                assert_eq!(&arr, &expected)
            }
            s => {
                panic!("invalid ColumnarValue variant ColumnarValue::Scalar({s:?})")
            }
        }
    }

    #[tokio::test]
    async fn test_dict_utf8_arr_values_scalar_pattern_scalar_group() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), lit(1u16)],
        ));

        let input_col = DictionaryArray::new(
            UInt8Array::from_iter_values(vec![0, 0, 1]),
            Arc::new(StringArray::from_iter_values(["hello world", "arrow"])),
        );

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "test_col",
                input_col.data_type().clone(),
                true,
            )])),
            vec![Arc::new(input_col)],
        )
        .unwrap();

        let df_schema =
            DFSchema::from_unqualified_fields(input.schema().fields.clone(), Default::default())
                .unwrap();

        let physical_expr =
            create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                .unwrap();

        let result = physical_expr.evaluate(&input).unwrap();

        let expected: ArrayRef = Arc::new(DictionaryArray::new(
            UInt8Array::from_iter([Some(0), Some(0), None]),
            Arc::new(StringArray::from_iter_values(["world", ""])),
        ));
        match result {
            ColumnarValue::Array(arr) => {
                assert_eq!(&arr, &expected)
            }
            s => {
                panic!("invalid ColumnarValue variant ColumnarValue::Scalar({s:?})")
            }
        }
    }

    #[tokio::test]
    async fn test_dict_utf8_arr_values_dict_utf8_arr_pattern_scalar_group() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), col("regexp_col"), lit(1u16)],
        ));

        let input_col = DictionaryArray::new(
            UInt8Array::from_iter_values(vec![0, 0, 1]),
            Arc::new(StringArray::from_iter_values(["hello world", "arrow"])),
        );
        let regex_col = StringArray::from_iter_values([".(.).*", "..(..).*", "(arr)(ow)"]);

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("test_col", input_col.data_type().clone(), true),
                Field::new("regexp_col", regex_col.data_type().clone(), true),
            ])),
            vec![Arc::new(input_col), Arc::new(regex_col)],
        )
        .unwrap();

        let df_schema =
            DFSchema::from_unqualified_fields(input.schema().fields.clone(), Default::default())
                .unwrap();

        let physical_expr =
            create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                .unwrap();

        let result = physical_expr.evaluate(&input).unwrap();

        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["e", "ll", "arr"]));
        match result {
            ColumnarValue::Array(arr) => {
                assert_eq!(&arr, &expected)
            }
            s => {
                panic!("invalid ColumnarValue variant ColumnarValue::Scalar({s:?})")
            }
        }
    }

    #[tokio::test]
    async fn test_utf8_arr_utf8_arr_pattern_scalar_group() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), col("regexp_col"), lit(1u16)],
        ));

        let input_col = StringArray::from_iter_values(["hello world", "hello otap", "arrow"]);
        let regex_col = StringArray::from_iter_values([".(.).*", "..(..).*", "(arr)(ow)"]);

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("test_col", input_col.data_type().clone(), true),
                Field::new("regexp_col", regex_col.data_type().clone(), true),
            ])),
            vec![Arc::new(input_col), Arc::new(regex_col)],
        )
        .unwrap();

        let df_schema =
            DFSchema::from_unqualified_fields(input.schema().fields.clone(), Default::default())
                .unwrap();

        let physical_expr =
            create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                .unwrap();

        let result = physical_expr.evaluate(&input).unwrap();

        let expected: ArrayRef =
            Arc::new(StringArray::from_iter([Some("e"), Some("ll"), Some("arr")]));
        match result {
            ColumnarValue::Array(arr) => {
                assert_eq!(&arr, &expected)
            }
            s => {
                panic!("invalid ColumnarValue variant ColumnarValue::Scalar({s:?})")
            }
        }
    }
}
