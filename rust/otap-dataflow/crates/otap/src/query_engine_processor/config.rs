// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

/// Configuration for the [`QueryEngineProcessor`](super::QueryEngineProcessor)
#[derive(Debug, Deserialize)]
pub struct Config {
    /// the query that defines the transformation to be applied
    pub query: String,
}
