// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub(crate) const ATTRIBUTES_FIELD_NAME: &str = "attributes";
pub(crate) const BODY_FIELD_NAME: &str = "body";
pub(crate) const RESOURCES_FIELD_NAME: &str = "resource";
pub(crate) const SCOPE_FIELD_NAME: &str = "instrumentation_scope";
pub(crate) const VALUE_FIELD_NAME: &str = "value";

pub(crate) const ENCODE_FUNC_NAME: &str = "encode";
pub(crate) const FORMAT_DATETIME_FUNC_NAME: &str = "format_datetime";
pub(crate) const LOG_FUNC_NAME: &str = "log10";
pub(crate) const LTRIM_FUNC_NAME: &str = "ltrim";
pub(crate) const REGEXP_SUBSTR_FUNC_NAME: &str = "regexp_substr";
pub(crate) const RTRIM_FUNC_NAME: &str = "rtrim";
pub(crate) const SHA256_FUNC_NAME: &str = "sha256";
pub(crate) const MD5_FUNC_NAME: &str = "md5";
pub(crate) const FNV_FUNC_NAME: &str = "fnv";
pub(crate) const MURMUR3_FUNC_NAME: &str = "murmur3";
#[cfg(feature = "sha1-hash")]
pub(crate) const SHA1_FUNC_NAME: &str = "sha1";
pub(crate) const SHA512_FUNC_NAME: &str = "sha512";
pub(crate) const XXH3_FUNC_NAME: &str = "xxh3";
pub(crate) const XXH128_FUNC_NAME: &str = "xxh128";
pub(crate) const UUID_FUNC_NAME: &str = "uuid";
pub(crate) const UUIDV7_FUNC_NAME: &str = "uuidv7";
pub(crate) const LOWER_CASE_FUNC_NAME: &str = "lower_case";
pub(crate) const UPPER_CASE_FUNC_NAME: &str = "upper_case";
