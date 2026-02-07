// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Token provider extension trait.

use async_trait::async_trait;
use std::borrow::Cow;

/// Represents a secret value that should not be exposed in logs or debug output.
///
/// The [`Debug`] implementation will not print the actual secret value.
#[derive(Clone, Eq)]
pub struct Secret(Cow<'static, str>);

impl Secret {
    /// Creates a new `Secret`.
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

// Constant-time comparison to prevent timing attacks.
// Note: LLVM may optimize this in unexpected ways.
impl PartialEq for Secret {
    fn eq(&self, other: &Self) -> bool {
        let a = self.secret();
        let b = other.secret();

        if a.len() != b.len() {
            return false;
        }

        a.bytes()
            .zip(b.bytes())
            .fold(0, |acc, (a, b)| acc | (a ^ b))
            == 0
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

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret")
    }
}

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

/// A trait for components that can provide bearer authentication tokens.
///
/// Extensions implementing this trait can be looked up by other components
/// (e.g., exporters) to obtain tokens for authentication.
///
/// # Thread Safety
///
/// - The returned future is `Send` for use with async runtimes like tokio
/// - The error type is `Send + Sync` for safe propagation across threads
///
/// # Subscribing to Token Refresh Events
///
/// Use [`subscribe_token_refresh`](BearerTokenProvider::subscribe_token_refresh) to receive notifications when
/// tokens are refreshed. This is useful for updating HTTP headers or other
/// authentication state without polling.
///
/// # Implementing This Trait
///
/// External crates can implement this trait on their extension types:
///
/// ```ignore
/// use async_trait::async_trait;
/// use otap_df_engine::extensions::{BearerToken, BearerTokenProvider, Error};
///
/// struct MyAuthExtension { /* ... */ }
///
/// #[async_trait]
/// impl BearerTokenProvider for MyAuthExtension {
///     async fn get_token(&self) -> Result<BearerToken, Error> {
///         // ... acquire token ...
///         Ok(BearerToken { token: "...".into(), expires_on: 0 })
///     }
///
///     fn subscribe_token_refresh(&self) -> tokio::sync::watch::Receiver<Option<BearerToken>> {
///         self.token_sender.subscribe()
///     }
/// }
/// ```
#[async_trait]
pub trait BearerTokenProvider: Send {
    /// Returns an authentication token.
    ///
    /// # Errors
    ///
    /// Returns an error if the token cannot be obtained.
    async fn get_token(&self) -> Result<BearerToken, super::Error>;

    /// Subscribes to token refresh events.
    ///
    /// Returns a new receiver that will be notified whenever the token
    /// is refreshed. Each call creates an independent subscription.
    /// The receiver always contains the latest token value (or `None`
    /// if no token has been acquired yet).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let auth = effect_handler.get_extension::<dyn BearerTokenProvider>("auth")?;
    /// let mut token_rx = auth.subscribe_token_refresh();
    ///
    /// loop {
    ///     tokio::select! {
    ///         _ = token_rx.changed() => {
    ///             if let Some(token) = token_rx.borrow().as_ref() {
    ///                 // Update headers, etc.
    ///             }
    ///         }
    ///         // ... other branches
    ///     }
    /// }
    /// ```
    fn subscribe_token_refresh(&self) -> tokio::sync::watch::Receiver<Option<BearerToken>>;
}
