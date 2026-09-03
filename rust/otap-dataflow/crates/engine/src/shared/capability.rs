// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared (Send + Sync) capability trait re-exports.
//!
//! Each production capability's shared trait variant is re-exported here
//! under a module path mirroring its `capability::<domain>::<name>` scoping
//! (e.g. `shared::capability::auth::bearer_token_provider::BearerTokenProvider`),
//! so every surface is scoped the same way. Capability traits are defined by
//! the `#[capability]` proc macro in per-capability modules under
//! [`capability`](crate::capability). Test-only reference capabilities live
//! under [`crate::testing::capability`] and are intentionally not re-exported
//! here.

/// Auth capabilities (shared `Send + Sync` trait variants).
pub mod auth {
    /// Shared (Send + Sync) trait variant of the agent-fed credential-provider capability.
    pub mod agent_fed_credential_provider {
        pub use crate::capability::auth::agent_fed_credential_provider::shared::AgentFedCredentialProvider;
    }
    /// Shared (Send + Sync) trait variant of the api-key-provider capability.
    pub mod api_key_provider {
        pub use crate::capability::auth::api_key_provider::shared::ApiKeyProvider;
    }
    /// Shared (Send + Sync) trait variant of the bearer-token-authorizer capability.
    pub mod bearer_token_authorizer {
        pub use crate::capability::auth::bearer_token_authorizer::shared::BearerTokenAuthorizer;
    }
    /// Shared (Send + Sync) trait variant of the bearer-token-provider capability.
    pub mod bearer_token_provider {
        pub use crate::capability::auth::bearer_token_provider::shared::BearerTokenProvider;
    }
}

/// Shared (Send + Sync) trait variant of the vendor-bundle capability.
pub mod vendor_bundle {
    pub use crate::capability::vendor_bundle::shared::VendorBundle;
}
