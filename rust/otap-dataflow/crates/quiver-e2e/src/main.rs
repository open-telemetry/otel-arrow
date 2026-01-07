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
mod dashboard;
mod memory;
mod steady_state;
mod stress;
mod stress_runner;
mod subscriber;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use quiver::segment_store::SegmentReadMode;
use quiver::{QuiverConfig, QuiverEngine, SegmentStore};
use tempfile::TempDir;
use tracing::{Level, info, warn};
use tracing_subscriber::FmtSubscriber;

use crate::memory::MemoryTracker;
use crate::stress::{StressStats, parse_duration};
use crate::dashboard::{Dashboard, DashboardConfig};
use crate::subscriber::SubscriberDelay;

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
#[derive(Parser, Debug, Clone)]
#[command(name = "quiver-e2e")]
#[command(about = "Stress test harness for Quiver persistence layer")]
struct Args {
    /// Number of unique test bundles to generate
    #[arg(short, long, default_value = "50")]
    bundles: usize,

    /// Number of rows per bundle (per slot)
    #[arg(short = 'r', long, default_value = "100")]
    rows_per_bundle: usize,

    /// Average size of string values in bytes
    #[arg(long, default_value = "1000")]
    string_size: usize,

    /// Number of concurrent subscribers
    #[arg(short, long, default_value = "1")]
    subscribers: usize,

    /// Target segment size in MB
    #[arg(short = 't', long, default_value = "32")]
    segment_size_mb: u64,

    /// Simulate network failures (subscriber goes offline mid-stream)
    #[arg(long, default_value = "false")]
    simulate_failures: bool,

    /// Probability of simulated failure (0.0-1.0)
    #[arg(long, default_value = "0.1")]
    failure_probability: f64,

    /// Segment read mode (standard, mmap, or compare)
    #[arg(long, default_value = "mmap")]
    read_mode: ReadModeArg,

    /// How often to flush progress files (in bundles consumed)
    #[arg(long, default_value = "100")]
    flush_interval: usize,

    /// Keep temp directory after test (for inspection)
    #[arg(long)]
    keep_temp: bool,

    /// Run stress test for a duration (e.g., "10m", "1h", "24h")
    /// When set, runs multiple iterations until duration expires.
    #[arg(long)]
    duration: Option<String>,

    /// Report interval in seconds for stress mode (memory, disk, throughput)
    #[arg(long, default_value = "10")]
    report_interval: u64,

    /// Memory growth threshold in MB to flag as potential leak
    #[arg(long, default_value = "50.0")]
    leak_threshold_mb: f64,

    /// Use a persistent data directory instead of temp (for long stress tests)
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Delay in milliseconds per bundle for subscriber consumption (simulates slow egress)
    #[arg(long, default_value = "0")]
    subscriber_delay_ms: u64,

    /// Disable TUI dashboard for stress mode (use text output instead)
    #[arg(long)]
    no_tui: bool,

    /// Steady-state mode: single long-running QuiverEngine with concurrent ingest/consume.
    /// Tests internal cleanup/retention rather than external cleanup between iterations.
    #[arg(long)]
    steady_state: bool,

    /// WAL flush interval in milliseconds (0 = flush after every write)
    #[arg(long, default_value = "25")]
    wal_flush_interval_ms: u64,

    /// Disable WAL for higher throughput (data only durable after segment finalization)
    #[arg(long)]
    no_wal: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Check if this is stress mode with TUI - don't initialize tracing in TUI mode
    // as it interferes with the terminal display
    let use_tui = args.duration.is_some() && !args.no_tui;
    
    if !use_tui {
        // Initialize tracing only for non-TUI modes
        let tracing_sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();
        let _ = tracing::subscriber::set_global_default(tracing_sub);
    }

    // Check if this is stress mode
    if let Some(ref duration_str) = args.duration {
        let duration = parse_duration(duration_str)
            .map_err(|e| format!("Invalid duration '{}': {}", duration_str, e))?;
        
        // Steady-state mode: single engine, concurrent ingest/consume, no external cleanup
        if args.steady_state {
            return run_steady_state_mode(&args, duration);
        }
        
        return run_stress_mode(&args, duration);
    }

    // Single-run mode
    run_single_iteration(&args, None)
}

/// Runs the stress test for a specified duration with periodic reporting.
fn run_stress_mode(args: &Args, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
    use stress_runner::{OutputMode, StressTestConfig, IterationResult};
    
    // Initialize tracing for text mode (before any logging)
    if args.no_tui {
        let tracing_sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();
        let _ = tracing::subscriber::set_global_default(tracing_sub);
    }

    // Set up data directory
    let (tmp, data_dir) = setup_data_dir(args)?;

    // Create dashboard config
    let dashboard_config = DashboardConfig {
        bundles_per_iteration: args.bundles,
        rows_per_bundle: args.rows_per_bundle,
        subscribers: args.subscribers,
        subscriber_delay_ms: args.subscriber_delay_ms,
        simulate_failures: args.simulate_failures,
        data_dir: data_dir.display().to_string(),
    };

    // Create output mode
    let output_mode = if !args.no_tui {
        let dashboard = Dashboard::new(duration, data_dir.clone())?;
        OutputMode::tui(dashboard)
    } else {
        OutputMode::Text
    };

    // Build config
    let config = StressTestConfig {
        duration,
        report_interval: Duration::from_secs(args.report_interval),
        leak_threshold_mb: args.leak_threshold_mb,
        keep_temp: args.keep_temp,
    };

    // Clone what we need for closures
    let args_clone = args.clone();
    let data_dir_clone = data_dir.clone();

    // Run the unified stress test
    stress_runner::run(
        output_mode,
        config,
        data_dir,
        tmp,
        dashboard_config,
        |iteration| {
            let result = run_iteration_for_stress(&args_clone, &data_dir_clone, iteration)?;
            Ok(IterationResult {
                bundles_ingested: result.bundles_ingested,
                bundles_consumed: result.bundles_consumed,
            })
        },
        || cleanup_iteration_data(&data_dir_clone),
    )
}

/// Runs steady-state stress test: single long-running QuiverEngine with concurrent ingest/consume.
/// 
/// Unlike the lifecycle stress test, this mode:
/// - Creates ONE QuiverEngine that runs for the entire duration
/// - Continuously ingests data while subscribers consume concurrently
/// - Uses a shared SubscriberRegistry for all subscribers to enable coordinated cleanup
/// - Periodically cleans up completed segments from disk
/// - Tests whether disk/memory stabilize over time
fn run_steady_state_mode(args: &Args, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
    use steady_state::{OutputMode, SteadyStateTestConfig};
    
    // Set up data directory
    let (tmp, data_dir) = setup_data_dir(args)?;
    
    // Convert read mode
    let read_mode = match args.read_mode {
        ReadModeArg::Standard => SegmentReadMode::Standard,
        ReadModeArg::Mmap | ReadModeArg::Compare => SegmentReadMode::Mmap,
    };
    
    // Build config
    let config = SteadyStateTestConfig {
        duration,
        bundles: args.bundles,
        rows_per_bundle: args.rows_per_bundle,
        string_size: args.string_size,
        subscribers: args.subscribers,
        subscriber_delay_ms: args.subscriber_delay_ms,
        flush_interval: args.flush_interval,
        segment_size_mb: args.segment_size_mb,
        read_mode,
        leak_threshold_mb: args.leak_threshold_mb,
        keep_temp: args.keep_temp,
        report_interval: Duration::from_secs(args.report_interval),
        wal_flush_interval_ms: args.wal_flush_interval_ms,
        no_wal: args.no_wal,
    };
    
    // Create output mode (TUI or Text)
    let output_mode = if !args.no_tui {
        let dashboard = Dashboard::new(duration, data_dir.clone())?;
        OutputMode::tui(dashboard)
    } else {
        // Initialize tracing for text mode
        let tracing_sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();
        let _ = tracing::subscriber::set_global_default(tracing_sub);
        OutputMode::Text
    };
    
    // Run the unified steady-state test
    steady_state::run(config, tmp, data_dir, output_mode)
}

/// Result from a single stress iteration.
struct IterationResult {
    bundles_ingested: usize,
    bundles_consumed: usize,
}

/// Runs a single iteration for stress testing (simplified, no detailed output).
fn run_iteration_for_stress(
    args: &Args,
    data_dir: &Path,
    iteration: u64,
) -> Result<IterationResult, Box<dyn std::error::Error>> {
    // Generate test data
    let bundles =
        bundle::generate_test_bundles(args.bundles, args.rows_per_bundle, args.string_size);

    // Ingest
    let config = create_config(data_dir, args.segment_size_mb, args.wal_flush_interval_ms, args.no_wal);
    let engine = Arc::new(QuiverEngine::new(config)?);

    for test_bundle in &bundles {
        engine.ingest(test_bundle)?;
    }
    engine.shutdown()?;

    // Consume
    let segment_dir = data_dir.join("segments");
    let mode = match args.read_mode {
        ReadModeArg::Standard => SegmentReadMode::Standard,
        ReadModeArg::Mmap | ReadModeArg::Compare => SegmentReadMode::Mmap,
    };
    let segment_store = Arc::new(SegmentStore::with_mode(&segment_dir, mode));
    let segments = segment_store.scan_existing()?;

    let mut total_consumed = 0usize;

    let export_dir = data_dir.join("exports").join(format!("iter_{}", iteration));
    std::fs::create_dir_all(&export_dir)?;

    for sub_id in 0..args.subscribers {
        let sub_name = format!("subscriber-{}", sub_id);
        let export_path = export_dir.join(format!("{}.txt", sub_name));

        let result = subscriber::consume_with_registry(
            segment_store.clone(),
            &segments,
            data_dir,
            &export_path,
            &sub_name,
            args.simulate_failures,
            args.failure_probability,
            args.flush_interval,
            SubscriberDelay::new(args.subscriber_delay_ms),
        )?;
        total_consumed += result.consumed;
    }

    Ok(IterationResult {
        bundles_ingested: args.bundles,
        bundles_consumed: total_consumed,
    })
}

/// Cleans up data between stress iterations.
fn cleanup_iteration_data(data_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Remove segments and exports from previous iteration
    let segments_dir = data_dir.join("segments");
    let exports_dir = data_dir.join("exports");
    let wal_dir = data_dir.join("wal");

    for dir in [&segments_dir, &exports_dir, &wal_dir] {
        if dir.exists() {
            // Make all files writable before removal (segments are read-only after finalization)
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(metadata) = path.metadata() {
                            let mut perms = metadata.permissions();
                            #[allow(clippy::permissions_set_readonly_false)]
                            perms.set_readonly(false);
                            let _ = std::fs::set_permissions(&path, perms);
                        }
                    }
                }
            }
            std::fs::remove_dir_all(dir)?;
        }
    }

    // Also clean progress files
    for entry in std::fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("quiver.sub.") {
                    std::fs::remove_file(&path)?;
                }
            }
        }
    }

    Ok(())
}

/// Sets up the data directory (temp or persistent).
/// 
/// When no data-dir is specified, creates a temp directory in ~/.quiver-e2e/
/// rather than /tmp, since /tmp may be a tmpfs (RAM-backed) filesystem
/// with limited capacity.
fn setup_data_dir(args: &Args) -> Result<(Option<TempDir>, PathBuf), Box<dyn std::error::Error>> {
    if let Some(ref dir) = args.data_dir {
        std::fs::create_dir_all(dir)?;
        info!(path = %dir.display(), "Using persistent data directory");
        Ok((None, dir.clone()))
    } else {
        // Use ~/.quiver-e2e/ instead of system temp dir (/tmp) which may be tmpfs
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| ".".into()));
        let base_dir = home.join(".quiver-e2e");
        std::fs::create_dir_all(&base_dir)?;
        let tmp = tempfile::Builder::new()
            .prefix("run-")
            .tempdir_in(&base_dir)?;
        let path = tmp.path().to_path_buf();
        info!(path = %path.display(), "Created temp directory (in ~/.quiver-e2e to avoid tmpfs)");
        Ok((Some(tmp), path))
    }
}

/// Prints common configuration.
fn print_config(args: &Args) {
    info!("Configuration:");
    info!("  Bundles per iteration: {}", args.bundles);
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
    info!("  Flush interval: {} bundles", args.flush_interval);
    info!("");
}

/// Runs a single test iteration (original behavior).
fn run_single_iteration(
    args: &Args,
    _stress_stats: Option<&mut StressStats>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║           Quiver Stress Test Harness                       ║");
    info!("╚════════════════════════════════════════════════════════════╝");
    info!("");
    print_config(args);

    // Initialize memory tracking
    let mut mem_tracker = MemoryTracker::new();
    mem_tracker.checkpoint("startup");

    // Create temp directory for test data
    let (_tmp, data_dir) = setup_data_dir(args)?;

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

    let config = create_config(&data_dir, args.segment_size_mb, args.wal_flush_interval_ms, args.no_wal);
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

            let result = subscriber::consume_with_registry(
                segment_store.clone(),
                &segments,
                &data_dir,
                &export_path,
                &sub_name,
                args.simulate_failures,
                args.failure_probability,
                args.flush_interval,
                SubscriberDelay::new(args.subscriber_delay_ms),
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
                flushes = result.flushes,
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
        if let Some(tmp) = _tmp {
            // Prevent cleanup by keeping the tempdir
            let path = tmp.keep();
            info!(path = ?path, "Keeping temp directory for inspection");
        }
    }

    if all_verified {
        info!("🎉 Stress test completed successfully!");
        Ok(())
    } else {
        Err("Verification failed: bundle count mismatch".into())
    }
}

fn create_config(data_dir: &Path, segment_size_mb: u64, wal_flush_interval_ms: u64, no_wal: bool) -> QuiverConfig {
    use quiver::DurabilityMode;
    
    let mut config = QuiverConfig::default().with_data_dir(data_dir);

    // Set durability mode
    if no_wal {
        config.durability = DurabilityMode::SegmentOnly;
    }

    config.segment.target_size_bytes =
        std::num::NonZeroU64::new(segment_size_mb * 1024 * 1024).expect("segment size is non-zero");
    config.segment.max_open_duration = Duration::from_secs(30);

    // WAL config - ensure rotation target <= max size (ignored if no_wal)
    config.wal.max_size_bytes =
        std::num::NonZeroU64::new(256 * 1024 * 1024).expect("256MB is non-zero");
    config.wal.rotation_target_bytes =
        std::num::NonZeroU64::new(32 * 1024 * 1024).expect("32MB is non-zero");
    config.wal.flush_interval = Duration::from_millis(wal_flush_interval_ms);

    config
}
