// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Quiver Stress Test Harness
//!
//! This binary exercises the complete persistence flow with:
//! - Configurable data volume and bundle sizes
//! - Memory consumption tracking (via jemalloc)
//! - Multiple concurrent subscribers
//! - Network offline + recovery simulation
//! - Memory-mapped segment reading with comparison mode
//!
//! Usage:
//!   cargo run -p quiver-e2e -- --help
//!   cargo run -p quiver-e2e -- --bundles 1000 --subscribers 3

mod bundle;
mod memory;
mod subscriber;

use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use quiver::segment_store::SegmentReadMode;
use quiver::{QuiverConfig, QuiverEngine, SegmentStore};
use tempfile::tempdir;
use tracing::{Level, info, warn};
use tracing_subscriber::FmtSubscriber;

use crate::memory::MemoryTracker;

// Use jemalloc for accurate memory tracking
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Results from running a single read mode.
struct ModeResult {
    mode_name: String,
    scan_duration: Duration,
    consume_duration: Duration,
    subscriber_results: Vec<subscriber::ConsumptionResult>,
    total_bundles: u32,
    segment_count: usize,
}

/// Read mode for segment files
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum ReadModeArg {
    /// Standard file I/O
    Standard,
    /// Memory-mapped I/O
    #[default]
    Mmap,
    /// Run both modes and compare
    Compare,
}

/// Quiver stress test configuration
#[derive(Parser, Debug)]
#[command(name = "quiver-e2e")]
#[command(about = "Stress test harness for Quiver persistence layer")]
struct Args {
    /// Number of bundles to ingest
    #[arg(short, long, default_value = "1000")]
    bundles: usize,

    /// Number of rows per bundle (per slot)
    #[arg(short = 'r', long, default_value = "100")]
    rows_per_bundle: usize,

    /// Average size of string values in bytes
    #[arg(long, default_value = "32")]
    string_size: usize,

    /// Number of concurrent subscribers
    #[arg(short, long, default_value = "3")]
    subscribers: usize,

    /// Target segment size in MB
    #[arg(short = 't', long, default_value = "32")]
    segment_size_mb: u64,

    /// Simulate network failures (subscriber goes offline mid-stream)
    #[arg(long, default_value = "true")]
    simulate_failures: bool,

    /// Probability of simulated failure (0.0-1.0)
    #[arg(long, default_value = "0.1")]
    failure_probability: f64,

    /// Segment read mode (standard, mmap, or compare)
    #[arg(long, default_value = "mmap")]
    read_mode: ReadModeArg,

    /// Keep temp directory after test (for inspection)
    #[arg(long)]
    keep_temp: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing
    let tracing_sub = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(tracing_sub);

    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║           Quiver Stress Test Harness                       ║");
    info!("╚════════════════════════════════════════════════════════════╝");
    info!("");
    info!("Configuration:");
    info!("  Bundles to ingest: {}", args.bundles);
    info!("  Rows per bundle: {}", args.rows_per_bundle);
    info!("  String value size: {} bytes", args.string_size);
    info!("  Subscribers: {}", args.subscribers);
    info!("  Target segment size: {} MB", args.segment_size_mb);
    info!("  Simulate failures: {}", args.simulate_failures);
    info!(
        "  Failure probability: {:.1}%",
        args.failure_probability * 100.0
    );
    info!("  Read mode: {:?}", args.read_mode);
    info!("");

    // Initialize memory tracking
    let mut mem_tracker = MemoryTracker::new();
    mem_tracker.checkpoint("startup");

    // Create temp directory for test data
    let tmp = tempdir()?;
    let data_dir = tmp.path().to_path_buf();
    info!(path = %data_dir.display(), "Created temp directory");

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 1: Generate test data (not counted in memory tracking)
    // ─────────────────────────────────────────────────────────────────────────
    info!("");
    info!("═══ Phase 1: Generating test data ═══");

    let bundles =
        bundle::generate_test_bundles(args.bundles, args.rows_per_bundle, args.string_size);
    info!(count = bundles.len(), "Generated test bundles");

    // Reset memory baseline after bundle creation
    mem_tracker.reset_baseline();
    mem_tracker.checkpoint("after_bundle_generation");

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 2: Ingest data into Quiver
    // ─────────────────────────────────────────────────────────────────────────
    info!("");
    info!("═══ Phase 2: Ingesting data ═══");

    let config = create_config(&data_dir, args.segment_size_mb);
    let engine = Arc::new(QuiverEngine::new(config)?);
    mem_tracker.checkpoint("engine_created");

    let ingest_start = Instant::now();
    for (i, test_bundle) in bundles.iter().enumerate() {
        engine.ingest(test_bundle)?;
        if (i + 1) % 100 == 0 {
            mem_tracker.checkpoint_silent(&format!("ingest_{}", i + 1));
        }
    }
    let ingest_duration = ingest_start.elapsed();

    let total_rows = args.bundles * args.rows_per_bundle;
    info!(
        bundles = args.bundles,
        rows = total_rows,
        duration_ms = ingest_duration.as_millis(),
        bundles_per_sec = format!("{:.0}", args.bundles as f64 / ingest_duration.as_secs_f64()),
        rows_per_sec = format!("{:.0}", total_rows as f64 / ingest_duration.as_secs_f64()),
        "Ingestion complete"
    );
    mem_tracker.checkpoint("after_ingestion");

    // Finalize remaining data
    engine.shutdown()?;
    mem_tracker.checkpoint("after_shutdown");
    info!("Engine shutdown complete");

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 3: Create segment store and scan segments
    // ─────────────────────────────────────────────────────────────────────────
    info!("");
    info!("═══ Phase 3: Scanning segments ═══");

    let segment_dir = data_dir.join("segments");

    // Determine which modes to test
    let modes_to_test: Vec<(SegmentReadMode, &str)> = match args.read_mode {
        ReadModeArg::Standard => vec![(SegmentReadMode::Standard, "standard")],
        ReadModeArg::Mmap => vec![(SegmentReadMode::Mmap, "mmap")],
        ReadModeArg::Compare => vec![
            (SegmentReadMode::Standard, "standard"),
            (SegmentReadMode::Mmap, "mmap"),
        ],
    };

    let mut mode_results: Vec<ModeResult> = Vec::new();

    for (mode, mode_name) in &modes_to_test {
        info!("");
        info!("─── Testing {} read mode ───", mode_name);

        let scan_start = Instant::now();
        let segment_store = Arc::new(SegmentStore::with_mode(&segment_dir, *mode));
        let segments = segment_store.scan_existing()?;
        let scan_duration = scan_start.elapsed();

        let total_bundles_in_segments: u32 = segments.iter().map(|(_, count)| *count).sum();
        info!(
            segment_count = segments.len(),
            total_bundles = total_bundles_in_segments,
            scan_ms = scan_duration.as_millis(),
            "Segments loaded ({} mode)",
            mode_name
        );
        mem_tracker.checkpoint(&format!("segments_loaded_{}", mode_name));

        // Phase 4: Multiple subscriber consumption
        info!("");
        info!(
            "═══ Phase 4: Multi-subscriber consumption ({}) ═══",
            mode_name
        );

        let export_dir = data_dir.join("exports").join(mode_name);
        std::fs::create_dir_all(&export_dir)?;

        let consume_start = Instant::now();
        let mut subscriber_results = Vec::new();

        for sub_id in 0..args.subscribers {
            let sub_name = format!("subscriber-{}", sub_id);
            let export_path = export_dir.join(format!("{}.txt", sub_name));

            let result = subscriber::consume_with_failures(
                &segment_store,
                &segments,
                &export_path,
                &sub_name,
                args.simulate_failures,
                args.failure_probability,
            )?;

            subscriber_results.push(result);
            mem_tracker.checkpoint_silent(&format!("subscriber_{}_{}_done", mode_name, sub_id));
        }

        let consume_duration = consume_start.elapsed();
        mem_tracker.checkpoint(&format!("after_consumption_{}", mode_name));

        mode_results.push(ModeResult {
            mode_name: mode_name.to_string(),
            scan_duration,
            consume_duration,
            subscriber_results,
            total_bundles: total_bundles_in_segments,
            segment_count: segments.len(),
        });
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 5: Verify and report
    // ─────────────────────────────────────────────────────────────────────────
    info!("");
    info!("═══ Phase 5: Results ═══");

    let mut all_verified = true;
    for mode_result in &mode_results {
        info!("");
        info!("─── {} mode results ───", mode_result.mode_name);

        for (sub_id, result) in mode_result.subscriber_results.iter().enumerate() {
            let verified = result.consumed == mode_result.total_bundles as usize;
            if !verified {
                warn!(
                    subscriber = sub_id,
                    expected = mode_result.total_bundles,
                    actual = result.consumed,
                    "Bundle count mismatch!"
                );
                all_verified = false;
            }
            info!(
                subscriber = sub_id,
                consumed = result.consumed,
                retries = result.retries,
                failures = result.failures,
                "Subscriber result"
            );
        }
    }

    // Memory summary
    info!("");
    info!("═══ Memory Analysis ═══");
    mem_tracker.print_summary();

    // Final summary
    info!("");
    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║                    Test Summary                            ║");
    info!("╚════════════════════════════════════════════════════════════╝");
    info!("Data directory: {}", data_dir.display());
    info!("Bundles ingested: {}", args.bundles);
    info!("Rows per bundle: {}", args.rows_per_bundle);
    info!("Total rows: {}", args.bundles * args.rows_per_bundle);
    info!("String value size: {} bytes", args.string_size);
    info!("Ingestion time: {:?}", ingest_duration);
    info!(
        "Ingestion rate: {:.0} bundles/sec | {:.0} rows/sec",
        args.bundles as f64 / ingest_duration.as_secs_f64(),
        (args.bundles * args.rows_per_bundle) as f64 / ingest_duration.as_secs_f64()
    );
    info!("Subscribers: {}", args.subscribers);

    // Per-mode summary
    for mode_result in &mode_results {
        info!("");
        info!("─── {} mode ───", mode_result.mode_name);
        info!("  Segments: {}", mode_result.segment_count);
        info!("  Scan time: {:?}", mode_result.scan_duration);
        info!("  Consumption time: {:?}", mode_result.consume_duration);
        let bundles_per_sec = mode_result.total_bundles as f64 * args.subscribers as f64
            / mode_result.consume_duration.as_secs_f64();
        let rows_per_sec = bundles_per_sec * args.rows_per_bundle as f64;
        info!(
            "  Consumption rate: {:.0} bundles/sec | {:.0} rows/sec (total across {} subscribers)",
            bundles_per_sec, rows_per_sec, args.subscribers
        );
    }

    // Compare modes if both were run
    if mode_results.len() == 2 {
        info!("");
        info!("═══ Mode Comparison ═══");
        let std_result = &mode_results[0];
        let mmap_result = &mode_results[1];

        let scan_speedup =
            std_result.scan_duration.as_secs_f64() / mmap_result.scan_duration.as_secs_f64();
        let consume_speedup =
            std_result.consume_duration.as_secs_f64() / mmap_result.consume_duration.as_secs_f64();

        info!(
            "Scan time: standard {:?} vs mmap {:?} ({:.2}x {})",
            std_result.scan_duration,
            mmap_result.scan_duration,
            if scan_speedup >= 1.0 {
                scan_speedup
            } else {
                1.0 / scan_speedup
            },
            if scan_speedup >= 1.0 {
                "faster with mmap"
            } else {
                "faster with standard"
            }
        );
        info!(
            "Consumption time: standard {:?} vs mmap {:?} ({:.2}x {})",
            std_result.consume_duration,
            mmap_result.consume_duration,
            if consume_speedup >= 1.0 {
                consume_speedup
            } else {
                1.0 / consume_speedup
            },
            if consume_speedup >= 1.0 {
                "faster with mmap"
            } else {
                "faster with standard"
            }
        );
    }

    info!("");
    info!("Peak memory (post-baseline): {} MB", mem_tracker.peak_mb());
    info!(
        "All data verified: {}",
        if all_verified { "✓ YES" } else { "✗ NO" }
    );
    info!("");

    // Handle temp directory
    if args.keep_temp {
        // Prevent cleanup by keeping the tempdir
        let path = tmp.keep();
        info!(path = ?path, "Keeping temp directory for inspection");
    }

    if all_verified {
        info!("🎉 Stress test completed successfully!");
        Ok(())
    } else {
        Err("Verification failed: bundle count mismatch".into())
    }
}

fn create_config(data_dir: &std::path::Path, segment_size_mb: u64) -> QuiverConfig {
    let mut config = QuiverConfig::default().with_data_dir(data_dir);

    config.segment.target_size_bytes =
        std::num::NonZeroU64::new(segment_size_mb * 1024 * 1024).expect("segment size is non-zero");
    config.segment.max_open_duration = Duration::from_secs(30);

    // WAL config - ensure rotation target <= max size
    config.wal.max_size_bytes =
        std::num::NonZeroU64::new(256 * 1024 * 1024).expect("256MB is non-zero");
    config.wal.rotation_target_bytes =
        std::num::NonZeroU64::new(32 * 1024 * 1024).expect("32MB is non-zero");

    config
}
