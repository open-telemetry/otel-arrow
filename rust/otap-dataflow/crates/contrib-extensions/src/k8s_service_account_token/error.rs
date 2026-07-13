// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Kubernetes Service Account Token extension.

use std::path::PathBuf;

/// Errors raised while reading the service account token from disk.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reading the token file failed (e.g. it does not exist or is unreadable).
    #[error("failed to read service account token file `{path}`: {source}")]
    ReadTokenFile {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// The token file exists but is empty (after trimming whitespace).
    #[error("service account token file `{path}` is empty")]
    EmptyToken {
        /// Path that contained no token.
        path: PathBuf,
    },
}
