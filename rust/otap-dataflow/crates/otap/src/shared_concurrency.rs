// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared concurrency limiting across multiple protocol servers.
//!
//! Provides a Tower middleware layer that uses an external semaphore for
//! concurrency control, allowing multiple servers (gRPC, HTTP) to draw
//! from the same capacity pool.

use futures::future::BoxFuture;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tower::{Layer, Service};

/// Tower layer that enforces concurrency limits using a shared semaphore.
///
/// This layer acquires a permit from the provided semaphore before processing
/// each request. The permit is held for the entire request lifetime, ensuring
/// the total number of concurrent requests across all services using this
/// semaphore never exceeds its capacity.
///
/// # Behavior Notes
///
/// - **Queuing**: This layer uses `acquire_owned().await` in the request future,
///   which allows requests to queue when at capacity. Unlike
///   `tower::limit::GlobalConcurrencyLimitLayer` which applies backpressure at
///   the `poll_ready` stage, this implementation accepts connections that wait
///   for permits. This is acceptable for most use cases but means the server
///   may have more pending tasks.
///
/// - **Service Cloning**: The inner service is cloned per request, which is
///   standard for Tower middleware. For tonic services this is typically cheap
///   (Arc-based cloning).
///
/// # Important: Service Contract Requirements
///
/// This layer calls `poll_ready` on the inner service but then executes the
/// request on a **cloned** instance. This pattern is safe **only** for services
/// that meet one of these criteria:
///
/// 1. **Stateless readiness**: `poll_ready` always returns `Poll::Ready(Ok(()))`
///    regardless of instance state (e.g., tonic gRPC services).
///
/// 2. **Shared readiness state**: The service uses `Arc`-based internal state
///    so that clones share the same readiness tracking.
///
/// **Do NOT use this layer** with services that track per-instance readiness
/// or backpressure state (e.g., services wrapping bounded channels where each
/// clone has independent capacity). For such services, use Tower's
/// `GlobalConcurrencyLimitLayer` or `Buffer` which hold the same service
/// instance through the `poll_ready` → `call` sequence.
///
/// # Example
///
/// ```ignore
/// let semaphore = Arc::new(Semaphore::new(100));
/// let layer = SharedConcurrencyLayer::new(semaphore.clone());
///
/// // Use with tower services (e.g., tonic Server)
/// Server::builder()
///     .layer(layer)
///     .add_service(my_service)
/// ```
#[derive(Clone)]
pub struct SharedConcurrencyLayer {
    semaphore: Arc<Semaphore>,
}

impl SharedConcurrencyLayer {
    /// Creates a new layer using the provided semaphore for concurrency control.
    pub fn new(semaphore: Arc<Semaphore>) -> Self {
        Self { semaphore }
    }
}

impl<S> Layer<S> for SharedConcurrencyLayer {
    type Service = SharedConcurrencyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SharedConcurrencyService {
            inner,
            semaphore: self.semaphore.clone(),
        }
    }
}

/// Service that wraps an inner service with shared semaphore-based concurrency limiting.
///
/// # Safety Note
///
/// This implementation clones the inner service per request (see `call`), meaning
/// `poll_ready` is checked on one instance while the request executes on a clone.
/// See [`SharedConcurrencyLayer`] documentation for service compatibility requirements.
#[derive(Clone)]
pub struct SharedConcurrencyService<S> {
    inner: S,
    semaphore: Arc<Semaphore>,
}

impl<S, Request> Service<Request> for SharedConcurrencyService<S>
where
    S: Service<Request> + Clone + Send + 'static,
    S::Future: Send + 'static,
    Request: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Check inner service readiness.
        // NOTE: This checks readiness on `self.inner`, but `call()` will execute
        // on a clone. This is safe only for services with stateless or shared
        // readiness state. See struct-level documentation.
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        // Clone the inner service for the async block. This is a standard Tower
        // pattern but means readiness was checked on a different instance.
        // For services with per-instance state, consider tower::Buffer instead.
        let semaphore = self.semaphore.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Acquire permit - this will wait if at capacity
            let _permit: OwnedSemaphorePermit = semaphore
                .acquire_owned()
                .await
                .expect("semaphore should never be closed");

            // Permit is held until the future completes (RAII)
            // Process the request with the inner service
            inner.call(request).await
        })
    }
}
