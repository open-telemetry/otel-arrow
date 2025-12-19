// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Subscriber registry for lifecycle management.
//!
//! The registry is the central coordinator for subscriber management:
//!
//! - Registers and unregisters subscribers
//! - Tracks per-subscriber state
//! - Coordinates ack log writes
//! - Provides bundle delivery API
//!
//! # Lifecycle
//!
//! ```text
//! register(id) --> Inactive subscriber (state loaded from quiver.ack if present)
//!     |
//!     v
//! activate(id) --> Active subscriber (receives new bundles)
//!     |
//!     v
//! [normal operation: next_bundle, claim_bundle, ack, reject, defer]
//!     |
//!     v
//! deactivate(id) --> Inactive (orphan detection starts)
//!     |
//!     v
//! unregister(id) --> Removed (only for permanent removal)
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use crate::segment::{ReconstructedBundle, SegmentSeq};

use super::ack_log::{AckLogEntry, AckLogReader, AckLogWriter};
use super::error::{Result, SubscriberError};
use super::handle::{BundleHandle, ResolutionCallback};
use super::state::SubscriberState;
use super::types::{AckOutcome, BundleRef, SubscriberId};

// ─────────────────────────────────────────────────────────────────────────────
// SegmentProvider
// ─────────────────────────────────────────────────────────────────────────────

/// Trait for accessing segment data.
///
/// The registry uses this to read bundles from segments without coupling
/// directly to the engine's segment storage.
pub trait SegmentProvider: Send + Sync {
    /// Returns the bundle count for a segment.
    fn bundle_count(&self, segment_seq: SegmentSeq) -> Result<u32>;

    /// Reads a bundle from a segment.
    fn read_bundle(&self, bundle_ref: BundleRef) -> Result<ReconstructedBundle>;

    /// Lists all available segment sequences.
    fn available_segments(&self) -> Vec<SegmentSeq>;
}

// ─────────────────────────────────────────────────────────────────────────────
// RegistryConfig
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for the subscriber registry.
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Directory containing the ack log file.
    pub data_dir: PathBuf,
}

impl RegistryConfig {
    /// Creates a new registry configuration.
    #[must_use]
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    /// Returns the path to the ack log file.
    #[must_use]
    pub fn ack_log_path(&self) -> PathBuf {
        self.data_dir.join("quiver.ack")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SubscriberRegistry
// ─────────────────────────────────────────────────────────────────────────────

/// Central registry for subscriber management.
///
/// Thread-safe coordinator for subscriber lifecycle, state tracking, and
/// bundle delivery.
pub struct SubscriberRegistry<P: SegmentProvider> {
    config: RegistryConfig,
    /// Per-subscriber state, keyed by subscriber ID.
    subscribers: RwLock<HashMap<SubscriberId, SubscriberState>>,
    /// Ack log writer (serializes writes).
    ack_log: Mutex<AckLogWriter>,
    /// Segment data provider.
    segment_provider: Arc<P>,
    /// Self-reference for callbacks.
    self_ref: RwLock<Option<Arc<Self>>>,
}

impl<P: SegmentProvider> SubscriberRegistry<P> {
    /// Opens or creates a subscriber registry.
    ///
    /// If an ack log exists, replays it to rebuild subscriber state.
    ///
    /// # Errors
    ///
    /// Returns an error if the ack log cannot be opened or is corrupted.
    pub fn open(config: RegistryConfig, segment_provider: Arc<P>) -> Result<Arc<Self>> {
        let ack_log_path = config.ack_log_path();

        // Load existing state from ack log if present
        let mut subscribers = HashMap::new();
        if ack_log_path.exists() {
            let reader = AckLogReader::open(&ack_log_path)?;
            let entries = reader.read_all()?;

            for entry in entries {
                let state = subscribers
                    .entry(entry.subscriber_id.clone())
                    .or_insert_with(|| SubscriberState::new(entry.subscriber_id.clone()));

                // Ensure segment is tracked
                // Note: We don't know the bundle count here, so we create with
                // a placeholder. The actual count will be updated when the
                // subscriber is activated.
                state.add_segment(entry.bundle_ref.segment_seq, u32::MAX);
                let _ = state.record_outcome(entry.bundle_ref, entry.outcome);
            }
        }

        let ack_log = AckLogWriter::open(&ack_log_path)?;

        let registry = Arc::new(Self {
            config,
            subscribers: RwLock::new(subscribers),
            ack_log: Mutex::new(ack_log),
            segment_provider,
            self_ref: RwLock::new(None),
        });

        // Store self-reference for callbacks
        *registry.self_ref.write().expect("self_ref lock poisoned") = Some(registry.clone());

        Ok(registry)
    }

    /// Returns the registry configuration.
    #[must_use]
    pub fn config(&self) -> &RegistryConfig {
        &self.config
    }

    /// Registers a new subscriber.
    ///
    /// If the subscriber was previously registered (state exists from ack log),
    /// this reactivates the existing state. Otherwise, creates new state
    /// starting from the latest segment.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber ID is invalid.
    pub fn register(&self, id: SubscriberId) -> Result<()> {
        let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");

        if subscribers.contains_key(&id) {
            // Already registered (possibly from ack log recovery)
            return Ok(());
        }

        // Create new state
        let state = SubscriberState::new(id.clone());
        let _ = subscribers.insert(id, state);

        Ok(())
    }

    /// Activates a subscriber to receive new bundles.
    ///
    /// This initializes tracking for all available segments that the subscriber
    /// hasn't yet processed.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber is not registered.
    pub fn activate(&self, id: &SubscriberId) -> Result<()> {
        let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");

        let state = subscribers
            .get_mut(id)
            .ok_or_else(|| SubscriberError::not_found(id.as_str()))?;

        if state.is_active() {
            return Ok(());
        }

        // Add all available segments
        for segment_seq in self.segment_provider.available_segments() {
            let bundle_count = self.segment_provider.bundle_count(segment_seq)?;
            state.add_segment(segment_seq, bundle_count);
        }

        state.activate();
        Ok(())
    }

    /// Deactivates a subscriber.
    ///
    /// The subscriber stops receiving new bundles but retains its state for
    /// later reactivation.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber is not registered.
    pub fn deactivate(&self, id: &SubscriberId) -> Result<()> {
        let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");

        let state = subscribers
            .get_mut(id)
            .ok_or_else(|| SubscriberError::not_found(id.as_str()))?;

        state.deactivate();
        Ok(())
    }

    /// Permanently unregisters a subscriber.
    ///
    /// This removes all state for the subscriber. Use with caution—any
    /// unprocessed bundles will be lost.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber is not registered.
    pub fn unregister(&self, id: &SubscriberId) -> Result<()> {
        let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");

        if subscribers.remove(id).is_none() {
            return Err(SubscriberError::not_found(id.as_str()));
        }

        Ok(())
    }

    /// Notifies the registry of a new finalized segment.
    ///
    /// Called by the engine when a segment is finalized. Active subscribers
    /// will start tracking the new segment.
    pub fn on_segment_finalized(&self, segment_seq: SegmentSeq, bundle_count: u32) {
        let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");

        for state in subscribers.values_mut() {
            if state.is_active() {
                state.add_segment(segment_seq, bundle_count);
            }
        }
    }

    /// Returns the next pending bundle for a subscriber.
    ///
    /// Claims the bundle and returns a handle for resolution.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber is not registered or not active.
    pub fn next_bundle(
        self: &Arc<Self>,
        id: &SubscriberId,
    ) -> Result<Option<BundleHandle<RegistryCallback<P>>>> {
        let bundle_ref = {
            let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");

            let state = subscribers
                .get_mut(id)
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?;

            if !state.is_active() {
                return Err(SubscriberError::not_found(id.as_str()));
            }

            // Find next pending
            let bundle_ref = match state.next_pending() {
                Some(br) => br,
                None => return Ok(None),
            };

            // Claim it
            let _ = state.claim(bundle_ref);
            bundle_ref
        };

        // Read the bundle data (outside lock)
        let data = self.segment_provider.read_bundle(bundle_ref)?;

        let callback = Arc::new(RegistryCallback {
            registry: self.clone(),
        });

        Ok(Some(BundleHandle::new(
            bundle_ref,
            id.clone(),
            data,
            callback,
        )))
    }

    /// Claims a specific bundle for a subscriber.
    ///
    /// Used for retry scenarios where the embedding layer needs to re-acquire
    /// a previously deferred bundle.
    ///
    /// # Errors
    ///
    /// Returns an error if the bundle is not available (already resolved or
    /// claimed by another).
    pub fn claim_bundle(
        self: &Arc<Self>,
        id: &SubscriberId,
        bundle_ref: BundleRef,
    ) -> Result<BundleHandle<RegistryCallback<P>>> {
        {
            let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");

            let state = subscribers
                .get_mut(id)
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?;

            // Check if already resolved
            if state.is_resolved(&bundle_ref) {
                return Err(SubscriberError::bundle_not_available("already resolved"));
            }

            // Check if already claimed
            if state.is_claimed(&bundle_ref) {
                return Err(SubscriberError::bundle_not_available("already claimed"));
            }

            // Claim it
            if !state.claim(bundle_ref) {
                return Err(SubscriberError::bundle_not_available("claim failed"));
            }
        }

        // Read the bundle data (outside lock)
        let data = self.segment_provider.read_bundle(bundle_ref)?;

        let callback = Arc::new(RegistryCallback {
            registry: self.clone(),
        });

        Ok(BundleHandle::new(bundle_ref, id.clone(), data, callback))
    }

    /// Records a resolution and writes to the ack log.
    fn record_resolution(
        &self,
        subscriber_id: &SubscriberId,
        bundle_ref: BundleRef,
        outcome: AckOutcome,
    ) {
        // Update in-memory state
        {
            let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");
            if let Some(state) = subscribers.get_mut(subscriber_id) {
                let _ = state.record_outcome(bundle_ref, outcome);
            }
        }

        // Write to ack log
        let entry = AckLogEntry::new(subscriber_id.clone(), bundle_ref, outcome);
        let mut ack_log = self.ack_log.lock().expect("ack_log lock poisoned");
        if let Err(e) = ack_log.append(&entry) {
            // Log error but don't fail—in-memory state is already updated
            tracing::error!(
                subscriber_id = %subscriber_id,
                bundle_ref = %bundle_ref,
                outcome = %outcome,
                error = %e,
                "failed to write to ack log"
            );
        }
    }

    /// Releases a bundle claim without resolving.
    fn release_bundle(&self, subscriber_id: &SubscriberId, bundle_ref: BundleRef) {
        let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");
        if let Some(state) = subscribers.get_mut(subscriber_id) {
            state.release(bundle_ref);
        }
    }

    /// Returns the number of registered subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers
            .read()
            .expect("subscribers lock poisoned")
            .len()
    }

    /// Returns the oldest incomplete segment across all subscribers.
    ///
    /// Segments older than this can be safely deleted (all subscribers have
    /// completed them).
    #[must_use]
    pub fn oldest_incomplete_segment(&self) -> Option<SegmentSeq> {
        self.subscribers
            .read()
            .expect("subscribers lock poisoned")
            .values()
            .filter_map(|s| s.oldest_incomplete_segment())
            .min()
    }

    /// Returns whether a subscriber is registered.
    #[must_use]
    pub fn is_registered(&self, id: &SubscriberId) -> bool {
        self.subscribers
            .read()
            .expect("subscribers lock poisoned")
            .contains_key(id)
    }

    /// Returns whether a subscriber is active.
    #[must_use]
    pub fn is_active(&self, id: &SubscriberId) -> bool {
        self.subscribers
            .read()
            .expect("subscribers lock poisoned")
            .get(id)
            .is_some_and(|s| s.is_active())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RegistryCallback
// ─────────────────────────────────────────────────────────────────────────────

/// Resolution callback that writes to the registry.
pub struct RegistryCallback<P: SegmentProvider> {
    registry: Arc<SubscriberRegistry<P>>,
}

impl<P: SegmentProvider> ResolutionCallback for RegistryCallback<P> {
    fn on_resolved(
        &self,
        subscriber_id: &SubscriberId,
        bundle_ref: BundleRef,
        outcome: AckOutcome,
    ) {
        self.registry
            .record_resolution(subscriber_id, bundle_ref, outcome);
    }

    fn on_deferred(&self, subscriber_id: &SubscriberId, bundle_ref: BundleRef) {
        self.registry.release_bundle(subscriber_id, bundle_ref);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, unused_results)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;
    use tempfile::tempdir;

    use crate::subscriber::types::BundleIndex;

    /// Mock segment provider for testing.
    struct MockSegmentProvider {
        segments: Mutex<BTreeMap<SegmentSeq, u32>>,
    }

    impl MockSegmentProvider {
        fn new() -> Self {
            Self {
                segments: Mutex::new(BTreeMap::new()),
            }
        }

        fn add_segment(&self, seq: u64, bundle_count: u32) {
            self.segments
                .lock()
                .unwrap()
                .insert(SegmentSeq::new(seq), bundle_count);
        }
    }

    impl SegmentProvider for MockSegmentProvider {
        fn bundle_count(&self, segment_seq: SegmentSeq) -> Result<u32> {
            self.segments
                .lock()
                .unwrap()
                .get(&segment_seq)
                .copied()
                .ok_or_else(|| SubscriberError::segment_not_found(segment_seq.raw()))
        }

        fn read_bundle(&self, bundle_ref: BundleRef) -> Result<ReconstructedBundle> {
            // Check segment exists
            if !self
                .segments
                .lock()
                .unwrap()
                .contains_key(&bundle_ref.segment_seq)
            {
                return Err(SubscriberError::segment_not_found(
                    bundle_ref.segment_seq.raw(),
                ));
            }
            Ok(ReconstructedBundle::empty())
        }

        fn available_segments(&self) -> Vec<SegmentSeq> {
            self.segments.lock().unwrap().keys().copied().collect()
        }
    }

    fn setup_registry() -> (
        Arc<SubscriberRegistry<MockSegmentProvider>>,
        tempfile::TempDir,
    ) {
        let dir = tempdir().unwrap();
        let config = RegistryConfig::new(dir.path());
        let provider = Arc::new(MockSegmentProvider::new());
        let registry = SubscriberRegistry::open(config, provider).unwrap();
        (registry, dir)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Registration tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn register_new_subscriber() {
        let (registry, _dir) = setup_registry();

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();

        assert!(registry.is_registered(&id));
        assert!(!registry.is_active(&id));
    }

    #[test]
    fn register_duplicate_ok() {
        let (registry, _dir) = setup_registry();

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.register(id.clone()).unwrap(); // Should not error

        assert_eq!(registry.subscriber_count(), 1);
    }

    #[test]
    fn activate_subscriber() {
        let (registry, _dir) = setup_registry();

        // Add a segment first
        let provider = registry.segment_provider.clone();
        provider.add_segment(1, 5);

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        assert!(registry.is_active(&id));
    }

    #[test]
    fn activate_not_registered() {
        let (registry, _dir) = setup_registry();

        let id = SubscriberId::new("unknown").unwrap();
        let result = registry.activate(&id);

        assert!(matches!(result, Err(SubscriberError::NotFound { .. })));
    }

    #[test]
    fn deactivate_subscriber() {
        let (registry, _dir) = setup_registry();

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();
        registry.deactivate(&id).unwrap();

        assert!(!registry.is_active(&id));
        assert!(registry.is_registered(&id));
    }

    #[test]
    fn unregister_subscriber() {
        let (registry, _dir) = setup_registry();

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.unregister(&id).unwrap();

        assert!(!registry.is_registered(&id));
    }

    #[test]
    fn unregister_not_registered() {
        let (registry, _dir) = setup_registry();

        let id = SubscriberId::new("unknown").unwrap();
        let result = registry.unregister(&id);

        assert!(matches!(result, Err(SubscriberError::NotFound { .. })));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Bundle delivery tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn next_bundle_empty() {
        let (registry, _dir) = setup_registry();

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        let result = registry.next_bundle(&id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn next_bundle_returns_handle() {
        let (registry, _dir) = setup_registry();
        let provider = registry.segment_provider.clone();
        provider.add_segment(1, 3);

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        let handle = registry.next_bundle(&id).unwrap().unwrap();
        assert_eq!(handle.bundle_ref().segment_seq, SegmentSeq::new(1));
        assert_eq!(handle.bundle_ref().bundle_index, BundleIndex::new(0));

        handle.ack();
    }

    #[test]
    fn next_bundle_skips_claimed() {
        let (registry, _dir) = setup_registry();
        let provider = registry.segment_provider.clone();
        provider.add_segment(1, 3);

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        // Get first bundle (claims it)
        let handle1 = registry.next_bundle(&id).unwrap().unwrap();
        assert_eq!(handle1.bundle_ref().bundle_index, BundleIndex::new(0));

        // Next should skip the claimed one
        let handle2 = registry.next_bundle(&id).unwrap().unwrap();
        assert_eq!(handle2.bundle_ref().bundle_index, BundleIndex::new(1));

        handle1.ack();
        handle2.ack();
    }

    #[test]
    fn ack_persists_to_log() {
        let dir = tempdir().unwrap();
        let config = RegistryConfig::new(dir.path());
        let provider = Arc::new(MockSegmentProvider::new());
        provider.add_segment(1, 2);

        let id = SubscriberId::new("test-sub").unwrap();

        {
            let registry = SubscriberRegistry::open(config.clone(), provider.clone()).unwrap();
            registry.register(id.clone()).unwrap();
            registry.activate(&id).unwrap();

            let handle = registry.next_bundle(&id).unwrap().unwrap();
            handle.ack();
        }

        // Reopen and verify state was recovered
        {
            let registry = SubscriberRegistry::open(config, provider).unwrap();
            registry.register(id.clone()).unwrap();
            registry.activate(&id).unwrap();

            // First bundle should be skipped (already acked)
            let handle = registry.next_bundle(&id).unwrap().unwrap();
            assert_eq!(handle.bundle_ref().bundle_index, BundleIndex::new(1));
            handle.ack();
        }
    }

    #[test]
    fn reject_persists_to_log() {
        let dir = tempdir().unwrap();
        let config = RegistryConfig::new(dir.path());
        let provider = Arc::new(MockSegmentProvider::new());
        provider.add_segment(1, 2);

        let id = SubscriberId::new("test-sub").unwrap();

        {
            let registry = SubscriberRegistry::open(config.clone(), provider.clone()).unwrap();
            registry.register(id.clone()).unwrap();
            registry.activate(&id).unwrap();

            let handle = registry.next_bundle(&id).unwrap().unwrap();
            handle.reject(); // Dropped
        }

        // Reopen and verify state was recovered
        {
            let registry = SubscriberRegistry::open(config, provider).unwrap();
            registry.register(id.clone()).unwrap();
            registry.activate(&id).unwrap();

            // First bundle should be skipped (dropped)
            let handle = registry.next_bundle(&id).unwrap().unwrap();
            assert_eq!(handle.bundle_ref().bundle_index, BundleIndex::new(1));
            handle.ack();
        }
    }

    #[test]
    fn defer_releases_claim() {
        let (registry, _dir) = setup_registry();
        let provider = registry.segment_provider.clone();
        provider.add_segment(1, 2);

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        // Get and defer first bundle
        let handle = registry.next_bundle(&id).unwrap().unwrap();
        let bundle_ref = handle.defer();
        assert_eq!(bundle_ref.bundle_index, BundleIndex::new(0));

        // Should be able to get it again
        let handle = registry.next_bundle(&id).unwrap().unwrap();
        assert_eq!(handle.bundle_ref().bundle_index, BundleIndex::new(0));
        handle.ack();
    }

    #[test]
    fn claim_bundle_for_retry() {
        let (registry, _dir) = setup_registry();
        let provider = registry.segment_provider.clone();
        provider.add_segment(1, 2);

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        // Get and defer
        let handle = registry.next_bundle(&id).unwrap().unwrap();
        let bundle_ref = handle.defer();

        // Explicitly claim it
        let handle = registry.claim_bundle(&id, bundle_ref).unwrap();
        assert_eq!(handle.bundle_ref(), bundle_ref);
        handle.ack();
    }

    #[test]
    fn claim_bundle_already_resolved() {
        let (registry, _dir) = setup_registry();
        let provider = registry.segment_provider.clone();
        provider.add_segment(1, 2);

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        // Get and ack
        let handle = registry.next_bundle(&id).unwrap().unwrap();
        let bundle_ref = handle.bundle_ref();
        handle.ack();

        // Try to claim again
        let result = registry.claim_bundle(&id, bundle_ref);
        assert!(matches!(
            result,
            Err(SubscriberError::BundleNotAvailable { .. })
        ));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Segment notification tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn on_segment_finalized_updates_active() {
        let (registry, _dir) = setup_registry();

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        // Notify of new segment
        registry.on_segment_finalized(SegmentSeq::new(1), 5);

        // Add to provider so read works
        registry.segment_provider.add_segment(1, 5);

        // Should be able to get bundles
        let handle = registry.next_bundle(&id).unwrap().unwrap();
        assert_eq!(handle.bundle_ref().segment_seq, SegmentSeq::new(1));
        handle.ack();
    }

    #[test]
    fn on_segment_finalized_ignores_inactive() {
        let (registry, _dir) = setup_registry();

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        // Not activated

        // Notify of new segment
        registry.on_segment_finalized(SegmentSeq::new(1), 5);
        registry.segment_provider.add_segment(1, 5);

        // Activate now
        registry.activate(&id).unwrap();

        // Should see the segment (added during activate)
        let handle = registry.next_bundle(&id).unwrap().unwrap();
        assert_eq!(handle.bundle_ref().segment_seq, SegmentSeq::new(1));
        handle.ack();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Multi-subscriber tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn multiple_subscribers_independent() {
        let (registry, _dir) = setup_registry();
        let provider = registry.segment_provider.clone();
        provider.add_segment(1, 2);

        let id1 = SubscriberId::new("sub-1").unwrap();
        let id2 = SubscriberId::new("sub-2").unwrap();

        registry.register(id1.clone()).unwrap();
        registry.register(id2.clone()).unwrap();
        registry.activate(&id1).unwrap();
        registry.activate(&id2).unwrap();

        // Both should get bundle 0
        let h1 = registry.next_bundle(&id1).unwrap().unwrap();
        let h2 = registry.next_bundle(&id2).unwrap().unwrap();

        assert_eq!(h1.bundle_ref().bundle_index, BundleIndex::new(0));
        assert_eq!(h2.bundle_ref().bundle_index, BundleIndex::new(0));

        h1.ack();
        h2.ack();
    }

    #[test]
    fn oldest_incomplete_segment() {
        let (registry, _dir) = setup_registry();
        let provider = registry.segment_provider.clone();
        provider.add_segment(1, 1);
        provider.add_segment(2, 1);

        let id1 = SubscriberId::new("sub-1").unwrap();
        let id2 = SubscriberId::new("sub-2").unwrap();

        registry.register(id1.clone()).unwrap();
        registry.register(id2.clone()).unwrap();
        registry.activate(&id1).unwrap();
        registry.activate(&id2).unwrap();

        // Both have pending in segment 1
        assert_eq!(
            registry.oldest_incomplete_segment(),
            Some(SegmentSeq::new(1))
        );

        // Sub1 completes segment 1
        let h = registry.next_bundle(&id1).unwrap().unwrap();
        h.ack();

        // Still waiting on sub2
        assert_eq!(
            registry.oldest_incomplete_segment(),
            Some(SegmentSeq::new(1))
        );

        // Sub2 completes segment 1
        let h = registry.next_bundle(&id2).unwrap().unwrap();
        h.ack();

        // Now segment 2 is oldest incomplete
        assert_eq!(
            registry.oldest_incomplete_segment(),
            Some(SegmentSeq::new(2))
        );
    }
}
