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

use quiver::SegmentReadMode;
use quiver::subscriber::SubscriberId;
use quiver::{QuiverConfig, QuiverEngine};
use tempfile::TempDir;
use tracing::{info, warn};

use crate::bundle;
use crate::dashboard::{Dashboard, SteadyStateConfig};
use crate::memory::MemoryTracker;
use crate::stress::{SteadyStateStats, calculate_disk_usage};
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
    pub progress_flush_interval: usize,
    pub segment_size_mb: u64,
    /// Read mode for segment files (mmap vs standard I/O).
    pub read_mode: SegmentReadMode,
    pub leak_threshold_mb: f64,
    pub keep_temp: bool,
    pub report_interval: Duration,
    pub wal_flush_interval_ms: u64,
    pub no_wal: bool,
    pub engines: usize,
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
    let running = Arc::new(AtomicBool::new(true));
    let ingest_running = Arc::new(AtomicBool::new(true));
    let total_ingested = Arc::new(AtomicU64::new(0));
    let total_consumed = Arc::new(AtomicU64::new(0));
    let total_cleaned = Arc::new(AtomicU64::new(0));
    let segments_written = Arc::new(AtomicU64::new(0));

    // Create per-engine resources (engine owns segment store and registry)
    let mut engines: Vec<Arc<QuiverEngine>> = Vec::with_capacity(config.engines);
    let mut all_sub_ids: Vec<(usize, SubscriberId)> = Vec::new();

    for engine_idx in 0..config.engines {
        // Each engine gets its own subdirectory
        let engine_dir = if config.engines > 1 {
            data_dir.join(format!("engine_{}", engine_idx))
        } else {
            data_dir.clone()
        };
        std::fs::create_dir_all(&engine_dir)?;

        // Create engine with staggered timing to avoid synchronized I/O
        let engine_config = create_engine_config(
            &engine_dir,
            config.segment_size_mb,
            config.wal_flush_interval_ms,
            config.no_wal,
            config.read_mode,
            engine_idx,
            config.engines,
        );

        // Log staggered config for multi-engine runs
        if config.engines > 1 && !output.is_tui() {
            let seg_mb = engine_config.segment.target_size_bytes.get() / 1024 / 1024;
            let flush_ms = engine_config.wal.flush_interval.as_millis();
            let startup_delay_ms = 100 * engine_idx;
            output.log(&format!(
                "  Engine {}: segment={}MB, wal_flush={}ms, startup_delay={}ms",
                engine_idx, seg_mb, flush_ms, startup_delay_ms
            ));
        }

        // Engine now owns segment store and registry internally
        let engine = QuiverEngine::new(engine_config)?;

        // Register subscribers using engine's unified API
        let sub_ids = register_subscribers_on_engine(&engine, config.subscribers)?;
        for sub_id in sub_ids {
            all_sub_ids.push((engine_idx, sub_id));
        }

        engines.push(engine);
    }

    output.log(&format!(
        "Started {} engine(s) with {} total subscribers",
        config.engines,
        all_sub_ids.len()
    ));

    let start = Instant::now();
    let cleanup_interval = Duration::from_secs(2);
    let mut last_cleanup = Instant::now();
    let mut last_report = Instant::now();

    // Spawn ingest threads (one per engine) with staggered startup delays
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
        let handle = spawn_ingest_thread(
            engine.clone(),
            test_bundles.clone(),
            ingest_running.clone(),
            total_ingested.clone(),
            startup_delay_ms,
        );
        ingest_handles.push(handle);
    }

    // Spawn subscriber threads (distributed across engines)
    let mut subscriber_handles = Vec::new();
    for (engine_idx, sub_id) in &all_sub_ids {
        let engine = engines[*engine_idx].clone();
        let sub_running = running.clone();
        let sub_consumed = total_consumed.clone();
        let delay_ms = config.subscriber_delay_ms;
        let flush_interval = config.progress_flush_interval;
        let sub_id_clone = sub_id.clone();

        let handle = thread::spawn(move || {
            let delay = SubscriberDelay::new(delay_ms);
            let mut bundles_since_flush = 0;

            while sub_running.load(Ordering::Relaxed) {
                // Use engine's unified subscriber API
                let bundle_handle = match engine.next_bundle_blocking(
                    &sub_id_clone,
                    None,
                    || !sub_running.load(Ordering::Relaxed),
                ) {
                    Ok(Some(h)) => h,
                    Ok(None) => continue,
                    Err(_) => break,
                };

                delay.apply();
                bundle_handle.ack();
                let _ = sub_consumed.fetch_add(1, Ordering::Relaxed);
                bundles_since_flush += 1;

                if bundles_since_flush >= flush_interval {
                    let _ = engine.flush_progress();
                    bundles_since_flush = 0;
                }
            }
        });

        subscriber_handles.push(handle);
    }

    // Main monitoring loop
    let mut quit_requested = false;
    while start.elapsed() < config.duration && !quit_requested {
        thread::sleep(Duration::from_millis(100));

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
                // Sum WAL bytes across all engines
                let wal_bytes: u64 = engines.iter().map(|e| e.wal_bytes_written()).sum();
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
    for handle in ingest_handles {
        let _ = handle.join();
    }

    let final_ingested = total_ingested.load(Ordering::Relaxed);

    // 2. Finalize any remaining open segments (all engines)
    for engine in &engines {
        if let Err(e) = engine.shutdown() {
            output.log_warn(&format!("Engine shutdown error: {}", e));
        }
    }

    // 3. Final segment scan (all engines)
    let final_segment_count = engines
        .iter()
        .map(|e| e.segment_store().segment_count() as u64)
        .sum::<u64>()
        .max(segments_written.load(Ordering::Relaxed));

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

    // Flush final progress (all engines)
    for engine in &engines {
        let _ = engine.flush_progress();
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
        output.log(&format!("Engines: {}", config.engines));
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

fn register_subscribers_on_engine(
    engine: &Arc<QuiverEngine>,
    count: usize,
) -> Result<Vec<SubscriberId>, Box<dyn std::error::Error>> {
    let mut sub_ids = Vec::with_capacity(count);
    for sub_id in 0..count {
        let sub_name = format!("subscriber-{}", sub_id);
        let id = SubscriberId::new(&sub_name)?;
        engine.register_subscriber(id.clone())?;
        engine.activate_subscriber(&id)?;
        sub_ids.push(id);
    }
    Ok(sub_ids)
}

fn spawn_ingest_thread(
    engine: Arc<QuiverEngine>,
    test_bundles: Arc<Vec<bundle::TestBundle>>,
    ingest_running: Arc<AtomicBool>,
    total_ingested: Arc<AtomicU64>,
    startup_delay_ms: u64,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // Staggered startup to desynchronize segment writes
        if startup_delay_ms > 0 {
            thread::sleep(Duration::from_millis(startup_delay_ms));
        }
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
        std::num::NonZeroU64::new(256 * 1024 * 1024).expect("256MB is non-zero");
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
