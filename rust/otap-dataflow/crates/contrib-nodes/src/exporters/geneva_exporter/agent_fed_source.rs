// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Agent-fed credential adapter for the Geneva exporter.
//!
//! Bridges the atomic engine
//! `agent_fed_credential_provider` capability to geneva-uploader's
//! [`AgentFedCredentialSource`], so the uploader uses the host-provisioned token
//! and routing instead of the GCS handshake.

use geneva_uploader::client::{
    AgentFedCredential, AgentFedCredentialFuture, AgentFedCredentialSource,
};
use otap_df_engine::shared::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider;
use otap_df_telemetry::otel_warn;
use serde_json::{Map, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use url::Url;

const ENDPOINT_KEY: &str = "endpoint";
const MONIKER_MAP_KEY: &str = "moniker_map";
const DEFAULT_MONIKER_KEY: &str = "default";
const TOKEN_USABLE_MARGIN: Duration = Duration::from_secs(30);

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

/// Adapts the engine's atomic agent-fed credential capability to the uploader.
///
/// The resolved `shared` trait object is `Send` but not necessarily `Sync`; a
/// [`tokio::sync::Mutex`] restores `Sync` and lets reads await under a `Send`
/// guard (satisfying `AgentFedCredentialSource: Send + Sync`).
pub(crate) struct AgentFedGenevaSource {
    credential_provider: Mutex<Box<dyn AgentFedCredentialProvider>>,
    account: String,
    credential_failures: FailureLogLimiter,
    empty_token_failures: FailureLogLimiter,
    expiry_failures: FailureLogLimiter,
    routing_failures: FailureLogLimiter,
}

impl std::fmt::Debug for AgentFedGenevaSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AgentFedGenevaSource")
    }
}

impl AgentFedGenevaSource {
    /// Builds the adapter from the resolved shared capability.
    pub(crate) fn new(
        credential_provider: Box<dyn AgentFedCredentialProvider>,
        account: String,
    ) -> Self {
        Self {
            credential_provider: Mutex::new(credential_provider),
            account,
            credential_failures: FailureLogLimiter::default(),
            empty_token_failures: FailureLogLimiter::default(),
            expiry_failures: FailureLogLimiter::default(),
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
            let snapshot = match self.credential_provider.lock().await.get_credential().await {
                Ok(snapshot) => {
                    self.credential_failures.record_success();
                    snapshot
                }
                Err(error) => {
                    if let Some(consecutive_failures) = self.credential_failures.record_failure() {
                        otel_warn!(
                            "geneva_exporter.agent_fed.credential_unavailable",
                            error = %error,
                            consecutive_failures = consecutive_failures
                        );
                    }
                    return None;
                }
            };
            let token = snapshot.token();
            if token.expose_token().trim().is_empty() {
                Self::log_invalid_credential(
                    &self.empty_token_failures,
                    "bearer token is empty or whitespace",
                );
                return None;
            }
            self.empty_token_failures.record_success();
            if token
                .expires_on()
                .is_some_and(|expires_on| expires_on <= Instant::now() + TOKEN_USABLE_MARGIN)
            {
                Self::log_invalid_credential(
                    &self.expiry_failures,
                    "bearer token is expired or near expiry",
                );
                return None;
            }
            self.expiry_failures.record_success();

            let (endpoint, moniker) = match resolve_routing(snapshot.attributes(), &self.account) {
                Ok(routing) => {
                    self.routing_failures.record_success();
                    routing
                }
                Err(reason) => {
                    Self::log_invalid_credential(&self.routing_failures, reason);
                    return None;
                }
            };

            // Keep the engine token secret-wrapped across the routing lookup;
            // the uploader creates its zeroizing owned copy only after success.
            Some(AgentFedCredential::new(
                token.expose_token(),
                endpoint,
                moniker,
            ))
        })
    }
}

fn non_blank_string(value: Option<&Value>) -> Option<&str> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn validated_moniker<'a>(
    value: Option<&'a Value>,
    invalid_reason: &'static str,
) -> Result<&'a str, &'static str> {
    let moniker = non_blank_string(value).ok_or(invalid_reason)?;
    if !moniker
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~'))
    {
        return Err(
            "agent-fed routing selected moniker must contain only ASCII URL-unreserved characters",
        );
    }
    Ok(moniker)
}

fn validated_endpoint(value: Option<&Value>) -> Result<&str, &'static str> {
    let endpoint =
        non_blank_string(value).ok_or("agent-fed routing endpoint is missing or empty")?;
    let parsed = Url::parse(endpoint)
        .map_err(|_| "agent-fed routing endpoint is not a valid absolute URL")?;

    if parsed.scheme() != "https" {
        return Err("agent-fed routing endpoint must use https");
    }
    if parsed.host_str().is_none() {
        return Err("agent-fed routing endpoint must include a host");
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("agent-fed routing endpoint must not include credentials");
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("agent-fed routing endpoint must not include a query or fragment");
    }

    Ok(endpoint)
}

fn resolve_routing<'a>(
    attributes: &'a Map<String, Value>,
    account: &str,
) -> Result<(&'a str, &'a str), &'static str> {
    let endpoint = validated_endpoint(attributes.get(ENDPOINT_KEY))?;
    let moniker_map = attributes
        .get(MONIKER_MAP_KEY)
        .and_then(Value::as_object)
        .ok_or("agent-fed routing moniker_map is missing or invalid")?;

    let account = account.trim();
    let moniker = if let Some(value) = moniker_map.get(account) {
        validated_moniker(
            Some(value),
            "agent-fed routing moniker for the configured account is invalid or empty",
        )?
    } else if let Some(value) = moniker_map.get(DEFAULT_MONIKER_KEY) {
        validated_moniker(
            Some(value),
            "agent-fed routing default moniker is invalid or empty",
        )?
    } else if moniker_map.is_empty() {
        return Err("agent-fed routing moniker_map is empty");
    } else {
        return Err(
            "agent-fed routing moniker_map has no entry for the configured account or default",
        );
    };

    Ok((endpoint, moniker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use otap_df_engine::capability::auth::BearerToken;
    use otap_df_engine::capability::auth::agent_fed_credential_provider::{
        AgentFedCredentialProvider as AgentFedCredentialProviderCap, AgentFedCredentialSnapshot,
    };
    use otap_df_engine::capability::{CapabilityError, CapabilityErrorSource};
    use serde_json::{Map, Value, json};
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Arc, RwLock};

    struct MockCredential {
        token: BearerToken,
        yield_first: bool,
        attributes: Arc<Map<String, Value>>,
    }

    #[async_trait]
    impl AgentFedCredentialProvider for MockCredential {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            if self.yield_first {
                tokio::task::yield_now().await;
            }
            Ok(Arc::new(AgentFedCredentialSnapshot::new(
                self.token.clone(),
                Arc::clone(&self.attributes),
            )))
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
        source_with_token(
            BearerToken::without_expiry(token.to_owned()),
            yield_first,
            attrs,
            account,
        )
    }

    fn source_with_token(
        token: BearerToken,
        yield_first: bool,
        attrs: Value,
        account: &str,
    ) -> AgentFedGenevaSource {
        AgentFedGenevaSource::new(
            Box::new(MockCredential {
                token,
                yield_first,
                attributes: obj(attrs),
            }),
            account.to_owned(),
        )
    }

    fn full_attrs() -> Value {
        json!({
            "endpoint": "https://ep",
            "moniker_map": { "default": "mon" },
        })
    }

    /// Scenario: The provider returns a snapshot containing valid token and routing data.
    /// Guarantees: The adapter returns the exact token, endpoint, and moniker.
    #[tokio::test]
    async fn returns_credential_when_token_and_routing_present() {
        let s = source("tok", false, full_attrs());
        let c = s.current().await.expect("credential");
        assert_eq!(c.expose_token(), "tok");
        assert_eq!(c.endpoint, "https://ep");
        assert_eq!(c.moniker, "mon");
    }

    /// Scenario: The credential-provider future yields once before returning a snapshot.
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

    /// Scenario: A credential token is expired, near expiry, or comfortably usable.
    /// Guarantees: Uploads reject tokens inside the safety margin and accept later expiry.
    #[tokio::test]
    async fn token_expiry_enforces_safety_margin() {
        for expires_on in [Instant::now(), Instant::now() + TOKEN_USABLE_MARGIN] {
            let s = source_with_token(
                BearerToken::with_expiry("tok".to_owned(), Some(expires_on)),
                false,
                full_attrs(),
                "account",
            );
            assert!(s.current().await.is_none());
        }

        let s = source_with_token(
            BearerToken::with_expiry(
                "tok".to_owned(),
                Some(Instant::now() + TOKEN_USABLE_MARGIN + Duration::from_secs(60)),
            ),
            false,
            full_attrs(),
            "account",
        );
        assert!(s.current().await.is_some());
    }

    /// Scenario: The credential snapshot omits the ingestion endpoint.
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

    /// Scenario: The credential snapshot supplies an unsafe or malformed ingestion endpoint.
    /// Guarantees: Only absolute HTTPS endpoints without credentials, queries, or fragments pass.
    #[tokio::test]
    async fn fails_closed_on_invalid_endpoint_urls() {
        for endpoint in [
            "not a url",
            "http://ep",
            "https://user:password@ep",
            "https://ep?query=value",
            "https://ep#fragment",
        ] {
            let attrs = json!({
                "endpoint": endpoint,
                "moniker_map": { "default": "mon" },
            });
            assert!(
                source("tok", false, attrs).current().await.is_none(),
                "endpoint should be rejected: {endpoint}"
            );
        }
    }

    /// Scenario: The credential snapshot omits the moniker map.
    /// Guarantees: Missing moniker routing causes the adapter to fail closed.
    #[tokio::test]
    async fn fails_closed_on_missing_moniker() {
        let attrs = json!({ "endpoint": "https://ep" });
        let s = source("tok", false, attrs);
        assert!(s.current().await.is_none());
    }

    /// Scenario: The moniker map is empty or is not a JSON object.
    /// Guarantees: Malformed routing cannot produce an uploader credential.
    #[tokio::test]
    async fn fails_closed_on_invalid_moniker_map_shapes() {
        for moniker_map in [json!({}), json!(["not", "an", "object"])] {
            let attrs = json!({
                "endpoint": "https://ep",
                "moniker_map": moniker_map,
            });
            assert!(source("tok", false, attrs).current().await.is_none());
        }
    }

    /// Scenario: The selected moniker contains bytes that require URL encoding.
    /// Guarantees: Reserved, whitespace, and non-ASCII values cannot alter the upload query.
    #[tokio::test]
    async fn fails_closed_on_monikers_requiring_url_encoding() {
        for moniker in [
            "moniker&namespace=other",
            "moniker#fragment",
            "moniker with space",
            "moniker%26encoded",
            "moniker*reserved",
            "moniker/non-ascii-\u{00e9}",
        ] {
            let attrs = json!({
                "endpoint": "https://ep",
                "moniker_map": { "default": moniker },
            });
            assert!(
                source("tok", false, attrs).current().await.is_none(),
                "moniker should be rejected: {moniker}"
            );
        }
    }

    /// Scenario: The selected moniker uses every supported URL-safe character class.
    /// Guarantees: ASCII letters, digits, hyphen, dot, underscore, and tilde remain valid.
    #[tokio::test]
    async fn accepts_ascii_url_unreserved_moniker() {
        let moniker = "AZaz09-._~";
        let attrs = json!({
            "endpoint": "https://ep",
            "moniker_map": { "default": moniker },
        });

        assert_eq!(
            source("tok", false, attrs)
                .current()
                .await
                .expect("credential")
                .moniker,
            moniker
        );
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

    /// Scenario: The map has one unrelated entry and no account or default match.
    /// Guarantees: A sole unrelated entry is rejected instead of selected implicitly.
    #[tokio::test]
    async fn fails_closed_on_sole_unmatched_moniker() {
        let attrs = json!({
            "endpoint": "https://ep",
            "moniker_map": { "only": "sole-moniker" },
        });
        let s = source("tok", false, attrs);
        assert!(s.current().await.is_none());
    }

    /// Scenario: An explicitly selected account or default entry is malformed.
    /// Guarantees: Invalid selected routing fails instead of using another map entry.
    #[tokio::test]
    async fn invalid_selected_moniker_does_not_fall_back() {
        let invalid_account = json!({
            "endpoint": "https://ep",
            "moniker_map": {
                "account": 42,
                "default": "default-moniker"
            },
        });
        assert!(
            source("tok", false, invalid_account)
                .current()
                .await
                .is_none()
        );

        let invalid_default = json!({
            "endpoint": "https://ep",
            "moniker_map": {
                "default": 42,
                "other": "other-moniker"
            },
        });
        assert!(
            source("tok", false, invalid_default)
                .current()
                .await
                .is_none()
        );
    }

    /// Scenario: Multiple unrelated entries exist with no account or default match.
    /// Guarantees: Unmatched routing is rejected instead of selecting arbitrarily.
    #[tokio::test]
    async fn fails_closed_on_multiple_unmatched_monikers() {
        let attrs = json!({
            "endpoint": "https://ep",
            "moniker_map": { "b": "mb", "a": "ma" },
        });
        let s = source("tok", false, attrs);
        assert!(s.current().await.is_none());
    }

    struct ErrorCredential;

    #[async_trait]
    impl AgentFedCredentialProvider for ErrorCredential {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            Err(CapabilityErrorSource::<AgentFedCredentialProviderCap>::new(
                "mock-credential".into(),
            )
            .error("credential unavailable"))
        }
    }

    struct RecoveringCredential(AtomicUsize);

    #[async_trait]
    impl AgentFedCredentialProvider for RecoveringCredential {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            if self.0.fetch_add(1, Ordering::Relaxed) == 0 {
                return Err(CapabilityErrorSource::<AgentFedCredentialProviderCap>::new(
                    "recovering-credential".into(),
                )
                .error("credential unavailable"));
            }
            Ok(Arc::new(AgentFedCredentialSnapshot::new(
                BearerToken::without_expiry("recovered-token".to_owned()),
                obj(full_attrs()),
            )))
        }
    }

    /// Scenario: The atomic credential capability reports an error.
    /// Guarantees: Capability failure does not produce a partial credential.
    #[tokio::test]
    async fn fails_closed_when_credential_provider_errors() {
        let source = AgentFedGenevaSource::new(Box::new(ErrorCredential), "account".to_owned());
        assert!(source.current().await.is_none());
    }

    /// Scenario: The credential provider recovers after an initial failure.
    /// Guarantees: Subsequent reads recover without reconstructing the exporter.
    #[tokio::test]
    async fn recovers_when_credential_becomes_available() {
        let source = AgentFedGenevaSource::new(
            Box::new(RecoveringCredential(AtomicUsize::new(0))),
            "account".to_owned(),
        );

        assert!(source.current().await.is_none());
        let credential = source.current().await.expect("recovered credential");
        assert_eq!(credential.expose_token(), "recovered-token");
        assert_eq!(credential.endpoint, "https://ep");
        assert_eq!(credential.moniker, "mon");
    }

    struct RotatingCredential(AtomicUsize);

    #[async_trait]
    impl AgentFedCredentialProvider for RotatingCredential {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            let sequence = self.0.fetch_add(1, Ordering::Relaxed) + 1;
            Ok(Arc::new(AgentFedCredentialSnapshot::new(
                BearerToken::without_expiry(format!("token-{sequence}")),
                obj(json!({
                    "endpoint": format!("https://endpoint-{sequence}"),
                    "moniker_map": { "default": format!("moniker-{sequence}") },
                })),
            )))
        }
    }

    /// Scenario: The provider rotates its complete credential snapshot.
    /// Guarantees: Every lookup observes token and routing from one generation.
    #[tokio::test]
    async fn observes_coherent_rotation_on_each_read() {
        let source = AgentFedGenevaSource::new(
            Box::new(RotatingCredential(AtomicUsize::new(0))),
            "account".to_owned(),
        );

        let first = source.current().await.expect("first credential");
        assert_eq!(first.expose_token(), "token-1");
        assert_eq!(first.endpoint, "https://endpoint-1");
        assert_eq!(first.moniker, "moniker-1");

        let second = source.current().await.expect("second credential");
        assert_eq!(second.expose_token(), "token-2");
        assert_eq!(second.endpoint, "https://endpoint-2");
        assert_eq!(second.moniker, "moniker-2");
    }

    struct RotateAfterLoadCredential {
        current: Arc<RwLock<Arc<AgentFedCredentialSnapshot>>>,
        replacement: Arc<AgentFedCredentialSnapshot>,
    }

    #[async_trait]
    impl AgentFedCredentialProvider for RotateAfterLoadCredential {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            let loaded = {
                let mut current = self.current.write().expect("snapshot write lock");
                let loaded = Arc::clone(&current);
                *current = Arc::clone(&self.replacement);
                loaded
            };
            tokio::task::yield_now().await;
            Ok(loaded)
        }
    }

    fn snapshot(token: &str, endpoint: &str, moniker: &str) -> Arc<AgentFedCredentialSnapshot> {
        Arc::new(AgentFedCredentialSnapshot::new(
            BearerToken::without_expiry(token.to_owned()),
            obj(json!({
                "endpoint": endpoint,
                "moniker_map": { "default": moniker },
            })),
        ))
    }

    /// Scenario: Host state rotates after the provider atomically loads a snapshot.
    /// Guarantees: The in-flight read returns one complete generation, never a mixed pair.
    #[tokio::test]
    async fn rotation_during_read_preserves_atomic_snapshot() {
        let before = snapshot(
            "token-before-rotation",
            "https://endpoint-before-rotation",
            "moniker-before-rotation",
        );
        let after = snapshot(
            "token-after-rotation",
            "https://endpoint-after-rotation",
            "moniker-after-rotation",
        );
        let source = AgentFedGenevaSource::new(
            Box::new(RotateAfterLoadCredential {
                current: Arc::new(RwLock::new(before)),
                replacement: Arc::clone(&after),
            }),
            "account".to_owned(),
        );

        let credential = source.current().await.expect("atomic credential");
        assert_eq!(credential.expose_token(), "token-before-rotation");
        assert_eq!(credential.endpoint, "https://endpoint-before-rotation");
        assert_eq!(credential.moniker, "moniker-before-rotation");
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

    /// Scenario: Empty-token, expiry, and routing validation fail independently.
    /// Guarantees: One invalid-data category cannot suppress another category's warning.
    #[test]
    fn failure_categories_are_sampled_independently() {
        let source = source("tok", false, full_attrs());

        assert_eq!(source.empty_token_failures.record_failure(), Some(1));
        assert_eq!(source.empty_token_failures.record_failure(), Some(2));
        assert_eq!(source.expiry_failures.record_failure(), Some(1));
        assert_eq!(source.routing_failures.record_failure(), Some(1));
    }
}
