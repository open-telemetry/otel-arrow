// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Options for protocol conversion.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Options for protocol conversion.
///
/// Currently empty; intended as a placeholder for future per-conversion
/// configuration (e.g., size limits) to be threaded through pipeline config.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ConversionOptions {}
