// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the Kubernetes SAT authorizer extension.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use otap_df_config::error::Error as ConfigError;
use otap_df_engine::capability::auth::{AuthzDecision, BearerToken, DenyReason};
use otap_df_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as SharedBearerTokenAuthorizer;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::testing::EmptyAttributes;

use super::config::{Config, normalize_service_account};
use super::extension::K8sSatTokenAuthorizerExtension;
use super::metrics::{K8sSatTokenAuthorizerMetrics, K8sSatTokenAuthorizerMetricsTracker};
use super::*;

// ── Config tests ───────────────────────────────────────────

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

/// Scenario: parse a minimal config specifying only the required `audiences`.
/// Guarantees: every optional field takes its documented default (cache TTL
/// 300s, cache max 1024, startup timeout 30s, empty allow-list).
#[test]
fn config_defaults_apply() {
    let cfg = config_from_json(serde_json::json!({ "audiences": ["my-service"] }))
        .expect("minimal config is valid");
    assert_eq!(cfg.audiences, vec!["my-service".to_string()]);
    assert!(cfg.allowed_service_accounts.is_empty());
    assert_eq!(cfg.cache_ttl, Duration::from_secs(300));
    assert_eq!(cfg.cache_max_entries, 1024);
    assert_eq!(cfg.startup_timeout, Duration::from_secs(30));
}

/// Scenario: parse configs that omit `audiences` entirely and that supply an
/// empty `audiences` list.
/// Guarantees: both are rejected, so a token is never admitted without an
/// audience constraint.
#[test]
fn audiences_are_required_and_non_empty() {
    assert!(
        config_from_json(serde_json::json!({})).is_err(),
        "missing audiences must be rejected"
    );
    assert!(
        config_from_json(serde_json::json!({ "audiences": [] })).is_err(),
        "empty audiences must be rejected"
    );
    assert!(
        config_from_json(serde_json::json!({ "audiences": ["  "] })).is_err(),
        "whitespace-only audience must be rejected"
    );
}

/// Scenario: parse configs with a zero `cache_ttl`, zero `cache_max_entries`,
/// and zero `startup_timeout`.
/// Guarantees: each zero value is rejected so the extension never runs with a
/// degenerate cache or readiness gate.
#[test]
fn zero_valued_fields_are_rejected() {
    assert!(
        config_from_json(serde_json::json!({ "audiences": ["a"], "cache_ttl": "0s" })).is_err(),
        "zero cache_ttl must be rejected"
    );
    assert!(
        config_from_json(serde_json::json!({ "audiences": ["a"], "cache_max_entries": 0 }))
            .is_err(),
        "zero cache_max_entries must be rejected"
    );
    assert!(
        config_from_json(serde_json::json!({ "audiences": ["a"], "startup_timeout": "0s" }))
            .is_err(),
        "zero startup_timeout must be rejected"
    );
}

/// Scenario: parse a config carrying a field name the schema does not define.
/// Guarantees: an unknown field is rejected (deny_unknown_fields), catching
/// typos rather than silently ignoring them.
#[test]
fn unknown_field_is_rejected() {
    let err = config_from_json(serde_json::json!({ "audiences": ["a"], "typo": true }))
        .expect_err("unknown field must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

/// Scenario: parse human-readable durations for `cache_ttl` and
/// `startup_timeout`.
/// Guarantees: the durations deserialize to the exact wall-clock values.
#[test]
fn human_readable_durations_parse() {
    let cfg = config_from_json(serde_json::json!({
        "audiences": ["a"],
        "cache_ttl": "90s",
        "startup_timeout": "1m",
    }))
    .expect("durations parse");
    assert_eq!(cfg.cache_ttl, Duration::from_secs(90));
    assert_eq!(cfg.startup_timeout, Duration::from_secs(60));
}

/// Scenario: parse a config whose `allowed_service_accounts` contains a
/// malformed entry (empty name).
/// Guarantees: validation fails at wiring time rather than silently never
/// matching the entry at request time.
#[test]
fn malformed_allow_list_entry_is_rejected() {
    assert!(
        config_from_json(serde_json::json!({
            "audiences": ["a"],
            "allowed_service_accounts": ["default:"],
        }))
        .is_err(),
        "allow-list entry with empty name must be rejected"
    );
}

// ── Service-account normalization tests ────────────────────

/// Scenario: normalize the three accepted allow-list shapes (full username,
/// `<ns>/<name>`, `<ns>:<name>`).
/// Guarantees: all three canonicalize to the same full
/// `system:serviceaccount:<ns>:<name>` username the API server returns.
#[test]
fn normalize_accepts_all_forms() {
    let canonical = "system:serviceaccount:default:my-sa".to_string();
    assert_eq!(
        normalize_service_account("system:serviceaccount:default:my-sa"),
        Ok(canonical.clone())
    );
    assert_eq!(
        normalize_service_account("default/my-sa"),
        Ok(canonical.clone())
    );
    assert_eq!(normalize_service_account("default:my-sa"), Ok(canonical));
}

/// Scenario: normalize entries with an empty namespace or name, and an entry
/// with no separator.
/// Guarantees: every malformed entry is rejected.
#[test]
fn normalize_rejects_malformed() {
    assert!(normalize_service_account("").is_err());
    assert!(normalize_service_account("only-one-part").is_err());
    assert!(normalize_service_account("/my-sa").is_err());
    assert!(normalize_service_account("default/").is_err());
    assert!(normalize_service_account("system:serviceaccount:default").is_err());
}

/// Scenario: build the canonical allow-list set from mixed-shape entries, and
/// from an empty list.
/// Guarantees: the set contains the canonical usernames, and an empty list
/// yields `None` (admit any authenticated account).
#[test]
fn allowed_set_canonicalizes_and_empty_is_none() {
    let cfg = config_from_json(serde_json::json!({
        "audiences": ["a"],
        "allowed_service_accounts": ["default/reader", "system:serviceaccount:kube-system:writer"],
    }))
    .expect("valid config");
    let set = cfg
        .allowed_service_account_set()
        .expect("non-empty allow-list");
    assert!(set.contains("system:serviceaccount:default:reader"));
    assert!(set.contains("system:serviceaccount:kube-system:writer"));

    let cfg_empty = config_from_json(serde_json::json!({ "audiences": ["a"] })).unwrap();
    assert!(cfg_empty.allowed_service_account_set().is_none());
}

// ── Factory registration tests ─────────────────────────────

/// Scenario: inspect the linkme-registered factory entry.
/// Guarantees: the factory is registered under the documented URN and advertises
/// the `bearer_token_authorizer` capability on its shared variant.
#[test]
fn factory_is_registered_with_capability() {
    assert_eq!(
        K8S_SAT_TOKEN_AUTHORIZER_EXTENSION.name,
        K8S_SAT_TOKEN_AUTHORIZER_URN
    );
    let capabilities = K8S_SAT_TOKEN_AUTHORIZER_EXTENSION
        .capabilities
        .as_ref()
        .expect("active extension advertises capabilities");
    assert!(
        capabilities.shared.contains(&"bearer_token_authorizer"),
        "BearerTokenAuthorizer must be advertised as a shared capability"
    );
}

/// Scenario: run the static `validate_config` hook against a valid and an
/// invalid config value.
/// Guarantees: it accepts a well-formed config and rejects one missing the
/// required audiences.
#[test]
fn validate_config_hook_accepts_valid_and_rejects_invalid() {
    assert!(validate_config(&serde_json::json!({ "audiences": ["svc"] })).is_ok());
    assert!(validate_config(&serde_json::json!({})).is_err());
}

// ── Extension behavior tests ───────────────────────────────

fn make_tracker() -> K8sSatTokenAuthorizerMetricsTracker {
    let registry = TelemetryRegistryHandle::new();
    let metric_set =
        registry.register_metric_set::<K8sSatTokenAuthorizerMetrics>(EmptyAttributes());
    K8sSatTokenAuthorizerMetricsTracker::new(metric_set)
}

fn make_extension(allowed: Option<HashSet<String>>) -> K8sSatTokenAuthorizerExtension {
    K8sSatTokenAuthorizerExtension::new(
        "test-authorizer",
        vec!["my-service".to_string()],
        allowed,
        Duration::from_secs(300),
        1024,
        make_tracker(),
    )
}

/// Scenario: admit an authenticated identity when no allow-list is configured.
/// Guarantees: the request is allowed and the returned identity carries the
/// authenticated subject and the audience.
#[test]
fn admit_without_allow_list_allows_any_authenticated() {
    let ext = make_extension(None);
    let decision = ext.admit_for_test(
        Some("system:serviceaccount:default:my-sa".to_string()),
        vec!["my-service".to_string()],
    );
    assert!(decision.is_allowed());
    let identity = decision.identity().expect("allow carries identity");
    assert_eq!(
        identity.subject(),
        Some("system:serviceaccount:default:my-sa")
    );
    assert_eq!(identity.audience(), Some("my-service"));
}

/// Scenario: admit an identity that is present in the configured allow-list.
/// Guarantees: the matching service account is allowed.
#[test]
fn admit_allows_service_account_in_allow_list() {
    let allowed: HashSet<String> = ["system:serviceaccount:default:my-sa".to_string()]
        .into_iter()
        .collect();
    let ext = make_extension(Some(allowed));
    let decision = ext.admit_for_test(
        Some("system:serviceaccount:default:my-sa".to_string()),
        vec!["my-service".to_string()],
    );
    assert!(decision.is_allowed());
}

/// Scenario: admit an authenticated identity that is absent from the allow-list,
/// and one with no username at all.
/// Guarantees: both are denied with `NotPermitted` (admission failure after
/// successful authentication).
#[test]
fn admit_denies_service_account_absent_from_allow_list() {
    let allowed: HashSet<String> = ["system:serviceaccount:default:allowed".to_string()]
        .into_iter()
        .collect();
    let ext = make_extension(Some(allowed));

    let denied = ext.admit_for_test(
        Some("system:serviceaccount:default:other".to_string()),
        vec!["my-service".to_string()],
    );
    assert!(!denied.is_allowed());
    assert_eq!(
        denied,
        AuthzDecision::deny_with_detail(
            DenyReason::NotPermitted,
            "service account not in allow-list"
        )
    );

    let no_user = ext.admit_for_test(None, vec!["my-service".to_string()]);
    assert!(!no_user.is_allowed());
}

/// Scenario: authorize an empty credential.
/// Guarantees: it is denied with `MissingCredential` without contacting the API
/// server (the reviewer is never initialized in this test).
#[tokio::test]
async fn authorize_empty_credential_is_missing() {
    let ext = make_extension(None);
    let decision = ext
        .authorize(&BearerToken::without_expiry(String::new()))
        .await
        .expect("empty credential yields a decision, not an error");
    assert_eq!(decision, AuthzDecision::deny(DenyReason::MissingCredential));
}

/// Scenario: authorize a non-empty token before the Kubernetes client has been
/// constructed (reviewer not yet published).
/// Guarantees: the call fails closed with a capability error rather than
/// allowing the request while undetermined.
#[tokio::test]
async fn authorize_before_ready_fails_closed() {
    let ext = make_extension(None);
    let result = ext
        .authorize(&BearerToken::without_expiry("some-token".to_string()))
        .await;
    assert!(
        result.is_err(),
        "a request before readiness must fail closed, not allow"
    );
}

// ── Decision cache tests ───────────────────────────────────

/// Scenario: insert a decision and read it back before and after its TTL
/// elapses.
/// Guarantees: a fresh entry is returned; once expired it is treated as absent
/// (forcing a fresh TokenReview).
#[test]
fn cache_returns_fresh_and_drops_expired() {
    let ext = make_extension(None);
    let now = Instant::now();
    let decision = AuthzDecision::allow_anonymous();
    ext.cache_insert_for_test("tok", decision.clone(), now);

    assert_eq!(ext.cache_get_for_test("tok", now), Some(decision));
    // Just past the 300s TTL the entry is gone.
    let later = now + Duration::from_secs(301);
    assert_eq!(ext.cache_get_for_test("tok", later), None);
}

/// Scenario: insert more distinct tokens than the cache capacity allows, all
/// unexpired.
/// Guarantees: the cache never exceeds `cache_max_entries`, bounding memory.
#[test]
fn cache_respects_max_entries() {
    let ext = K8sSatTokenAuthorizerExtension::new(
        "test-authorizer",
        vec!["my-service".to_string()],
        None,
        Duration::from_secs(300),
        2,
        make_tracker(),
    );
    let now = Instant::now();
    for i in 0..10 {
        ext.cache_insert_for_test(&format!("tok-{i}"), AuthzDecision::allow_anonymous(), now);
    }
    assert!(
        ext.cache_len_for_test() <= 2,
        "cache must not exceed its max_entries bound"
    );
}
