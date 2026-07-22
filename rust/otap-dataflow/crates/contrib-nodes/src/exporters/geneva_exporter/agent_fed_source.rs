// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Agent-fed credential adapter for the Geneva exporter.
//!
//! Bridges the agent-fed `bearer_token_provider` + `vendor_bundle` capabilities
//! to geneva-uploader's [`AgentFedCredentialSource`], so the uploader uses the
//! host-provisioned token and routing instead of the GCS handshake.

use geneva_uploader::client::{
    AgentFedCredential, AgentFedCredentialFuture, AgentFedCredentialSource,
};
use otap_df_engine::shared::capability::bearer_token_provider::BearerTokenProvider;
use otap_df_engine::shared::capability::vendor_bundle::VendorBundle;
use serde_json::Value;
use tokio::sync::Mutex;

/// Adapts the agent-fed bearer-token + vendor-bundle capabilities to
/// geneva-uploader's agent-fed credential source.
///
/// The resolved `shared` trait objects are `Send` but not `Sync`; a
/// [`tokio::sync::Mutex`] restores `Sync` and lets reads await under a `Send`
/// guard (satisfying `AgentFedCredentialSource: Send + Sync`).
pub(crate) struct AgentFedGenevaSource {
    bearer: Mutex<Box<dyn BearerTokenProvider>>,
    vendor: Mutex<Box<dyn VendorBundle>>,
}

impl std::fmt::Debug for AgentFedGenevaSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AgentFedGenevaSource")
    }
}

impl AgentFedGenevaSource {
    /// Builds the adapter from the resolved shared capabilities.
    pub(crate) fn new(bearer: Box<dyn BearerTokenProvider>, vendor: Box<dyn VendorBundle>) -> Self {
        Self {
            bearer: Mutex::new(bearer),
            vendor: Mutex::new(vendor),
        }
    }
}

impl AgentFedCredentialSource for AgentFedGenevaSource {
    fn current(&self) -> AgentFedCredentialFuture<'_> {
        Box::pin(async move {
            // Await the token so a cache-miss credential call completes (not dropped).
            let token = match self.bearer.lock().await.get_token().await {
                Ok(t) => t.expose_token().to_owned(),
                Err(_) => return None,
            };
            if token.is_empty() {
                return None;
            }

            let attributes = match self.vendor.lock().await.attributes() {
                Ok(a) => a,
                Err(_) => return None,
            };
            let endpoint = attributes
                .get("endpoint")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            // Prefer "default"; else the lowest key (deterministic).
            let moniker = attributes
                .get("moniker_map")
                .and_then(Value::as_object)
                .and_then(|m| {
                    m.get("default")
                        .or_else(|| m.keys().min().and_then(|k| m.get(k)))
                })
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();

            // Fail closed on partial routing; the host reseeds when ready.
            if endpoint.is_empty() || moniker.is_empty() {
                return None;
            }

            Some(AgentFedCredential {
                token,
                endpoint,
                moniker,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use futures::StreamExt;
    use otap_df_engine::capability::CapabilityError;
    use otap_df_engine::capability::bearer_token_provider::{BearerToken, TokenStream};
    use serde_json::{Map, Value, json};
    use std::sync::Arc;

    struct MockBearer {
        token: String,
        yield_first: bool,
    }

    #[async_trait]
    impl BearerTokenProvider for MockBearer {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            if self.yield_first {
                // Force the future to be pending once; a `now_or_never` poll
                // would drop the result here.
                tokio::task::yield_now().await;
            }
            Ok(BearerToken::new(self.token.clone(), None))
        }
        fn token_stream(&self) -> TokenStream {
            futures::stream::empty::<BearerToken>().boxed()
        }
    }

    struct MockVendor(Arc<Map<String, Value>>);

    impl VendorBundle for MockVendor {
        fn attributes(&self) -> Result<Arc<Map<String, Value>>, CapabilityError> {
            Ok(self.0.clone())
        }
    }

    fn obj(v: Value) -> Arc<Map<String, Value>> {
        Arc::new(v.as_object().cloned().unwrap_or_default())
    }

    fn source(token: &str, yield_first: bool, attrs: Value) -> AgentFedGenevaSource {
        AgentFedGenevaSource::new(
            Box::new(MockBearer {
                token: token.to_owned(),
                yield_first,
            }),
            Box::new(MockVendor(obj(attrs))),
        )
    }

    fn full_attrs() -> Value {
        json!({
            "endpoint": "https://ep",
            "moniker_map": { "default": "mon" },
        })
    }

    #[tokio::test]
    async fn returns_credential_when_token_and_routing_present() {
        let s = source("tok", false, full_attrs());
        let c = s.current().await.expect("credential");
        assert_eq!(c.token, "tok");
        assert_eq!(c.endpoint, "https://ep");
        assert_eq!(c.moniker, "mon");
    }

    #[tokio::test]
    async fn awaits_pending_provider_future() {
        // Regression for the old `now_or_never()`: a provider whose future is
        // not immediately ready must still yield a credential.
        let s = source("tok", true, full_attrs());
        assert!(s.current().await.is_some());
    }

    #[tokio::test]
    async fn fails_closed_on_empty_token() {
        let s = source("", false, full_attrs());
        assert!(s.current().await.is_none());
    }

    #[tokio::test]
    async fn fails_closed_on_missing_endpoint() {
        let attrs = json!({ "moniker_map": { "default": "mon" } });
        let s = source("tok", false, attrs);
        assert!(s.current().await.is_none());
    }

    #[tokio::test]
    async fn fails_closed_on_missing_moniker() {
        let attrs = json!({ "endpoint": "https://ep" });
        let s = source("tok", false, attrs);
        assert!(s.current().await.is_none());
    }

    #[tokio::test]
    async fn moniker_prefers_default_then_lowest_key() {
        // No "default": the lowest key wins, deterministically.
        let attrs = json!({
            "endpoint": "https://ep",
            "moniker_map": { "b": "mb", "a": "ma" },
        });
        let s = source("tok", false, attrs);
        assert_eq!(s.current().await.unwrap().moniker, "ma");
    }
}
