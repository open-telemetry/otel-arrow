// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit and integration tests for the Google Cloud (GCP) Auth extension.

use std::time::{Duration, Instant};

use futures::StreamExt;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider as SharedBearerTokenProvider;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
use tokio::sync::watch;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::auth::Auth;
use super::config::{Config, DEFAULT_METADATA_ENDPOINT, DEFAULT_SCOPE, DEFAULT_SERVICE_ACCOUNT};
use super::metrics::GcpAuthMetrics;
use super::*;
use crate::common::background_refresh::{
    BackgroundProviderMetricsTracker, BackgroundProviderSource,
};

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

fn make_tracker() -> BackgroundProviderMetricsTracker<GcpAuthMetrics> {
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<GcpAuthMetrics>(EmptyAttributes());
    BackgroundProviderMetricsTracker::new(metric_set)
}

fn make_extension(endpoint: &str) -> GcpAuthExtension {
    let cfg = config_from_json(serde_json::json!({
        "endpoint": endpoint,
    }))
    .expect("valid config");
    let auth = Auth::new(&cfg).expect("auth builds");
    let (tx, _rx) = watch::channel(None);
    GcpAuthExtension::new(
        "test-gcp-auth",
        auth,
        BackgroundProviderRefreshPolicy::expiry_driven(
            TOKEN_USABLE_MARGIN,
            NON_EXPIRING_TOKEN_REFRESH_INTERVAL,
            cfg.expiry_buffer,
        )
        .expect("valid refresh_policy"),
        tx,
        make_tracker(),
    )
}

async fn start_metadata_server(
    service_account: &str,
    expected_scopes: Option<&str>,
    access_token: &str,
    expires_in: u64,
) -> MockServer {
    let server = MockServer::start().await;
    let expected_path =
        format!("/computeMetadata/v1/instance/service-accounts/{service_account}/token");
    let mut mock = Mock::given(method("GET"))
        .and(path(expected_path))
        .and(header("Metadata-Flavor", "Google"));
    if let Some(scopes) = expected_scopes {
        mock = mock.and(query_param("scopes", scopes));
    }
    mock.respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": expires_in,
    })))
    .mount(&server)
    .await;
    server
}

/// Scenario: An empty JSON object or `null` configuration is deserialized.
/// Guarantees: Documented defaults (default service account, cloud-platform scope, 5m expiry
/// buffer, 1h fallback token lifetime, finite request/connect/startup timeouts, and default
/// metadata endpoint) are applied.
#[test]
fn config_defaults_apply_on_empty_or_null_config() {
    for val in [serde_json::json!({}), serde_json::Value::Null] {
        let cfg = config_from_json(val).expect("empty/null config is valid");
        assert!(cfg.endpoint.is_none());
        assert_eq!(cfg.service_account, DEFAULT_SERVICE_ACCOUNT);
        assert_eq!(cfg.scopes, vec![DEFAULT_SCOPE.to_string()]);
        assert_eq!(cfg.expiry_buffer, Duration::from_secs(300));
        assert_eq!(cfg.default_token_lifetime, Duration::from_secs(3600));
        assert_eq!(cfg.timeout, Duration::from_secs(10));
        assert_eq!(cfg.connect_timeout, Duration::from_secs(5));
        assert_eq!(cfg.startup_timeout, Duration::from_secs(30));
        assert_eq!(
            cfg.resolved_endpoint_with_env(None),
            DEFAULT_METADATA_ENDPOINT
        );
    }
}

/// Scenario: Metadata server endpoint resolution is evaluated with and without an explicit
/// `endpoint` config and `GCE_METADATA_HOST` environment value.
/// Guarantees: Explicit `endpoint` takes precedence over `GCE_METADATA_HOST`, bare hosts in
/// `GCE_METADATA_HOST` receive an `http://` prefix, trailing slashes are stripped, and empty or
/// whitespace-only overrides fall back to `DEFAULT_METADATA_ENDPOINT`.
#[test]
fn resolved_endpoint_honors_config_and_env_override_precedence() {
    let default_cfg = Config::default();
    assert_eq!(
        default_cfg.resolved_endpoint_with_env(None),
        DEFAULT_METADATA_ENDPOINT
    );
    assert_eq!(
        default_cfg.resolved_endpoint_with_env(Some("   ")),
        DEFAULT_METADATA_ENDPOINT
    );
    assert_eq!(
        default_cfg.resolved_endpoint_with_env(Some("169.254.169.254")),
        "http://169.254.169.254"
    );
    assert_eq!(
        default_cfg.resolved_endpoint_with_env(Some("http://169.254.169.254:8080/")),
        "http://169.254.169.254:8080"
    );
    assert_eq!(
        default_cfg.resolved_endpoint_with_env(Some("https://metadata.example.com/")),
        "https://metadata.example.com"
    );

    let explicit_cfg = config_from_json(serde_json::json!({
        "endpoint": "http://explicit.metadata.local/",
    }))
    .expect("valid config");
    assert_eq!(
        explicit_cfg.resolved_endpoint_with_env(Some("169.254.169.254")),
        "http://explicit.metadata.local"
    );
}

/// Scenario: Configurations with unknown fields, invalid endpoint URLs, path-traversal or
/// malformed service accounts, empty scopes, out-of-range expiry/lifetime bounds, or zero
/// timeouts are validated.
/// Guarantees: `validate_config` rejects each invalid input with `ConfigError::InvalidUserConfig`.
#[test]
fn config_rejects_invalid_values() {
    let cases = [
        serde_json::json!({ "unknown_field": true }),
        serde_json::json!({ "endpoint": "   " }),
        serde_json::json!({ "endpoint": "metadata.google.internal" }),
        serde_json::json!({ "endpoint": "http://" }),
        serde_json::json!({ "endpoint": "https://" }),
        serde_json::json!({ "endpoint": "http://metadata.google.internal?foo=1" }),
        serde_json::json!({ "endpoint": "http://metadata.google.internal#frag" }),
        serde_json::json!({ "endpoint": "http://user:pass@metadata.google.internal" }),
        serde_json::json!({ "service_account": "" }),
        serde_json::json!({ "service_account": "../other" }),
        serde_json::json!({ "service_account": "sa/with/slash" }),
        serde_json::json!({ "service_account": "sa\\backslash" }),
        serde_json::json!({ "service_account": "sa?query" }),
        serde_json::json!({ "service_account": "sa#fragment" }),
        serde_json::json!({ "service_account": "%2e%2e" }),
        serde_json::json!({ "service_account": "." }),
        serde_json::json!({ "service_account": ".." }),
        serde_json::json!({ "service_account": "sa with space" }),
        serde_json::json!({ "scopes": [""] }),
        serde_json::json!({ "expiry_buffer": "30s" }),
        serde_json::json!({ "expiry_buffer": "10m", "default_token_lifetime": "10m" }),
        serde_json::json!({ "expiry_buffer": "10m", "default_token_lifetime": "5m" }),
        serde_json::json!({ "timeout": "0s" }),
        serde_json::json!({ "connect_timeout": "0s" }),
        serde_json::json!({ "startup_timeout": "0s" }),
    ];

    for case in cases {
        let err = validate_config(&case).expect_err("invalid config must be rejected");
        assert!(
            matches!(err, ConfigError::InvalidUserConfig { .. }),
            "expected InvalidUserConfig for {case}, got {err:?}"
        );
    }
}

/// Scenario: A default-configured extension acquires a token from a mock GCE/GKE metadata server.
/// Guarantees: The request carries `Metadata-Flavor: Google` and the default `cloud-platform`
/// scope, and the acquired token is exposed both via `get_token()` and `token_stream()`.
#[tokio::test]
async fn acquires_token_with_metadata_flavor_header_and_default_scope() {
    let server =
        start_metadata_server("default", Some(DEFAULT_SCOPE), "gcp-test-token", 3599).await;
    let ext = make_extension(&server.uri());

    let token = ext.get_token().await.expect("token acquisition succeeds");
    assert_eq!(token.expose_token(), "gcp-test-token");

    let mut stream = ext.token_stream();
    let streamed = tokio::time::timeout(Duration::from_secs(1), stream.next())
        .await
        .expect("token stream yields immediately")
        .expect("stream is open");
    assert_eq!(streamed.expose_token(), "gcp-test-token");
}

/// Scenario: A custom service account email and multiple OAuth 2.0 scopes are configured.
/// Guarantees: `Auth::fetch` targets `/computeMetadata/v1/instance/service-accounts/{sa}/token`
/// and passes comma-joined scopes in the `scopes` query parameter.
#[tokio::test]
async fn custom_service_account_and_multiple_scopes_are_forwarded() {
    let server = start_metadata_server(
        "collector@my-proj.iam.gserviceaccount.com",
        Some("https://www.googleapis.com/auth/monitoring.write,https://www.googleapis.com/auth/logging.write"),
        "gcp-custom-sa-token",
        1800,
    )
    .await;

    let cfg = config_from_json(serde_json::json!({
        "endpoint": server.uri(),
        "service_account": "collector@my-proj.iam.gserviceaccount.com",
        "scopes": [
            "https://www.googleapis.com/auth/monitoring.write",
            "https://www.googleapis.com/auth/logging.write",
        ],
    }))
    .expect("valid config");
    let auth = Auth::new(&cfg).expect("auth builds");
    let token = auth.fetch().await.expect("acquisition succeeds");
    assert_eq!(token.expose_token(), "gcp-custom-sa-token");
}

/// Scenario: The metadata server returns `expires_in` as a JSON string and omits `token_type`.
/// Guarantees: Deserialization accepts the string expiry, defaults `token_type` to `Bearer`, and
/// sets `expires_on()` to the acquisition time plus the reported duration.
#[tokio::test]
async fn string_expires_in_and_missing_token_type_are_accepted() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .and(header("Metadata-Flavor", "Google"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "gcp-string-expiry-token",
            "expires_in": "120",
        })))
        .mount(&server)
        .await;

    let cfg = config_from_json(serde_json::json!({
        "endpoint": server.uri(),
    }))
    .expect("valid config");
    let auth = Auth::new(&cfg).expect("auth builds");
    let before = Instant::now();
    let token = auth.fetch().await.expect("acquisition succeeds");
    let after = Instant::now();
    assert_eq!(token.expose_token(), "gcp-string-expiry-token");
    let expires_on = token.expires_on().expect("expiry present");
    let expected = Duration::from_secs(120);
    assert!(
        expires_on >= before + expected && expires_on <= after + expected,
        "expected 120s expiry bounded by [before + 120s, after + 120s], got {expires_on:?}"
    );
}

/// Scenario: The metadata server returns `expires_in: 0` in one response and omits `expires_in`
/// in another.
/// Guarantees: An explicit `expires_in: 0` is rejected as an already-expired token, whereas an
/// omitted `expires_in` falls back to `config.default_token_lifetime`.
#[tokio::test]
async fn zero_expires_in_is_rejected_while_omitted_expires_in_uses_default_lifetime() {
    let zero_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .and(header("Metadata-Flavor", "Google"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "gcp-zero-expiry-token",
            "expires_in": 0,
        })))
        .mount(&zero_server)
        .await;

    let zero_cfg =
        config_from_json(serde_json::json!({ "endpoint": zero_server.uri() })).expect("valid");
    let zero_auth = Auth::new(&zero_cfg).expect("auth builds");
    let zero_err = zero_auth
        .fetch()
        .await
        .expect_err("expires_in: 0 must be rejected as already expired");
    assert!(
        zero_err.to_string().contains("expires_in"),
        "expected expires_in error, got {zero_err}"
    );

    let omitted_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .and(header("Metadata-Flavor", "Google"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "gcp-omitted-expiry-token",
        })))
        .mount(&omitted_server)
        .await;

    let omitted_cfg = config_from_json(serde_json::json!({
        "endpoint": omitted_server.uri(),
        "default_token_lifetime": "15m",
    }))
    .expect("valid");
    let omitted_auth = Auth::new(&omitted_cfg).expect("auth builds");
    let before = Instant::now();
    let token = omitted_auth
        .fetch()
        .await
        .expect("omitted expires_in falls back to default_token_lifetime");
    let after = Instant::now();
    let expires_on = token.expires_on().expect("expiry present");
    let expected = Duration::from_secs(15 * 60);
    assert!(
        expires_on >= before + expected && expires_on <= after + expected,
        "expected default_token_lifetime (15m), got {expires_on:?}"
    );
}

/// Scenario: The metadata server returns a non-Bearer `token_type`, an empty or whitespace-laden
/// `access_token`, an HTTP 403 with a large error body, an oversized 200 payload, or an HTTP 302
/// redirect.
/// Guarantees: `Auth::fetch` rejects each response with a bounded `Error::TokenAcquisition` and
/// does not follow HTTP redirects.
#[tokio::test]
async fn non_bearer_token_type_and_http_errors_are_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "gcp-mac-token",
            "token_type": "mac",
            "expires_in": 120,
        })))
        .mount(&server)
        .await;

    let cfg = config_from_json(serde_json::json!({ "endpoint": server.uri() })).expect("valid");
    let auth = Auth::new(&cfg).expect("auth builds");
    let err = auth
        .fetch()
        .await
        .expect_err("non-bearer token_type must be rejected");
    assert!(err.to_string().contains("unsupported token_type"));

    for bad_token in ["   ", "tok with space", "tok\nwith\nnewline"] {
        let bad_token_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/computeMetadata/v1/instance/service-accounts/default/token",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": bad_token,
                "expires_in": 120,
            })))
            .mount(&bad_token_server)
            .await;

        let bad_cfg = config_from_json(serde_json::json!({ "endpoint": bad_token_server.uri() }))
            .expect("valid");
        let bad_auth = Auth::new(&bad_cfg).expect("auth builds");
        let _ = bad_auth
            .fetch()
            .await
            .expect_err("empty or whitespace access_token must be rejected");
    }

    let failing_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(ResponseTemplate::new(403).set_body_string("x".repeat(2048)))
        .mount(&failing_server)
        .await;

    let failing_cfg =
        config_from_json(serde_json::json!({ "endpoint": failing_server.uri() })).expect("valid");
    let failing_auth = Auth::new(&failing_cfg).expect("auth builds");
    let http_err = failing_auth
        .fetch()
        .await
        .expect_err("HTTP 403 must fail")
        .to_string();
    assert!(http_err.contains("403"));
    assert!(http_err.contains("[truncated]"));
    assert!(
        http_err.len() < 512,
        "error message must remain bounded, got {} bytes",
        http_err.len()
    );

    let oversized_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string("x".repeat(128 * 1024)))
        .mount(&oversized_server)
        .await;

    let oversized_cfg =
        config_from_json(serde_json::json!({ "endpoint": oversized_server.uri() })).expect("valid");
    let oversized_auth = Auth::new(&oversized_cfg).expect("auth builds");
    let oversized_err = oversized_auth
        .fetch()
        .await
        .expect_err("oversized 200 response must fail")
        .to_string();
    assert!(
        oversized_err.contains("exceeds limit"),
        "expected size limit error, got {oversized_err}"
    );

    let redirect_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(ResponseTemplate::new(302).insert_header(
            "Location",
            format!(
                "{}/computeMetadata/v1/instance/service-accounts/default/token",
                server.uri()
            ),
        ))
        .mount(&redirect_server)
        .await;

    let redirect_cfg =
        config_from_json(serde_json::json!({ "endpoint": redirect_server.uri() })).expect("valid");
    let redirect_auth = Auth::new(&redirect_cfg).expect("auth builds");
    let redirect_err = redirect_auth
        .fetch()
        .await
        .expect_err("HTTP 302 redirect must not be followed")
        .to_string();
    assert!(
        redirect_err.contains("302"),
        "expected 302 status error, got {redirect_err}"
    );
}

/// Scenario: The static extension factory registration is inspected.
/// Guarantees: `GCP_AUTH_EXTENSION` is registered under `urn:otel:extension:gcp_auth` and
/// advertises `bearer_token_provider` as a shared capability.
#[test]
fn factory_is_registered_with_bearer_token_provider_capability() {
    assert_eq!(GCP_AUTH_EXTENSION.name, GCP_AUTH_URN);
    let caps = GCP_AUTH_EXTENSION
        .capabilities
        .as_ref()
        .expect("capabilities registered");
    assert!(
        caps.shared.contains(&"bearer_token_provider"),
        "BearerTokenProvider must be advertised as a shared capability"
    );
}

/// Scenario: The factory `create` hook is invoked with a default configuration.
/// Guarantees: Wiring produces an active shared extension bundle with no local variant.
#[test]
fn create_builds_a_shared_active_bundle() {
    let (ext_ctx, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
    let name: otel_arrow_dfe_config::ExtensionId = "gcp-auth".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        GCP_AUTH_URN.into(),
        serde_json::json!({}),
    ));
    let extension_config = ExtensionConfig::new(name.clone());
    let bundle = create(&ext_ctx, name, user_config, &extension_config)
        .expect("a default config wires successfully");
    assert!(bundle.local().is_none());
    let shared = bundle.shared().expect("a shared variant is produced");
    assert_eq!(shared.variant(), ExtensionVariant::Shared);
    assert!(!shared.is_passive());
}
