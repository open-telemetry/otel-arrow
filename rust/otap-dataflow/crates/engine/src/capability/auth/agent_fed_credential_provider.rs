// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The `AgentFedCredentialProvider` capability.
//!
//! Supplies one immutable, atomically-published snapshot containing a bearer
//! token and its vendor-defined routing attributes. Consumers that require the
//! two values to stay generation-consistent must use this capability instead
//! of reading `bearer_token_provider` and `vendor_bundle` separately.

use super::BearerToken;
use crate::capability::error::CapabilityError;
use otel_arrow_dfe_engine_macros::capability;
use serde_json::{Map, Value};
use std::sync::Arc;

/// One generation-consistent host credential and attribute snapshot.
///
/// `Debug` redacts the token through [`BearerToken`], but renders `attributes`
/// verbatim, so hosts must keep secrets out of the vendor attribute map.
#[derive(Clone, Debug)]
pub struct AgentFedCredentialSnapshot {
    token: BearerToken,
    attributes: Arc<Map<String, Value>>,
}

impl AgentFedCredentialSnapshot {
    /// Creates a snapshot from values loaded during one atomic host-state read.
    #[must_use]
    pub fn new(token: BearerToken, attributes: Arc<Map<String, Value>>) -> Self {
        Self { token, attributes }
    }

    /// Returns the snapshot's bearer token.
    #[must_use]
    pub const fn token(&self) -> &BearerToken {
        &self.token
    }

    /// Returns the snapshot's vendor-defined attributes.
    #[must_use]
    pub fn attributes(&self) -> &Map<String, Value> {
        &self.attributes
    }
}

/// Provides atomically-paired agent-fed credentials and vendor attributes.
#[capability(
    name = "agent_fed_credential_provider",
    description = "Provides one atomic bearer-token and vendor-attribute snapshot"
)]
pub trait AgentFedCredentialProvider {
    /// Returns the current immutable credential snapshot.
    ///
    /// The provider must load one host snapshot and return both values from
    /// that same generation. It must not reconstruct this result by calling
    /// separate token and vendor capabilities.
    ///
    /// The returned future must be cancellation-safe because consumers may
    /// enforce a lookup deadline and drop it before completion. Cancellation
    /// must not leave shared state or locks unusable. Implementations should
    /// normally clone an already-published snapshot and avoid network I/O or
    /// other unbounded work in this method.
    async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: A snapshot is created with a secret token and routing attributes.
    /// Guarantees: Both values remain available from one immutable snapshot.
    #[test]
    fn snapshot_keeps_token_and_attributes_together() {
        let attributes = Arc::new(
            json!({"endpoint": "https://ingest.example"})
                .as_object()
                .cloned()
                .expect("object"),
        );
        let snapshot = AgentFedCredentialSnapshot::new(
            BearerToken::without_expiry("secret-token".to_owned()),
            Arc::clone(&attributes),
        );

        assert_eq!(snapshot.token().expose_token(), "secret-token");
        assert_eq!(snapshot.attributes(), attributes.as_ref());
        assert!(!format!("{snapshot:?}").contains("secret-token"));
    }
}
