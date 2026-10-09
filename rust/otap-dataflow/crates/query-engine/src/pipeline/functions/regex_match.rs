// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Regex match against a pattern compiled once at planning time.
//!
//! This mirrors DataFusion's `BinaryExpr` with `Operator::RegexMatch` against a scalar
//! pattern (`regex_match_dyn_scalar` -> `arrow_string::regexp::regexp_is_match_scalar`),
//! except that the arrow kernel calls `Regex::new` on every invocation, i.e. once per
//! batch. Here the compiled [`Regex`] is held by the UDF instance so evaluation only
//! searches.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use arrow::array::{
    Array, ArrayAccessor, ArrayRef, AsArray, BooleanArray, BooleanBufferBuilder, StringArrayType,
    downcast_dictionary_array,
};
use arrow::buffer::{BooleanBuffer, MutableBuffer, NullBuffer};
use arrow::datatypes::DataType;
use datafusion::common::exec_err;
use datafusion::error::Result;
use datafusion::logical_expr::{
    ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
};
use datafusion::scalar::ScalarValue;
use regex::Regex;

const FUNC_NAME: &str = "regex_match";

/// Scalar UDF `regex_match(haystack)` returning whether each string matches the
/// precompiled pattern. Null inputs produce null outputs.
#[derive(Debug)]
pub struct RegexMatchFunc {
    regex: Regex,
    signature: Signature,
}

impl RegexMatchFunc {
    pub fn new(regex: Regex) -> Self {
        Self {
            regex,
            signature: Signature::any(1, Volatility::Immutable),
        }
    }
}

impl PartialEq for RegexMatchFunc {
    fn eq(&self, other: &Self) -> bool {
        self.regex.as_str() == other.regex.as_str()
    }
}

impl Eq for RegexMatchFunc {}

impl Hash for RegexMatchFunc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.regex.as_str().hash(state);
    }
}

impl ScalarUDFImpl for RegexMatchFunc {
    fn name(&self) -> &str {
        FUNC_NAME
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Boolean)
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        if args.args.len() != 1 {
            return exec_err!(
                "invalid number of args. {} expected 1 arg, found {}",
                FUNC_NAME,
                args.args.len()
            );
        }

        match &args.args[0] {
            ColumnarValue::Array(arr) => {
                Ok(ColumnarValue::Array(regex_match_array(&self.regex, arr)?))
            }
            ColumnarValue::Scalar(s) => {
                Ok(ColumnarValue::Scalar(regex_match_scalar(&self.regex, s)?))
            }
        }
    }
}

fn regex_match_array(regex: &Regex, arr: &ArrayRef) -> Result<ArrayRef> {
    let result: ArrayRef = match arr.data_type() {
        DataType::Utf8 => Arc::new(is_match(regex, arr.as_string::<i32>())),
        DataType::LargeUtf8 => Arc::new(is_match(regex, arr.as_string::<i64>())),
        DataType::Utf8View => Arc::new(is_match(regex, arr.as_string_view())),
        DataType::Dictionary(_, _) => {
            // Evaluate once per distinct value, then expand through the keys.

            // TODO: Consider refactoring `is_match` to return `MutableBuffer`
            // instead of `BooleanArray`. This way we can convert the mutable buffer
            // to an array when needed here. This will save us an Arc for dicts
            // don't need a full BooleanArray for the values, just the keys.
            let values = arr.as_any_dictionary().values();
            let evaluated_values = regex_match_array(regex, values)?;
            let evaluated_values = evaluated_values.as_boolean();
            downcast_dictionary_array! {
                arr => {
                    let unpacked = evaluated_values
                        .take_iter(arr.keys().iter().map(|k| k.map(|v| v as usize)))
                        .collect::<BooleanArray>();
                    Arc::new(unpacked) as ArrayRef
                },
                other => return exec_err!("{FUNC_NAME}: unsupported dictionary {other}"),
            }
        }
        other => return exec_err!("{FUNC_NAME}: unsupported haystack type {other}"),
    };
    Ok(result)
}

fn regex_match_scalar(regex: &Regex, s: &ScalarValue) -> Result<ScalarValue> {
    match s {
        ScalarValue::Utf8(v) | ScalarValue::LargeUtf8(v) | ScalarValue::Utf8View(v) => Ok(
            ScalarValue::Boolean(v.as_deref().map(|v| regex.is_match(v))),
        ),
        ScalarValue::Dictionary(_, inner) => regex_match_scalar(regex, inner),
        other => exec_err!(
            "{FUNC_NAME}: unsupported haystack type {}",
            other.data_type()
        ),
    }
}

/// Copy of the loop in `arrow_string::regexp::regexp_is_match_scalar` minus the
/// per-call `Regex::new`.
fn is_match<'a, S>(regex: &Regex, array: &'a S) -> BooleanArray
where
    &'a S: StringArrayType<'a>,
{
    let values = BooleanBuffer::collect_bool(array.len(), |i| regex.is_match(array.value(i)));
    let nulls = array
        .nulls()
        .map(|n| n.inner().sliced())
        .and_then(|b| NullBuffer::from_unsliced_buffer(b, array.len()));
    BooleanArray::new(values, nulls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{DictionaryArray, StringArray, StringViewArray};
    use arrow::datatypes::UInt16Type;

    fn re(p: &str) -> Regex {
        Regex::new(p).unwrap()
    }

    // Scenario: match a case-insensitive pattern against a Utf8 array containing a null.
    // Guarantees: non-null values report their match result and null inputs stay null.
    #[test]
    fn test_utf8_with_nulls() {
        let arr: ArrayRef = Arc::new(StringArray::from(vec![
            Some("an ERROR here"),
            None,
            Some("all good"),
        ]));
        let out = regex_match_array(&re("(?i)error"), &arr).unwrap();
        assert_eq!(
            out.as_boolean(),
            &BooleanArray::from(vec![Some(true), None, Some(false)])
        );
    }

    // Scenario: match a pattern against a Utf8View array.
    // Guarantees: Utf8View haystacks are supported and produce per-element match results.
    #[test]
    fn test_utf8_view() {
        let arr: ArrayRef = Arc::new(StringViewArray::from(vec!["a@b.c", "nope"]));
        let out = regex_match_array(&re("[a-z]+@[a-z.]+"), &arr).unwrap();
        assert_eq!(out.as_boolean(), &BooleanArray::from(vec![true, false]));
    }

    // Scenario: match a pattern against a dictionary-encoded array with a null and a repeated key.
    // Guarantees: the pattern is evaluated per distinct value and expanded through the keys, with
    // nulls preserved.
    #[test]
    fn test_dictionary() {
        let arr: ArrayRef = Arc::new(
            vec![Some("x"), Some("panic!"), None, Some("x")]
                .into_iter()
                .collect::<DictionaryArray<UInt16Type>>(),
        );
        let out = regex_match_array(&re("panic"), &arr).unwrap();
        assert_eq!(
            out.as_boolean(),
            &BooleanArray::from(vec![Some(false), Some(true), None, Some(false)])
        );
    }

    // Scenario: match a pattern against scalar Utf8 values, including a null scalar.
    // Guarantees: a non-null scalar yields its boolean match and a null scalar yields a null boolean.
    #[test]
    fn test_scalar() {
        let r = re("^a");
        assert_eq!(
            regex_match_scalar(&r, &ScalarValue::Utf8(Some("abc".into()))).unwrap(),
            ScalarValue::Boolean(Some(true))
        );
        assert_eq!(
            regex_match_scalar(&r, &ScalarValue::Utf8(None)).unwrap(),
            ScalarValue::Boolean(None)
        );
    }
}
