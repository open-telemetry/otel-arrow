// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unified steady-state stress test implementation.
//!
//! This module provides a single implementation that works with both TUI and text output modes.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use quiver::segment_store::SegmentReadMode;
use quiver::subscriber::{RegistryConfig, SubscriberId, SubscriberRegistry};
use quiver::{QuiverConfig, QuiverEngine, SegmentStore};
use tempfile::TempDir;
use tracing::{info, warn};

use crate::bundle;
use crate::dashboard::{Dashboard, SteadyStateConfig};
use crate::memory::MemoryTracker;
use crate::stress::{SteadyStateStats, calculate_disk_usage};
use crate::subscriber::{SharedStoreProvider, SubscriberDelay, cleanup_completed_segments};

/// Output mode for the steady-state runner.
pub enum OutputMode {
    /// TUI dashboard with sparklines.
    Tui(Option<Box<Dashboard>>),
    /// Text-based logging via tracing.
    Text,
}

impl OutputMode {
    /// Create TUI mode with a dashboard.
    pub fn tui(dashboard: Dashboard) -> Self {
        OutputMode::Tui(Some(Box::new(dashboard)))
    }

    /// Check if the user requested to quit (TUI mode only).
    pub fn check_quit(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        match self {
            OutputMode::Tui(Some(dashboard)) => Ok(dashboard.check_quit()?),
            _ => Ok(false),
        }
    }

    /// Log a message.
    pub fn log(&self, msg: &str) {
        match self {
            OutputMode::Tui(_) => eprintln!("{}", msg),
            OutputMode::Text => info!("{}", msg),
        }
    }

    /// Log a warning.
    pub fn log_warn(&self, msg: &str) {
        match self {
            OutputMode::Tui(_) => eprintln!("⚠️  {}", msg),
            OutputMode::Text => warn!("{}", msg),
        }
    }

    /// Cleanup resources (TUI mode only). Takes ownership of the dashboard.
    pub fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            OutputMode::Tui(dashboard_opt) => {
                if let Some(dashboard) = dashboard_opt.take() {
                    Ok(dashboard.cleanup()?)
                } else {
                    Ok(())
                }
            }
            OutputMode::Text => Ok(()),
        }
    }

    /// Check if this is TUI mode.
    pub fn is_tui(&self) -> bool {
        matches!(self, OutputMode::Tui(_))
    }
}

/// Configuration for steady-state test.
pub struct SteadyStateTestConfig {
    pub duration: Duration,
    pub bundles: usize,
    pub rows_per_bundle: usize,
    pub string_size: usize,
    pub subscribers: usize,
    pub subscriber_delay_ms: u64,
    pub flush_interval: usize,
    pub segment_size_mb: u64,
    pub read_mode: SegmentReadMode,
    pub leak_threshold_mb: f64,
    pub keep_temp: bool,
    pub report_interval: Duration,
    pub wal_flush_interval_ms: u64,
    pub no_wal: bool,
}

/// Run the unified steady-state stress test.
pub fn run(
    config: SteadyStateTestConfig,
    tmp: Option<TempDir>,
    data_dir: PathBuf,
    mut output: OutputMode,
) -> Result<(), Box<dyn std::error::Error>> {
    // Log header (text mode only)
    if !output.is_tui() {
        output.log("╔════════════════════════════════════════════════════════════╗");
        output.log("║         Quiver Steady-State Stress Test                    ║");
        output.log("╚════════════════════════════════════════════════════════════╝");
        output.log("");
        output.log("Mode: Single long-running engine with concurrent ingest/consume");
        output.log(&format!("Duration: {:?}", config.duration));
        output.log(&format!("Bundles per batch: {}", config.bundles));
        output.log(&format!("Subscribers: {}", config.subscribers));
        output.log(&format!(
            "Subscriber delay: {} ms",
            config.subscriber_delay_ms
        ));
        output.log("");
        output.log(&format!("Data directory: {}", data_dir.display()));
    }

    // Estimate bundle size for throughput calculations
    let bundle_size_bytes = config.rows_per_bundle * config.string_size;

    // Generate test bundles UPFRONT (before measuring memory baseline)
    output.log(&format!("Generating {} test bundles...", config.bundles));
    let test_bundles = Arc::new(bundle::generate_test_bundles(
        config.bundles,
        config.rows_per_bundle,
        config.string_size,
    ));
    output.log(&format!(
        "Test bundles ready ({} bundles)",
        test_bundles.len()
    ));

    // Create stats tracker
    let mut stats = SteadyStateStats::new(
        config.subscribers,
        config.rows_per_bundle,
        bundle_size_bytes,
    );

    // Initial metrics (measured AFTER bundle generation so they're not counted)
    let initial_mem = MemoryTracker::current_allocated_mb();
    let initial_disk = calculate_disk_usage(&data_dir).unwrap_or(0);
    stats.set_initial(initial_mem, initial_disk);

    if !output.is_tui() {
        output.log(&format!("Initial memory: {:.2} MB", initial_mem));
        output.log(&format!(
            "Initial disk: {:.2} MB",
            initial_disk as f64 / 1024.0 / 1024.0
        ));
        output.log("");
    }

    // Create dashboard config for TUI
    let dashboard_config = SteadyStateConfig {
        subscribers: config.subscribers,
        bundles_per_batch: config.bundles,
        rows_per_bundle: config.rows_per_bundle,
        subscriber_delay_ms: config.subscriber_delay_ms,
        data_dir: data_dir.display().to_string(),
    };

    // Create a single QuiverEngine that will run for the entire duration
    let engine_config = create_engine_config(
        &data_dir,
        config.segment_size_mb,
        config.wal_flush_interval_ms,
        config.no_wal,
    );
    let engine = Arc::new(QuiverEngine::new(engine_config)?);

    // Create SHARED segment store and registry for all subscribers
    let segment_dir = data_dir.join("segments");
    let segment_store = Arc::new(SegmentStore::with_mode(&segment_dir, config.read_mode));
    let store_provider = SharedStoreProvider::new(segment_store.clone());
    let registry_config = RegistryConfig::new(&data_dir);
    let registry = SubscriberRegistry::open(registry_config, store_provider.clone())?;

    // Shared state for coordination (declare segments_written early for use in callback)
    let running = Arc::new(AtomicBool::new(true));
    let ingest_running = Arc::new(AtomicBool::new(true));
    let total_ingested = Arc::new(AtomicU64::new(0));
    let total_consumed = Arc::new(AtomicU64::new(0));
    let total_cleaned = Arc::new(AtomicU64::new(0));
    let segments_written = Arc::new(AtomicU64::new(0));

    // Wire the notification chain: engine -> store -> registry
    // When engine finalizes a segment, it calls store.register_segment()
    // The store callback notifies the registry which wakes waiting subscribers
    {
        let store_provider_for_callback = store_provider.clone();
        let registry_for_callback = registry.clone();
        let segments_written_ref = segments_written.clone();

        segment_store.set_on_segment_registered(move |seq, bundle_count| {
            store_provider_for_callback.add_segment(seq, bundle_count);
            registry_for_callback.on_segment_finalized(seq, bundle_count);
            let _ = segments_written_ref.fetch_add(1, Ordering::Relaxed);
        });

        // Give engine a reference to the store for segment registration
        engine.set_segment_store(segment_store.clone());
    }

    // Register and activate all subscribers upfront
    let sub_ids = register_subscribers(&registry, config.subscribers)?;
    output.log(&format!("Registered {} subscribers", config.subscribers));

    let start = Instant::now();
    let cleanup_interval = Duration::from_secs(2);
    let mut last_cleanup = Instant::now();
    let mut last_report = Instant::now();

    // Spawn worker threads
    let ingest_handle = spawn_ingest_thread(
        engine.clone(),
        test_bundles,
        ingest_running.clone(),
        total_ingested.clone(),
    );

    // No scanner thread needed - engine notifies store directly via callback

    let subscriber_handles = spawn_subscriber_threads(
        sub_ids.clone(),
        registry.clone(),
        running.clone(),
        total_consumed.clone(),
        config.subscriber_delay_ms,
        config.flush_interval,
    );

    // Main monitoring loop
    let mut quit_requested = false;
    while start.elapsed() < config.duration && !quit_requested {
        thread::sleep(Duration::from_millis(100));

        // Check for quit key (TUI mode only)
        if output.check_quit()? {
            quit_requested = true;
            break;
        }

        // Periodic cleanup of completed segments
        if last_cleanup.elapsed() >= cleanup_interval {
            if let Ok(deleted) = cleanup_completed_segments(&*registry, &segment_store) {
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

        // Output mode-specific updates
        match &mut output {
            OutputMode::Tui(Some(dashboard)) => {
                let wal_bytes = engine.wal_bytes_written();
                dashboard.update_steady_state(&stats, &dashboard_config, wal_bytes)?;
            }
            OutputMode::Tui(None) => {}
            OutputMode::Text => {
                if last_report.elapsed() >= config.report_interval {
                    let ingested = total_ingested.load(Ordering::Relaxed);
                    let consumed = total_consumed.load(Ordering::Relaxed);
                    let cleaned = total_cleaned.load(Ordering::Relaxed);
                    let elapsed = start.elapsed().as_secs_f64();
                    let ingest_rate = ingested as f64 / elapsed;
                    let consume_rate = consumed as f64 / elapsed;

                    output.log(&format!(
                        "[{:.0}s] Ingested: {} ({:.0}/s) | Consumed: {} ({:.0}/s) | Cleaned: {} | Mem: {:.1}MB | Disk: {:.1}MB",
                        elapsed, ingested, ingest_rate, consumed, consume_rate, cleaned,
                        current_mem, current_disk as f64 / 1024.0 / 1024.0
                    ));
                    last_report = Instant::now();
                }
            }
        }
    }

    // === Graceful shutdown ===
    output.log("Shutting down...");

    // 1. Stop ingestion first
    ingest_running.store(false, Ordering::SeqCst);
    let _ = ingest_handle.join();

    let final_ingested = total_ingested.load(Ordering::Relaxed);

    // 2. Finalize any remaining open segment
    if let Err(e) = engine.shutdown() {
        output.log_warn(&format!("Engine shutdown error: {}", e));
    }

    // 3. Final segment scan
    let final_segment_count = if let Ok(final_segments) = segment_store.scan_existing() {
        for (seq, bundle_count) in &final_segments {
            store_provider.add_segment(*seq, *bundle_count);
            registry.on_segment_finalized(*seq, *bundle_count);
        }
        final_segments.len() as u64
    } else {
        segments_written.load(Ordering::Relaxed)
    };

    // 4. Drain remaining bundles
    let pre_drain_consumed = total_consumed.load(Ordering::Relaxed);
    let remaining_bundles = final_ingested.saturating_sub(pre_drain_consumed);
    output.log(&format!(
        "Draining {} remaining bundles...",
        remaining_bundles
    ));

    let drain_start = Instant::now();
    let drain_timeout = Duration::from_secs(5);
    while drain_start.elapsed() < drain_timeout {
        let consumed = total_consumed.load(Ordering::Relaxed);
        if consumed >= final_ingested {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    let post_drain_consumed = total_consumed.load(Ordering::Relaxed);
    let drained = post_drain_consumed.saturating_sub(pre_drain_consumed);
    output.log(&format!(
        "Drain complete: consumed {} bundles in {:?}",
        drained,
        drain_start.elapsed()
    ));

    // 5. Stop subscribers
    running.store(false, Ordering::SeqCst);
    for handle in subscriber_handles {
        let _ = handle.join();
    }

    // Flush final progress
    let _ = registry.flush_progress();

    // 6. Final cleanup
    let cleanup_start = Instant::now();
    let mut final_cleanup_count = 0u64;
    while let Ok(deleted) = cleanup_completed_segments(&*registry, &segment_store) {
        if deleted > 0 {
            final_cleanup_count += deleted as u64;
        } else {
            break;
        }
    }
    let cleanup_duration = cleanup_start.elapsed();
    output.log(&format!(
        "Final cleanup: {} segments deleted in {:?}",
        final_cleanup_count, cleanup_duration
    ));

    // Calculate totals
    let total_segments_cleaned = total_cleaned.load(Ordering::Relaxed) + final_cleanup_count;
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

    let final_mem = MemoryTracker::current_allocated_mb();
    let final_disk = calculate_disk_usage(&data_dir).unwrap_or(0);
    stats.update_memory(final_mem);
    stats.update_disk(final_disk);

    // Cleanup TUI and print summary
    output.cleanup()?;

    if output.is_tui() {
        stats.print_final_summary_stdout(config.leak_threshold_mb, &data_dir.display().to_string());
    } else {
        // Print text-mode summary
        let elapsed = start.elapsed();
        let bundle_rate = final_ingested as f64 / elapsed.as_secs_f64();
        output.log("");
        output.log("═══ Final Summary ═══");
        output.log(&format!("Duration: {:?}", elapsed));
        output.log(&format!("Bundles ingested: {}", final_ingested));
        output.log(&format!("Bundles consumed: {}", post_drain_consumed));
        output.log(&format!("Segments written: {}", total_segments));
        output.log(&format!("Segments cleaned: {}", total_segments_cleaned));
        output.log(&format!("Throughput: {:.0} bundles/sec", bundle_rate));
        output.log(&format!(
            "Memory: {:.2} MB initial -> {:.2} MB final (growth: {:.2} MB)",
            initial_mem,
            final_mem,
            final_mem - initial_mem
        ));
        output.log(&format!(
            "Disk: {:.2} MB initial -> {:.2} MB final",
            initial_disk as f64 / 1024.0 / 1024.0,
            final_disk as f64 / 1024.0 / 1024.0
        ));
    }

    // Handle temp directory
    if let Some(tmp) = tmp {
        if config.keep_temp {
            let kept_path = tmp.keep();
            output.log(&format!("Keeping temp directory: {}", kept_path.display()));
        }
    }

    // Check for concerning growth
    let mem_growth = stats.memory_growth_mb();
    if mem_growth > config.leak_threshold_mb {
        Err(format!(
            "Memory growth ({:.2} MB) exceeds threshold ({:.0} MB)",
            mem_growth, config.leak_threshold_mb
        )
        .into())
    } else if quit_requested {
        output.log("Steady-state test stopped by user");
        Ok(())
    } else {
        output.log("🎉 Steady-state stress test completed!");
        Ok(())
    }
}

// === Helper functions ===

fn register_subscribers(
    registry: &SubscriberRegistry<SharedStoreProvider>,
    count: usize,
) -> Result<Vec<SubscriberId>, Box<dyn std::error::Error>> {
    let mut sub_ids = Vec::with_capacity(count);
    for sub_id in 0..count {
        let sub_name = format!("subscriber-{}", sub_id);
        let id = SubscriberId::new(&sub_name)?;
        registry.register(id.clone())?;
        registry.activate(&id)?;
        sub_ids.push(id);
    }
    Ok(sub_ids)
}

fn spawn_ingest_thread(
    engine: Arc<QuiverEngine>,
    test_bundles: Arc<Vec<bundle::TestBundle>>,
    ingest_running: Arc<AtomicBool>,
    total_ingested: Arc<AtomicU64>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while ingest_running.load(Ordering::Relaxed) {
            for test_bundle in test_bundles.iter() {
                if !ingest_running.load(Ordering::Relaxed) {
                    break;
                }
                if engine.ingest(test_bundle).is_err() {
                    break;
                }
                let _ = total_ingested.fetch_add(1, Ordering::Relaxed);
            }
        }
    })
}

fn spawn_subscriber_threads(
    sub_ids: Vec<SubscriberId>,
    registry: Arc<SubscriberRegistry<SharedStoreProvider>>,
    running: Arc<AtomicBool>,
    total_consumed: Arc<AtomicU64>,
    delay_ms: u64,
    flush_interval: usize,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::with_capacity(sub_ids.len());

    for sub_id in sub_ids {
        let sub_running = running.clone();
        let sub_consumed = total_consumed.clone();
        let sub_registry = registry.clone();

        let handle = thread::spawn(move || {
            let delay = SubscriberDelay::new(delay_ms);
            let mut bundles_since_flush = 0;

            while sub_running.load(Ordering::Relaxed) {
                // Use blocking API instead of polling with sleeps
                let bundle_handle = match sub_registry.next_bundle_blocking(
                    &sub_id,
                    None, // No timeout
                    || !sub_running.load(Ordering::Relaxed), // Stop on shutdown
                ) {
                    Ok(Some(h)) => h,
                    Ok(None) => {
                        // Shutdown requested or timeout
                        continue;
                    }
                    Err(_) => {
                        // Subscriber error, exit thread
                        break;
                    }
                };

                // Bundle data is already loaded in bundle_handle.data()
                // No need to read again from sub_store
                delay.apply();
                bundle_handle.ack();
                let _ = sub_consumed.fetch_add(1, Ordering::Relaxed);
                bundles_since_flush += 1;

                if bundles_since_flush >= flush_interval {
                    let _ = sub_registry.flush_progress();
                    bundles_since_flush = 0;
                }
            }
        });

        handles.push(handle);
    }

    handles
}

fn create_engine_config(
    data_dir: &std::path::Path,
    segment_size_mb: u64,
    wal_flush_interval_ms: u64,
    no_wal: bool,
) -> QuiverConfig {
    use quiver::DurabilityMode;

    let mut config = QuiverConfig::default().with_data_dir(data_dir);

    // Set durability mode
    if no_wal {
        config.durability = DurabilityMode::SegmentOnly;
    }

    config.segment.target_size_bytes =
        std::num::NonZeroU64::new(segment_size_mb * 1024 * 1024).expect("segment size is non-zero");
    config.segment.max_open_duration = Duration::from_secs(30);

    config.wal.max_size_bytes =
        std::num::NonZeroU64::new(256 * 1024 * 1024).expect("256MB is non-zero");
    config.wal.rotation_target_bytes =
        std::num::NonZeroU64::new(32 * 1024 * 1024).expect("32MB is non-zero");
    config.wal.flush_interval = Duration::from_millis(wal_flush_interval_ms);

    config
}
