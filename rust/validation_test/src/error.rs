// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for pipeline validation helpers.

use std::fmt;

#[derive(Debug)]
pub enum PipelineError {
    Io(String),
    Template(String),
    Config(String),
    Http(String),
    Ready(String),
    Validation(String),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::Io(e) => write!(f, "io error: {e}"),
            PipelineError::Template(e) => write!(f, "template error: {e}"),
            PipelineError::Config(e) => write!(f, "config error: {e}"),
            PipelineError::Http(e) => write!(f, "http error: {e}"),
            PipelineError::Ready(e) => write!(f, "ready check failed: {e}"),
            PipelineError::Validation(e) => write!(f, "validation failed: {e}"),
        }
    }
}
