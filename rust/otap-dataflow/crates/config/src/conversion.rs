// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Options for protocol conversion.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

/// Options for protocol conversion.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ConversionOptions {
    /// default applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub otlp_size_limit: Option<NonZeroUsize>,
}

/// Default is 16KiB.
pub const DEFAULT_OTLP_SIZE_LIMIT: usize = 1 << 14;

impl ConversionOptions {
    /// Placeholder for callers that need to be plumbed with real options.
    ///
    /// TODO(#1746): thread ConversionOptions through from pipeline config.
    #[must_use]
    pub fn options_todo() -> Self {
        Self::default()
    }

    /// Returns the OTLP message size limit.
    #[must_use]
    pub const fn otlp_size_limit(&self) -> usize {
        if let Some(limit) = self.otlp_size_limit {
            limit.get()
        } else {
            DEFAULT_OTLP_SIZE_LIMIT
        }
    }
}
