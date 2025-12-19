// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Subscriber simulation with failure/recovery scenarios.
//!
//! Simulates real-world subscriber behavior including:
//! - Normal consumption
//! - Network failures mid-stream
//! - Recovery and retry from last known position

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use quiver::SegmentStore;
use quiver::segment::SegmentSeq;
use quiver::subscriber::SegmentProvider;
use rand::Rng;
use tracing::{debug, info, warn};

/// Result of a subscriber consumption run.
#[derive(Debug)]
pub struct ConsumptionResult {
    /// Total bundles successfully consumed
    pub consumed: usize,
    /// Number of retries due to simulated failures
    pub retries: usize,
    /// Number of simulated failure events
    pub failures: usize,
}

/// Consumes all bundles with simulated network failures and recovery.
///
/// This simulates a realistic subscriber that:
/// 1. Consumes bundles in order
/// 2. May experience "network failures" (random disconnects)
/// 3. Recovers and resumes from where it left off
/// 4. Tracks which bundles have been successfully processed
pub fn consume_with_failures(
    store: &SegmentStore,
    segments: &[(SegmentSeq, u32)],
    output_path: &Path,
    subscriber_name: &str,
    simulate_failures: bool,
    failure_probability: f64,
) -> Result<ConsumptionResult, Box<dyn std::error::Error>> {
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
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        // Integration tests would require full engine setup
    }
}
