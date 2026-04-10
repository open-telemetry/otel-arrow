// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, AsArray, DictionaryArray, PrimitiveArray, StringArray,
    StringBuilder, make_array,
};
use arrow::buffer::{BooleanBuffer, NullBuffer};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, DataType, Field, FieldRef, Int8Type, Int16Type,
    Int32Type, Int64Type, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow::util::bit_util;
use datafusion::common::exec_err;
use datafusion::common::types::NativeType;
use datafusion::error::{DataFusionError, Result};
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
/// If capture group < 0, this always returns null.
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

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        // datafusion won't call this since we implement return_field_from_args
        return Err(DataFusionError::Internal(format!(
            "return_type should not be called on {}",
            FUNC_NAME
        )));
    }

    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        if args.arg_fields.len() != 3 || args.scalar_arguments.len() != 3 {
            return exec_err!(
                "{} `return_field_from_args` called with invalid number of args. expected 3",
                FUNC_NAME
            );
        }
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
            return exec_err!(
                "{} `return_field_from_args` called with invalid number of args. expected 3. got {}",
                FUNC_NAME,
                args.args.len()
            );
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

        let all_scalar = matches!(sources, ColumnarValue::Scalar(_))
            && matches!(regexes, ColumnarValue::Scalar(_))
            && matches!(groups, ColumnarValue::Scalar(_));

        if all_scalar {
            let scalar = ScalarValue::try_from_array(&result, 0)?;
            Ok(ColumnarValue::Scalar(scalar))
        } else {
            Ok(ColumnarValue::Array(result))
        }
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
            let typed_str_dict = dict_arr
                .downcast_dict::<StringArray>()
                .expect("dict vals are strings");
            regex_capture(typed_str_dict.into_iter(), regexes, groups)
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
            DataType::Dictionary(k, v) => {
                if !matches!(v.as_ref(), &DataType::Utf8) {
                    return exec_err!(
                        "Unsupported dictionary values type for regexes array {:?} in function `{}`",
                        k.as_ref(),
                        FUNC_NAME
                    );
                }

                match k.as_ref() {
                    DataType::Int8 => {
                        invoke_for_dict_regex::<_, Int8Type>(values, regex_arr, groups)
                    }
                    DataType::Int16 => {
                        invoke_for_dict_regex::<_, Int16Type>(values, regex_arr, groups)
                    }
                    DataType::Int32 => {
                        invoke_for_dict_regex::<_, Int32Type>(values, regex_arr, groups)
                    }
                    DataType::Int64 => {
                        invoke_for_dict_regex::<_, Int64Type>(values, regex_arr, groups)
                    }
                    DataType::UInt8 => {
                        invoke_for_dict_regex::<_, UInt8Type>(values, regex_arr, groups)
                    }
                    DataType::UInt16 => {
                        invoke_for_dict_regex::<_, UInt16Type>(values, regex_arr, groups)
                    }
                    DataType::UInt32 => {
                        invoke_for_dict_regex::<_, UInt32Type>(values, regex_arr, groups)
                    }
                    DataType::UInt64 => {
                        invoke_for_dict_regex::<_, UInt64Type>(values, regex_arr, groups)
                    }
                    other => {
                        exec_err!(
                            "Unsupported dictionary key type for regexes array {other:?} in function `{}`",
                            FUNC_NAME
                        )
                    }
                }
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

/// Invoke the UDF for the case that the regex is a dictionary encoded array.
///
/// This expects that the type has already been verified to be `DataType::<K, Utf8>`
fn invoke_for_dict_regex<'a, I, K: ArrowDictionaryKeyType>(
    values: I,
    regex_dict_arr: &ArrayRef,
    groups: &ColumnarValue,
) -> Result<ArrayRef>
where
    I: Iterator<Item = Option<&'a str>>,
{
    let dict_arr = regex_dict_arr.as_dictionary::<K>();
    let typed_regex_dict = dict_arr
        .downcast_dict::<StringArray>()
        .expect("dict values are strings");
    regex_capture_with_regex_iter(values, typed_regex_dict.into_iter(), groups)
}

macro_rules! dispatch_dict_groups {
    ($values:expr, $regexes:expr, $groups:expr, $key_dt:expr, $val_dt:expr) => {
        paste::paste! {
            match $key_dt {
                DataType::Int8 => dispatch_dict_groups!(@val $values, $regexes, $groups, Int8Type, $val_dt),
                DataType::Int16 => dispatch_dict_groups!(@val $values, $regexes, $groups, Int16Type, $val_dt),
                DataType::Int32 => dispatch_dict_groups!(@val $values, $regexes, $groups, Int32Type, $val_dt),
                DataType::Int64 => dispatch_dict_groups!(@val $values, $regexes, $groups, Int64Type, $val_dt),
                DataType::UInt8 => dispatch_dict_groups!(@val $values, $regexes, $groups, UInt8Type, $val_dt),
                DataType::UInt16 => dispatch_dict_groups!(@val $values, $regexes, $groups, UInt16Type, $val_dt),
                DataType::UInt32 => dispatch_dict_groups!(@val $values, $regexes, $groups, UInt32Type, $val_dt),
                DataType::UInt64 => dispatch_dict_groups!(@val $values, $regexes, $groups, UInt64Type, $val_dt),
                other => exec_err!("Unsupported dictionary key type for groups: {other:?}"),
            }
        }
    };
    (@val $values:expr, $regexes:expr, $groups:expr, $ktyp:ty, $val_dt:expr) => {
        match $val_dt {
            DataType::Int8 => regex_capture_with_group_dict_arr::<_, _, $ktyp, Int8Type>($values, $regexes, $groups),
            DataType::Int16 => regex_capture_with_group_dict_arr::<_, _, $ktyp, Int16Type>($values, $regexes, $groups),
            DataType::Int32 => regex_capture_with_group_dict_arr::<_, _, $ktyp, Int32Type>($values, $regexes, $groups),
            DataType::Int64 => regex_capture_with_group_dict_arr::<_, _, $ktyp, Int64Type>($values, $regexes, $groups),
            DataType::UInt8 => regex_capture_with_group_dict_arr::<_, _, $ktyp, UInt8Type>($values, $regexes, $groups),
            DataType::UInt16 => regex_capture_with_group_dict_arr::<_, _, $ktyp, UInt16Type>($values, $regexes, $groups),
            DataType::UInt32 => regex_capture_with_group_dict_arr::<_, _, $ktyp, UInt32Type>($values, $regexes, $groups),
            DataType::UInt64 => regex_capture_with_group_dict_arr::<_, _, $ktyp, UInt64Type>($values, $regexes, $groups),
            other => exec_err!("Unsupported dictionary value type for groups: {other:?}"),
        }
    };
}

fn regex_capture_with_group_dict_arr<'a, 'b, I1, I2, K, V>(
    values: I1,
    regexes: I2,
    groups: &ArrayRef,
) -> Result<ArrayRef>
where
    I1: Iterator<Item = Option<&'a str>>,
    I2: Iterator<Item = Option<&'b str>>,
    K: ArrowDictionaryKeyType,
    V: ArrowPrimitiveType,
{
    let dict = groups.as_dictionary::<K>();
    let dict_values = dict.values().as_primitive::<V>();
    let zero = V::Native::default();
    regex_capture_with_group_iter(
        values,
        regexes,
        dict.keys().iter().map(|key| {
            key.and_then(|k| {
                let idx = k.as_usize();
                if dict_values.is_null(idx) {
                    None
                } else {
                    let v = dict_values.value(idx);
                    (v >= zero).then_some(v.as_usize())
                }
            })
        }),
    )
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
            DataType::Dictionary(k, v) => {
                dispatch_dict_groups!(values, regexes, arr, k.as_ref(), v.as_ref())
            }
            other => {
                exec_err!(
                    "Unsupported data type for groups array {other:?} in function `{}`",
                    FUNC_NAME
                )
            }
        },
        ColumnarValue::Scalar(scalar) => {
            let group = match scalar.cast_to(&DataType::UInt64) {
                Ok(unsigned_scalar) => {
                    let ScalarValue::UInt64(group) = unsigned_scalar else {
                        unreachable!("we've casted this to UInt64")
                    };
                    group.map(|i| i as usize)
                }

                // if we couldn't cast, it means we had a negative integer scalar
                // so always treat the group as null, meaning the result for all
                // the rows will be null.
                _ => None,
            };

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
        Array, ArrayRef, ArrowPrimitiveType, DictionaryArray, Int8Array, Int16Array, Int32Array,
        Int64Array, LargeStringArray, PrimitiveArray, RecordBatch, StringArray, StringViewArray,
        UInt8Array, UInt16Array, UInt32Array, UInt64Array,
    };
    use arrow::datatypes::{
        ArrowDictionaryKeyType, ArrowNativeType, Field, Int8Type, Int16Type, Int32Type, Int64Type,
        Schema, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
    };
    use datafusion::common::DFSchema;
    use datafusion::logical_expr::ColumnarValue;
    use datafusion::logical_expr::{Expr, col, expr::ScalarFunction, lit};
    use datafusion::physical_expr::create_physical_expr;
    use datafusion::scalar::ScalarValue;
    use std::sync::Arc;

    use crate::pipeline::Pipeline;
    use crate::pipeline::functions::regexp_capture;

    fn make_dict_array<K: ArrowDictionaryKeyType, V: ArrowPrimitiveType>(
        keys: &[K::Native],
        values: &[V::Native],
    ) -> ArrayRef {
        Arc::new(DictionaryArray::new(
            PrimitiveArray::<K>::from_iter_values(keys.iter().copied()),
            Arc::new(PrimitiveArray::<V>::from_iter_values(
                values.iter().copied(),
            )),
        ))
    }

    #[tokio::test]
    async fn test_utf8_arr_values_scalar_pattern_scalar_group() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), lit(1u16)],
        ));

        let input_col =
            StringArray::from_iter_values(["hello world", "otap", "hello otap", "arrow"]);

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

        let expected: ArrayRef = Arc::new(StringArray::from_iter([
            Some("world"),
            None,
            Some("otap"),
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

    async fn test_dict_utf8_arr_values_scalar_pattern_scalar_group_generic<
        K: ArrowDictionaryKeyType,
    >() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), lit(1u16)],
        ));

        let input_col = DictionaryArray::new(
            PrimitiveArray::<K>::from_iter_values(
                [0usize, 0, 1].map(|i| K::Native::from_usize(i).unwrap()),
            ),
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
            PrimitiveArray::<K>::from_iter(
                [Some(0usize), Some(0), None].map(|i| i.map(|i| K::Native::from_usize(i).unwrap())),
            ),
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
    async fn test_dict_utf8_arr_values_scalar_pattern_scalar_group() {
        test_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<Int8Type>().await;
        test_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<Int16Type>().await;
        test_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<Int32Type>().await;
        test_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<Int64Type>().await;
        test_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<UInt8Type>().await;
        test_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<UInt16Type>().await;
        test_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<UInt32Type>().await;
        test_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<UInt64Type>().await;
    }

    #[tokio::test]
    async fn test_dict_utf8_arr_values_with_nulls_scalar_pattern_scalar_group() {
        // test to ensure we preserve the original nulls when reconciling the
        // nulls from dict values that didn't match the passed scalar regex
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), lit(1u16)],
        ));

        let input_col = DictionaryArray::new(
            UInt8Array::from_iter([Some(0), Some(1), None]),
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
            UInt8Array::from_iter([Some(0), None, None]),
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

    async fn test_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic<K: ArrowDictionaryKeyType>() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), col("regexp_col"), lit(1u16)],
        ));

        let input_col = StringArray::from_iter_values(["hello world", "hello otap", "arrow"]);
        let regex_col = DictionaryArray::new(
            PrimitiveArray::<K>::from_iter_values(
                [0usize, 1usize, 2usize].map(|i| K::Native::from_usize(i).unwrap()),
            ),
            Arc::new(StringArray::from_iter_values([
                ".(.).*",
                "..(..).*",
                "(arr)(ow)",
            ])),
        );

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
    async fn test_utf8_arr_dict_utf8_arr_pattern_scalar_group() {
        test_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<Int8Type>().await;
        test_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<Int16Type>().await;
        test_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<Int32Type>().await;
        test_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<Int64Type>().await;
        test_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<UInt8Type>().await;
        test_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<UInt16Type>().await;
        test_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<UInt32Type>().await;
        test_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<UInt64Type>().await;
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
    async fn test_invalid_but_coercible_dict_regex_input_arrays_are_rejected() {
        let inputs: Vec<ArrayRef> = vec![
            Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values([0]),
                Arc::new(LargeStringArray::from_iter_values([".*(.).*"])),
            )),
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
                    .contains("Unsupported dictionary values type for regexes array"),
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
    }

    #[tokio::test]
    async fn test_dict_encoded_group_arrays() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), col("group_col")],
        ));

        // Each dict array maps all 3 rows to a single value of 1.
        // Tests a sample of key/value type combinations.
        let group_cols: Vec<ArrayRef> = vec![
            make_dict_array::<UInt8Type, Int32Type>(&[0, 0, 0], &[1]),
            make_dict_array::<Int16Type, UInt64Type>(&[0, 0, 0], &[1]),
            make_dict_array::<Int32Type, Int8Type>(&[0, 0, 0], &[1]),
            make_dict_array::<UInt64Type, UInt16Type>(&[0, 0, 0], &[1]),
            make_dict_array::<Int8Type, Int64Type>(&[0, 0, 0], &[1]),
        ];

        for group_col in group_cols {
            let input_col = StringArray::from_iter_values(["hello world", "hello otap", "arrow"]);
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("test_col", input_col.data_type().clone(), true),
                    Field::new("group_col", group_col.data_type().clone(), true),
                ])),
                vec![Arc::new(input_col), Arc::new(group_col.clone())],
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
                    assert_eq!(
                        &arr,
                        &expected,
                        "failed for group_col type {:?}",
                        group_col.data_type()
                    )
                }
                s => {
                    panic!("invalid ColumnarValue variant ColumnarValue::Scalar({s:?})")
                }
            }
        }
    }

    #[tokio::test]
    async fn test_dict_encoded_group_arrays_with_negative_vals() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), col("group_col")],
        ));

        let group_cols: Vec<ArrayRef> = vec![
            make_dict_array::<UInt8Type, Int8Type>(&[0], &[-1]),
            make_dict_array::<UInt8Type, Int16Type>(&[0], &[-1]),
            make_dict_array::<UInt8Type, Int32Type>(&[0], &[-1]),
            make_dict_array::<UInt8Type, Int64Type>(&[0], &[-1]),
        ];

        for group_col in group_cols {
            let input_col = StringArray::from_iter_values(["hello world"]);
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("test_col", input_col.data_type().clone(), true),
                    Field::new("group_col", group_col.data_type().clone(), true),
                ])),
                vec![Arc::new(input_col), Arc::new(group_col.clone())],
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
                    assert_eq!(
                        &arr,
                        &expected,
                        "failed for group_col type {:?}",
                        group_col.data_type()
                    )
                }
                s => {
                    panic!("invalid ColumnarValue variant ColumnarValue::Scalar({s:?})")
                }
            }
        }
    }

    #[tokio::test]
    async fn test_dict_encoded_group_arrays_with_null_keys() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![col("test_col"), lit("hello (.*)"), col("group_col")],
        ));

        // Key index 0 -> value 1, but second row has a null key
        let group_col: ArrayRef = Arc::new(DictionaryArray::new(
            PrimitiveArray::<UInt8Type>::from_iter([Some(0), None, Some(0)]),
            Arc::new(PrimitiveArray::<Int32Type>::from_iter_values([1])),
        ));

        let input_col = StringArray::from_iter_values(["hello world", "hello otap", "arrow"]);
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("test_col", input_col.data_type().clone(), true),
                Field::new("group_col", group_col.data_type().clone(), true),
            ])),
            vec![Arc::new(input_col), Arc::new(group_col)],
        )
        .unwrap();

        let df_schema =
            DFSchema::from_unqualified_fields(input.schema().fields.clone(), Default::default())
                .unwrap();

        let physical_expr =
            create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                .unwrap();

        let result = physical_expr.evaluate(&input).unwrap();

        let expected: ArrayRef = Arc::new(StringArray::from_iter([Some("world"), None, None]));
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
    async fn test_all_scalar_args_returns_scalar() {
        let session_context = Pipeline::create_session_context();

        // matching case
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![lit("hello world"), lit("hello (.*)"), lit(1u16)],
        ));

        let input = RecordBatch::new_empty(Arc::new(Schema::empty()));
        let df_schema = DFSchema::empty();

        let physical_expr =
            create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                .unwrap();

        let result = physical_expr.evaluate(&input).unwrap();
        match result {
            ColumnarValue::Scalar(ScalarValue::Utf8(Some(s))) => assert_eq!(s, "world"),
            other => panic!("expected Scalar(Utf8(Some(\"world\"))), got {other:?}"),
        }

        // non-matching case
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_capture(),
            vec![lit("arrow"), lit("hello (.*)"), lit(1u16)],
        ));

        let physical_expr =
            create_physical_expr(&plan, &df_schema, session_context.state().execution_props())
                .unwrap();

        let result = physical_expr.evaluate(&input).unwrap();
        match result {
            ColumnarValue::Scalar(ScalarValue::Utf8(None)) => {}
            other => panic!("expected Scalar(Utf8(None)), got {other:?}"),
        }
    }
}
