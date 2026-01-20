// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! RAII handle for bundle consumption.
//!
//! A `BundleHandle` represents a claimed bundle that must be resolved before
//! being dropped. The handle provides methods to acknowledge, reject, or defer
//! the bundle.
//!
//! # Lifecycle
//!
//! ```text
//! next_bundle() --> BundleHandle --> ack()    (success, logged)
//!                               --> reject() (failure, logged as Dropped)
//!                               --> defer()  (retry later, not logged)
//!                               --> drop     (implicit defer)
//! ```
//!
//! # Example
//!
//! ```ignore
//! let handle = registry.next_bundle(&subscriber_id).await?;
//! match process_bundle(&handle.data).await {
//!     Ok(()) => handle.ack().await?,
//!     Err(e) if e.is_transient() => {
//!         let bundle_ref = handle.defer();
//!         schedule_retry(bundle_ref, backoff);
//!     }
//!     Err(e) => handle.reject().await?,
//! }
//! ```

use std::sync::Arc;

use crate::segment::ReconstructedBundle;

use super::types::{AckOutcome, BundleRef, SubscriberId};

// ─────────────────────────────────────────────────────────────────────────────
// Resolution Callback
// ─────────────────────────────────────────────────────────────────────────────

/// Callback invoked when a bundle is resolved or deferred.
///
/// The registry provides this callback to receive resolution notifications.
pub trait ResolutionCallback: Send + Sync {
    /// Called when a bundle is resolved with a terminal outcome.
    fn on_resolved(&self, subscriber_id: &SubscriberId, bundle_ref: BundleRef, outcome: AckOutcome);

    /// Called when a bundle is deferred (released for later retry).
    fn on_deferred(&self, subscriber_id: &SubscriberId, bundle_ref: BundleRef);
}

// ─────────────────────────────────────────────────────────────────────────────
// BundleHandle
// ─────────────────────────────────────────────────────────────────────────────

/// RAII handle for a claimed bundle.
///
/// The handle must be resolved via `ack()`, `reject()`, or `defer()` before
/// being dropped. Dropping without explicit resolution is treated as an
/// implicit `defer()`.
///
/// The handle provides access to the bundle data and reference information.
pub struct BundleHandle<C: ResolutionCallback> {
    /// Reference to this bundle (for retry scheduling).
    bundle_ref: BundleRef,
    /// The subscriber that claimed this bundle.
    subscriber_id: SubscriberId,
    /// The reconstructed bundle data.
    data: ReconstructedBundle,
    /// Callback for resolution notifications.
    callback: Arc<C>,
    /// Whether the handle has been explicitly resolved.
    resolved: bool,
}

impl<C: ResolutionCallback> BundleHandle<C> {
    /// Creates a new bundle handle.
    ///
    /// This is called by the registry when claiming a bundle.
    #[must_use]
    pub(crate) fn new(
        bundle_ref: BundleRef,
        subscriber_id: SubscriberId,
        data: ReconstructedBundle,
        callback: Arc<C>,
    ) -> Self {
        Self {
            bundle_ref,
            subscriber_id,
            data,
            callback,
            resolved: false,
        }
    }

    /// Returns a reference to the bundle data.
    #[must_use]
    pub fn data(&self) -> &ReconstructedBundle {
        &self.data
    }

    /// Returns the bundle reference (for retry scheduling).
    #[must_use]
    pub fn bundle_ref(&self) -> BundleRef {
        self.bundle_ref
    }

    /// Returns the subscriber ID.
    #[must_use]
    pub fn subscriber_id(&self) -> &SubscriberId {
        &self.subscriber_id
    }

    /// Acknowledges successful processing of the bundle.
    ///
    /// This marks the bundle as complete and records the outcome in the
    /// subscriber's progress file. The bundle will not be delivered again.
    pub fn ack(mut self) {
        self.resolved = true;
        self.callback
            .on_resolved(&self.subscriber_id, self.bundle_ref, AckOutcome::Acked);
    }

    /// Rejects the bundle after permanent failure.
    ///
    /// This marks the bundle as dropped and logs the outcome. Use this when
    /// the bundle cannot be processed after exhausting retries.
    pub fn reject(mut self) {
        self.resolved = true;
        self.callback
            .on_resolved(&self.subscriber_id, self.bundle_ref, AckOutcome::Dropped);
    }

    /// Defers the bundle for later retry.
    ///
    /// This releases the bundle's claim without logging any outcome. The
    /// bundle becomes immediately eligible for redelivery via `next_bundle()`.
    ///
    /// The embedding layer can either:
    /// - Call `next_bundle()` again (the deferred bundle will be returned
    ///   since it's the oldest unresolved unclaimed bundle), or
    /// - Use the returned `BundleRef` with `claim_bundle()` for explicit retry
    ///
    /// Returns the bundle reference (useful for custom retry scheduling).
    #[must_use]
    pub fn defer(mut self) -> BundleRef {
        self.resolved = true;
        let bundle_ref = self.bundle_ref;
        self.callback.on_deferred(&self.subscriber_id, bundle_ref);
        bundle_ref
    }
}

impl<C: ResolutionCallback> Drop for BundleHandle<C> {
    fn drop(&mut self) {
        if !self.resolved {
            // Implicit defer on drop
            self.callback
                .on_deferred(&self.subscriber_id, self.bundle_ref);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test utilities
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
pub(crate) mod test_utils {
    use super::*;
    use std::sync::Mutex;

    /// A mock resolution callback for testing.
    #[derive(Default)]
    pub struct MockCallback {
        pub resolutions: Mutex<Vec<(SubscriberId, BundleRef, AckOutcome)>>,
        pub deferrals: Mutex<Vec<(SubscriberId, BundleRef)>>,
    }

    impl ResolutionCallback for MockCallback {
        fn on_resolved(
            &self,
            subscriber_id: &SubscriberId,
            bundle_ref: BundleRef,
            outcome: AckOutcome,
        ) {
            self.resolutions
                .lock()
                .unwrap()
                .push((subscriber_id.clone(), bundle_ref, outcome));
        }

        fn on_deferred(&self, subscriber_id: &SubscriberId, bundle_ref: BundleRef) {
            self.deferrals
                .lock()
                .unwrap()
                .push((subscriber_id.clone(), bundle_ref));
        }
    }

    impl MockCallback {
        pub fn resolution_count(&self) -> usize {
            self.resolutions.lock().unwrap().len()
        }

        pub fn deferral_count(&self) -> usize {
            self.deferrals.lock().unwrap().len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::MockCallback;
    use super::*;
    use crate::segment::SegmentSeq;
    use crate::subscriber::types::BundleIndex;

    fn make_handle(callback: Arc<MockCallback>) -> BundleHandle<MockCallback> {
        let bundle_ref = BundleRef::new(SegmentSeq::new(1), BundleIndex::new(0));
        let subscriber_id = SubscriberId::new("test-sub").unwrap();
        let data = ReconstructedBundle::empty();

        BundleHandle::new(bundle_ref, subscriber_id, data, callback)
    }

    #[test]
    fn handle_ack() {
        let callback = Arc::new(MockCallback::default());
        let handle = make_handle(callback.clone());

        handle.ack();

        assert_eq!(callback.resolution_count(), 1);
        assert_eq!(callback.deferral_count(), 0);

        let resolutions = callback.resolutions.lock().unwrap();
        assert_eq!(resolutions[0].2, AckOutcome::Acked);
    }

    #[test]
    fn handle_reject() {
        let callback = Arc::new(MockCallback::default());
        let handle = make_handle(callback.clone());

        handle.reject();

        assert_eq!(callback.resolution_count(), 1);
        assert_eq!(callback.deferral_count(), 0);

        let resolutions = callback.resolutions.lock().unwrap();
        assert_eq!(resolutions[0].2, AckOutcome::Dropped);
    }

    #[test]
    fn handle_defer() {
        let callback = Arc::new(MockCallback::default());
        let handle = make_handle(callback.clone());

        let bundle_ref = handle.defer();

        assert_eq!(bundle_ref.segment_seq, SegmentSeq::new(1));
        assert_eq!(callback.resolution_count(), 0);
        assert_eq!(callback.deferral_count(), 1);
    }

    #[test]
    fn handle_drop_implicit_defer() {
        let callback = Arc::new(MockCallback::default());

        {
            let _handle = make_handle(callback.clone());
            // Drop without resolving
        }

        assert_eq!(callback.resolution_count(), 0);
        assert_eq!(callback.deferral_count(), 1);
    }

    #[test]
    fn handle_accessors() {
        let callback = Arc::new(MockCallback::default());
        let handle = make_handle(callback);

        assert_eq!(handle.subscriber_id().as_str(), "test-sub");
        assert_eq!(handle.bundle_ref().segment_seq, SegmentSeq::new(1));
        assert_eq!(handle.bundle_ref().bundle_index, BundleIndex::new(0));
    }
}
