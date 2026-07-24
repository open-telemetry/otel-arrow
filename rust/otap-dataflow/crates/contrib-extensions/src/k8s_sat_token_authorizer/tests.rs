// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the Kubernetes SAT authorizer extension.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use otap_df_config::error::Error as ConfigError;
use otap_df_engine::capability::auth::{AuthzDecision, BearerToken, DenyReason};
use otap_df_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as SharedBearerTokenAuthorizer;

use super::config::{Config, ResourceAttributesConfig, normalize_service_account};
use super::extension::K8sSatTokenAuthorizerExtension;
use super::*;

// ── Config tests ───────────────────────────────────────────

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

/// Scenario: parse a minimal config specifying only the required `audiences`.
/// Guarantees: every optional field takes its documented default (cache TTL
/// 300s, cache max 1024, empty allow-list).
#[test]
fn config_defaults_apply() {
    let cfg = config_from_json(serde_json::json!({ "audiences": ["my-service"] }))
        .expect("minimal config is valid");
    assert_eq!(cfg.audiences, vec!["my-service".to_string()]);
    assert!(cfg.allowed_service_accounts.is_empty());
    assert_eq!(cfg.cache_ttl, Duration::from_secs(300));
    assert_eq!(cfg.cache_max_entries, 1024);
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

/// Scenario: parse configs with a zero `cache_ttl` and zero `cache_max_entries`.
/// Guarantees: each zero value is rejected so the extension never runs with a
/// degenerate cache.
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

/// Scenario: parse a human-readable duration for `cache_ttl`.
/// Guarantees: the duration deserializes to the exact wall-clock value.
#[test]
fn human_readable_durations_parse() {
    let cfg = config_from_json(serde_json::json!({
        "audiences": ["a"],
        "cache_ttl": "90s",
    }))
    .expect("durations parse");
    assert_eq!(cfg.cache_ttl, Duration::from_secs(90));
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

// ── RBAC (SubjectAccessReview) config tests ────────────────

/// Scenario: parse a config with a valid `resource_attributes` RBAC block.
/// Guarantees: the required `resource`/`verb` and optional fields deserialize.
#[test]
fn resource_attributes_parses() {
    let cfg = config_from_json(serde_json::json!({
        "audiences": ["a"],
        "resource_attributes": {
            "group": "telemetry.opentelemetry.io",
            "resource": "telemetry",
            "verb": "export",
            "namespace": "observability",
        },
    }))
    .expect("valid RBAC config parses");
    let ra = cfg
        .resource_attributes
        .expect("resource_attributes present");
    assert_eq!(ra.resource, "telemetry");
    assert_eq!(ra.verb, "export");
    assert_eq!(ra.group.as_deref(), Some("telemetry.opentelemetry.io"));
    assert_eq!(ra.namespace.as_deref(), Some("observability"));
}

/// Scenario: parse `resource_attributes` blocks that omit the required
/// `resource` or `verb`, or leave them blank.
/// Guarantees: each is rejected so an RBAC check is never issued without a
/// resource and verb to authorize.
#[test]
fn resource_attributes_requires_resource_and_verb() {
    // Missing verb (deserialization: required field absent).
    assert!(
        config_from_json(serde_json::json!({
            "audiences": ["a"],
            "resource_attributes": { "resource": "telemetry" },
        }))
        .is_err(),
        "missing verb must be rejected"
    );
    // Missing resource.
    assert!(
        config_from_json(serde_json::json!({
            "audiences": ["a"],
            "resource_attributes": { "verb": "export" },
        }))
        .is_err(),
        "missing resource must be rejected"
    );
    // Blank verb (validation).
    assert!(
        config_from_json(serde_json::json!({
            "audiences": ["a"],
            "resource_attributes": { "resource": "telemetry", "verb": "  " },
        }))
        .is_err(),
        "blank verb must be rejected"
    );
}

/// Scenario: parse a config that sets both `allowed_service_accounts` and
/// `resource_attributes`.
/// Guarantees: the two admission strategies are rejected together, so exactly
/// one admission model is active.
#[test]
fn allow_list_and_rbac_are_mutually_exclusive() {
    assert!(
        config_from_json(serde_json::json!({
            "audiences": ["a"],
            "allowed_service_accounts": ["default/reader"],
            "resource_attributes": { "resource": "telemetry", "verb": "export" },
        }))
        .is_err(),
        "allow-list and RBAC must be mutually exclusive"
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

fn make_extension(allowed: Option<HashSet<String>>) -> K8sSatTokenAuthorizerExtension {
    K8sSatTokenAuthorizerExtension::new(
        "test-authorizer",
        vec!["my-service".to_string()],
        allowed,
        None,
        Duration::from_secs(300),
        1024,
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
        None,
        Duration::from_secs(300),
        2,
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

// ── Live-cluster integration tests ─────────────────────────
//
// These are #[ignore]d: they require a reachable Kubernetes cluster (via the
// ambient kubeconfig/in-cluster config) and a valid projected service-account
// token supplied through the environment. Run them explicitly with, e.g.:
//
//   K8S_SAT_TOKEN="$(kubectl create token sat-tester -n sat-authz-test \
//     --audience=https://sat-authz-test.example)" \
//   cargo test -p otap-df-contrib-extensions \
//     --features k8s-sat-token-authorizer-extension \
//     k8s_sat_token_authorizer -- --ignored --nocapture
//
// The cluster is expected to have the fixtures from the extension's test setup:
// namespace `sat-authz-test`, service account `sat-tester`, and a Role granting
// `get`/`list` on `pods` in that namespace. Apply `testdata/integration-fixtures.yaml`
// to create them.

/// Default audience the test token is minted for; override with `K8S_SAT_AUDIENCE`.
fn it_audience() -> String {
    std::env::var("K8S_SAT_AUDIENCE")
        .unwrap_or_else(|_| "https://sat-authz-test.example".to_string())
}

/// The expected authenticated subject; override with `K8S_SAT_SUBJECT`.
fn it_subject() -> String {
    std::env::var("K8S_SAT_SUBJECT")
        .unwrap_or_else(|_| "system:serviceaccount:sat-authz-test:sat-tester".to_string())
}

/// Namespace used for RBAC checks; override with `K8S_SAT_NAMESPACE`.
fn it_namespace() -> String {
    std::env::var("K8S_SAT_NAMESPACE").unwrap_or_else(|_| "sat-authz-test".to_string())
}

/// Returns the token under test, or `None` when `K8S_SAT_TOKEN` is unset (so the
/// ignored test no-ops instead of failing when run without a configured token).
fn it_token() -> Option<String> {
    std::env::var("K8S_SAT_TOKEN")
        .ok()
        .filter(|t| !t.trim().is_empty())
}

fn it_extension(
    allowed: Option<HashSet<String>>,
    resource_attributes: Option<ResourceAttributesConfig>,
) -> K8sSatTokenAuthorizerExtension {
    K8sSatTokenAuthorizerExtension::new(
        "it-authorizer",
        vec![it_audience()],
        allowed,
        resource_attributes,
        Duration::from_secs(300),
        1024,
    )
}

/// Scenario: a valid projected service-account token is authorized with no
/// admission policy (audience-only) against a live cluster.
/// Guarantees: `TokenReview` authenticates the token and the request is admitted
/// with the authenticated service-account subject.
#[tokio::test]
#[ignore = "requires a live Kubernetes cluster and K8S_SAT_TOKEN"]
async fn it_valid_token_is_admitted_audience_only() {
    let Some(token) = it_token() else {
        eprintln!("skipping: set K8S_SAT_TOKEN to run this integration test");
        return;
    };
    let ext = it_extension(None, None);
    let decision = ext
        .authorize(&BearerToken::without_expiry(token))
        .await
        .expect("TokenReview must reach a decision");
    assert!(decision.is_allowed(), "valid token must be admitted");
    assert_eq!(
        decision.identity().and_then(|i| i.subject()),
        Some(it_subject().as_str()),
        "identity subject must be the authenticated service account"
    );
}

/// Scenario: a syntactically-bogus bearer token is submitted to a live cluster.
/// Guarantees: `TokenReview` reaches a verdict (not an error) and the request is
/// denied as an invalid credential.
#[tokio::test]
#[ignore = "requires a live Kubernetes cluster"]
async fn it_bogus_token_is_denied_invalid() {
    // This test needs a cluster but not a real token; gate it on the same env so
    // it only runs as part of a configured integration run.
    if it_token().is_none() {
        eprintln!("skipping: set K8S_SAT_TOKEN to run this integration test");
        return;
    }
    let ext = it_extension(None, None);
    let decision = ext
        .authorize(&BearerToken::without_expiry("not.a.real.token".to_string()))
        .await
        .expect("TokenReview must reach a decision, not error");
    assert!(!decision.is_allowed(), "a bogus token must not be admitted");
    assert!(
        matches!(
            decision,
            AuthzDecision::Deny {
                reason: DenyReason::InvalidCredential,
                ..
            }
        ),
        "a bogus token must be denied as an invalid credential, got {decision:?}"
    );
}

/// Scenario: allow-list admission against a live cluster, both with the
/// authenticated subject present and absent.
/// Guarantees: the matching subject is admitted and a non-matching allow-list
/// denies with `NotPermitted` after successful authentication.
#[tokio::test]
#[ignore = "requires a live Kubernetes cluster and K8S_SAT_TOKEN"]
async fn it_allow_list_admits_and_denies() {
    let Some(token) = it_token() else {
        eprintln!("skipping: set K8S_SAT_TOKEN to run this integration test");
        return;
    };

    let permitted: HashSet<String> = [it_subject()].into_iter().collect();
    let ext = it_extension(Some(permitted), None);
    assert!(
        ext.authorize(&BearerToken::without_expiry(token.clone()))
            .await
            .expect("decision")
            .is_allowed(),
        "subject in the allow-list must be admitted"
    );

    let other: HashSet<String> = ["system:serviceaccount:sat-authz-test:someone-else".to_string()]
        .into_iter()
        .collect();
    let ext = it_extension(Some(other), None);
    let decision = ext
        .authorize(&BearerToken::without_expiry(token))
        .await
        .expect("decision");
    assert!(
        !decision.is_allowed(),
        "subject absent from the allow-list must be denied"
    );
}

/// Scenario: RBAC admission via `SubjectAccessReview` against a live cluster for
/// a permitted verb (`get pods`) and an unpermitted verb (`delete pods`).
/// Guarantees: the permitted action is admitted and the unpermitted action is
/// denied with `NotPermitted`, exercising the real SubjectAccessReview path.
#[tokio::test]
#[ignore = "requires a live Kubernetes cluster, K8S_SAT_TOKEN, and the RBAC fixtures"]
async fn it_rbac_allows_permitted_and_denies_unpermitted() {
    let Some(token) = it_token() else {
        eprintln!("skipping: set K8S_SAT_TOKEN to run this integration test");
        return;
    };
    let namespace = it_namespace();

    let permitted = ResourceAttributesConfig {
        group: None,
        version: None,
        resource: "pods".to_string(),
        verb: "get".to_string(),
        namespace: Some(namespace.clone()),
        name: None,
        subresource: None,
    };
    let ext = it_extension(None, Some(permitted));
    assert!(
        ext.authorize(&BearerToken::without_expiry(token.clone()))
            .await
            .expect("SubjectAccessReview must reach a decision")
            .is_allowed(),
        "RBAC must admit a permitted verb (get pods)"
    );

    let unpermitted = ResourceAttributesConfig {
        group: None,
        version: None,
        resource: "pods".to_string(),
        verb: "delete".to_string(),
        namespace: Some(namespace),
        name: None,
        subresource: None,
    };
    let ext = it_extension(None, Some(unpermitted));
    let decision = ext
        .authorize(&BearerToken::without_expiry(token))
        .await
        .expect("SubjectAccessReview must reach a decision");
    assert!(
        !decision.is_allowed(),
        "RBAC must deny an unpermitted verb (delete pods)"
    );
}
