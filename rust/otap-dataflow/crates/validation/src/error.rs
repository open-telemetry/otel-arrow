// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for pipeline validation helpers.

use std::fmt;

/// Error
#[derive(Debug)]
pub enum ValidationError {
    /// Error with reading in configs
    Io(String),
    /// Error with rendering the template
    Template(String),
    /// Error with config when trying to run pipeline
    Config(String),
    /// Error with admin endpoints
    Http(String),
    /// Error if pipeline is not ready
    Ready(String),
    /// Error if validation failed to pass
    Validation(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::Io(e) => write!(f, "io error: {e}"),
            ValidationError::Template(e) => write!(f, "template error: {e}"),
            ValidationError::Config(e) => write!(f, "config error: {e}"),
            ValidationError::Http(e) => write!(f, "http error: {e}"),
            ValidationError::Ready(e) => write!(f, "ready check failed: {e}"),
            ValidationError::Validation(e) => write!(f, "validation failed: {e}"),
        }
    }
}
