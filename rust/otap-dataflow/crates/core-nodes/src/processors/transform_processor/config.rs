// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;

use serde::Deserialize;

/// Configuration for the [`TransformProcessor`](super::TransformProcessor)
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub query: Query,

    #[serde(default = "default_inbound_request_limit")]
    pub inbound_request_limit: NonZeroUsize,

    #[serde(default = "default_outbound_request_limit")]
    pub outbound_request_limit: NonZeroUsize,

    /// Flag for whether to skip "sanitizing" the results produced by the transformation.
    ///
    /// Default is false (perform sanitization), but can be set to true to improve performance if
    /// the purpose of the transformation isn't to remove sensitive data.
    ///
    /// The transformation will produce an OTAP batch that the semantically represents the desired
    /// result. Some data that was "removed" (through filtering, value reassignment, etc.) may
    /// still be present in the various arrow buffers. For example, the data may be in some
    /// values array inside a dictionary array with no keys pointing to it. The transformations are
    /// done this way for best performance. If the purpose of the transformation was to redact data
    /// for some security purpose, this sanitization pass should not be skipped.
    #[serde(default = "default_skip_sanitize_result")]
    pub skip_sanitize_result: bool,

    /// Whether to treat attribute key match as case sensitive during filtering stages.
    ///
    /// For example, in a query like the following:
    /// ```text
    /// logs | where attributes["x"] = "y"
    /// ```
    /// If this were set to `false`, logs with attributes `{"x": "y"}` as well as those
    /// with attributes `{"X": "y"}` would pass the filter`
    #[serde(default = "default_filter_attribute_keys_case_sensitive")]
    pub filter_attribute_keys_case_sensitive: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum Query {
    KqlQuery(String),
    OplQuery(String),
    Ottl(OttlConfig),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OttlConfig {
    /// OTTL Statements for transforming logs
    pub log_statements: Option<Vec<String>>,
    // TODO add trace/metrics statements
}

const fn default_inbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(1024).expect("ok") // default_min_size/8
}

const fn default_outbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(512).expect("ok") // default_min_size/16
}

const fn default_skip_sanitize_result() -> bool {
    false
}

const fn default_filter_attribute_keys_case_sensitive() -> bool {
    true
}
