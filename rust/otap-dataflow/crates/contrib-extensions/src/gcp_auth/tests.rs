// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit and integration tests for the Google Cloud (GCP) Auth extension.

use std::time::Duration;

use futures::{FutureExt, StreamExt};
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider as SharedBearerTokenProvider;
use otel_arrow_dfe_engine::shared::extension::{ControlChannel, Extension as SharedExtension};
use otel_arrow_dfe_engine::shared::message::SharedReceiver;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
use tokio::sync::watch;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::auth::Auth;
use super::config::{Config, DEFAULT_METADATA_ENDPOINT, DEFAULT_SCOPE};
use super::metrics::GcpAuthMetrics;
use super::*;
use crate::common::background_refresh::{
    BackgroundFetchOutcome, BackgroundProviderMetricsTracker, BackgroundProviderSource,
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
        BackgroundProviderRefreshPolicy::periodic(TOKEN_CACHE_POLL_INTERVAL)
            .expect("valid refresh_policy"),
        tx,
        make_tracker(),
    )
}

async fn start_metadata_server(
    expected_scopes: Option<&str>,
    access_token: &str,
    expires_in: u64,
) -> MockServer {
    let server = MockServer::start().await;
    let mut mock = Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
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
/// Guarantees: Documented defaults (`cloud-platform` scope, `30s` startup timeout, and `None`
/// endpoint override) are applied.
#[test]
fn config_defaults_apply_on_empty_or_null_config() {
    for val in [serde_json::json!({}), serde_json::Value::Null] {
        let cfg = config_from_json(val).expect("empty/null config is valid");
        assert!(cfg.endpoint.is_none());
        assert_eq!(cfg.scopes, vec![DEFAULT_SCOPE.to_string()]);
        assert_eq!(cfg.timeout, Duration::from_secs(10));
        assert_eq!(cfg.startup_timeout, Duration::from_secs(30));
        assert_eq!(cfg.normalized_endpoint().unwrap(), None);
    }
}

/// Scenario: `Config::normalized_endpoint` is evaluated with valid URLs containing trailing slashes.
/// Guarantees: Trailing slashes are stripped without stripping the `http://` or `https://` scheme
/// prefix.
#[test]
fn normalized_endpoint_strips_trailing_slashes_and_preserves_scheme() {
    let cfg = config_from_json(serde_json::json!({
        "endpoint": "https://metadata.example.internal:8080///",
    }))
    .expect("valid config");
    assert_eq!(
        cfg.normalized_endpoint().unwrap().as_deref(),
        Some("https://metadata.example.internal:8080")
    );
    let upper_cfg = config_from_json(serde_json::json!({
        "endpoint": "HTTP://metadata.example.internal:8080//",
    }))
    .expect("valid uppercase scheme config");
    assert_eq!(
        upper_cfg.normalized_endpoint().unwrap().as_deref(),
        Some("HTTP://metadata.example.internal:8080")
    );
    assert_eq!(
        config::trim_endpoint_trailing_slashes("http://"),
        "http://",
        "scheme prefix must not be stripped by trailing-slash trimmer"
    );
    assert_eq!(
        config::trim_endpoint_trailing_slashes("https://"),
        "https://",
        "scheme prefix must not be stripped by trailing-slash trimmer"
    );
}

/// Scenario: Unknown configuration fields are supplied in the JSON config.
/// Guarantees: `#[serde(deny_unknown_fields)]` rejects the configuration at parse time.
#[test]
fn config_rejects_unknown_fields() {
    let err = config_from_json(serde_json::json!({
        "unknown_field": "nope",
    }))
    .expect_err("unknown fields must be rejected");
    assert!(err.to_string().contains("unknown_field"));
}

/// Scenario: Invalid `endpoint`, `scopes`, and `startup_timeout` values are validated via
/// `validate_config`.
/// Guarantees: Each invalid field fails fast with a descriptive `ConfigError::InvalidUserConfig`
/// message.
#[test]
fn config_validation_rejects_invalid_values() {
    let cases = [
        (serde_json::json!({ "endpoint": "   " }), "`endpoint`"),
        (
            serde_json::json!({ "endpoint": "metadata.google.internal" }),
            "must start with `http://` or `https://`",
        ),
        (
            serde_json::json!({ "endpoint": "http://" }),
            "non-empty host",
        ),
        (
            serde_json::json!({ "endpoint": "https://" }),
            "non-empty host",
        ),
        (
            serde_json::json!({ "endpoint": "http:///" }),
            "non-empty host",
        ),
        (
            serde_json::json!({ "endpoint": "http://:8080" }),
            "non-empty host",
        ),
        (
            serde_json::json!({ "endpoint": "http://[::1" }),
            "invalid `endpoint` URL",
        ),
        (
            serde_json::json!({ "endpoint": "http://user:pass@127.0.0.1:8080" }),
            "must not include userinfo credentials",
        ),
        (
            serde_json::json!({ "endpoint": "http://127.0.0.1:8080?foo=bar" }),
            "must not include query parameters or a URL fragment",
        ),
        (
            serde_json::json!({ "endpoint": "http://127.0.0.1:8080#frag" }),
            "must not include query parameters or a URL fragment",
        ),
        (serde_json::json!({ "scopes": ["  "] }), "`scopes`"),
        (serde_json::json!({ "timeout": "0s" }), "`timeout`"),
        (
            serde_json::json!({ "startup_timeout": "0s" }),
            "`startup_timeout`",
        ),
    ];

    for (raw, expected_substr) in cases {
        let err = validate_config(&raw)
            .expect_err(&format!("expected {raw} to fail"))
            .to_string();
        assert!(
            err.contains(expected_substr),
            "expected error for {raw} to mention {expected_substr}, got: {err}"
        );
    }
}

/// Scenario: A token is requested from a mock GCE metadata server using default configuration.
/// Guarantees: The request carries `Metadata-Flavor: Google` and the default `cloud-platform`
/// scope, and the acquired token is exposed both via `get_token()` and `token_stream()`.
#[tokio::test]
async fn acquires_token_with_metadata_flavor_header_and_default_scope() {
    let server = start_metadata_server(Some(DEFAULT_SCOPE), "gcp-test-token", 3599).await;
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

/// Scenario: `endpoint` is omitted (`None`) so Application Default Credentials (ADC) discovery runs
/// against `GOOGLE_APPLICATION_CREDENTIALS` for both a valid `authorized_user` file and a missing
/// file path, and is compared against an explicit `endpoint` override.
/// Guarantees: `Auth` exchanges the ADC refresh token with the configured scopes and caches the
/// resulting token, surfaces a `BuildCredentials` error when `GOOGLE_APPLICATION_CREDENTIALS`
/// points to a missing file, and bypasses ADC file discovery when `endpoint` is explicitly set.
#[test]
fn adc_credentials_acquire_and_cache_token_when_endpoint_is_omitted() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("current-thread runtime builds");

    let oauth_server = rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(header("content-type", "application/json"))
            .and(body_json(serde_json::json!({
                "grant_type": "refresh_token",
                "client_id": "adc-test-client-id",
                "client_secret": "adc-test-client-secret",
                "refresh_token": "adc-test-refresh-token",
                "scopes": DEFAULT_SCOPE,
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "adc-authorized-user-token",
                "token_type": "Bearer",
                "expires_in": 3600,
            })))
            .mount(&server)
            .await;
        server
    });

    let adc_file = tempfile::NamedTempFile::new().expect("temp ADC file created");
    std::fs::write(
        adc_file.path(),
        serde_json::json!({
            "type": "authorized_user",
            "client_id": "adc-test-client-id",
            "client_secret": "adc-test-client-secret",
            "refresh_token": "adc-test-refresh-token",
            "token_uri": format!("{}/token", oauth_server.uri()),
        })
        .to_string(),
    )
    .expect("ADC JSON written");

    temp_env::with_var(
        "GOOGLE_APPLICATION_CREDENTIALS",
        Some(adc_file.path()),
        || {
            rt.block_on(async {
                let cfg = config_from_json(serde_json::json!({})).expect("default ADC config");
                assert!(cfg.endpoint.is_none());
                let auth = Auth::new(&cfg).expect("ADC auth builds");

                let (tx, _rx) = watch::channel(None);
                let ext = GcpAuthExtension::new(
                    "test-gcp-adc-auth",
                    auth.clone(),
                    BackgroundProviderRefreshPolicy::periodic(TOKEN_CACHE_POLL_INTERVAL)
                        .expect("valid refresh_policy"),
                    tx,
                    make_tracker(),
                );

                let token = ext
                    .get_token()
                    .await
                    .expect("ADC token acquisition succeeds");
                assert_eq!(token.expose_token(), "adc-authorized-user-token");

                let outcome_2 = auth
                    .fetch_outcome()
                    .await
                    .expect("cached ADC fetch_outcome succeeds");
                assert!(matches!(outcome_2, BackgroundFetchOutcome::Unchanged(_)));
                assert_eq!(
                    outcome_2.into_value().expose_token(),
                    "adc-authorized-user-token"
                );

                let requests = oauth_server
                    .received_requests()
                    .await
                    .expect("requests recorded");
                assert_eq!(
                    requests.len(),
                    1,
                    "cached ADC credentials must not re-fetch on NotModified"
                );
            });
        },
    );

    let missing_path = adc_file.path().with_file_name("nonexistent-adc-creds.json");
    temp_env::with_var(
        "GOOGLE_APPLICATION_CREDENTIALS",
        Some(&missing_path),
        || {
            rt.block_on(async {
                let adc_cfg = config_from_json(serde_json::json!({})).expect("default ADC config");
                let adc_auth = Auth::new(&adc_cfg).expect("Auth::new defers ADC loading");
                let err = adc_auth
                    .fetch()
                    .await
                    .expect_err("missing GOOGLE_APPLICATION_CREDENTIALS file must fail");
                assert!(
                    err.to_string().contains("GOOGLE_APPLICATION_CREDENTIALS"),
                    "expected error mentioning GOOGLE_APPLICATION_CREDENTIALS, got: {err}"
                );

                let mds_server =
                    start_metadata_server(Some(DEFAULT_SCOPE), "mds-override-token", 3600).await;
                let mds_cfg = config_from_json(serde_json::json!({
                    "endpoint": mds_server.uri(),
                }))
                .expect("explicit endpoint config");
                let mds_auth = Auth::new(&mds_cfg).expect("explicit endpoint auth builds");
                let mds_token = mds_auth
                    .fetch()
                    .await
                    .expect("explicit endpoint bypasses GOOGLE_APPLICATION_CREDENTIALS");
                assert_eq!(mds_token.expose_token(), "mds-override-token");
            });
        },
    );
}

/// Scenario: Repeated `get_token()` and `fetch_outcome()` calls are made while the cached token is
/// valid.
/// Guarantees: `get_token()` Across cloned extension handles hits the shared watch cache, and
/// `Auth::fetch_outcome()` returns `BackgroundFetchOutcome::Unchanged` on subsequent calls without
/// hitting the metadata server again.
#[tokio::test]
async fn get_token_and_fetch_outcome_reuse_cached_token_without_refetching() {
    let server = start_metadata_server(Some(DEFAULT_SCOPE), "gcp-cached-token", 3599).await;
    let consumer_a = make_extension(&server.uri());
    let consumer_b = consumer_a.clone();

    let first = consumer_a.get_token().await.expect("first acquisition");
    assert_eq!(first.expose_token(), "gcp-cached-token");

    let second = consumer_b.get_token().await.expect("cached acquisition");
    assert_eq!(second.expose_token(), "gcp-cached-token");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        requests.len(),
        1,
        "clones must share one cache and not re-fetch on the fast path"
    );

    let cfg = config_from_json(serde_json::json!({ "endpoint": server.uri() })).expect("valid");
    let auth = Auth::new(&cfg).expect("auth builds");
    let outcome_1 = auth.fetch_outcome().await.expect("first fetch_outcome");
    assert!(matches!(outcome_1, BackgroundFetchOutcome::Updated(_)));
    let outcome_2 = auth.fetch_outcome().await.expect("second fetch_outcome");
    assert!(matches!(outcome_2, BackgroundFetchOutcome::Unchanged(_)));
    assert_eq!(outcome_2.into_value().expose_token(), "gcp-cached-token");
}

/// Scenario: The active extension runs its periodic poll loop across an unchanged tick and a
/// token rotation tick.
/// Guarantees: `NotModified` ticks do not emit duplicate tokens on `token_stream()`, and once
/// `google-cloud-auth` rotates the token ahead of expiry, the rotated token is published to
/// subscribers.
#[tokio::test]
async fn background_poll_loop_publishes_rotated_token_without_duplicate_emissions() {
    let server = MockServer::start().await;
    // First token expires in 300s (so google-cloud-auth refreshes at 300s - 240s = 60s).
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .and(header("Metadata-Flavor", "Google"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "gcp-token-v1",
            "token_type": "Bearer",
            "expires_in": 300,
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .and(header("Metadata-Flavor", "Google"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "gcp-token-v2",
            "token_type": "Bearer",
            "expires_in": 3600,
        })))
        .mount(&server)
        .await;

    let ext = make_extension(&server.uri());
    let mut stream = ext.token_stream();

    let (ctrl_tx, ctrl_rx) = tokio::sync::mpsc::channel(1);
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let ctrl = ControlChannel::new(SharedReceiver::mpsc(ctrl_rx), shutdown_rx);
    let (effects, probe) =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
            "test-gcp-auth".into(),
            Duration::from_secs(30),
        );
    let task = tokio::spawn(Box::new(ext.clone()).start(ctrl, effects));

    probe.wait_ready().await.expect("initial token ready");
    assert_eq!(
        stream
            .next()
            .await
            .expect("initial token emitted")
            .expose_token(),
        "gcp-token-v1"
    );

    // Pause time after the initial real loopback HTTP exchange completes so the
    // in-memory NotModified poll ticks can be stepped deterministically.
    tokio::time::pause();
    // Advance 15s, 30s, and 45s: google-cloud-auth has not rotated yet (rotates at 60s), so no
    // duplicate emission occurs on token_stream.
    for _ in 0..3 {
        tokio::time::advance(TOKEN_CACHE_POLL_INTERVAL).await;
        tokio::task::yield_now().await;
        assert!(stream.next().now_or_never().is_none());
    }

    // Advance past 60s so google-cloud-auth wakes to refresh in the background, resume
    // wall-clock time for the real loopback TCP exchange with wiremock, and then step
    // periodic poll ticks in a bounded loop until the rotated token is published.
    tokio::time::advance(Duration::from_secs(16)).await;
    tokio::time::resume();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if server.received_requests().await.unwrap().len() >= 2 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("google-cloud-auth background refresh request sent");

    let mut rotated = None;
    for _ in 0..100 {
        tokio::time::pause();
        tokio::time::advance(TOKEN_CACHE_POLL_INTERVAL).await;
        tokio::task::yield_now().await;
        tokio::time::resume();
        if let Some(Some(token)) = stream.next().now_or_never() {
            rotated = Some(token);
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let rotated = rotated.expect("rotated token published after google-cloud-auth refresh");
    assert_eq!(rotated.expose_token(), "gcp-token-v2");

    drop(ctrl_tx);
    let _ = task.await.expect("join").expect("clean shutdown");
}

/// Scenario: `get_token()` is called twice against a metadata server that returns HTTP 500.
/// Guarantees: The error maps to a `CapabilityError` for `bearer_token_provider` with this
/// extension's name, and the negative cache throttles the immediate retry so the server is hit
/// only once.
#[tokio::test]
async fn get_token_error_maps_to_capability_error_and_throttles_retries() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let ext = make_extension(&server.uri());

    let err = ext.get_token().await.expect_err("first attempt fails");
    assert_eq!(err.capability, "bearer_token_provider");
    assert_eq!(err.extension, "test-gcp-auth");

    let _ = ext
        .get_token()
        .await
        .expect_err("second attempt is throttled by the negative cache");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        requests.len(),
        1,
        "recent failure must throttle the next slow-path fetch"
    );
}

/// Scenario: Multiple OAuth 2.0 scopes are configured in one case, and an empty `scopes: []` list
/// is configured in another.
/// Guarantees: Multiple scopes are passed comma-joined in the `scopes` query parameter, and an
/// empty `scopes` list omits the `scopes` query parameter entirely.
#[tokio::test]
async fn multiple_scopes_and_empty_scopes_are_handled_correctly() {
    let multi_server = start_metadata_server(
        Some("https://www.googleapis.com/auth/monitoring.write,https://www.googleapis.com/auth/logging.write"),
        "gcp-multi-scope-token",
        1800,
    )
    .await;

    let multi_cfg = config_from_json(serde_json::json!({
        "endpoint": format!("{}///", multi_server.uri()),
        "scopes": [
            "  https://www.googleapis.com/auth/monitoring.write  ",
            "https://www.googleapis.com/auth/logging.write",
        ],
    }))
    .expect("valid config");
    let multi_auth = Auth::new(&multi_cfg).expect("auth builds");
    let token = multi_auth.fetch().await.expect("acquisition succeeds");
    assert_eq!(token.expose_token(), "gcp-multi-scope-token");

    let empty_server = start_metadata_server(None, "gcp-no-scope-token", 1800).await;
    let empty_cfg = config_from_json(serde_json::json!({
        "endpoint": empty_server.uri(),
        "scopes": [],
    }))
    .expect("valid config");
    let empty_auth = Auth::new(&empty_cfg).expect("auth builds");
    let empty_token = empty_auth.fetch().await.expect("acquisition succeeds");
    assert_eq!(empty_token.expose_token(), "gcp-no-scope-token");

    let requests = empty_server
        .received_requests()
        .await
        .expect("requests recorded");
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0].url.query().is_none(),
        "empty scopes must not append a query string, got {:?}",
        requests[0].url.query()
    );
}

/// Scenario: The metadata server returns a non-Bearer `token_type`, an empty or whitespace-laden
/// `access_token`, an HTTP 403 error, or malformed JSON, and then recovers on a subsequent call.
/// Guarantees: `Auth::fetch` rejects each invalid response with `Error::TokenAcquisition`, resets
/// its cached credentials on error, and succeeds once the metadata server returns a valid token.
#[tokio::test]
async fn invalid_token_responses_and_http_errors_are_rejected_and_recover() {
    for bad_body in [
        serde_json::json!({
            "access_token": "gcp-mac-token",
            "token_type": "mac",
            "expires_in": 120,
        }),
        serde_json::json!({
            "access_token": "   ",
            "token_type": "Bearer",
            "expires_in": 120,
        }),
        serde_json::json!({
            "access_token": "tok with space",
            "token_type": "Bearer",
            "expires_in": 120,
        }),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/computeMetadata/v1/instance/service-accounts/default/token",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(bad_body))
            .mount(&server)
            .await;

        let cfg = config_from_json(serde_json::json!({ "endpoint": server.uri() })).expect("valid");
        let auth = Auth::new(&cfg).expect("auth builds");
        let bad_err = auth
            .fetch()
            .await
            .expect_err("invalid token_type or access_token must be rejected");
        assert!(
            bad_err
                .to_string()
                .contains("invalid Bearer Authorization header from google-cloud-auth"),
            "unexpected error for bad token body: {bad_err}"
        );
    }

    let recovery_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(ResponseTemplate::new(403).set_body_string("permission denied"))
        .up_to_n_times(1)
        .mount(&recovery_server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string("{not-json"))
        .up_to_n_times(1)
        .mount(&recovery_server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "gcp-mac-token",
            "token_type": "mac",
            "expires_in": 3600,
        })))
        .up_to_n_times(1)
        .mount(&recovery_server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "recovered-token",
            "token_type": "Bearer",
            "expires_in": 3600,
        })))
        .mount(&recovery_server)
        .await;

    let cfg = config_from_json(serde_json::json!({
        "endpoint": recovery_server.uri(),
    }))
    .expect("valid");
    let auth = Auth::new(&cfg).expect("auth builds");
    let err_403 = auth.fetch().await.expect_err("HTTP 403 must fail");
    auth.log_refresh_failure(&err_403);
    assert!(
        err_403.to_string().contains("403")
            && err_403.to_string().contains("failed to fetch")
            && err_403.to_string().contains("permission denied"),
        "expected full causal error chain for 403, got: {err_403}"
    );

    let json_err = auth.fetch().await.expect_err("malformed JSON must fail");
    assert!(
        json_err.to_string().contains("failed to fetch"),
        "expected malformed JSON acquisition error, got: {json_err}"
    );
    let mac_err = auth
        .fetch()
        .await
        .expect_err("non-Bearer token_type must fail and reset cached TokenCache");
    assert!(
        mac_err
            .to_string()
            .contains("invalid Bearer Authorization header"),
        "expected invalid Bearer error, got: {mac_err}"
    );

    let recovered = auth
        .fetch()
        .await
        .expect("Auth resets dead TokenCache after error and recovers");
    assert_eq!(recovered.expose_token(), "recovered-token");
    auth.shutdown_in_flight().await;
    let shutdown_err = auth
        .fetch()
        .await
        .expect_err("fetch after shutdown_in_flight must fail");
    assert!(
        shutdown_err.to_string().contains("shut down"),
        "expected shut down error, got: {shutdown_err}"
    );

    let timeout_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/computeMetadata/v1/instance/service-accounts/default/token",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(5))
                .set_body_json(serde_json::json!({
                    "access_token": "slow-token",
                    "token_type": "Bearer",
                    "expires_in": 3600,
                })),
        )
        .mount(&timeout_server)
        .await;
    let timeout_cfg = config_from_json(serde_json::json!({
        "endpoint": timeout_server.uri(),
        "timeout": "50ms",
    }))
    .expect("valid");
    let timeout_auth = Auth::new(&timeout_cfg).expect("auth builds");
    let timeout_err = timeout_auth
        .fetch()
        .await
        .expect_err("slow response must time out");
    assert!(
        timeout_err.to_string().contains("timed out"),
        "expected timeout error, got {timeout_err}"
    );
}

/// Scenario: Acquisition successes, failures, and token publishes are recorded on `GcpAuthMetrics`.
/// Guarantees: Every metric counter and latency instrument updates and flushes through
/// `MetricsReporter`.
#[test]
fn metrics_tracker_records_snapshots_and_reports() {
    let mut tracker = make_tracker();

    let before = tracker.snapshot();
    assert!(
        before.get_metrics().iter().all(|m| m.is_zero()),
        "a new tracker starts at zero"
    );

    tracker.record_success(8.5);
    tracker.record_failure();
    tracker.record_publish();

    let after = tracker.snapshot();
    assert!(
        after.get_metrics().iter().all(|m| !m.is_zero()),
        "every metric is non-zero after recording"
    );

    let (rx, mut reporter) =
        otel_arrow_dfe_telemetry::reporter::MetricsReporter::create_new_and_receiver(4);
    tracker.report(&mut reporter).expect("report succeeds");
    assert!(
        rx.try_recv().is_ok(),
        "reporter received the metric snapshot"
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

/// Scenario: The factory `create` hook and `Auth::new` are invoked with valid and invalid
/// configurations outside an async runtime.
/// Guarantees: Wiring with a valid configuration produces an active shared extension bundle with
/// no local variant without panicking outside a Tokio runtime, and invalid configurations fail fast
/// with `ConfigError::InvalidUserConfig` or `Error::BuildCredentials`.
#[test]
fn create_builds_a_shared_active_bundle() {
    let invalid_auth_cfg = Config {
        endpoint: Some("not-a-url".to_string()),
        ..Config::default()
    };
    let build_err = Auth::new(&invalid_auth_cfg)
        .err()
        .expect("invalid endpoint must fail Auth::new");
    assert!(
        build_err
            .to_string()
            .contains("failed to build the GCP metadata credential"),
        "expected BuildCredentials error, got {build_err}"
    );

    let empty_scope_cfg = Config {
        scopes: vec!["   ".to_string()],
        ..Config::default()
    };
    let scope_err = Auth::new(&empty_scope_cfg)
        .err()
        .expect("empty scope must fail Auth::new");
    assert!(scope_err.to_string().contains("`scopes`"));

    let (ext_ctx, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
    let name: otel_arrow_dfe_config::ExtensionId = "gcp-auth".into();
    let extension_config = ExtensionConfig::new(name.clone());

    let bad_user_config = Arc::new(ExtensionUserConfig::new(
        GCP_AUTH_URN.into(),
        serde_json::json!({ "startup_timeout": "0s" }),
    ));
    let create_err = create(&ext_ctx, name.clone(), bad_user_config, &extension_config)
        .err()
        .expect("invalid config must fail create()");
    assert!(matches!(create_err, ConfigError::InvalidUserConfig { .. }));

    let user_config = Arc::new(ExtensionUserConfig::new(
        GCP_AUTH_URN.into(),
        serde_json::json!({ "endpoint": DEFAULT_METADATA_ENDPOINT }),
    ));
    let bundle = create(&ext_ctx, name, user_config, &extension_config)
        .expect("a valid config wires successfully");
    assert!(bundle.local().is_none());
    let shared = bundle.shared().expect("a shared variant is produced");
    assert_eq!(shared.variant(), ExtensionVariant::Shared);
    assert!(!shared.is_passive());
}
