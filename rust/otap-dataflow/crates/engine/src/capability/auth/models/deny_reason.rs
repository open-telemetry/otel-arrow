// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The [`DenyReason`] categories an authorizer returns on a deny.

/// Why a request was denied.
///
/// Coarse and scheme-agnostic so it is safe to use as a low-cardinality metric
/// label. Scheme-specific detail (`invalid_aud`, `expired`, a service-account
/// name, an IP, …) belongs in the decision's `detail` field (see
/// [`AuthzDecision`](super::AuthzDecision)) for logs, or in an authorizer's own
/// metrics — never inflate this enum with per-request values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DenyReason {
    /// No credential was presented (maps to HTTP 401 / gRPC `UNAUTHENTICATED`).
    MissingCredential,
    /// A credential was presented but authentication failed — malformed,
    /// expired, bad signature, or untrusted issuer/audience (401 /
    /// `UNAUTHENTICATED`).
    InvalidCredential,
    /// Authentication succeeded, but the identity is not allowed by the
    /// authorizer's own admission policy / allow-list (HTTP 403 / gRPC
    /// `PERMISSION_DENIED`). This is admission, not contextual per-request
    /// authorization — that needs request context and belongs downstream.
    NotPermitted,
}
