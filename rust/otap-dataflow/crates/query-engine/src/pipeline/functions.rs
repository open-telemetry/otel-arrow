// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Contains function definitions such as datafusion UDFs

use std::ops::Range;

use datafusion::functions::{export_functions, make_udf_function};

// Note: this is imported like this because the make_udf_function macro uses
// `datafusion_expr` internally to reference this crate
use datafusion::logical_expr::{self as datafusion_expr, TypeSignature};
use datafusion::logical_expr_common::signature::Arity;

pub(crate) mod compare;
mod contains;
mod fnv;
pub(crate) mod is_type;
mod murmur3;
mod now;
mod regexp_substr;
#[cfg(feature = "sha1-hash")]
mod sha1;
mod substring;
mod uuidv7;
mod xxh128;
mod xxh3;

make_udf_function!(contains::ExtendedContainsFunc, contains);
make_udf_function!(fnv::FnvHashFunc, fnv_hash);
make_udf_function!(now::NowFunc, now);
make_udf_function!(murmur3::Murmur3HashFunc, murmur3_hash);
#[cfg(feature = "sha1-hash")]
make_udf_function!(sha1::Sha1Func, sha1_hash);
make_udf_function!(xxh128::Xxh128Func, xxh128_hash);
make_udf_function!(xxh3::Xxh3Func, xxh3_hash);
make_udf_function!(substring::SubstringFunc, substring);
make_udf_function!(regexp_substr::RegexpSubstrFunc, regexp_substr);
make_udf_function!(uuidv7::UuidV7Func, uuidv7);

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

#[cfg(test)]
pub(crate) mod test {
    use arrow::datatypes::DataType;
    use datafusion::error::Result;
    use datafusion::functions::make_udf_function;
    use datafusion::logical_expr::{
        self as datafusion_expr, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
    };

    make_udf_function!(AlwaysPanicUdf, always_panic);

    /// UDF that will always panic when evaluated.
    ///
    /// This can be used in tests that are asserting that some expression does not evaluate
    #[derive(Debug, Hash, Eq, PartialEq)]
    struct AlwaysPanicUdf {
        signature: Signature,
    }

    impl AlwaysPanicUdf {
        fn new() -> Self {
            Self {
                signature: Signature::any(1, Volatility::Volatile),
            }
        }
    }

    impl ScalarUDFImpl for AlwaysPanicUdf {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn name(&self) -> &str {
            "always_panic"
        }

        fn signature(&self) -> &Signature {
            &self.signature
        }

        fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
            Ok(DataType::Boolean)
        }

        fn invoke_with_args(
            &self,
            _args: ScalarFunctionArgs,
        ) -> Result<datafusion::logical_expr::ColumnarValue> {
            panic!("AlwaysPanicUdf not expected to evaluate")
        }
    }
}
