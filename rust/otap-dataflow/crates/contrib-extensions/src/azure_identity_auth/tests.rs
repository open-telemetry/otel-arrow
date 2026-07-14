// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the Azure Identity Auth extension.

use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fmt, fs};

use async_trait::async_trait;
use azure_core::Bytes;
use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
use azure_core::http::StatusCode::BadRequest;
use azure_core::http::headers::{HeaderValue, Headers};
use azure_core::http::{AsyncRawResponse, HttpClient, Method, Request, StatusCode, Transport};
use azure_core::time::{Duration as AzureDuration, OffsetDateTime};
use azure_identity::ManagedIdentityCredentialOptions;
use azure_identity::UserAssignedId::ObjectId;
use futures::future::BoxFuture;
use futures::lock::Mutex;
use futures::{FutureExt, StreamExt};
use otap_df_config::error::Error as ConfigError;
use otap_df_engine::shared::capability::BearerTokenProvider as SharedBearerTokenProvider;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::testing::EmptyAttributes;
use tokio::sync::watch;

use crate::azure_identity_auth::arc_server_managed_identity::ArcServerManagedIdentity;

use super::auth::Auth;
use super::config::{AuthMethod, Config};
use super::error::Error;
use super::extension::AzureIdentityAuthExtension;
use super::metrics::{AzureIdentityAuthMetrics, AzureIdentityAuthMetricsTracker};
use super::*;

// ── Config tests ───────────────────────────────────────────

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

#[test]
fn config_defaults_apply() {
    let cfg = config_from_json(serde_json::json!({})).expect("empty config is valid");
    assert_eq!(cfg.method, AuthMethod::ManagedIdentity);
    assert_eq!(cfg.scope, "https://monitor.azure.com/.default");
    assert!(cfg.client_id.is_none());
    assert!(cfg.tenant_id.is_none());
    assert!(cfg.token_file_path.is_none());
    assert_eq!(cfg.startup_timeout, std::time::Duration::from_secs(30));
}

#[test]
fn startup_timeout_parses_and_rejects_zero() {
    let cfg = config_from_json(serde_json::json!({ "startup_timeout": "45s" }))
        .expect("human-readable duration parses");
    assert_eq!(cfg.startup_timeout, std::time::Duration::from_secs(45));

    assert!(
        config_from_json(serde_json::json!({ "startup_timeout": "0s" })).is_err(),
        "zero startup_timeout must be rejected"
    );
}

#[test]
fn method_aliases_deserialize() {
    let cases = [
        ("msi", AuthMethod::ManagedIdentity),
        ("managed_identity", AuthMethod::ManagedIdentity),
        ("managedidentity", AuthMethod::ManagedIdentity),
        ("dev", AuthMethod::Development),
        ("developer", AuthMethod::Development),
        ("cli", AuthMethod::Development),
        ("development", AuthMethod::Development),
        ("wif", AuthMethod::WorkloadIdentity),
        ("workload_identity", AuthMethod::WorkloadIdentity),
        ("workloadidentity", AuthMethod::WorkloadIdentity),
    ];
    for (alias, expected) in cases {
        let cfg = config_from_json(serde_json::json!({ "method": alias }))
            .unwrap_or_else(|e| panic!("alias `{alias}` should deserialize: {e}"));
        assert_eq!(cfg.method, expected, "alias `{alias}`");
    }
}

#[test]
fn empty_scope_is_rejected() {
    let err = config_from_json(serde_json::json!({ "scope": "   " }))
        .expect_err("whitespace scope must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

#[test]
fn unknown_fields_are_rejected() {
    let err = config_from_json(serde_json::json!({ "bogus": true }))
        .expect_err("unknown field must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

#[test]
fn per_method_fields_are_validated() {
    // `tenant_id` / `token_file_path` only apply to workload_identity.
    assert!(
        config_from_json(serde_json::json!({ "method": "managed_identity", "tenant_id": "t" }))
            .is_err()
    );
    assert!(
        config_from_json(
            serde_json::json!({ "method": "development", "token_file_path": "/tmp/x" })
        )
        .is_err()
    );
    // `client_id` is not valid for developer tooling.
    assert!(
        config_from_json(serde_json::json!({ "method": "development", "client_id": "c" })).is_err()
    );
    // Valid combinations still pass.
    assert!(
        config_from_json(serde_json::json!({ "method": "managed_identity", "client_id": "c" }))
            .is_ok()
    );
    assert!(
        config_from_json(serde_json::json!({
            "method": "workload_identity",
            "tenant_id": "t",
            "token_file_path": "/tmp/x",
            "client_id": "c",
        }))
        .is_ok()
    );
}

#[test]
fn validate_config_hook_accepts_valid_config() {
    assert!(validate_config(&serde_json::json!({ "method": "managed_identity" })).is_ok());
}

#[test]
fn factory_is_registered_with_capability() {
    assert_eq!(AZURE_IDENTITY_AUTH_EXTENSION.name, AZURE_IDENTITY_AUTH_URN);
    let capabilities = AZURE_IDENTITY_AUTH_EXTENSION
        .capabilities
        .as_ref()
        .expect("active extension advertises capabilities");
    assert!(
        capabilities.shared.contains(&"bearer_token_provider"),
        "BearerTokenProvider must be advertised as a shared capability"
    );
}

// ── Credential construction tests ──────────────────────────────

#[test]
fn managed_identity_system_assigned_credential_constructs() {
    otap_df_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({ "method": "managed_identity" })).unwrap();
    assert!(Auth::new(&cfg).is_ok());
}

#[test]
fn managed_identity_user_assigned_credential_constructs() {
    otap_df_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({
        "method": "managed_identity",
        "client_id": "00000000-0000-0000-0000-000000000000",
    }))
    .unwrap();
    assert!(Auth::new(&cfg).is_ok());
}

#[test]
fn development_credential_constructs() {
    otap_df_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({ "method": "development" })).unwrap();
    assert!(Auth::new(&cfg).is_ok());
}

#[test]
fn workload_identity_credential_construct_is_attempted() {
    otap_df_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({
        "method": "workload_identity",
        "client_id": "test-client",
        "tenant_id": "test-tenant",
        "token_file_path": "/tmp/does-not-exist",
    }))
    .unwrap();
    // Construction only validates configuration; a missing env/file surfaces as
    // a CreateCredential error. Both outcomes are acceptable here.
    match Auth::new(&cfg) {
        Ok(_) => {}
        Err(Error::CreateCredential { method, .. }) => {
            assert_eq!(method, AuthMethod::WorkloadIdentity);
        }
        Err(other) => panic!("unexpected error: {other:?}"),
    }
}

// ── Token acquisition / cache tests ───────────────────────────

#[derive(Debug)]
struct MockCredential {
    token: String,
    expires_in: AzureDuration,
    call_count: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl TokenCredential for MockCredential {
    async fn get_token(
        &self,
        _scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        let _ = self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(AccessToken {
            token: self.token.clone().into(),
            expires_on: OffsetDateTime::now_utc() + self.expires_in,
        })
    }
}

#[derive(Debug)]
struct FailingCredential {
    call_count: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl TokenCredential for FailingCredential {
    async fn get_token(
        &self,
        _scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        let _ = self.call_count.fetch_add(1, Ordering::SeqCst);
        Err(azure_core::error::Error::with_message(
            azure_core::error::ErrorKind::Credential,
            "mock credential failure",
        ))
    }
}

fn make_extension(credential: Arc<dyn TokenCredential>) -> AzureIdentityAuthExtension {
    let auth = Auth::from_credential(credential, "test_scope".to_string());
    let (tx, _rx) = watch::channel(None);
    AzureIdentityAuthExtension::new("test-ext", auth, tx, make_tracker())
}

fn make_tracker() -> AzureIdentityAuthMetricsTracker {
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<AzureIdentityAuthMetrics>(EmptyAttributes());
    AzureIdentityAuthMetricsTracker::new(metric_set)
}

#[tokio::test]
async fn get_token_slow_path_then_fast_path_caches() {
    let calls = Arc::new(AtomicUsize::new(0));
    let credential = Arc::new(MockCredential {
        token: "tok".to_string(),
        expires_in: AzureDuration::minutes(60),
        call_count: Arc::clone(&calls),
    });
    let ext = make_extension(credential);

    let first = ext.get_token().await.expect("first acquisition");
    assert_eq!(first.expose_secret(), "tok");
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    // Fresh cached token is returned without another credential call.
    let second = ext.get_token().await.expect("cached acquisition");
    assert_eq!(second.expose_secret(), "tok");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "fast path must not re-fetch"
    );
}

#[tokio::test]
async fn near_expiry_token_is_refreshed() {
    let calls = Arc::new(AtomicUsize::new(0));
    // Expiry inside the usability safety margin -> always treated as stale.
    let credential = Arc::new(MockCredential {
        token: "tok".to_string(),
        expires_in: AzureDuration::seconds(5),
        call_count: Arc::clone(&calls),
    });
    let ext = make_extension(credential);

    let _ = ext.get_token().await.expect("first");
    let _ = ext.get_token().await.expect("second");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "stale token must be refreshed on each call"
    );
}

#[tokio::test]
async fn get_token_error_maps_to_capability_error() {
    let ext = make_extension(Arc::new(FailingCredential {
        call_count: Arc::new(AtomicUsize::new(0)),
    }));
    let err = ext
        .get_token()
        .await
        .expect_err("failing credential errors");
    assert_eq!(err.capability, "bearer_token_provider");
    assert_eq!(err.extension, "test-ext");
}

#[tokio::test]
async fn clones_share_one_token_cache() {
    let calls = Arc::new(AtomicUsize::new(0));
    let credential = Arc::new(MockCredential {
        token: "shared".to_string(),
        expires_in: AzureDuration::minutes(60),
        call_count: Arc::clone(&calls),
    });

    // The engine hands the capability to each consumer as a clone of the same
    // extension; model that with two clones sharing one `Arc<Inner>`.
    let consumer_a = make_extension(credential);
    let consumer_b = consumer_a.clone();

    // Consumer A's first call takes the slow path and fetches exactly once.
    let a = consumer_a.get_token().await.expect("A acquires");
    assert_eq!(a.expose_secret(), "shared");
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    // Consumer B (a separate clone) sees the same cached token on the fast
    // path -- no second credential call. This proves clones share one cache
    // and refresh loop rather than each keeping its own.
    let b = consumer_b.get_token().await.expect("B acquires");
    assert_eq!(b.expose_secret(), "shared");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "clones must share one cache; B must not re-fetch"
    );

    // A token published through one clone is also visible via another clone's
    // stream subscription (shared watch channel).
    let mut stream_b = consumer_b.token_stream();
    let streamed = stream_b
        .next()
        .await
        .expect("stream yields the shared token");
    assert_eq!(streamed.expose_secret(), "shared");
}

// ── Arc server managed identity tests ─────────────────────────

#[tokio::test]
async fn arc_challenge_response() {
    let key_id = rand::random::<u8>();
    let token_path = env::temp_dir().join(format!("arc-{key_id}.key"));
    let token_path_str = token_path.to_str().unwrap().to_string();
    let mut token_file = File::create(&token_path).unwrap();
    token_file.write_all("abc".as_bytes()).unwrap();
    drop(token_file);

    let mut model_naive_req = Request::new(
        "http://localhost:40342/metadata/identity/oauth2/token"
            .parse()
            .unwrap(),
        Method::Get,
    );
    model_naive_req.insert_header("metadata", "true");

    let params = Vec::from([
        ("api-version", "2021-02-01"),
        ("resource", "https://monitor.azure.com"),
        ("object_id", "abc-123"),
    ]);
    let _ = model_naive_req
        .url_mut()
        .query_pairs_mut()
        .extend_pairs(params);

    let mut model_challenge_response_request = model_naive_req.clone();
    model_challenge_response_request.insert_header("authorization", "Basic abc");

    let mut challenge_headers = Headers::default();
    let response_header = HeaderValue::from(format!("Basic realm={token_path_str}"));

    challenge_headers.insert("www-authenticate", response_header);

    run_get_token_test(Some(ManagedIdentityCredentialOptions{
            user_assigned_id: Some(ObjectId("abc-123".into())),
            ..Default::default()
        }),
        vec![
            MockRequestResponse {
                request: model_challenge_response_request,
                response_status: StatusCode::Ok,
                response_headers: Headers::default(),
                response_format: r#"{"token_type":"Bearer","expires_in":"85770","expires_on":"EXPIRES_ON","ext_expires_in":86399,"access_token":"*","resource":"https://monitor.azure.com/.default"}"#.to_string(),
            },
            MockRequestResponse {
                request: model_naive_req,
                response_status: StatusCode::Unauthorized,
                response_headers: challenge_headers,
                response_format: String::from(r#"{"error":"unauthorized_client","error_description":"Missing Basic Authorization header","error_codes":[401]}"#),
            },
        ],
        None
    ).await;

    let _ = fs::remove_file(token_path); // try our best to clean up the temp file
}

#[tokio::test]
async fn arc_challenge_failed() {
    let key_id = rand::random::<u8>();
    let token_path = env::temp_dir().join(format!("arc-{key_id}.key"));
    let token_path_str = token_path.to_str().unwrap().to_string();
    let mut token_file = File::create(&token_path).unwrap();
    token_file.write_all("abc".as_bytes()).unwrap();
    drop(token_file);

    let mut model_naive_req = Request::new(
        "http://localhost:40342/metadata/identity/oauth2/token"
            .parse()
            .unwrap(),
        Method::Get,
    );
    model_naive_req.insert_header("metadata", "true");

    let params = Vec::from([
        ("api-version", "2021-02-01"),
        ("resource", "https://monitor.azure.com"),
        ("object_id", "abc-123"),
    ]);
    let _ = model_naive_req
        .url_mut()
        .query_pairs_mut()
        .extend_pairs(params);

    let mut model_challenge_response_request = model_naive_req.clone();
    model_challenge_response_request.insert_header("authorization", "Basic abc");

    let mut challenge_headers = Headers::default();
    let response_header = HeaderValue::from(format!("Basic realm={token_path_str}"));
    challenge_headers.insert("www-authenticate", response_header);

    run_get_token_test(Some(ManagedIdentityCredentialOptions{
            user_assigned_id: Some(ObjectId("abc-123".into())),
            ..Default::default()
        }),
        vec![
            MockRequestResponse {
                request: model_challenge_response_request,
                response_status: BadRequest,
                response_headers: Headers::default(),
                response_format: String::from(r#"{"error":"bad_request","error_description":"Something happened","error_codes":[400]}"#),
            },
            MockRequestResponse {
                request: model_naive_req,
                response_status: StatusCode::Unauthorized,
                response_headers: challenge_headers,
                response_format: String::from(r#"{"error":"unauthorized_client","error_description":"Missing Basic Authorization header","error_codes":[401]}"#),
            },
        ],
        Some(azure_core::Error::with_message(
            azure_core::error::ErrorKind::HttpResponse { status: BadRequest, error_code: Some("(unknown error code)".into()), raw_response: None },
            "The requested identity has not been assigned to this resource"
        ))
    ).await;

    let _ = fs::remove_file(token_path); // try our best to clean up the temp file
}

#[tokio::test]
async fn arc_multiple_scope_fails() {
    let mock_client =
        MockHttpClient::new(|_: &Request| panic!("Should not have made an HTTP call"));

    let mut options = ManagedIdentityCredentialOptions::default();
    options.client_options.transport = Some(Transport::new(Arc::new(mock_client)));

    let cred = ArcServerManagedIdentity::new(Some(options)).expect("credential");

    let _ = cred
        .get_token(
            &[
                "https://monitor.azure.com/.default",
                "https://notallowed.azure.com/.default",
            ],
            None,
        )
        .await
        .expect_err("Expected get_token to fail");
}

/// When using multiple entries in model_request_responses, it's important that the most specific request is first,
/// because this function will use the first request-response pair that matches the request received.
async fn run_get_token_test(
    options: Option<ManagedIdentityCredentialOptions>,
    model_request_responses: Vec<MockRequestResponse>,
    expected_token_err: Option<azure_core::Error>,
) {
    let expected_token_request_count = model_request_responses.len();
    let token_requests = Arc::new(AtomicUsize::new(0));
    let token_requests_clone = token_requests.clone();
    let expires_on = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;
    let mock_client = MockHttpClient::new(move |actual_req: &Request| {
        {
            let _ = token_requests_clone.fetch_add(1, Ordering::SeqCst);
            let model_request_responses = model_request_responses.clone();

            async move {
                for mock_request_response in model_request_responses {
                    let expected_request = mock_request_response.request;
                    if expected_request.method() != actual_req.method() {
                        continue;
                    }

                    let mut actual_params: Vec<_> =
                        actual_req.url().query_pairs().into_owned().collect();
                    actual_params.sort();
                    let mut expected_params: Vec<_> =
                        expected_request.url().query_pairs().into_owned().collect();
                    expected_params.sort();

                    if expected_params != actual_params {
                        continue;
                    }

                    let mut actual_url = actual_req.url().clone();
                    actual_url.set_query(None);
                    let mut expected_url = expected_request.url().clone();
                    expected_url.set_query(None);

                    if actual_url != expected_url {
                        continue;
                    }

                    // allow additional headers in the actual request so changing
                    // the underlying client in the future won't break tests
                    if !expected_request.headers().iter().all(
                        |(expected_header_name, expected_header_val)| {
                            actual_req
                                .headers()
                                .get_str(expected_header_name)
                                .is_ok_and(|actual_header| {
                                    actual_header == expected_header_val.as_str()
                                })
                        },
                    ) {
                        continue;
                    }

                    return Ok(AsyncRawResponse::from_bytes(
                        mock_request_response.response_status,
                        mock_request_response.response_headers,
                        Bytes::from(mock_request_response.response_format.replacen(
                            "EXPIRES_ON",
                            &expires_on.to_string(),
                            1,
                        )),
                    ));
                }
                // if we got here, none of the model requests matched.
                panic!("None of the model requests matched the actual request received");
            }
        }
        .boxed()
    });
    let mut options = options.unwrap_or_default();
    options.client_options.transport = Some(Transport::new(Arc::new(mock_client)));

    let cred = ArcServerManagedIdentity::new(Some(options)).expect("credential");
    for _ in 0..4 {
        let token_result = cred
            .get_token(&["https://monitor.azure.com/.default"], None)
            .await;

        if let Some(ref expected_token_err) = expected_token_err {
            let actual_err = token_result.expect_err("Expected get_token to fail");
            assert_eq!(
                actual_err.kind().to_string(),
                expected_token_err.kind().to_string()
            );
            assert_eq!(actual_err.http_status(), expected_token_err.http_status());
            assert_eq!(actual_err.to_string(), expected_token_err.to_string());
        } else {
            let token = token_result.expect("Expected get_token to succeed");
            assert_eq!(token.expires_on.unix_timestamp(), expires_on as i64);
            assert_eq!(token.token.secret(), "*");
            assert_eq!(
                token_requests.load(Ordering::SeqCst),
                expected_token_request_count
            );
        }
    }
}

#[derive(Debug, Clone)]
struct MockRequestResponse {
    request: Request,

    response_status: StatusCode,
    response_headers: Headers,
    response_format: String,
}

pub struct MockHttpClient<C>(Mutex<C>);

impl<C> MockHttpClient<C>
where
    C: FnMut(&Request) -> BoxFuture<'_, azure_core::Result<AsyncRawResponse>> + Send + Sync,
{
    /// Creates a new `MockHttpClient` using a capture.
    ///
    /// The capture takes a `&Request` and returns a `BoxedFuture<Output = azure_core::Result<Response>>`.
    /// See the example on [`MockHttpClient`].
    pub fn new(client: C) -> Self {
        Self(Mutex::new(client))
    }
}

impl<C> fmt::Debug for MockHttpClient<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(stringify!("MockHttpClient"))
    }
}

#[async_trait]
impl<C> HttpClient for MockHttpClient<C>
where
    C: FnMut(&Request) -> BoxFuture<'_, azure_core::Result<AsyncRawResponse>> + Send + Sync,
{
    async fn execute_request(&self, req: &Request) -> azure_core::Result<AsyncRawResponse> {
        let mut client = self.0.lock().await;
        (client)(req).await
    }
}

// ── Metrics tracker tests ─────────────────────────────────────

#[test]
fn metrics_tracker_records_snapshots_and_reports() {
    let mut tracker = make_tracker();

    // Debug formatting is exercised for observability tooling.
    assert!(format!("{tracker:?}").contains("AzureIdentityAuthMetricsTracker"));

    // A fresh tracker snapshots to all-zero values.
    let before = tracker.snapshot();
    assert!(
        before.get_metrics().iter().all(|m| m.is_zero()),
        "a new tracker starts at zero"
    );

    tracker.record_success(12.5);
    tracker.record_failure();
    tracker.record_publish();

    // Every metric is non-zero once each counter/latency has been recorded.
    let after = tracker.snapshot();
    assert!(
        after.get_metrics().iter().all(|m| !m.is_zero()),
        "every metric is non-zero after recording"
    );

    // Reporting flushes the recorded metrics to the telemetry channel.
    let (rx, mut reporter) =
        otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(4);
    tracker.report(&mut reporter).expect("report succeeds");
    assert!(
        rx.try_recv().is_ok(),
        "reporter received the metric snapshot"
    );
}

#[tokio::test]
async fn get_token_throttles_after_recent_failure() {
    let calls = Arc::new(AtomicUsize::new(0));
    let ext = make_extension(Arc::new(FailingCredential {
        call_count: Arc::clone(&calls),
    }));

    // First miss actually hits the credential and fails.
    let _ = ext.get_token().await.expect_err("first attempt fails");
    // Second miss within the cooldown is throttled by the negative cache: it
    // errors without a further credential call.
    let _ = ext.get_token().await.expect_err("second attempt throttled");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "recent failure must throttle the next slow-path fetch"
    );
}

#[tokio::test]
async fn token_stream_yields_published_token() {
    let calls = Arc::new(AtomicUsize::new(0));
    let credential = Arc::new(MockCredential {
        token: "streamed".to_string(),
        expires_in: AzureDuration::minutes(60),
        call_count: Arc::clone(&calls),
    });
    let ext = make_extension(credential);

    let mut stream = ext.token_stream();
    // Acquiring a token publishes it onto the watch channel.
    let _ = ext.get_token().await.expect("token acquired");
    let published = stream.next().await.expect("stream yields a value");
    assert_eq!(published.expose_secret(), "streamed");
}

#[tokio::test]
async fn token_stream_skips_initial_none() {
    use std::time::Duration;

    let calls = Arc::new(AtomicUsize::new(0));
    let credential = Arc::new(MockCredential {
        token: "streamed".to_string(),
        expires_in: AzureDuration::minutes(60),
        call_count: Arc::clone(&calls),
    });
    let ext = make_extension(credential);

    let mut stream = ext.token_stream();
    // The cache starts as `None`; the stream must filter it out and stay
    // pending rather than yielding a spurious value.
    let before = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
    assert!(
        before.is_err(),
        "stream must not yield before a token is published"
    );

    // Once a token is published, the stream yields it.
    let _ = ext.get_token().await.expect("token acquired");
    let published = tokio::time::timeout(Duration::from_millis(200), stream.next())
        .await
        .expect("stream yields after publish")
        .expect("stream is not closed");
    assert_eq!(published.expose_secret(), "streamed");
}

// ── schedule_next timing tests ────────────────────────────────

#[tokio::test]
async fn schedule_next_refreshes_before_expiry() {
    use otap_df_engine::capability::BearerToken;
    use std::time::{Duration, Instant};

    let token = BearerToken::new(
        "t".to_owned(),
        Some(Instant::now() + Duration::from_secs(3600)),
    );
    let refresh_at = extension::schedule_next(&token);
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs_f64();
    // 3600 - TOKEN_EXPIRY_BUFFER_SECS (299) = 3301, allowing execution slack.
    assert!((secs - 3301.0).abs() < 5.0, "expected ~3301s, got {secs}");
}

#[tokio::test]
async fn schedule_next_floors_near_expiry() {
    use otap_df_engine::capability::BearerToken;
    use std::time::{Duration, Instant};

    // Expires in 5s: the refresh target underflows past `now`, so the
    // MIN_TOKEN_REFRESH_INTERVAL_SECS (10s) floor applies.
    let token = BearerToken::new(
        "t".to_owned(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    let refresh_at = extension::schedule_next(&token);
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs_f64();
    assert!((secs - 10.0).abs() < 2.0, "expected ~10s floor, got {secs}");
}

#[tokio::test]
async fn schedule_next_pushes_non_expiring_far_out() {
    use otap_df_engine::capability::BearerToken;

    let token = BearerToken::new("t".to_owned(), None);
    let refresh_at = extension::schedule_next(&token);
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs();
    // Non-expiring tokens push the refresh ~1 year out.
    assert!(
        secs > 300 * 24 * 60 * 60,
        "expected far-future refresh, got {secs}s"
    );
}

// ── retry backoff tests ───────────────────────────────────────

#[test]
fn retry_backoff_grows_exponentially_and_caps() {
    // Zero prior failures starts at the base retry interval (10s).
    assert_eq!(extension::retry_backoff_secs(0), 10);
    // Each consecutive failure doubles the base delay.
    assert_eq!(extension::retry_backoff_secs(1), 20);
    assert_eq!(extension::retry_backoff_secs(2), 40);
    assert_eq!(extension::retry_backoff_secs(3), 80);
    assert_eq!(extension::retry_backoff_secs(4), 160);
    // Growth is clamped at the max (300s) and stays there.
    assert_eq!(extension::retry_backoff_secs(5), 300);
    assert_eq!(extension::retry_backoff_secs(6), 300);
    // A very large failure count must not overflow the shift.
    assert_eq!(extension::retry_backoff_secs(u32::MAX), 300);
}

// ── jitter_refresh tests ──────────────────────────────────────

#[tokio::test]
async fn jitter_refresh_preserves_min_interval_floor() {
    use std::time::Duration;

    // A target exactly at the 10s minimum-refresh floor has no slack to jitter,
    // so it must be returned unchanged rather than pulled toward `now` (which
    // would busy-loop the refresh task while the token is still fresh).
    let target = tokio::time::Instant::now() + Duration::from_secs(10);
    for _ in 0..1000 {
        assert_eq!(
            extension::jitter_refresh(target),
            target,
            "near-floor target must not be jittered earlier"
        );
    }
}

#[tokio::test]
async fn jitter_refresh_stays_within_bounds() {
    use std::time::Duration;

    // A far-out target is jittered earlier by at most REFRESH_JITTER_SECS (60s)
    // and never earlier than the 10s floor from `now`.
    let now = tokio::time::Instant::now();
    let target = now + Duration::from_secs(3600);
    let floor = now + Duration::from_secs(10);
    for _ in 0..1000 {
        let jittered = extension::jitter_refresh(target);
        assert!(
            jittered <= target,
            "jitter must only move the refresh earlier"
        );
        assert!(
            jittered >= target - Duration::from_secs(60),
            "jitter must not exceed REFRESH_JITTER_SECS"
        );
        assert!(
            jittered >= floor,
            "jitter must not precede the min-interval floor"
        );
    }
}
