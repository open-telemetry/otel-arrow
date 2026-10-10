// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The `SaslCredentialProvider` capability.
//!
//! This purpose-built capability supplies username/password credential
//! material for SASL mechanisms. Consumers retain ownership of mechanism
//! selection and mechanism-specific validation.

use crate::capability::auth::SaslCredential;
use crate::capability::error::CapabilityError;
use futures::Stream;
use otel_arrow_dfe_engine_macros::capability;
use std::pin::Pin;
use std::time::Duration;

/// How close to [`SaslCredential::expires_on`] a credential stops being usable.
///
/// Providers must publish a replacement before the current credential enters
/// this margin. Consumers must stop using a credential inside the margin so a
/// connection attempt cannot outlive it in the presence of clock skew.
pub const SASL_CREDENTIAL_USABLE_MARGIN: Duration = Duration::from_secs(30);

/// A per-consumer subscription to SASL credential refreshes.
///
/// A refresh failure does not terminate the stream. The provider emits the next
/// successfully acquired credential when one becomes available.
pub type SaslCredentialStream = Pin<Box<dyn Stream<Item = SaslCredential> + 'static>>;

/// Provides username/password credentials for SASL authentication.
#[capability(
    name = "sasl_credential_provider",
    description = "Provides SASL username/password credentials, refreshed in the background"
)]
pub trait SaslCredentialProvider {
    /// Returns the current usable credential.
    ///
    /// Providers may serve a cached credential or acquire one on demand.
    /// Mechanism selection and mechanism-specific validation remain the
    /// consumer's responsibility.
    async fn get_credential(&self) -> Result<SaslCredential, CapabilityError>;

    /// Subscribes to credential refreshes.
    ///
    /// Each call returns an independent subscription. A subscription created
    /// after a credential has been published MUST immediately yield the current
    /// credential, then yield replacements as they are published.
    fn credential_stream(&self) -> SaslCredentialStream;
}
