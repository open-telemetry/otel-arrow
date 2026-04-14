// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, AsArray, DictionaryArray, PrimitiveArray, StringArray, StringBuilder,
    make_array,
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
use datafusion::functions::regex::compile_and_cache_regex;
use datafusion::logical_expr::{
    Coercion, ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl,
    Signature, TypeSignature, TypeSignatureClass, Volatility,
};
use datafusion::scalar::ScalarValue;

const FUNC_NAME: &str = "regexp_substr";

/// This scalar UDF implements the SQL Server `REGEXP_SUBSTR` function signature:
///
/// ```text
/// regexp_substr(string_expression, pattern [, start [, occurrence [, flags [, group]]]])
/// ```
///
/// It returns the portion of `string_expression` that matches the `pattern` (or a capture
/// group within it). Returns `NULL` if no match is found.
///
/// # Parameters
///
/// - `string_expression` (string): The source text to search.
/// - `pattern` (string): The regular expression pattern.
/// - `start` (integer, optional, default `1`): 1-based character position to begin searching.
///   Must be >= 1. If greater than the string length, returns NULL.
/// - `occurrence` (integer, optional, default `1`): Which occurrence of the pattern to return.
///   Must be >= 1.
/// - `flags` (string, optional): Regex modifier flags passed to the regex engine. Supported:
///   `i` (case-insensitive), `m` (multi-line), `s` (dot-all), `c` (case-sensitive, default).
/// - `group` (integer, optional, default `0`): Which capture group to return. `0` returns the
///   full match. If the group is not present in the pattern, returns NULL. Negative → NULL.
///
/// # Return Type
///
/// If the source is a dictionary and all other arguments are scalars (or absent), the return
/// type preserves the dictionary encoding. Otherwise the return type is `Utf8`.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RegexpSubstrFunc {
    signature: Signature,
}

impl Default for RegexpSubstrFunc {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a `Coercible` type signature for a given arity (2..=6).
///
/// Parameter types in order:
///   0: String  (source)
///   1: String  (pattern)
///   2: Integer (start)
///   3: Integer (occurrence)
///   4: String  (flags)
///   5: Integer (group)
fn coercible_for_arity(n: usize) -> TypeSignature {
    let all_types: [Coercion; 6] = [
        Coercion::new_exact(TypeSignatureClass::Native(Arc::new(NativeType::String))),
        Coercion::new_exact(TypeSignatureClass::Native(Arc::new(NativeType::String))),
        Coercion::new_exact(TypeSignatureClass::Integer),
        Coercion::new_exact(TypeSignatureClass::Integer),
        Coercion::new_exact(TypeSignatureClass::Native(Arc::new(NativeType::String))),
        Coercion::new_exact(TypeSignatureClass::Integer),
    ];
    TypeSignature::Coercible(all_types.into_iter().take(n).collect())
}

/// All parameter names (truncated to match the arity at construction).
const PARAM_NAMES: &[&str] = &["str", "pattern", "start", "occurrence", "flags", "group"];

/// Build a arity-6 variant where the group param (6th) is a string (named group).
fn coercible_for_arity_6_named_group() -> TypeSignature {
    TypeSignature::Coercible(vec![
        Coercion::new_exact(TypeSignatureClass::Native(Arc::new(NativeType::String))),
        Coercion::new_exact(TypeSignatureClass::Native(Arc::new(NativeType::String))),
        Coercion::new_exact(TypeSignatureClass::Integer),
        Coercion::new_exact(TypeSignatureClass::Integer),
        Coercion::new_exact(TypeSignatureClass::Native(Arc::new(NativeType::String))),
        Coercion::new_exact(TypeSignatureClass::Native(Arc::new(NativeType::String))),
    ])
}

impl RegexpSubstrFunc {
    pub fn new() -> Self {
        let mut variants: Vec<TypeSignature> = (2..=6).map(coercible_for_arity).collect();
        // Add a second arity-6 variant where the group param is a string (named group).
        variants.push(coercible_for_arity_6_named_group());

        Self {
            signature: Signature::one_of(variants, Volatility::Immutable)
                .with_parameter_names(PARAM_NAMES.iter().map(|s| s.to_string()).collect())
                // safety: these parameter names are valid because they cover the max arity
                // of the signature and there are no duplicates, so it is safe to expect here
                .expect("valid parameter names"),
        }
    }
}

impl ScalarUDFImpl for RegexpSubstrFunc {
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
        // won't be called this since we implement return_field_from_args
        Err(DataFusionError::Internal(format!(
            "return_type should not be called on {}",
            FUNC_NAME
        )))
    }

    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let n = args.arg_fields.len();
        if !(2..=6).contains(&n) || args.scalar_arguments.len() != n {
            return exec_err!(
                "{} `return_field_from_args` called with invalid number of args. expected 2-6",
                FUNC_NAME
            );
        }

        // Preserve the source data type (e.g. dictionary encoding) only when every
        // non-source argument is a scalar. If any is a non-scalar array we fall back
        // to Utf8.
        let all_extra_scalar = args.scalar_arguments[1..].iter().all(|s| s.is_some());
        let return_type = if all_extra_scalar {
            args.arg_fields[0].data_type().clone()
        } else {
            DataType::Utf8
        };

        Ok(Arc::new(Field::new(self.name(), return_type, true)))
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let n = args.args.len();
        if !(2..=6).contains(&n) {
            return exec_err!(
                "{} called with invalid number of args. expected 2-6. got {}",
                FUNC_NAME,
                n
            );
        }

        let sources = &args.args[0];
        let patterns = &args.args[1];
        let starts = if n >= 3 {
            args.args[2].clone()
        } else {
            ColumnarValue::Scalar(ScalarValue::Int64(Some(1)))
        };
        let occurrences = if n >= 4 {
            args.args[3].clone()
        } else {
            ColumnarValue::Scalar(ScalarValue::Int64(Some(1)))
        };
        let flags = if n >= 5 {
            args.args[4].clone()
        } else {
            ColumnarValue::Scalar(ScalarValue::Utf8(None))
        };
        let groups = if n >= 6 {
            args.args[5].clone()
        } else {
            ColumnarValue::Scalar(ScalarValue::Int64(Some(0)))
        };

        let len = match sources {
            ColumnarValue::Array(arr) => arr.len(),
            ColumnarValue::Scalar(_) => 1,
        };
        let source_arr = sources.to_array(len)?;

        let result = match source_arr.data_type() {
            DataType::Utf8 => {
                let str_arr = source_arr.as_string::<i32>();
                regex_substr(
                    str_arr.iter(),
                    patterns,
                    &starts,
                    &occurrences,
                    &flags,
                    &groups,
                )
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
                    DataType::Int8 => invoke_for_dict_source::<Int8Type>(
                        &source_arr,
                        patterns,
                        &starts,
                        &occurrences,
                        &flags,
                        &groups,
                    ),
                    DataType::Int16 => invoke_for_dict_source::<Int16Type>(
                        &source_arr,
                        patterns,
                        &starts,
                        &occurrences,
                        &flags,
                        &groups,
                    ),
                    DataType::Int32 => invoke_for_dict_source::<Int32Type>(
                        &source_arr,
                        patterns,
                        &starts,
                        &occurrences,
                        &flags,
                        &groups,
                    ),
                    DataType::Int64 => invoke_for_dict_source::<Int64Type>(
                        &source_arr,
                        patterns,
                        &starts,
                        &occurrences,
                        &flags,
                        &groups,
                    ),
                    DataType::UInt8 => invoke_for_dict_source::<UInt8Type>(
                        &source_arr,
                        patterns,
                        &starts,
                        &occurrences,
                        &flags,
                        &groups,
                    ),
                    DataType::UInt16 => invoke_for_dict_source::<UInt16Type>(
                        &source_arr,
                        patterns,
                        &starts,
                        &occurrences,
                        &flags,
                        &groups,
                    ),
                    DataType::UInt32 => invoke_for_dict_source::<UInt32Type>(
                        &source_arr,
                        patterns,
                        &starts,
                        &occurrences,
                        &flags,
                        &groups,
                    ),
                    DataType::UInt64 => invoke_for_dict_source::<UInt64Type>(
                        &source_arr,
                        patterns,
                        &starts,
                        &occurrences,
                        &flags,
                        &groups,
                    ),
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

        let all_scalar = args
            .args
            .iter()
            .all(|a| matches!(a, ColumnarValue::Scalar(_)));

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
    patterns: &ColumnarValue,
    starts: &ColumnarValue,
    occurrences: &ColumnarValue,
    flags: &ColumnarValue,
    groups: &ColumnarValue,
) -> Result<ArrayRef> {
    let dict_arr = source_arr.as_dictionary::<K>();
    let dict_vals = dict_arr.values();

    let all_extra_scalar = matches!(patterns, ColumnarValue::Scalar(_))
        && matches!(starts, ColumnarValue::Scalar(_))
        && matches!(occurrences, ColumnarValue::Scalar(_))
        && matches!(flags, ColumnarValue::Scalar(_))
        && matches!(groups, ColumnarValue::Scalar(_));

    if all_extra_scalar {
        // Fast path: match against dictionary values only, then remap keys.
        let dict_vals_str = dict_vals.as_string::<i32>();
        let new_vals = regex_substr(
            dict_vals_str.iter(),
            patterns,
            starts,
            occurrences,
            flags,
            groups,
        )?;
        replace_dict_values(dict_arr, new_vals)
    } else {
        // Slow path: expand dictionary to per-row strings.
        let typed_str_dict = dict_arr
            .downcast_dict::<StringArray>()
            .expect("dict vals are strings");
        regex_substr(
            typed_str_dict.into_iter(),
            patterns,
            starts,
            occurrences,
            flags,
            groups,
        )
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

fn regex_substr<'a, I>(
    values: I,
    patterns: &ColumnarValue,
    starts: &ColumnarValue,
    occurrences: &ColumnarValue,
    flags: &ColumnarValue,
    groups: &ColumnarValue,
) -> Result<ArrayRef>
where
    I: Iterator<Item = Option<&'a str>>,
{
    match patterns {
        ColumnarValue::Array(pattern_arr) => {
            let casted = cast_str_array_to_utf8(pattern_arr)?;
            let str_arr = casted.as_string::<i32>();
            regex_substr_with_pattern_iter(
                values,
                str_arr.iter(),
                starts,
                occurrences,
                flags,
                groups,
            )
        }
        ColumnarValue::Scalar(regex_scalar) => {
            let regex_scalar_str = regex_scalar.cast_to(&DataType::Utf8)?;
            let ScalarValue::Utf8(str_regex) = regex_scalar_str else {
                unreachable!("we've casted this to Utf8")
            };
            regex_substr_with_pattern_iter(
                values,
                std::iter::repeat(str_regex.as_deref()),
                starts,
                occurrences,
                flags,
                groups,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Resolve the flags ColumnarValue into an iterator
// ---------------------------------------------------------------------------

fn regex_substr_with_pattern_iter<'a, 'b, I1, I2>(
    values: I1,
    patterns: I2,
    starts: &ColumnarValue,
    occurrences: &ColumnarValue,
    flags: &ColumnarValue,
    groups: &ColumnarValue,
) -> Result<ArrayRef>
where
    I1: Iterator<Item = Option<&'a str>>,
    I2: Iterator<Item = Option<&'b str>>,
{
    match flags {
        ColumnarValue::Array(flags_arr) => {
            let casted = cast_str_array_to_utf8(flags_arr)?;
            let str_arr = casted.as_string::<i32>();
            regex_substr_with_flags_iter(
                values,
                patterns,
                str_arr.iter(),
                starts,
                occurrences,
                groups,
            )
        }
        ColumnarValue::Scalar(flags_scalar) => {
            let flags_opt = match flags_scalar {
                ScalarValue::Utf8(s) => s.clone(),
                ScalarValue::Null => None,
                other => {
                    let casted = other.cast_to(&DataType::Utf8)?;
                    let ScalarValue::Utf8(s) = casted else {
                        unreachable!("we've casted this to Utf8")
                    };
                    s
                }
            };
            regex_substr_with_flags_iter(
                values,
                patterns,
                std::iter::repeat(flags_opt.as_deref()),
                starts,
                occurrences,
                groups,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Resolve the `start` ColumnarValue into an iterator
// ---------------------------------------------------------------------------

/// Cast any integer array (including dictionary-encoded) to `Int64Array`.
///
/// This avoids the combinatorial monomorphization explosion that would result
/// from dispatching on every possible (key, value) type pair for dictionary
/// integer arrays.
fn cast_int_array_to_int64(arr: &ArrayRef) -> Result<ArrayRef> {
    arrow::compute::cast(arr, &DataType::Int64)
        .map_err(|e| DataFusionError::ArrowError(Box::new(e), None))
}

/// Cast any string array (including dictionary-encoded) to `StringArray` (Utf8).
fn cast_str_array_to_utf8(arr: &ArrayRef) -> Result<ArrayRef> {
    arrow::compute::cast(arr, &DataType::Utf8)
        .map_err(|e| DataFusionError::ArrowError(Box::new(e), None))
}

fn regex_substr_with_flags_iter<'a, 'b, 'c, I1, I2, I3>(
    values: I1,
    patterns: I2,
    flags: I3,
    starts: &ColumnarValue,
    occurrences: &ColumnarValue,
    groups: &ColumnarValue,
) -> Result<ArrayRef>
where
    I1: Iterator<Item = Option<&'a str>>,
    I2: Iterator<Item = Option<&'b str>>,
    I3: Iterator<Item = Option<&'c str>>,
{
    match starts {
        ColumnarValue::Array(arr) => {
            let casted = cast_int_array_to_int64(arr)?;
            let int_arr = casted.as_primitive::<Int64Type>();
            regex_substr_with_start_iter(
                values,
                patterns,
                flags,
                int_arr
                    .iter()
                    .map(|i| i.and_then(|i| (i >= 1).then_some(i as usize))),
                occurrences,
                groups,
            )
        }
        ColumnarValue::Scalar(scalar) => {
            let start_val = columnar_int_scalar_to_usize(scalar)?;
            regex_substr_with_start_iter(
                values,
                patterns,
                flags,
                std::iter::repeat(start_val),
                occurrences,
                groups,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Resolve the `occurrence` ColumnarValue into an iterator
// ---------------------------------------------------------------------------

fn regex_substr_with_start_iter<'a, 'b, 'c, I1, I2, I3, I4>(
    values: I1,
    patterns: I2,
    flags: I3,
    starts: I4,
    occurrences: &ColumnarValue,
    groups: &ColumnarValue,
) -> Result<ArrayRef>
where
    I1: Iterator<Item = Option<&'a str>>,
    I2: Iterator<Item = Option<&'b str>>,
    I3: Iterator<Item = Option<&'c str>>,
    I4: Iterator<Item = Option<usize>>,
{
    match occurrences {
        ColumnarValue::Array(arr) => {
            let casted = cast_int_array_to_int64(arr)?;
            let int_arr = casted.as_primitive::<Int64Type>();
            regex_substr_with_occ_iter(
                values,
                patterns,
                flags,
                starts,
                int_arr
                    .iter()
                    .map(|i| i.and_then(|i| (i >= 1).then_some(i as usize))),
                groups,
            )
        }
        ColumnarValue::Scalar(scalar) => {
            let occ_val = columnar_int_scalar_to_usize(scalar)?;
            regex_substr_with_occ_iter(
                values,
                patterns,
                flags,
                starts,
                std::iter::repeat(occ_val),
                groups,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Resolve the `group` ColumnarValue into an iterator
// ---------------------------------------------------------------------------

fn regex_substr_with_occ_iter<'a, 'b, 'c, I1, I2, I3, I4, I5>(
    values: I1,
    patterns: I2,
    flags: I3,
    starts: I4,
    occurrences: I5,
    groups: &ColumnarValue,
) -> Result<ArrayRef>
where
    I1: Iterator<Item = Option<&'a str>>,
    I2: Iterator<Item = Option<&'b str>>,
    I3: Iterator<Item = Option<&'c str>>,
    I4: Iterator<Item = Option<usize>>,
    I5: Iterator<Item = Option<usize>>,
{
    match groups {
        ColumnarValue::Array(arr) => {
            if arr.data_type().is_integer()
                || matches!(arr.data_type(), DataType::Dictionary(_, v) if v.is_integer())
            {
                // Integer group array — capture group by index
                let casted = cast_int_array_to_int64(arr)?;
                let int_arr = casted.as_primitive::<Int64Type>();
                regex_substr_core(
                    values,
                    patterns,
                    flags,
                    starts,
                    occurrences,
                    int_arr.iter().map(|i| {
                        i.and_then(|i| (i >= 0).then_some(CaptureGroupRef::Index(i as usize)))
                    }),
                )
            } else {
                // String group array — capture group by name
                let casted = cast_str_array_to_utf8(arr)?;
                let str_arr = casted.as_string::<i32>();
                regex_substr_core(
                    values,
                    patterns,
                    flags,
                    starts,
                    occurrences,
                    str_arr.iter().map(|s| s.map(CaptureGroupRef::Name)),
                )
            }
        }
        ColumnarValue::Scalar(scalar) => match scalar {
            // String scalar — capture group by name
            ScalarValue::Utf8(name) => {
                let group = name.as_deref().map(CaptureGroupRef::Name);
                regex_substr_core(
                    values,
                    patterns,
                    flags,
                    starts,
                    occurrences,
                    std::iter::repeat(group),
                )
            }
            // Integer scalar — capture group by index
            _ => {
                let group = match scalar.cast_to(&DataType::UInt64) {
                    Ok(unsigned_scalar) => {
                        let ScalarValue::UInt64(group) = unsigned_scalar else {
                            unreachable!("we've casted this to UInt64")
                        };
                        group.map(|i| CaptureGroupRef::Index(i as usize))
                    }
                    // if we couldn't cast, it means we had a negative integer scalar
                    // so always treat the group as null, meaning the result for all
                    // the rows will be null.
                    _ => None,
                };

                regex_substr_core(
                    values,
                    patterns,
                    flags,
                    starts,
                    occurrences,
                    std::iter::repeat(group),
                )
            }
        },
    }
}

// ---------------------------------------------------------------------------
// Helpers & core matching logic
// ---------------------------------------------------------------------------

/// Convert a scalar integer ColumnarValue to `Option<usize>`.
///
/// Returns `Some(n)` for a positive integer, `None` for null or values that
/// cannot be represented as a non-negative usize (e.g. negative integers).
fn columnar_int_scalar_to_usize(scalar: &ScalarValue) -> Result<Option<usize>> {
    match scalar.cast_to(&DataType::UInt64) {
        Ok(unsigned_scalar) => {
            let ScalarValue::UInt64(val) = unsigned_scalar else {
                unreachable!("we've casted this to UInt64")
            };
            Ok(val.map(|i| i as usize))
        }
        // negative integer → treat as null
        _ => Ok(None),
    }
}

/// A reference to a capture group, either by numeric index or by name.
#[derive(Clone, Copy)]
enum CaptureGroupRef<'a> {
    Index(usize),
    Name(&'a str),
}

/// The innermost matching function. All six parameter iterators have been fully
/// resolved by this point.
///
/// For each row this function:
/// 1. Applies the `start` offset (1-based character position) to the source.
/// 2. Compiles the regex with the given flags.
/// 3. Finds the `occurrence`-th match (1-based).
/// 4. Extracts the specified capture `group`.
fn regex_substr_core<'a, 'b, 'c, 'd, I1, I2, I3, I4, I5, I6>(
    values: I1,
    patterns: I2,
    flags: I3,
    starts: I4,
    occurrences: I5,
    groups: I6,
) -> Result<ArrayRef>
where
    I1: Iterator<Item = Option<&'a str>>,
    I2: Iterator<Item = Option<&'b str>>,
    I3: Iterator<Item = Option<&'c str>>,
    I4: Iterator<Item = Option<usize>>,
    I5: Iterator<Item = Option<usize>>,
    I6: Iterator<Item = Option<CaptureGroupRef<'d>>>,
{
    let mut regex_cache = HashMap::new();
    let mut result_builder = StringBuilder::new();
    let mut null_run = 0;

    let iter = values
        .zip(patterns)
        .zip(flags)
        .zip(starts)
        .zip(occurrences)
        .zip(groups);

    for (((((value, pattern), flag), start), occurrence), group) in iter {
        match (value, pattern, start, occurrence, group) {
            (Some(value), Some(pattern), Some(start), Some(occurrence), Some(group)) => {
                // Apply the start offset (1-based char position → byte offset).
                let byte_offset = if start <= 1 {
                    0
                } else {
                    match value.char_indices().nth(start - 1) {
                        Some((byte_idx, _)) => byte_idx,
                        // start is beyond the string length → null
                        None => {
                            null_run += 1;
                            continue;
                        }
                    }
                };
                let search_str = &value[byte_offset..];

                let regex = compile_and_cache_regex(pattern, flag, &mut regex_cache)?;

                // Find the occurrence-th match (1-based).
                let matched = if occurrence <= 1 {
                    regex.captures(search_str)
                } else {
                    regex.captures_iter(search_str).nth(occurrence - 1)
                };

                if let Some(captures) = matched {
                    let capture = match group {
                        CaptureGroupRef::Index(i) => captures.get(i),
                        CaptureGroupRef::Name(n) => captures.name(n),
                    };
                    if let Some(capture) = capture {
                        if null_run > 0 {
                            result_builder.append_nulls(null_run);
                        }
                        result_builder.append_value(&search_str[capture.start()..capture.end()]);
                        null_run = 0;
                    } else {
                        null_run += 1;
                    }
                } else {
                    null_run += 1;
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
        Schema, UInt8Type, UInt16Type, UInt64Type,
    };
    use datafusion::common::DFSchema;
    use datafusion::logical_expr::ColumnarValue;
    use datafusion::logical_expr::{Expr, col, expr::ScalarFunction, lit};
    use datafusion::physical_expr::create_physical_expr;
    use datafusion::scalar::ScalarValue;
    use std::sync::Arc;

    use crate::pipeline::Pipeline;
    use crate::pipeline::functions::regexp_substr;

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

    /// Helper for testing regexp capture group behaviour: build 6-arg form of args
    fn capture_args(source: Expr, pattern: Expr, group: Expr) -> Vec<Expr> {
        vec![
            source,
            pattern,
            lit(1i64),                    // start
            lit(1i64),                    // occurrence
            lit(ScalarValue::Utf8(None)), // flags
            group,                        // group
        ]
    }

    #[tokio::test]
    async fn test_capture_utf8_arr_values_scalar_pattern_scalar_group() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), lit(1u16)),
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

    async fn test_capture_dict_utf8_arr_values_scalar_pattern_scalar_group_generic<
        K: ArrowDictionaryKeyType,
    >() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), lit(1u16)),
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
    async fn test_capture_dict_utf8_arr_values_scalar_pattern_scalar_group() {
        test_capture_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<UInt8Type>().await;
        test_capture_dict_utf8_arr_values_scalar_pattern_scalar_group_generic::<UInt16Type>().await;
    }

    #[tokio::test]
    async fn test_capture_dict_utf8_arr_values_with_nulls_scalar_pattern_scalar_group() {
        // test to ensure we preserve the original nulls when reconciling the
        // nulls from dict values that didn't match the passed scalar regex
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), lit(1u16)),
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
    async fn test_capture_dict_utf8_arr_values_dict_utf8_arr_pattern_scalar_group() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), col("regexp_col"), lit(1u16)),
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
    async fn test_capture_utf8_arr_utf8_arr_pattern_scalar_group() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), col("regexp_col"), lit(1u16)),
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

    async fn test_capture_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic<
        K: ArrowDictionaryKeyType,
    >() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), col("regexp_col"), lit(1u16)),
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
    async fn test_capture_utf8_arr_dict_utf8_arr_pattern_scalar_group() {
        test_capture_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<UInt8Type>().await;
        test_capture_utf8_arr_dict_utf8_arr_pattern_scalar_group_generic::<UInt16Type>().await;
    }

    #[tokio::test]
    async fn test_capture_capture_group_not_in_regex() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), lit(2u16)),
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
    async fn test_capture_zero_capture_group_matches_full_text() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), lit(0u16)),
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
    async fn test_capture_invalid_but_coercible_source_input_types_are_rejected() {
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
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), lit(0u16)),
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
    async fn test_capture_coercible_regex_input_arrays_are_cast_to_utf8() {
        let inputs: Vec<ArrayRef> = vec![
            Arc::new(LargeStringArray::from_iter_values([".*(.).*"])),
            Arc::new(StringViewArray::from_iter_values([".*(.).*"])),
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
            regexp_substr(),
            capture_args(col("test_col"), col("regex_col"), lit(0u16)),
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

            let result = physical_expr.evaluate(&input).unwrap();
            let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["hello"]));
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
    async fn test_capture_coercing_scalar_integer_types() {
        let session_context = Pipeline::create_session_context();

        for group_expr in [
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
                regexp_substr(),
                capture_args(col("test_col"), lit("hello (.*)"), group_expr),
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
    async fn test_capture_coercing_scalar_array_types() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), col("group_col")),
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
    async fn test_capture_coercing_scalar_array_types_with_negative_vals() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), col("group_col")),
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
    async fn test_capture_coercing_negative_signed_scalar_integer_types() {
        let session_context = Pipeline::create_session_context();

        for group_expr in [lit(-1i8), lit(-1i16), lit(-1i32), lit(-1i64)] {
            let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
                regexp_substr(),
                capture_args(col("test_col"), lit("hello (.*)"), group_expr),
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
    async fn test_capture_dict_encoded_group_arrays() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), col("group_col")),
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
    async fn test_capture_dict_encoded_group_arrays_with_negative_vals() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), col("group_col")),
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
    async fn test_capture_dict_encoded_group_arrays_with_null_keys() {
        let session_context = Pipeline::create_session_context();
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(col("test_col"), lit("hello (.*)"), col("group_col")),
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
    async fn test_capture_all_scalar_args_returns_scalar() {
        let session_context = Pipeline::create_session_context();

        // matching case
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            capture_args(lit("hello world"), lit("hello (.*)"), lit(1u16)),
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
            regexp_substr(),
            capture_args(lit("arrow"), lit("hello (.*)"), lit(1u16)),
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

    // -----------------------------------------------------------------------
    // Tests for new regexp_substr parameters: start, occurrence, flags
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_two_arg_form_returns_full_match() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![col("test_col"), lit("hello \\w+")],
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
            Some("hello world"),
            None,
            Some("hello otap"),
            None,
        ]));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_start_parameter() {
        let session_context = Pipeline::create_session_context();

        // regexp_substr("xxhello world", "hello (\\w+)", start=3) — skip first 2 chars
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![col("test_col"), lit("hello (\\w+)"), lit(3i64)],
        ));

        let input_col =
            StringArray::from_iter_values(["xxhello world", "hello world", "xxhello otap"]);
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
        // start=3 means search from char index 2 onward:
        //   "xxhello world" -> search "hello world" -> full match "hello world"
        //   "hello world"   -> search "llo world"   -> no match (pattern starts with "hello")
        //   "xxhello otap"  -> search "hello otap"  -> full match "hello otap"
        let expected: ArrayRef = Arc::new(StringArray::from_iter([
            Some("hello world"),
            None,
            Some("hello otap"),
        ]));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_start_beyond_string_length_returns_null() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![col("test_col"), lit("hello"), lit(100i64)],
        ));

        let input_col = StringArray::from_iter_values(["hello world"]);
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
        let expected: ArrayRef = Arc::new(StringArray::new_null(1));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_occurrence_parameter() {
        let session_context = Pipeline::create_session_context();

        // Find the 2nd occurrence of a word
        // regexp_substr(source, "\\w+", start=1, occurrence=2)
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![col("test_col"), lit("\\w+"), lit(1i64), lit(2i64)],
        ));

        let input_col = StringArray::from_iter_values(["hello world", "one", "a b c"]);
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
        // "hello world" -> 2nd word match = "world"
        // "one"         -> only 1 match, 2nd doesn't exist = null
        // "a b c"       -> 2nd word match = "b"
        let expected: ArrayRef = Arc::new(StringArray::from_iter([Some("world"), None, Some("b")]));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_occurrence_beyond_matches_returns_null() {
        let session_context = Pipeline::create_session_context();

        // occurrence=10 but there's only 1 match
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![col("test_col"), lit("hello"), lit(1i64), lit(10i64)],
        ));

        let input_col = StringArray::from_iter_values(["hello world"]);
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
        let expected: ArrayRef = Arc::new(StringArray::new_null(1));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_flags_case_insensitive() {
        let session_context = Pipeline::create_session_context();

        // regexp_substr(source, "HELLO", start=1, occurrence=1, flags="i")
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![
                col("test_col"),
                lit("HELLO"),
                lit(1i64),
                lit(1i64),
                lit("i"),
            ],
        ));

        let input_col = StringArray::from_iter_values(["hello world", "HELLO WORLD", "no match"]);
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
        // Case-insensitive: "HELLO" matches "hello" and "HELLO", not "no match"
        let expected: ArrayRef =
            Arc::new(StringArray::from_iter([Some("hello"), Some("HELLO"), None]));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_start_and_occurrence_combined() {
        let session_context = Pipeline::create_session_context();

        // Start from position 6, find 2nd word
        // regexp_substr(source, "\\w+", start=6, occurrence=2)
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![col("test_col"), lit("\\w+"), lit(6i64), lit(2i64)],
        ));

        let input_col = StringArray::from_iter_values(["hello world foo bar"]);
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
        // start=6 -> search " world foo bar" (from char 5 onward)
        // 1st word match in " world foo bar" = "world"
        // 2nd word match = "foo"
        let expected: ArrayRef = Arc::new(StringArray::from_iter([Some("foo")]));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_all_six_args() {
        let session_context = Pipeline::create_session_context();

        // regexp_substr("xxHELLO world HELLO foo", "hello (\\w+)", 3, 2, "i", 1)
        // start=3 -> search from char 2: "HELLO world HELLO foo"
        // pattern with flag "i": case-insensitive "hello (\\w+)"
        // occurrence=2 -> 2nd match: "HELLO foo"
        // group=1 -> capture group 1: "foo"
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![
                col("test_col"),
                lit("hello (\\w+)"),
                lit(3i64),
                lit(2i64),
                lit("i"),
                lit(1i64),
            ],
        ));

        let input_col = StringArray::from_iter_values(["xxHELLO world HELLO foo"]);
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
        let expected: ArrayRef = Arc::new(StringArray::from_iter([Some("foo")]));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Named capture group tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_named_group_scalar() {
        let session_context = Pipeline::create_session_context();

        // regexp_substr(source, "hello (?P<rest>.*)", 1, 1, NULL, "rest")
        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![
                col("test_col"),
                lit("hello (?P<rest>.*)"),
                lit(1i64),
                lit(1i64),
                lit(ScalarValue::Utf8(None)),
                lit("rest"),
            ],
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
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_named_group_not_in_regex_returns_null() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![
                col("test_col"),
                lit("hello (.*)"),
                lit(1i64),
                lit(1i64),
                lit(ScalarValue::Utf8(None)),
                lit("nonexistent"),
            ],
        ));

        let input_col = StringArray::from_iter_values(["hello world", "hello otap"]);
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
        let expected: ArrayRef = Arc::new(StringArray::new_null(2));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_named_group_string_array() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![
                col("test_col"),
                lit("(?P<first>\\w+) (?P<second>\\w+)"),
                lit(1i64),
                lit(1i64),
                lit(ScalarValue::Utf8(None)),
                col("group_col"),
            ],
        ));

        let input_col = StringArray::from_iter_values(["hello world", "foo bar", "one two"]);
        let group_col = StringArray::from_iter_values(["first", "second", "first"]);

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
        let expected: ArrayRef = Arc::new(StringArray::from_iter_values(["hello", "bar", "one"]));
        match result {
            ColumnarValue::Array(arr) => assert_eq!(&arr, &expected),
            s => panic!("expected Array, got {s:?}"),
        }
    }

    #[tokio::test]
    async fn test_named_group_all_scalar() {
        let session_context = Pipeline::create_session_context();

        let plan = Expr::ScalarFunction(ScalarFunction::new_udf(
            regexp_substr(),
            vec![
                lit("hello world"),
                lit("hello (?P<rest>.*)"),
                lit(1i64),
                lit(1i64),
                lit(ScalarValue::Utf8(None)),
                lit("rest"),
            ],
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
    }
}
