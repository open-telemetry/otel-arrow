// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime error surface for capability method calls.
//!
//! [`CapabilityError`] is the error a capability method returns when an
//! *invocation* fails at run time (e.g. a `BearerTokenProvider` cannot
//! acquire a token). It is distinct from the engine's resolution/
//! registration errors ([`crate::error::Error::CapabilityNotBound`],
//! [`crate::error::Error::CapabilityAlreadyConsumed`]), which surface at
//! *wiring* time when a node binds a capability.
//!
//! Every `CapabilityError` carries the `(extension, capability)` identity
//! of the failing provider plus the underlying source error.
//! [`CapabilityErrorSource`] is the small, capability-typed helper an
//! extension stores once and uses to stamp that identity onto each error
//! it mints, so the capability name is taken from the type system rather
//! than re-typed at every call site.

use super::ExtensionCapability;
use otap_df_config::ExtensionId;
use std::marker::PhantomData;

/// The error returned by a failed capability method invocation.
///
/// Carries the identity of the provider that failed — the extension
/// instance id and the capability name — alongside the underlying
/// `source` error, so a consumer can attribute the failure without
/// knowing the provider's internal error types.
#[derive(Debug, thiserror::Error)]
#[error("capability '{capability}' on extension '{extension}' failed: {source}")]
pub struct CapabilityError {
    /// The extension instance whose capability invocation failed.
    pub extension: ExtensionId,
    /// The capability name (e.g. `"bearer_token_provider"`).
    pub capability: &'static str,
    /// The underlying error that caused the failure.
    #[source]
    pub source: Box<dyn std::error::Error + Send + Sync + 'static>,
}

/// A capability-typed factory for [`CapabilityError`]s.
///
/// An extension constructs one of these once (at `create()` time) with
/// its own [`ExtensionId`] and stores it. Each call to [`error`](Self::error)
/// stamps the stored extension id and the capability name — read from
/// `C::NAME` via the [`ExtensionCapability`] type — onto a fresh
/// `CapabilityError`. This keeps the capability name a compile-time fact
/// rather than a string repeated at every failure site.
///
/// `C` appears only in [`PhantomData`], so the source is `Send + Sync +
/// Clone` regardless of `C`.
pub struct CapabilityErrorSource<C: ExtensionCapability> {
    extension: ExtensionId,
    _capability: PhantomData<fn() -> C>,
}

impl<C: ExtensionCapability> CapabilityErrorSource<C> {
    /// Creates a source that stamps `extension` and `C`'s capability name
    /// onto every error it mints.
    #[must_use]
    pub fn new(extension: ExtensionId) -> Self {
        Self {
            extension,
            _capability: PhantomData,
        }
    }

    /// Mints a [`CapabilityError`] wrapping `source`, tagged with the
    /// stored extension id and `C`'s capability name.
    pub fn error<E>(&self, source: E) -> CapabilityError
    where
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        CapabilityError {
            extension: self.extension.clone(),
            capability: C::NAME,
            source: source.into(),
        }
    }
}

// Manual `Clone`: deriving would add a `C: Clone` bound, but `C` is only
// a `PhantomData<fn() -> C>` marker and never cloned, so the source is
// always cloneable.
impl<C: ExtensionCapability> Clone for CapabilityErrorSource<C> {
    fn clone(&self) -> Self {
        Self {
            extension: self.extension.clone(),
            _capability: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::BearerTokenProvider;

    #[test]
    fn stamps_extension_and_capability_identity() {
        let source = CapabilityErrorSource::<BearerTokenProvider>::new("azure_identity".into());
        let err = source.error("boom");
        assert_eq!(err.extension, "azure_identity");
        assert_eq!(err.capability, "bearer_token_provider");
        assert_eq!(err.source.to_string(), "boom");
    }

    #[test]
    fn clone_preserves_identity_without_c_clone_bound() {
        let source = CapabilityErrorSource::<BearerTokenProvider>::new("ext-a".into());
        let cloned = source.clone();
        assert_eq!(cloned.error("x").extension, "ext-a");
    }
}
