// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the Kubernetes service-account-token auth extension.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use k8s_openapi::api::authentication::v1::{TokenReviewStatus, UserInfo};
use k8s_openapi::api::authorization::v1::SubjectAccessReviewStatus;

use otap_df_config::error::Error as ConfigError;
use otap_df_engine::capability::auth::{
    AuthorizedIdentity, AuthzDecision, BearerToken, DenyReason,
};
use otap_df_engine::local::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as LocalBearerTokenAuthorizer;
use otap_df_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as SharedBearerTokenAuthorizer;

use super::authorizer::{LocalK8sServiceAccountTokenAuth, SharedK8sServiceAccountTokenAuth};
use super::cache::{Entries, LocalDecisionCache, SharedDecisionCache, SharedSlot, digest};
use super::config::{AudienceConfig, Config, ResourceAttributesConfig, normalize_service_account};
use super::core::Core;
use super::error::Error;
use super::reviewer::{
    AccessOutcome, AuthenticatedUser, KubeReviews, ReviewOutcome, access_outcome,
    access_review_request, review_outcome, token_review_request,
};
use super::*;

// -- Config tests -----------------------------------------------------------

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
    assert_eq!(cfg.review_timeout, Duration::from_secs(10));
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
        "duplicate audiences must be rejected"
    );
}

/// Scenario: parse configs with a zero cache TTL, entry cap, or review timeout.
/// Guarantees: each zero value is rejected so the extension never runs with a
/// degenerate cache or an immediately-expiring API request.
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
    assert!(
        config_from_json(serde_json::json!({
            "audiences": [{ "audience": "a" }],
            "review_timeout": "0s",
        }))
        .is_err(),
        "zero review_timeout must be rejected"
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

/// Scenario: parse human-readable cache and Kubernetes review durations.
/// Guarantees: both durations deserialize to their exact wall-clock values.
#[test]
fn human_readable_durations_parse() {
    let cfg = config_from_json(serde_json::json!({
        "audiences": [{ "audience": "a" }],
        "cache_ttl": "90s",
        "review_timeout": "750ms",
    }))
    .expect("durations parse");
    assert_eq!(cfg.cache_ttl, Duration::from_secs(90));
    assert_eq!(cfg.review_timeout, Duration::from_millis(750));
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

// -- RBAC (SubjectAccessReview) config tests --------------------------------

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

// -- Service-account normalization tests ------------------------------------

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

// -- Factory registration tests ---------------------------------------------

/// Scenario: inspect the linkme-registered factory entry.
/// Guarantees: the factory is registered under the documented URN and advertises
/// the `bearer_token_authorizer` capability on its shared variant.
#[test]
fn factory_is_registered_with_capability() {
    assert_eq!(
        K8S_SERVICE_ACCOUNT_TOKEN_AUTH_EXTENSION.name,
        K8S_SERVICE_ACCOUNT_TOKEN_AUTH_URN
    );
    let capabilities = K8S_SERVICE_ACCOUNT_TOKEN_AUTH_EXTENSION
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

/// Scenario: run the factory's `create` hook with a valid config, outside any
/// Kubernetes cluster.
/// Guarantees: it builds a passive bundle carrying BOTH the shared and local
/// variants, and does so without contacting Kubernetes -- confirming the client
/// is built lazily on first authorize rather than at pipeline construction, so
/// a collector still starts when the API server is briefly unreachable.
#[test]
fn factory_create_builds_both_variants_without_a_cluster() {
    let (ctx, _registry) = otap_df_engine::testing::test_extension_ctx();
    let user_config = Arc::new(ExtensionUserConfig::new(
        K8S_SERVICE_ACCOUNT_TOKEN_AUTH_URN.into(),
        serde_json::json!({
            "audiences": [{ "audience": "my-service" }],
            "cache_ttl": "1m",
        }),
    ));
    let extension_config = ExtensionConfig::new("k8s-authz");

    let bundle = create(&ctx, "k8s-authz".into(), user_config, &extension_config)
        .expect("a valid config must build an extension bundle");

    let shared = bundle.shared().expect("shared variant must be present");
    let local = bundle.local().expect("local variant must be present");
    assert!(
        shared.is_passive() && local.is_passive(),
        "both variants must be passive: the authorizer is driven by requests, not a control loop"
    );
}

/// Scenario: run the factory's `create` hook with a config that fails
/// validation.
/// Guarantees: the error surfaces as a `ConfigError` at construction time, so a
/// misconfigured pipeline fails to build instead of denying every request at
/// runtime.
#[test]
fn factory_create_rejects_an_invalid_config() {
    let (ctx, _registry) = otap_df_engine::testing::test_extension_ctx();
    let user_config = Arc::new(ExtensionUserConfig::new(
        K8S_SERVICE_ACCOUNT_TOKEN_AUTH_URN.into(),
        serde_json::json!({}),
    ));
    let extension_config = ExtensionConfig::new("k8s-authz");

    assert!(
        create(&ctx, "k8s-authz".into(), user_config, &extension_config).is_err(),
        "a config without audiences must be rejected at build time"
    );
}

// -- Extension behavior tests -----------------------------------------------

// -- Admission tests (Core) -------------------------------------------------

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

/// Scenario: audience-only admission (no allow-list, which admits any
/// authenticated caller) receives a `TokenReview` identity that authenticated
/// successfully but is not a service account -- e.g. an OIDC or webhook user,
/// which `TokenReview` also authenticates.
/// Guarantees: the identity is denied as an invalid credential rather than
/// admitted, so a non-service-account caller is never accepted by an
/// audience-only entry nor emitted under the `k8s_sat` scheme, including
/// usernames that merely wear the `system:serviceaccount:` prefix but carry a
/// name no real service account could have.
#[test]
fn admit_denies_non_service_account_identity() {
    let core = make_core(None);
    for username in [
        "alice@example.com",
        "system:node:worker-1",
        "system:serviceaccount:default:",
        "system:serviceaccount::my-sa",
        // A real SA name cannot contain a colon; these come from an
        // authenticator that does not police the `system:` prefix, and must not
        // yield an attacker-chosen `k8s.namespace` claim.
        "system:serviceaccount:tenant-a:sa:extra",
        "system:serviceaccount:tenant-a:sa:",
    ] {
        assert_eq!(
            core.admit_for_test(Some(username.to_string()), "my-service"),
            AuthzDecision::deny(DenyReason::InvalidCredential),
            "non-service-account username {username} must not be admitted"
        );
    }
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
/// Guarantees: that audience is returned as the sole bound audience.
#[test]
fn match_audience_selects_single_bound() {
    let core = two_audience_core();
    assert_eq!(
        core.match_audiences_for_test(&["aud-tenant-a".to_string()]),
        Ok(vec!["aud-tenant-a".to_string()])
    );
}

/// Scenario: `TokenReview` confirms an audience that is not configured (plus
/// none that are).
/// Guarantees: it is denied as unbound, so an authenticated token for an
/// unconfigured audience is never admitted.
#[test]
fn match_audience_denies_unbound() {
    let core = two_audience_core();
    let result = core.match_audiences_for_test(&["aud-unconfigured".to_string()]);
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
/// Guarantees: both bound audiences are returned in a stable, order-independent
/// (sorted) set, so admission can require every matched audience's policy and
/// the identity can list them all -- rather than picking one nondeterministically.
#[test]
fn match_audience_returns_all_bound_sorted() {
    let core = two_audience_core();
    // Both orders must yield the same sorted set (no dependence on response
    // ordering).
    for confirmed in [
        vec!["aud-tenant-a".to_string(), "aud-tenant-b".to_string()],
        vec!["aud-tenant-b".to_string(), "aud-tenant-a".to_string()],
    ] {
        assert_eq!(
            core.match_audiences_for_test(&confirmed),
            Ok(vec!["aud-tenant-a".to_string(), "aud-tenant-b".to_string()]),
            "a token confirmed for two bound audiences must return both, sorted"
        );
    }
}

/// Scenario: the confirmed-audience list repeats the same bound audience.
/// Guarantees: a duplicated audience is deduplicated, so a repeated audience in
/// the response is not counted twice.
#[test]
fn match_audience_dedups_repeated_audience() {
    let core = two_audience_core();
    assert_eq!(
        core.match_audiences_for_test(&["aud-tenant-a".to_string(), "aud-tenant-a".to_string()]),
        Ok(vec!["aud-tenant-a".to_string()])
    );
}

/// Scenario: a token is confirmed for TWO configured audiences, and the SA is
/// admitted by BOTH (each audience allow-lists it).
/// Guarantees: admission (AND across every matched audience) allows the request,
/// and the emitted identity carries a multi-valued `aud` listing both audiences
/// -- so a downstream resolver sees every audience the token was admitted for.
#[test]
fn admit_multi_audience_allows_and_lists_all_when_all_pass() {
    let sa = "system:serviceaccount:ns:shared".to_string();
    let core = Core::new(
        "test-authorizer",
        vec![
            AudienceConfig {
                audience: "aud-tenant-a".to_string(),
                allowed_service_accounts: vec![sa.clone()],
                resource_attributes: None,
            },
            AudienceConfig {
                audience: "aud-tenant-b".to_string(),
                allowed_service_accounts: vec![sa.clone()],
                resource_attributes: None,
            },
        ],
    );

    let decision = core.admit_multi_for_test(
        Some(sa),
        &["aud-tenant-b".to_string(), "aud-tenant-a".to_string()],
    );
    assert!(decision.is_allowed(), "both audiences admit -> allowed");

    let identity = decision.identity().expect("allow carries identity");
    let audience_claim = identity
        .claim(AuthorizedIdentity::CLAIM_AUDIENCE)
        .expect("identity carries an aud claim");
    let mut got: Vec<&str> = audience_claim
        .as_slice()
        .iter()
        .map(String::as_str)
        .collect();
    got.sort_unstable();
    assert_eq!(
        got,
        vec!["aud-tenant-a", "aud-tenant-b"],
        "the identity must list every audience the token was admitted for"
    );
    // A multi-valued aud is not a scalar, so the scalar accessor is empty.
    assert_eq!(identity.audience(), None);
}

/// Scenario: a token is confirmed for TWO configured audiences, but only ONE
/// admits the SA (the other's allow-list omits it).
/// Guarantees: admission fails closed with `NotPermitted` -- a laxer audience
/// can never let a token bypass a stricter audience's policy.
#[test]
fn admit_multi_audience_denies_when_any_fails() {
    let sa = "system:serviceaccount:ns:shared".to_string();
    let core = Core::new(
        "test-authorizer",
        vec![
            AudienceConfig {
                audience: "aud-tenant-a".to_string(),
                allowed_service_accounts: vec![sa.clone()],
                resource_attributes: None,
            },
            // Tenant B does NOT admit this SA.
            AudienceConfig {
                audience: "aud-tenant-b".to_string(),
                allowed_service_accounts: vec!["system:serviceaccount:ns:other".to_string()],
                resource_attributes: None,
            },
        ],
    );

    let decision = core.admit_multi_for_test(
        Some(sa),
        &["aud-tenant-a".to_string(), "aud-tenant-b".to_string()],
    );
    assert!(
        !decision.is_allowed(),
        "a token confirmed for an audience it is not admitted for must fail closed"
    );
    assert_eq!(
        decision,
        AuthzDecision::deny_with_detail(
            DenyReason::NotPermitted,
            "service account not in allow-list"
        )
    );
}

/// Scenario: authorize an empty credential through the shared variant.
/// Guarantees: it is denied with `MissingCredential` without contacting the API
/// server (the client is never initialized in this test).
#[tokio::test]
async fn authorize_empty_credential_is_missing_shared() {
    let ext = SharedK8sServiceAccountTokenAuth::new(
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
    let ext = LocalK8sServiceAccountTokenAuth::new(
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

// -- Decision cache tests ---------------------------------------------------

/// Stores `decision` in the slot `cache` hands out for `token`, mirroring what
/// `Core::authorize` does after reaching a decision.
fn set_slot(cache: &mut Entries<SharedSlot>, token: &str, decision: AuthzDecision, now: Instant) {
    let key = digest(token);
    let lease = cache.slot(key, now);
    lease
        .handle()
        .set(Ok(decision))
        .expect("slot must be empty");
    cache.complete(&key, lease.handle(), now);
}

/// Reads the decision currently cached for `token`, if any.
fn slot_value(cache: &mut Entries<SharedSlot>, token: &str, now: Instant) -> Option<AuthzDecision> {
    cache
        .live_cell(&digest(token), now)
        .and_then(|slot| slot.get())
        .and_then(|result| result.as_ref().ok())
        .cloned()
}

/// Scenario: store a decision in a token's slot and read it back before and
/// after its TTL elapses.
/// Guarantees: a fresh entry is returned; once expired it is treated as absent
/// (forcing a fresh TokenReview).
#[test]
fn cache_returns_fresh_and_drops_expired() {
    let mut cache = Entries::<SharedSlot>::new(Duration::from_secs(300), 1024);
    let now = Instant::now();
    let decision = AuthzDecision::allow_anonymous();
    set_slot(&mut cache, "tok", decision.clone(), now);

    assert_eq!(slot_value(&mut cache, "tok", now), Some(decision));
    // Just past the 300s TTL the entry is gone.
    let later = now + Duration::from_secs(301);
    assert_eq!(slot_value(&mut cache, "tok", later), None);
}

/// Scenario: two requests for the same token ask the cache for its slot while
/// the first decision is still in flight (the cell not yet initialized).
/// Guarantees: both receive the *same* cell, which is what lets concurrent
/// requests bearing one token collapse onto a single `TokenReview` instead of
/// stampeding the API server.
#[test]
fn cache_hands_concurrent_requests_the_same_slot() {
    let mut cache = Entries::<SharedSlot>::new(Duration::from_secs(300), 1024);
    let now = Instant::now();

    let first = cache.slot(digest("tok"), now);
    let second = cache.slot(digest("tok"), now);

    assert!(
        Arc::ptr_eq(first.handle(), second.handle()),
        "concurrent requests for one token must share a single decision slot"
    );
    assert_eq!(
        cache.len(),
        1,
        "sharing a slot must not duplicate the entry"
    );
}

/// Scenario: a token's entry expires and a later request asks for its slot.
/// Guarantees: a fresh, empty cell is handed out rather than the stale one, so
/// an expired decision is never resurrected, and the entry is replaced in place
/// rather than duplicated.
#[test]
fn cache_replaces_an_expired_slot() {
    let mut cache = Entries::<SharedSlot>::new(Duration::from_secs(300), 1024);
    let now = Instant::now();
    let key = digest("tok");
    let stale_lease = cache.slot(key, now);
    let stale = Arc::clone(stale_lease.handle());
    stale
        .set(Ok(AuthzDecision::allow_anonymous()))
        .expect("cell is empty");
    cache.complete(&key, &stale, now);

    let later = now + Duration::from_secs(301);
    let fresh = cache.slot(digest("tok"), later);

    assert!(
        !Arc::ptr_eq(&stale, fresh.handle()),
        "an expired entry must not hand back its stale cell"
    );
    assert!(
        fresh.handle().get().is_none(),
        "the replacement cell must be empty"
    );
    assert_eq!(cache.len(), 1, "replacing must not add a second entry");
}

/// Scenario: many concurrent tasks race to initialize one shared decision slot,
/// each attempting the (simulated) `TokenReview`.
/// Guarantees: the initializer runs exactly once and every task observes the
/// same decision, so one token costs one round-trip however many requests
/// arrive together.
#[tokio::test]
async fn slot_initialization_collapses_into_a_single_review() {
    let slot: SharedSlot = Arc::new(tokio::sync::OnceCell::new());
    let reviews = Arc::new(AtomicUsize::new(0));

    let mut tasks = Vec::new();
    for _ in 0..32 {
        let slot = Arc::clone(&slot);
        let reviews = Arc::clone(&reviews);
        tasks.push(tokio::spawn(async move {
            slot.get_or_init(|| async {
                let _ = reviews.fetch_add(1, Ordering::SeqCst);
                tokio::task::yield_now().await;
                Ok::<_, Error>(AuthzDecision::allow_anonymous())
            })
            .await
            .as_ref()
            .expect("init must succeed")
            .clone()
        }));
    }

    for task in tasks {
        let decision = task.await.expect("task must not panic");
        assert_eq!(decision, AuthzDecision::allow_anonymous());
    }
    assert_eq!(
        reviews.load(Ordering::SeqCst),
        1,
        "concurrent requests for one token must trigger exactly one TokenReview"
    );
}

/// Scenario: 32 parallel tasks call `SharedDecisionCache::get_or_decide` for
/// the same token while the cache is cold, so every task misses together.
/// Guarantees: the synchronized miss collapses onto exactly one `TokenReview`
/// and all callers observe the same allow, so a burst bearing one token -- or a
/// TTL expiry landing on many in-flight requests -- cannot stampede the API
/// server.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_authorize_for_one_token_performs_a_single_review() {
    let core = Arc::new(make_core(None));
    let cache = Arc::new(SharedDecisionCache::new(Duration::from_secs(300), 16));
    let reviews = Arc::new(AtomicUsize::new(0));

    let mut tasks = Vec::new();
    for _ in 0..32 {
        let core = Arc::clone(&core);
        let cache = Arc::clone(&cache);
        let reviews = Arc::clone(&reviews);
        tasks.push(tokio::spawn(async move {
            let credential = BearerToken::without_expiry("herd-token".to_string());
            let reviewer = FakeReviewer::authenticated(
                "system:serviceaccount:test-ns:test-sa",
                &["my-service"],
            );
            cache
                .get_or_decide(digest(credential.expose_token()), || async {
                    // Stands in for the TokenReview round-trip; the yield lets
                    // the other tasks race this one.
                    let _ = reviews.fetch_add(1, Ordering::SeqCst);
                    tokio::task::yield_now().await;
                    core.decide_with(&reviewer, credential.expose_token()).await
                })
                .await
                .expect("a decision must be reached")
        }));
    }

    for task in tasks {
        let decision = task.await.expect("task must not panic");
        assert!(
            matches!(decision, AuthzDecision::Allow { .. }),
            "every racing caller must observe the same allow, got {decision:?}"
        );
    }
    assert_eq!(
        reviews.load(Ordering::SeqCst),
        1,
        "a cold-cache burst for one token must cost exactly one TokenReview"
    );
}

/// Scenario: 32 concurrent shared-cache requests encounter one failed review,
/// followed by a later successful request for the same token.
/// Guarantees: current waiters share one failure, while the later request starts
/// exactly one new flight instead of retaining the undetermined result.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn shared_cache_coalesces_a_failure_and_allows_a_later_retry() {
    let cache = Arc::new(SharedDecisionCache::new(Duration::from_secs(300), 16));
    let reviews = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(tokio::sync::Notify::new());
    let key = digest("failed-flight");

    let mut tasks = Vec::new();
    for _ in 0..32 {
        let cache = Arc::clone(&cache);
        let reviews = Arc::clone(&reviews);
        let release = Arc::clone(&release);
        tasks.push(tokio::spawn(async move {
            cache
                .get_or_decide(key, || async {
                    let _ = reviews.fetch_add(1, Ordering::SeqCst);
                    release.notified().await;
                    Err(Error::MissingStatus)
                })
                .await
        }));
    }

    while cache.slot_ref_count(&key) < 33 {
        tokio::task::yield_now().await;
    }
    release.notify_waiters();

    for task in tasks {
        assert!(
            matches!(
                task.await.expect("task must not panic"),
                Err(Error::MissingStatus)
            ),
            "every waiter must observe the shared review failure"
        );
    }
    assert_eq!(
        reviews.load(Ordering::SeqCst),
        1,
        "concurrent failures must cost one review"
    );

    let decision = cache
        .get_or_decide(key, || async {
            let _ = reviews.fetch_add(1, Ordering::SeqCst);
            Ok(AuthzDecision::allow_anonymous())
        })
        .await
        .expect("a later request must retry");
    assert_eq!(decision, AuthzDecision::allow_anonymous());
    assert_eq!(
        reviews.load(Ordering::SeqCst),
        2,
        "the later request must start one fresh review"
    );
}

/// Scenario: 32 local tasks encounter one failed review, followed by a later
/// successful request for the same token.
/// Guarantees: interleaved waiters share one failure and the next request starts
/// one fresh flight, matching the shared cache's failure behavior.
#[tokio::test]
async fn local_cache_coalesces_a_failure_and_allows_a_later_retry() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let cache = Rc::new(LocalDecisionCache::new(Duration::from_secs(300), 16));
            let reviews = Rc::new(std::cell::Cell::new(0usize));
            let key = digest("failed-local-flight");

            let mut tasks = Vec::new();
            for _ in 0..32 {
                let cache = Rc::clone(&cache);
                let reviews = Rc::clone(&reviews);
                tasks.push(tokio::task::spawn_local(async move {
                    cache
                        .get_or_decide(key, || async {
                            reviews.set(reviews.get() + 1);
                            tokio::task::yield_now().await;
                            Err(Error::MissingStatus)
                        })
                        .await
                }));
            }

            for task in tasks {
                assert!(
                    matches!(
                        task.await.expect("task must not panic"),
                        Err(Error::MissingStatus)
                    ),
                    "every waiter must observe the shared review failure"
                );
            }
            assert_eq!(reviews.get(), 1, "concurrent failures must cost one review");

            let decision = cache
                .get_or_decide(key, || async {
                    reviews.set(reviews.get() + 1);
                    Ok(AuthzDecision::allow_anonymous())
                })
                .await
                .expect("a later request must retry");
            assert_eq!(decision, AuthzDecision::allow_anonymous());
            assert_eq!(
                reviews.get(),
                2,
                "the later request must start one fresh review"
            );
        })
        .await;
}

/// Scenario: an in-flight slot remains unresolved beyond the configured TTL.
/// Guarantees: another request receives the same slot because TTL starts only
/// after a decision is reached.
#[test]
fn cache_does_not_expire_an_in_flight_review() {
    let mut cache = Entries::<SharedSlot>::new(Duration::from_secs(1), 2);
    let now = Instant::now();
    let first = cache.slot(digest("tok"), now);
    let later = cache.slot(digest("tok"), now + Duration::from_secs(10));

    assert!(
        Arc::ptr_eq(first.handle(), later.handle()),
        "an in-flight review must remain shared beyond the decision TTL"
    );
}

/// Scenario: a decision completes long after its slot was inserted.
/// Guarantees: it receives the full configured TTL from completion rather than
/// expiring relative to the start of the review.
#[test]
fn cache_ttl_starts_when_the_decision_completes() {
    let mut cache = Entries::<SharedSlot>::new(Duration::from_secs(5), 2);
    let inserted = Instant::now();
    let completed = inserted + Duration::from_secs(10);
    let key = digest("tok");
    let lease = cache.slot(key, inserted);
    lease
        .handle()
        .set(Ok(AuthzDecision::allow_anonymous()))
        .expect("slot must be empty");
    cache.complete(&key, lease.handle(), completed);

    assert!(
        cache
            .live_cell(&key, completed + Duration::from_secs(4))
            .is_some(),
        "the decision must remain live for its full post-completion TTL"
    );
    assert!(
        cache
            .live_cell(&key, completed + Duration::from_secs(6))
            .is_none(),
        "the decision must expire after its post-completion TTL"
    );
}

/// Scenario: cache capacity is occupied by an in-flight review and a distinct
/// token arrives.
/// Guarantees: the active flight is retained and the new token receives an
/// untracked slot, preserving both the capacity bound and duplicate suppression
/// for the request already in progress.
#[test]
fn cache_never_evicts_an_in_flight_review() {
    let mut cache = Entries::<SharedSlot>::new(Duration::from_secs(300), 1);
    let now = Instant::now();
    let first = cache.slot(digest("first"), now);
    let second = cache.slot(digest("second"), now);
    let first_again = cache.slot(digest("first"), now);

    assert!(first.is_tracked(), "the first flight must be retained");
    assert!(
        !second.is_tracked(),
        "a distinct token must remain uncached while all capacity is in flight"
    );
    assert!(
        Arc::ptr_eq(first.handle(), first_again.handle()),
        "the active flight must remain available to matching requests"
    );
    assert_eq!(cache.len(), 1, "the cache must stay within its bound");
}

/// Scenario: the only shared caller awaiting a review is cancelled.
/// Guarantees: its abandoned slot is removed, so it cannot pin cache capacity
/// or force later decisions to remain uncached.
#[tokio::test]
async fn shared_cache_reclaims_an_abandoned_flight() {
    let cache = Arc::new(SharedDecisionCache::new(Duration::from_secs(300), 1));
    let started = Arc::new(tokio::sync::Notify::new());
    let task = {
        let cache = Arc::clone(&cache);
        let started = Arc::clone(&started);
        tokio::spawn(async move {
            cache
                .get_or_decide(digest("abandoned"), || async {
                    started.notify_one();
                    std::future::pending::<Result<AuthzDecision, Error>>().await
                })
                .await
        })
    };

    started.notified().await;
    task.abort();
    assert!(
        task.await.is_err(),
        "the abandoned request must be cancelled"
    );

    let decisions = AtomicUsize::new(0);
    for _ in 0..2 {
        let _ = cache
            .get_or_decide(digest("replacement"), || async {
                let _ = decisions.fetch_add(1, Ordering::SeqCst);
                Ok(AuthzDecision::allow_anonymous())
            })
            .await
            .expect("the replacement decision must succeed");
    }
    assert_eq!(
        decisions.load(Ordering::SeqCst),
        1,
        "reclaimed capacity must retain the replacement decision"
    );
}

/// Scenario: the only local task awaiting a review is cancelled.
/// Guarantees: its abandoned `Rc` slot is removed without atomic bookkeeping,
/// leaving capacity available for a later cached decision.
#[tokio::test]
async fn local_cache_reclaims_an_abandoned_flight() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let cache = Rc::new(LocalDecisionCache::new(Duration::from_secs(300), 1));
            let started = Rc::new(std::cell::Cell::new(false));
            let task = {
                let cache = Rc::clone(&cache);
                let started = Rc::clone(&started);
                tokio::task::spawn_local(async move {
                    cache
                        .get_or_decide(digest("abandoned"), || async {
                            started.set(true);
                            std::future::pending::<Result<AuthzDecision, Error>>().await
                        })
                        .await
                })
            };

            while !started.get() {
                tokio::task::yield_now().await;
            }
            task.abort();
            assert!(
                task.await.is_err(),
                "the abandoned request must be cancelled"
            );

            let decisions = std::cell::Cell::new(0usize);
            for _ in 0..2 {
                let _ = cache
                    .get_or_decide(digest("replacement"), || async {
                        decisions.set(decisions.get() + 1);
                        Ok(AuthzDecision::allow_anonymous())
                    })
                    .await
                    .expect("the replacement decision must succeed");
            }
            assert_eq!(
                decisions.get(),
                1,
                "reclaimed capacity must retain the replacement decision"
            );
        })
        .await;
}

/// Scenario: claim more distinct token slots than the cache capacity allows,
/// all unexpired.
/// Guarantees: the cache never exceeds `max_entries`, bounding memory, and the
/// most recent token is still cached -- a full cache evicts rather than
/// silently refusing to cache the new decision.
#[test]
fn cache_respects_max_entries() {
    let mut cache = Entries::<SharedSlot>::new(Duration::from_secs(300), 2);
    let now = Instant::now();
    for i in 0..10 {
        set_slot(
            &mut cache,
            &format!("tok-{i}"),
            AuthzDecision::allow_anonymous(),
            now,
        );
    }
    assert!(
        cache.len() <= 2,
        "cache must not exceed its max_entries bound"
    );
    assert!(
        slot_value(&mut cache, "tok-9", now).is_some(),
        "the most recent entry must be cached even when the cache was full"
    );
}

/// Scenario: an unauthenticated caller floods the cache with distinct junk
/// tokens (each a unique key) until it is full of live deny entries, then a
/// legitimate token is authorized.
/// Guarantees: the legitimate decision is still cached, so attacker-chosen
/// cache keys cannot pin the cache and force a `TokenReview` round-trip on
/// every subsequent legitimate request.
#[test]
fn cache_flood_does_not_starve_a_legitimate_entry() {
    let mut cache = Entries::<SharedSlot>::new(Duration::from_secs(300), 8);
    let now = Instant::now();
    for i in 0..64 {
        set_slot(
            &mut cache,
            &format!("junk-{i}"),
            AuthzDecision::deny(DenyReason::InvalidCredential),
            now,
        );
    }
    assert_eq!(cache.len(), 8, "the flood must not exceed the bound");

    let allow = AuthzDecision::allow_anonymous();
    set_slot(&mut cache, "legit", allow.clone(), now);
    assert_eq!(
        slot_value(&mut cache, "legit", now),
        Some(allow),
        "a legitimate decision must still be cacheable after a flood"
    );
}

/// Scenario: a cache configured with a zero entry cap hands out a slot.
/// Guarantees: the slot still works (the caller reaches a decision) but nothing
/// is retained, and the eviction loop terminates rather than spinning toward an
/// unreachable capacity.
#[test]
fn cache_with_zero_capacity_caches_nothing() {
    let mut cache = Entries::<SharedSlot>::new(Duration::from_secs(300), 0);
    let now = Instant::now();
    let slot = cache.slot(digest("tok"), now);
    slot.handle()
        .set(Ok(AuthzDecision::allow_anonymous()))
        .expect("the slot must still be usable");

    assert_eq!(cache.len(), 0, "a zero-capacity cache must stay empty");
    assert_eq!(slot_value(&mut cache, "tok", now), None);
}

/// Scenario: the shared variant's cache is asked twice for one token, counting
/// how many times it has to reach a decision.
/// Guarantees: the second request is served from cache without deciding again,
/// so a hit never reaches the API server. Covered separately from the local
/// cache, which implements its hit path independently.
#[tokio::test]
async fn shared_cache_serves_a_hit_without_deciding_again() {
    let cache = SharedDecisionCache::new(Duration::from_secs(300), 1024);
    let decision = AuthzDecision::allow_anonymous();
    let decisions = AtomicUsize::new(0);

    for attempt in 1..=2 {
        let served = cache
            .get_or_decide(digest("tok"), || async {
                let _ = decisions.fetch_add(1, Ordering::SeqCst);
                Ok::<_, Error>(decision.clone())
            })
            .await
            .expect("a decision must be reached");
        assert_eq!(
            served, decision,
            "attempt {attempt} must serve the decision"
        );
    }

    assert_eq!(
        decisions.load(Ordering::SeqCst),
        1,
        "a repeat request for one token must not decide again"
    );
}

/// Scenario: the local variant's cache is asked twice for one token, counting
/// how many times it has to reach a decision.
/// Guarantees: the second request is served from cache without deciding again,
/// so a hit never reaches the API server. Covered separately from the shared
/// cache, which implements its hit path independently.
#[tokio::test]
async fn local_cache_serves_a_hit_without_deciding_again() {
    let cache = LocalDecisionCache::new(Duration::from_secs(300), 1024);
    let decision = AuthzDecision::allow_anonymous();
    let decisions = std::cell::Cell::new(0usize);

    for attempt in 1..=2 {
        let served = cache
            .get_or_decide(digest("tok"), || async {
                decisions.set(decisions.get() + 1);
                Ok::<_, Error>(decision.clone())
            })
            .await
            .expect("a decision must be reached");
        assert_eq!(
            served, decision,
            "attempt {attempt} must serve the decision"
        );
    }

    assert_eq!(
        decisions.get(),
        1,
        "a repeat request for one token must not decide again"
    );
}

/// Scenario: 32 tasks interleave on a single thread, all calling
/// `LocalDecisionCache::get_or_decide` for the same token while the cache is
/// cold.
/// Guarantees: the burst collapses onto exactly one `TokenReview`. Tasks
/// interleave at the `.await` on the round-trip, so a single thread is not by
/// itself protection; the local variant holds the same guarantee as the shared
/// one.
#[tokio::test]
async fn interleaved_local_authorize_for_one_token_performs_a_single_review() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let core = Rc::new(make_core(None));
            let cache = Rc::new(LocalDecisionCache::new(Duration::from_secs(300), 16));
            let reviews = Rc::new(std::cell::Cell::new(0usize));

            let mut tasks = Vec::new();
            for _ in 0..32 {
                let core = Rc::clone(&core);
                let cache = Rc::clone(&cache);
                let reviews = Rc::clone(&reviews);
                tasks.push(tokio::task::spawn_local(async move {
                    let credential = BearerToken::without_expiry("herd-token".to_string());
                    let reviewer = FakeReviewer::authenticated(
                        "system:serviceaccount:test-ns:test-sa",
                        &["my-service"],
                    );
                    cache
                        .get_or_decide(digest(credential.expose_token()), || async {
                            reviews.set(reviews.get() + 1);
                            tokio::task::yield_now().await;
                            core.decide_with(&reviewer, credential.expose_token()).await
                        })
                        .await
                        .expect("a decision must be reached")
                }));
            }

            for task in tasks {
                let decision = task.await.expect("task must not panic");
                assert!(
                    matches!(decision, AuthzDecision::Allow { .. }),
                    "every interleaved caller must observe the same allow, got {decision:?}"
                );
            }
            assert_eq!(
                reviews.get(),
                1,
                "interleaved requests for one token must cost exactly one TokenReview"
            );
        })
        .await;
}

// -- TokenReview / SubjectAccessReview mapping tests -------------------------

/// Scenario: build the `TokenReview` submitted for a token.
/// Guarantees: the token is sent verbatim and the request always carries the
/// configured audiences, so the API server can only ever confirm an audience
/// this authorizer governs.
#[test]
fn token_review_request_carries_token_and_configured_audiences() {
    let audiences = vec!["aud-a".to_string(), "aud-b".to_string()];
    let request = token_review_request("the-token", &audiences);

    assert_eq!(request.spec.token, "the-token");
    assert_eq!(request.spec.audiences.as_deref(), Some(&audiences[..]));
}

/// Scenario: map an authenticated `TokenReviewStatus` carrying a full subject.
/// Guarantees: username, uid, groups, extra, and the confirmed audiences are
/// all preserved, since admission and the emitted identity are built from them.
#[test]
fn review_outcome_preserves_the_full_authenticated_subject() {
    let status = TokenReviewStatus {
        authenticated: Some(true),
        audiences: Some(vec!["aud-a".to_string()]),
        user: Some(UserInfo {
            username: Some("system:serviceaccount:ns:sa".to_string()),
            uid: Some("uid-1".to_string()),
            groups: Some(vec!["system:serviceaccounts".to_string()]),
            extra: Some(BTreeMap::from([(
                "authentication.kubernetes.io/pod-name".to_string(),
                vec!["pod-1".to_string()],
            )])),
        }),
        error: None,
    };

    match review_outcome(status) {
        ReviewOutcome::Authenticated(user) => {
            assert_eq!(
                user.username.as_deref(),
                Some("system:serviceaccount:ns:sa")
            );
            assert_eq!(user.uid.as_deref(), Some("uid-1"));
            assert_eq!(user.groups, vec!["system:serviceaccounts".to_string()]);
            assert_eq!(user.audiences, vec!["aud-a".to_string()]);
            assert_eq!(
                user.extra
                    .get("authentication.kubernetes.io/pod-name")
                    .map(Vec::as_slice),
                Some(&["pod-1".to_string()][..])
            );
        }
        ReviewOutcome::Unauthenticated { .. } => panic!("status was authenticated"),
    }
}

/// Scenario: map statuses that deny authentication outright, and one that omits
/// the `authenticated` flag entirely.
/// Guarantees: both are `Unauthenticated`, so a malformed or partial status
/// fails closed rather than being read as a successful authentication.
#[test]
fn review_outcome_fails_closed_without_an_explicit_authenticated_flag() {
    let denied = TokenReviewStatus {
        authenticated: Some(false),
        error: Some("token expired".to_string()),
        ..Default::default()
    };
    match review_outcome(denied) {
        ReviewOutcome::Unauthenticated { error } => {
            assert_eq!(error.as_deref(), Some("token expired"));
        }
        ReviewOutcome::Authenticated(_) => panic!("an explicit false must not authenticate"),
    }

    // No `authenticated` field at all: absence must not mean success.
    let absent = TokenReviewStatus::default();
    assert!(
        matches!(
            review_outcome(absent),
            ReviewOutcome::Unauthenticated { .. }
        ),
        "a status without an authenticated flag must fail closed"
    );
}

/// Scenario: build the `SubjectAccessReview` for an authenticated subject and a
/// configured resource-attribute policy.
/// Guarantees: the exact identity Kubernetes authenticated (user, uid, groups,
/// extra) and every configured resource attribute are forwarded, so RBAC is
/// evaluated against the real subject rather than a reconstructed one.
#[test]
fn access_review_request_forwards_subject_and_resource_attributes() {
    let user = AuthenticatedUser {
        username: Some("system:serviceaccount:ns:sa".to_string()),
        uid: Some("uid-1".to_string()),
        groups: vec!["system:serviceaccounts".to_string()],
        extra: BTreeMap::from([("k".to_string(), vec!["v".to_string()])]),
        audiences: vec!["aud-a".to_string()],
    };
    let attrs = ResourceAttributesConfig {
        group: Some("telemetry.io".to_string()),
        version: Some("v1".to_string()),
        resource: "telemetry".to_string(),
        verb: "export".to_string(),
        namespace: Some("tenant-a".to_string()),
        name: Some("stream".to_string()),
        subresource: Some("logs".to_string()),
    };

    let spec = access_review_request(&user, &attrs).spec;
    assert_eq!(spec.user.as_deref(), Some("system:serviceaccount:ns:sa"));
    assert_eq!(spec.uid.as_deref(), Some("uid-1"));
    assert_eq!(
        spec.groups.as_deref(),
        Some(&["system:serviceaccounts".to_string()][..])
    );
    assert_eq!(
        spec.extra.and_then(|e| e.get("k").cloned()),
        Some(vec!["v".to_string()])
    );

    let resource = spec
        .resource_attributes
        .expect("resource attributes are always sent");
    assert_eq!(resource.group.as_deref(), Some("telemetry.io"));
    assert_eq!(resource.version.as_deref(), Some("v1"));
    assert_eq!(resource.resource.as_deref(), Some("telemetry"));
    assert_eq!(resource.verb.as_deref(), Some("export"));
    assert_eq!(resource.namespace.as_deref(), Some("tenant-a"));
    assert_eq!(resource.name.as_deref(), Some("stream"));
    assert_eq!(resource.subresource.as_deref(), Some("logs"));
}

/// Scenario: map every combination of the `allowed` / `denied` flags a
/// `SubjectAccessReviewStatus` can carry.
/// Guarantees: only a plain `allowed` grants; an explicit `denied` overrides
/// `allowed`, and anything else denies -- so an ambiguous RBAC answer never
/// admits a caller.
#[test]
fn access_outcome_admits_only_on_an_unopposed_allow() {
    let allowed = SubjectAccessReviewStatus {
        allowed: true,
        ..Default::default()
    };
    assert!(matches!(access_outcome(allowed), AccessOutcome::Allowed));

    // An explicit deny wins over `allowed`.
    let contradictory = SubjectAccessReviewStatus {
        allowed: true,
        denied: Some(true),
        reason: Some("explicitly denied".to_string()),
        ..Default::default()
    };
    match access_outcome(contradictory) {
        AccessOutcome::Denied { reason } => {
            assert_eq!(reason.as_deref(), Some("explicitly denied"))
        }
        AccessOutcome::Allowed => panic!("an explicit denied must override allowed"),
    }

    let not_allowed = SubjectAccessReviewStatus {
        allowed: false,
        ..Default::default()
    };
    assert!(matches!(
        access_outcome(not_allowed),
        AccessOutcome::Denied { reason: None }
    ));
}

/// Scenario: map a status that carries no `reason` but does report an
/// evaluation error.
/// Guarantees: the evaluation error is surfaced as the deny reason, so an RBAC
/// misconfiguration is diagnosable from the decision alone.
#[test]
fn access_outcome_falls_back_to_the_evaluation_error() {
    let status = SubjectAccessReviewStatus {
        allowed: false,
        evaluation_error: Some("webhook unavailable".to_string()),
        ..Default::default()
    };
    match access_outcome(status) {
        AccessOutcome::Denied { reason } => {
            assert_eq!(reason.as_deref(), Some("webhook unavailable"));
        }
        AccessOutcome::Allowed => panic!("a failed evaluation must not admit"),
    }
}

// -- Decision flow tests (Core::decide against canned review responses) ------

/// A [`KubeReviews`] stand-in returning canned API-server responses, so the
/// real decision flow can be driven without a cluster.
struct FakeReviewer {
    outcome: ReviewOutcome,
    access: AccessOutcome,
}

impl FakeReviewer {
    /// A reviewer that authenticates `username` for `audiences` and, if RBAC is
    /// consulted, allows.
    fn authenticated(username: &str, audiences: &[&str]) -> Self {
        Self {
            outcome: ReviewOutcome::Authenticated(AuthenticatedUser {
                username: Some(username.to_string()),
                uid: Some("uid-1".to_string()),
                groups: vec!["system:serviceaccounts".to_string()],
                extra: BTreeMap::new(),
                audiences: audiences.iter().map(|a| (*a).to_string()).collect(),
            }),
            access: AccessOutcome::Allowed,
        }
    }

    fn with_access(mut self, access: AccessOutcome) -> Self {
        self.access = access;
        self
    }
}

impl KubeReviews for FakeReviewer {
    async fn review(&self, _token: &str) -> Result<ReviewOutcome, Error> {
        Ok(self.outcome.clone())
    }

    async fn check_access(
        &self,
        _user: &AuthenticatedUser,
        _attrs: &ResourceAttributesConfig,
    ) -> Result<AccessOutcome, Error> {
        Ok(self.access.clone())
    }
}

/// Scenario: the API server does not authenticate the token.
/// Guarantees: the decision is an `InvalidCredential` deny carrying the API
/// server's reason, and no admission policy is consulted.
#[tokio::test]
async fn decide_denies_an_unauthenticated_token() {
    let core = make_core(None);
    let reviewer = FakeReviewer {
        outcome: ReviewOutcome::Unauthenticated {
            error: Some("token expired".to_string()),
        },
        access: AccessOutcome::Allowed,
    };

    let decision = core.decide_with(&reviewer, "tok").await.expect("decision");
    assert!(!decision.is_allowed());
    assert_eq!(
        decision,
        AuthzDecision::deny_with_detail(DenyReason::InvalidCredential, "token expired")
    );
}

/// Scenario: `TokenReview` authenticates a non-service-account identity (an
/// OIDC or webhook user) for a configured audience whose entry admits any
/// authenticated caller.
/// Guarantees: the request is denied. This is the gate that stops a
/// non-Kubernetes identity from being admitted, and emitted, under the
/// `k8s_sat` scheme.
#[tokio::test]
async fn decide_denies_a_non_service_account_even_when_the_audience_admits_any() {
    let core = make_core(None);
    let reviewer = FakeReviewer::authenticated("alice@example.com", &["my-service"]);

    let decision = core.decide_with(&reviewer, "tok").await.expect("decision");
    assert_eq!(
        decision,
        AuthzDecision::deny(DenyReason::InvalidCredential),
        "only canonical service-account identities may be admitted"
    );
}

/// Scenario: a service account is authenticated for an audience the authorizer
/// does not govern.
/// Guarantees: the request is denied rather than admitted on the strength of
/// authentication alone.
#[tokio::test]
async fn decide_denies_a_token_bound_to_no_configured_audience() {
    let core = make_core(None);
    let reviewer =
        FakeReviewer::authenticated("system:serviceaccount:ns:sa", &["some-other-service"]);

    let decision = core.decide_with(&reviewer, "tok").await.expect("decision");
    assert!(!decision.is_allowed());
}

/// Scenario: an allow-listed service account is authenticated for the governed
/// audience.
/// Guarantees: the request is allowed and the emitted identity carries the
/// verified claims (scheme, principal, audience, and the parsed namespace and
/// service-account name) a downstream resolver matches on.
#[tokio::test]
async fn decide_allows_an_allow_listed_service_account_and_emits_its_claims() {
    let core = make_core(Some(vec!["ns/sa"]));
    let reviewer = FakeReviewer::authenticated("system:serviceaccount:ns:sa", &["my-service"]);

    let decision = core.decide_with(&reviewer, "tok").await.expect("decision");
    let identity = match decision {
        AuthzDecision::Allow { identity, .. } => identity,
        other => panic!("expected an allow, got {other:?}"),
    };
    assert_eq!(identity.scheme(), Some("k8s_sat"));
    assert_eq!(identity.subject(), Some("system:serviceaccount:ns:sa"));
    assert_eq!(identity.audience(), Some("my-service"));
    assert_eq!(
        identity.claim_str("k8s.namespace"),
        Some("ns"),
        "the namespace must be surfaced so a resolver need not re-parse the username"
    );
    assert_eq!(identity.claim_str("k8s.serviceaccount"), Some("sa"));
}

/// Scenario: a service account that is not on the audience's allow-list is
/// authenticated for that audience.
/// Guarantees: authentication alone does not admit; the allow-list still
/// governs, and the deny is `NotPermitted` rather than a credential error.
#[tokio::test]
async fn decide_denies_a_service_account_missing_from_the_allow_list() {
    let core = make_core(Some(vec!["ns/other"]));
    let reviewer = FakeReviewer::authenticated("system:serviceaccount:ns:sa", &["my-service"]);

    let decision = core.decide_with(&reviewer, "tok").await.expect("decision");
    assert!(!decision.is_allowed());
    assert!(
        matches!(decision, AuthzDecision::Deny { reason, .. } if reason == DenyReason::NotPermitted)
    );
}

/// Scenario: RBAC admission where the `SubjectAccessReview` allows, then denies.
/// Guarantees: the RBAC verdict decides the request, and an RBAC denial is
/// surfaced as `NotPermitted` with the API server's reason.
#[tokio::test]
async fn decide_honors_the_rbac_verdict() {
    let core = Core::new(
        "test-authorizer",
        vec![AudienceConfig {
            audience: "my-service".to_string(),
            allowed_service_accounts: Vec::new(),
            resource_attributes: Some(ResourceAttributesConfig {
                group: None,
                version: None,
                resource: "telemetry".to_string(),
                verb: "export".to_string(),
                namespace: None,
                name: None,
                subresource: None,
            }),
        }],
    );

    let allowed = FakeReviewer::authenticated("system:serviceaccount:ns:sa", &["my-service"]);
    assert!(
        core.decide_with(&allowed, "tok")
            .await
            .expect("decision")
            .is_allowed(),
        "an RBAC allow must admit"
    );

    let denied = FakeReviewer::authenticated("system:serviceaccount:ns:sa", &["my-service"])
        .with_access(AccessOutcome::Denied {
            reason: Some("no binding".to_string()),
        });
    assert_eq!(
        core.decide_with(&denied, "tok").await.expect("decision"),
        AuthzDecision::deny_with_detail(DenyReason::NotPermitted, "no binding"),
        "an RBAC deny must not admit"
    );
}

/// Scenario: a token confirmed for two governed audiences at once, where the
/// second audience's allow-list excludes the service account.
/// Guarantees: admission requires EVERY matched audience to admit, so carrying
/// a laxer audience cannot be used to bypass a stricter tenant's policy.
#[tokio::test]
async fn decide_requires_every_matched_audience_to_admit() {
    let core = Core::new(
        "test-authorizer",
        vec![
            // Permissive: admits any authenticated service account.
            AudienceConfig {
                audience: "lax".to_string(),
                allowed_service_accounts: Vec::new(),
                resource_attributes: None,
            },
            // Strict: admits only a different service account.
            AudienceConfig {
                audience: "strict".to_string(),
                allowed_service_accounts: vec!["ns/someone-else".to_string()],
                resource_attributes: None,
            },
        ],
    );
    let reviewer = FakeReviewer::authenticated("system:serviceaccount:ns:sa", &["lax", "strict"]);

    let decision = core.decide_with(&reviewer, "tok").await.expect("decision");
    assert!(
        !decision.is_allowed(),
        "the strict audience must veto, even though the lax one would admit"
    );
}

/// Scenario: a token confirmed for two governed audiences that both admit it.
/// Guarantees: the request is allowed and the emitted identity lists every
/// matched audience, so a downstream resolver sees exactly what was confirmed
/// rather than one arbitrarily chosen audience.
#[tokio::test]
async fn decide_lists_every_matched_audience_when_all_admit() {
    let core = Core::new(
        "test-authorizer",
        vec![
            AudienceConfig {
                audience: "aud-a".to_string(),
                allowed_service_accounts: Vec::new(),
                resource_attributes: None,
            },
            AudienceConfig {
                audience: "aud-b".to_string(),
                allowed_service_accounts: vec!["ns/sa".to_string()],
                resource_attributes: None,
            },
        ],
    );
    let reviewer = FakeReviewer::authenticated("system:serviceaccount:ns:sa", &["aud-b", "aud-a"]);

    let decision = core.decide_with(&reviewer, "tok").await.expect("decision");
    let identity = match decision {
        AuthzDecision::Allow { identity, .. } => identity,
        other => panic!("expected an allow, got {other:?}"),
    };
    let audiences = identity
        .claim(AuthorizedIdentity::CLAIM_AUDIENCE)
        .expect("the audience claim is always emitted");
    assert_eq!(
        audiences.as_slice(),
        &["aud-a".to_string(), "aud-b".to_string()][..],
        "every matched audience must be listed, in a stable order"
    );
}

// -- Live-cluster integration tests -----------------------------------------
//
// These are #[ignore]d: they require a reachable Kubernetes cluster (via the
// ambient kubeconfig/in-cluster config) and a valid projected service-account
// token supplied through the environment. Run them explicitly with, e.g.:
//
//   K8S_SAT_TOKEN="$(kubectl create token sat-tester -n sat-authz-test \
//     --audience=https://sat-authz-test.example)" \
//   cargo test -p otap-df-contrib-extensions \
//     --features k8s-service-account-token-auth-extension \
//     k8s_service_account_token_auth -- --ignored --nocapture
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
) -> SharedK8sServiceAccountTokenAuth {
    SharedK8sServiceAccountTokenAuth::new(
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
    let ext = LocalK8sServiceAccountTokenAuth::new(
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
    let ext = SharedK8sServiceAccountTokenAuth::new(
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
    let ext = SharedK8sServiceAccountTokenAuth::new(
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
/// audiences are configured entries that each admit the SA, against a live
/// cluster.
/// Guarantees: `TokenReview` confirms both audiences, every matched audience's
/// policy admits (AND), so the request is allowed and the emitted identity
/// lists both audiences in its multi-valued `aud` claim.
#[tokio::test]
#[ignore = "requires a live Kubernetes cluster and K8S_SAT_MULTI_TOKEN (a token minted for two audiences)"]
async fn it_multi_audience_token_admitted_when_all_pass() {
    let Some(token) = it_multi_token() else {
        eprintln!("skipping: set K8S_SAT_MULTI_TOKEN to run this integration test");
        return;
    };
    let subject = it_subject();

    // Both audiences allow-list the SA, so admission (AND across all matched
    // audiences) admits, and the identity must carry both audiences.
    let ext = SharedK8sServiceAccountTokenAuth::new(
        "it-multi-audience",
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
        decision.is_allowed(),
        "a token whose every confirmed audience admits must be allowed"
    );
    let identity = decision.identity().expect("allow carries identity");
    let audience_claim = identity
        .claim(AuthorizedIdentity::CLAIM_AUDIENCE)
        .expect("identity carries an aud claim");
    let mut got: Vec<&str> = audience_claim
        .as_slice()
        .iter()
        .map(String::as_str)
        .collect();
    got.sort_unstable();
    let mut want = [it_audience(), it_second_audience()];
    want.sort_unstable();
    assert_eq!(
        got,
        want.iter().map(String::as_str).collect::<Vec<_>>(),
        "the identity must list every audience the token was admitted for"
    );
}

/// Scenario: a single token minted for TWO audiences is presented, but only ONE
/// of the two configured audiences admits the SA (the other's allow-list omits
/// it), against a live cluster.
/// Guarantees: admission requires EVERY matched audience's policy to admit, so a
/// token that also carries an audience it is not admitted for is denied
/// (fail closed) -- one lax audience can never bypass a stricter one.
#[tokio::test]
#[ignore = "requires a live Kubernetes cluster and K8S_SAT_MULTI_TOKEN (a token minted for two audiences)"]
async fn it_multi_audience_token_denied_when_any_fails() {
    let Some(token) = it_multi_token() else {
        eprintln!("skipping: set K8S_SAT_MULTI_TOKEN to run this integration test");
        return;
    };
    let subject = it_subject();

    let ext = SharedK8sServiceAccountTokenAuth::new(
        "it-multi-audience-partial",
        vec![
            AudienceConfig {
                audience: it_audience(),
                allowed_service_accounts: vec![subject],
                resource_attributes: None,
            },
            // The second audience does NOT admit this SA.
            AudienceConfig {
                audience: it_second_audience(),
                allowed_service_accounts: vec![
                    "system:serviceaccount:sat-authz-test:someone-else".to_string(),
                ],
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
        "a token confirmed for an audience it is not admitted for must fail closed"
    );
}
