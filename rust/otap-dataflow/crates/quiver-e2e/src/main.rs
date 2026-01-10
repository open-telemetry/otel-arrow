// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Quiver End-to-End Test Harness
//!
//! This binary exercises the complete Quiver persistence flow with:
//! - Configurable data volume and bundle sizes
//! - Memory consumption tracking (via jemalloc)
//! - Multiple concurrent subscribers
//! - Configurable segment read mode (standard or mmap)
//! - Concurrent ingest and consume for production-like testing
//!
//! Usage:
//!   cargo run -p quiver-e2e -- --help
//!   cargo run -p quiver-e2e -- --duration 1h --no-tui
//!   cargo run -p quiver-e2e -- --duration 10s --no-tui --bundles 100

mod bundle;
mod dashboard;
mod memory;
mod stats;
mod steady_state;
mod subscriber;

use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use quiver::SegmentReadMode;
use quiver::config::RetentionPolicy;
use tempfile::TempDir;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use crate::dashboard::Dashboard;
use crate::stats::parse_duration;

// Use jemalloc for accurate memory tracking
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Read mode for segment files
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum ReadModeArg {
    /// Standard file I/O
    Standard,
    /// Memory-mapped I/O
    #[default]
    Mmap,
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

    /// Segment read mode (standard or mmap)
    #[arg(long, default_value = "mmap")]
    read_mode: ReadModeArg,

    /// How often to call maintain() - flushes progress and cleans up segments (in milliseconds, 0 = never)
    #[arg(long, default_value = "1000")]
    maintain_interval_ms: u64,

    /// Keep temp directory after test (for inspection)
    #[arg(long)]
    keep_temp: bool,

    /// Test duration (e.g., "10s", "10m", "1h", "24h")
    #[arg(long, default_value = "10s")]
    duration: String,

    /// Report interval in seconds (memory, disk, throughput)
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

    /// Disable TUI dashboard (use text output instead)
    #[arg(long)]
    no_tui: bool,

    /// WAL flush interval in milliseconds (0 = flush after every write)
    #[arg(long, default_value = "25")]
    wal_flush_interval_ms: u64,

    /// Disable WAL for higher throughput (data only durable after segment finalization)
    #[arg(long)]
    no_wal: bool,

    /// Number of parallel QuiverEngine instances (each with its own data directory)
    #[arg(long, default_value = "1")]
    engines: usize,

    /// Disk budget cap in MB (0 = unlimited, default 10 GB)
    #[arg(long, default_value = "10240")]
    disk_budget_mb: u64,

    /// Retention policy when disk budget is exceeded (backpressure or drop-oldest)
    #[arg(long, default_value = "backpressure", value_parser = parse_retention_policy)]
    retention_policy: RetentionPolicy,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Parse duration
    let duration = parse_duration(&args.duration)
        .map_err(|e| format!("Invalid duration '{}': {}", args.duration, e))?;

    // Check if using TUI - don't initialize tracing in TUI mode
    // as it interferes with the terminal display
    let use_tui = !args.no_tui;

    if !use_tui {
        // Initialize tracing only for non-TUI modes
        let tracing_sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();
        let _ = tracing::subscriber::set_global_default(tracing_sub);
    }

    run_steady_state_mode(&args, duration)
}

/// Runs steady-state stress test: single long-running QuiverEngine with concurrent ingest/consume.
///
/// This mode:
/// - Creates ONE QuiverEngine that runs for the entire duration
/// - Continuously ingests data while subscribers consume concurrently
/// - Uses a shared SubscriberRegistry for all subscribers to enable coordinated cleanup
/// - Periodically cleans up completed segments from disk
/// - Tests whether disk/memory stabilize over time
fn run_steady_state_mode(
    args: &Args,
    duration: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    use steady_state::{OutputMode, SteadyStateTestConfig};

    // Set up data directory
    let (tmp, data_dir) = setup_data_dir(args)?;

    // Convert read mode
    let read_mode = match args.read_mode {
        ReadModeArg::Standard => SegmentReadMode::Standard,
        ReadModeArg::Mmap => SegmentReadMode::Mmap,
    };

    // Build config
    let config = SteadyStateTestConfig {
        duration,
        bundles: args.bundles,
        rows_per_bundle: args.rows_per_bundle,
        string_size: args.string_size,
        subscribers: args.subscribers,
        subscriber_delay_ms: args.subscriber_delay_ms,
        maintain_interval_ms: args.maintain_interval_ms,
        segment_size_mb: args.segment_size_mb,
        read_mode,
        leak_threshold_mb: args.leak_threshold_mb,
        keep_temp: args.keep_temp,
        report_interval: Duration::from_secs(args.report_interval),
        wal_flush_interval_ms: args.wal_flush_interval_ms,
        no_wal: args.no_wal,
        engines: args.engines,
        disk_budget_mb: args.disk_budget_mb,
        retention_policy: args.retention_policy,
    };

    // Create output mode (TUI or Text)
    let output_mode = if !args.no_tui {
        let dashboard = Dashboard::new(duration)?;
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

/// Parses a retention policy from CLI string.
fn parse_retention_policy(s: &str) -> Result<RetentionPolicy, String> {
    match s.to_lowercase().as_str() {
        "backpressure" | "bp" => Ok(RetentionPolicy::Backpressure),
        "drop-oldest" | "dropoldest" | "drop" => Ok(RetentionPolicy::DropOldest),
        _ => Err(format!(
            "Invalid retention policy '{}'. Use 'backpressure' or 'drop-oldest'",
            s
        )),
    }
}
