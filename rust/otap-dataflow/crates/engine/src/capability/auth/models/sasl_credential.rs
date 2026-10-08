// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The shared [`SaslCredential`].

use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use std::time::Instant;

/// A SASL credential error.
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, Clone)]
pub enum SaslCredentialError {
    #[error("Username is invalid: {reason}")]
    InvalidUsername { reason: &'static str },

    #[error("Password is invalid: {reason}")]
    InvalidPassword { reason: &'static str },
}

/// A username/password credential for SASL authentication.
///
/// The credential is wrapped in [`SecretString`]s, which zeroize on drop and
/// mask themselves in [`Debug`] output. The values sit behind [`Arc`] so
/// cloning a credential is a cheap refcount bump rather than a copy of the
/// plaintext allocations.
///
/// `expires_on` is a monotonic [`Instant`]. `None` means no known expiry.
/// Mechanism-specific validation belongs to the consumer because PLAIN and
/// SCRAM impose different requirements.
#[derive(Clone, Debug)]
pub struct SaslCredential {
    username: Arc<SecretString>,
    password: Arc<SecretString>,
    expires_on: Option<Instant>,
}

impl SaslCredential {
    /// Creates a non-empty SASL username/password credential.
    pub fn new(
        username: impl Into<SecretString>,
        password: impl Into<SecretString>,
    ) -> Result<Self, SaslCredentialError> {
        let username: SecretString = username.into();
        if username.expose_secret().is_empty() {
            return Err(SaslCredentialError::InvalidUsername {
                reason: "Username cannot be empty",
            });
        }

        let password: SecretString = password.into();
        if password.expose_secret().is_empty() {
            return Err(SaslCredentialError::InvalidPassword {
                reason: "Password cannot be empty",
            });
        }

        Ok(Self {
            username: Arc::new(username),
            password: Arc::new(password),
            expires_on: None,
        })
    }

    /// Adds expiry to a credential.
    #[must_use]
    pub const fn with_expiry(mut self, expires_on: Instant) -> Self {
        self.expires_on = Some(expires_on);
        self
    }

    /// Exposes the SASL username.
    ///
    /// Named `expose_username` so every plaintext access is explicit and
    /// greppable.
    #[must_use]
    pub fn expose_username(&self) -> &str {
        self.username.expose_secret()
    }

    /// Exposes the SASL password.
    ///
    /// Named `expose_password` so every plaintext access is explicit and
    /// greppable.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Scenario: A provider creates a SASL credential with non-empty values.
    /// Guarantees: Consumers can explicitly expose the original username and password.
    #[test]
    fn creates_non_empty_credential() {
        let credential = SaslCredential::new("user", "password").expect("credential is valid");

        assert_eq!(credential.expose_username(), "user");
        assert_eq!(credential.expose_password(), "password");
    }

    /// Scenario: A provider supplies an empty username or password.
    /// Guarantees: Invalid empty credential values are rejected before publication.
    #[test]
    fn rejects_empty_values() {
        assert!(matches!(
            SaslCredential::new("", "password"),
            Err(SaslCredentialError::InvalidUsername { .. })
        ));
        assert!(matches!(
            SaslCredential::new("user", ""),
            Err(SaslCredentialError::InvalidPassword { .. })
        ));
    }

    /// Scenario: A SASL credential is formatted for debug output.
    /// Guarantees: Neither plaintext credential value appears in diagnostics.
    #[test]
    fn debug_output_redacts_values() {
        let credential =
            SaslCredential::new("sasl-user", "sasl-password").expect("credential is valid");
        let debug = format!("{credential:?}");

        assert!(!debug.contains("sasl-user"));
        assert!(!debug.contains("sasl-password"));
    }

    /// Scenario: A provider attaches a known expiration instant to a credential.
    /// Guarantees: Consumers observe the exact monotonic expiration value.
    #[test]
    fn preserves_expiry() {
        let expires_on = Instant::now() + Duration::from_secs(60);
        let credential = SaslCredential::new("user", "password")
            .expect("credential is valid")
            .with_expiry(expires_on);

        assert_eq!(credential.expires_on(), Some(expires_on));
    }
}
