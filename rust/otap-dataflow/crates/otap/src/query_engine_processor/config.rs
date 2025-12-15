// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

/// Configuration for the [`QueryEngineProcessor`](super::QueryEngineProcessor)
#[derive(Debug, Deserialize)]
pub struct Config {
    /// the program that defines the transformation to be applied
    pub program: String,
}
