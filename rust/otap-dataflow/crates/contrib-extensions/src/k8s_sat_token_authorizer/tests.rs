// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the Kubernetes SAT authorizer extension.

use std::time::{Duration, Instant};

use otap_df_config::error::Error as ConfigError;
use otap_df_engine::capability::auth::{AuthzDecision, BearerToken, DenyReason};
use otap_df_engine::local::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as LocalBearerTokenAuthorizer;
use otap_df_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as SharedBearerTokenAuthorizer;

use super::authorizer::{LocalK8sSatTokenAuthorizer, SharedK8sSatTokenAuthorizer};
use super::cache::DecisionCache;
use super::config::{AudienceConfig, Config, ResourceAttributesConfig, normalize_service_account};
use super::core::Core;
use super::*;

// ── Config tests ───────────────────────────────────────────

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

/// Scenario: parse a minimal config with a single audience entry.
/// Guarantees: cache fields take their defaults and the entry defaults to an
/// empty allow-list and no RBAC (audience-only admission).
#[test]
fn config_defaults_apply() {
    let cfg = config_from_json(serde_json::json!({
        "audiences": [{ "audience": "my-service" }],
    }))
    .expect("minimal config is valid");
    assert_eq!(cfg.audiences.len(), 1);
    assert_eq!(cfg.audiences[0].audience, "my-service");
    assert!(cfg.audiences[0].allowed_service_accounts.is_empty());
    assert!(cfg.audiences[0].resource_attributes.is_none());
    assert_eq!(cfg.cache_ttl, Duration::from_secs(300));
    assert_eq!(cfg.cache_max_entries, 1024);
}

/// Scenario: parse configs that omit `audiences`, supply an empty list, or a
/// entry with a blank audience.
/// Guarantees: all are rejected, so a token is never admitted without an
/// audience-scoped entry.
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
        config_from_json(serde_json::json!({ "audiences": [{ "audience": "  " }] })).is_err(),
        "blank entry audience must be rejected"
    );
}

/// Scenario: parse a config with two entries sharing the same audience.
/// Guarantees: the duplicate is rejected, so admission for an audience is never
/// ambiguous.
#[test]
fn duplicate_audience_is_rejected() {
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [
                { "audience": "dup", "allowed_service_accounts": ["ns:a"] },
                { "audience": "dup", "allowed_service_accounts": ["ns:b"] },
            ],
        }))
        .is_err(),
        "duplicate duplicate audiences must be rejected"
    );
}

/// Scenario: parse configs with a zero `cache_ttl` and zero `cache_max_entries`.
/// Guarantees: each zero value is rejected so the extension never runs with a
/// degenerate cache.
#[test]
fn zero_valued_fields_are_rejected() {
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [{ "audience": "a" }],
            "cache_ttl": "0s",
        }))
        .is_err(),
        "zero cache_ttl must be rejected"
    );
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [{ "audience": "a" }],
            "cache_max_entries": 0,
        }))
        .is_err(),
        "zero cache_max_entries must be rejected"
    );
}

/// Scenario: parse configs carrying a field name the schema does not define, at
/// the top level and inside an entry.
/// Guarantees: an unknown field is rejected (deny_unknown_fields), catching
/// typos rather than silently ignoring them.
#[test]
fn unknown_field_is_rejected() {
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [{ "audience": "a" }],
            "typo": true,
        }))
        .is_err(),
        "unknown top-level field must be rejected"
    );
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [{ "audience": "a", "typo": true }],
        }))
        .is_err(),
        "unknown entry field must be rejected"
    );
}

/// Scenario: parse a human-readable duration for `cache_ttl`.
/// Guarantees: the duration deserializes to the exact wall-clock value.
#[test]
fn human_readable_durations_parse() {
    let cfg = config_from_json(serde_json::json!({
        "audiences": [{ "audience": "a" }],
        "cache_ttl": "90s",
    }))
    .expect("durations parse");
    assert_eq!(cfg.cache_ttl, Duration::from_secs(90));
}

/// Scenario: parse an entry whose `allowed_service_accounts` contains a
/// malformed entry (empty name).
/// Guarantees: validation fails at wiring time rather than silently never
/// matching the entry at request time.
#[test]
fn malformed_allow_list_entry_is_rejected() {
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [{ "audience": "a", "allowed_service_accounts": ["default:"] }],
        }))
        .is_err(),
        "allow-list entry with empty name must be rejected"
    );
}

// ── RBAC (SubjectAccessReview) config tests ────────────────

/// Scenario: parse an entry with a valid `resource_attributes` RBAC block.
/// Guarantees: the required `resource`/`verb` and optional fields deserialize.
#[test]
fn resource_attributes_parses() {
    let cfg = config_from_json(serde_json::json!({
        "audiences": [{
            "audience": "a",
            "resource_attributes": {
                "group": "telemetry.opentelemetry.io",
                "resource": "telemetry",
                "verb": "export",
                "namespace": "observability",
            },
        }],
    }))
    .expect("valid RBAC config parses");
    let ra = cfg.audiences[0]
        .resource_attributes
        .as_ref()
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
            "audiences": [{ "audience": "a", "resource_attributes": { "resource": "telemetry" } }],
        }))
        .is_err(),
        "missing verb must be rejected"
    );
    // Missing resource.
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [{ "audience": "a", "resource_attributes": { "verb": "export" } }],
        }))
        .is_err(),
        "missing resource must be rejected"
    );
    // Blank verb (validation).
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [{
                "audience": "a",
                "resource_attributes": { "resource": "telemetry", "verb": "  " },
            }],
        }))
        .is_err(),
        "blank verb must be rejected"
    );
}

/// Scenario: parse an entry that sets both `allowed_service_accounts` and
/// `resource_attributes`.
/// Guarantees: the two admission strategies are rejected together, so exactly
/// one admission model is active per entry.
#[test]
fn allow_list_and_rbac_are_mutually_exclusive() {
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [{
                "audience": "a",
                "allowed_service_accounts": ["default/reader"],
                "resource_attributes": { "resource": "telemetry", "verb": "export" },
            }],
        }))
        .is_err(),
        "allow-list and RBAC must be mutually exclusive within an entry"
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

/// Scenario: build the canonical allow-list set of an entry from mixed-shape
/// entries, and from an empty list.
/// Guarantees: the set contains the canonical usernames, and an empty list
/// yields `None` (admit any authenticated account for that audience).
#[test]
fn allowed_set_canonicalizes_and_empty_is_none() {
    let cfg = config_from_json(serde_json::json!({
        "audiences": [{
            "audience": "a",
            "allowed_service_accounts": [
                "default/reader",
                "system:serviceaccount:kube-system:writer",
            ],
        }],
    }))
    .expect("valid config");
    let set = cfg.audiences[0]
        .allowed_service_account_set()
        .expect("non-empty allow-list");
    assert!(set.contains("system:serviceaccount:default:reader"));
    assert!(set.contains("system:serviceaccount:kube-system:writer"));

    let cfg_empty =
        config_from_json(serde_json::json!({ "audiences": [{ "audience": "a" }] })).unwrap();
    assert!(
        cfg_empty.audiences[0]
            .allowed_service_account_set()
            .is_none()
    );
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
        .expect("extension advertises capabilities");
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
    assert!(validate_config(&serde_json::json!({ "audiences": [{ "audience": "svc" }] })).is_ok());
    assert!(validate_config(&serde_json::json!({})).is_err());
}

// ── Extension behavior tests ───────────────────────────────

// ── Admission tests (Core) ─────────────────────────────────

/// Builds a single-entry core for audience `my-service` with an optional
/// allow-list.
fn make_core(allowed: Option<Vec<&str>>) -> Core {
    let allowed_service_accounts = allowed
        .map(|v| v.into_iter().map(String::from).collect())
        .unwrap_or_default();
    Core::new(
        "test-authorizer",
        vec![AudienceConfig {
            audience: "my-service".to_string(),
            allowed_service_accounts,
            resource_attributes: None,
        }],
    )
}

/// Scenario: admit an authenticated identity when no allow-list is configured.
/// Guarantees: the request is allowed and the returned identity carries the
/// authenticated subject and the matched audience.
#[test]
fn admit_without_allow_list_allows_any_authenticated() {
    let core = make_core(None);
    let decision = core.admit_for_test(
        Some("system:serviceaccount:default:my-sa".to_string()),
        "my-service",
    );
    assert!(decision.is_allowed());
    let identity = decision.identity().expect("allow carries identity");
    assert_eq!(
        identity.subject(),
        Some("system:serviceaccount:default:my-sa")
    );
    assert_eq!(identity.audience(), Some("my-service"));
    // The identity is tagged with the scheme and a best-effort principal, and
    // the SA username is parsed into namespace / serviceaccount claims.
    assert_eq!(identity.scheme(), Some("k8s_sat"));
    assert_eq!(
        identity.principal(),
        Some("system:serviceaccount:default:my-sa")
    );
    assert_eq!(identity.claim_str("k8s.namespace"), Some("default"));
    assert_eq!(identity.claim_str("k8s.serviceaccount"), Some("my-sa"));
}

/// Scenario: admit a fully-populated authenticated user (username, uid, groups,
/// extra) and inspect the emitted identity.
/// Guarantees: every verified `TokenReview` attribute is surfaced as a claim a
/// downstream resolver can match -- `sub`/`principal`, `aud`, parsed
/// `k8s.namespace`/`k8s.serviceaccount`, `uid`, multi-valued `groups`, and
/// namespaced `extra.<key>` entries.
#[test]
fn allow_emits_full_verified_claims() {
    use super::reviewer::AuthenticatedUser;
    use std::collections::BTreeMap;

    let core = make_core(None);
    let mut extra = BTreeMap::new();
    let _ = extra.insert(
        "authentication.kubernetes.io/pod-name".to_string(),
        vec!["sender-abc".to_string()],
    );
    let user = AuthenticatedUser {
        username: Some("system:serviceaccount:team-a:sender".to_string()),
        uid: Some("uid-123".to_string()),
        groups: vec![
            "system:serviceaccounts".to_string(),
            "system:serviceaccounts:team-a".to_string(),
        ],
        extra,
        audiences: vec!["my-service".to_string()],
    };

    let decision = core.allow_for_test(&user, "my-service");
    let identity = decision.identity().expect("allow carries identity");

    assert_eq!(identity.scheme(), Some("k8s_sat"));
    assert_eq!(
        identity.principal(),
        Some("system:serviceaccount:team-a:sender")
    );
    assert_eq!(
        identity.subject(),
        Some("system:serviceaccount:team-a:sender")
    );
    assert_eq!(identity.audience(), Some("my-service"));
    assert_eq!(identity.claim_str("k8s.namespace"), Some("team-a"));
    assert_eq!(identity.claim_str("k8s.serviceaccount"), Some("sender"));
    assert_eq!(identity.claim_str("uid"), Some("uid-123"));

    let groups = identity.claim("groups").expect("groups claim present");
    assert!(groups.contains("system:serviceaccounts:team-a"));
    assert_eq!(groups.as_slice().len(), 2);

    let pod = identity
        .claim("extra.authentication.kubernetes.io/pod-name")
        .expect("extra claim present");
    assert!(pod.contains("sender-abc"));
}

/// Scenario: admit an identity that is present in the configured allow-list.
/// Guarantees: the matching service account is allowed.
#[test]
fn admit_allows_service_account_in_allow_list() {
    let core = make_core(Some(vec!["system:serviceaccount:default:my-sa"]));
    let decision = core.admit_for_test(
        Some("system:serviceaccount:default:my-sa".to_string()),
        "my-service",
    );
    assert!(decision.is_allowed());
}

/// Scenario: admit an authenticated identity that is absent from the allow-list,
/// and one with no username at all.
/// Guarantees: both are denied with `NotPermitted` (admission failure after
/// successful authentication).
#[test]
fn admit_denies_service_account_absent_from_allow_list() {
    let core = make_core(Some(vec!["system:serviceaccount:default:allowed"]));

    let denied = core.admit_for_test(
        Some("system:serviceaccount:default:other".to_string()),
        "my-service",
    );
    assert!(!denied.is_allowed());
    assert_eq!(
        denied,
        AuthzDecision::deny_with_detail(
            DenyReason::NotPermitted,
            "service account not in allow-list"
        )
    );

    let no_user = core.admit_for_test(None, "my-service");
    assert!(!no_user.is_allowed());
}

/// Scenario: admit against an audience that has no configured entry.
/// Guarantees: it is denied with `NotPermitted`, so a token authenticated for an
/// unbound audience is never admitted.
#[test]
fn admit_denies_unbound_audience() {
    let core = make_core(None);
    let decision = core.admit_for_test(
        Some("system:serviceaccount:default:my-sa".to_string()),
        "some-other-audience",
    );
    assert!(!decision.is_allowed());
}

/// Scenario: two tenants, each mapping its own audience to its own allow-list; a
/// service account allowed for tenant A is checked against tenant B's audience.
/// Guarantees: admission keys off the matched audience's entry, so tenant A's
/// SA is denied under tenant B's audience -- no cross-tenant admission.
#[test]
fn admit_is_scoped_per_audience() {
    let core = Core::new(
        "test-authorizer",
        vec![
            AudienceConfig {
                audience: "aud-tenant-a".to_string(),
                allowed_service_accounts: vec!["ns-a:sa-a".to_string()],
                resource_attributes: None,
            },
            AudienceConfig {
                audience: "aud-tenant-b".to_string(),
                allowed_service_accounts: vec!["ns-b:sa-b".to_string()],
                resource_attributes: None,
            },
        ],
    );

    let sa_a = "system:serviceaccount:ns-a:sa-a".to_string();
    // sa-a is admitted for its own tenant's audience...
    assert!(
        core.admit_for_test(Some(sa_a.clone()), "aud-tenant-a")
            .is_allowed()
    );
    // ...but denied when presented for tenant B's audience.
    assert!(
        !core.admit_for_test(Some(sa_a), "aud-tenant-b").is_allowed(),
        "a tenant's SA must not be admitted through another tenant's audience"
    );
}

/// Builds a two-audience core (tenant A + tenant B), each with its own
/// allow-list, for audience-matching tests.
fn two_audience_core() -> Core {
    Core::new(
        "test-authorizer",
        vec![
            AudienceConfig {
                audience: "aud-tenant-a".to_string(),
                allowed_service_accounts: vec!["ns-a:sa-a".to_string()],
                resource_attributes: None,
            },
            AudienceConfig {
                audience: "aud-tenant-b".to_string(),
                allowed_service_accounts: vec!["ns-b:sa-b".to_string()],
                resource_attributes: None,
            },
        ],
    )
}

/// Scenario: `TokenReview` confirms exactly one configured audience.
/// Guarantees: that audience is selected deterministically.
#[test]
fn match_audience_selects_single_bound() {
    let core = two_audience_core();
    assert_eq!(
        core.match_audience_for_test(&["aud-tenant-a".to_string()]),
        Ok("aud-tenant-a".to_string())
    );
}

/// Scenario: `TokenReview` confirms an audience that is not configured (plus
/// none that are).
/// Guarantees: it is denied as unbound, so an authenticated token for an
/// unconfigured audience is never admitted.
#[test]
fn match_audience_denies_unbound() {
    let core = two_audience_core();
    let result = core.match_audience_for_test(&["aud-unconfigured".to_string()]);
    assert!(matches!(
        result,
        Err(AuthzDecision::Deny {
            reason: DenyReason::NotPermitted,
            ..
        })
    ));
}

/// Scenario: a single token is confirmed for TWO configured audiences whose
/// policies differ (the multi-audience case; `status.audiences` order is
/// unspecified by Kubernetes).
/// Guarantees: admission fails closed with `NotPermitted` rather than
/// nondeterministically applying one tenant's policy -- preserving cross-tenant
/// isolation.
#[test]
fn match_audience_denies_ambiguous_multi_bound() {
    let core = two_audience_core();
    // Both orders must deny identically (no dependence on response ordering).
    for confirmed in [
        vec!["aud-tenant-a".to_string(), "aud-tenant-b".to_string()],
        vec!["aud-tenant-b".to_string(), "aud-tenant-a".to_string()],
    ] {
        let result = core.match_audience_for_test(&confirmed);
        assert!(
            matches!(
                result,
                Err(AuthzDecision::Deny {
                    reason: DenyReason::NotPermitted,
                    ..
                })
            ),
            "a token confirmed for two bound audiences must fail closed, got {result:?}"
        );
    }
}

/// Scenario: the confirmed-audience list repeats the same bound audience.
/// Guarantees: a duplicated audience is deduplicated, not mistaken for an
/// ambiguous multi-match, so a legitimate single-audience token is still
/// admitted.
#[test]
fn match_audience_dedups_repeated_audience() {
    let core = two_audience_core();
    assert_eq!(
        core.match_audience_for_test(&["aud-tenant-a".to_string(), "aud-tenant-a".to_string()]),
        Ok("aud-tenant-a".to_string())
    );
}

/// Scenario: authorize an empty credential through the shared variant.
/// Guarantees: it is denied with `MissingCredential` without contacting the API
/// server (the client is never initialized in this test).
#[tokio::test]
async fn authorize_empty_credential_is_missing_shared() {
    let ext = SharedK8sSatTokenAuthorizer::new(
        "test-authorizer",
        vec![AudienceConfig {
            audience: "my-service".to_string(),
            allowed_service_accounts: Vec::new(),
            resource_attributes: None,
        }],
        Duration::from_secs(300),
        1024,
    );
    let decision =
        SharedBearerTokenAuthorizer::authorize(&ext, &BearerToken::without_expiry(String::new()))
            .await
            .expect("empty credential yields a decision, not an error");
    assert_eq!(decision, AuthzDecision::deny(DenyReason::MissingCredential));
}

/// Scenario: authorize an empty credential through the local (lock-free)
/// variant.
/// Guarantees: the local variant is wired correctly and denies with
/// `MissingCredential` without contacting the API server.
#[tokio::test]
async fn authorize_empty_credential_is_missing_local() {
    let ext = LocalK8sSatTokenAuthorizer::new(
        "test-authorizer",
        vec![AudienceConfig {
            audience: "my-service".to_string(),
            allowed_service_accounts: Vec::new(),
            resource_attributes: None,
        }],
        Duration::from_secs(300),
        1024,
    );
    let decision =
        LocalBearerTokenAuthorizer::authorize(&ext, &BearerToken::without_expiry(String::new()))
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
    let mut cache = DecisionCache::new(Duration::from_secs(300), 1024);
    let now = Instant::now();
    let decision = AuthzDecision::allow_anonymous();
    cache.insert("tok", decision.clone(), now);

    assert_eq!(cache.get("tok", now), Some(decision));
    // Just past the 300s TTL the entry is gone.
    let later = now + Duration::from_secs(301);
    assert_eq!(cache.get("tok", later), None);
}

/// Scenario: insert more distinct tokens than the cache capacity allows, all
/// unexpired.
/// Guarantees: the cache never exceeds `max_entries`, bounding memory.
#[test]
fn cache_respects_max_entries() {
    let mut cache = DecisionCache::new(Duration::from_secs(300), 2);
    let now = Instant::now();
    for i in 0..10 {
        cache.insert(&format!("tok-{i}"), AuthzDecision::allow_anonymous(), now);
    }
    assert!(
        cache.len() <= 2,
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

/// Builds a single-entry config for the integration-test audience.
fn it_audience_entries(
    allowed: Vec<String>,
    resource_attributes: Option<ResourceAttributesConfig>,
) -> Vec<AudienceConfig> {
    vec![AudienceConfig {
        audience: it_audience(),
        allowed_service_accounts: allowed,
        resource_attributes,
    }]
}

fn it_extension(
    allowed: Vec<String>,
    resource_attributes: Option<ResourceAttributesConfig>,
) -> SharedK8sSatTokenAuthorizer {
    SharedK8sSatTokenAuthorizer::new(
        "it-authorizer",
        it_audience_entries(allowed, resource_attributes),
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
    let ext = it_extension(vec![], None);
    let decision = ext
        .authorize(&BearerToken::without_expiry(token))
        .await
        .expect("TokenReview must reach a decision");
    assert!(decision.is_allowed(), "valid token must be admitted");
    let identity = decision.identity().expect("allow carries identity");
    assert_eq!(
        identity.subject(),
        Some(it_subject().as_str()),
        "identity subject must be the authenticated service account"
    );
    // The real TokenReview response is mapped into verified claims: scheme,
    // parsed namespace/serviceaccount, and the group every SA belongs to.
    assert_eq!(identity.scheme(), Some("k8s_sat"));
    assert_eq!(
        identity.claim_str("k8s.namespace"),
        Some(it_namespace().as_str())
    );
    assert_eq!(identity.claim_str("k8s.serviceaccount"), Some("sat-tester"));
    let groups = identity.claim("groups").expect("groups claim present");
    assert!(
        groups.contains("system:serviceaccounts"),
        "every SA is a member of system:serviceaccounts, got {groups:?}"
    );
}

/// Scenario: a valid token is authorized through the local (lock-free) variant
/// against a live cluster.
/// Guarantees: the local `!Send` variant reaches the same admit decision as the
/// shared one, exercising the lock-free RefCell cache path end-to-end.
#[tokio::test]
#[ignore = "requires a live Kubernetes cluster and K8S_SAT_TOKEN"]
async fn it_local_variant_admits_valid_token() {
    let Some(token) = it_token() else {
        eprintln!("skipping: set K8S_SAT_TOKEN to run this integration test");
        return;
    };
    let ext = LocalK8sSatTokenAuthorizer::new(
        "it-local-authorizer",
        it_audience_entries(vec![], None),
        Duration::from_secs(300),
        1024,
    );
    let decision = LocalBearerTokenAuthorizer::authorize(&ext, &BearerToken::without_expiry(token))
        .await
        .expect("TokenReview must reach a decision");
    assert!(
        decision.is_allowed(),
        "valid token must be admitted by the local variant"
    );
    assert_eq!(
        decision.identity().and_then(|i| i.subject()),
        Some(it_subject().as_str()),
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
    let ext = it_extension(vec![], None);
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

    let ext = it_extension(vec![it_subject()], None);
    assert!(
        ext.authorize(&BearerToken::without_expiry(token.clone()))
            .await
            .expect("decision")
            .is_allowed(),
        "subject in the allow-list must be admitted"
    );

    let ext = it_extension(
        vec!["system:serviceaccount:sat-authz-test:someone-else".to_string()],
        None,
    );
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
    let ext = it_extension(vec![], Some(permitted));
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
    let ext = it_extension(vec![], Some(unpermitted));
    let decision = ext
        .authorize(&BearerToken::without_expiry(token))
        .await
        .expect("SubjectAccessReview must reach a decision");
    assert!(
        !decision.is_allowed(),
        "RBAC must deny an unpermitted verb (delete pods)"
    );
}

/// Scenario: two audience entries against a live cluster; a token minted for
/// audience A is presented while the service account is allow-listed only under
/// a *different* audience B.
/// Guarantees: admission keys off the audience `TokenReview` confirms (A), so the
/// token is admitted when A's entry allows it and the returned identity carries
/// audience A -- and it is denied when only B's entry lists the SA, proving one
/// tenant's identity cannot be admitted through another tenant's audience.
#[tokio::test]
#[ignore = "requires a live Kubernetes cluster and K8S_SAT_TOKEN"]
async fn it_multi_tenant_scopes_admission_to_matched_audience() {
    let Some(token) = it_token() else {
        eprintln!("skipping: set K8S_SAT_TOKEN to run this integration test");
        return;
    };
    let subject = it_subject();
    // A second audience the presented token is NOT valid for.
    let other_audience = "https://other-tenant.sat-authz-test.example".to_string();

    // Bind the token's audience (A) to an allow-list containing the SA, plus an
    // unrelated entry (B). The token is admitted through A and the identity
    // reports audience A.
    let ext = SharedK8sSatTokenAuthorizer::new(
        "it-multitenant",
        vec![
            AudienceConfig {
                audience: it_audience(),
                allowed_service_accounts: vec![subject.clone()],
                resource_attributes: None,
            },
            AudienceConfig {
                audience: other_audience.clone(),
                allowed_service_accounts: Vec::new(),
                resource_attributes: None,
            },
        ],
        Duration::from_secs(300),
        1024,
    );
    let decision = ext
        .authorize(&BearerToken::without_expiry(token.clone()))
        .await
        .expect("decision");
    assert!(
        decision.is_allowed(),
        "token must be admitted under its own audience"
    );
    assert_eq!(
        decision.identity().and_then(|i| i.audience()),
        Some(it_audience().as_str()),
        "identity must carry the matched (tenant) audience"
    );

    // Now allow-list the SA only under audience B; the token (valid only for A)
    // must be denied, since admission uses A's entry, not B's.
    let ext = SharedK8sSatTokenAuthorizer::new(
        "it-multitenant",
        vec![
            AudienceConfig {
                audience: it_audience(),
                allowed_service_accounts: vec![
                    "system:serviceaccount:sat-authz-test:someone-else".to_string(),
                ],
                resource_attributes: None,
            },
            AudienceConfig {
                audience: other_audience,
                allowed_service_accounts: vec![subject],
                resource_attributes: None,
            },
        ],
        Duration::from_secs(300),
        1024,
    );
    let decision = ext
        .authorize(&BearerToken::without_expiry(token))
        .await
        .expect("decision");
    assert!(
        !decision.is_allowed(),
        "a SA allow-listed only under another tenant's audience must not be admitted"
    );
}

/// The secondary audience used by the multi-audience ambiguity test; override
/// with `K8S_SAT_AUDIENCE_2`.
fn it_second_audience() -> String {
    std::env::var("K8S_SAT_AUDIENCE_2")
        .unwrap_or_else(|_| "https://sat-authz-test-2.example".to_string())
}

/// The multi-audience token (valid for both `it_audience()` and
/// `it_second_audience()`), or `None` when `K8S_SAT_MULTI_TOKEN` is unset.
fn it_multi_token() -> Option<String> {
    std::env::var("K8S_SAT_MULTI_TOKEN")
        .ok()
        .filter(|t| !t.trim().is_empty())
}

/// Scenario: a single token minted for TWO audiences is presented while both
/// audiences are configured entries (with differing policies) against a live
/// cluster.
/// Guarantees: `TokenReview` confirms both audiences, and admission fails closed
/// with a deny (ambiguous) rather than nondeterministically applying one
/// entry's policy -- the runtime side of cross-tenant isolation.
#[tokio::test]
#[ignore = "requires a live Kubernetes cluster and K8S_SAT_MULTI_TOKEN (a token minted for two audiences)"]
async fn it_multi_audience_token_is_denied_ambiguous() {
    let Some(token) = it_multi_token() else {
        eprintln!("skipping: set K8S_SAT_MULTI_TOKEN to run this integration test");
        return;
    };
    let subject = it_subject();

    // Both audiences allow-list the SA, so if a single policy were (wrongly)
    // selected the request would be admitted; the ambiguity guard must deny.
    let ext = SharedK8sSatTokenAuthorizer::new(
        "it-ambiguous",
        vec![
            AudienceConfig {
                audience: it_audience(),
                allowed_service_accounts: vec![subject.clone()],
                resource_attributes: None,
            },
            AudienceConfig {
                audience: it_second_audience(),
                allowed_service_accounts: vec![subject],
                resource_attributes: None,
            },
        ],
        Duration::from_secs(300),
        1024,
    );
    let decision = ext
        .authorize(&BearerToken::without_expiry(token))
        .await
        .expect("decision");
    assert!(
        !decision.is_allowed(),
        "a token confirmed for two configured audiences must fail closed (ambiguous)"
    );
}
