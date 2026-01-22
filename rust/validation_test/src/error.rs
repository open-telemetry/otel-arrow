// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for pipeline validation helpers.

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum PipelineError {
    Io(io::Error),
    Template(tera::Error),
    Config(String),
    Http(reqwest::Error),
    Status(String),
    Ready(String),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::Io(e) => write!(f, "io error: {}", e),
            PipelineError::Template(e) => write!(f, "template error: {}", e),
            PipelineError::Config(e) => write!(f, "config error: {}", e),
            PipelineError::Http(e) => write!(f, "http error: {}", e),
            PipelineError::Status(e) => write!(f, "status error: {}", e),
            PipelineError::Ready(e) => write!(f, "ready check failed: {}", e),
        }
    }
}

impl std::error::Error for PipelineError {}

impl From<io::Error> for PipelineError {
    fn from(err: io::Error) -> Self {
        PipelineError::Io(err)
    }
}

impl From<tera::Error> for PipelineError {
    fn from(err: tera::Error) -> Self {
        PipelineError::Template(err)
    }
}

impl From<reqwest::Error> for PipelineError {
    fn from(err: reqwest::Error) -> Self {
        PipelineError::Http(err)
    }
}
