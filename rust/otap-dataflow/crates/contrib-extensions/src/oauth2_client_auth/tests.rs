// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit and integration tests for the OAuth 2.0 Client Auth extension.

use std::time::Duration;

use futures::StreamExt;
use otap_df_config::error::Error as ConfigError;
use otap_df_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider as SharedBearerTokenProvider;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::testing::EmptyAttributes;
use tokio::sync::watch;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::auth::Auth;
use super::config::{Config, GrantType, SignatureAlgorithm};
use super::metrics::OAuth2ClientAuthMetrics;
use super::*;
use crate::common::token_refresh::TokenProviderMetricsTracker;

// -- Helpers ---------------------------------------------------

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

/// A minimal valid config pointing at `token_url` with inline credentials.
fn valid_config_json(token_url: &str) -> serde_json::Value {
    serde_json::json!({
        "token_url": token_url,
        "client_id": "id",
        "client_secret": "secret",
    })
}

fn make_tracker() -> TokenProviderMetricsTracker<OAuth2ClientAuthMetrics> {
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<OAuth2ClientAuthMetrics>(EmptyAttributes());
    TokenProviderMetricsTracker::new(metric_set)
}

fn make_extension(token_url: &str) -> OAuth2ClientAuthExtension {
    let cfg = config_from_json(valid_config_json(token_url)).expect("valid config");
    let auth = Auth::new(&cfg).expect("auth builds");
    let (tx, _rx) = watch::channel(None);
    OAuth2ClientAuthExtension::new("test-ext", auth, cfg.expiry_buffer, tx, make_tracker())
}

/// Starts a mock token endpoint at `/token` returning the given access token
/// and relative expiry.
async fn start_token_server(access_token: &str, expires_in: u64) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": access_token,
            "token_type": "Bearer",
            "expires_in": expires_in,
        })))
        .mount(&server)
        .await;
    server
}

/// Starts a mock token endpoint that always fails with HTTP 500.
async fn start_failing_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    server
}

/// Generates a throwaway 2048-bit RSA keypair (private + public PEM) at runtime,
/// so no key material is ever committed to the repository.
fn generate_test_rsa_keypair() -> (String, String) {
    let key_pair =
        rcgen::KeyPair::generate_rsa_for(&rcgen::PKCS_RSA_SHA256, rcgen::RsaKeySize::_2048)
            .expect("generate test RSA key pair");
    (key_pair.serialize_pem(), key_pair.public_key_pem())
}

// -- Config tests ----------------------------------------------

// Scenario: A minimal valid config is deserialized with all optional fields omitted.
// Guarantees: Documented defaults (client_credentials grant, 5m buffer, 30s startup, empty
// scopes/params, no timeout/tls) are applied so operators can rely on them.
#[test]
fn config_defaults_apply() {
    let cfg =
        config_from_json(valid_config_json("https://idp.example.com/token")).expect("valid config");
    assert_eq!(cfg.grant_type, GrantType::ClientCredentials);
    assert_eq!(cfg.expiry_buffer, Duration::from_secs(300));
    assert_eq!(cfg.startup_timeout, Duration::from_secs(30));
    assert!(cfg.scopes.is_empty());
    assert!(cfg.endpoint_params.is_empty());
    assert!(cfg.timeout.is_none());
    assert!(cfg.tls.is_none());
}

// Scenario: A config omits the required `token_url`.
// Guarantees: Deserialization fails, so a pipeline cannot start without a token endpoint.
#[test]
fn token_url_is_required() {
    let err = config_from_json(serde_json::json!({ "client_id": "id", "client_secret": "s" }))
        .expect_err("missing token_url must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A config supplies a whitespace-only `token_url`.
// Guarantees: Validation rejects it rather than building a client against an empty endpoint.
#[test]
fn empty_token_url_is_rejected() {
    let err = config_from_json(serde_json::json!({
        "token_url": "   ",
        "client_id": "id",
        "client_secret": "s",
    }))
    .expect_err("empty token_url must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A config provides neither `client_id` nor `client_id_file`.
// Guarantees: Validation rejects it, since every grant needs a client identifier.
#[test]
fn missing_client_id_is_rejected() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_secret": "s",
    }))
    .expect_err("missing client identifier must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A client_credentials config provides neither `client_secret` nor `client_secret_file`.
// Guarantees: Validation rejects it, since the client-credentials grant authenticates with a secret.
#[test]
fn missing_client_secret_is_rejected_for_client_credentials() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
    }))
    .expect_err("missing client secret must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: The client identifier and secret are supplied only via their `*_file` forms.
// Guarantees: Validation accepts the file forms, enabling credential rotation without inline values.
#[test]
fn file_forms_satisfy_credential_requirements() {
    let cfg = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id_file": "/etc/secrets/client_id",
        "client_secret_file": "/etc/secrets/client_secret",
    }))
    .expect("file-based credentials are valid");
    assert!(cfg.client_id.is_none());
    assert!(cfg.client_id_file.is_some());
    assert!(cfg.client_secret_file.is_some());
}

// Scenario: A config sets `expiry_buffer` to zero.
// Guarantees: Validation rejects it, preventing a refresh schedule with no lead time.
#[test]
fn zero_expiry_buffer_is_rejected() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "expiry_buffer": "0s",
    }))
    .expect_err("zero expiry_buffer must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A config sets `startup_timeout` to zero.
// Guarantees: Validation rejects it, so the readiness gate always has a positive bound.
#[test]
fn zero_startup_timeout_is_rejected() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "startup_timeout": "0s",
    }))
    .expect_err("zero startup_timeout must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: Duration fields are given as human-readable strings.
// Guarantees: `expiry_buffer`, `timeout`, and `startup_timeout` parse to the expected durations.
#[test]
fn durations_parse_as_human_readable() {
    let cfg = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "expiry_buffer": "10m",
        "timeout": "2s",
        "startup_timeout": "45s",
    }))
    .expect("durations parse");
    assert_eq!(cfg.expiry_buffer, Duration::from_secs(600));
    assert_eq!(cfg.timeout, Some(Duration::from_secs(2)));
    assert_eq!(cfg.startup_timeout, Duration::from_secs(45));
}

// Scenario: Scopes and endpoint params are provided.
// Guarantees: Both deserialize into the config so they can be forwarded to the token endpoint.
#[test]
fn scopes_and_endpoint_params_parse() {
    let cfg = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "scopes": ["a", "b"],
        "endpoint_params": { "audience": "https://otlp.example.com" },
    }))
    .expect("scopes and params parse");
    assert_eq!(cfg.scopes, vec!["a".to_string(), "b".to_string()]);
    assert_eq!(
        cfg.endpoint_params.get("audience").map(String::as_str),
        Some("https://otlp.example.com")
    );
}

// Scenario: A config contains an unknown field.
// Guarantees: `deny_unknown_fields` rejects it, catching typos instead of silently ignoring them.
#[test]
fn unknown_fields_are_rejected() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "bogus": true,
    }))
    .expect_err("unknown field must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A TLS config sets `server_name_override`.
// Guarantees: Validation rejects it, because the reqwest/rustls token client cannot override SNI.
#[test]
fn server_name_override_is_rejected() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "tls": { "server_name_override": "example.com" },
    }))
    .expect_err("server_name_override must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A TLS config enables `insecure_skip_verify`.
// Guarantees: The config is accepted (the option is honored at client-build time), unlike the
// unsupported `server_name_override`.
#[test]
fn insecure_skip_verify_is_accepted_by_config() {
    let cfg = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "tls": { "insecure_skip_verify": true },
    }))
    .expect("insecure_skip_verify is a valid option");
    assert!(cfg.tls.is_some());
}

// Scenario: The factory's static `validate_config` hook is called with a valid config.
// Guarantees: It accepts the config, mirroring the parse-then-validate path used at wiring time.
#[test]
fn validate_config_hook_accepts_valid_config() {
    assert!(validate_config(&valid_config_json("https://idp.example.com/token")).is_ok());
}

// Scenario: The extension registers itself into the factory slice.
// Guarantees: It is discoverable under its URN and advertises the BearerTokenProvider capability.
#[test]
fn factory_is_registered_with_capability() {
    assert_eq!(OAUTH2_CLIENT_AUTH_EXTENSION.name, OAUTH2_CLIENT_AUTH_URN);
    let capabilities = OAUTH2_CLIENT_AUTH_EXTENSION
        .capabilities
        .as_ref()
        .expect("active extension advertises capabilities");
    assert!(
        capabilities.shared.contains(&"bearer_token_provider"),
        "BearerTokenProvider must be advertised as a shared capability"
    );
}

// -- Token acquisition / cache tests ---------------------------

// Scenario: A first get_token() misses the cache and a second call finds a fresh token.
// Guarantees: The slow path fetches once and the fast path serves the cached token without re-fetching.
#[tokio::test]
async fn get_token_slow_path_then_fast_path_caches() {
    let server = start_token_server("tok", 3600).await;
    let ext = make_extension(&format!("{}/token", server.uri()));

    let first = ext.get_token().await.expect("first acquisition");
    assert_eq!(first.expose_token(), "tok");

    let second = ext.get_token().await.expect("cached acquisition");
    assert_eq!(second.expose_token(), "tok");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(requests.len(), 1, "fast path must not re-fetch");
}

// Scenario: The token endpoint returns a token expiring inside the usability margin.
// Guarantees: Each get_token() treats the near-expiry token as stale and re-fetches.
#[tokio::test]
async fn near_expiry_token_is_refreshed() {
    // 5s expiry is within TOKEN_USABLE_MARGIN (30s), so the cached token is never "fresh".
    let server = start_token_server("tok", 5).await;
    let ext = make_extension(&format!("{}/token", server.uri()));

    let _ = ext.get_token().await.expect("first");
    let _ = ext.get_token().await.expect("second");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        requests.len(),
        2,
        "stale token must be refreshed on each call"
    );
}

// Scenario: The token endpoint returns HTTP 500.
// Guarantees: get_token() surfaces a CapabilityError tagged with this capability and extension name.
#[tokio::test]
async fn get_token_error_maps_to_capability_error() {
    let server = start_failing_server().await;
    let ext = make_extension(&format!("{}/token", server.uri()));

    let err = ext.get_token().await.expect_err("failing endpoint errors");
    assert_eq!(err.capability, "bearer_token_provider");
    assert_eq!(err.extension, "test-ext");
}

// Scenario: A second get_token() is issued immediately after a failed acquisition.
// Guarantees: The negative cache throttles the retry, so the token endpoint is hit only once.
#[tokio::test]
async fn get_token_throttles_after_recent_failure() {
    let server = start_failing_server().await;
    let ext = make_extension(&format!("{}/token", server.uri()));

    let _ = ext.get_token().await.expect_err("first attempt fails");
    let _ = ext.get_token().await.expect_err("second attempt throttled");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        requests.len(),
        1,
        "recent failure must throttle the next slow-path fetch"
    );
}

// Scenario: Two clones of the extension (as the engine hands to each consumer) each call get_token().
// Guarantees: They share one Arc<Inner> cache -- only one fetch occurs and both observe the same token,
// including via a stream subscription.
#[tokio::test]
async fn clones_share_one_token_cache() {
    let server = start_token_server("shared", 3600).await;
    let consumer_a = make_extension(&format!("{}/token", server.uri()));
    let consumer_b = consumer_a.clone();

    let a = consumer_a.get_token().await.expect("A acquires");
    assert_eq!(a.expose_token(), "shared");

    let b = consumer_b.get_token().await.expect("B acquires");
    assert_eq!(b.expose_token(), "shared");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        requests.len(),
        1,
        "clones must share one cache; B must not re-fetch"
    );

    let mut stream_b = consumer_b.token_stream();
    let streamed = stream_b
        .next()
        .await
        .expect("stream yields the shared token");
    assert_eq!(streamed.expose_token(), "shared");
}

// Scenario: A token is acquired after a stream subscription exists.
// Guarantees: The published token is delivered to the subscriber over the watch channel.
#[tokio::test]
async fn token_stream_yields_published_token() {
    let server = start_token_server("streamed", 3600).await;
    let ext = make_extension(&format!("{}/token", server.uri()));

    let mut stream = ext.token_stream();
    let _ = ext.get_token().await.expect("token acquired");
    let published = stream.next().await.expect("stream yields a value");
    assert_eq!(published.expose_token(), "streamed");
}

// Scenario: The config sets scopes and endpoint params, and the mock only responds when the request
// body carries them.
// Guarantees: The token request forwards `scope` and the extra endpoint params to the token endpoint.
#[tokio::test]
async fn request_includes_scope_and_endpoint_params() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("scope=telemetry.write"))
        .and(body_string_contains("audience=https"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "tok",
            "token_type": "Bearer",
            "expires_in": 3600,
        })))
        .mount(&server)
        .await;

    let cfg = config_from_json(serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "client_id": "id",
        "client_secret": "secret",
        "scopes": ["telemetry.write"],
        "endpoint_params": { "audience": "https://otlp.example.com" },
    }))
    .expect("valid config");
    let auth = Auth::new(&cfg).expect("auth builds");
    let (tx, _rx) = watch::channel(None);
    let ext =
        OAuth2ClientAuthExtension::new("test-ext", auth, cfg.expiry_buffer, tx, make_tracker());

    let token = ext
        .get_token()
        .await
        .expect("token acquired with matching request body");
    assert_eq!(token.expose_token(), "tok");
}

// Scenario: A client_secret_file is rewritten between two acquisitions of a near-expiry token.
// Guarantees: The next acquisition re-reads the file and authenticates with the rotated secret,
// with no restart.
#[tokio::test]
async fn client_secret_file_rotation_takes_effect() {
    let dir = tempfile::tempdir().expect("tempdir");
    let secret_path = dir.path().join("client_secret");
    std::fs::write(&secret_path, "secret-1").expect("write initial secret");

    // 1s expiry keeps the token inside the usability margin so the second call re-fetches.
    let server = start_token_server("tok", 1).await;
    let cfg = config_from_json(serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "client_id": "id",
        "client_secret_file": secret_path.to_string_lossy(),
    }))
    .expect("valid config");
    let auth = Auth::new(&cfg).expect("auth builds");
    let (tx, _rx) = watch::channel(None);
    let ext =
        OAuth2ClientAuthExtension::new("test-ext", auth, cfg.expiry_buffer, tx, make_tracker());

    let _ = ext.get_token().await.expect("first acquisition");
    std::fs::write(&secret_path, "secret-2").expect("rotate secret");
    let _ = ext
        .get_token()
        .await
        .expect("second acquisition after rotation");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(requests.len(), 2, "near-expiry token must be refetched");
    let auth1 = requests[0]
        .headers
        .get("authorization")
        .expect("first request carries an Authorization header");
    let auth2 = requests[1]
        .headers
        .get("authorization")
        .expect("second request carries an Authorization header");
    assert_ne!(
        auth1, auth2,
        "the rotated secret must change the Authorization header"
    );
}

// -- Metrics tracker tests -------------------------------------

// Scenario: The metric tracker records a success, a failure, and a publish, then reports.
// Guarantees: Snapshots move from all-zero to all-non-zero and reporting flushes to the channel.
#[test]
fn metrics_tracker_records_snapshots_and_reports() {
    let mut tracker = make_tracker();

    assert!(format!("{tracker:?}").contains("TokenProviderMetricsTracker"));

    let before = tracker.snapshot();
    assert!(
        before.get_metrics().iter().all(|m| m.is_zero()),
        "a new tracker starts at zero"
    );

    tracker.record_success(12.5);
    tracker.record_failure();
    tracker.record_publish();

    let after = tracker.snapshot();
    assert!(
        after.get_metrics().iter().all(|m| !m.is_zero()),
        "every metric is non-zero after recording"
    );

    let (rx, mut reporter) =
        otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(4);
    tracker.report(&mut reporter).expect("report succeeds");
    assert!(
        rx.try_recv().is_ok(),
        "reporter received the metric snapshot"
    );
}

// -- JWT-bearer grant tests ------------------------------------

// Scenario: A valid jwt-bearer config with all optional fields is deserialized.
// Guarantees: The grant and signature algorithm parse, so operators can select the jwt-bearer flow.
#[test]
fn jwt_bearer_config_parses() {
    let cfg = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "grant_type": "jwt-bearer",
        "client_id": "id",
        "client_certificate_key": "test-signing-key",
        "signature_algorithm": "RS384",
        "client_certificate_key_id": "kid-1",
        "iss": "issuer",
        "audience": "https://aud.example.com",
        "claims": { "foo": "bar" },
    }))
    .expect("valid jwt-bearer config");
    assert_eq!(cfg.grant_type, GrantType::JwtBearer);
    assert_eq!(cfg.signature_algorithm, Some(SignatureAlgorithm::Rs384));
}

// Scenario: A jwt-bearer config omits both signing-key fields.
// Guarantees: Validation rejects it, since the grant cannot sign an assertion without a key.
#[test]
fn jwt_bearer_requires_signing_key() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "grant_type": "jwt-bearer",
        "client_id": "id",
    }))
    .expect_err("missing signing key must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A jwt-bearer config also sets a client secret.
// Guarantees: Validation rejects the client_credentials-only field for the jwt-bearer grant.
#[test]
fn jwt_bearer_rejects_client_secret() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "grant_type": "jwt-bearer",
        "client_id": "id",
        "client_certificate_key": "test-signing-key",
        "client_secret": "nope",
    }))
    .expect_err("client_secret must be rejected for jwt-bearer");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A client_credentials config sets a jwt-bearer-only field (`iss`).
// Guarantees: Validation rejects the jwt-bearer-only field for the client_credentials grant.
#[test]
fn client_credentials_rejects_jwt_fields() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "secret",
        "iss": "issuer",
    }))
    .expect_err("jwt-bearer fields must be rejected for client_credentials");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A jwt-bearer acquisition signs an assertion and posts it to the token endpoint.
// Guarantees: The request carries the jwt-bearer grant_type and an `assertion` that verifies against
// the public key and carries the expected iss/sub/aud/exp/jti claims; the token response is returned.
#[tokio::test]
async fn jwt_bearer_signs_assertion_and_acquires_token() {
    let (private_key_pem, public_key_pem) = generate_test_rsa_keypair();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=urn"))
        .and(body_string_contains("assertion="))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "jwt-tok",
            "token_type": "Bearer",
            "expires_in": 3600,
        })))
        .mount(&server)
        .await;

    let token_url = format!("{}/token", server.uri());
    let cfg = config_from_json(serde_json::json!({
        "token_url": token_url.clone(),
        "grant_type": "jwt-bearer",
        "client_id": "svc-account",
        "client_certificate_key": private_key_pem,
        "scopes": ["telemetry.write"],
    }))
    .expect("valid jwt-bearer config");
    let auth = Auth::new(&cfg).expect("auth builds");
    let (tx, _rx) = watch::channel(None);
    let ext =
        OAuth2ClientAuthExtension::new("test-ext", auth, cfg.expiry_buffer, tx, make_tracker());

    let token = ext.get_token().await.expect("jwt-bearer token acquired");
    assert_eq!(token.expose_token(), "jwt-tok");

    // Recover the assertion the extension sent and verify its signature + claims.
    let requests = server.received_requests().await.expect("requests recorded");
    let body = String::from_utf8(requests[0].body.clone()).expect("utf8 request body");
    let assertion = body
        .split('&')
        .find_map(|kv| kv.strip_prefix("assertion="))
        .expect("assertion parameter present");

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    // The audience is asserted explicitly below rather than through the validator.
    validation.validate_aud = false;
    let decoded = jsonwebtoken::decode::<serde_json::Value>(
        assertion,
        &jsonwebtoken::DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .expect("public key parses"),
        &validation,
    )
    .expect("assertion verifies against the public key");
    let claims = decoded.claims;
    assert_eq!(claims["iss"], "svc-account");
    assert_eq!(claims["sub"], "svc-account");
    assert_eq!(claims["aud"], serde_json::Value::String(token_url));
    assert!(claims.get("jti").is_some(), "assertion carries a jti");
    assert!(claims.get("exp").is_some(), "assertion carries an exp");
}

// Scenario: A jwt-bearer config supplies the signing key via `client_certificate_key_file`.
// Guarantees: The key is read from the file and used to sign the assertion, yielding a token.
#[tokio::test]
async fn jwt_bearer_reads_signing_key_from_file() {
    let (private_key_pem, _) = generate_test_rsa_keypair();
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("signing_key.pem");
    std::fs::write(&key_path, &private_key_pem).expect("write signing key");

    let server = start_token_server("jwt-tok", 3600).await;
    let cfg = config_from_json(serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "grant_type": "jwt-bearer",
        "client_id": "svc-account",
        "client_certificate_key_file": key_path.to_string_lossy(),
    }))
    .expect("valid jwt-bearer config");
    let auth = Auth::new(&cfg).expect("auth builds");
    let (tx, _rx) = watch::channel(None);
    let ext =
        OAuth2ClientAuthExtension::new("test-ext", auth, cfg.expiry_buffer, tx, make_tracker());

    let token = ext
        .get_token()
        .await
        .expect("token acquired via key file");
    assert_eq!(token.expose_token(), "jwt-tok");
}
