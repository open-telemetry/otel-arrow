// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

/// Configuration for the [`TransformProcessor`](super::TransformProcessor)
#[derive(Debug, Deserialize)]
pub struct Config {
    /// the query that defines the transformation to be applied
    pub query: String,
    // TODO - add section to allow transforms to be specified in OTTL
}
