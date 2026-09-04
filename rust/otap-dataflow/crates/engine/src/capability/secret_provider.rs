// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The `Secret` capability.

use std::{pin::Pin, sync::Arc, time::Instant};

use futures::Stream;
use otap_df_engine_macros::capability;
use secrecy::{ExposeSecret, SecretString};

use crate::capability::CapabilityError;

/// A per-consumer subscription to secret refreshes.
pub type SecretStream = Pin<Box<dyn Stream<Item = Secret> + 'static>>;

/// Hands out secrets to data-path nodes.
#[capability(
    name = "secret_provider",
    description = "Provides secrets, refreshed in the background"
)]
pub trait SecretProvider {
    /// Returns a secret for the provider's configured scope(s).
    async fn get_secret(&self, name: &str) -> Result<Secret, CapabilityError>;

    /// Subscribes to the stream of secret refreshes.
    fn secret_stream(&self, name: &str) -> SecretStream;
}

/// A secret.
pub struct Secret {
    secret: Arc<SecretString>,
    expires_on: Option<Instant>,
}

impl Secret {
    /// Creates a secret with **no known expiry**.
    #[must_use]
    pub fn without_expiry(secret: impl Into<SecretString>) -> Self {
        Self {
            secret: Arc::new(secret.into()),
            expires_on: None,
        }
    }

    /// Creates a secret with an explicit optional monotonic expiry.
    #[must_use]
    pub fn with_expiry(secret: impl Into<SecretString>, expires_on: Option<Instant>) -> Self {
        Self {
            secret: Arc::new(secret.into()),
            expires_on,
        }
    }

    /// Exposes the secret.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        self.secret.expose_secret()
    }

    /// The monotonic instant at which this secret expires, if known.
    #[must_use]
    pub const fn expires_on(&self) -> Option<Instant> {
        self.expires_on
    }
}
