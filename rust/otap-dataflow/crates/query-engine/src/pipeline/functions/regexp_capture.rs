// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, AsArray, DictionaryArray, PrimitiveArray, StringArray,
    StringArrayType, StringBuilder, as_dictionary_array, make_array,
};
use arrow::buffer::{BooleanBuffer, NullBuffer};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, DataType, Field, FieldRef, Int8Type, Int16Type,
    Int32Type, Int64Type, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow::util::bit_util;
use datafusion::common::exec_err;
use datafusion::common::types::{
    LogicalUnionFields, NativeType, logical_int64, logical_string, logical_uint16,
};
use datafusion::error::Result;
use datafusion::functions::regex::compile_regex;
use datafusion::logical_expr::{
    Coercion, ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl,
    Signature, TypeSignature, TypeSignatureClass, Volatility,
};
use datafusion::scalar::ScalarValue;

const FUNC_NAME: &str = "regexp_capture";

/// This scalar UDF implementation takes three arguments: source (string), regex (string) and
/// capture group (int) and returns the portion of the text from the source that is matched by
/// the capture group.
///
/// For example, if passed: `regexp_capture("hello world", ".*hello (.*).*", 1)`, it would
/// return `"world"`. It returns `None` if the regex does not match the source, or if the
/// capture group is not in the regex.
///
/// Capture group 0 represents the full regex. For example if passed
/// `regexp_capture("hello world", ".*hello (.*).*", 1)`, this would return "hello world".
///
/// # Type information:
///
/// If the source is a dictionary, this will try to keep the return type as a dictionary array.
/// However, if different regexs/capture groups are passed for each row of input, we can't
/// guarantee the dictionary won't overflow. For this reason, if the regex/capture group args
/// are not scalars, the return type will be the native string array.
///
///
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
                    Coercion::new_exact(TypeSignatureClass::Native(Arc::new(NativeType::String))),
                    Coercion::new_exact(TypeSignatureClass::Native(Arc::new(NativeType::String))),
                    Coercion::new_exact(TypeSignatureClass::Integer),
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
        FUNC_NAME
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

        let sources = &args.args[0];
        let regexes = &args.args[1];
        let groups = &args.args[2];
        let len = match sources {
            ColumnarValue::Array(arr) => arr.len(),
            ColumnarValue::Scalar(_) => 1,
        };
        let source_arr = sources.to_array(len)?;

        let result = match source_arr.data_type() {
            DataType::Utf8 => {
                let str_arr = source_arr.as_string::<i32>();
                regex_capture(str_arr.iter(), regexes, groups)
            }
            DataType::Dictionary(k, v) => {
                if !matches!(v.as_ref(), &DataType::Utf8) {
                    return exec_err!(
                        "Unsupported data type {:?} for function `{}`",
                        source_arr.data_type(),
                        self.name()
                    );
                }

                match k.as_ref() {
                    DataType::Int8 => {
                        invoke_for_dict_source::<Int8Type>(&source_arr, regexes, groups)
                    }
                    DataType::Int16 => {
                        invoke_for_dict_source::<Int16Type>(&source_arr, regexes, groups)
                    }
                    DataType::Int32 => {
                        invoke_for_dict_source::<Int32Type>(&source_arr, regexes, groups)
                    }
                    DataType::Int64 => {
                        invoke_for_dict_source::<Int64Type>(&source_arr, regexes, groups)
                    }
                    DataType::UInt8 => {
                        invoke_for_dict_source::<UInt8Type>(&source_arr, regexes, groups)
                    }
                    DataType::UInt16 => {
                        invoke_for_dict_source::<UInt16Type>(&source_arr, regexes, groups)
                    }
                    DataType::UInt32 => {
                        invoke_for_dict_source::<UInt32Type>(&source_arr, regexes, groups)
                    }
                    DataType::UInt64 => {
                        invoke_for_dict_source::<UInt64Type>(&source_arr, regexes, groups)
                    }
                    other => exec_err!("Invalid dictionary key {other:?}"),
                }
            }
            other => {
                exec_err!(
                    "Unsupported data type {other:?} for function `{}`",
                    self.name()
                )
            }
        }?;

        // TODO - maybe coerce back to scalar ..
        Ok(ColumnarValue::Array(result))
    }

    fn documentation(&self) -> Option<&Documentation> {
        None
    }
}

fn invoke_for_dict_source<K: ArrowDictionaryKeyType>(
    source_arr: &ArrayRef,
    regexes: &ColumnarValue,
    groups: &ColumnarValue,
) -> Result<ArrayRef> {
    let dict_arr = source_arr.as_dictionary::<K>();
    let dict_vals = dict_arr.values();
    match (regexes, groups) {
        (ColumnarValue::Scalar(_), ColumnarValue::Scalar(_)) => {
            // fast path, can just match the regexes against the dicts values and
            // convert them to matched patterns
            let dict_vals_str = dict_vals.as_string::<i32>();
            let new_vals = regex_capture(dict_vals_str.iter(), regexes, groups)?;
            replace_dict_values(&dict_arr, new_vals)
        }
        _ => {
            let sources_iter = dict_arr
                .downcast_dict::<StringArray>()
                .expect("dict vals are strings");
            regex_capture(sources_iter.into_iter(), regexes, groups)
        }
    }
}

/// This is called after we've matched the scalar regex on the dictionary values.
///
/// It replaces the values array in the dictionary with the result which contains portions of
/// the dict values (which were used as input) that matched the regex capture group. For dict
/// values that didn't match, this handles swapping the nulls from the dict values null buffer
/// to the keys null buffer.
fn replace_dict_values<K: ArrowDictionaryKeyType>(
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
            other => {
                exec_err!(
                    "Unsupported data type for regexes array {other:?} in function `{}`",
                    FUNC_NAME
                )
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
        ColumnarValue::Array(arr) => match arr.data_type() {
            DataType::Int8 => {
                regex_capture_with_group_primitive_arr::<_, _, Int8Type>(values, regexes, arr)
            }
            DataType::Int16 => {
                regex_capture_with_group_primitive_arr::<_, _, Int16Type>(values, regexes, arr)
            }
            DataType::Int32 => {
                regex_capture_with_group_primitive_arr::<_, _, Int32Type>(values, regexes, arr)
            }
            DataType::Int64 => {
                regex_capture_with_group_primitive_arr::<_, _, Int64Type>(values, regexes, arr)
            }
            DataType::UInt8 => {
                regex_capture_with_group_primitive_arr::<_, _, UInt8Type>(values, regexes, arr)
            }
            DataType::UInt16 => {
                regex_capture_with_group_primitive_arr::<_, _, UInt16Type>(values, regexes, arr)
            }
            DataType::UInt32 => {
                regex_capture_with_group_primitive_arr::<_, _, UInt32Type>(values, regexes, arr)
            }
            DataType::UInt64 => {
                regex_capture_with_group_primitive_arr::<_, _, UInt64Type>(values, regexes, arr)
            }
            _ => {
                todo!()
            }
        },
        ColumnarValue::Scalar(scalar) => {
            let unsigned_scalar = scalar.cast_to(&DataType::UInt32)?;
            let ScalarValue::UInt32(group) = unsigned_scalar else {
                unreachable!("we've casted this to UInt32")
            };
            let group = group.map(|i| i as usize);
            regex_capture_with_group_iter(values, regexes, std::iter::repeat(group))
        }
    }
}

fn regex_capture_with_group_primitive_arr<'a, 'b, I1, I2, T: ArrowPrimitiveType>(
    values: I1,
    regexes: I2,
    groups: &ArrayRef,
) -> Result<ArrayRef>
where
    I1: Iterator<Item = Option<&'a str>>,
    I2: Iterator<Item = Option<&'b str>>,
{
    let zero = T::Native::default();
    regex_capture_with_group_iter(
        values,
        regexes,
        groups
            .as_primitive::<T>()
            .iter()
            .map(|i| i.and_then(|i| (i >= zero).then_some(i.as_usize()))),
    )
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
    use arrow::array::{
        Array, ArrayRef, BinaryArray, DictionaryArray, Int8Array, Int16Array, Int32Array,
        Int64Array, LargeStringArray, RecordBatch, StringArray, StringViewArray, UInt8Array,
        UInt16Array, UInt32Array, UInt64Array,
    };
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

    #[tokio::test]
    async fn test_capture_group_not_in_regex() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), lit(2u16)],
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

        let expected: ArrayRef = Arc::new(StringArray::new_null(3));
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
    async fn test_zero_capture_group_matches_full_text() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), lit(0u16)],
        ));

        let input_col = StringArray::from_iter_values(["hello world", "hello otap", "arrow"]);

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "test_col",
                input_col.data_type().clone(),
                true,
            )])),
            vec![Arc::new(input_col.clone())],
        )
        .unwrap();

        let df_schema =
            DFSchema::from_unqualified_fields(input.schema().fields.clone(), Default::default())
                .unwrap();

        let physical_expr =
            create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                .unwrap();

        let result = physical_expr.evaluate(&input).unwrap();

        let expected: ArrayRef = Arc::new(StringArray::from_iter([
            Some("hello world"),
            Some("hello otap"),
            None,
        ]));
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
    async fn test_invalid_but_coercible_source_input_types_are_rejected() {
        let inputs: Vec<ArrayRef> = vec![
            Arc::new(LargeStringArray::from_iter_values(["hello world"])),
            Arc::new(StringViewArray::from_iter_values(["hello world"])),
            Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values([0]),
                Arc::new(StringViewArray::from_iter_values(["hello world"])),
            )),
        ];

        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), lit(0u16)],
        ));

        for input_col in inputs {
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![Field::new(
                    "test_col",
                    input_col.data_type().clone(),
                    true,
                )])),
                vec![Arc::new(input_col.clone())],
            )
            .unwrap();

            let df_schema = DFSchema::from_unqualified_fields(
                input.schema().fields.clone(),
                Default::default(),
            )
            .unwrap();

            let physical_expr =
                create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                    .unwrap();

            let error = physical_expr.evaluate(&input).unwrap_err();
            assert!(
                error.to_string().contains("Unsupported data type"),
                "unexpected error: `{}`",
                error
            )
        }
    }

    #[tokio::test]
    async fn test_invalid_but_coercible_regex_input_arrays_are_rejected() {
        let inputs: Vec<ArrayRef> = vec![
            Arc::new(LargeStringArray::from_iter_values([".*(.).*"])),
            Arc::new(StringViewArray::from_iter_values([".*(.).*"])),
            Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values([0]),
                Arc::new(StringViewArray::from_iter_values([".*(.).*"])),
            )),
        ];

        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), col("regex_col"), lit(0u16)],
        ));

        for regex_col in inputs {
            let input_col = StringArray::from_iter_values(["hello"]);
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("test_col", input_col.data_type().clone(), true),
                    Field::new("regex_col", regex_col.data_type().clone(), true),
                ])),
                vec![Arc::new(input_col.clone()), Arc::new(regex_col.clone())],
            )
            .unwrap();

            let df_schema = DFSchema::from_unqualified_fields(
                input.schema().fields.clone(),
                Default::default(),
            )
            .unwrap();

            let physical_expr =
                create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                    .unwrap();

            let error = physical_expr.evaluate(&input).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("Unsupported data type for regexes array"),
                "unexpected error: `{}`",
                error
            )
        }
    }

    #[tokio::test]
    async fn test_coercing_scalar_integer_types() {
        let session_context = Pipeline::create_session_context();

        for group_expr in vec![
            lit(1u8),
            lit(1u16),
            lit(1u32),
            lit(1u64),
            lit(1i8),
            lit(1i16),
            lit(1i32),
            lit(1i64),
        ] {
            let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
                regexp_capture(),
                vec![col("test_col"), lit("hello (.*)"), group_expr],
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

            let df_schema = DFSchema::from_unqualified_fields(
                input.schema().fields.clone(),
                Default::default(),
            )
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
    }

    #[tokio::test]
    async fn test_coercing_scalar_array_types() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), col("group_col")],
        ));

        let group_cols: Vec<ArrayRef> = vec![
            Arc::new(UInt8Array::from_iter_values([1, 1, 1])),
            Arc::new(UInt16Array::from_iter_values([1, 1, 1])),
            Arc::new(UInt32Array::from_iter_values([1, 1, 1])),
            Arc::new(UInt64Array::from_iter_values([1, 1, 1])),
            Arc::new(Int8Array::from_iter_values([1, 1, 1])),
            Arc::new(Int16Array::from_iter_values([1, 1, 1])),
            Arc::new(Int32Array::from_iter_values([1, 1, 1])),
            Arc::new(Int64Array::from_iter_values([1, 1, 1])),
        ];

        for group_col in group_cols {
            let input_col = StringArray::from_iter_values(["hello world", "hello otap", "arrow"]);
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("test_col", input_col.data_type().clone(), true),
                    Field::new("group_col", group_col.data_type().clone(), true),
                ])),
                vec![Arc::new(input_col), Arc::new(group_col)],
            )
            .unwrap();

            let df_schema = DFSchema::from_unqualified_fields(
                input.schema().fields.clone(),
                Default::default(),
            )
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
    }

    #[tokio::test]
    async fn test_coercing_scalar_array_types_with_negative_vals() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), col("group_col")],
        ));

        let group_cols: Vec<ArrayRef> = vec![
            Arc::new(Int8Array::from_iter_values([-1])),
            Arc::new(Int16Array::from_iter_values([-1])),
            Arc::new(Int32Array::from_iter_values([-1])),
            Arc::new(Int64Array::from_iter_values([-1])),
        ];

        for group_col in group_cols {
            let input_col = StringArray::from_iter_values(["hello world"]);
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("test_col", input_col.data_type().clone(), true),
                    Field::new("group_col", group_col.data_type().clone(), true),
                ])),
                vec![Arc::new(input_col), Arc::new(group_col)],
            )
            .unwrap();

            let df_schema = DFSchema::from_unqualified_fields(
                input.schema().fields.clone(),
                Default::default(),
            )
            .unwrap();

            let physical_expr =
                create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                    .unwrap();

            let result = physical_expr.evaluate(&input).unwrap();

            let expected: ArrayRef = Arc::new(StringArray::new_null(1));
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

    #[tokio::test]
    async fn test_coercing_negative_signed_scalar_integer_types() {
        let session_context = Pipeline::create_session_context();

        for group_expr in vec![lit(-1i8), lit(-1i16), lit(-1i32), lit(-1i64)] {
            let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
                regexp_capture(),
                vec![col("test_col"), lit("hello (.*)"), group_expr],
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

            let df_schema = DFSchema::from_unqualified_fields(
                input.schema().fields.clone(),
                Default::default(),
            )
            .unwrap();

            let physical_expr =
                create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                    .unwrap();

            let error = physical_expr.evaluate(&input).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("Can't cast value -1 to type UInt"),
                "unexpected error message {error}"
            );
        }
    }
}
