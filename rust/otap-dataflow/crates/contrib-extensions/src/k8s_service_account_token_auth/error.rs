// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Kubernetes service-account-token auth extension.

use std::sync::Arc;
use std::time::Duration;

/// Errors raised while constructing the Kubernetes client or performing review
/// requests.
#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    /// Constructing the in-cluster Kubernetes client failed (e.g. no projected
    /// service-account token, unreadable cluster CA, or unresolvable API
    /// server).
    #[error("failed to construct Kubernetes client: {source}")]
    ClientInit {
        /// Underlying kube error.
        source: Arc<kube::Error>,
    },

    /// The `TokenReview` request to the API server could not be completed (e.g.
    /// the API server is unreachable or returned a transport error). This is an
    /// undetermined outcome: callers must fail closed.
    #[error("TokenReview request failed: {source}")]
    TokenReview {
        /// Underlying kube error.
        source: Arc<kube::Error>,
    },

    /// The `TokenReview` did not complete within the configured timeout.
    #[error("TokenReview request timed out after {timeout:?}")]
    TokenReviewTimeout {
        /// Configured request timeout.
        timeout: Duration,
    },

    /// The `SubjectAccessReview` (RBAC) request to the API server could not be
    /// completed. This is an undetermined outcome: callers must fail closed.
    #[error("SubjectAccessReview request failed: {source}")]
    SubjectAccessReview {
        /// Underlying kube error.
        source: Arc<kube::Error>,
    },

    /// The `SubjectAccessReview` did not complete within the configured timeout.
    #[error("SubjectAccessReview request timed out after {timeout:?}")]
    SubjectAccessReviewTimeout {
        /// Configured request timeout.
        timeout: Duration,
    },

    /// The API server returned a review response with no `status`, so no
    /// decision could be derived from it.
    #[error("review response had no status")]
    MissingStatus,
}
