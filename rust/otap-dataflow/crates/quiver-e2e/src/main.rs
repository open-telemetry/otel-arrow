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
mod stress;
mod subscriber;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use quiver::segment_store::SegmentReadMode;
use quiver::{QuiverConfig, QuiverEngine, SegmentStore};
use tempfile::{TempDir, tempdir};
use tracing::{Level, info, warn};
use tracing_subscriber::FmtSubscriber;

use crate::memory::MemoryTracker;
use crate::stress::{SteadyStateStats, StressStats, calculate_disk_usage, parse_duration};
use crate::dashboard::{Dashboard, DashboardConfig, SteadyStateConfig};
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
#[derive(Parser, Debug)]
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
    #[arg(long, default_value = "true")]
    simulate_failures: bool,

    /// Probability of simulated failure (0.0-1.0)
    #[arg(long, default_value = "0.1")]
    failure_probability: f64,

    /// Segment read mode (standard, mmap, or compare)
    #[arg(long, default_value = "mmap")]
    read_mode: ReadModeArg,

    /// Use SubscriberRegistry with progress file persistence
    #[arg(long, default_value = "true")]
    use_registry: bool,

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

    /// Use TUI dashboard for stress mode (default: true when --duration is set)
    #[arg(long, default_value = "true")]
    tui: bool,

    /// Steady-state mode: single long-running QuiverEngine with concurrent ingest/consume.
    /// Tests internal cleanup/retention rather than external cleanup between iterations.
    #[arg(long)]
    steady_state: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Check if this is stress mode with TUI - don't initialize tracing in TUI mode
    // as it interferes with the terminal display
    let use_tui = args.duration.is_some() && args.tui;
    
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
    // Use TUI dashboard if enabled
    if args.tui {
        // Run TUI mode - return its result directly (don't fall through to text mode)
        return run_stress_mode_tui(args, duration);
    }

    // Text mode (only when --tui is not set)
    // Initialize tracing for text mode
    let tracing_sub = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(tracing_sub);
    
    run_stress_mode_text(args, duration)
}

/// Runs stress test with TUI dashboard.
fn run_stress_mode_tui(args: &Args, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
    let mut stress_stats = StressStats::new();
    let start = Instant::now();

    // Set up data directory
    let (_tmp, data_dir) = setup_data_dir(args)?;

    // Initial memory reading
    let initial_mem = MemoryTracker::current_allocated_mb();
    stress_stats.set_initial_memory(initial_mem);

    // Initial disk reading
    let disk_usage = calculate_disk_usage(&data_dir).unwrap_or(0);
    stress_stats.record_disk(disk_usage);

    // Create dashboard config
    let dashboard_config = DashboardConfig {
        bundles_per_iteration: args.bundles,
        rows_per_bundle: args.rows_per_bundle,
        subscribers: args.subscribers,
        subscriber_delay_ms: args.subscriber_delay_ms,
        use_registry: args.use_registry,
        simulate_failures: args.simulate_failures,
        data_dir: data_dir.display().to_string(),
    };

    // Initialize TUI dashboard
    let mut dashboard = Dashboard::new(duration, data_dir.clone())?;

    // Main stress loop
    let mut quit_requested = false;
    while start.elapsed() < duration && !quit_requested {
        stress_stats.iterations += 1;
        let iteration = stress_stats.iterations;

        // Check for quit key
        if dashboard.check_quit()? {
            quit_requested = true;
            break;
        }

        // Run one iteration silently
        let result = run_iteration_for_stress(args, &data_dir, iteration)?;
        stress_stats.total_bundles_ingested += result.bundles_ingested as u64;
        stress_stats.total_bundles_consumed += result.bundles_consumed as u64;

        // Record current stats
        let current_mem = MemoryTracker::current_allocated_mb();
        stress_stats.record_memory(current_mem);

        let disk_usage = calculate_disk_usage(&data_dir).unwrap_or(0);
        stress_stats.record_disk(disk_usage);

        // Update dashboard
        dashboard.update(&stress_stats, &data_dir, &dashboard_config)?;

        // Clean up data between iterations (skip if keeping temp and this might be the last iteration)
        let time_remaining = duration.saturating_sub(start.elapsed());
        if !args.keep_temp || time_remaining > Duration::from_secs(1) {
            cleanup_iteration_data(&data_dir)?;
        }
    }

    // Cleanup TUI
    dashboard.cleanup()?;

    // Print final summary to stdout (tracing not initialized in TUI mode)
    stress_stats.print_final_summary_stdout(args.leak_threshold_mb, &data_dir.display().to_string());

    // Clean up temp dir if not keeping
    if let Some(tmp) = _tmp {
        if args.keep_temp {
            // Prevent TempDir from deleting on drop
            let kept_path = tmp.keep();
            println!("Keeping temp directory: {}", kept_path.display());
        }
        // else: tmp drops here and cleans up
    }

    let leaked = stress_stats.detect_memory_leak(args.leak_threshold_mb);
    if leaked {
        Err("Stress test detected potential memory leak".into())
    } else if quit_requested {
        println!("Stress test stopped by user");
        Ok(())
    } else {
        println!("🎉 Stress test completed successfully!");
        Ok(())
    }
}

/// Runs stress test with text logging (non-TUI mode).
fn run_stress_mode_text(args: &Args, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║           Quiver Long-Running Stress Test                  ║");
    info!("╚════════════════════════════════════════════════════════════╝");
    info!("");
    info!("Stress Configuration:");
    info!("  Duration: {:?}", duration);
    info!("  Report interval: {} seconds", args.report_interval);
    info!("  Leak threshold: {:.1} MB", args.leak_threshold_mb);
    info!("  Subscriber delay: {} ms", args.subscriber_delay_ms);
    info!("");
    print_config(args);

    let mut stress_stats = StressStats::new();
    let report_interval = Duration::from_secs(args.report_interval);
    let start = Instant::now();

    // Set up data directory - for stress tests, use persistent directory if provided
    let (_tmp, data_dir) = setup_data_dir(args)?;

    // Initial memory reading
    let initial_mem = MemoryTracker::current_allocated_mb();
    stress_stats.set_initial_memory(initial_mem);
    info!("Initial memory: {:.2} MB", initial_mem);
    info!("");

    // Main stress loop
    while start.elapsed() < duration {
        stress_stats.iterations += 1;
        let iteration = stress_stats.iterations;

        // Run one iteration with cleanup (no per-iteration logging)
        let result = run_iteration_for_stress(args, &data_dir, iteration)?;
        stress_stats.total_bundles_ingested += result.bundles_ingested as u64;
        stress_stats.total_bundles_consumed += result.bundles_consumed as u64;

        // Record current stats
        let current_mem = MemoryTracker::current_allocated_mb();
        stress_stats.record_memory(current_mem);

        let disk_usage = calculate_disk_usage(&data_dir).unwrap_or(0);
        stress_stats.record_disk(disk_usage);

        // Periodic reporting
        if stress_stats.should_report(report_interval) {
            stress_stats.print_status(current_mem, disk_usage);

            // Early leak warning
            if stress_stats.detect_memory_leak(args.leak_threshold_mb) {
                warn!("⚠️  Potential memory leak detected during stress test!");
            }
        }

        // Clean up data between iterations to test lifecycle (skip if keeping temp and this might be the last)
        let time_remaining = duration.saturating_sub(start.elapsed());
        if !args.keep_temp || time_remaining > Duration::from_secs(1) {
            cleanup_iteration_data(&data_dir)?;
        }
    }

    // Final summary
    stress_stats.print_final_summary(args.leak_threshold_mb);

    // Clean up temp dir if not keeping
    if let Some(tmp) = _tmp {
        if args.keep_temp {
            // Prevent TempDir from deleting on drop
            let kept_path = tmp.keep();
            info!("Keeping temp directory: {}", kept_path.display());
        }
        // else: tmp drops here and cleans up
    }

    let leaked = stress_stats.detect_memory_leak(args.leak_threshold_mb);
    if leaked {
        Err("Stress test detected potential memory leak".into())
    } else {
        info!("🎉 Stress test completed successfully!");
        Ok(())
    }
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
    // Use TUI dashboard if enabled
    if args.tui {
        // Run TUI mode - return its result directly (don't fall through to text mode)
        return run_steady_state_mode_tui(args, duration);
    }
    
    // Text mode (only when --tui is not set)
    // Initialize tracing for text mode
    let tracing_sub = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(tracing_sub);
    
    run_steady_state_mode_text(args, duration)
}

/// Runs steady-state stress test with TUI dashboard.
fn run_steady_state_mode_tui(args: &Args, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::thread;
    use quiver::subscriber::{RegistryConfig, SegmentProvider, SubscriberId, SubscriberRegistry};
    use crate::subscriber::{SharedStoreProvider, cleanup_completed_segments};

    // Set up data directory
    let (_tmp, data_dir) = setup_data_dir(args)?;

    // Estimate bundle size for throughput calculations
    let bundle_size_bytes = args.rows_per_bundle * args.string_size;

    // Generate test bundles UPFRONT (before measuring memory baseline)
    // This avoids counting test data generation as memory overhead
    eprintln!("Generating {} test bundles...", args.bundles);
    let test_bundles = Arc::new(bundle::generate_test_bundles(
        args.bundles,
        args.rows_per_bundle,
        args.string_size,
    ));
    eprintln!("Test bundles ready ({} bundles)", test_bundles.len());

    // Create stats tracker
    let mut stats = SteadyStateStats::new(args.subscribers, args.rows_per_bundle, bundle_size_bytes);

    // Initial metrics (measured AFTER bundle generation so they're not counted)
    let initial_mem = MemoryTracker::current_allocated_mb();
    let initial_disk = calculate_disk_usage(&data_dir).unwrap_or(0);
    stats.set_initial(initial_mem, initial_disk);

    // Create dashboard config
    let config = SteadyStateConfig {
        subscribers: args.subscribers,
        bundles_per_batch: args.bundles,
        rows_per_bundle: args.rows_per_bundle,
        subscriber_delay_ms: args.subscriber_delay_ms,
        data_dir: data_dir.display().to_string(),
    };

    // Initialize TUI dashboard
    let mut dashboard = Dashboard::new(duration, data_dir.clone())?;

    // Create a single QuiverEngine that will run for the entire duration
    let engine_config = create_config(&data_dir, args.segment_size_mb);
    let engine = Arc::new(QuiverEngine::new(engine_config)?);

    // Create SHARED segment store and registry for all subscribers
    let segment_dir = data_dir.join("segments");
    let mode = match args.read_mode {
        ReadModeArg::Standard => SegmentReadMode::Standard,
        ReadModeArg::Mmap | ReadModeArg::Compare => SegmentReadMode::Mmap,
    };
    let segment_store = Arc::new(SegmentStore::with_mode(&segment_dir, mode));
    let store_provider = SharedStoreProvider::new(segment_store.clone());
    let registry_config = RegistryConfig::new(&data_dir);
    let registry = Arc::new(SubscriberRegistry::open(registry_config, store_provider.clone())?);

    // Register and activate all subscribers upfront
    let mut sub_ids = Vec::new();
    for sub_id in 0..args.subscribers {
        let sub_name = format!("subscriber-{}", sub_id);
        let id = SubscriberId::new(&sub_name)?;
        registry.register(id.clone())?;
        registry.activate(&id)?;
        sub_ids.push(id);
    }

    // Shared state for coordination
    let running = Arc::new(AtomicBool::new(true));
    let ingest_running_flag = Arc::new(AtomicBool::new(true)); // Separate flag for ingestion
    let total_ingested = Arc::new(AtomicU64::new(0));
    let total_consumed = Arc::new(AtomicU64::new(0));
    let total_cleaned = Arc::new(AtomicU64::new(0));
    let segments_written = Arc::new(AtomicU64::new(0));

    let start = Instant::now();
    let cleanup_interval = Duration::from_secs(2);
    let mut last_cleanup = Instant::now();

    // Spawn ingestion thread (uses pre-generated bundles, cycling through them)
    let ingest_engine = engine.clone();
    let ingest_running = ingest_running_flag.clone();
    let ingest_count = total_ingested.clone();
    let ingest_bundles = test_bundles.clone();

    let ingest_handle = thread::spawn(move || {
        while ingest_running.load(Ordering::Relaxed) {
            // Cycle through pre-generated bundles (no regeneration overhead)
            for test_bundle in ingest_bundles.iter() {
                if !ingest_running.load(Ordering::Relaxed) {
                    break;
                }
                if let Err(_e) = ingest_engine.ingest(test_bundle) {
                    break;
                }
                let _ = ingest_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    // Spawn segment scanner thread
    let scanner_running = running.clone();
    let scanner_store = segment_store.clone();
    let scanner_provider = store_provider.clone();
    let scanner_registry = registry.clone();
    let scanner_segments_written = segments_written.clone();
    let scanner_handle = thread::spawn(move || {
        let mut known_segments: std::collections::HashSet<u64> = std::collections::HashSet::new();
        
        while scanner_running.load(Ordering::Relaxed) {
            if let Ok(segments) = scanner_store.scan_existing() {
                for (seq, bundle_count) in segments {
                    if known_segments.insert(seq.raw()) {
                        scanner_provider.add_segment(seq, bundle_count);
                        scanner_registry.on_segment_finalized(seq, bundle_count);
                        let _ = scanner_segments_written.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Spawn subscriber threads
    let mut subscriber_handles = Vec::new();
    for sub_id in sub_ids.into_iter() {
        let sub_running = running.clone();
        let sub_consumed = total_consumed.clone();
        let sub_registry = registry.clone();
        let sub_store = segment_store.clone();
        let delay_ms = args.subscriber_delay_ms;
        let flush_interval = args.flush_interval;

        let handle = thread::spawn(move || {
            let delay = SubscriberDelay::new(delay_ms);
            let mut bundles_since_flush = 0;
            
            while sub_running.load(Ordering::Relaxed) {
                let handle = match sub_registry.next_bundle(&sub_id) {
                    Ok(Some(h)) => h,
                    Ok(None) => {
                        thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                    Err(_e) => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                };

                let bundle_ref = handle.bundle_ref();

                match sub_store.read_bundle(bundle_ref) {
                    Ok(_bundle) => {
                        delay.apply();
                        handle.ack();
                        let _ = sub_consumed.fetch_add(1, Ordering::Relaxed);
                        bundles_since_flush += 1;
                        
                        if bundles_since_flush >= flush_interval {
                            let _ = sub_registry.flush_progress();
                            bundles_since_flush = 0;
                        }
                    }
                    Err(_e) => {
                        let _ = handle.defer();
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });
        subscriber_handles.push(handle);
    }

    // Main monitoring loop with TUI updates
    let mut quit_requested = false;
    while start.elapsed() < duration && !quit_requested {
        thread::sleep(Duration::from_millis(100));

        // Check for quit key
        if dashboard.check_quit()? {
            quit_requested = true;
            break;
        }

        // Periodic cleanup of completed segments
        if last_cleanup.elapsed() >= cleanup_interval {
            if let Ok(deleted) = cleanup_completed_segments(&*registry, &*segment_store) {
                if deleted > 0 {
                    let _ = total_cleaned.fetch_add(deleted as u64, Ordering::Relaxed);
                }
            }
            last_cleanup = Instant::now();
        }

        // Update stats
        let current_mem = MemoryTracker::current_allocated_mb();
        let current_disk = calculate_disk_usage(&data_dir).unwrap_or(0);
        stats.update_memory(current_mem);
        stats.update_disk(current_disk);
        stats.update_counters(
            total_ingested.load(Ordering::Relaxed),
            total_consumed.load(Ordering::Relaxed),
            total_cleaned.load(Ordering::Relaxed),
            segments_written.load(Ordering::Relaxed),
        );

        // Update dashboard with cumulative WAL bytes from engine
        let wal_bytes = engine.wal_bytes_written();
        dashboard.update_steady_state(&stats, &config, wal_bytes)?;
    }

    // === Graceful shutdown with drain phase ===
    
    // 1. Stop ingestion first (no new data coming in)
    ingest_running_flag.store(false, Ordering::Relaxed);
    let _ = ingest_handle.join();
    
    // Get final ingested count AFTER stopping ingestion
    let final_ingested = total_ingested.load(Ordering::Relaxed);
    
    // 2. Finalize any remaining open segment
    if let Err(e) = engine.shutdown() {
        eprintln!("Engine shutdown error: {}", e);
    }
    
    // 3. Ensure all segments are discovered by doing a final scan
    // This is deterministic - we scan until we find all segments on disk
    let final_segment_count = if let Ok(final_segments) = segment_store.scan_existing() {
        for (seq, bundle_count) in &final_segments {
            // Add to provider and notify registry (idempotent if already known)
            store_provider.add_segment(*seq, *bundle_count);
            registry.on_segment_finalized(*seq, *bundle_count);
        }
        final_segments.len() as u64
    } else {
        segments_written.load(Ordering::Relaxed)
    };
    
    // 4. Let subscribers drain remaining segments (up to 5 seconds)
    let pre_drain_consumed = total_consumed.load(Ordering::Relaxed);
    let remaining_bundles = final_ingested.saturating_sub(pre_drain_consumed);
    eprintln!("Draining {} remaining bundles...", remaining_bundles);
    
    let drain_start = Instant::now();
    let drain_timeout = Duration::from_secs(5);
    while drain_start.elapsed() < drain_timeout {
        // Update stats to check consumed count
        let consumed = total_consumed.load(Ordering::Relaxed);
        if consumed >= final_ingested {
            break; // All bundles consumed
        }
        // Don't run cleanup during drain - wait until all bundles are consumed
        thread::sleep(Duration::from_millis(100));
    }
    
    let post_drain_consumed = total_consumed.load(Ordering::Relaxed);
    let drained = post_drain_consumed.saturating_sub(pre_drain_consumed);
    eprintln!("Drain complete: consumed {} bundles in {:?}", drained, drain_start.elapsed());
    
    // 5. Stop subscribers and scanner
    running.store(false, Ordering::Relaxed);
    let _ = scanner_handle.join();
    for handle in subscriber_handles {
        let _ = handle.join();
    }
    
    // Flush final progress before cleanup
    let _ = registry.flush_progress();
    
    // 6. Final cleanup of any remaining completed segments
    // Keep cleaning until no more segments can be deleted
    let cleanup_start = Instant::now();
    let mut final_cleanup_count = 0u64;
    loop {
        if let Ok(deleted) = cleanup_completed_segments(&*registry, &*segment_store) {
            if deleted > 0 {
                final_cleanup_count += deleted as u64;
            } else {
                break; // No more segments to clean
            }
        } else {
            break;
        }
    }
    let cleanup_duration = cleanup_start.elapsed();
    eprintln!("Final cleanup: {} segments deleted in {:?}", final_cleanup_count, cleanup_duration);
    
    // Calculate total segments cleaned (during test + final cleanup)
    let total_segments_cleaned = total_cleaned.load(Ordering::Relaxed) + final_cleanup_count;
    
    // Get final segment count - must be at least as many as were cleaned
    // (scanner might miss segments that are written and cleaned between scans)
    let segments_discovered = final_segment_count.max(segments_written.load(Ordering::Relaxed));
    let total_segments = segments_discovered.max(total_segments_cleaned);
    
    // Update stats with final values
    stats.update_counters(
        final_ingested,
        post_drain_consumed,
        total_segments_cleaned,
        total_segments,
    );
    stats.cleanup_duration_ms = cleanup_duration.as_millis() as u64;
    
    // Update final memory/disk readings
    let final_mem = MemoryTracker::current_allocated_mb();
    let final_disk = calculate_disk_usage(&data_dir).unwrap_or(0);
    stats.update_memory(final_mem);
    stats.update_disk(final_disk);

    // Cleanup TUI
    dashboard.cleanup()?;

    // Print final summary to stdout (tracing not initialized in TUI mode)
    stats.print_final_summary_stdout(args.leak_threshold_mb, &data_dir.display().to_string());

    // Keep temp if requested
    if let Some(tmp) = _tmp {
        if args.keep_temp {
            let kept_path = tmp.keep();
            println!("Keeping temp directory: {}", kept_path.display());
        }
    }

    // Check for concerning growth
    let mem_growth = stats.memory_growth_mb();
    if mem_growth > args.leak_threshold_mb {
        Err(format!(
            "Memory growth ({:.2} MB) exceeds threshold ({:.0} MB)",
            mem_growth, args.leak_threshold_mb
        ).into())
    } else if quit_requested {
        println!("Steady-state test stopped by user");
        Ok(())
    } else {
        println!("🎉 Steady-state stress test completed!");
        Ok(())
    }
}

/// Runs steady-state stress test with text logging (non-TUI mode).
fn run_steady_state_mode_text(args: &Args, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::thread;
    use quiver::subscriber::{RegistryConfig, SegmentProvider, SubscriberId, SubscriberRegistry};
    use crate::subscriber::{SharedStoreProvider, cleanup_completed_segments};


    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║         Quiver Steady-State Stress Test                    ║");
    info!("╚════════════════════════════════════════════════════════════╝");
    info!("");
    info!("Mode: Single long-running engine with concurrent ingest/consume");
    info!("Duration: {:?}", duration);
    info!("Bundles per batch: {}", args.bundles);
    info!("Subscribers: {}", args.subscribers);
    info!("Subscriber delay: {} ms", args.subscriber_delay_ms);
    info!("");

    // Set up data directory
    let (_tmp, data_dir) = setup_data_dir(args)?;
    info!("Data directory: {}", data_dir.display());

    // Generate test bundles UPFRONT (before measuring memory baseline)
    info!("Generating {} test bundles...", args.bundles);
    let test_bundles = Arc::new(bundle::generate_test_bundles(
        args.bundles,
        args.rows_per_bundle,
        args.string_size,
    ));
    info!("Test bundles ready ({} bundles)", test_bundles.len());

    // Create a single QuiverEngine that will run for the entire duration
    let config = create_config(&data_dir, args.segment_size_mb);
    let engine = Arc::new(QuiverEngine::new(config)?);

    // Create SHARED segment store and registry for all subscribers
    let segment_dir = data_dir.join("segments");
    let mode = match args.read_mode {
        ReadModeArg::Standard => SegmentReadMode::Standard,
        ReadModeArg::Mmap | ReadModeArg::Compare => SegmentReadMode::Mmap,
    };
    let segment_store = Arc::new(SegmentStore::with_mode(&segment_dir, mode));
    let store_provider = SharedStoreProvider::new(segment_store.clone());
    let registry_config = RegistryConfig::new(&data_dir);
    let registry = Arc::new(SubscriberRegistry::open(registry_config, store_provider.clone())?);

    // Register and activate all subscribers upfront
    let mut sub_ids = Vec::new();
    for sub_id in 0..args.subscribers {
        let sub_name = format!("subscriber-{}", sub_id);
        let id = SubscriberId::new(&sub_name)?;
        registry.register(id.clone())?;
        registry.activate(&id)?;
        sub_ids.push(id);
    }
    info!("Registered {} subscribers", args.subscribers);

    // Shared state for coordination
    let running = Arc::new(AtomicBool::new(true));
    let ingest_running_flag = Arc::new(AtomicBool::new(true)); // Separate flag for ingestion
    let total_ingested = Arc::new(AtomicU64::new(0));
    let total_consumed = Arc::new(AtomicU64::new(0));
    let total_cleaned = Arc::new(AtomicU64::new(0));

    // Initial metrics
    let initial_mem = MemoryTracker::current_allocated_mb();
    let initial_disk = calculate_disk_usage(&data_dir).unwrap_or(0);
    info!("Initial memory: {:.2} MB", initial_mem);
    info!("Initial disk: {:.2} MB", initial_disk as f64 / 1024.0 / 1024.0);
    info!("");

    let start = Instant::now();
    let report_interval = Duration::from_secs(args.report_interval);
    let mut last_report = Instant::now();
    let mut last_cleanup = Instant::now();
    let cleanup_interval = Duration::from_secs(2); // Cleanup every 2 seconds

    // Spawn ingestion thread (uses pre-generated bundles, cycling through them)
    let ingest_engine = engine.clone();
    let ingest_running = ingest_running_flag.clone();
    let ingest_count = total_ingested.clone();
    let ingest_bundles = test_bundles.clone();

    let ingest_handle = thread::spawn(move || {
        while ingest_running.load(Ordering::Relaxed) {
            // Cycle through pre-generated bundles (no regeneration overhead)
            for test_bundle in ingest_bundles.iter() {
                if !ingest_running.load(Ordering::Relaxed) {
                    break;
                }
                if let Err(e) = ingest_engine.ingest(test_bundle) {
                    warn!("Ingest error: {}", e);
                    break;
                }
                let _ = ingest_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    // Spawn segment scanner thread - discovers new segments and notifies registry
    let scanner_running = running.clone();
    let scanner_store = segment_store.clone();
    let scanner_provider = store_provider.clone();
    let scanner_registry = registry.clone();
    let segments_discovered = Arc::new(AtomicU64::new(0));
    let segments_discovered_scanner = segments_discovered.clone();
    let scanner_handle = thread::spawn(move || {
        let mut known_segments: std::collections::HashSet<u64> = std::collections::HashSet::new();
        
        while scanner_running.load(Ordering::Relaxed) {
            // Scan for new segments
            if let Ok(segments) = scanner_store.scan_existing() {
                for (seq, bundle_count) in segments {
                    if known_segments.insert(seq.raw()) {
                        // New segment discovered
                        scanner_provider.add_segment(seq, bundle_count);
                        scanner_registry.on_segment_finalized(seq, bundle_count);
                        let _ = segments_discovered_scanner.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Spawn subscriber threads
    let mut subscriber_handles = Vec::new();
    for (idx, sub_id) in sub_ids.into_iter().enumerate() {
        let sub_running = running.clone();
        let sub_consumed = total_consumed.clone();
        let sub_registry = registry.clone();
        let sub_store = segment_store.clone();
        let delay_ms = args.subscriber_delay_ms;
        let flush_interval = args.flush_interval;

        let handle = thread::spawn(move || {
            let delay = SubscriberDelay::new(delay_ms);
            let mut bundles_since_flush = 0;
            
            while sub_running.load(Ordering::Relaxed) {
                // Get next bundle from shared registry
                let handle = match sub_registry.next_bundle(&sub_id) {
                    Ok(Some(h)) => h,
                    Ok(None) => {
                        // No bundles available, wait a bit
                        thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                    Err(e) => {
                        warn!("Subscriber {} error getting bundle: {}", idx, e);
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                };

                let bundle_ref = handle.bundle_ref();

                // Read and "process" the bundle
                match sub_store.read_bundle(bundle_ref) {
                    Ok(_bundle) => {
                        // Apply delay to simulate slow processing
                        delay.apply();
                        
                        // Acknowledge
                        handle.ack();
                        let _ = sub_consumed.fetch_add(1, Ordering::Relaxed);
                        bundles_since_flush += 1;
                        
                        // Periodic flush
                        if bundles_since_flush >= flush_interval {
                            let _ = sub_registry.flush_progress();
                            bundles_since_flush = 0;
                        }
                    }
                    Err(e) => {
                        warn!("Subscriber {} read error for seg {}: {}", 
                            idx, bundle_ref.segment_seq.raw(), e);
                        // Defer to retry later (segment might not be fully written yet)
                        let _ = handle.defer();
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });
        subscriber_handles.push(handle);
    }

    // Main monitoring loop with cleanup
    let mut peak_memory = initial_mem;
    let mut peak_disk = initial_disk;

    while start.elapsed() < duration {
        thread::sleep(Duration::from_millis(500));

        // Periodic cleanup of completed segments
        if last_cleanup.elapsed() >= cleanup_interval {
            // Debug: log registry state
            let oldest = registry.oldest_incomplete_segment();
            let segments_on_disk = segment_store.segment_sequences();
            let discovered = segments_discovered.load(Ordering::Relaxed);
            let sub_stats = registry.debug_subscriber_segment_counts();
            if !segments_on_disk.is_empty() {
                info!("Cleanup check: oldest_incomplete={:?}, disk={}, discovered={}, subs={:?}", 
                    oldest.map(|s| s.raw()), 
                    segments_on_disk.len(),
                    discovered,
                    sub_stats);
            }
            
            match cleanup_completed_segments(&*registry, &*segment_store) {
                Ok(deleted) => {
                    if deleted > 0 {
                        let _ = total_cleaned.fetch_add(deleted as u64, Ordering::Relaxed);
                        info!("Cleaned up {} completed segments", deleted);
                    }
                }
                Err(e) => {
                    warn!("Cleanup error: {}", e);
                }
            }
            last_cleanup = Instant::now();
        }

        // Periodic reporting
        if last_report.elapsed() >= report_interval {
            let current_mem = MemoryTracker::current_allocated_mb();
            let current_disk = calculate_disk_usage(&data_dir).unwrap_or(0);
            let ingested = total_ingested.load(Ordering::Relaxed);
            let consumed = total_consumed.load(Ordering::Relaxed);
            let cleaned = total_cleaned.load(Ordering::Relaxed);
            let elapsed = start.elapsed();

            peak_memory = peak_memory.max(current_mem);
            peak_disk = peak_disk.max(current_disk);

            let ingest_rate = ingested as f64 / elapsed.as_secs_f64();
            let consume_rate = consumed as f64 / elapsed.as_secs_f64();
            let rows_per_bundle = args.rows_per_bundle as f64;
            let bundle_size_bytes = args.rows_per_bundle * args.string_size; // approximate
            let consume_rows_rate = consume_rate * rows_per_bundle;
            let consume_mb_rate = consume_rate * bundle_size_bytes as f64 / 1024.0 / 1024.0;

            info!("╔════════════════════════════════════════════════════════════╗");
            info!("║             Steady-State Status Report                     ║");
            info!("╚════════════════════════════════════════════════════════════╝");
            info!("Elapsed: {:02}:{:02}:{:02}",
                elapsed.as_secs() / 3600,
                (elapsed.as_secs() % 3600) / 60,
                elapsed.as_secs() % 60
            );
            info!("Bundles ingested: {} ({:.0}/sec)", ingested, ingest_rate);
            info!("Bundles consumed: {} ({:.0}/sec)", consumed, consume_rate);
            info!("Throughput: {:.0} rows/sec, {:.1} MB/sec", consume_rows_rate, consume_mb_rate);
            info!("Segments cleaned: {}", cleaned);
            info!("Backlog: {} bundles", ingested.saturating_sub(consumed / args.subscribers as u64));
            info!("");
            info!("Memory: {:.2} MB (initial: {:.2}, peak: {:.2}, growth: {:.2})",
                current_mem, initial_mem, peak_memory, current_mem - initial_mem);
            info!("Disk: {:.2} MB (initial: {:.2}, peak: {:.2})",
                current_disk as f64 / 1024.0 / 1024.0,
                initial_disk as f64 / 1024.0 / 1024.0,
                peak_disk as f64 / 1024.0 / 1024.0
            );
            info!("");

            last_report = Instant::now();
        }
    }

    // === Graceful shutdown with drain phase ===
    info!("Shutting down...");
    
    // 1. Stop ingestion first (no new data coming in)
    // The ingest thread has its own flag so subscribers keep running
    ingest_running_flag.store(false, Ordering::Relaxed);
    let _ = ingest_handle.join();
    info!("Ingestion stopped");
    
    // 2. Finalize any remaining open segment
    engine.shutdown()?;
    info!("Engine finalized");
    
    // 3. Let subscribers drain remaining segments (up to 5 seconds)
    // Subscribers are still running because we only stopped ingest_running_flag
    let pre_drain_consumed = total_consumed.load(Ordering::Relaxed);
    let final_ingested = total_ingested.load(Ordering::Relaxed);
    let remaining_bundles = final_ingested.saturating_sub(pre_drain_consumed);
    info!("Draining {} remaining bundles...", remaining_bundles);
    
    let drain_start = Instant::now();
    let drain_timeout = Duration::from_secs(5);
    while drain_start.elapsed() < drain_timeout {
        let consumed = total_consumed.load(Ordering::Relaxed);
        if consumed >= final_ingested {
            break; // All bundles consumed
        }
        // Don't run cleanup during drain - wait until all bundles are consumed
        thread::sleep(Duration::from_millis(100));
    }
    
    let post_drain_consumed = total_consumed.load(Ordering::Relaxed);
    let drained = post_drain_consumed.saturating_sub(pre_drain_consumed);
    info!("Drain complete: consumed {} bundles in {:?}", drained, drain_start.elapsed());
    
    
    // 4. Stop subscribers and scanner
    running.store(false, Ordering::Relaxed);
    let _ = scanner_handle.join();
    for handle in subscriber_handles {
        let _ = handle.join();
    }
    
    // Flush final progress before cleanup
    let _ = registry.flush_progress();
    
    // 5. Final cleanup of any remaining completed segments
    // Keep cleaning until no more segments can be deleted
    let mut total_final_cleaned = 0u64;
    loop {
        match cleanup_completed_segments(&*registry, &*segment_store) {
            Ok(deleted) => {
                if deleted > 0 {
                    total_final_cleaned += deleted as u64;
                    let _ = total_cleaned.fetch_add(deleted as u64, Ordering::Relaxed);
                } else {
                    break; // No more segments to clean
                }
            }
            Err(e) => {
                warn!("Final cleanup error: {}", e);
                break;
            }
        }
    }
    if total_final_cleaned > 0 {
        info!("Final cleanup: {} segments deleted", total_final_cleaned);
    }

    // Final report
    let final_mem = MemoryTracker::current_allocated_mb();
    let final_disk = calculate_disk_usage(&data_dir).unwrap_or(0);
    let total_time = start.elapsed();

    info!("");
    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║           Steady-State Final Summary                       ║");
    info!("╚════════════════════════════════════════════════════════════╝");
    info!("Data directory: {}", data_dir.display());
    info!("Total runtime: {:02}:{:02}:{:02}",
        total_time.as_secs() / 3600,
        (total_time.as_secs() % 3600) / 60,
        total_time.as_secs() % 60
    );
    info!("Total bundles ingested: {}", total_ingested.load(Ordering::Relaxed));
    info!("Total bundles consumed: {}", total_consumed.load(Ordering::Relaxed));
    info!("Total segments cleaned: {}", total_cleaned.load(Ordering::Relaxed));
    info!("");
    
    // Calculate throughput
    let elapsed_secs = total_time.as_secs_f64();
    let final_consumed = total_consumed.load(Ordering::Relaxed) as f64;
    let bundle_rate = final_consumed / elapsed_secs;
    let rows_rate = bundle_rate * args.rows_per_bundle as f64;
    let bundle_size_bytes = args.rows_per_bundle * args.string_size;
    let mb_rate = bundle_rate * bundle_size_bytes as f64 / 1024.0 / 1024.0;
    info!("Throughput:");
    info!("  Bundles: {:.0}/sec", bundle_rate);
    info!("  Rows:    {:.0}/sec", rows_rate);
    info!("  Data:    {:.1} MB/sec", mb_rate);
    info!("");
    info!("Memory Analysis:");
    info!("  Initial: {:.2} MB", initial_mem);
    info!("  Final:   {:.2} MB", final_mem);
    info!("  Peak:    {:.2} MB", peak_memory);
    info!("  Growth:  {:.2} MB", final_mem - initial_mem);
    info!("");
    info!("Disk Analysis:");
    info!("  Initial: {:.2} MB", initial_disk as f64 / 1024.0 / 1024.0);
    info!("  Final:   {:.2} MB", final_disk as f64 / 1024.0 / 1024.0);
    info!("  Peak:    {:.2} MB", peak_disk as f64 / 1024.0 / 1024.0);

    // Check for concerning growth
    let mem_growth = final_mem - initial_mem;
    if mem_growth > args.leak_threshold_mb {
        warn!("⚠️  Memory growth ({:.2} MB) exceeds threshold ({:.0} MB)", mem_growth, args.leak_threshold_mb);
    } else {
        info!("✓ Memory growth within threshold");
    }

    // Keep temp if requested
    if let Some(tmp) = _tmp {
        if args.keep_temp {
            let kept_path = tmp.keep();
            info!("Keeping temp directory: {}", kept_path.display());
        }
    }

    info!("🎉 Steady-state stress test completed!");
    Ok(())
}

/// Result from a single stress iteration.
struct IterationResult {
    bundles_ingested: usize,
    bundles_consumed: usize,
}

/// Runs a single iteration for stress testing (simplified, no detailed output).
fn run_iteration_for_stress(
    args: &Args,
    data_dir: &PathBuf,
    iteration: u64,
) -> Result<IterationResult, Box<dyn std::error::Error>> {
    // Generate test data
    let bundles =
        bundle::generate_test_bundles(args.bundles, args.rows_per_bundle, args.string_size);

    // Ingest
    let config = create_config(data_dir, args.segment_size_mb);
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

        let result = if args.use_registry {
            subscriber::consume_with_registry(
                segment_store.clone(),
                &segments,
                data_dir,
                &export_path,
                &sub_name,
                args.simulate_failures,
                args.failure_probability,
                args.flush_interval,
                SubscriberDelay::new(args.subscriber_delay_ms),
            )?
        } else {
            subscriber::consume_with_failures(
                &segment_store,
                &segments,
                &export_path,
                &sub_name,
                args.simulate_failures,
                args.failure_probability,
            )?
        };
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
fn setup_data_dir(args: &Args) -> Result<(Option<TempDir>, PathBuf), Box<dyn std::error::Error>> {
    if let Some(ref dir) = args.data_dir {
        std::fs::create_dir_all(dir)?;
        info!(path = %dir.display(), "Using persistent data directory");
        Ok((None, dir.clone()))
    } else {
        let tmp = tempdir()?;
        let path = tmp.path().to_path_buf();
        info!(path = %path.display(), "Created temp directory");
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
    info!("  Use registry: {}", args.use_registry);
    if args.use_registry {
        info!("  Flush interval: {} bundles", args.flush_interval);
    }
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

            let result = if args.use_registry {
                subscriber::consume_with_registry(
                    segment_store.clone(),
                    &segments,
                    &data_dir,
                    &export_path,
                    &sub_name,
                    args.simulate_failures,
                    args.failure_probability,
                    args.flush_interval,
                    SubscriberDelay::new(args.subscriber_delay_ms),
                )?
            } else {
                subscriber::consume_with_failures(
                    &segment_store,
                    &segments,
                    &export_path,
                    &sub_name,
                    args.simulate_failures,
                    args.failure_probability,
                )?
            };

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
