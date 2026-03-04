// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types surfaced by validation helpers, wrapping IO, template, config,
//! HTTP, readiness, and validation failures.

use std::fmt;

/// High-level error categories returned by validation helpers.
#[derive(Debug)]
pub enum ValidationError {
    /// Errors while reading files or talking to the filesystem.
    Io(String),
    /// Template rendering errors when producing pipeline YAML.
    Template(String),
    /// Malformed or incomplete configuration detected before execution.
    Config(String),
    /// HTTP request/response failures against the admin API.
    Http(String),
    /// Validation pipelines failed to report readiness within the budget.
    Ready(String),
    /// Validation checks did not succeed.
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
