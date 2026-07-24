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
use otap_df_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider;
use otap_df_engine::shared::capability::vendor_bundle::VendorBundle;
use otap_df_telemetry::otel_warn;
use serde_json::{Map, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;

const ENDPOINT_KEY: &str = "endpoint";
const MONIKER_MAP_KEY: &str = "moniker_map";
const DEFAULT_MONIKER_KEY: &str = "default";

#[derive(Default)]
struct FailureLogLimiter {
    consecutive_failures: AtomicU64,
}

impl FailureLogLimiter {
    fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    fn record_failure(&self) -> Option<u64> {
        let count = self
            .consecutive_failures
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        // Log failures 1, 2, 4, 8, ... and reset after recovery.
        count.is_power_of_two().then_some(count)
    }
}

/// Adapts the agent-fed bearer-token + vendor-bundle capabilities to
/// geneva-uploader's agent-fed credential source.
///
/// The resolved `shared` trait objects are `Send` but not `Sync`; a
/// [`tokio::sync::Mutex`] restores `Sync` and lets reads await under a `Send`
/// guard (satisfying `AgentFedCredentialSource: Send + Sync`).
pub(crate) struct AgentFedGenevaSource {
    bearer: Mutex<Box<dyn BearerTokenProvider>>,
    vendor: Mutex<Box<dyn VendorBundle>>,
    account: String,
    bearer_failures: FailureLogLimiter,
    vendor_failures: FailureLogLimiter,
    empty_token_failures: FailureLogLimiter,
    routing_failures: FailureLogLimiter,
}

impl std::fmt::Debug for AgentFedGenevaSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AgentFedGenevaSource")
    }
}

impl AgentFedGenevaSource {
    /// Builds the adapter from the resolved shared capabilities.
    pub(crate) fn new(
        bearer: Box<dyn BearerTokenProvider>,
        vendor: Box<dyn VendorBundle>,
        account: String,
    ) -> Self {
        Self {
            bearer: Mutex::new(bearer),
            vendor: Mutex::new(vendor),
            account,
            bearer_failures: FailureLogLimiter::default(),
            vendor_failures: FailureLogLimiter::default(),
            empty_token_failures: FailureLogLimiter::default(),
            routing_failures: FailureLogLimiter::default(),
        }
    }

    fn log_invalid_credential(limiter: &FailureLogLimiter, reason: &'static str) {
        if let Some(consecutive_failures) = limiter.record_failure() {
            otel_warn!(
                "geneva_exporter.agent_fed.invalid_credential",
                reason = reason,
                consecutive_failures = consecutive_failures
            );
        }
    }
}

impl AgentFedCredentialSource for AgentFedGenevaSource {
    fn current(&self) -> AgentFedCredentialFuture<'_> {
        Box::pin(async move {
            // Await the token so a cache-miss credential call completes (not dropped).
            let token = match self.bearer.lock().await.get_token().await {
                Ok(token) => {
                    self.bearer_failures.record_success();
                    // The uploader credential owns a String, so this is the
                    // required plaintext copy at the adapter boundary. Its
                    // Debug implementation redacts the token.
                    token.expose_token().to_owned()
                }
                Err(error) => {
                    if let Some(consecutive_failures) = self.bearer_failures.record_failure() {
                        otel_warn!(
                            "geneva_exporter.agent_fed.bearer_token_unavailable",
                            error = %error,
                            consecutive_failures = consecutive_failures
                        );
                    }
                    return None;
                }
            };
            if token.trim().is_empty() {
                Self::log_invalid_credential(
                    &self.empty_token_failures,
                    "bearer token is empty or whitespace",
                );
                return None;
            }
            self.empty_token_failures.record_success();

            let attributes = match self.vendor.lock().await.attributes() {
                Ok(attributes) => {
                    self.vendor_failures.record_success();
                    attributes
                }
                Err(error) => {
                    if let Some(consecutive_failures) = self.vendor_failures.record_failure() {
                        otel_warn!(
                            "geneva_exporter.agent_fed.vendor_bundle_unavailable",
                            error = %error,
                            consecutive_failures = consecutive_failures
                        );
                    }
                    return None;
                }
            };
            let (endpoint, moniker) = match resolve_routing(&attributes, &self.account) {
                Ok(routing) => {
                    self.routing_failures.record_success();
                    routing
                }
                Err(reason) => {
                    Self::log_invalid_credential(&self.routing_failures, reason);
                    return None;
                }
            };

            Some(AgentFedCredential {
                token,
                endpoint: endpoint.to_owned(),
                moniker: moniker.to_owned(),
            })
        })
    }
}

fn non_blank_string(value: Option<&Value>) -> Option<&str> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn resolve_routing<'a>(
    attributes: &'a Map<String, Value>,
    account: &str,
) -> Result<(&'a str, &'a str), &'static str> {
    let endpoint = non_blank_string(attributes.get(ENDPOINT_KEY))
        .ok_or("vendor bundle endpoint is missing or empty")?;
    let moniker_map = attributes
        .get(MONIKER_MAP_KEY)
        .and_then(Value::as_object)
        .ok_or("vendor bundle moniker_map is missing or invalid")?;

    let account = account.trim();
    let moniker = if let Some(value) = moniker_map.get(account) {
        non_blank_string(Some(value))
            .ok_or("vendor bundle moniker for the configured account is invalid or empty")?
    } else if let Some(value) = moniker_map.get(DEFAULT_MONIKER_KEY) {
        non_blank_string(Some(value)).ok_or("vendor bundle default moniker is invalid or empty")?
    } else if moniker_map.len() == 1 {
        non_blank_string(moniker_map.values().next())
            .ok_or("vendor bundle sole moniker is invalid or empty")?
    } else if moniker_map.is_empty() {
        return Err("vendor bundle moniker_map is empty");
    } else {
        return Err("vendor bundle moniker_map is ambiguous for the configured account");
    };

    Ok((endpoint, moniker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use futures::StreamExt;
    use otap_df_engine::capability::auth::BearerToken;
    use otap_df_engine::capability::auth::bearer_token_provider::{
        BearerTokenProvider as BearerTokenProviderCap, TokenStream,
    };
    use otap_df_engine::capability::vendor_bundle::VendorBundle as VendorBundleCap;
    use otap_df_engine::capability::{CapabilityError, CapabilityErrorSource};
    use serde_json::{Map, Value, json};
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

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
            Ok(BearerToken::without_expiry(self.token.clone()))
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
        source_for_account(token, yield_first, attrs, "account")
    }

    fn source_for_account(
        token: &str,
        yield_first: bool,
        attrs: Value,
        account: &str,
    ) -> AgentFedGenevaSource {
        AgentFedGenevaSource::new(
            Box::new(MockBearer {
                token: token.to_owned(),
                yield_first,
            }),
            Box::new(MockVendor(obj(attrs))),
            account.to_owned(),
        )
    }

    fn full_attrs() -> Value {
        json!({
            "endpoint": "https://ep",
            "moniker_map": { "default": "mon" },
        })
    }

    /// Scenario: The provider returns a token and the bundle contains valid routing.
    /// Guarantees: The adapter returns the exact token, endpoint, and moniker.
    #[tokio::test]
    async fn returns_credential_when_token_and_routing_present() {
        let s = source("tok", false, full_attrs());
        let c = s.current().await.expect("credential");
        assert_eq!(c.token, "tok");
        assert_eq!(c.endpoint, "https://ep");
        assert_eq!(c.moniker, "mon");
    }

    /// Scenario: The token provider future yields once before returning a token.
    /// Guarantees: The adapter awaits the future instead of dropping a pending result.
    #[tokio::test]
    async fn awaits_pending_provider_future() {
        // Regression for the old `now_or_never()`: a provider whose future is
        // not immediately ready must still yield a credential.
        let s = source("tok", true, full_attrs());
        assert!(s.current().await.is_some());
    }

    /// Scenario: The bearer token is empty.
    /// Guarantees: The adapter fails closed and does not emit a credential.
    #[tokio::test]
    async fn fails_closed_on_empty_token() {
        let s = source("", false, full_attrs());
        assert!(s.current().await.is_none());
    }

    /// Scenario: The bearer token contains only whitespace.
    /// Guarantees: Whitespace cannot be sent as an apparently valid credential.
    #[tokio::test]
    async fn fails_closed_on_whitespace_only_token() {
        let s = source("   ", false, full_attrs());
        assert!(s.current().await.is_none());
    }

    /// Scenario: The vendor bundle omits the ingestion endpoint.
    /// Guarantees: Missing endpoint routing causes the adapter to fail closed.
    #[tokio::test]
    async fn fails_closed_on_missing_endpoint() {
        let attrs = json!({ "moniker_map": { "default": "mon" } });
        let s = source("tok", false, attrs);
        assert!(s.current().await.is_none());
    }

    /// Scenario: The endpoint or selected moniker contains only whitespace.
    /// Guarantees: Blank routing values are rejected instead of reaching the uploader.
    #[tokio::test]
    async fn fails_closed_on_whitespace_only_routing() {
        let blank_endpoint = json!({
            "endpoint": "   ",
            "moniker_map": { "default": "mon" },
        });
        assert!(
            source("tok", false, blank_endpoint)
                .current()
                .await
                .is_none()
        );

        let blank_moniker = json!({
            "endpoint": "https://ep",
            "moniker_map": { "default": "   " },
        });
        assert!(
            source("tok", false, blank_moniker)
                .current()
                .await
                .is_none()
        );
    }

    /// Scenario: The vendor bundle omits the moniker map.
    /// Guarantees: Missing moniker routing causes the adapter to fail closed.
    #[tokio::test]
    async fn fails_closed_on_missing_moniker() {
        let attrs = json!({ "endpoint": "https://ep" });
        let s = source("tok", false, attrs);
        assert!(s.current().await.is_none());
    }

    /// Scenario: The map contains both account-specific and default monikers.
    /// Guarantees: The configured account mapping takes precedence over default.
    #[tokio::test]
    async fn moniker_prefers_configured_account() {
        let attrs = json!({
            "endpoint": "https://ep",
            "moniker_map": {
                "account": "account-moniker",
                "default": "default-moniker"
            },
        });
        let s = source("tok", false, attrs);
        assert_eq!(s.current().await.unwrap().moniker, "account-moniker");
    }

    /// Scenario: The configured account has no entry and a default is present.
    /// Guarantees: The adapter selects the explicit default moniker.
    #[tokio::test]
    async fn moniker_falls_back_to_default() {
        let attrs = json!({
            "endpoint": "https://ep",
            "moniker_map": {
                "other": "other-moniker",
                "default": "default-moniker"
            },
        });
        let s = source("tok", false, attrs);
        assert_eq!(s.current().await.unwrap().moniker, "default-moniker");
    }

    /// Scenario: The map has one non-default entry and no account match.
    /// Guarantees: A sole unambiguous entry is accepted as the moniker.
    #[tokio::test]
    async fn moniker_falls_back_to_sole_entry() {
        let attrs = json!({
            "endpoint": "https://ep",
            "moniker_map": { "only": "sole-moniker" },
        });
        let s = source("tok", false, attrs);
        assert_eq!(s.current().await.unwrap().moniker, "sole-moniker");
    }

    /// Scenario: The account-specific entry is present but malformed.
    /// Guarantees: Invalid account routing fails instead of silently using default.
    #[tokio::test]
    async fn invalid_account_moniker_does_not_fall_back_to_default() {
        let attrs = json!({
            "endpoint": "https://ep",
            "moniker_map": {
                "account": 42,
                "default": "default-moniker"
            },
        });
        let s = source("tok", false, attrs);
        assert!(s.current().await.is_none());
    }

    /// Scenario: Multiple map entries exist with no account or default match.
    /// Guarantees: Ambiguous routing is rejected instead of selecting arbitrarily.
    #[tokio::test]
    async fn fails_closed_on_ambiguous_moniker_map() {
        let attrs = json!({
            "endpoint": "https://ep",
            "moniker_map": { "b": "mb", "a": "ma" },
        });
        let s = source("tok", false, attrs);
        assert!(s.current().await.is_none());
    }

    struct ErrorBearer;

    #[async_trait]
    impl BearerTokenProvider for ErrorBearer {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            Err(
                CapabilityErrorSource::<BearerTokenProviderCap>::new("mock-bearer".into())
                    .error("token unavailable"),
            )
        }

        fn token_stream(&self) -> TokenStream {
            futures::stream::empty::<BearerToken>().boxed()
        }
    }

    struct ErrorVendor;

    impl VendorBundle for ErrorVendor {
        fn attributes(&self) -> Result<Arc<Map<String, Value>>, CapabilityError> {
            Err(
                CapabilityErrorSource::<VendorBundleCap>::new("mock-vendor".into())
                    .error("bundle unavailable"),
            )
        }
    }

    struct RecoveringBearer(AtomicUsize);

    #[async_trait]
    impl BearerTokenProvider for RecoveringBearer {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            if self.0.fetch_add(1, Ordering::Relaxed) == 0 {
                return Err(CapabilityErrorSource::<BearerTokenProviderCap>::new(
                    "recovering-bearer".into(),
                )
                .error("token unavailable"));
            }
            Ok(BearerToken::without_expiry("recovered-token".to_owned()))
        }

        fn token_stream(&self) -> TokenStream {
            futures::stream::empty::<BearerToken>().boxed()
        }
    }

    struct RecoveringVendor(AtomicUsize);

    impl VendorBundle for RecoveringVendor {
        fn attributes(&self) -> Result<Arc<Map<String, Value>>, CapabilityError> {
            if self.0.fetch_add(1, Ordering::Relaxed) == 0 {
                return Err(CapabilityErrorSource::<VendorBundleCap>::new(
                    "recovering-vendor".into(),
                )
                .error("bundle unavailable"));
            }
            Ok(obj(full_attrs()))
        }
    }

    /// Scenario: The bearer-token capability reports an error.
    /// Guarantees: Capability failure does not produce a partial credential.
    #[tokio::test]
    async fn fails_closed_when_token_provider_errors() {
        let source = AgentFedGenevaSource::new(
            Box::new(ErrorBearer),
            Box::new(MockVendor(obj(full_attrs()))),
            "account".to_owned(),
        );
        assert!(source.current().await.is_none());
    }

    /// Scenario: The vendor-bundle capability reports an error.
    /// Guarantees: Routing failure does not produce a token-only credential.
    #[tokio::test]
    async fn fails_closed_when_vendor_bundle_errors() {
        let source = AgentFedGenevaSource::new(
            Box::new(MockBearer {
                token: "tok".to_owned(),
                yield_first: false,
            }),
            Box::new(ErrorVendor),
            "account".to_owned(),
        );
        assert!(source.current().await.is_none());
    }

    /// Scenario: Token and bundle providers recover after initial failures.
    /// Guarantees: Subsequent reads converge without reconstructing the exporter.
    #[tokio::test]
    async fn recovers_when_capabilities_become_available() {
        let source = AgentFedGenevaSource::new(
            Box::new(RecoveringBearer(AtomicUsize::new(0))),
            Box::new(RecoveringVendor(AtomicUsize::new(0))),
            "account".to_owned(),
        );

        assert!(source.current().await.is_none());
        assert!(source.current().await.is_none());
        let credential = source.current().await.expect("recovered credential");
        assert_eq!(credential.token, "recovered-token");
        assert_eq!(credential.endpoint, "https://ep");
        assert_eq!(credential.moniker, "mon");
    }

    struct RotatingBearer(AtomicUsize);

    #[async_trait]
    impl BearerTokenProvider for RotatingBearer {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            let sequence = self.0.fetch_add(1, Ordering::Relaxed) + 1;
            Ok(BearerToken::without_expiry(format!("token-{sequence}")))
        }

        fn token_stream(&self) -> TokenStream {
            futures::stream::empty::<BearerToken>().boxed()
        }
    }

    /// Scenario: The provider changes its cached bearer token between reads.
    /// Guarantees: Every upload lookup observes the provider's current token.
    #[tokio::test]
    async fn observes_token_rotation_on_each_read() {
        let source = AgentFedGenevaSource::new(
            Box::new(RotatingBearer(AtomicUsize::new(0))),
            Box::new(MockVendor(obj(full_attrs()))),
            "account".to_owned(),
        );
        assert_eq!(source.current().await.unwrap().token, "token-1");
        assert_eq!(source.current().await.unwrap().token, "token-2");
    }

    /// Scenario: Credential failures continue and later recover.
    /// Guarantees: Warning sampling follows powers of two and resets after success.
    #[test]
    fn failure_log_limiter_uses_exponential_sampling_and_resets() {
        let limiter = FailureLogLimiter::default();
        assert_eq!(limiter.record_failure(), Some(1));
        assert_eq!(limiter.record_failure(), Some(2));
        assert_eq!(limiter.record_failure(), None);
        assert_eq!(limiter.record_failure(), Some(4));
        limiter.record_success();
        assert_eq!(limiter.record_failure(), Some(1));
    }

    /// Scenario: Empty-token and routing validation fail independently.
    /// Guarantees: One failure category cannot suppress another category's warning.
    #[test]
    fn failure_categories_are_sampled_independently() {
        let source = source("tok", false, full_attrs());

        assert_eq!(source.empty_token_failures.record_failure(), Some(1));
        assert_eq!(source.empty_token_failures.record_failure(), Some(2));
        assert_eq!(source.routing_failures.record_failure(), Some(1));
    }
}
