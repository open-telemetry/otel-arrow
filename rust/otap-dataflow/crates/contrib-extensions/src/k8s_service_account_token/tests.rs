// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the Kubernetes Service Account Token extension.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use otap_df_config::error::Error as ConfigError;
use otap_df_engine::capability::BearerToken;
use otap_df_engine::shared::capability::BearerTokenProvider as SharedBearerTokenProvider;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::testing::EmptyAttributes;
use tempfile::TempDir;
use tokio::sync::watch;

use super::config::Config;
use super::extension::K8sServiceAccountTokenExtension;
use super::metrics::{K8sServiceAccountTokenMetrics, K8sServiceAccountTokenMetricsTracker};
use super::token_source::{FileTokenSource, TokenSource};
use super::*;

// ── Config tests ───────────────────────────────────────────

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

#[test]
fn config_defaults_apply() {
    let cfg = config_from_json(serde_json::json!({})).expect("empty config is valid");
    assert_eq!(
        cfg.token_file_path,
        std::path::PathBuf::from(config::DEFAULT_TOKEN_FILE_PATH)
    );
    assert_eq!(cfg.startup_timeout, Duration::from_secs(15));
}

#[test]
fn token_file_path_override() {
    let cfg =
        config_from_json(serde_json::json!({ "token_file_path": "/var/run/secrets/tokens/x" }))
            .expect("custom path is valid");
    assert_eq!(
        cfg.token_file_path,
        std::path::PathBuf::from("/var/run/secrets/tokens/x")
    );
}

#[test]
fn startup_timeout_parses_and_rejects_zero() {
    let cfg = config_from_json(serde_json::json!({ "startup_timeout": "45s" }))
        .expect("human-readable duration parses");
    assert_eq!(cfg.startup_timeout, Duration::from_secs(45));

    assert!(
        config_from_json(serde_json::json!({ "startup_timeout": "0s" })).is_err(),
        "zero startup_timeout must be rejected"
    );
}

#[test]
fn empty_token_file_path_rejected() {
    let err = config_from_json(serde_json::json!({ "token_file_path": "" }))
        .expect_err("empty path must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

#[test]
fn unknown_field_rejected() {
    assert!(config_from_json(serde_json::json!({ "nope": true })).is_err());
}

// ── Helpers ────────────────────────────────────────────────

fn make_jwt(exp: Option<u64>) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
    let claims = match exp {
        Some(exp) => serde_json::json!({ "exp": exp, "aud": ["api"] }),
        None => serde_json::json!({ "aud": ["api"] }),
    };
    let body = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).unwrap());
    format!("{header}.{body}.sig")
}

fn write_token(dir: &TempDir, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join("token");
    std::fs::write(&path, contents).unwrap();
    path
}

fn source_for(path: std::path::PathBuf) -> FileTokenSource {
    FileTokenSource::new(&Config {
        token_file_path: path,
        ..Config::default()
    })
}

fn make_tracker() -> K8sServiceAccountTokenMetricsTracker {
    let registry = TelemetryRegistryHandle::new();
    let metric_set =
        registry.register_metric_set::<K8sServiceAccountTokenMetrics>(EmptyAttributes());
    K8sServiceAccountTokenMetricsTracker::new(metric_set)
}

fn make_extension(path: std::path::PathBuf) -> K8sServiceAccountTokenExtension {
    let (tx, _rx) = watch::channel(None);
    K8sServiceAccountTokenExtension::new("test-ext", Arc::new(source_for(path)), tx, make_tracker())
}

// ── TokenSource tests ──────────────────────────────────────

#[tokio::test]
async fn token_source_reads_and_parses_expiry() {
    let far_future = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;
    let dir = TempDir::new().unwrap();
    let path = write_token(&dir, &format!("  {}\n", make_jwt(Some(far_future))));

    let token = source_for(path).get_token().await.expect("token read");
    assert!(token.expose_secret().starts_with("ey")); // trimmed JWT
    assert!(token.expires_on().is_some());
}

#[tokio::test]
async fn token_source_without_exp_has_no_expiry() {
    let dir = TempDir::new().unwrap();
    let path = write_token(&dir, &make_jwt(None));
    let token = source_for(path).get_token().await.expect("token read");
    assert!(token.expires_on().is_none());
}

#[tokio::test]
async fn token_source_empty_file_errors() {
    let dir = TempDir::new().unwrap();
    let path = write_token(&dir, "   \n");
    let err = source_for(path)
        .get_token()
        .await
        .expect_err("empty errors");
    assert!(matches!(err, error::Error::EmptyToken { .. }));
}

#[tokio::test]
async fn token_source_missing_file_errors() {
    let err = source_for("/nonexistent/k8s/token".into())
        .get_token()
        .await
        .expect_err("missing file errors");
    assert!(matches!(err, error::Error::ReadTokenFile { .. }));
}

// ── Provider tests ─────────────────────────────────────────

#[tokio::test]
async fn get_token_slow_path_then_fast_path() {
    let dir = TempDir::new().unwrap();
    let far_future = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;
    let path = write_token(&dir, &make_jwt(Some(far_future)));
    let ext = make_extension(path);

    let first = ext.get_token().await.expect("first read");
    let secret = first.expose_secret().to_owned();

    // Rewriting the file should not matter: the cached, still-fresh token is
    // returned without re-reading.
    let _ = write_token(&dir, "different");
    let second = ext.get_token().await.expect("cached read");
    assert_eq!(second.expose_secret(), secret);
}

#[tokio::test]
async fn get_token_missing_file_errors() {
    let ext = make_extension("/nonexistent/k8s/token".into());
    assert!(ext.get_token().await.is_err());
}

// ── next_refresh (backstop timer) tests ────────────────────

#[test]
fn next_refresh_is_backstop_before_expiry() {
    let src = source_for("/x/token".into());
    let token = BearerToken::from_absolute_expiry(
        "t".to_string(),
        SystemTime::now() + Duration::from_secs(3600),
    );
    let next = src
        .next_refresh(&token)
        .expect("expiry yields a backstop timer");
    let now = std::time::Instant::now();
    // ~3600s expiry minus a ~60s backstop margin -> just before expiry.
    assert!(next > now + Duration::from_secs(3000));
    assert!(next < now + Duration::from_secs(3600));
}

#[test]
fn next_refresh_none_without_expiry() {
    let src = source_for("/x/token".into());
    let token = BearerToken::new("t".to_string(), None);
    assert!(src.next_refresh(&token).is_none());
}

#[test]
fn next_refresh_floored_for_near_expiry() {
    let src = source_for("/x/token".into());
    let token = BearerToken::from_absolute_expiry(
        "t".to_string(),
        SystemTime::now() + Duration::from_secs(5),
    );
    let next = src
        .next_refresh(&token)
        .expect("expiry yields a backstop timer");
    let now = std::time::Instant::now();
    // Backstop is in the past, so it is floored to the minimum interval, not now.
    assert!(next > now);
    assert!(next <= now + Duration::from_secs(10));
}

#[test]
fn watch_dir_is_the_token_parent() {
    let src = source_for("/var/run/secrets/x/token".into());
    assert_eq!(
        src.watch_dir(),
        Some(std::path::PathBuf::from("/var/run/secrets/x"))
    );
}
