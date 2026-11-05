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
        assert_eq!(cfg.resolved_endpoint(), DEFAULT_METADATA_ENDPOINT);
    }
}

#[test]
fn config_rejects_invalid_values() {
    let cases = [
        serde_json::json!({ "unknown_field": true }),
        serde_json::json!({ "endpoint": "   " }),
        serde_json::json!({ "endpoint": "metadata.google.internal" }),
        serde_json::json!({ "service_account": "" }),
        serde_json::json!({ "service_account": "../other" }),
        serde_json::json!({ "service_account": "sa/with/slash" }),
        serde_json::json!({ "scopes": [""] }),
        serde_json::json!({ "expiry_buffer": "30s" }),
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
    let token = auth.fetch().await.expect("acquisition succeeds");
    assert_eq!(token.expose_token(), "gcp-string-expiry-token");
    let remaining = token
        .expires_on()
        .expect("expiry present")
        .duration_since(Instant::now());
    assert!(
        remaining > Duration::from_secs(110) && remaining <= Duration::from_secs(120),
        "expected ~120s expiry, got {remaining:?}"
    );
}

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
}

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
