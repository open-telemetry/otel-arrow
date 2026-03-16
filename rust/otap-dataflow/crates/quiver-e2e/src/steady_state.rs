// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unified steady-state stress test implementation.
//!
//! This module provides a single implementation that works with both TUI and text output modes.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use quiver::SegmentReadMode;
use quiver::budget::DiskBudget;
use quiver::config::RetentionPolicy;
use quiver::subscriber::SubscriberId;
use quiver::{CancellationToken, QuiverConfig, QuiverEngine};
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::bundle;
use crate::dashboard::{Dashboard, SteadyStateConfig};
use crate::memory::MemoryTracker;
use crate::stats::{SteadyStateStats, calculate_disk_usage};
use crate::subscriber::SubscriberDelay;

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
    pub const fn is_tui(&self) -> bool {
        matches!(self, OutputMode::Tui(_))
    }
}

/// Creates a per-engine disk budget using Phase 1 static quota approach.
///
/// The global disk budget is divided evenly among engines. Each engine gets
/// `global_cap / num_engines` as its quota, which it enforces independently
/// without any cross-engine coordination.
///
/// Takes the engine's [`QuiverConfig`] to read actual segment/WAL sizes
/// (which may be staggered per engine), ensuring the budget always matches
/// the engine that will use it.
///
/// Returns a 100 GB budget per engine if disk_budget_mb is 0 (effectively unlimited).
fn create_per_engine_budget(
    global_disk_budget_mb: u64,
    num_engines: usize,
    policy: RetentionPolicy,
    config: &QuiverConfig,
) -> Result<Arc<DiskBudget>, String> {
    // Phase 1: Static quota per engine = global_cap / num_engines
    let global_cap_bytes = if global_disk_budget_mb == 0 {
        100 * 1024 * 1024 * 1024 // 100 GB "unlimited"
    } else {
        global_disk_budget_mb * 1024 * 1024
    };
    let per_engine_cap = global_cap_bytes / num_engines as u64;

    DiskBudget::for_config(per_engine_cap, config, policy)
        .map(Arc::new)
        .map_err(|e| {
            let segment_bytes = config.segment.target_size_bytes.get();
            let wal_bytes = DiskBudget::effective_wal_size(config);
            let min_global_mb =
                DiskBudget::minimum_hard_cap(segment_bytes, wal_bytes) * num_engines as u64
                    / (1024 * 1024);
            format!(
                "{e} (global {global_disk_budget_mb} MB / {num_engines} engines = {} MB per engine). \
                 Minimum global for {num_engines} engines: {min_global_mb} MB",
                per_engine_cap / (1024 * 1024),
            )
        })
}

/// Configuration for steady-state test.
pub struct SteadyStateTestConfig {
    pub duration: Duration,
    pub bundles: usize,
    pub rows_per_bundle: usize,
    pub string_size: usize,
    pub subscribers: usize,
    pub subscriber_delay_ms: u64,
    /// How often to call maintain() (in milliseconds, 0 = never)
    pub maintain_interval_ms: u64,
    pub segment_size_mb: u64,
    /// Read mode for segment files (mmap vs standard I/O).
    pub read_mode: SegmentReadMode,
    pub leak_threshold_mb: f64,
    pub keep_temp: bool,
    pub report_interval: Duration,
    pub wal_flush_interval_ms: u64,
    pub no_wal: bool,
    pub engines: usize,
    /// Disk budget cap in MB (0 = effectively unlimited).
    pub disk_budget_mb: u64,
    /// Retention policy when disk budget is exceeded.
    pub retention_policy: RetentionPolicy,
}

/// Run the unified steady-state stress test.
pub async fn run(
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
        if config.engines > 1 {
            output.log(&format!(
                "Mode: {} parallel engines with concurrent ingest/consume",
                config.engines
            ));
        } else {
            output.log("Mode: Single long-running engine with concurrent ingest/consume");
        }
        output.log(&format!("Duration: {:?}", config.duration));
        output.log(&format!("Bundles per batch: {}", config.bundles));
        output.log(&format!("Subscribers per engine: {}", config.subscribers));
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
    // All engines share the same test bundles
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
    // Note: We pass subscribers per engine (not total) because each subscriber only
    // consumes bundles from its own engine, not from a shared pool.
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
        subscribers: config.subscribers * config.engines,
        bundles_per_batch: config.bundles,
        rows_per_bundle: config.rows_per_bundle,
        subscriber_delay_ms: config.subscriber_delay_ms,
        data_dir: data_dir.display().to_string(),
    };

    // Global coordination state shared across all engines
    let ingest_running = Arc::new(AtomicBool::new(true));
    let total_ingested = Arc::new(AtomicU64::new(0));
    let total_consumed = Arc::new(AtomicU64::new(0));
    let total_cleaned = Arc::new(AtomicU64::new(0));
    let backpressure_count = Arc::new(AtomicU64::new(0));

    // Log Phase 1 budget allocation (static quotas per engine)
    if !output.is_tui() && config.disk_budget_mb > 0 {
        let per_engine_mb = config.disk_budget_mb / config.engines as u64;
        output.log(&format!(
            "Disk budget: {} MB global / {} engines = {} MB per engine (Phase 1 static quotas)",
            config.disk_budget_mb, config.engines, per_engine_mb
        ));
    }

    // Create per-engine resources (engine owns segment store and registry)
    // Using async QuiverEngine::open which handles WAL initialization asynchronously
    let num_engines = config.engines;
    let subscribers_per_engine = config.subscribers;
    let segment_size_mb = config.segment_size_mb;
    let wal_flush_interval_ms = config.wal_flush_interval_ms;
    let no_wal = config.no_wal;
    let read_mode = config.read_mode;
    let disk_budget_mb = config.disk_budget_mb;
    let retention_policy = config.retention_policy;
    let data_dir_clone = data_dir.clone();
    let is_tui = output.is_tui();

    let (engines, all_sub_ids): (Vec<Arc<QuiverEngine>>, Vec<(usize, SubscriberId)>) =
        create_engines(
            num_engines,
            subscribers_per_engine,
            segment_size_mb,
            wal_flush_interval_ms,
            no_wal,
            read_mode,
            disk_budget_mb,
            retention_policy,
            &data_dir_clone,
            is_tui,
        )
        .await
        .map_err(|e: String| -> Box<dyn std::error::Error> { e.into() })?;

    output.log(&format!(
        "Started {} engine(s) with {} total subscribers",
        config.engines,
        all_sub_ids.len()
    ));

    let start = Instant::now();
    let cleanup_interval = Duration::from_secs(2);
    let mut last_cleanup = Instant::now();
    let mut last_report = Instant::now();

    // Spawn ingest tasks (one per engine) with staggered startup delays
    // to desynchronize segment writes across engines
    let mut ingest_handles = Vec::with_capacity(config.engines);
    for (engine_idx, engine) in engines.iter().enumerate() {
        // Stagger startup: spread engines evenly across one segment write cycle
        // Approximate time to fill a segment = segment_size_mb / ingest_rate_mb_per_sec
        // Rough estimate: ~100ms per engine for a 4-engine setup
        let startup_delay_ms = if config.engines > 1 {
            (100 * engine_idx) as u64
        } else {
            0
        };
        let handle = spawn_ingest_task(
            engine.clone(),
            test_bundles.clone(),
            ingest_running.clone(),
            total_ingested.clone(),
            backpressure_count.clone(),
            startup_delay_ms,
        );
        ingest_handles.push(handle);
    }

    // Create a cancellation token for graceful shutdown
    let shutdown_token = CancellationToken::new();

    // Spawn subscriber tasks (distributed across engines)
    let mut subscriber_handles = Vec::new();
    for (engine_idx, sub_id) in &all_sub_ids {
        let engine = engines[*engine_idx].clone();
        let sub_consumed = total_consumed.clone();
        let delay_ms = config.subscriber_delay_ms;
        let maintain_interval_ms = config.maintain_interval_ms;
        let sub_id_clone = sub_id.clone();
        let cancel = shutdown_token.clone();

        let handle = tokio::spawn(async move {
            let delay = SubscriberDelay::new(delay_ms);
            let maintain_interval = Duration::from_millis(maintain_interval_ms);
            let mut last_maintain = Instant::now();

            loop {
                // Use engine's async subscriber API with cancellation
                let bundle_handle = match engine
                    .next_bundle(
                        &sub_id_clone,
                        Some(Duration::from_millis(100)),
                        Some(&cancel),
                    )
                    .await
                {
                    Ok(Some(h)) => h,
                    Ok(None) => continue, // Timeout, loop back to check cancellation
                    Err(e) if e.is_cancelled() => break, // Graceful shutdown
                    Err(_) => break,      // Other error
                };

                delay.apply().await;
                bundle_handle.ack();
                let _ = sub_consumed.fetch_add(1, Ordering::Relaxed);

                // Time-based maintenance: flush progress + cleanup (0 = disabled)
                if maintain_interval_ms > 0 && last_maintain.elapsed() >= maintain_interval {
                    let _ = engine.maintain().await;
                    last_maintain = Instant::now();
                }
            }
        });

        subscriber_handles.push(handle);
    }

    // Main monitoring loop
    let mut quit_requested = false;
    while start.elapsed() < config.duration && !quit_requested {
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check for quit key (TUI mode only)
        if output.check_quit()? {
            quit_requested = true;
            break;
        }

        // Periodic cleanup of completed segments (all engines use their own cleanup)
        if last_cleanup.elapsed() >= cleanup_interval {
            for engine in &engines {
                if let Ok(deleted) = engine.cleanup_completed_segments() {
                    if deleted > 0 {
                        let _ = total_cleaned.fetch_add(deleted as u64, Ordering::Relaxed);
                    }
                }
            }
            last_cleanup = Instant::now();
        }

        // Update stats
        let current_mem = MemoryTracker::current_allocated_mb();
        let current_disk = calculate_disk_usage(&data_dir).unwrap_or(0);
        // Get total segments written across all engines (monotonically increasing)
        let total_segments_written: u64 = engines.iter().map(|e| e.total_segments_written()).sum();
        // Get actual active segment count from segment stores (accurate regardless of cleanup source)
        let active_segments: u64 = engines
            .iter()
            .map(|e| e.segment_store().segment_count() as u64)
            .sum();
        // Get total force-dropped segments and bundles (DropOldest policy)
        let force_dropped_segments: u64 = engines.iter().map(|e| e.force_dropped_segments()).sum();
        let force_dropped_bundles: u64 = engines.iter().map(|e| e.force_dropped_bundles()).sum();
        stats.update_memory(current_mem);
        stats.update_disk(current_disk);
        stats.update_counters(
            total_ingested.load(Ordering::Relaxed),
            total_consumed.load(Ordering::Relaxed),
            active_segments,
            total_segments_written,
            backpressure_count.load(Ordering::Relaxed),
            force_dropped_segments,
            force_dropped_bundles,
        );

        // Output mode-specific updates
        match &mut output {
            OutputMode::Tui(Some(dashboard)) => {
                // Sum WAL and segment bytes across all engines
                let wal_bytes: u64 = engines.iter().map(|e| e.wal_bytes_written()).sum();
                let segment_bytes: u64 = engines.iter().map(|e| e.segment_bytes_written()).sum();
                dashboard.update_steady_state(
                    &stats,
                    &dashboard_config,
                    wal_bytes,
                    segment_bytes,
                )?;
            }
            OutputMode::Tui(None) => {}
            OutputMode::Text => {
                if last_report.elapsed() >= config.report_interval {
                    let ingested = total_ingested.load(Ordering::Relaxed);
                    let consumed = total_consumed.load(Ordering::Relaxed);
                    let backpressure = backpressure_count.load(Ordering::Relaxed);
                    let elapsed = start.elapsed().as_secs_f64();
                    let ingest_rate = ingested as f64 / elapsed;
                    let consume_rate = consumed as f64 / elapsed;

                    output.log(&format!(
                        "[{:.0}s] Ingested: {} ({:.0}/s) | Consumed: {} ({:.0}/s) | Active: {} | Cleaned: {} | BP: {} | Dropped: {} segs/{} bundles | Mem: {:.1}MB | Disk: {:.1}MB",
                        elapsed, ingested, ingest_rate, consumed, consume_rate,
                        active_segments, stats.total_cleaned, backpressure,
                        force_dropped_segments, force_dropped_bundles,
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
    for handle in ingest_handles {
        let _ = handle.await;
    }

    let final_ingested = total_ingested.load(Ordering::Relaxed);

    // 2. Finalize any remaining open segments (all engines) using async shutdown
    let mut shutdown_errors = Vec::new();
    for engine in &engines {
        if let Err(e) = engine.shutdown().await {
            shutdown_errors.push(format!("Engine shutdown error: {}", e));
        }
    }
    for err in shutdown_errors {
        output.log_warn(&err);
    }

    // 3. Final segment count (total written, monotonically increasing)
    let final_total_segments_written: u64 =
        engines.iter().map(|e| e.total_segments_written()).sum();

    // Get final force-dropped count (DropOldest policy)
    let final_force_dropped_segments: u64 =
        engines.iter().map(|e| e.force_dropped_segments()).sum();
    let final_force_dropped_bundles: u64 = engines.iter().map(|e| e.force_dropped_bundles()).sum();

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
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let post_drain_consumed = total_consumed.load(Ordering::Relaxed);
    let drained = post_drain_consumed.saturating_sub(pre_drain_consumed);
    output.log(&format!(
        "Drain complete: consumed {} bundles in {:?}",
        drained,
        drain_start.elapsed()
    ));

    // 5. Stop subscribers using cancellation token
    shutdown_token.cancel();
    for handle in subscriber_handles {
        let _ = handle.await;
    }

    // Flush final progress (all engines)
    for engine in &engines {
        let _ = engine.flush_progress().await;
    }

    // 6. Final cleanup (all engines)
    let cleanup_start = Instant::now();
    let mut final_cleanup_count = 0u64;
    for engine in &engines {
        loop {
            match engine.cleanup_completed_segments() {
                Ok(deleted) if deleted > 0 => {
                    final_cleanup_count += deleted as u64;
                }
                _ => break,
            }
        }
    }
    let cleanup_duration = cleanup_start.elapsed();
    output.log(&format!(
        "Final cleanup: {} segments deleted in {:?}",
        final_cleanup_count, cleanup_duration
    ));

    // Get final active segment count (should be near 0 after cleanup)
    let final_active_segments: u64 = engines
        .iter()
        .map(|e| e.segment_store().segment_count() as u64)
        .sum();

    // Update stats with final values
    stats.update_counters(
        final_ingested,
        post_drain_consumed,
        final_active_segments,
        final_total_segments_written,
        backpressure_count.load(Ordering::Relaxed),
        final_force_dropped_segments,
        final_force_dropped_bundles,
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
        output.log(&format!("Engines: {}", config.engines));
        output.log(&format!("Bundles ingested: {}", final_ingested));
        output.log(&format!("Bundles consumed: {}", post_drain_consumed));
        output.log(&format!(
            "Segments written: {}",
            final_total_segments_written
        ));
        output.log(&format!(
            "Segments cleaned: {} (active: {})",
            stats.total_cleaned, final_active_segments
        ));
        output.log(&format!(
            "Dropped: {} segments / {} bundles (DropOldest policy)",
            final_force_dropped_segments, final_force_dropped_bundles
        ));
        output.log(&format!(
            "Consumed throughput: {:.0} bundles/sec",
            bundle_rate
        ));
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

/// Register subscribers on an engine.
fn register_subscribers(
    engine: &Arc<QuiverEngine>,
    count: usize,
) -> Result<Vec<SubscriberId>, String> {
    let mut sub_ids = Vec::with_capacity(count);
    for sub_id in 0..count {
        let sub_name = format!("subscriber-{}", sub_id);
        let id =
            SubscriberId::new(&sub_name).map_err(|e| format!("Invalid subscriber id: {}", e))?;
        engine
            .register_subscriber(id.clone())
            .map_err(|e| format!("Failed to register subscriber: {}", e))?;
        engine
            .activate_subscriber(&id)
            .map_err(|e| format!("Failed to activate subscriber: {}", e))?;
        sub_ids.push(id);
    }
    Ok(sub_ids)
}

fn spawn_ingest_task(
    engine: Arc<QuiverEngine>,
    test_bundles: Arc<Vec<bundle::TestBundle>>,
    ingest_running: Arc<AtomicBool>,
    total_ingested: Arc<AtomicU64>,
    backpressure_count: Arc<AtomicU64>,
    startup_delay_ms: u64,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        // Staggered startup to desynchronize segment writes
        if startup_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(startup_delay_ms)).await;
        }
        while ingest_running.load(Ordering::Relaxed) {
            for test_bundle in test_bundles.iter() {
                if !ingest_running.load(Ordering::Relaxed) {
                    break;
                }
                // Retry loop for backpressure handling
                loop {
                    match engine.ingest(test_bundle).await {
                        Ok(()) => {
                            let _ = total_ingested.fetch_add(1, Ordering::Relaxed);
                            break; // Success, move to next bundle
                        }
                        Err(e) if e.is_at_capacity() => {
                            // Backpressure: wait for consumers to catch up
                            let _ = backpressure_count.fetch_add(1, Ordering::Relaxed);
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            // Check if we should stop
                            if !ingest_running.load(Ordering::Relaxed) {
                                return;
                            }
                            // Retry this bundle
                            continue;
                        }
                        Err(_) => {
                            // Other error: stop ingestion
                            return;
                        }
                    }
                }
            }
        }
    })
}

fn create_engine_config(
    data_dir: &std::path::Path,
    segment_size_mb: u64,
    wal_flush_interval_ms: u64,
    no_wal: bool,
    read_mode: SegmentReadMode,
    engine_idx: usize,
    engine_count: usize,
) -> QuiverConfig {
    use quiver::DurabilityMode;

    let mut config = QuiverConfig::default().with_data_dir(data_dir);

    // Set durability mode
    if no_wal {
        config.durability = DurabilityMode::SegmentOnly;
    }

    // Set read mode
    config.read_mode = read_mode;

    // Stagger segment sizes slightly to avoid synchronized finalization across engines.
    // Each engine gets a +/- 10% offset from the base size, distributed evenly.
    let stagger_factor = if engine_count > 1 {
        // Range from -0.1 to +0.1 across engines
        let position = engine_idx as f64 / (engine_count - 1) as f64; // 0.0 to 1.0
        0.9 + (position * 0.2) // 0.9 to 1.1
    } else {
        1.0
    };
    let staggered_segment_size = ((segment_size_mb as f64 * stagger_factor) as u64).max(1);

    config.segment.target_size_bytes =
        std::num::NonZeroU64::new(staggered_segment_size * 1024 * 1024)
            .expect("segment size is non-zero");
    config.segment.max_open_duration = Duration::from_secs(30);

    config.wal.max_size_bytes =
        std::num::NonZeroU64::new(128 * 1024 * 1024).expect("128MB is non-zero");
    config.wal.rotation_target_bytes =
        std::num::NonZeroU64::new(32 * 1024 * 1024).expect("32MB is non-zero");

    // Stagger WAL flush intervals similarly to avoid synchronized flushes.
    // Vary the interval slightly (+/- 10%) to desynchronize over time.
    let interval_stagger = if engine_count > 1 {
        let position = engine_idx as f64 / (engine_count - 1) as f64;
        0.9 + (position * 0.2)
    } else {
        1.0
    };
    let staggered_flush_ms = ((wal_flush_interval_ms as f64 * interval_stagger) as u64).max(1);

    config.wal.flush_interval = Duration::from_millis(staggered_flush_ms);

    config
}

/// Creates engines asynchronously using the async `QuiverEngine::open` API.
#[allow(clippy::too_many_arguments)]
async fn create_engines(
    num_engines: usize,
    subscribers_per_engine: usize,
    segment_size_mb: u64,
    wal_flush_interval_ms: u64,
    no_wal: bool,
    read_mode: SegmentReadMode,
    disk_budget_mb: u64,
    retention_policy: RetentionPolicy,
    data_dir: &std::path::Path,
    _is_tui: bool,
) -> Result<(Vec<Arc<QuiverEngine>>, Vec<(usize, SubscriberId)>), String> {
    let mut engines: Vec<Arc<QuiverEngine>> = Vec::with_capacity(num_engines);
    let mut all_sub_ids: Vec<(usize, SubscriberId)> = Vec::new();

    for engine_idx in 0..num_engines {
        // Each engine gets its own subdirectory
        let engine_dir = if num_engines > 1 {
            data_dir.join(format!("engine_{}", engine_idx))
        } else {
            data_dir.to_path_buf()
        };
        tokio::fs::create_dir_all(&engine_dir)
            .await
            .map_err(|e| format!("Failed to create engine dir: {}", e))?;

        // Create engine with staggered timing to avoid synchronized I/O
        let engine_config = create_engine_config(
            &engine_dir,
            segment_size_mb,
            wal_flush_interval_ms,
            no_wal,
            read_mode,
            engine_idx,
            num_engines,
        );

        // Create per-engine disk budget (Phase 1: static quota per engine).
        // Budget reads segment/WAL sizes directly from the engine config,
        // which includes any per-engine staggering applied above.
        let budget = create_per_engine_budget(
            disk_budget_mb,
            num_engines,
            retention_policy,
            &engine_config,
        )?;

        // Engine now owns segment store and registry internally - using async open
        let engine = QuiverEngine::open(engine_config, budget)
            .await
            .map_err(|e| format!("Failed to create engine: {}", e))?;

        // Register subscribers using engine's unified API
        let sub_ids = register_subscribers(&engine, subscribers_per_engine)?;
        for sub_id in sub_ids {
            all_sub_ids.push((engine_idx, sub_id));
        }

        engines.push(engine);
    }

    Ok((engines, all_sub_ids))
}
