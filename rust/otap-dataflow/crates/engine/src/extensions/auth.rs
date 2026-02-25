// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait-based authentication handles for extensions.
//!
//! This module defines a generic, pluggable authentication contract:
//!
//! - [`ServerAuthenticator`] + [`ServerAuthenticatorHandle`] — for **receivers**
//!   that need to validate credentials on incoming requests.
//! - [`ClientAuthenticator`] + [`ClientAuthenticatorHandle`] — for **exporters**
//!   that need to attach credentials to outgoing requests.
//!
//! These are the only auth types that receivers and exporters need to know about.
//! Concrete auth strategies (bearer tokens, API keys, OIDC, etc.) implement
//! the traits and are selected purely through configuration.
//!
//! # Examples
//!
//! ## Extension factory — registering both handles
//!
//! ```rust,ignore
//! let auth = MyAuthImpl { /* ... */ };
//!
//! let mut handles = ExtensionHandles::new();
//! handles.register(ServerAuthenticatorHandle::new(auth.clone()));
//! handles.register(ClientAuthenticatorHandle::new(auth));
//! ```
//!
//! ## Receiver — validating incoming requests
//!
//! ```rust,ignore
//! let auth = extension_registry
//!     .get::<ServerAuthenticatorHandle>("my_auth")?;
//!
//! // In the gRPC/HTTP handler:
//! auth.authenticate(request.headers())?;
//! ```
//!
//! ## Exporter — attaching outgoing credentials
//!
//! ```rust,ignore
//! let auth = extension_registry
//!     .get::<ClientAuthenticatorHandle>("my_auth")?;
//!
//! for (key, value) in auth.get_request_metadata()? {
//!     request.headers_mut().insert(key, value);
//! }
//! ```

use std::fmt;
use std::sync::{Arc, Mutex};

/// An error returned by authenticator operations.
#[derive(Debug, Clone)]
pub struct AuthError {
    /// A human-readable description of the authentication failure.
    pub message: String,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "authentication error: {}", self.message)
    }
}

impl std::error::Error for AuthError {}

// ─── Server side (receivers validate incoming requests) ────────────────────

/// Trait for validating credentials on incoming requests.
///
/// Implement this trait in an auth extension to provide server-side
/// authentication. Different strategies (bearer token validation, API key
/// allow-lists, OIDC token verification) all implement this same interface.
///
/// Receivers call [`ServerAuthenticatorHandle::authenticate`] without knowing
/// which concrete strategy is behind it — swapping auth is a config change.
pub trait ServerAuthenticator: Send {
    /// Validates the request headers.
    ///
    /// Returns `Ok(())` if the request is authenticated, or an [`AuthError`]
    /// describing why authentication failed.
    fn authenticate(&self, headers: &http::HeaderMap) -> Result<(), AuthError>;
}

/// A cloneable handle that receivers use to authenticate incoming requests.
///
/// This wraps any [`ServerAuthenticator`] behind an `Arc<Mutex<…>>` so that
/// each receiver gets its own clone. The `Mutex` makes the handle `Sync`
/// (required by tonic services) without requiring `Sync` on the trait itself.
/// The lock is never contended because the engine uses a thread-per-core
/// architecture in both local and shared modes.
#[derive(Clone)]
pub struct ServerAuthenticatorHandle {
    inner: Arc<Mutex<Box<dyn ServerAuthenticator>>>,
}

impl ServerAuthenticatorHandle {
    /// Creates a new handle wrapping the given authenticator implementation.
    pub fn new(auth: impl ServerAuthenticator + 'static) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Box::new(auth))),
        }
    }

    /// Validates the request headers.
    ///
    /// Delegates to the underlying [`ServerAuthenticator`] implementation.
    pub fn authenticate(&self, headers: &http::HeaderMap) -> Result<(), AuthError> {
        self.inner
            .lock()
            .expect("ServerAuthenticator lock poisoned")
            .authenticate(headers)
    }
}

impl fmt::Debug for ServerAuthenticatorHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerAuthenticatorHandle").finish()
    }
}

// ─── Client side (exporters attach outgoing credentials) ───────────────────

/// Trait for producing credentials to attach to outgoing requests.
///
/// Implement this trait in an auth extension to provide client-side
/// authentication. The extension decides what headers to attach
/// (e.g., `Authorization: Bearer <token>`, custom API key headers).
pub trait ClientAuthenticator: Send {
    /// Returns the headers to attach to an outgoing request.
    ///
    /// Each entry is a `(header_name, header_value)` pair. The exporter
    /// inserts them into the request's header map before sending.
    ///
    /// # Errors
    ///
    /// Returns an [`AuthError`] if credentials are unavailable
    /// (e.g., token not yet refreshed, provider unreachable).
    fn get_request_metadata(&self)
    -> Result<Vec<(http::HeaderName, http::HeaderValue)>, AuthError>;
}

/// A cloneable handle that exporters use to attach credentials to outgoing requests.
///
/// This wraps any [`ClientAuthenticator`] behind an `Arc<Mutex<…>>` so that
/// each exporter gets its own clone. See [`ServerAuthenticatorHandle`] for
/// the rationale behind the `Mutex` wrapper.
#[derive(Clone)]
pub struct ClientAuthenticatorHandle {
    inner: Arc<Mutex<Box<dyn ClientAuthenticator>>>,
}

impl ClientAuthenticatorHandle {
    /// Creates a new handle wrapping the given authenticator implementation.
    pub fn new(auth: impl ClientAuthenticator + 'static) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Box::new(auth))),
        }
    }

    /// Returns the headers to attach to an outgoing request.
    ///
    /// Delegates to the underlying [`ClientAuthenticator`] implementation.
    pub fn get_request_metadata(
        &self,
    ) -> Result<Vec<(http::HeaderName, http::HeaderValue)>, AuthError> {
        self.inner
            .lock()
            .expect("ClientAuthenticator lock poisoned")
            .get_request_metadata()
    }
}

impl fmt::Debug for ClientAuthenticatorHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientAuthenticatorHandle").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trivial static bearer token authenticator that implements both traits.
    struct StaticBearerAuth {
        token: String,
    }

    impl ServerAuthenticator for StaticBearerAuth {
        fn authenticate(&self, headers: &http::HeaderMap) -> Result<(), AuthError> {
            let auth_value = headers
                .get(http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| AuthError {
                    message: "missing Authorization header".into(),
                })?;

            let expected = format!("Bearer {}", self.token);
            if auth_value != expected {
                return Err(AuthError {
                    message: "invalid bearer token".into(),
                });
            }
            Ok(())
        }
    }

    impl ClientAuthenticator for StaticBearerAuth {
        fn get_request_metadata(
            &self,
        ) -> Result<Vec<(http::HeaderName, http::HeaderValue)>, AuthError> {
            Ok(vec![(
                http::header::AUTHORIZATION,
                http::HeaderValue::from_str(&format!("Bearer {}", self.token)).map_err(|e| {
                    AuthError {
                        message: e.to_string(),
                    }
                })?,
            )])
        }
    }

    #[test]
    fn server_auth_valid_token() {
        let auth = ServerAuthenticatorHandle::new(StaticBearerAuth {
            token: "secret123".into(),
        });

        let mut headers = http::HeaderMap::new();
        let _ = headers.insert(
            http::header::AUTHORIZATION,
            "Bearer secret123".parse().unwrap(),
        );
        assert!(auth.authenticate(&headers).is_ok());
    }

    #[test]
    fn server_auth_invalid_token() {
        let auth = ServerAuthenticatorHandle::new(StaticBearerAuth {
            token: "secret123".into(),
        });

        let mut headers = http::HeaderMap::new();
        let _ = headers.insert(http::header::AUTHORIZATION, "Bearer wrong".parse().unwrap());
        let err = auth.authenticate(&headers).unwrap_err();
        assert!(err.message.contains("invalid"));
    }

    #[test]
    fn server_auth_missing_header() {
        let auth = ServerAuthenticatorHandle::new(StaticBearerAuth {
            token: "secret123".into(),
        });

        let headers = http::HeaderMap::new();
        let err = auth.authenticate(&headers).unwrap_err();
        assert!(err.message.contains("missing"));
    }

    #[test]
    fn client_auth_produces_metadata() {
        let auth = ClientAuthenticatorHandle::new(StaticBearerAuth {
            token: "mytoken".into(),
        });

        let metadata = auth.get_request_metadata().unwrap();
        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata[0].0, http::header::AUTHORIZATION);
        assert_eq!(metadata[0].1, "Bearer mytoken");
    }

    #[test]
    fn separate_handles_from_same_type() {
        let server = ServerAuthenticatorHandle::new(StaticBearerAuth {
            token: "shared".into(),
        });
        let client = ClientAuthenticatorHandle::new(StaticBearerAuth {
            token: "shared".into(),
        });

        // Server side validates
        let mut headers = http::HeaderMap::new();
        let _ = headers.insert(
            http::header::AUTHORIZATION,
            "Bearer shared".parse().unwrap(),
        );
        assert!(server.authenticate(&headers).is_ok());

        // Client side produces the same token
        let metadata = client.get_request_metadata().unwrap();
        assert_eq!(metadata[0].1, "Bearer shared");
    }

    /// End-to-end scenario tests demonstrating realistic auth extension
    /// patterns: a receiver-side header allow-list and an exporter-side
    /// token refresher backed by `tokio::sync::watch`.
    mod scenario_tests {
        use super::*;
        use crate::extensions::{ExtensionHandles, ExtensionRegistryBuilder};

        // ─── Scenario 1: Header allow-list (receiver-side) ────────────

        /// A server authenticator that checks a specific header is present
        /// and its value belongs to a known allow-list.
        struct HeaderAllowListAuth {
            header_name: http::HeaderName,
            allowed_values: Vec<String>,
        }

        impl ServerAuthenticator for HeaderAllowListAuth {
            fn authenticate(&self, headers: &http::HeaderMap) -> Result<(), AuthError> {
                let value = headers
                    .get(&self.header_name)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| AuthError {
                        message: format!("missing required header: {}", self.header_name),
                    })?;

                if !self.allowed_values.iter().any(|allowed| allowed == value) {
                    return Err(AuthError {
                        message: format!("header value '{}' is not in the allow-list", value),
                    });
                }
                Ok(())
            }
        }

        #[test]
        fn header_allowlist_valid_value() {
            let auth = ServerAuthenticatorHandle::new(HeaderAllowListAuth {
                header_name: http::HeaderName::from_static("x-tenant-id"),
                allowed_values: vec!["tenant-a".into(), "tenant-b".into()],
            });

            let mut headers = http::HeaderMap::new();
            let _ = headers.insert("x-tenant-id", "tenant-a".parse().unwrap());
            assert!(auth.authenticate(&headers).is_ok());
        }

        #[test]
        fn header_allowlist_invalid_value() {
            let auth = ServerAuthenticatorHandle::new(HeaderAllowListAuth {
                header_name: http::HeaderName::from_static("x-tenant-id"),
                allowed_values: vec!["tenant-a".into(), "tenant-b".into()],
            });

            let mut headers = http::HeaderMap::new();
            let _ = headers.insert("x-tenant-id", "tenant-unknown".parse().unwrap());
            let err = auth.authenticate(&headers).unwrap_err();
            assert!(err.message.contains("not in the allow-list"));
        }

        #[test]
        fn header_allowlist_missing_header() {
            let auth = ServerAuthenticatorHandle::new(HeaderAllowListAuth {
                header_name: http::HeaderName::from_static("x-tenant-id"),
                allowed_values: vec!["tenant-a".into()],
            });

            let headers = http::HeaderMap::new();
            let err = auth.authenticate(&headers).unwrap_err();
            assert!(err.message.contains("missing required header"));
        }

        #[test]
        fn header_allowlist_via_registry() {
            let auth = HeaderAllowListAuth {
                header_name: http::HeaderName::from_static("x-tenant-id"),
                allowed_values: vec!["tenant-a".into(), "tenant-b".into()],
            };

            // Extension factory registers the handle
            let mut handles = ExtensionHandles::new();
            handles.register(ServerAuthenticatorHandle::new(auth));

            let mut builder = ExtensionRegistryBuilder::new();
            builder.merge("header_allowlist", handles).unwrap();
            let registry = builder.build();

            // Receiver retrieves it by name + type at startup
            let handle = registry
                .get::<ServerAuthenticatorHandle>("header_allowlist")
                .unwrap();

            let mut headers = http::HeaderMap::new();
            let _ = headers.insert("x-tenant-id", "tenant-b".parse().unwrap());
            assert!(handle.authenticate(&headers).is_ok());

            let _ = headers.insert("x-tenant-id", "tenant-c".parse().unwrap());
            assert!(handle.authenticate(&headers).is_err());
        }

        // ─── Scenario 2: Watch-based bearer token (exporter-side) ─────

        /// A client authenticator backed by a `tokio::sync::watch::Receiver`.
        /// The extension task owns the `Sender` and refreshes the token
        /// periodically; each exporter clone reads the latest value.
        struct WatchBearerAuth {
            rx: tokio::sync::watch::Receiver<String>,
        }

        impl ClientAuthenticator for WatchBearerAuth {
            fn get_request_metadata(
                &self,
            ) -> Result<Vec<(http::HeaderName, http::HeaderValue)>, AuthError> {
                let token = self.rx.borrow().clone();
                if token.is_empty() {
                    return Err(AuthError {
                        message: "token not yet available".into(),
                    });
                }
                Ok(vec![(
                    http::header::AUTHORIZATION,
                    http::HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| {
                        AuthError {
                            message: e.to_string(),
                        }
                    })?,
                )])
            }
        }

        #[test]
        fn watch_bearer_initial_empty_token_fails() {
            let (_tx, rx) = tokio::sync::watch::channel(String::new());
            let auth = ClientAuthenticatorHandle::new(WatchBearerAuth { rx });

            let err = auth.get_request_metadata().unwrap_err();
            assert!(err.message.contains("not yet available"));
        }

        #[test]
        fn watch_bearer_returns_current_token() {
            let (tx, rx) = tokio::sync::watch::channel("initial-token".to_string());
            let auth = ClientAuthenticatorHandle::new(WatchBearerAuth { rx });

            let metadata = auth.get_request_metadata().unwrap();
            assert_eq!(metadata[0].1, "Bearer initial-token");

            // Simulate token refresh
            tx.send("refreshed-token".to_string()).unwrap();

            let metadata = auth.get_request_metadata().unwrap();
            assert_eq!(metadata[0].1, "Bearer refreshed-token");
        }

        #[test]
        fn watch_bearer_cloned_handle_sees_updates() {
            let (tx, rx) = tokio::sync::watch::channel("v1".to_string());
            let auth = ClientAuthenticatorHandle::new(WatchBearerAuth { rx });

            // Clone the handle (simulating multiple exporter instances)
            let auth2 = auth.clone();

            let m1 = auth.get_request_metadata().unwrap();
            let m2 = auth2.get_request_metadata().unwrap();
            assert_eq!(m1[0].1, "Bearer v1");
            assert_eq!(m2[0].1, "Bearer v1");

            // Refresh token — both clones see the update
            tx.send("v2".to_string()).unwrap();

            let m1 = auth.get_request_metadata().unwrap();
            let m2 = auth2.get_request_metadata().unwrap();
            assert_eq!(m1[0].1, "Bearer v2");
            assert_eq!(m2[0].1, "Bearer v2");
        }

        #[test]
        fn watch_bearer_via_registry() {
            let (tx, rx) = tokio::sync::watch::channel("tok-abc".to_string());
            let auth = WatchBearerAuth { rx };

            // Extension factory registers the handle
            let mut handles = ExtensionHandles::new();
            handles.register(ClientAuthenticatorHandle::new(auth));

            let mut builder = ExtensionRegistryBuilder::new();
            builder.merge("token_refresher", handles).unwrap();
            let registry = builder.build();

            // Exporter retrieves it by name + type at startup
            let handle = registry
                .get::<ClientAuthenticatorHandle>("token_refresher")
                .unwrap();

            let metadata = handle.get_request_metadata().unwrap();
            assert_eq!(metadata[0].1, "Bearer tok-abc");

            // Token refresh propagates through the registry-retrieved handle
            tx.send("tok-xyz".to_string()).unwrap();
            let metadata = handle.get_request_metadata().unwrap();
            assert_eq!(metadata[0].1, "Bearer tok-xyz");
        }
    }
}
