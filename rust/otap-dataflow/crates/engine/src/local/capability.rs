// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Local (!Send) capability trait re-exports.
//!
//! Each production capability's local trait variant is re-exported here under a
//! module path mirroring its `capability::<domain>::<name>` scoping (e.g.
//! `local::capability::auth::bearer_token_provider::BearerTokenProvider`), so
//! every surface is scoped the same way. Capability traits are defined by the
//! `#[capability]` proc macro in per-capability modules under
//! [`capability`](crate::capability). Test-only reference capabilities live
//! under [`crate::testing::capability`] and are intentionally not re-exported
//! here.

/// Auth capabilities (local `!Send` trait variants).
pub mod auth {
    /// Local (!Send) trait variant of the bearer-token-authorizer capability.
    pub mod bearer_token_authorizer {
        pub use crate::capability::auth::bearer_token_authorizer::local::BearerTokenAuthorizer;
    }
    /// Local (!Send) trait variant of the bearer-token-provider capability.
    pub mod bearer_token_provider {
        pub use crate::capability::auth::bearer_token_provider::local::BearerTokenProvider;
    }
}

/// Local (!Send) trait variant of the vendor-bundle capability.
pub mod vendor_bundle {
    pub use crate::capability::vendor_bundle::local::VendorBundle;
}
