// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The shared [`BasicAuthCredential`].

use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use std::time::Instant;

/// A Basic Auth Credential.
///
/// The credential is wrapped in [`SecretString`]s, which zeroizes on drop and
/// masks itself in [`Debug`] output, so it cannot leak into logs or telemetry.
/// The `SecretString`s sit behind an [`Arc`] so cloning a credential (handing
/// it to multiple subscribers, or returning it from `get_credential` on the hot
/// path) is a cheap refcount bump that shares one plaintext allocation rather
/// than copying the secret bytes.
///
/// `expires_on` is a monotonic [`Instant`] -- an absolute wall-clock expiry is
/// converted to an `Instant` once, so the value is immune to wall-clock jumps
/// thereafter. `None` means no known expiry. The credential is opaque to this
/// type: an expiry is only ever what a caller supplies from the issuer's
/// response metadata, never parsed out of the credential itself.
#[derive(Clone, Debug)]
pub struct BasicAuthCredential {
    username: Arc<SecretString>,
    password: Arc<SecretString>,
    expires_on: Option<Instant>,
}

impl BasicAuthCredential {
    /// Creates a credential.
    #[must_use]
    pub fn new(
        username: impl Into<SecretString>,
        password: impl Into<SecretString>,
    ) -> Self {
        Self {
            username: Arc::new(username.into()),
            password: Arc::new(password.into()),
            expires_on: None,
        }
    }

    /// Adds expiry to a credential.
    #[must_use]
    pub const fn with_expiry(mut self, expires_on: Instant) -> Self {
        self.expires_on = Some(expires_on);
        self
    }

    /// Exposes the credential username secret, for the authorizer to validate
    /// or for injection into an `Authorization` header.
    ///
    /// Named `expose_username` (rather than a plain getter) so every plaintext
    /// access is explicit and greppable.
    #[must_use]
    pub fn expose_username(&self) -> &str {
        self.username.expose_secret()
    }

    /// Exposes the credential password secret, for the authorizer to validate
    /// or for injection into an `Authorization` header.
    ///
    /// Named `expose_password` (rather than a plain getter) so every plaintext
    /// access is explicit and greppable.
    #[must_use]
    pub fn expose_password(&self) -> &str {
        self.password.expose_secret()
    }

    /// The monotonic instant at which this credential expires, if known.
    #[must_use]
    pub const fn expires_on(&self) -> Option<Instant> {
        self.expires_on
    }
}