// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bearer token provider handle for extensions.
//!
//! This module defines a token provider contract for extensions that manage
//! bearer authentication tokens (e.g., Azure Managed Identity, OAuth2 flows):
//!
//! - [`BearerTokenProvider`] — trait for components that acquire and refresh tokens.
//! - [`BearerTokenProviderHandle`] — a cloneable handle that consumers use to
//!   obtain tokens and subscribe to refresh events.
//!
//! # Examples
//!
//! ## Extension factory — registering the handle
//!
//! ```rust,ignore
//! let provider = MyTokenProvider { /* ... */ };
//!
//! let mut handles = ExtensionHandles::new();
//! handles.register(BearerTokenProviderHandle::new(provider));
//! ```
//!
//! ## Exporter — obtaining a token
//!
//! ```rust,ignore
//! let token_handle = extension_registry
//!     .get::<BearerTokenProviderHandle>("my_auth")?;
//!
//! let token = token_handle.get_token().await?;
//! request.headers_mut().insert(
//!     http::header::AUTHORIZATION,
//!     format!("Bearer {}", token.token.secret()).parse().unwrap(),
//! );
//! ```
//!
//! ## Subscribing to token refresh
//!
//! ```rust,ignore
//! let token_handle = extension_registry
//!     .get::<BearerTokenProviderHandle>("my_auth")?;
//!
//! let mut token_rx = token_handle.subscribe_token_refresh().await;
//! loop {
//!     tokio::select! {
//!         _ = token_rx.changed() => {
//!             if let Some(token) = token_rx.borrow().as_ref() {
//!                 // Update headers, etc.
//!             }
//!         }
//!     }
//! }
//! ```

use async_trait::async_trait;
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

// ─── Secret ────────────────────────────────────────────────────────────────

/// Represents a secret value that should not be exposed in logs or debug output.
///
/// The [`Debug`] implementation will not print the actual secret value.
#[derive(Clone, Eq)]
pub struct Secret(Cow<'static, str>);

impl Secret {
    /// Creates a new `Secret`.
    #[must_use]
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self(value.into())
    }

    /// Returns the secret value.
    #[must_use]
    pub fn secret(&self) -> &str {
        &self.0
    }
}

impl PartialEq for Secret {
    fn eq(&self, other: &Self) -> bool {
        self.secret() == other.secret()
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&'static str> for Secret {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret")
    }
}

// ─── BearerToken ───────────────────────────────────────────────────────────

/// Represents a bearer token with its expiration time.
///
/// The token value is wrapped in [`Secret`] to prevent accidental exposure
/// in logs or debug output.
#[derive(Debug, Clone)]
pub struct BearerToken {
    /// The token value.
    pub token: Secret,
    /// The expiration time as a UNIX timestamp (seconds since epoch).
    pub expires_on: i64,
}

impl BearerToken {
    /// Creates a new bearer token.
    #[must_use]
    pub fn new<T>(token: T, expires_on: i64) -> Self
    where
        T: Into<Secret>,
    {
        Self {
            token: token.into(),
            expires_on,
        }
    }
}

// ─── Error ─────────────────────────────────────────────────────────────────

/// An error returned by bearer token provider operations.
#[derive(Debug, Clone)]
pub struct BearerTokenError {
    /// A human-readable description of the failure.
    pub message: String,
}

impl fmt::Display for BearerTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bearer token error: {}", self.message)
    }
}

impl std::error::Error for BearerTokenError {}

// ─── Trait ─────────────────────────────────────────────────────────────────

/// A trait for components that can provide bearer authentication tokens.
///
/// Extensions implementing this trait can be looked up by other components
/// (e.g., exporters) to obtain tokens for authentication.
///
/// The extension background task handles periodic token refresh. Consumers
/// can either call [`get_token`](BearerTokenProvider::get_token) on demand
/// or subscribe to refresh notifications via
/// [`subscribe_token_refresh`](BearerTokenProvider::subscribe_token_refresh).
#[async_trait]
pub trait BearerTokenProvider: Send {
    /// Returns an authentication token.
    ///
    /// # Errors
    ///
    /// Returns a [`BearerTokenError`] if the token cannot be obtained.
    async fn get_token(&self) -> Result<BearerToken, BearerTokenError>;

    /// Subscribes to token refresh events.
    ///
    /// Returns a new receiver that will be notified whenever the token
    /// is refreshed. Each call creates an independent subscription.
    /// The receiver always contains the latest token value (or `None`
    /// if no token has been acquired yet).
    fn subscribe_token_refresh(&self) -> tokio::sync::watch::Receiver<Option<BearerToken>>;
}

// ─── Handle ────────────────────────────────────────────────────────────────

/// A cloneable handle that consumers use to obtain bearer tokens.
///
/// This wraps any [`BearerTokenProvider`] behind an `Arc<tokio::sync::Mutex<…>>`
/// because [`get_token`](BearerTokenProvider::get_token) is async — a
/// `std::sync::Mutex` cannot be held across `.await` points. Each consumer
/// gets its own clone. The `tokio::Mutex` makes the handle `Sync` without
/// requiring `Sync` on the trait itself.
#[derive(Clone)]
pub struct BearerTokenProviderHandle {
    inner: Arc<tokio::sync::Mutex<Box<dyn BearerTokenProvider>>>,
}

impl BearerTokenProviderHandle {
    /// Creates a new handle wrapping the given provider implementation.
    pub fn new(provider: impl BearerTokenProvider + 'static) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(Box::new(provider))),
        }
    }

    /// Returns an authentication token.
    ///
    /// Acquires the internal lock, then delegates to the underlying
    /// [`BearerTokenProvider`] implementation.
    pub async fn get_token(&self) -> Result<BearerToken, BearerTokenError> {
        self.inner.lock().await.get_token().await
    }

    /// Subscribes to token refresh events.
    ///
    /// Acquires the internal lock, then delegates to the underlying
    /// [`BearerTokenProvider`] implementation.
    pub async fn subscribe_token_refresh(
        &self,
    ) -> tokio::sync::watch::Receiver<Option<BearerToken>> {
        self.inner.lock().await.subscribe_token_refresh()
    }
}

impl fmt::Debug for BearerTokenProviderHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BearerTokenProviderHandle").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::watch;

    /// A trivial in-memory token provider for testing.
    struct StaticTokenProvider {
        token: String,
        expires_on: i64,
        sender: Arc<watch::Sender<Option<BearerToken>>>,
    }

    #[async_trait]
    impl BearerTokenProvider for StaticTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, BearerTokenError> {
            Ok(BearerToken::new(self.token.clone(), self.expires_on))
        }

        fn subscribe_token_refresh(&self) -> watch::Receiver<Option<BearerToken>> {
            self.sender.subscribe()
        }
    }

    fn make_static_provider(token: &str, expires_on: i64) -> StaticTokenProvider {
        let (sender, _) = watch::channel(None);
        StaticTokenProvider {
            token: token.to_owned(),
            expires_on,
            sender: Arc::new(sender),
        }
    }

    #[tokio::test]
    async fn handle_get_token() {
        let handle =
            BearerTokenProviderHandle::new(make_static_provider("test-token", 1_700_000_000));

        let token = handle.get_token().await.unwrap();
        assert_eq!(token.token.secret(), "test-token");
        assert_eq!(token.expires_on, 1_700_000_000);
    }

    #[tokio::test]
    async fn handle_subscribe_receives_updates() {
        let (sender, _) = watch::channel(None);
        let sender = Arc::new(sender);

        let provider = StaticTokenProvider {
            token: "initial".to_owned(),
            expires_on: 100,
            sender: Arc::clone(&sender),
        };

        let handle = BearerTokenProviderHandle::new(provider);
        let mut rx = handle.subscribe_token_refresh().await;

        // Simulate a token refresh from the extension background task.
        let _ = sender.send(Some(BearerToken::new("refreshed", 200)));

        rx.changed().await.unwrap();
        let refreshed = rx.borrow().clone().unwrap();
        assert_eq!(refreshed.token.secret(), "refreshed");
        assert_eq!(refreshed.expires_on, 200);
    }

    #[test]
    fn secret_debug_does_not_leak() {
        let s = Secret::new("super-secret-value");
        assert_eq!(format!("{:?}", s), "Secret");
    }

    #[test]
    fn secret_equality() {
        let a = Secret::new("same");
        let b = Secret::new("same");
        let c = Secret::new("different");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn bearer_token_from_string() {
        let token = BearerToken::new("my-token".to_string(), 42);
        assert_eq!(token.token.secret(), "my-token");
        assert_eq!(token.expires_on, 42);
    }
}
