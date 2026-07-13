// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Token sources for the Kubernetes Service Account Token extension.
//!
//! A [`TokenSource`] both *obtains* a token and *declares how it is refreshed*,
//! so each acquisition mode carries its own refresh mechanics:
//!
//! - [`FileTokenSource`] reads a mounted token file (projected or legacy secret
//!   volume). kubelet swaps the projected token via an atomic symlink rename in
//!   the mount directory, so refresh is driven by a filesystem watch on that
//!   directory, with an `exp`-derived timer as a backstop against missed events.
//!
//! Additional sources (e.g. the `TokenRequest` API or reading a `Secret` via the
//! Kubernetes API) can implement [`TokenSource`] with their own refresh strategy
//! without changing the extension's background loop.

use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use otap_df_engine::capability::BearerToken;

use super::config::Config;
use super::error::Error;

/// The filesystem watch is the primary refresh trigger; this backstop timer only
/// guards against a missed watch event, so it fires close to expiry. It is
/// comfortably below the enforced 10-minute minimum projected-token lifetime, so
/// it always lands after kubelet's ~80%-of-lifetime rotation.
const REFRESH_BACKSTOP_MARGIN: Duration = Duration::from_secs(60);

/// Floor for the backstop timer so a near-expired token cannot schedule a
/// re-read in the past (which would busy-loop).
const MIN_REFRESH_INTERVAL: Duration = Duration::from_secs(10);

/// Obtains a bearer token and declares how it should be refreshed.
#[async_trait]
pub trait TokenSource: Send + Sync {
    /// Fetches the currently valid token.
    async fn get_token(&self) -> Result<BearerToken, Error>;

    /// Directory to watch for token rotation events, if this source is backed by
    /// a mounted file. Returns `None` for sources with no filesystem to watch.
    fn watch_dir(&self) -> Option<PathBuf>;

    /// The instant at which the token should next be re-fetched, or `None` when
    /// the token has no known expiry (rotation is then picked up by the watch
    /// alone).
    fn next_refresh(&self, token: &BearerToken) -> Option<Instant>;
}

/// Reads the service account token from a mounted file (projected volume or
/// legacy secret volume).
#[derive(Clone)]
pub struct FileTokenSource {
    token_file_path: PathBuf,
}

impl FileTokenSource {
    /// Builds a `FileTokenSource` from the extension configuration.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            token_file_path: config.token_file_path.clone(),
        }
    }
}

#[async_trait]
impl TokenSource for FileTokenSource {
    async fn get_token(&self) -> Result<BearerToken, Error> {
        let raw = tokio::fs::read_to_string(&self.token_file_path)
            .await
            .map_err(|source| Error::ReadTokenFile {
                path: self.token_file_path.clone(),
                source,
            })?;

        let token = raw.trim();
        if token.is_empty() {
            return Err(Error::EmptyToken {
                path: self.token_file_path.clone(),
            });
        }

        Ok(match parse_jwt_expiry(token) {
            Some(expires_on) => BearerToken::from_absolute_expiry(token.to_owned(), expires_on),
            None => BearerToken::new(token.to_owned(), None),
        })
    }

    fn watch_dir(&self) -> Option<PathBuf> {
        self.token_file_path.parent().map(PathBuf::from)
    }

    fn next_refresh(&self, token: &BearerToken) -> Option<Instant> {
        let expires_on = token.expires_on()?;
        let now = Instant::now();
        let target = expires_on
            .checked_sub(REFRESH_BACKSTOP_MARGIN)
            .unwrap_or(now);
        Some(target.max(now + MIN_REFRESH_INTERVAL))
    }
}

/// Extracts the `exp` (expiration) claim from a JWT as a wall-clock time.
///
/// Returns `None` if the value is not a well-formed JWT with a numeric `exp`
/// claim; the token is still usable, only its expiry is unknown (a legacy,
/// non-expiring secret token).
fn parse_jwt_expiry(token: &str) -> Option<SystemTime> {
    // JWT layout: header.payload.signature — the claims are the middle segment,
    // base64url-encoded (no padding) JSON.
    let payload = token.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let exp = claims.get("exp")?.as_u64()?;
    // Saturating/checked add: a malformed, absurdly large `exp` must fall back to
    // the "expiry unknown" path (this fn's contract) rather than panic the
    // background refresh task on `SystemTime` overflow.
    UNIX_EPOCH.checked_add(Duration::from_secs(exp))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jwt(payload: &serde_json::Value) -> String {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
        let body = URL_SAFE_NO_PAD.encode(serde_json::to_vec(payload).unwrap());
        format!("{header}.{body}.signature")
    }

    #[test]
    fn parses_exp_claim() {
        let jwt = make_jwt(&serde_json::json!({ "exp": 1_700_000_000u64, "aud": ["x"] }));
        let exp = parse_jwt_expiry(&jwt).expect("exp parsed");
        assert_eq!(exp, UNIX_EPOCH + Duration::from_secs(1_700_000_000));
    }

    #[test]
    fn missing_exp_returns_none() {
        let jwt = make_jwt(&serde_json::json!({ "aud": ["x"] }));
        assert!(parse_jwt_expiry(&jwt).is_none());
    }

    #[test]
    fn non_jwt_returns_none() {
        assert!(parse_jwt_expiry("not-a-jwt").is_none());
        assert!(parse_jwt_expiry("a.b").is_none());
    }

    #[test]
    fn overflowing_exp_returns_none() {
        // An absurdly large `exp` must not panic on `SystemTime` overflow; it
        // falls back to the "expiry unknown" path.
        let jwt = make_jwt(&serde_json::json!({ "exp": u64::MAX }));
        assert!(parse_jwt_expiry(&jwt).is_none());
    }
}
