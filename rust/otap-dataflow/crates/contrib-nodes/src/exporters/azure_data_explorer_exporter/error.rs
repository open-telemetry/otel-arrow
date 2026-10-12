// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Errors currently exposed by the Azure Data Explorer exporter framework.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// The exporter runtime will be added in a follow-up change.
    #[error("Azure Data Explorer exporter is not implemented")]
    NotImplemented,
}
