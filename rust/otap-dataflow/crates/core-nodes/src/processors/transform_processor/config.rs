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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Query {
    KqlQuery(String),
    OplQuery(String),
    // TODO - add section to allow transforms to be specified in OTTL
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
