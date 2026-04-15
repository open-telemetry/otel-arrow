// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub(crate) const ATTRIBUTES_FIELD_NAME: &str = "attributes";
pub(crate) const RESOURCES_FIELD_NAME: &str = "resource";
pub(crate) const SCOPE_FIELD_NAME: &str = "instrumentation_scope";
pub(crate) const VALUE_FIELD_NAME: &str = "value";

pub(crate) const ENCODE_FUNC_NAME: &str = "encode";
pub(crate) const REGEXP_SUBSTR_FUNC_NAME: &str = "regexp_substr";
pub(crate) const SHA256_FUNC_NAME: &str = "sha256";

/// Arrow field metadata key indicating a column is an AnyValue (type-discriminated union).
///
/// AnyValue columns are represented as struct columns containing a `type` discriminant (UInt8)
/// and multiple typed value sub-columns (`str`, `int`, `double`, `bool`, `bytes`, `ser`).
/// Only the sub-column corresponding to the `type` discriminant contains valid data for a given
/// row.
pub(crate) const ANY_VALUE_METADATA_KEY: &str = "otel.any_value";
