// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Contains function definitions such as datafusion UDFs

use datafusion::functions::{export_functions, make_udf_function};

// Note: this is imported like this because the make_udf_function macro uses
// `datafusion_expr` internally to reference this crate
use datafusion::logical_expr as datafusion_expr;

mod contains;

make_udf_function!(contains::ExtendedContainsFunc, contains);

/// helper functions to create logical plan expressions that invoke UDFs
pub mod expr_fn {
    use super::*;

    export_functions!((
        contains,
        "Return true if `search_string` is found within `string`.",
        string search_string
    ));
}
