// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unified stress test runner supporting both TUI and text output modes.

#![allow(clippy::print_stdout)]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use tracing::{info, warn};

use crate::dashboard::{Dashboard, DashboardConfig};
use crate::memory::MemoryTracker;
use crate::stress::{StressStats, calculate_disk_usage};

/// Output mode for the stress runner.
pub enum OutputMode {
    /// TUI dashboard mode.
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

/// Configuration for stress test.
pub struct StressTestConfig {
    pub duration: Duration,
    pub report_interval: Duration,
    pub leak_threshold_mb: f64,
    pub keep_temp: bool,
}

/// Results from a single iteration.
pub struct IterationResult {
    pub bundles_ingested: usize,
    pub bundles_consumed: usize,
}

/// Runs the unified stress test.
///
/// # Arguments
/// * `output` - TUI or Text output mode
/// * `config` - Test configuration
/// * `data_dir` - Directory for data files
/// * `tmp` - Optional temp directory handle (for cleanup)
/// * `dashboard_config` - Dashboard configuration (for TUI updates)
/// * `run_iteration` - Closure that runs a single stress iteration
/// * `cleanup_iteration` - Closure that cleans up after each iteration
pub fn run<F, C>(
    mut output: OutputMode,
    config: StressTestConfig,
    data_dir: PathBuf,
    tmp: Option<TempDir>,
    dashboard_config: DashboardConfig,
    mut run_iteration: F,
    mut cleanup_iteration: C,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(u64) -> Result<IterationResult, Box<dyn std::error::Error>>,
    C: FnMut() -> Result<(), Box<dyn std::error::Error>>,
{
    // Log configuration in text mode
    if !output.is_tui() {
        info!("╔════════════════════════════════════════════════════════════╗");
        info!("║           Quiver Long-Running Stress Test                  ║");
        info!("╚════════════════════════════════════════════════════════════╝");
        info!("");
        info!("Stress Configuration:");
        info!("  Duration: {:?}", config.duration);
        info!(
            "  Report interval: {} seconds",
            config.report_interval.as_secs()
        );
        info!("  Leak threshold: {:.1} MB", config.leak_threshold_mb);
        info!("");
    }

    let mut stress_stats = StressStats::new();
    let start = Instant::now();

    // Initial memory reading
    let initial_mem = MemoryTracker::current_allocated_mb();
    stress_stats.set_initial_memory(initial_mem);

    // Initial disk reading
    let disk_usage = calculate_disk_usage(&data_dir).unwrap_or(0);
    stress_stats.record_disk(disk_usage);

    if !output.is_tui() {
        info!("Initial memory: {:.2} MB", initial_mem);
        info!("");
    }

    // Main stress loop
    let mut quit_requested = false;
    while start.elapsed() < config.duration && !quit_requested {
        stress_stats.iterations += 1;
        let iteration = stress_stats.iterations;

        // Check for quit key (TUI mode only)
        if output.check_quit()? {
            quit_requested = true;
            break;
        }

        // Run one iteration
        let result = run_iteration(iteration)?;
        stress_stats.total_bundles_ingested += result.bundles_ingested as u64;
        stress_stats.total_bundles_consumed += result.bundles_consumed as u64;

        // Record current stats
        let current_mem = MemoryTracker::current_allocated_mb();
        stress_stats.record_memory(current_mem);

        let disk_usage = calculate_disk_usage(&data_dir).unwrap_or(0);
        stress_stats.record_disk(disk_usage);

        // Mode-specific updates
        match &mut output {
            OutputMode::Tui(Some(dashboard)) => {
                dashboard.update(&stress_stats, &data_dir, &dashboard_config)?;
            }
            OutputMode::Text => {
                // Periodic reporting
                if stress_stats.should_report(config.report_interval) {
                    stress_stats.print_status(current_mem, disk_usage);

                    // Early leak warning
                    if stress_stats.detect_memory_leak(config.leak_threshold_mb) {
                        warn!("⚠️  Potential memory leak detected during stress test!");
                    }
                }
            }
            OutputMode::Tui(None) => {}
        }

        // Clean up data between iterations (skip if keeping temp and this might be the last iteration)
        let time_remaining = config.duration.saturating_sub(start.elapsed());
        if !config.keep_temp || time_remaining > Duration::from_secs(1) {
            cleanup_iteration()?;
        }
    }

    // Cleanup TUI and print summary
    output.cleanup()?;

    // Print final summary
    if output.is_tui() {
        stress_stats
            .print_final_summary_stdout(config.leak_threshold_mb, &data_dir.display().to_string());
    } else {
        stress_stats.print_final_summary(config.leak_threshold_mb);
    }

    // Handle temp directory
    if let Some(tmp) = tmp {
        if config.keep_temp {
            let kept_path = tmp.keep();
            if output.is_tui() {
                println!("Keeping temp directory: {}", kept_path.display());
            } else {
                info!("Keeping temp directory: {}", kept_path.display());
            }
        }
    }

    // Check for leaks and return result
    let leaked = stress_stats.detect_memory_leak(config.leak_threshold_mb);
    if leaked {
        Err("Stress test detected potential memory leak".into())
    } else if quit_requested {
        if output.is_tui() {
            println!("Stress test stopped by user");
        }
        Ok(())
    } else {
        if output.is_tui() {
            println!("🎉 Stress test completed successfully!");
        } else {
            info!("🎉 Stress test completed successfully!");
        }
        Ok(())
    }
}
