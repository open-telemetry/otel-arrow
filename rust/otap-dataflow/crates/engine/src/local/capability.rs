// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Local (!Send) capability trait re-exports.
//!
//! Each production capability's local trait variant is re-exported here under a
//! module named after the capability (e.g.
//! `local::capability::bearer_token_provider::BearerTokenProvider`), mirroring
//! the `capability::<name>` scoping so every surface is scoped the same way.
//! Capability traits are defined by the `#[capability]` proc macro in
//! per-capability modules under [`capability`](crate::capability). Test-only
//! reference capabilities live under [`crate::testing::capability`] and are
//! intentionally not re-exported here.

/// Local (!Send) trait variant of the bearer-token-authorizer capability.
pub mod bearer_token_authorizer {
    pub use crate::capability::bearer_token_authorizer::local::BearerTokenAuthorizer;
}
/// Local (!Send) trait variant of the bearer-token-provider capability.
pub mod bearer_token_provider {
    pub use crate::capability::bearer_token_provider::local::BearerTokenProvider;
}
