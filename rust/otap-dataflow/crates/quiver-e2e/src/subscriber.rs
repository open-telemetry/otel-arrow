// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Subscriber simulation with failure/recovery scenarios.
//!
//! Simulates real-world subscriber behavior including:
//! - Normal consumption via SubscriberRegistry
//! - Network failures mid-stream
//! - Recovery and retry from last known position
//! - Periodic progress file flushing

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

use quiver::SegmentStore;
use quiver::segment::SegmentSeq;
use quiver::subscriber::{RegistryConfig, SegmentProvider, SubscriberId, SubscriberRegistry};
use rand::Rng;
use tracing::{debug, info, warn};

/// Delay configuration for simulating slow subscribers.
#[derive(Debug, Clone, Copy, Default)]
pub struct SubscriberDelay {
    /// Delay in milliseconds per bundle consumed
    pub per_bundle_ms: u64,
}

impl SubscriberDelay {
    /// Creates a new delay configuration.
    pub fn new(per_bundle_ms: u64) -> Self {
        Self { per_bundle_ms }
    }

    /// Applies the delay if configured.
    pub fn apply(&self) {
        if self.per_bundle_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.per_bundle_ms));
        }
    }
}

/// Result of a subscriber consumption run.
#[derive(Debug)]
pub struct ConsumptionResult {
    /// Total bundles successfully consumed
    pub consumed: usize,
    /// Number of retries due to simulated failures
    pub retries: usize,
    /// Number of simulated failure events
    pub failures: usize,
    /// Number of progress flushes performed
    pub flushes: usize,
}

/// Adapter to make SegmentStore implement SegmentProvider.
struct StoreProvider {
    store: Arc<SegmentStore>,
    segments: Vec<(SegmentSeq, u32)>,
}

impl SegmentProvider for StoreProvider {
    fn bundle_count(&self, segment_seq: SegmentSeq) -> quiver::subscriber::Result<u32> {
        self.segments
            .iter()
            .find(|(s, _)| *s == segment_seq)
            .map(|(_, count)| *count)
            .ok_or_else(|| {
                quiver::subscriber::SubscriberError::segment_not_found(segment_seq.raw())
            })
    }

    fn read_bundle(
        &self,
        bundle_ref: quiver::subscriber::BundleRef,
    ) -> quiver::subscriber::Result<quiver::segment::ReconstructedBundle> {
        self.store.read_bundle(bundle_ref).map_err(|e| {
            quiver::subscriber::SubscriberError::segment_io(
                std::path::PathBuf::from(format!(
                    "seg_{:016x}.bin",
                    bundle_ref.segment_seq.raw()
                )),
                std::io::Error::other(e.to_string()),
            )
        })
    }

    fn available_segments(&self) -> Vec<SegmentSeq> {
        self.segments.iter().map(|(s, _)| *s).collect()
    }
}

/// Consumes all bundles using SubscriberRegistry with progress file persistence.
///
/// This simulates a realistic subscriber that:
/// 1. Uses SubscriberRegistry for proper progress tracking
/// 2. May experience "network failures" (random disconnects)
/// 3. Calls flush_progress() periodically to persist state
/// 4. Recovers and resumes from where it left off
/// 5. Optionally delays consumption to simulate slow egress
#[allow(clippy::too_many_arguments)]
pub fn consume_with_registry(
    store: Arc<SegmentStore>,
    segments: &[(SegmentSeq, u32)],
    data_dir: &Path,
    output_path: &Path,
    subscriber_name: &str,
    simulate_failures: bool,
    failure_probability: f64,
    flush_interval: usize,
    delay: SubscriberDelay,
) -> Result<ConsumptionResult, Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "=== {} Export ===", subscriber_name)?;
    writeln!(writer, "Timestamp: {:?}", std::time::SystemTime::now())?;
    writeln!(writer)?;

    let mut rng = rand::rng();

    // Create registry with the segment store as provider
    let provider = Arc::new(StoreProvider {
        store: store.clone(),
        segments: segments.to_vec(),
    });
    let registry_config = RegistryConfig::new(data_dir);
    let registry = SubscriberRegistry::open(registry_config, provider)?;

    // Register and activate our subscriber
    let sub_id = SubscriberId::new(subscriber_name)?;
    registry.register(sub_id.clone())?;
    registry.activate(&sub_id)?;

    // Initialize segments in the registry
    for (seq, bundle_count) in segments {
        registry.on_segment_finalized(*seq, *bundle_count);
    }

    let mut total_consumed = 0;
    let mut total_retries = 0;
    let mut total_failures = 0;
    let mut total_flushes = 0;
    let mut bundles_since_flush = 0;

    // State machine: simulate going offline and recovering
    let mut is_online = true;
    let mut offline_bundles_remaining = 0;

    // Consume bundles via the registry
    loop {
        // Simulate network state changes
        if simulate_failures && is_online && rng.random_bool(failure_probability) {
            is_online = false;
            offline_bundles_remaining = rng.random_range(1..=10);
            total_failures += 1;
            debug!(
                subscriber = subscriber_name,
                offline_for = offline_bundles_remaining,
                "Simulated network failure"
            );
        }

        // Get next bundle from registry
        let handle = match registry.next_bundle(&sub_id)? {
            Some(h) => h,
            None => break, // No more bundles
        };

        let bundle_ref = handle.bundle_ref();

        if !is_online {
            // Simulate being offline - defer this bundle for later
            let _ = handle.defer();
            offline_bundles_remaining -= 1;
            if offline_bundles_remaining == 0 {
                is_online = true;
                debug!(subscriber = subscriber_name, "Network recovered");
            }
            total_retries += 1;
            continue;
        }

        // Normal consumption - read the actual bundle data
        match store.read_bundle(bundle_ref) {
            Ok(bundle) => {
                // Apply subscriber delay (simulates slow egress)
                delay.apply();

                writeln!(
                    writer,
                    "Seg {} Bundle {}: {} slots, {} total rows",
                    bundle_ref.segment_seq.raw(),
                    bundle_ref.bundle_index.raw(),
                    bundle.slot_count(),
                    bundle
                        .payloads()
                        .values()
                        .map(|b| b.num_rows())
                        .sum::<usize>()
                )?;

                // Acknowledge the bundle
                handle.ack();
                total_consumed += 1;
                bundles_since_flush += 1;

                // Periodic flush (simulating the embedding layer's flush interval)
                if bundles_since_flush >= flush_interval {
                    let flushed = registry.flush_progress()?;
                    if flushed > 0 {
                        total_flushes += 1;
                        debug!(
                            subscriber = subscriber_name,
                            flushed = flushed,
                            "Flushed progress"
                        );
                    }
                    bundles_since_flush = 0;
                }
            }
            Err(e) => {
                warn!(
                    subscriber = subscriber_name,
                    segment = bundle_ref.segment_seq.raw(),
                    bundle = bundle_ref.bundle_index.raw(),
                    error = %e,
                    "Failed to read bundle, rejecting"
                );
                handle.reject();
            }
        }
    }

    // Recovery pass: retry any bundles that were deferred
    let mut recovery_iterations = 0;
    const MAX_RECOVERY_ITERATIONS: usize = 100;

    while let Some(handle) = registry.next_bundle(&sub_id)? {
        recovery_iterations += 1;
        if recovery_iterations > MAX_RECOVERY_ITERATIONS {
            warn!(
                subscriber = subscriber_name,
                "Max recovery iterations reached"
            );
            let _ = handle.defer();
            break;
        }

        let bundle_ref = handle.bundle_ref();

        match store.read_bundle(bundle_ref) {
            Ok(bundle) => {
                writeln!(
                    writer,
                    "RECOVERED Seg {} Bundle {}: {} slots",
                    bundle_ref.segment_seq.raw(),
                    bundle_ref.bundle_index.raw(),
                    bundle.slot_count()
                )?;
                handle.ack();
                total_consumed += 1;
            }
            Err(e) => {
                warn!(
                    subscriber = subscriber_name,
                    segment = bundle_ref.segment_seq.raw(),
                    bundle = bundle_ref.bundle_index.raw(),
                    error = %e,
                    "Failed to recover bundle"
                );
                handle.reject();
            }
        }
    }

    // Final flush before shutdown
    let flushed = registry.flush_progress()?;
    if flushed > 0 {
        total_flushes += 1;
        debug!(
            subscriber = subscriber_name,
            flushed = flushed,
            "Final progress flush"
        );
    }

    writeln!(writer)?;
    writeln!(writer, "=== Summary ===")?;
    writeln!(writer, "Total consumed: {}", total_consumed)?;
    writeln!(writer, "Failures simulated: {}", total_failures)?;
    writeln!(writer, "Bundles retried: {}", total_retries)?;
    writeln!(writer, "Progress flushes: {}", total_flushes)?;

    writer.flush()?;

    info!(
        subscriber = subscriber_name,
        consumed = total_consumed,
        failures = total_failures,
        retries = total_retries,
        flushes = total_flushes,
        "Consumption complete"
    );

    Ok(ConsumptionResult {
        consumed: total_consumed,
        retries: total_retries,
        failures: total_failures,
        flushes: total_flushes,
    })
}

/// Legacy function that uses direct segment store access (no registry).
///
/// Kept for comparison testing.
pub fn consume_with_failures(
    store: &SegmentStore,
    segments: &[(SegmentSeq, u32)],
    output_path: &Path,
    subscriber_name: &str,
    simulate_failures: bool,
    failure_probability: f64,
) -> Result<ConsumptionResult, Box<dyn std::error::Error>> {
    use std::collections::HashSet;

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "=== {} Export ===", subscriber_name)?;
    writeln!(writer, "Timestamp: {:?}", std::time::SystemTime::now())?;
    writeln!(writer)?;

    let mut rng = rand::rng();

    // Track consumed bundles (simulating durable ack state)
    let mut consumed_set: HashSet<(u64, u32)> = HashSet::new();
    let mut total_consumed = 0;
    let mut total_retries = 0;
    let mut total_failures = 0;

    // State machine: simulate going offline and recovering
    let mut is_online = true;
    let mut offline_bundles_remaining = 0;

    for (seq, bundle_count) in segments {
        for idx in 0..*bundle_count {
            let bundle_key = (seq.raw(), idx);

            // Skip if already consumed (recovery scenario)
            if consumed_set.contains(&bundle_key) {
                continue;
            }

            // Simulate network state changes
            if simulate_failures && is_online && rng.random_bool(failure_probability) {
                // Go offline for a random number of bundles (1-10)
                is_online = false;
                offline_bundles_remaining = rng.random_range(1..=10);
                total_failures += 1;
                debug!(
                    subscriber = subscriber_name,
                    segment = seq.raw(),
                    bundle = idx,
                    offline_for = offline_bundles_remaining,
                    "Simulated network failure"
                );
            }

            if !is_online {
                // Simulate being offline - skip this bundle, will retry later
                offline_bundles_remaining -= 1;
                if offline_bundles_remaining == 0 {
                    is_online = true;
                    debug!(
                        subscriber = subscriber_name,
                        segment = seq.raw(),
                        bundle = idx,
                        "Network recovered"
                    );
                }
                total_retries += 1;
                continue;
            }

            // Normal consumption
            let bundle_ref = quiver::subscriber::BundleRef {
                segment_seq: *seq,
                bundle_index: quiver::subscriber::BundleIndex::new(idx),
            };

            match store.read_bundle(bundle_ref) {
                Ok(bundle) => {
                    // Write summary to export file
                    writeln!(
                        writer,
                        "Seg {} Bundle {}: {} slots, {} total rows",
                        seq.raw(),
                        idx,
                        bundle.slot_count(),
                        bundle
                            .payloads()
                            .values()
                            .map(|b| b.num_rows())
                            .sum::<usize>()
                    )?;

                    // Mark as consumed
                    let _ = consumed_set.insert(bundle_key);
                    total_consumed += 1;
                }
                Err(e) => {
                    warn!(
                        subscriber = subscriber_name,
                        segment = seq.raw(),
                        bundle = idx,
                        error = %e,
                        "Failed to read bundle"
                    );
                }
            }
        }
    }

    // Recovery pass: retry any bundles that were skipped due to failures
    if total_retries > 0 {
        info!(
            subscriber = subscriber_name,
            retries = total_retries,
            "Running recovery pass for missed bundles"
        );

        for (seq, bundle_count) in segments {
            for idx in 0..*bundle_count {
                let bundle_key = (seq.raw(), idx);

                if consumed_set.contains(&bundle_key) {
                    continue;
                }

                let bundle_ref = quiver::subscriber::BundleRef {
                    segment_seq: *seq,
                    bundle_index: quiver::subscriber::BundleIndex::new(idx),
                };

                match store.read_bundle(bundle_ref) {
                    Ok(bundle) => {
                        writeln!(
                            writer,
                            "RECOVERED Seg {} Bundle {}: {} slots",
                            seq.raw(),
                            idx,
                            bundle.slot_count()
                        )?;
                        let _ = consumed_set.insert(bundle_key);
                        total_consumed += 1;
                    }
                    Err(e) => {
                        warn!(
                            subscriber = subscriber_name,
                            segment = seq.raw(),
                            bundle = idx,
                            error = %e,
                            "Failed to recover bundle"
                        );
                    }
                }
            }
        }
    }

    writeln!(writer)?;
    writeln!(writer, "=== Summary ===")?;
    writeln!(writer, "Total consumed: {}", total_consumed)?;
    writeln!(writer, "Failures simulated: {}", total_failures)?;
    writeln!(writer, "Bundles retried: {}", total_retries)?;

    writer.flush()?;

    info!(
        subscriber = subscriber_name,
        consumed = total_consumed,
        failures = total_failures,
        retries = total_retries,
        "Consumption complete"
    );

    Ok(ConsumptionResult {
        consumed: total_consumed,
        retries: total_retries,
        failures: total_failures,
        flushes: 0,
    })
}

/// Wrapper around SegmentStore that implements SegmentProvider.
/// 
/// Can be used to create a shared registry for multiple subscribers.
#[allow(dead_code)]
pub struct SharedStoreProvider {
    store: Arc<SegmentStore>,
    /// Known segments and their bundle counts.
    segments: std::sync::RwLock<Vec<(SegmentSeq, u32)>>,
}

#[allow(dead_code)]
impl SharedStoreProvider {
    /// Creates a new shared store provider.
    pub fn new(store: Arc<SegmentStore>) -> Arc<Self> {
        Arc::new(Self {
            store,
            segments: std::sync::RwLock::new(Vec::new()),
        })
    }

    /// Updates the known segments list from scanning the store.
    pub fn refresh_segments(&self) -> Result<(), Box<dyn std::error::Error>> {
        let found = self.store.scan_existing()?;
        let mut segments = self.segments.write().expect("lock poisoned");
        *segments = found;
        Ok(())
    }

    /// Adds a newly finalized segment.
    pub fn add_segment(&self, seq: SegmentSeq, bundle_count: u32) {
        let mut segments = self.segments.write().expect("lock poisoned");
        if !segments.iter().any(|(s, _)| *s == seq) {
            segments.push((seq, bundle_count));
            segments.sort_by_key(|(s, _)| *s);
        }
    }
}

impl SegmentProvider for SharedStoreProvider {
    fn bundle_count(&self, segment_seq: SegmentSeq) -> quiver::subscriber::Result<u32> {
        self.segments
            .read()
            .expect("lock poisoned")
            .iter()
            .find(|(s, _)| *s == segment_seq)
            .map(|(_, count)| *count)
            .ok_or_else(|| {
                quiver::subscriber::SubscriberError::segment_not_found(segment_seq.raw())
            })
    }

    fn read_bundle(
        &self,
        bundle_ref: quiver::subscriber::BundleRef,
    ) -> quiver::subscriber::Result<quiver::segment::ReconstructedBundle> {
        self.store.read_bundle(bundle_ref).map_err(|e| {
            quiver::subscriber::SubscriberError::segment_io(
                std::path::PathBuf::from(format!(
                    "seg_{:016x}.bin",
                    bundle_ref.segment_seq.raw()
                )),
                std::io::Error::other(e.to_string()),
            )
        })
    }

    fn available_segments(&self) -> Vec<SegmentSeq> {
        self.segments
            .read()
            .expect("lock poisoned")
            .iter()
            .map(|(s, _)| *s)
            .collect()
    }
}

/// Cleans up completed segments from the store.
///
/// This function:
/// 1. Queries the registry for the oldest incomplete segment
/// 2. Deletes all segments older than that from the store
/// 3. Cleans up registry internal state
///
/// Returns the number of segments deleted.
pub fn cleanup_completed_segments<P: SegmentProvider>(
    registry: &SubscriberRegistry<P>,
    store: &SegmentStore,
) -> std::io::Result<usize> {
    // Get the oldest incomplete segment across all subscribers.
    // This tells us the boundary - all segments before this are complete.
    let delete_boundary = match registry.oldest_incomplete_segment() {
        Some(seq) => {
            // Normal case: delete segments strictly before the oldest incomplete
            seq
        }
        None => {
            // No incomplete segments. This could mean:
            // 1. No subscribers or no segments tracked yet - should not delete
            // 2. All tracked segments are complete - can delete up to and including highest
            //
            // Use min_highest_tracked_segment() to get the safe upper bound.
            // If it returns a value, we can delete all segments up to and including it.
            match registry.min_highest_tracked_segment() {
                Some(highest) => {
                    // All segments up through `highest` are complete. We want to delete
                    // these segments, so set boundary to highest + 1.
                    highest.next()
                }
                None => {
                    // No active subscribers tracking segments - can't determine what's safe
                    return Ok(0);
                }
            }
        }
    };

    let mut deleted = 0;
    for seq in store.segment_sequences() {
        // Only delete segments older than the boundary
        if seq < delete_boundary {
            match store.delete_segment(seq) {
                Ok(()) => {
                    deleted += 1;
                    debug!(segment = seq.raw(), "Deleted completed segment");
                }
                Err(e) => {
                    warn!(segment = seq.raw(), error = %e, "Failed to delete segment");
                }
            }
        }
    }

    // Clean up registry internal state for deleted segments
    registry.cleanup_segments_before(delete_boundary);

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        // Integration tests would require full engine setup
    }
}
