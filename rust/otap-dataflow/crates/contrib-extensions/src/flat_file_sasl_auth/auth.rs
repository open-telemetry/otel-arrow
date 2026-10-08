// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Immutable SASL credential provider.

use async_trait::async_trait;
use futures::{StreamExt, stream};
use otel_arrow_dfe_engine::capability::CapabilityError;
use otel_arrow_dfe_engine::capability::auth::SaslCredential;
use otel_arrow_dfe_engine::capability::auth::sasl_credential_provider::SaslCredentialStream;
use otel_arrow_dfe_engine::shared::capability::auth::sasl_credential_provider::SaslCredentialProvider;

/// Clones share the credential's immutable secret allocations, as required by
/// the shared capability contract; no locks or background tasks are needed.
#[derive(Clone)]
pub(crate) struct FlatFileSaslAuth {
    pub(crate) credential: SaslCredential,
}

#[async_trait]
impl SaslCredentialProvider for FlatFileSaslAuth {
    async fn get_credential(&self) -> Result<SaslCredential, CapabilityError> {
        Ok(self.credential.clone())
    }

    fn credential_stream(&self) -> SaslCredentialStream {
        Box::pin(stream::iter([self.credential.clone()]).chain(stream::pending()))
    }
}
