// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Error definitions for azure monitor exporter.
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    /// Error during configuration of a component.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Error due to duplicate columns in schema mapping.
    #[error("Configuration error: duplicate columns found: {columns:?}")]
    ConfigurationDuplicateColumnsError {
        /// The duplicate columns found.
        columns: Vec<String>,
    },
}
