// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the public admin SDK.

use thiserror::Error;

/// Endpoint validation and URL construction errors.
#[derive(Debug, Error)]
pub enum EndpointError {
    /// The host is empty.
    #[error("admin endpoint host must not be empty")]
    EmptyHost,

    /// The scheme is invalid.
    #[error("invalid admin endpoint scheme: {scheme}")]
    InvalidScheme {
        /// Invalid scheme string.
        scheme: String,
    },

    /// The host is invalid.
    #[error("invalid admin endpoint host: {host}")]
    InvalidHost {
        /// Invalid host string.
        host: String,
    },

    /// The port is invalid.
    #[error("invalid admin endpoint port: {port}")]
    InvalidPort {
        /// Invalid port number.
        port: u16,
    },

    /// The base path is invalid.
    #[error("invalid admin endpoint base path '{base_path}': {reason}")]
    InvalidBasePath {
        /// Invalid base path string.
        base_path: String,
        /// Validation failure details.
        reason: String,
    },

    /// URL construction failed.
    #[error("failed to build admin endpoint URL: {details}")]
    UrlBuild {
        /// URL builder details.
        details: String,
    },

    /// URL parsing failed.
    #[error("invalid admin endpoint URL '{url}': {details}")]
    UrlParse {
        /// URL input that failed parsing.
        url: String,
        /// Parse failure details.
        details: String,
    },
}

/// Public admin SDK error.
#[derive(Debug, Error)]
pub enum Error {
    /// Endpoint configuration or URL building failed.
    #[error(transparent)]
    Endpoint(#[from] EndpointError),

    /// HTTP client construction or TLS configuration failed.
    #[cfg(feature = "http-client")]
    #[error("admin client configuration error: {details}")]
    ClientConfig {
        /// Client configuration failure details.
        details: String,
    },

    /// Transport failed before a complete response was received.
    #[cfg(feature = "http-client")]
    #[error("admin transport error: {details}")]
    Transport {
        /// Transport failure details.
        details: String,
    },

    /// Response body decoding failed.
    #[error("admin response decode error: {details}")]
    Decode {
        /// Decode failure details.
        details: String,
    },

    /// Remote endpoint returned an unexpected HTTP status.
    #[error("admin endpoint returned unexpected status {status} for {method} {url}")]
    RemoteStatus {
        /// HTTP method.
        method: String,
        /// Request URL.
        url: String,
        /// HTTP status code.
        status: u16,
        /// Response body, if available.
        body: String,
    },
}
