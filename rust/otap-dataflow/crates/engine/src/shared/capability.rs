// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared (Send) capability trait re-exports.
//!
//! Each production capability's shared trait variant is re-exported here
//! under a module named after the capability (e.g.
//! `shared::capability::bearer_token_provider::BearerTokenProvider`), mirroring
//! the `capability::<name>` scoping so every surface is scoped the same way.
//! Capability traits are defined by the `#[capability]` proc macro in
//! per-capability modules under [`capability`](crate::capability). Test-only
//! reference capabilities live under [`crate::testing::capability`] and are
//! intentionally not re-exported here.

/// Shared (Send) trait variant of the bearer-token-provider capability.
pub mod bearer_token_provider {
    pub use crate::capability::bearer_token_provider::shared::BearerTokenProvider;
}

/// Shared (Send) trait variant of the vendor-bundle capability.
pub mod vendor_bundle {
    pub use crate::capability::vendor_bundle::shared::VendorBundle;
}
