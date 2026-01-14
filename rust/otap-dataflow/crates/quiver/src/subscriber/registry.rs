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
use std::sync::Arc;
use std::time::Duration;

use parking_lot::{Condvar, Mutex, RwLock};
use tokio::sync::Notify;

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
    /// Notify for waking waiting subscribers when new segments arrive (sync API).
    bundle_available: (Mutex<bool>, Condvar),
    /// Async notification for waking waiting subscribers when new segments arrive.
    bundle_available_async: Arc<Notify>,
}

impl<P: SegmentProvider> SubscriberRegistry<P> {
    /// Opens or creates a subscriber registry.
    ///
    /// Scans for existing progress files and loads subscriber state.
    ///
    /// # Errors
    ///
    /// Returns an error if progress files cannot be read or are corrupted.
    #[must_use = "the registry should be stored and used for subscriber management"]
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
            bundle_available: (Mutex::new(false), Condvar::new()),
            bundle_available_async: Arc::new(Notify::new()),
        });

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
        let mut subscribers = self.subscribers.write();

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
            let subscribers = self.subscribers.read();
            subscribers
                .get(id)
                .cloned()
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?
        };

        let mut state = state_lock.write();

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
            let subscribers = self.subscribers.read();
            subscribers
                .get(id)
                .cloned()
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?
        };

        let mut state = state_lock.write();
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
            let mut subscribers = self.subscribers.write();
            if subscribers.remove(id).is_none() {
                return Err(SubscriberError::not_found(id.as_str()));
            }
        }

        // Remove from dirty set
        {
            let mut dirty = self.dirty_subscribers.lock();
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
        let subscribers = self.subscribers.read();

        for state_lock in subscribers.values() {
            let mut state = state_lock.write();
            if state.is_active() {
                state.add_segment(segment_seq, bundle_count);
            }
        }

        // Notify waiting subscribers that new bundles are available (sync)
        let (lock, cvar) = &self.bundle_available;
        {
            let mut available = lock.lock();
            *available = true;
        }
        let _ = cvar.notify_all();

        // Notify async waiters
        self.bundle_available_async.notify_waiters();
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
            let subscribers = self.subscribers.read();
            subscribers
                .get(id)
                .cloned()
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?
        };

        let bundle_ref = {
            let mut state = state_lock.write();

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

    /// Returns the next pending bundle, blocking until one is available.
    ///
    /// This is more efficient than polling `next_bundle` in a loop with sleeps,
    /// as it uses a condvar to wait for new segment notifications.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscriber ID
    /// * `timeout` - Maximum time to wait for a bundle. If `None`, waits indefinitely.
    /// * `should_stop` - A function called periodically to check if waiting should stop
    ///   (e.g., for shutdown). If it returns `true`, returns `Ok(None)`.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(handle))` - A bundle is available
    /// * `Ok(None)` - Timeout expired or `should_stop` returned true
    /// * `Err(_)` - Subscriber not found or not active
    pub fn next_bundle_blocking<F>(
        self: &Arc<Self>,
        id: &SubscriberId,
        timeout: Option<Duration>,
        should_stop: F,
    ) -> Result<Option<BundleHandle<RegistryCallback<P>>>>
    where
        F: Fn() -> bool,
    {
        let deadline = timeout.map(|t| std::time::Instant::now() + t);

        loop {
            // Check for shutdown
            if should_stop() {
                return Ok(None);
            }

            // Try to get a bundle
            match self.next_bundle(id)? {
                Some(handle) => return Ok(Some(handle)),
                None => {
                    // No bundle available, wait for notification
                    let (lock, cvar) = &self.bundle_available;
                    let mut available = lock.lock();

                    // Check if data became available while acquiring lock
                    if *available {
                        *available = false;
                        continue;
                    }

                    // Calculate remaining timeout
                    let wait_duration = if let Some(deadline) = deadline {
                        let now = std::time::Instant::now();
                        if now >= deadline {
                            return Ok(None);
                        }
                        // Wait for at most 100ms to allow periodic shutdown checks
                        (deadline - now).min(Duration::from_millis(100))
                    } else {
                        // No timeout, but still check shutdown periodically
                        Duration::from_millis(100)
                    };

                    // parking_lot's wait_for takes &mut guard, doesn't consume it
                    let _ = cvar.wait_for(&mut available, wait_duration);

                    // Reset the flag since we're about to check
                    *available = false;
                }
            }
        }
    }

    /// Returns the next pending bundle, waiting asynchronously until one is available.
    ///
    /// This is the async version of [`next_bundle_blocking`](Self::next_bundle_blocking).
    /// It uses tokio's async notification instead of a condvar.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscriber ID
    /// * `timeout` - Maximum time to wait for a bundle. If `None`, waits indefinitely.
    /// * `cancellation` - A future that resolves when the operation should be cancelled
    ///   (e.g., for shutdown). Use `std::future::pending()` if no cancellation is needed.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(handle))` - A bundle is available
    /// * `Ok(None)` - Timeout expired or cancellation requested
    /// * `Err(_)` - Subscriber not found or not active
    pub async fn next_bundle_async(
        self: &Arc<Self>,
        id: &SubscriberId,
        timeout: Option<Duration>,
    ) -> Result<Option<BundleHandle<RegistryCallback<P>>>> {
        let deadline = timeout.map(|t| tokio::time::Instant::now() + t);
        let notify = self.bundle_available_async.clone();

        loop {
            // Try to get a bundle
            match self.next_bundle(id)? {
                Some(handle) => return Ok(Some(handle)),
                None => {
                    // No bundle available, wait for notification or timeout
                    let wait_future = notify.notified();

                    if let Some(deadline) = deadline {
                        let now = tokio::time::Instant::now();
                        if now >= deadline {
                            return Ok(None);
                        }

                        // Wait for notification or timeout
                        match tokio::time::timeout_at(deadline, wait_future).await {
                            Ok(()) => {
                                // Notified, loop around to try getting a bundle
                            }
                            Err(_) => {
                                // Timeout expired
                                return Ok(None);
                            }
                        }
                    } else {
                        // No timeout, just wait for notification
                        wait_future.await;
                    }
                }
            }
        }
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
            let subscribers = self.subscribers.read();
            subscribers
                .get(id)
                .cloned()
                .ok_or_else(|| SubscriberError::not_found(id.as_str()))?
        };

        {
            let mut state = state_lock.write();

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
            let subscribers = self.subscribers.read();
            subscribers.get(subscriber_id).cloned()
        };

        // Update in-memory state
        if let Some(state_lock) = state_lock {
            let mut state = state_lock.write();
            let _ = state.record_outcome(bundle_ref, outcome);
        }

        // Mark subscriber as dirty (needs flushing)
        {
            let mut dirty = self.dirty_subscribers.lock();
            let _ = dirty.insert(subscriber_id.clone());
        }
    }

    /// Releases a bundle claim without resolving.
    fn release_bundle(&self, subscriber_id: &SubscriberId, bundle_ref: BundleRef) {
        let state_lock = {
            let subscribers = self.subscribers.read();
            subscribers.get(subscriber_id).cloned()
        };

        if let Some(state_lock) = state_lock {
            let mut state = state_lock.write();
            state.release(bundle_ref);
        }
    }

    /// Returns the number of registered subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.read().len()
    }

    /// Returns the oldest incomplete segment across all subscribers.
    ///
    /// Segments older than this can be safely deleted (all subscribers have
    /// completed them).
    #[must_use]
    pub fn oldest_incomplete_segment(&self) -> Option<SegmentSeq> {
        self.subscribers
            .read()
            .values()
            .filter_map(|state_lock| state_lock.read().oldest_incomplete_segment())
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
            .values()
            .filter_map(|state_lock| {
                let state = state_lock.read();
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
            .iter()
            .map(|(id, state_lock)| {
                let state = state_lock.read();
                (
                    id.as_str().to_string(),
                    state.segment_count(),
                    state.is_active(),
                )
            })
            .collect()
    }

    /// Cleans up internal tracking state for segments before the given sequence.
    ///
    /// Call this after deleting segment files to free memory tracking completed
    /// segments. This calls `remove_completed_segments_before` on each subscriber.
    pub fn cleanup_segments_before(&self, before: SegmentSeq) {
        let subscribers = self.subscribers.read();
        for state_lock in subscribers.values() {
            state_lock.write().remove_completed_segments_before(before);
        }
    }

    /// Returns whether a subscriber is registered.
    #[must_use]
    pub fn is_registered(&self, id: &SubscriberId) -> bool {
        self.subscribers.read().contains_key(id)
    }

    /// Returns whether a subscriber is active.
    #[must_use]
    pub fn is_active(&self, id: &SubscriberId) -> bool {
        let state_lock = {
            let subscribers = self.subscribers.read();
            subscribers.get(id).cloned()
        };

        state_lock.is_some_and(|lock| lock.read().is_active())
    }

    /// Flushes all dirty subscribers to their progress files.
    ///
    /// This should be called periodically by the embedding layer (e.g., every
    /// 25ms) to persist progress. Each subscriber's progress file is written
    /// atomically.
    ///
    /// # Best-Effort Semantics
    ///
    /// This method uses best-effort semantics: if one subscriber fails to flush,
    /// the method continues attempting to flush remaining subscribers. Failed
    /// subscribers are re-added to the dirty set for retry on the next call.
    ///
    /// # Returns
    ///
    /// On success, returns the number of subscribers that were flushed.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered after attempting all flushes.
    /// Check the dirty count after an error to see how many subscribers still
    /// need flushing.
    pub fn flush_progress(&self) -> Result<usize> {
        // Take the dirty set
        let dirty: Vec<SubscriberId> = {
            let mut dirty_set = self.dirty_subscribers.lock();
            dirty_set.drain().collect()
        };

        if dirty.is_empty() {
            return Ok(0);
        }

        // Collect data we need while holding read lock briefly
        let to_flush: Vec<(SubscriberId, Arc<RwLock<SubscriberState>>)> = {
            let subscribers = self.subscribers.read();
            dirty
                .into_iter()
                .filter_map(|sub_id| subscribers.get(&sub_id).map(|s| (sub_id, s.clone())))
                .collect()
        };

        let mut flushed = 0;
        let mut first_error: Option<SubscriberError> = None;

        for (sub_id, state_lock) in to_flush {
            // Get state data with per-subscriber lock
            let (entries, oldest_incomplete) = {
                let state = state_lock.read();
                let entries = state.to_progress_entries();
                let oldest = state
                    .oldest_incomplete_segment()
                    .unwrap_or_else(|| SegmentSeq::new(0));
                (entries, oldest)
            };

            match write_progress_file(&self.config.data_dir, &sub_id, oldest_incomplete, &entries) {
                Ok(()) => {
                    flushed += 1;
                }
                Err(e) => {
                    // Re-add to dirty set for retry
                    let mut dirty_set = self.dirty_subscribers.lock();
                    let _ = dirty_set.insert(sub_id.clone());
                    // Keep the first error, continue with remaining subscribers
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
            }
        }

        // Return first error if any occurred, otherwise success
        match first_error {
            Some(e) => Err(e),
            None => Ok(flushed),
        }
    }

    /// Returns the number of subscribers with uncommitted progress.
    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.dirty_subscribers.lock().len()
    }

    /// Force-drops the oldest pending segments that have no active readers.
    ///
    /// This is used by the `DropOldest` retention policy to reclaim disk space
    /// by forcibly completing segments that no subscriber is currently reading.
    ///
    /// A segment is considered to have "no active readers" if no subscriber
    /// has any claimed bundles from that segment.
    ///
    /// Returns the list of segment sequences that were force-completed and
    /// can be safely deleted from disk.
    pub fn force_drop_oldest_pending_segments(&self) -> Vec<SegmentSeq> {
        let subscribers = self.subscribers.read();

        // Build a set of all pending (incomplete) segments across all subscribers
        // and track which segments have claimed bundles
        let mut pending_segments: std::collections::BTreeSet<SegmentSeq> =
            std::collections::BTreeSet::new();
        let mut segments_with_readers: HashSet<SegmentSeq> = HashSet::new();

        for state_lock in subscribers.values() {
            let state = state_lock.read();

            // Collect all incomplete segments from progress entries
            for entry in state.to_progress_entries() {
                if !entry.is_complete() {
                    let _ = pending_segments.insert(entry.seg_seq);
                }
            }

            // Check which segments have active readers (claimed bundles)
            for segment_seq in &pending_segments {
                if state.has_claimed_in_segment(*segment_seq) {
                    let _ = segments_with_readers.insert(*segment_seq);
                }
            }
        }

        // Find segments with no active readers (in oldest-first order)
        let droppable: Vec<SegmentSeq> = pending_segments
            .into_iter()
            .filter(|seg| !segments_with_readers.contains(seg))
            .collect();

        if droppable.is_empty() {
            return Vec::new();
        }

        // Take only the oldest segment to drop (be conservative)
        let to_drop = vec![droppable[0]];

        // Force-complete these segments for all subscribers
        for state_lock in subscribers.values() {
            let mut state = state_lock.write();
            let mut any_completed = false;

            // Process all segments before releasing the lock
            for &segment_seq in &to_drop {
                if state.force_complete_segment(segment_seq) {
                    any_completed = true;
                }
            }

            // Mark as dirty if any segments were completed
            if any_completed {
                let id = state.id().clone();
                drop(state); // Release write lock before acquiring dirty lock
                let mut dirty_set = self.dirty_subscribers.lock();
                let _ = dirty_set.insert(id);
            }
        }

        to_drop
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

    #[test]
    fn next_bundle_blocking_returns_immediately_when_available() {
        let (registry, _dir) = setup_registry();
        let provider = registry.segment_provider.clone();
        provider.add_segment(1, 2);

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        // Should return immediately since bundle is available
        let start = std::time::Instant::now();
        let handle = registry
            .next_bundle_blocking(&id, Some(Duration::from_secs(5)), || false)
            .unwrap()
            .unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(100),
            "Should return immediately"
        );
        assert_eq!(handle.bundle_ref().bundle_index, BundleIndex::new(0));
        handle.ack();
    }

    #[test]
    fn next_bundle_blocking_respects_timeout() {
        let (registry, _dir) = setup_registry();
        // No segments added - no bundles available

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        // Should timeout after ~200ms
        let start = std::time::Instant::now();
        let result = registry
            .next_bundle_blocking(&id, Some(Duration::from_millis(200)), || false)
            .unwrap();
        let elapsed = start.elapsed();

        assert!(result.is_none(), "Should return None on timeout");
        assert!(
            elapsed >= Duration::from_millis(200),
            "Should wait for timeout"
        );
        assert!(
            elapsed < Duration::from_millis(500),
            "Should not wait too long"
        );
    }

    #[test]
    fn next_bundle_blocking_respects_stop_condition() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let (registry, _dir) = setup_registry();
        // No segments added - no bundles available

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        let should_stop = Arc::new(AtomicBool::new(false));
        let stop_flag = should_stop.clone();

        // Spawn a thread that will set should_stop after a short delay
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            stop_flag.store(true, Ordering::Relaxed);
        });

        // Should stop when the flag is set (checked every 100ms)
        let start = std::time::Instant::now();
        let result = registry
            .next_bundle_blocking(&id, None, || should_stop.load(Ordering::Relaxed))
            .unwrap();
        let elapsed = start.elapsed();

        assert!(result.is_none(), "Should return None when stopped");
        assert!(
            elapsed >= Duration::from_millis(100),
            "Should wait at least one cycle"
        );
        assert!(
            elapsed < Duration::from_millis(500),
            "Should stop reasonably quickly"
        );
    }

    #[test]
    fn next_bundle_blocking_wakes_on_segment_finalized() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let (registry, _dir) = setup_registry();
        let provider = registry.segment_provider.clone();
        // Start with no segments

        let id = SubscriberId::new("test-sub").unwrap();
        registry.register(id.clone()).unwrap();
        registry.activate(&id).unwrap();

        let got_bundle = Arc::new(AtomicBool::new(false));
        let got_bundle_clone = got_bundle.clone();
        let registry_clone = registry.clone();
        let id_clone = id.clone();

        // Spawn consumer thread that will block waiting for a bundle
        let consumer = std::thread::spawn(move || {
            let result = registry_clone
                .next_bundle_blocking(&id_clone, Some(Duration::from_secs(5)), || false)
                .unwrap();
            if result.is_some() {
                got_bundle_clone.store(true, Ordering::Relaxed);
            }
        });

        // Wait a bit, then add a segment
        std::thread::sleep(Duration::from_millis(100));
        provider.add_segment(1, 1);
        registry.on_segment_finalized(SegmentSeq::new(1), 1);

        // Consumer should wake up and get the bundle
        consumer.join().unwrap();
        assert!(
            got_bundle.load(Ordering::Relaxed),
            "Consumer should have received bundle"
        );
    }
}
