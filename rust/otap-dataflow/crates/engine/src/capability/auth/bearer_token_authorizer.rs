// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The `BearerTokenAuthorizer` capability.
//!
//! The request-facing access-control capability for data-path nodes (typically
//! receivers) that authenticate callers with an OAuth/OIDC-style **bearer
//! token**. Given the token presented on an inbound request, it returns a
//! single allow/deny [`AuthzDecision`].
//!
//! A `BearerTokenAuthorizer` performs **authentication** (establishing who the
//! caller is from the token) and **admission** (deciding whether that token is
//! acceptable — e.g. against a configured allow-list), behind one call, so a
//! receiver depends on this single capability rather than orchestrating the
//! steps itself. For example, a Kubernetes service-account-token authorizer
//! validates the token via the `TokenReview` API (authentication) and then
//! checks the returned service account against a configured allow-list
//! (admission) — deriving both the trust source and the allowed identities from
//! its own configuration, so the caller supplies only the token.
//!
//! It admits on the token alone; it does not perform contextual, per-request
//! authorization (route, tenant, signal, or action scoping), which needs
//! request context it never receives and belongs downstream — consuming the
//! [`AuthorizedIdentity`](super::AuthorizedIdentity) this capability emits.
//!
//! This capability is bearer-specific by design (the credential is always a
//! token string) and **transport- and library-agnostic**: the token is carried
//! by [`BearerToken`](super::BearerToken), a secret-protecting wrapper built
//! from plain `&str` (a bare token or a whole `Authorization` header value),
//! never from any HTTP/RPC crate's request type. A receiver extracts it from
//! whatever transport it uses (gRPC, HTTP, …).
//!
//! The `#[capability]` proc macro expands the trait into:
//!
//! - `pub(crate) mod local::BearerTokenAuthorizer` (`!Send` trait variant)
//! - `pub(crate) mod shared::BearerTokenAuthorizer` (`Send` trait variant)
//! - A `SharedAsLocalBearerTokenAuthorizer` adapter
//! - A zero-sized `pub struct BearerTokenAuthorizer` registration handle
//! - `local_entry::<E>` / `shared_entry::<E>` factory bridges
//! - A `KNOWN_CAPABILITIES` distributed-slice entry

use super::{AuthzDecision, BearerToken};
use crate::capability::error::CapabilityError;
use otap_df_engine_macros::capability;

/// Authenticates and admits an inbound bearer token against a configured
/// policy.
#[capability(
    name = "bearer_token_authorizer",
    description = "Authenticates and admits an inbound bearer token against a configured policy"
)]
pub trait BearerTokenAuthorizer {
    /// Decides whether the caller presenting `credential` is permitted.
    ///
    /// `credential` wraps the bearer token exactly as presented (see
    /// [`BearerToken`] for constructing one from a bare token or a whole
    /// `Authorization` header value).
    ///
    /// The authorizer **authenticates** the token (verifying its
    /// signature/authenticity, expiry, issuer, audience, or via an external
    /// check such as a Kubernetes `TokenReview`) and then **admits** the
    /// resulting identity against its own policy (e.g. an allow-list) — both
    /// steps behind this one call, so a receiver need not authenticate
    /// separately. The trust source and the allowed identities come from the
    /// authorizer's own configuration, so the caller supplies only the
    /// credential. Admission is on the token alone; contextual per-request
    /// authorization (route, tenant, signal, action) is a downstream concern.
    ///
    /// Decision caching and freshness are the implementation's concern: an
    /// authorizer that wants to avoid a backend round-trip per request (e.g. a
    /// `TokenReview` call) should cache internally, keyed by the opaque token
    /// and bounded by its own TTL. [`AuthzDecision::Allow`] deliberately carries
    /// no expiry or validity window, so callers stay simple and never manage
    /// freshness.
    ///
    /// Returns `Ok(`[`AuthzDecision::Allow`]`)` or `Ok(`[`AuthzDecision::Deny`]`)`
    /// when the authorizer reaches a verdict — a deny (including an
    /// authentication failure such as a missing, expired, or untrusted token)
    /// is a normal outcome, not an error. Returns a [`CapabilityError`] only
    /// when the authorizer **cannot reach a decision** (e.g. its token-review or
    /// policy backend is unreachable). Callers must **fail closed** — treat an
    /// `Err` as a deny — since an undetermined decision must never grant access.
    async fn authorize(&self, credential: &BearerToken) -> Result<AuthzDecision, CapabilityError>;
}
