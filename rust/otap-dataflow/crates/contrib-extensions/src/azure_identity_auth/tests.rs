// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the Azure Identity Auth extension.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
use azure_core::time::{Duration as AzureDuration, OffsetDateTime};
use futures::StreamExt;
use otap_df_config::error::Error as ConfigError;
use otap_df_engine::shared::capability::BearerTokenProvider as SharedBearerTokenProvider;
use tokio::sync::watch;

use super::auth::Auth;
use super::config::{AuthMethod, Config};
use super::error::Error;
use super::extension::AzureIdentityAuthExtension;
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
struct FailingCredential;

#[async_trait::async_trait]
impl TokenCredential for FailingCredential {
    async fn get_token(
        &self,
        _scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        Err(azure_core::error::Error::with_message(
            azure_core::error::ErrorKind::Credential,
            "mock credential failure",
        ))
    }
}

fn make_extension(credential: Arc<dyn TokenCredential>) -> AzureIdentityAuthExtension {
    let auth = Auth::from_credential(credential, "test_scope".to_string());
    let (tx, _rx) = watch::channel(None);
    AzureIdentityAuthExtension::new("test-ext", auth, tx)
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
    // Expiry well inside the refresh-skew window -> always treated as stale.
    let credential = Arc::new(MockCredential {
        token: "tok".to_string(),
        expires_in: AzureDuration::seconds(30),
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
    let ext = make_extension(Arc::new(FailingCredential));
    let err = ext
        .get_token()
        .await
        .expect_err("failing credential errors");
    assert_eq!(err.capability, "bearer_token_provider");
    assert_eq!(err.extension, "test-ext");
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
