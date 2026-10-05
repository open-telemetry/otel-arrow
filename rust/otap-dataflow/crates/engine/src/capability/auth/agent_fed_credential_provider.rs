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
use futures::Stream;
use otel_arrow_dfe_engine_macros::capability;
use serde_json::{Map, Value};
use std::pin::Pin;
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

/// A per-consumer subscription to credential snapshot refreshes.
///
/// The item is a plain [`AgentFedCredentialSnapshot`], not a `Result`: a refresh failure does not
/// terminate the subscription. The stream simply does not emit until the next
/// successful refresh, and failures surface via [`AgentFedCredentialProvider::get_credential`]
/// and telemetry instead. Because the item is [`Clone`], a provider can fan one
/// refreshed credential snapshot out to all subscribers via a `watch`/`broadcast` channel.
///
/// Boxed to hide the concrete stream type so providers can back it differently
/// (e.g. a `watch` channel or an `unfold`) without changing the signature. The
/// `Send` bound is intentionally omitted: the subscription is always consumed
/// on the core that created it (thread-per-core), so it need not be `Send`. The
/// `#[capability]` macro emits this signature into both the `local` (`?Send`)
/// and `shared` (`Send + Sync`) trait variants unchanged.
pub type AgentFedCredentialSnapshotStream =
    Pin<Box<dyn Stream<Item = AgentFedCredentialSnapshot> + 'static>>;

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
    /// Providers must clone and return the same published `Arc` while the host
    /// snapshot is unchanged. Consumers use `Arc::ptr_eq` as the snapshot
    /// generation identity so rejected credentials are not retried until the
    /// host publishes a replacement.
    ///
    /// The returned future must be cancellation-safe because consumers may
    /// enforce a lookup deadline and drop it before completion. Cancellation
    /// must not leave shared state or locks unusable. Implementations should
    /// avoid network I/O or other unbounded work in this method.
    async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError>;

    /// Subscribes to the stream of credential snapshot refreshes.
    ///
    /// Yields each newly published credential snapshot for the lifetime of the extension;
    /// each call returns an independent subscription. The stream does not carry
    /// errors: a failed refresh does not end the subscription, and the next
    /// successful refresh still yields an credential snapshot (see [`AgentFedCredentialSnapshotStream`]).
    ///
    /// # Contract
    ///
    /// A subscription created *after* an credential snapshot has already been published
    /// MUST immediately yield the current credential snapshot rather than block until the
    /// next refresh. This lets a consumer subscribe at any point (for example
    /// after the provider's readiness gate has fired) and obtain a usable API
    /// Key without a separate [`get_credential`](Self::get_credential) call, avoiding
    /// a race between reading the current token and subscribing to updates. A
    /// `tokio::sync::watch`-backed implementation satisfies this naturally,
    /// since a fresh receiver observes the channel's current value on its first
    /// poll.
    fn credential_stream(&self) -> AgentFedCredentialSnapshotStream;
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
