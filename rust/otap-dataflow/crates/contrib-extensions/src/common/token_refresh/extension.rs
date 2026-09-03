// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bearer token provider extension built on the shared background refresh machinery.
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use otel_arrow_dfe_engine::capability::auth::BearerToken;
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::BearerTokenProvider as BearerTokenProviderCap;
use otel_arrow_dfe_engine::capability::{
    CapabilityError, auth::bearer_token_provider::TokenStream,
};
use otel_arrow_dfe_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider as SharedBearerTokenProvider;
use tokio_stream::wrappers::WatchStream;

use crate::common::background_refresh::{
    BackgroundProviderExtension, BackgroundProviderMetrics, BackgroundProviderSource,
};

/// The Bearer Token Provider extension.
pub type TokenProviderExtension<S, M> =
    BackgroundProviderExtension<S, M, BearerToken, BearerTokenProviderCap>;

/// Next-refresh delay used for non-expiring tokens (~1 year). The loop is still
/// woken by control messages in the meantime.
pub const NON_EXPIRING_TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(365 * 24 * 60 * 60);

#[async_trait]
impl<S: BackgroundProviderSource<BearerToken>, M: BackgroundProviderMetrics>
    SharedBearerTokenProvider for TokenProviderExtension<S, M>
{
    async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
        self.get_value().await
    }

    fn token_stream(&self) -> TokenStream {
        let rx = self.subscribe();
        // Yield the current cached value immediately, then each refresh. The
        // initial `None` (and any future `None`) is filtered out. The stream
        // item is a plain `BearerToken`: a refresh failure does not terminate
        // the subscription, it simply does not emit until the next success.
        let stream = WatchStream::new(rx).filter_map(|opt| async move { opt });
        Box::pin(stream)
    }
}
