// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Errors exposed by the Microsoft JSON receiver framework.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// The receiver runtime will be added in a follow-up change.
    #[error("Microsoft JSON receiver is not implemented")]
    NotImplemented,
}
