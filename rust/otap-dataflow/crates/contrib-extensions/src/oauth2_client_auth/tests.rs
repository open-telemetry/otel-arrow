// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit and integration tests for the OAuth 2.0 Client Auth extension.

use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::StreamExt;
use otap_df_config::error::Error as ConfigError;
use otap_df_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider as SharedBearerTokenProvider;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::testing::EmptyAttributes;
use otap_test_tls_certs::{ExtendedKeyUsage, generate_ca};
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio_rustls::TlsAcceptor;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::auth::Auth;
use super::config::{Config, GrantType, SignatureAlgorithm};
use super::metrics::OAuth2ClientAuthMetrics;
use super::*;
use crate::common::token_refresh::{TokenProviderMetricsTracker, TokenSource};

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
pub(super) fn generate_test_rsa_keypair() -> (String, String) {
    let key_pair =
        rcgen::KeyPair::generate_rsa_for(&rcgen::PKCS_RSA_SHA256, rcgen::RsaKeySize::_2048)
            .expect("generate test RSA key pair");
    (key_pair.serialize_pem(), key_pair.public_key_pem())
}

/// Builds a token config from the shared valid base plus `extra` top-level keys.
fn config_json_with(token_url: &str, extra: serde_json::Value) -> serde_json::Value {
    let mut base = valid_config_json(token_url);
    let object = base.as_object_mut().expect("config is a JSON object");
    for (key, value) in extra.as_object().expect("extra is a JSON object") {
        let _ = object.insert(key.clone(), value.clone());
    }
    base
}

/// Builds an extension from an already-parsed config, sharing the wiring every
/// acquisition test repeats.
fn extension_from_config(cfg: &Config) -> OAuth2ClientAuthExtension {
    let auth = Auth::new(cfg).expect("auth builds");
    let (tx, _rx) = watch::channel(None);
    OAuth2ClientAuthExtension::new("test-ext", auth, cfg.expiry_buffer, tx, make_tracker())
}

/// A TLS-terminating token endpoint whose certificate is signed by a throwaway
/// CA that the system trust store does not know.
struct TlsTokenServer {
    /// PEM of the CA that signed the server certificate.
    ca_pem: String,
    /// `https://localhost:<port>/token`.
    token_url: String,
}

/// Starts [`TlsTokenServer`]. wiremock cannot terminate TLS, so this is a
/// minimal hand-rolled HTTP/1.1 responder over `tokio-rustls` that replies to
/// any request with a canned token response.
async fn start_tls_token_server(access_token: &'static str) -> TlsTokenServer {
    otap_df_otap::crypto::ensure_crypto_provider();

    let ca = generate_ca("oauth2 client auth test CA");
    let ca_pem = ca.cert_pem.clone();
    let leaf = ca.issue_leaf(
        "localhost",
        Some("localhost"),
        Some(ExtendedKeyUsage::ServerAuth),
    );

    let cert_chain: Vec<_> = CertificateDer::pem_slice_iter(leaf.cert_pem.as_bytes())
        .collect::<Result<_, _>>()
        .expect("parse server cert chain");
    let key = PrivateKeyDer::from_pem_slice(leaf.key_pem.as_bytes()).expect("parse server key");
    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .expect("build tls server config");
    let acceptor = TlsAcceptor::from(Arc::new(server_config));

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("local addr").port();

    let _server = tokio::spawn(async move {
        loop {
            let Ok((tcp, _)) = listener.accept().await else {
                return;
            };
            let acceptor = acceptor.clone();
            let _conn = tokio::spawn(async move {
                let Ok(mut tls) = acceptor.accept(tcp).await else {
                    return;
                };
                // Read just past the request headers; the body is irrelevant
                // because the response is canned.
                let mut buf = vec![0u8; 8192];
                let mut read = 0usize;
                loop {
                    let Ok(n) = tls.read(&mut buf[read..]).await else {
                        return;
                    };
                    if n == 0 {
                        return;
                    }
                    read += n;
                    if buf[..read].windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                let body = serde_json::json!({
                    "access_token": access_token,
                    "token_type": "Bearer",
                    "expires_in": 3600,
                })
                .to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = tls.write_all(response.as_bytes()).await;
                let _ = tls.flush().await;
            });
        }
    });

    TlsTokenServer {
        ca_pem,
        token_url: format!("https://localhost:{port}/token"),
    }
}

// -- Config tests ----------------------------------------------

// Scenario: A minimal valid config is deserialized with all optional fields omitted.
// Guarantees: Documented defaults (client_credentials grant, 5m buffer, 30s startup, finite
// request/connect timeouts, 24h assumed lifetime, empty scopes/params, no tls) are applied so
// operators can rely on them.
#[test]
fn config_defaults_apply() {
    let cfg =
        config_from_json(valid_config_json("https://idp.example.com/token")).expect("valid config");
    assert_eq!(cfg.grant_type, GrantType::ClientCredentials);
    assert_eq!(cfg.expiry_buffer, Duration::from_secs(300));
    assert_eq!(cfg.startup_timeout, Duration::from_secs(30));
    assert!(cfg.scopes.is_empty());
    assert!(cfg.endpoint_params.is_empty());
    assert_eq!(cfg.timeout, Duration::from_secs(30));
    assert_eq!(cfg.connect_timeout, Duration::from_secs(10));
    assert_eq!(
        cfg.default_token_lifetime,
        Duration::from_secs(24 * 60 * 60)
    );
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

// Scenario: A config sets `expiry_buffer` at or below the fixed usability margin, so the
// background refresh would be scheduled only once the cached token is already unusable.
// Guarantees: Validation rejects it, so the extension can never be configured into a stall on
// every token cycle; a buffer just above the margin stays accepted.
#[test]
fn expiry_buffer_within_usability_margin_is_rejected() {
    let base = |buffer: &str| {
        serde_json::json!({
            "token_url": "https://idp.example.com/token",
            "client_id": "id",
            "client_secret": "s",
            "expiry_buffer": buffer,
        })
    };

    for buffer in ["0s", "10s", "30s"] {
        let err = config_from_json(base(buffer))
            .expect_err("expiry_buffer within the usability margin must be rejected");
        assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
    }

    let cfg = config_from_json(base("31s")).expect("expiry_buffer above the margin is valid");
    assert_eq!(cfg.expiry_buffer, Duration::from_secs(31));
}

// Scenario: A config sets `timeout` or `connect_timeout` to zero.
// Guarantees: Validation rejects both, so a token request can never be left without a finite
// bound and hold the acquisition lock indefinitely.
#[test]
fn zero_request_or_connect_timeout_is_rejected() {
    for field in ["timeout", "connect_timeout"] {
        let err = config_from_json(serde_json::json!({
            "token_url": "https://idp.example.com/token",
            "client_id": "id",
            "client_secret": "s",
            field: "0s",
        }))
        .unwrap_err();
        assert!(
            matches!(err, ConfigError::InvalidUserConfig { .. }),
            "{field}"
        );
    }
}

// Scenario: A config sets `default_token_lifetime` at or below `expiry_buffer`.
// Guarantees: Validation rejects it, so the fallback cannot place every refresh in the past and
// collapse the refresh loop onto its minimum cadence.
#[test]
fn a_fallback_lifetime_below_the_expiry_buffer_is_rejected() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "expiry_buffer": "5m",
        "default_token_lifetime": "5m",
    }))
    .expect_err("a fallback lifetime at the expiry buffer must be rejected");
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
    assert_eq!(cfg.timeout, Duration::from_secs(2));
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

// Scenario: A TLS config sets `insecure` (no TLS) while `token_url` uses the `https://` scheme.
// Guarantees: Validation rejects the contradiction, so an operator who believes they disabled TLS
// finds out at startup instead of silently getting a TLS connection.
#[test]
fn insecure_with_https_token_url_is_rejected() {
    let err = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "tls": { "insecure": true },
    }))
    .expect_err("insecure with an https token_url must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

// Scenario: A TLS config sets `insecure` alongside a plaintext `http://` token_url.
// Guarantees: The config is accepted, because the flag agrees with the scheme and is a no-op.
#[test]
fn insecure_with_http_token_url_is_accepted() {
    let cfg = config_from_json(serde_json::json!({
        "token_url": "http://localhost:8080/token",
        "client_id": "id",
        "client_secret": "s",
        "tls": { "insecure": true },
    }))
    .expect("insecure agrees with a plaintext token_url");
    assert!(cfg.tls.is_some());
}

// Scenario: A TLS config sets `insecure: false` with an `https://` token_url.
// Guarantees: The config is accepted, since only an explicit `true` contradicts the scheme.
#[test]
fn insecure_false_with_https_token_url_is_accepted() {
    let cfg = config_from_json(serde_json::json!({
        "token_url": "https://idp.example.com/token",
        "client_id": "id",
        "client_secret": "s",
        "tls": { "insecure": false },
    }))
    .expect("insecure=false is consistent with https");
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

/// Invokes the factory's `create` hook with `config` against a throwaway
/// extension context, mirroring how the engine wires the extension.
fn create_bundle(config: serde_json::Value) -> Result<ExtensionBundle, ConfigError> {
    let (ext_ctx, _registry) = otap_df_engine::testing::test_extension_ctx();
    let name: otap_df_config::ExtensionId = "oauth2-client-auth".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        OAUTH2_CLIENT_AUTH_URN.into(),
        config,
    ));
    let extension_config = ExtensionConfig::new(name.clone());
    create(&ext_ctx, name, user_config, &extension_config)
}

// Scenario: The factory's `create` hook runs against a valid client-credentials config.
// Guarantees: Wiring succeeds and yields a shared, active extension bundle usable by the engine.
#[test]
fn create_builds_a_shared_active_bundle() {
    let bundle = create_bundle(valid_config_json("https://idp.example.com/token"))
        .expect("a valid config wires successfully");
    assert!(
        bundle.local().is_none(),
        "the OAuth2 client auth extension has no local variant"
    );
    let shared = bundle.shared().expect("a shared variant is produced");
    assert_eq!(shared.variant(), ExtensionVariant::Shared);
    assert!(
        !shared.is_passive(),
        "the extension must be active so its refresh loop runs"
    );
}

// Scenario: The factory's `create` hook runs against a config that fails deserialization.
// Guarantees: Wiring fails fast with InvalidUserConfig instead of building a broken extension.
#[test]
fn create_rejects_a_malformed_config() {
    let Err(err) = create_bundle(serde_json::json!({ "client_id": "id" })) else {
        panic!("a config without token_url must be rejected");
    };
    assert!(
        matches!(err, ConfigError::InvalidUserConfig { .. }),
        "expected InvalidUserConfig, got {err:?}"
    );
}

// Scenario: The factory's `create` hook runs against a config whose TLS material cannot build a client.
// Guarantees: The Auth::new failure is surfaced as InvalidUserConfig at wiring time, not at first token fetch.
#[test]
fn create_surfaces_auth_construction_failures() {
    let leaf =
        generate_ca("client CA").issue_leaf("client", None, Some(ExtendedKeyUsage::ClientAuth));
    let config = config_json_with(
        "https://idp.example.com/token",
        serde_json::json!({ "tls": { "cert_pem": leaf.cert_pem } }),
    );
    let Err(err) = create_bundle(config) else {
        panic!("an mTLS certificate without a key must be rejected");
    };
    let ConfigError::InvalidUserConfig { error } = &err else {
        panic!("expected InvalidUserConfig, got {err:?}");
    };
    assert!(
        error.contains("failed to initialize OAuth2 client"),
        "error must identify the client-construction failure, got {error}"
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

// Scenario: A `client_secret_file` larger than the collector's shared TLS/credential size limit
// (4MB) is configured on the per-acquisition read path.
// Guarantees: The acquisition fails with a read error instead of loading the whole file into
// memory on every refresh, so an oversized or hostile path cannot exhaust memory.
#[tokio::test]
async fn oversized_client_secret_file_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let secret_path = dir.path().join("client_secret");
    std::fs::write(&secret_path, vec![b'x'; 5 * 1024 * 1024]).expect("write oversized secret");

    let server = start_token_server("tok", 3600).await;
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

    let _ = ext
        .get_token()
        .await
        .expect_err("an oversized credential file must be rejected");
    assert!(
        server
            .received_requests()
            .await
            .expect("requests recorded")
            .is_empty(),
        "the token endpoint must not be contacted with an unread credential"
    );
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

// Scenario: The token endpoint reports an `expires_in` far too large to add to the current time
// (client-credentials grant).
// Guarantees: The acquisition returns a token with no known expiry instead of panicking on the
// `SystemTime` overflow, so a hostile or buggy endpoint cannot crash the node.
#[tokio::test]
async fn absurd_expires_in_yields_token_without_expiry() {
    let server = start_token_server("no-expiry-tok", u64::MAX).await;
    let ext = make_extension(&format!("{}/token", server.uri()));

    let token = ext.get_token().await.expect("token acquired");
    assert_eq!(token.expose_token(), "no-expiry-tok");
    assert!(
        token.expires_on().is_none(),
        "an unrepresentable expiry must degrade to `no known expiry`"
    );
}

// Scenario: The token endpoint reports an `expires_in` far too large to add to the current time on
// the jwt-bearer path.
// Guarantees: The acquisition returns a token with no known expiry instead of panicking, matching
// the client-credentials path.
#[tokio::test]
async fn absurd_expires_in_yields_token_without_expiry_jwt_bearer() {
    let (private_key_pem, _) = generate_test_rsa_keypair();
    let server = start_token_server("no-expiry-jwt-tok", u64::MAX).await;

    let cfg = config_from_json(serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "grant_type": "jwt-bearer",
        "client_id": "svc-account",
        "client_certificate_key": private_key_pem,
    }))
    .expect("valid jwt-bearer config");
    let auth = Auth::new(&cfg).expect("auth builds");
    let (tx, _rx) = watch::channel(None);
    let ext =
        OAuth2ClientAuthExtension::new("test-ext", auth, cfg.expiry_buffer, tx, make_tracker());

    let token = ext.get_token().await.expect("jwt-bearer token acquired");
    assert_eq!(token.expose_token(), "no-expiry-jwt-tok");
    assert!(
        token.expires_on().is_none(),
        "an unrepresentable expiry must degrade to `no known expiry`"
    );
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

    let token = ext.get_token().await.expect("token acquired via key file");
    assert_eq!(token.expose_token(), "jwt-tok");
}

/// Runs one jwt-bearer acquisition with `extra` merged into the config, then
/// decodes the assertion the extension sent using `algorithm`.
async fn decode_sent_assertion(
    extra: serde_json::Value,
    algorithm: jsonwebtoken::Algorithm,
) -> jsonwebtoken::TokenData<serde_json::Value> {
    let (private_key_pem, public_key_pem) = generate_test_rsa_keypair();
    let server = start_token_server("jwt-tok", 3600).await;

    let mut cfg_json = serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "grant_type": "jwt-bearer",
        "client_id": "svc-account",
        "client_certificate_key": private_key_pem,
    });
    let object = cfg_json.as_object_mut().expect("config is a JSON object");
    for (key, value) in extra.as_object().expect("extra is a JSON object") {
        let _ = object.insert(key.clone(), value.clone());
    }

    let cfg = config_from_json(cfg_json).expect("valid jwt-bearer config");
    let ext = extension_from_config(&cfg);
    let _ = ext.get_token().await.expect("jwt-bearer token acquired");

    let requests = server.received_requests().await.expect("requests recorded");
    let body = String::from_utf8(requests[0].body.clone()).expect("utf8 request body");
    let assertion = body
        .split('&')
        .find_map(|kv| kv.strip_prefix("assertion="))
        .expect("assertion parameter present")
        .to_string();

    let mut validation = jsonwebtoken::Validation::new(algorithm);
    validation.validate_aud = false;
    jsonwebtoken::decode::<serde_json::Value>(
        &assertion,
        &jsonwebtoken::DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .expect("public key parses"),
        &validation,
    )
    .expect("assertion verifies against the public key")
}

// Scenario: A jwt-bearer config selects `signature_algorithm: RS384`.
// Guarantees: The assertion is signed with RS384 and declares it in the JWT header, so an IdP that
// requires a specific algorithm is honored rather than silently downgraded to the RS256 default.
#[tokio::test]
async fn jwt_bearer_signs_with_rs384() {
    let decoded = decode_sent_assertion(
        serde_json::json!({ "signature_algorithm": "RS384" }),
        jsonwebtoken::Algorithm::RS384,
    )
    .await;
    assert_eq!(decoded.header.alg, jsonwebtoken::Algorithm::RS384);
}

// Scenario: A jwt-bearer config selects `signature_algorithm: RS512`.
// Guarantees: The assertion is signed with RS512 and declares it in the JWT header.
#[tokio::test]
async fn jwt_bearer_signs_with_rs512() {
    let decoded = decode_sent_assertion(
        serde_json::json!({ "signature_algorithm": "RS512" }),
        jsonwebtoken::Algorithm::RS512,
    )
    .await;
    assert_eq!(decoded.header.alg, jsonwebtoken::Algorithm::RS512);
}

// Scenario: A jwt-bearer config sets `client_certificate_key_id`.
// Guarantees: The value is emitted as the JWT `kid` header, which IdPs holding several registered
// public keys need in order to select the right one for verification.
#[tokio::test]
async fn jwt_bearer_key_id_becomes_the_kid_header() {
    let decoded = decode_sent_assertion(
        serde_json::json!({ "client_certificate_key_id": "signing-key-1" }),
        jsonwebtoken::Algorithm::RS256,
    )
    .await;
    assert_eq!(decoded.header.kid.as_deref(), Some("signing-key-1"));
}

// Scenario: A jwt-bearer config overrides `iss` and `audience`, which otherwise default to
// `client_id` and `token_url`.
// Guarantees: The overrides land in the assertion's `iss`/`aud` claims while `sub` stays the
// client_id, matching the documented defaults-and-overrides behavior.
#[tokio::test]
async fn jwt_bearer_iss_and_audience_overrides_apply() {
    let decoded = decode_sent_assertion(
        serde_json::json!({
            "iss": "custom-issuer",
            "audience": "https://aud.example.com",
        }),
        jsonwebtoken::Algorithm::RS256,
    )
    .await;
    assert_eq!(decoded.claims["iss"], "custom-issuer");
    assert_eq!(decoded.claims["aud"], "https://aud.example.com");
    assert_eq!(
        decoded.claims["sub"], "svc-account",
        "sub stays the client_id even when iss is overridden"
    );
}

// Scenario: A jwt-bearer config supplies extra `claims`, one of which collides with a standard
// claim the extension computes (`iss`).
// Guarantees: Non-colliding claims are merged into the assertion, and the extension's standard
// claims win over user-supplied ones, so a config cannot forge `iss`/`sub`/`aud`/`exp`.
#[tokio::test]
async fn jwt_bearer_merges_extra_claims_and_standard_claims_win() {
    let decoded = decode_sent_assertion(
        serde_json::json!({
            "claims": { "tenant": "acme", "iss": "forged-issuer" },
        }),
        jsonwebtoken::Algorithm::RS256,
    )
    .await;
    assert_eq!(decoded.claims["tenant"], "acme");
    assert_eq!(
        decoded.claims["iss"], "svc-account",
        "a user-supplied `iss` claim must not override the computed one"
    );
}

// -- TLS tests -------------------------------------------------

// Scenario: The token endpoint is served over TLS with a certificate signed by a private CA that
// the system trust store does not know, and the config supplies that CA via `tls.ca_pem`.
// Guarantees: The configured CA is added to the client's trust store, so the handshake succeeds and
// a token is acquired over HTTPS.
#[tokio::test]
async fn tls_ca_pem_trusts_a_private_ca() {
    let server = start_tls_token_server("tls-tok").await;
    let cfg = config_from_json(config_json_with(
        &server.token_url,
        serde_json::json!({ "tls": { "ca_pem": server.ca_pem.clone() } }),
    ))
    .expect("valid config");

    let token = extension_from_config(&cfg)
        .get_token()
        .await
        .expect("token acquired over TLS");
    assert_eq!(token.expose_token(), "tls-tok");
}

// Scenario: The same private-CA TLS endpoint is used with no `tls` block at all.
// Guarantees: Certificate verification is on by default, so an untrusted server certificate fails
// the acquisition instead of silently succeeding.
#[tokio::test]
async fn tls_untrusted_server_certificate_is_rejected() {
    let server = start_tls_token_server("tls-tok").await;
    let cfg = config_from_json(valid_config_json(&server.token_url)).expect("valid config");

    let _ = extension_from_config(&cfg)
        .get_token()
        .await
        .expect_err("a server certificate from an unknown CA must fail verification");
}

// Scenario: The private CA is supplied through `tls.ca_file` rather than inline `ca_pem`.
// Guarantees: The file form is read and trusted identically to the inline form.
#[tokio::test]
async fn tls_ca_file_trusts_a_private_ca() {
    let server = start_tls_token_server("tls-file-tok").await;
    let dir = tempfile::tempdir().expect("tempdir");
    let ca_path = dir.path().join("ca.pem");
    std::fs::write(&ca_path, &server.ca_pem).expect("write CA");

    let cfg = config_from_json(config_json_with(
        &server.token_url,
        serde_json::json!({ "tls": { "ca_file": ca_path.to_string_lossy() } }),
    ))
    .expect("valid config");

    let token = extension_from_config(&cfg)
        .get_token()
        .await
        .expect("token acquired with a CA file");
    assert_eq!(token.expose_token(), "tls-file-tok");
}

// Scenario: `tls.insecure_skip_verify` is enabled against the untrusted private-CA endpoint with no
// CA configured.
// Guarantees: Server certificate verification is disabled, so the acquisition that
// `tls_untrusted_server_certificate_is_rejected` proves fails now succeeds.
#[tokio::test]
async fn tls_insecure_skip_verify_accepts_an_untrusted_certificate() {
    let server = start_tls_token_server("skip-verify-tok").await;
    let cfg = config_from_json(config_json_with(
        &server.token_url,
        serde_json::json!({ "tls": { "insecure_skip_verify": true } }),
    ))
    .expect("valid config");

    let token = extension_from_config(&cfg)
        .get_token()
        .await
        .expect("insecure_skip_verify must bypass verification");
    assert_eq!(token.expose_token(), "skip-verify-tok");
}

// Scenario: `tls.include_system_ca_certs_pool` is disabled while the private CA is configured.
// Guarantees: The client trusts exactly the configured CA rather than falling back to an empty
// trust store, so the certs-only branch is wired correctly.
#[tokio::test]
async fn tls_certs_only_pool_trusts_the_configured_ca() {
    let server = start_tls_token_server("certs-only-tok").await;
    let cfg = config_from_json(config_json_with(
        &server.token_url,
        serde_json::json!({
            "tls": {
                "ca_pem": server.ca_pem.clone(),
                "include_system_ca_certs_pool": false,
            },
        }),
    ))
    .expect("valid config");

    let token = extension_from_config(&cfg)
        .get_token()
        .await
        .expect("token acquired against the only trusted CA");
    assert_eq!(token.expose_token(), "certs-only-tok");
}

// Scenario: A TLS config supplies an mTLS client certificate with no matching private key.
// Guarantees: Client construction fails with an explicit mTLS error at startup rather than
// producing a client that silently omits the certificate.
#[test]
fn mtls_client_certificate_without_key_is_rejected() {
    let leaf =
        generate_ca("client CA").issue_leaf("client", None, Some(ExtendedKeyUsage::ClientAuth));
    let cfg = config_from_json(config_json_with(
        "https://idp.example.com/token",
        serde_json::json!({ "tls": { "cert_pem": leaf.cert_pem } }),
    ))
    .expect("config parses");

    // `Auth` deliberately has no `Debug` impl (it holds secrets), so `expect_err`
    // is not available here.
    let Err(err) = Auth::new(&cfg) else {
        panic!("a client certificate without a key must be rejected");
    };
    assert!(
        err.to_string()
            .contains("both a client certificate and key are required for mTLS"),
        "unexpected error: {err}"
    );
}

// Scenario: A TLS config supplies an mTLS private key with no matching client certificate.
// Guarantees: The same explicit mTLS error is raised, so the check is symmetric.
#[test]
fn mtls_client_key_without_certificate_is_rejected() {
    let leaf =
        generate_ca("client CA").issue_leaf("client", None, Some(ExtendedKeyUsage::ClientAuth));
    let cfg = config_from_json(config_json_with(
        "https://idp.example.com/token",
        serde_json::json!({ "tls": { "key_pem": leaf.key_pem } }),
    ))
    .expect("config parses");

    let Err(err) = Auth::new(&cfg) else {
        panic!("a client key without a certificate must be rejected");
    };
    assert!(
        err.to_string()
            .contains("both a client certificate and key are required for mTLS"),
        "unexpected error: {err}"
    );
}

// -- Timeout and credential rotation ---------------------------

// Scenario: The token endpoint delays its response far beyond the configured `timeout`.
// Guarantees: The acquisition aborts on the configured deadline instead of blocking the refresh
// task for the endpoint's full response time.
#[tokio::test]
async fn request_timeout_aborts_a_slow_token_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(30))
                .set_body_json(serde_json::json!({
                    "access_token": "slow-tok",
                    "token_type": "Bearer",
                    "expires_in": 3600,
                })),
        )
        .mount(&server)
        .await;

    let cfg = config_from_json(config_json_with(
        &format!("{}/token", server.uri()),
        serde_json::json!({ "timeout": "100ms" }),
    ))
    .expect("valid config");

    let started = Instant::now();
    let _ = extension_from_config(&cfg)
        .get_token()
        .await
        .expect_err("a response slower than `timeout` must fail the acquisition");
    assert!(
        started.elapsed() < Duration::from_secs(30),
        "the acquisition must abort on the configured timeout, not wait for the response"
    );
}

// Scenario: A client_id_file is rewritten between two acquisitions of a near-expiry token.
// Guarantees: The next acquisition re-reads the file and authenticates with the rotated client id,
// matching the client_secret_file rotation behavior with no restart.
#[tokio::test]
async fn client_id_file_rotation_takes_effect() {
    let dir = tempfile::tempdir().expect("tempdir");
    let id_path = dir.path().join("client_id");
    std::fs::write(&id_path, "client-1").expect("write initial client id");

    // 1s expiry keeps the token inside the usability margin so the second call re-fetches.
    let server = start_token_server("tok", 1).await;
    let cfg = config_from_json(serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "client_id_file": id_path.to_string_lossy(),
        "client_secret": "secret",
    }))
    .expect("valid config");
    let ext = extension_from_config(&cfg);

    let _ = ext.get_token().await.expect("first acquisition");
    std::fs::write(&id_path, "client-2").expect("rotate client id");
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
        "the rotated client id must change the Authorization header"
    );
}

// -- Auth construction and credential-read error paths ----------

// Scenario: A plaintext http:// token endpoint is configured.
// Guarantees: Auth still builds (the endpoint is usable) while the insecure-endpoint
// warning path runs, so operators are told rather than blocked.
#[test]
fn plaintext_token_url_builds_auth() {
    let cfg = config_from_json(config_json_with(
        "http://idp.example.com/token",
        serde_json::json!({ "tls": { "insecure": true } }),
    ))
    .expect("http token_url with insecure=true parses");
    assert!(
        Auth::new(&cfg).is_ok(),
        "a plaintext endpoint must warn, not fail"
    );
}

// Scenario: A token_url passes config validation but is not a parsable URL.
// Guarantees: Auth::new fails with BuildHttpClient naming token_url instead of panicking later.
#[test]
fn unparsable_token_url_is_rejected_by_auth() {
    let cfg = config_from_json(config_json_with(
        "https://",
        serde_json::json!({ "tls": { "insecure": false } }),
    ))
    .expect("config validation only checks the scheme");
    let Err(err) = Auth::new(&cfg) else {
        panic!("an unparsable token_url must be rejected");
    };
    assert!(
        err.to_string().contains("invalid token_url"),
        "unexpected error: {err}"
    );
}

// Scenario: A ca_file points at a path that does not exist.
// Guarantees: Client construction fails with ReadCredentialFile naming the path,
// so a typo is diagnosed at startup rather than as a TLS handshake failure.
#[test]
fn missing_ca_file_is_reported_with_its_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("absent-ca.pem");
    let cfg = config_from_json(config_json_with(
        "https://idp.example.com/token",
        serde_json::json!({ "tls": { "ca_file": missing.to_string_lossy() } }),
    ))
    .expect("config parses");
    let Err(err) = Auth::new(&cfg) else {
        panic!("a missing ca_file must be rejected");
    };
    assert!(
        err.to_string().contains("absent-ca.pem"),
        "error must name the offending path, got: {err}"
    );
}

// Scenario: A ca_file exists but does not contain a certificate.
// Guarantees: Client construction fails rather than silently trusting nothing extra.
#[test]
fn unparsable_ca_file_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ca_path = dir.path().join("garbage-ca.pem");
    std::fs::write(
        &ca_path,
        "-----BEGIN CERTIFICATE-----\nZ\n-----END CERTIFICATE-----\n",
    )
    .expect("write garbage CA");
    let cfg = config_from_json(config_json_with(
        "https://idp.example.com/token",
        serde_json::json!({ "tls": { "ca_file": ca_path.to_string_lossy() } }),
    ))
    .expect("config parses");
    assert!(
        Auth::new(&cfg).is_err(),
        "an unparsable CA bundle must be rejected"
    );
}

// Scenario: A complete mTLS client certificate and key pair is configured.
// Guarantees: The identity is accepted and the HTTP client builds, exercising the
// success side of the mTLS branch that the two rejection tests only cover negatively.
#[test]
fn mtls_certificate_and_key_pair_builds_a_client() {
    let leaf =
        generate_ca("client CA").issue_leaf("client", None, Some(ExtendedKeyUsage::ClientAuth));
    let cfg = config_from_json(config_json_with(
        "https://idp.example.com/token",
        serde_json::json!({
            "tls": { "cert_pem": leaf.cert_pem, "key_pem": leaf.key_pem },
        }),
    ))
    .expect("config parses");
    assert!(
        Auth::new(&cfg).is_ok(),
        "a matching certificate and key must build an mTLS client"
    );
}

// Scenario: A client_secret_file points at a path that does not exist.
// Guarantees: Token acquisition fails with the path named, and the failure surfaces
// per-acquisition rather than at construction (the file is read on every refresh).
#[tokio::test]
async fn missing_client_secret_file_fails_acquisition() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("absent-secret");
    let server = start_token_server("tok", 3600).await;
    let cfg = config_from_json(serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "client_id": "id",
        "client_secret_file": missing.to_string_lossy(),
    }))
    .expect("config parses");
    let auth = Auth::new(&cfg).expect("auth builds");
    let err = auth
        .fetch_token()
        .await
        .expect_err("a missing credential file must fail acquisition");
    assert!(
        err.to_string().contains("absent-secret"),
        "error must name the offending path, got: {err}"
    );
}

// Scenario: A client_secret_file contains bytes that are not valid UTF-8.
// Guarantees: Acquisition fails with an explicit encoding error naming the field,
// instead of sending mangled credentials to the token endpoint.
#[tokio::test]
async fn non_utf8_client_secret_file_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let secret_path = dir.path().join("client_secret");
    std::fs::write(&secret_path, [0xff, 0xfe, 0xfd]).expect("write non-UTF-8 secret");
    let server = start_token_server("tok", 3600).await;
    let cfg = config_from_json(serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "client_id": "id",
        "client_secret_file": secret_path.to_string_lossy(),
    }))
    .expect("config parses");
    let auth = Auth::new(&cfg).expect("auth builds");
    let err = auth
        .fetch_token()
        .await
        .expect_err("non-UTF-8 credentials must be rejected");
    assert!(
        err.to_string().contains("valid UTF-8"),
        "unexpected error: {err}"
    );
}

// Scenario: A JWT-bearer client_certificate_key_file points at a path that does not exist.
// Guarantees: Acquisition fails with the path named rather than signing with empty key material.
#[tokio::test]
async fn missing_signing_key_file_fails_acquisition() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("absent-key.pem");
    let server = start_token_server("tok", 3600).await;
    let cfg = config_from_json(serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "grant_type": "jwt-bearer",
        "client_id": "id",
        "client_certificate_key_file": missing.to_string_lossy(),
    }))
    .expect("config parses");
    let auth = Auth::new(&cfg).expect("auth builds");
    let err = auth
        .fetch_token()
        .await
        .expect_err("a missing signing key file must fail acquisition");
    assert!(
        err.to_string().contains("absent-key.pem"),
        "error must name the offending path, got: {err}"
    );
}

// Scenario: A JWT-bearer signing key is present but is not a usable RSA private key.
// Guarantees: Acquisition fails with a signing error naming the key, and no request is sent.
#[tokio::test]
async fn unusable_signing_key_fails_before_any_request() {
    let server = start_token_server("tok", 3600).await;
    let cfg = config_from_json(serde_json::json!({
        "token_url": format!("{}/token", server.uri()),
        "grant_type": "jwt-bearer",
        "client_id": "id",
        "client_certificate_key": "-----BEGIN PRIVATE KEY-----\nZ\n-----END PRIVATE KEY-----\n",
    }))
    .expect("config parses");
    let auth = Auth::new(&cfg).expect("auth builds");
    let err = auth
        .fetch_token()
        .await
        .expect_err("an unusable signing key must fail acquisition");
    assert!(
        err.to_string().contains("invalid RSA signing key"),
        "unexpected error: {err}"
    );
    assert!(
        server
            .received_requests()
            .await
            .expect("requests recorded")
            .is_empty(),
        "signing must fail before the token endpoint is contacted"
    );
}

// -- JWT-bearer token endpoint response handling ----------------

/// Builds a JWT-bearer `Auth` pointed at `token_url` with a freshly generated key.
fn jwt_bearer_auth(token_url: &str) -> Auth {
    let (private_pem, _public_pem) = generate_test_rsa_keypair();
    let cfg = config_from_json(serde_json::json!({
        "token_url": token_url,
        "grant_type": "jwt-bearer",
        "client_id": "id",
        "client_certificate_key": private_pem,
    }))
    .expect("config parses");
    Auth::new(&cfg).expect("auth builds")
}

// Scenario: A JWT-bearer token endpoint answers with a non-2xx status and a body.
// Guarantees: The status and body are both surfaced in the error so the failure is diagnosable.
#[tokio::test]
async fn jwt_bearer_surfaces_error_status_and_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid_client"))
        .mount(&server)
        .await;

    let auth = jwt_bearer_auth(&format!("{}/token", server.uri()));
    let err = auth
        .fetch_token()
        .await
        .expect_err("a 401 must fail acquisition");
    let message = err.to_string();
    assert!(
        message.contains("401"),
        "error must carry the status: {message}"
    );
    assert!(
        message.contains("invalid_client"),
        "error must carry the body: {message}"
    );
}

// Scenario: A JWT-bearer token endpoint answers 200 with a body that is not a token response.
// Guarantees: Deserialization failure is reported as an acquisition error, not a panic.
#[tokio::test]
async fn jwt_bearer_rejects_an_unparsable_success_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let auth = jwt_bearer_auth(&format!("{}/token", server.uri()));
    let err = auth
        .fetch_token()
        .await
        .expect_err("an unparsable body must fail acquisition");
    assert!(
        err.to_string().contains("invalid token response"),
        "unexpected error: {err}"
    );
}

// Scenario: A JWT-bearer token endpoint omits expires_in from an otherwise valid response.
// Guarantees: The token is given the configured fallback lifetime instead of being cached as
// non-expiring, so a short-lived token from a silent endpoint is still rotated.
#[tokio::test]
async fn jwt_bearer_response_without_expires_in_uses_the_fallback_lifetime() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "tok",
            "token_type": "Bearer",
        })))
        .mount(&server)
        .await;

    let auth = jwt_bearer_auth(&format!("{}/token", server.uri()));
    let token = auth.fetch_token().await.expect("acquisition succeeds");
    assert_eq!(token.expose_token(), "tok");
    let expires_on = token
        .expires_on()
        .expect("a response without expires_in must still get a finite expiry");
    let remaining = expires_on.duration_since(Instant::now());
    assert!(
        remaining > Duration::from_secs(86_000) && remaining <= Duration::from_secs(86_400),
        "expected the 24h default fallback, got {remaining:?}"
    );
}

// -- Shared token-response parsing -------------------------------

/// Builds a client-credentials `Auth` pointed at `token_url`.
fn client_credentials_auth(token_url: &str) -> Auth {
    let cfg = config_from_json(valid_config_json(token_url)).expect("config parses");
    Auth::new(&cfg).expect("auth builds")
}

/// Mounts a single token endpoint returning `body` with a 200 status.
async fn start_json_token_server(body: serde_json::Value) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    server
}

// Scenario: A client-credentials token endpoint omits expires_in from an otherwise valid response.
// Guarantees: The configured fallback lifetime is applied on this grant too, so neither grant can
// pin a token in the cache indefinitely.
#[tokio::test]
async fn client_credentials_response_without_expires_in_uses_the_fallback_lifetime() {
    let server = start_json_token_server(serde_json::json!({
        "access_token": "tok",
        "token_type": "Bearer",
    }))
    .await;

    let auth = client_credentials_auth(&format!("{}/token", server.uri()));
    let token = auth.fetch_token().await.expect("acquisition succeeds");
    let remaining = token
        .expires_on()
        .expect("a response without expires_in must still get a finite expiry")
        .duration_since(Instant::now());
    assert!(
        remaining > Duration::from_secs(86_000) && remaining <= Duration::from_secs(86_400),
        "expected the 24h default fallback, got {remaining:?}"
    );
}

// Scenario: A token endpoint reports `expires_in` as a JSON string rather than a number, which
// some providers do.
// Guarantees: Both grants accept it and carry the reported expiry through, instead of failing or
// silently falling back to the assumed lifetime.
#[tokio::test]
async fn expires_in_is_accepted_as_a_string_on_both_grants() {
    for jwt_bearer in [false, true] {
        let server = start_json_token_server(serde_json::json!({
            "access_token": "tok",
            "token_type": "Bearer",
            "expires_in": "120",
        }))
        .await;
        let token_url = format!("{}/token", server.uri());
        let auth = if jwt_bearer {
            jwt_bearer_auth(&token_url)
        } else {
            client_credentials_auth(&token_url)
        };

        let token = auth.fetch_token().await.expect("acquisition succeeds");
        let remaining = token
            .expires_on()
            .expect("a string expires_in must still produce an expiry")
            .duration_since(Instant::now());
        assert!(
            remaining > Duration::from_secs(110) && remaining <= Duration::from_secs(120),
            "expected the reported 120s expiry (jwt_bearer={jwt_bearer}), got {remaining:?}"
        );
    }
}

// Scenario: A token endpoint issues a token whose `token_type` is not Bearer.
// Guarantees: Both grants reject it rather than presenting a non-bearer credential to exporters
// that will send it in an `Authorization: Bearer` header.
#[tokio::test]
async fn a_non_bearer_token_type_is_rejected_on_both_grants() {
    for jwt_bearer in [false, true] {
        let server = start_json_token_server(serde_json::json!({
            "access_token": "tok",
            "token_type": "mac",
            "expires_in": 120,
        }))
        .await;
        let token_url = format!("{}/token", server.uri());
        let auth = if jwt_bearer {
            jwt_bearer_auth(&token_url)
        } else {
            client_credentials_auth(&token_url)
        };

        let err = auth
            .fetch_token()
            .await
            .expect_err("a non-bearer token_type must fail acquisition");
        assert!(
            err.to_string().contains("unsupported token_type"),
            "unexpected error (jwt_bearer={jwt_bearer}): {err}"
        );
    }
}

// Scenario: A token endpoint omits `token_type`, which RFC 6749 requires but some providers drop,
// and spells it with a leading capital in the case where it is present.
// Guarantees: Both spellings are treated as Bearer, so a compliant-enough provider still works.
#[tokio::test]
async fn a_missing_or_differently_cased_token_type_is_treated_as_bearer() {
    for body in [
        serde_json::json!({ "access_token": "tok", "expires_in": 120 }),
        serde_json::json!({ "access_token": "tok", "token_type": "BEARER", "expires_in": 120 }),
    ] {
        let server = start_json_token_server(body).await;
        let auth = client_credentials_auth(&format!("{}/token", server.uri()));
        let token = auth.fetch_token().await.expect("acquisition succeeds");
        assert_eq!(token.expose_token(), "tok");
    }
}

// Scenario: A token endpoint fails and echoes back a very large response body.
// Guarantees: The error carries only a bounded prefix of the body, so the warn-level refresh
// failure log cannot be filled with whatever the endpoint or an intermediary chose to return.
#[tokio::test]
async fn a_large_error_body_is_truncated_before_it_reaches_the_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(400).set_body_string("x".repeat(10_000)))
        .mount(&server)
        .await;

    let auth = jwt_bearer_auth(&format!("{}/token", server.uri()));
    let err = auth
        .fetch_token()
        .await
        .expect_err("a 400 must fail acquisition")
        .to_string();
    assert!(err.contains("400"), "status must be reported: {err}");
    assert!(
        err.contains("[truncated]"),
        "an oversized body must be marked as truncated: {err}"
    );
    assert!(
        err.len() < 512,
        "the error must stay bounded, got {} bytes",
        err.len()
    );
}
