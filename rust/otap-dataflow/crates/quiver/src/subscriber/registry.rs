// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Subscriber registry for lifecycle management.
//!
//! The registry is the central coordinator for subscriber management:
//!
//! - Registers and unregisters subscribers
//! - Tracks per-subscriber state
//! - Coordinates progress file writes
//! - Provides bundle delivery API
//!
//! # Lifecycle
//!
//! ```text
//! register(id) --> Inactive subscriber (state loaded from progress file if present)
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
//!
//! # Progress Persistence
//!
//! Subscriber progress is persisted to individual files (`quiver.sub.<id>`).
//! In-memory state is updated immediately on each ack/reject. The embedding
//! layer should call `flush_progress()` periodically (e.g., every 25ms) to
//! write dirty subscribers to disk.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use crate::segment::{ReconstructedBundle, SegmentSeq};

use super::error::{Result, SubscriberError};
use super::handle::{BundleHandle, ResolutionCallback};
use super::progress::{
    delete_progress_file, progress_file_path, read_progress_file, scan_progress_files,
    write_progress_file,
};
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
    /// Directory containing subscriber progress files.
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
}

// ─────────────────────────────────────────────────────────────────────────────
// SubscriberRegistry
// ─────────────────────────────────────────────────────────────────────────────

/// Central registry for subscriber management.
///
/// Thread-safe coordinator for subscriber lifecycle, state tracking, and
/// bundle delivery.
///
/// Uses per-subscriber locks to minimize contention: the main map uses a
/// read-write lock for structural changes (register/unregister), while each
/// subscriber has its own lock for state updates.
pub struct SubscriberRegistry<P: SegmentProvider> {
    config: RegistryConfig,
    /// Per-subscriber state, keyed by subscriber ID.
    /// Each subscriber has its own RwLock to allow concurrent access.
    subscribers: RwLock<HashMap<SubscriberId, Arc<RwLock<SubscriberState>>>>,
    /// Subscribers with uncommitted changes that need flushing.
    dirty_subscribers: Mutex<HashSet<SubscriberId>>,
    /// Segment data provider.
    segment_provider: Arc<P>,
    /// Self-reference for callbacks.
    self_ref: RwLock<Option<Arc<Self>>>,
}

impl<P: SegmentProvider> SubscriberRegistry<P> {
    /// Opens or creates a subscriber registry.
    ///
    /// Scans for existing progress files and loads subscriber state.
    ///
    /// # Errors
    ///
    /// Returns an error if progress files cannot be read or are corrupted.
    pub fn open(config: RegistryConfig, segment_provider: Arc<P>) -> Result<Arc<Self>> {
        // Load existing state from progress files
        let mut subscribers = HashMap::new();

        // Scan for existing progress files
        if config.data_dir.exists() {
            let subscriber_ids = scan_progress_files(&config.data_dir)?;

            for sub_id in subscriber_ids {
                let path = progress_file_path(&config.data_dir, &sub_id);
                match read_progress_file(&path) {
                    Ok((_oldest_incomplete, entries)) => {
                        let mut state = SubscriberState::new(sub_id.clone());

                        // Restore segment progress from entries
                        for entry in entries {
                            state.add_segment(entry.seg_seq, entry.bundle_count);
                            // Mark acked bundles
                            for bundle_idx in 0..entry.bundle_count {
                                if entry.is_acked(bundle_idx) {
                                    let bundle_ref = BundleRef::new(
                                        entry.seg_seq,
                                        super::types::BundleIndex::new(bundle_idx),
                                    );
                                    let _ = state.record_outcome(bundle_ref, AckOutcome::Acked);
                                }
                            }
                        }

                        // Wrap in Arc<RwLock<>> for per-subscriber locking
                        let _ = subscribers.insert(sub_id, Arc::new(RwLock::new(state)));
                    }
                    Err(e) => {
                        tracing::warn!(
                            subscriber_id = %sub_id,
                            error = %e,
                            "failed to load progress file, subscriber will start fresh"
                        );
                    }
                }
            }
        }

        let registry = Arc::new(Self {
            config,
            subscribers: RwLock::new(subscribers),
            dirty_subscribers: Mutex::new(HashSet::new()),
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
    /// If the subscriber was previously registered (state exists from progress file),
    /// this reactivates the existing state. Otherwise, creates new state
    /// starting from the latest segment.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber ID is invalid.
    pub fn register(&self, id: SubscriberId) -> Result<()> {
        let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");

        if subscribers.contains_key(&id) {
            // Already registered (possibly from progress file recovery)
            return Ok(());
        }

        // Create new state wrapped in per-subscriber lock
        let state = SubscriberState::new(id.clone());
        let _ = subscribers.insert(id, Arc::new(RwLock::new(state)));

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
        // Get subscriber lock (read lock on map, then per-subscriber write lock)
        let state_lock = {
            let subscribers = self.subscribers.read().expect("subscribers lock poisoned");
            subscribers
                .get(id)
                .cloned()
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?
        };

        let mut state = state_lock.write().expect("subscriber lock poisoned");

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
        let state_lock = {
            let subscribers = self.subscribers.read().expect("subscribers lock poisoned");
            subscribers
                .get(id)
                .cloned()
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?
        };

        let mut state = state_lock.write().expect("subscriber lock poisoned");
        state.deactivate();
        Ok(())
    }

    /// Permanently unregisters a subscriber.
    ///
    /// This removes all state for the subscriber and deletes its progress file.
    /// Use with caution—any unprocessed bundles will be lost.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber is not registered or if the progress
    /// file cannot be deleted.
    pub fn unregister(&self, id: &SubscriberId) -> Result<()> {
        // Remove from in-memory state
        {
            let mut subscribers = self.subscribers.write().expect("subscribers lock poisoned");
            if subscribers.remove(id).is_none() {
                return Err(SubscriberError::not_found(id.as_str()));
            }
        }

        // Remove from dirty set
        {
            let mut dirty = self.dirty_subscribers.lock().expect("dirty lock poisoned");
            let _ = dirty.remove(id);
        }

        // Delete progress file
        delete_progress_file(&self.config.data_dir, id)?;

        Ok(())
    }

    /// Notifies the registry of a new finalized segment.
    ///
    /// Called by the engine when a segment is finalized. Active subscribers
    /// will start tracking the new segment.
    pub fn on_segment_finalized(&self, segment_seq: SegmentSeq, bundle_count: u32) {
        // Read lock on map, then per-subscriber write locks
        let subscribers = self.subscribers.read().expect("subscribers lock poisoned");

        for state_lock in subscribers.values() {
            let mut state = state_lock.write().expect("subscriber lock poisoned");
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
        // Get subscriber lock with minimal map lock time
        let state_lock = {
            let subscribers = self.subscribers.read().expect("subscribers lock poisoned");
            subscribers
                .get(id)
                .cloned()
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?
        };

        let bundle_ref = {
            let mut state = state_lock.write().expect("subscriber lock poisoned");

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

        // Read the bundle data (outside all locks)
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
        // Get subscriber lock with minimal map lock time
        let state_lock = {
            let subscribers = self.subscribers.read().expect("subscribers lock poisoned");
            subscribers
                .get(id)
                .cloned()
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?
        };

        {
            let mut state = state_lock.write().expect("subscriber lock poisoned");

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

        // Read the bundle data (outside all locks)
        let data = self.segment_provider.read_bundle(bundle_ref)?;

        let callback = Arc::new(RegistryCallback {
            registry: self.clone(),
        });

        Ok(BundleHandle::new(bundle_ref, id.clone(), data, callback))
    }

    /// Records a resolution and marks subscriber as dirty.
    ///
    /// The embedding layer should call `flush_progress()` periodically to
    /// persist dirty subscribers to disk.
    fn record_resolution(
        &self,
        subscriber_id: &SubscriberId,
        bundle_ref: BundleRef,
        outcome: AckOutcome,
    ) {
        // Get subscriber lock with minimal map lock time
        let state_lock = {
            let subscribers = self.subscribers.read().expect("subscribers lock poisoned");
            subscribers.get(subscriber_id).cloned()
        };

        // Update in-memory state
        if let Some(state_lock) = state_lock {
            let mut state = state_lock.write().expect("subscriber lock poisoned");
            let _ = state.record_outcome(bundle_ref, outcome);
        }

        // Mark subscriber as dirty (needs flushing)
        {
            let mut dirty = self.dirty_subscribers.lock().expect("dirty lock poisoned");
            let _ = dirty.insert(subscriber_id.clone());
        }
    }

    /// Releases a bundle claim without resolving.
    fn release_bundle(&self, subscriber_id: &SubscriberId, bundle_ref: BundleRef) {
        let state_lock = {
            let subscribers = self.subscribers.read().expect("subscribers lock poisoned");
            subscribers.get(subscriber_id).cloned()
        };

        if let Some(state_lock) = state_lock {
            let mut state = state_lock.write().expect("subscriber lock poisoned");
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
            .filter_map(|state_lock| {
                state_lock
                    .read()
                    .expect("subscriber lock poisoned")
                    .oldest_incomplete_segment()
            })
            .min()
    }

    /// Returns the minimum of the highest tracked segment across all active subscribers.
    ///
    /// When all tracked segments are complete (i.e., `oldest_incomplete_segment()` returns
    /// `None`), segments up to this point can be safely deleted.
    /// Returns `None` if there are no active subscribers with tracked segments.
    #[must_use]
    pub fn min_highest_tracked_segment(&self) -> Option<SegmentSeq> {
        self.subscribers
            .read()
            .expect("subscribers lock poisoned")
            .values()
            .filter_map(|state_lock| {
                let state = state_lock.read().expect("subscriber lock poisoned");
                if state.is_active() {
                    state.highest_tracked_segment()
                } else {
                    None
                }
            })
            .min()
    }

    /// Returns debug info about subscriber segment counts (for debugging).
    #[must_use]
    pub fn debug_subscriber_segment_counts(&self) -> Vec<(String, usize, bool)> {
        self.subscribers
            .read()
            .expect("subscribers lock poisoned")
            .iter()
            .map(|(id, state_lock)| {
                let state = state_lock.read().expect("subscriber lock poisoned");
                (id.as_str().to_string(), state.segment_count(), state.is_active())
            })
            .collect()
    }

    /// Cleans up internal tracking state for segments before the given sequence.
    ///
    /// Call this after deleting segment files to free memory tracking completed
    /// segments. This calls `remove_completed_segments_before` on each subscriber.
    pub fn cleanup_segments_before(&self, before: SegmentSeq) {
        let subscribers = self.subscribers.read().expect("subscribers lock poisoned");
        for state_lock in subscribers.values() {
            state_lock
                .write()
                .expect("subscriber lock poisoned")
                .remove_completed_segments_before(before);
        }
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
        let state_lock = {
            let subscribers = self.subscribers.read().expect("subscribers lock poisoned");
            subscribers.get(id).cloned()
        };

        state_lock.is_some_and(|lock| lock.read().expect("subscriber lock poisoned").is_active())
    }

    /// Flushes all dirty subscribers to their progress files.
    ///
    /// This should be called periodically by the embedding layer (e.g., every
    /// 25ms) to persist progress. Each subscriber's progress file is written
    /// atomically.
    ///
    /// # Returns
    ///
    /// The number of subscribers that were flushed.
    ///
    /// # Errors
    ///
    /// Returns an error if any progress file cannot be written. Partial flushes
    /// may occur—some subscribers may be persisted while others fail.
    pub fn flush_progress(&self) -> Result<usize> {
        // Take the dirty set
        let dirty: Vec<SubscriberId> = {
            let mut dirty_set = self.dirty_subscribers.lock().expect("dirty lock poisoned");
            dirty_set.drain().collect()
        };

        if dirty.is_empty() {
            return Ok(0);
        }

        // Collect data we need while holding read lock briefly
        let to_flush: Vec<(SubscriberId, Arc<RwLock<SubscriberState>>)> = {
            let subscribers = self.subscribers.read().expect("subscribers lock poisoned");
            dirty
                .into_iter()
                .filter_map(|sub_id| {
                    subscribers.get(&sub_id).map(|s| (sub_id, s.clone()))
                })
                .collect()
        };

        let mut flushed = 0;

        for (sub_id, state_lock) in to_flush {
            // Get state data with per-subscriber lock
            let (entries, oldest_incomplete) = {
                let state = state_lock.read().expect("subscriber lock poisoned");
                let entries = state.to_progress_entries();
                let oldest = state
                    .oldest_incomplete_segment()
                    .unwrap_or_else(|| SegmentSeq::new(0));
                (entries, oldest)
            };

            match write_progress_file(
                &self.config.data_dir,
                &sub_id,
                oldest_incomplete,
                &entries,
            ) {
                Ok(()) => {
                    flushed += 1;
                }
                Err(e) => {
                    // Re-add to dirty set and return error
                    let mut dirty_set =
                        self.dirty_subscribers.lock().expect("dirty lock poisoned");
                    let _ = dirty_set.insert(sub_id.clone());
                    return Err(e);
                }
            }
        }

        Ok(flushed)
    }

    /// Returns the number of subscribers with uncommitted progress.
    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.dirty_subscribers
            .lock()
            .expect("dirty lock poisoned")
            .len()
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

            // Flush progress files before closing
            registry.flush_progress().unwrap();
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

            // Flush progress files before closing
            registry.flush_progress().unwrap();
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
