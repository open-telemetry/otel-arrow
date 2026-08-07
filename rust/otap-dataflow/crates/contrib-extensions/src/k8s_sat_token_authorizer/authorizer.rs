// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The two capability variants of the Kubernetes SAT authorizer, side by side.
//!
//! Both delegate to [`Core::authorize`](super::core::Core::authorize), which
//! holds the entire request flow; each variant only picks the cache's
//! interior-mutability strategy via [`DecisionStore`](super::cache::DecisionStore):
//!
//! - [`SharedK8sSatTokenAuthorizer`]: `Send`, `Arc` + `Mutex` cache. The shared
//!   capability variant (the shared instance factory requires `Send`).
//! - [`LocalK8sSatTokenAuthorizer`]: `!Send`, `Rc` + lock-free `RefCell` cache.
//!   The local variant, so thread-per-core consumers avoid the `Mutex`.
//!
//! Kept in one file so the pair is obvious; since neither carries request logic
//! they cannot drift, but mirror any change to one wrapper's shape in the other.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use otap_df_engine::capability::CapabilityError;
use otap_df_engine::capability::auth::{AuthzDecision, BearerToken};
use otap_df_engine::local::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as LocalBearerTokenAuthorizer;
use otap_df_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as SharedBearerTokenAuthorizer;

use super::cache::DecisionCache;
use super::config::AudienceConfig;
use super::core::Core;

// -- Shared variant (Send; Arc + Mutex) -------------------------------------

/// Shared, `Send` Kubernetes SAT authorizer.
///
/// Every clone shares the same [`SharedInner`] via `Arc`, so they share one
/// lazily-built Kubernetes client and one decision cache.
#[derive(Clone)]
pub(crate) struct SharedK8sSatTokenAuthorizer {
    inner: Arc<SharedInner>,
}

/// State shared across clones of the shared variant.
struct SharedInner {
    /// Common authenticate + admit logic and lazily-built client.
    core: Core,
    /// Decision cache guarded by a `Mutex` (cross-thread safe).
    cache: Mutex<DecisionCache>,
}

impl SharedK8sSatTokenAuthorizer {
    /// Builds a new shared-variant instance.
    pub(crate) fn new(
        name: &str,
        audiences: Vec<AudienceConfig>,
        cache_ttl: Duration,
        cache_max_entries: usize,
    ) -> Self {
        Self {
            inner: Arc::new(SharedInner {
                core: Core::new(name, audiences),
                cache: Mutex::new(DecisionCache::new(cache_ttl, cache_max_entries)),
            }),
        }
    }
}

#[async_trait]
impl SharedBearerTokenAuthorizer for SharedK8sSatTokenAuthorizer {
    async fn authorize(&self, credential: &BearerToken) -> Result<AuthzDecision, CapabilityError> {
        // Delegates to Core::authorize; keep identical to the local variant.
        self.inner
            .core
            .authorize(credential, &self.inner.cache)
            .await
    }
}

// -- Local variant (!Send; Rc + RefCell, lock-free) -------------------------

/// Local, `!Send` Kubernetes SAT authorizer.
///
/// Every clone on a core shares the same [`LocalInner`] via `Rc`, so they share
/// one lazily-built Kubernetes client and one lock-free decision cache. Each core
/// gets its own instance and hence its own cache -- a shared-nothing, per-core
/// memoization consistent with the engine's thread-per-core model.
#[derive(Clone)]
pub(crate) struct LocalK8sSatTokenAuthorizer {
    inner: Rc<LocalInner>,
}

/// State shared across clones of the local variant.
struct LocalInner {
    /// Common authenticate + admit logic and lazily-built client.
    core: Core,
    /// Decision cache in a `RefCell`: on a single core there is no contention,
    /// so this replaces the shared variant's `Mutex` -- lock-free.
    cache: RefCell<DecisionCache>,
}

impl LocalK8sSatTokenAuthorizer {
    /// Builds a new local-variant instance.
    pub(crate) fn new(
        name: &str,
        audiences: Vec<AudienceConfig>,
        cache_ttl: Duration,
        cache_max_entries: usize,
    ) -> Self {
        Self {
            inner: Rc::new(LocalInner {
                core: Core::new(name, audiences),
                cache: RefCell::new(DecisionCache::new(cache_ttl, cache_max_entries)),
            }),
        }
    }
}

#[async_trait(?Send)]
impl LocalBearerTokenAuthorizer for LocalK8sSatTokenAuthorizer {
    async fn authorize(&self, credential: &BearerToken) -> Result<AuthzDecision, CapabilityError> {
        // Delegates to Core::authorize; keep identical to the shared variant.
        self.inner
            .core
            .authorize(credential, &self.inner.cache)
            .await
    }
}
