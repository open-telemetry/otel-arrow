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

use arrow::datatypes::DataType;
use datafusion::common::exec_err;
use datafusion::error::Result;
use datafusion::logical_expr::{
    ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
};
use datafusion::scalar::ScalarValue;
use otel_arrow_dfe_pdata::otap::filter::Pattern;
use regex::Regex;

const FUNC_NAME: &str = "regex_match";

/// Scalar UDF `regex_match(haystack)` returning whether each string matches the
/// precompiled pattern. Null inputs produce null outputs.
#[derive(Debug)]
pub struct RegexMatchFunc {
    pattern: Pattern,
    signature: Signature,
}

impl RegexMatchFunc {
    pub fn new(regex: Regex) -> Self {
        Self {
            pattern: Pattern::Regex(regex),
            signature: Signature::any(1, Volatility::Immutable),
        }
    }
}

impl PartialEq for RegexMatchFunc {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for RegexMatchFunc {}

impl Hash for RegexMatchFunc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self as *const Self).hash(state);
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
                // `Pattern::matches` preserves input nulls (as this UDF does); the
                // query engine resolves nulls later via `coerce_nulls_for_predicate`.
                let out = self.pattern.matches(arr).map_err(|e| {
                    datafusion::error::DataFusionError::Execution(format!("{FUNC_NAME}: {e}"))
                })?;
                Ok(ColumnarValue::Array(Arc::new(out)))
            }
            ColumnarValue::Scalar(s) => {
                Ok(ColumnarValue::Scalar(regex_match_scalar(&self.pattern, s)?))
            }
        }
    }
}

fn regex_match_scalar(pattern: &Pattern, s: &ScalarValue) -> Result<ScalarValue> {
    match s {
        ScalarValue::Utf8(v) | ScalarValue::LargeUtf8(v) | ScalarValue::Utf8View(v) => Ok(
            ScalarValue::Boolean(v.as_deref().map(|v| pattern.is_match(v))),
        ),
        ScalarValue::Dictionary(_, inner) => regex_match_scalar(pattern, inner),
        other => exec_err!(
            "{FUNC_NAME}: unsupported haystack type {}",
            other.data_type()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn re(p: &str) -> Pattern {
        Pattern::Regex(Regex::new(p).unwrap())
    }

    /// Scenario: match a pattern against scalar Utf8 values, including a null scalar.
    /// Guarantees: a non-null scalar yields its boolean match and a null scalar yields a null boolean.
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
