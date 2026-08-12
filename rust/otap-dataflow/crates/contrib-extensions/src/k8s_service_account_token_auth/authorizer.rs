// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The two capability variants of Kubernetes service-account-token auth.
//!
//! - [`SharedK8sServiceAccountTokenAuth`]: `Send`, `Arc` handle over a
//!   [`SharedDecisionCache`].
//! - [`LocalK8sServiceAccountTokenAuth`]: `!Send`, `Rc` handle over a
//!   [`LocalDecisionCache`], so thread-per-core consumers pay for neither a lock
//!   nor atomic refcounts.
//!
//! Each calls its own cache directly. Both use [`Core`], which reaches the
//! security decision on a miss.

use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use otap_df_engine::capability::CapabilityError;
use otap_df_engine::capability::auth::{AuthzDecision, BearerToken};
use otap_df_engine::local::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as LocalBearerTokenAuthorizer;
use otap_df_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as SharedBearerTokenAuthorizer;
use otap_df_telemetry::otel_debug;

use super::cache::{LocalDecisionCache, SharedDecisionCache, digest};
use super::config::AudienceConfig;
#[cfg(test)]
use super::config::DEFAULT_REVIEW_TIMEOUT;
use super::core::Core;

// -- Shared variant (Send; Arc + Mutex) -------------------------------------

/// Shared, `Send` Kubernetes service-account-token auth provider.
///
/// Every clone shares the same [`SharedInner`] via `Arc`, so they share one
/// lazily-built Kubernetes client and one decision cache.
#[derive(Clone)]
pub(crate) struct SharedK8sServiceAccountTokenAuth {
    inner: Arc<SharedInner>,
}

/// State shared across clones of the shared variant.
struct SharedInner {
    /// Authenticate + admit logic and lazily-built client.
    core: Core,
    /// Cross-thread decision cache guarded by a `Mutex`.
    cache: SharedDecisionCache,
}

impl SharedK8sServiceAccountTokenAuth {
    /// Builds a new shared-variant instance.
    #[cfg(test)]
    pub(crate) fn new(
        name: &str,
        audiences: Vec<AudienceConfig>,
        cache_ttl: Duration,
        cache_max_entries: usize,
    ) -> Self {
        Self::new_with_timeout(
            name,
            audiences,
            cache_ttl,
            cache_max_entries,
            DEFAULT_REVIEW_TIMEOUT,
        )
    }

    /// Builds a new shared-variant instance with an explicit review timeout.
    pub(crate) fn new_with_timeout(
        name: &str,
        audiences: Vec<AudienceConfig>,
        cache_ttl: Duration,
        cache_max_entries: usize,
        review_timeout: Duration,
    ) -> Self {
        Self {
            inner: Arc::new(SharedInner {
                core: Core::new_with_timeout(name, audiences, review_timeout),
                cache: SharedDecisionCache::new(cache_ttl, cache_max_entries),
            }),
        }
    }
}

#[async_trait]
impl SharedBearerTokenAuthorizer for SharedK8sServiceAccountTokenAuth {
    async fn authorize(&self, credential: &BearerToken) -> Result<AuthzDecision, CapabilityError> {
        let token = credential.expose_token();

        // An empty credential is a missing credential (401); never hash it,
        // cache it, or round-trip it to the API server.
        if token.is_empty() {
            otel_debug!("k8s_service_account_token_auth.credential_missing");
            return Ok(Core::missing());
        }

        let inner = self.inner.as_ref();
        inner
            .cache
            .get_or_decide(digest(token), || inner.core.decide(token))
            .await
            .map_err(|error| inner.core.capability_error(error))
    }
}

// -- Local variant (!Send; Rc + RefCell) ------------------------------------

/// Local, `!Send` Kubernetes service-account-token auth provider.
///
/// Every clone on a core shares the same [`LocalInner`] via `Rc`. Each core gets
/// its own instance and hence its own cache, consistent with the engine's
/// thread-per-core model.
#[derive(Clone)]
pub(crate) struct LocalK8sServiceAccountTokenAuth {
    inner: Rc<LocalInner>,
}

/// State shared across clones of the local variant.
struct LocalInner {
    /// Authenticate + admit logic and lazily-built client.
    core: Core,
    /// Thread-per-core decision cache.
    cache: LocalDecisionCache,
}

impl LocalK8sServiceAccountTokenAuth {
    /// Builds a new local-variant instance.
    #[cfg(test)]
    pub(crate) fn new(
        name: &str,
        audiences: Vec<AudienceConfig>,
        cache_ttl: Duration,
        cache_max_entries: usize,
    ) -> Self {
        Self::new_with_timeout(
            name,
            audiences,
            cache_ttl,
            cache_max_entries,
            DEFAULT_REVIEW_TIMEOUT,
        )
    }

    /// Builds a new local-variant instance with an explicit review timeout.
    pub(crate) fn new_with_timeout(
        name: &str,
        audiences: Vec<AudienceConfig>,
        cache_ttl: Duration,
        cache_max_entries: usize,
        review_timeout: Duration,
    ) -> Self {
        Self {
            inner: Rc::new(LocalInner {
                core: Core::new_with_timeout(name, audiences, review_timeout),
                cache: LocalDecisionCache::new(cache_ttl, cache_max_entries),
            }),
        }
    }
}

#[async_trait(?Send)]
impl LocalBearerTokenAuthorizer for LocalK8sServiceAccountTokenAuth {
    async fn authorize(&self, credential: &BearerToken) -> Result<AuthzDecision, CapabilityError> {
        let token = credential.expose_token();

        // An empty credential is a missing credential (401); never hash it,
        // cache it, or round-trip it to the API server.
        if token.is_empty() {
            otel_debug!("k8s_service_account_token_auth.credential_missing");
            return Ok(Core::missing());
        }

        let inner = self.inner.as_ref();
        inner
            .cache
            .get_or_decide(digest(token), || inner.core.decide(token))
            .await
            .map_err(|error| inner.core.capability_error(error))
    }
}
