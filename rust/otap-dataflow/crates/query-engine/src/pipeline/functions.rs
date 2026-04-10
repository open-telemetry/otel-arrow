// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Contains function definitions such as datafusion UDFs

use std::ops::Range;

use datafusion::functions::{export_functions, make_udf_function};

// Note: this is imported like this because the make_udf_function macro uses
// `datafusion_expr` internally to reference this crate
use datafusion::logical_expr::{self as datafusion_expr, TypeSignature};
use datafusion::logical_expr_common::signature::Arity;

mod contains;
mod regexp_substr;
mod substring;

make_udf_function!(contains::ExtendedContainsFunc, contains);
make_udf_function!(substring::SubstringFunc, substring);
make_udf_function!(regexp_substr::RegexpSubstrFunc, regexp_substr);

/// helper functions to create logical plan expressions that invoke UDFs
pub mod expr_fn {
    use super::*;

    export_functions!((
        contains,
        "Return true if `search_string` is found within `string`.",
        string search_string
    ));
}

/// Get the range of number of args the function signature will accept.
///
/// This is useful in cases where the function has [`TypeSignature::OneOf`] with many variants
/// and we want to check during planning that at least one of the internals has the correct
/// number of args for some signature. In these cases, we don't rely in [`Signature::arity`]
/// because it returns the max arity.
pub(crate) fn arity_range(signature: &TypeSignature) -> Option<Range<usize>> {
    match signature {
        TypeSignature::OneOf(variants) => {
            let mut min = usize::MAX;
            let mut max = 0;
            for variant in variants {
                match variant.arity() {
                    Arity::Fixed(n) => {
                        if n < min {
                            min = n;
                        }
                        if n > max {
                            max = n
                        }
                    }
                    Arity::Variable => {
                        // func can take any number of args
                        return None;
                    }
                }
            }

            Some(Range {
                start: min,
                end: max + 1,
            })
        }
        _ => {
            if let Arity::Fixed(n) = signature.arity() {
                Some(Range {
                    start: n,
                    end: n + 1,
                })
            } else {
                None
            }
        }
    }
}
