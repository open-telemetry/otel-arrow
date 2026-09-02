// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bearer token provider extension built on the shared background refresh machinery.
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

#[async_trait]
impl<S: BackgroundProviderSource<BearerToken>, M: BackgroundProviderMetrics>
    SharedBearerTokenProvider for TokenProviderExtension<S, M>
{
    async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
        // Fast path: lock-free read of the watch cache.
        if let Some(token) = self.current_fresh_value() {
            return Ok(token);
        }

        // Slow path: coalesce concurrent cache-miss callers onto a single
        // in-flight token call, with a double-check after acquiring the lock.
        let _guard = self.acquire_fetch_lock().await;
        if let Some(token) = self.current_fresh_value() {
            return Ok(token);
        }
        // Negative cache: if the most recent acquisition failed within the
        // cooldown window, surface the throttle instead of hitting the token
        // endpoint again. The background loop keeps retrying on its own cadence.
        if self.recently_failed() {
            return Err(self.capability_error("token acquisition throttled after recent failure"));
        }
        self.refresh_once()
            .await
            .map_err(|err| self.capability_error(err))
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
