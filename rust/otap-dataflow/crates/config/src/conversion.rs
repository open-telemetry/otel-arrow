// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Options for encoded output.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

/// Output-specific options applied while encoding telemetry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct EncodeOptions {
    /// Maximum size of an encoded OTLP protobuf message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub otlp_size_limit: Option<NonZeroUsize>,
}

/// Transitional alias for code that still passes the former conversion options.
///
/// Decoding has no configurable policy. New code should use [`EncodeOptions`]
/// only at an output boundary.
pub type ConversionOptions = EncodeOptions;
