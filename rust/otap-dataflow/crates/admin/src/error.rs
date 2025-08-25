// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the admin module.

use thiserror::Error;

/// Errors that can occur in the admin module.
#[derive(Error, Debug)]
pub enum Error {
    /// The configured bind address is invalid and could not be parsed.
    #[error("Invalid bind address '{bind_address}': {details}")]
    InvalidBindAddress {
        /// The bind address that failed to parse.
        bind_address: String,
        /// Human-readable details of the parsing failure.
        details: String,
    },

    /// Failed to bind the TCP listener on the given address.
    #[error("Failed to bind admin HTTP server on '{addr}': {details}")]
    BindFailed {
        /// The address we attempted to bind to.
        addr: String,
        /// Human-readable details of the bind failure.
        details: String,
    },

    /// The HTTP server encountered a fatal error while serving.
    #[error("Admin HTTP server error on '{addr}': {details}")]
    ServerError {
        /// The address the server was bound to.
        addr: String,
        /// Human-readable details of the server failure.
        details: String,
    },
}
