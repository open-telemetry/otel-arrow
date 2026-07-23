// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Kubernetes SAT authorizer extension.

/// Errors raised while constructing the Kubernetes client or performing a
/// `TokenReview`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Constructing the in-cluster Kubernetes client failed (e.g. no projected
    /// service-account token, unreadable cluster CA, or unresolvable API
    /// server).
    #[error("failed to construct Kubernetes client: {source}")]
    ClientInit {
        /// Underlying kube error.
        source: kube::Error,
    },

    /// The `TokenReview` request to the API server could not be completed (e.g.
    /// the API server is unreachable or returned a transport error). This is an
    /// undetermined outcome: callers must fail closed.
    #[error("TokenReview request failed: {source}")]
    TokenReview {
        /// Underlying kube error.
        source: kube::Error,
    },

    /// The API server returned a `TokenReview` response with no `status`, so no
    /// decision could be derived from it.
    #[error("TokenReview response had no status")]
    MissingStatus,
}
