// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Long-running stress test support for detecting memory and storage leaks.

#![allow(clippy::print_stdout)]

use std::path::Path;
use std::time::{Duration, Instant};

use tracing::{info, warn};

/// Parses a duration string like "10s", "5m", "1h", "24h".
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty duration string".to_string());
    }

    let (num_str, unit) = if let Some(stripped) = s.strip_suffix("ms") {
        (stripped, "ms")
    } else if let Some(stripped) = s.strip_suffix('s') {
        (stripped, "s")
    } else if let Some(stripped) = s.strip_suffix('m') {
        (stripped, "m")
    } else if let Some(stripped) = s.strip_suffix('h') {
        (stripped, "h")
    } else if let Some(stripped) = s.strip_suffix('d') {
        (stripped, "d")
    } else {
        // Default to seconds
        (s, "s")
    };

    let value: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number in duration: {}", num_str))?;

    let duration = match unit {
        "ms" => Duration::from_millis(value),
        "s" => Duration::from_secs(value),
        "m" => Duration::from_secs(value * 60),
        "h" => Duration::from_secs(value * 3600),
        "d" => Duration::from_secs(value * 86400),
        _ => return Err(format!("unknown duration unit: {}", unit)),
    };

    Ok(duration)
}

/// Tracks statistics across multiple stress test iterations.
#[derive(Default)]
pub struct StressStats {
    /// Number of iterations completed
    pub iterations: u64,
    /// Total bundles ingested across all iterations
    pub total_bundles_ingested: u64,
    /// Total bundles consumed across all iterations
    pub total_bundles_consumed: u64,
    /// Memory at start of stress test
    pub initial_memory_mb: f64,
    /// Peak memory observed
    pub peak_memory_mb: f64,
    /// Memory at each checkpoint (iteration, memory_mb)
    pub memory_history: Vec<(u64, f64)>,
    /// Disk usage at each checkpoint (iteration, bytes)
    pub disk_history: Vec<(u64, u64)>,
    /// Time stress test started
    pub start_time: Option<Instant>,
    /// Last report time
    pub last_report: Option<Instant>,
}

impl StressStats {
    /// Creates a new stress stats tracker.
    pub fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Records memory at end of an iteration.
    pub fn record_memory(&mut self, memory_mb: f64) {
        self.memory_history.push((self.iterations, memory_mb));
        if memory_mb > self.peak_memory_mb {
            self.peak_memory_mb = memory_mb;
        }
    }

    /// Records disk usage at end of an iteration.
    pub fn record_disk(&mut self, bytes: u64) {
        self.disk_history.push((self.iterations, bytes));
    }

    /// Sets initial memory baseline.
    pub fn set_initial_memory(&mut self, memory_mb: f64) {
        self.initial_memory_mb = memory_mb;
    }

    /// Returns elapsed time since start.
    pub fn elapsed(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |t| t.elapsed())
    }

    /// Returns memory growth since start (current - initial).
    pub fn memory_growth_mb(&self) -> f64 {
        self.memory_history.last().map(|(_, mb)| *mb).unwrap_or(0.0) - self.initial_memory_mb
    }

    /// Detects if there's a concerning memory growth trend.
    /// Returns true if memory has grown by more than `threshold_mb` over
    /// the last N iterations.
    pub fn detect_memory_leak(&self, threshold_mb: f64) -> bool {
        if self.memory_history.len() < 5 {
            return false;
        }

        // Compare average of last 5 vs first 5 measurements
        let first_avg: f64 = self
            .memory_history
            .iter()
            .take(5)
            .map(|(_, m)| m)
            .sum::<f64>()
            / 5.0;
        let last_avg: f64 = self
            .memory_history
            .iter()
            .rev()
            .take(5)
            .map(|(_, m)| m)
            .sum::<f64>()
            / 5.0;

        let growth = last_avg - first_avg;
        growth > threshold_mb
    }

    /// Checks if it's time for a periodic report.
    pub fn should_report(&mut self, interval: Duration) -> bool {
        match self.last_report {
            None => {
                self.last_report = Some(Instant::now());
                true
            }
            Some(last) => {
                if last.elapsed() >= interval {
                    self.last_report = Some(Instant::now());
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Prints current stress test status.
    pub fn print_status(&self, current_memory_mb: f64, disk_bytes: u64) {
        let elapsed = self.elapsed();
        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;

        info!("╔════════════════════════════════════════════════════════════╗");
        info!("║             Stress Test Status Report                      ║");
        info!("╚════════════════════════════════════════════════════════════╝");
        info!("Elapsed: {:02}:{:02}:{:02}", hours, minutes, seconds);
        info!("Iterations: {}", self.iterations);
        info!("Total bundles ingested: {}", self.total_bundles_ingested);
        info!("Total bundles consumed: {}", self.total_bundles_consumed);
        info!("");
        info!("Memory:");
        info!("  Initial: {:.2} MB", self.initial_memory_mb);
        info!("  Current: {:.2} MB", current_memory_mb);
        info!(
            "  Growth:  {:.2} MB",
            current_memory_mb - self.initial_memory_mb
        );
        info!("  Peak:    {:.2} MB", self.peak_memory_mb);
        info!("");
        info!("Disk usage: {:.2} MB", disk_bytes as f64 / 1024.0 / 1024.0);

        // Calculate throughput
        if elapsed.as_secs() > 0 {
            let bundles_per_sec = self.total_bundles_ingested as f64 / elapsed.as_secs_f64();
            info!("");
            info!("Average throughput: {:.0} bundles/sec", bundles_per_sec);
        }
    }

    /// Prints final summary.
    pub fn print_final_summary(&self, leak_threshold_mb: f64) {
        let elapsed = self.elapsed();
        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;

        info!("");
        info!("╔════════════════════════════════════════════════════════════╗");
        info!("║             Stress Test Final Summary                      ║");
        info!("╚════════════════════════════════════════════════════════════╝");
        info!("Total runtime: {:02}:{:02}:{:02}", hours, minutes, seconds);
        info!("Total iterations: {}", self.iterations);
        info!("Total bundles ingested: {}", self.total_bundles_ingested);
        info!("Total bundles consumed: {}", self.total_bundles_consumed);
        info!("");
        info!("Memory Analysis:");
        info!("  Initial: {:.2} MB", self.initial_memory_mb);
        info!(
            "  Final:   {:.2} MB",
            self.memory_growth_mb() + self.initial_memory_mb
        );
        info!("  Growth:  {:.2} MB", self.memory_growth_mb());
        info!("  Peak:    {:.2} MB", self.peak_memory_mb);

        // Check for leaks
        let potential_leak = self.detect_memory_leak(leak_threshold_mb);
        if potential_leak {
            warn!(
                "⚠️  POTENTIAL MEMORY LEAK DETECTED: growth exceeds {:.0} MB threshold",
                leak_threshold_mb
            );
        } else {
            info!(
                "✓ No memory leak detected (threshold: {:.0} MB)",
                leak_threshold_mb
            );
        }

        // Disk trend
        if self.disk_history.len() >= 2 {
            let first_disk = self.disk_history.first().map(|(_, b)| *b).unwrap_or(0);
            let last_disk = self.disk_history.last().map(|(_, b)| *b).unwrap_or(0);
            let disk_growth = last_disk.saturating_sub(first_disk);
            info!("");
            info!("Disk Analysis:");
            info!("  Initial: {:.2} MB", first_disk as f64 / 1024.0 / 1024.0);
            info!("  Final:   {:.2} MB", last_disk as f64 / 1024.0 / 1024.0);
            info!("  Growth:  {:.2} MB", disk_growth as f64 / 1024.0 / 1024.0);
        }
    }

    /// Prints final summary using println! (for TUI mode where tracing is disabled).
    pub fn print_final_summary_stdout(&self, leak_threshold_mb: f64, data_dir: &str) {
        let elapsed = self.elapsed();
        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;

        println!();
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║             Stress Test Final Summary                      ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!("Data directory: {}", data_dir);
        println!("Total runtime: {:02}:{:02}:{:02}", hours, minutes, seconds);
        println!("Total iterations: {}", self.iterations);
        println!("Total bundles ingested: {}", self.total_bundles_ingested);
        println!("Total bundles consumed: {}", self.total_bundles_consumed);
        println!();
        println!("Memory Analysis:");
        println!("  Initial: {:.2} MB", self.initial_memory_mb);
        println!(
            "  Final:   {:.2} MB",
            self.memory_growth_mb() + self.initial_memory_mb
        );
        println!("  Growth:  {:.2} MB", self.memory_growth_mb());
        println!("  Peak:    {:.2} MB", self.peak_memory_mb);

        // Check for leaks
        let potential_leak = self.detect_memory_leak(leak_threshold_mb);
        if potential_leak {
            println!(
                "⚠️  POTENTIAL MEMORY LEAK DETECTED: growth exceeds {:.0} MB threshold",
                leak_threshold_mb
            );
        } else {
            println!(
                "✓ No memory leak detected (threshold: {:.0} MB)",
                leak_threshold_mb
            );
        }

        // Disk trend
        if self.disk_history.len() >= 2 {
            let first_disk = self.disk_history.first().map(|(_, b)| *b).unwrap_or(0);
            let last_disk = self.disk_history.last().map(|(_, b)| *b).unwrap_or(0);
            let disk_growth = last_disk.saturating_sub(first_disk);
            println!();
            println!("Disk Analysis:");
            println!("  Initial: {:.2} MB", first_disk as f64 / 1024.0 / 1024.0);
            println!("  Final:   {:.2} MB", last_disk as f64 / 1024.0 / 1024.0);
            println!("  Growth:  {:.2} MB", disk_growth as f64 / 1024.0 / 1024.0);
        }
    }
}

/// Calculates total disk usage of a directory recursively.
pub fn calculate_disk_usage(path: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total += calculate_disk_usage(&path)?;
            } else {
                total += entry.metadata()?.len();
            }
        }
    } else if path.is_file() {
        total = std::fs::metadata(path)?.len();
    }

    Ok(total)
}

/// Statistics for steady-state stress test mode.
#[derive(Default)]
pub struct SteadyStateStats {
    /// Total bundles ingested
    pub total_ingested: u64,
    /// Total bundles consumed
    pub total_consumed: u64,
    /// Total segments cleaned up
    pub total_cleaned: u64,
    /// Total segments written
    pub total_segments_written: u64,
    /// Final cleanup duration in milliseconds
    pub cleanup_duration_ms: u64,
    /// Current backlog (ingested - consumed/subscribers)
    pub backlog: u64,
    /// Number of subscribers
    pub subscribers: usize,
    /// Rows per bundle (for calculating rows/sec)
    pub rows_per_bundle: usize,
    /// Approximate bytes per bundle (for calculating MB/s)
    pub bundle_size_bytes: usize,
    /// Memory at start
    pub initial_memory_mb: f64,
    /// Peak memory observed
    pub peak_memory_mb: f64,
    /// Current memory
    pub current_memory_mb: f64,
    /// Initial disk usage
    pub initial_disk_bytes: u64,
    /// Peak disk usage
    pub peak_disk_bytes: u64,
    /// Current disk usage
    pub current_disk_bytes: u64,
    /// Time stress test started
    pub start_time: Option<Instant>,
}

impl SteadyStateStats {
    /// Creates a new steady-state stats tracker.
    pub fn new(subscribers: usize, rows_per_bundle: usize, bundle_size_bytes: usize) -> Self {
        Self {
            subscribers,
            rows_per_bundle,
            bundle_size_bytes,
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Returns elapsed time since start.
    pub fn elapsed(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |t| t.elapsed())
    }

    /// Updates memory stats.
    pub fn update_memory(&mut self, current_mb: f64) {
        self.current_memory_mb = current_mb;
        if current_mb > self.peak_memory_mb {
            self.peak_memory_mb = current_mb;
        }
    }

    /// Updates disk stats.
    pub fn update_disk(&mut self, current_bytes: u64) {
        self.current_disk_bytes = current_bytes;
        if current_bytes > self.peak_disk_bytes {
            self.peak_disk_bytes = current_bytes;
        }
    }

    /// Sets initial metrics.
    pub fn set_initial(&mut self, memory_mb: f64, disk_bytes: u64) {
        self.initial_memory_mb = memory_mb;
        self.current_memory_mb = memory_mb;
        self.initial_disk_bytes = disk_bytes;
        self.current_disk_bytes = disk_bytes;
    }

    /// Updates counters.
    pub fn update_counters(
        &mut self,
        ingested: u64,
        consumed: u64,
        cleaned: u64,
        segments_written: u64,
    ) {
        self.total_ingested = ingested;
        self.total_consumed = consumed;
        self.total_cleaned = cleaned;
        self.total_segments_written = segments_written;
        // Backlog is ingested - (consumed / subscribers)
        if self.subscribers > 0 {
            self.backlog = ingested.saturating_sub(consumed / self.subscribers as u64);
        }
    }

    /// Returns ingest rate (bundles/sec).
    pub fn ingest_rate(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_ingested as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Returns consume rate (bundles/sec).
    pub fn consume_rate(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_consumed as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Returns ingest rate in rows/sec.
    pub fn ingest_rows_rate(&self) -> f64 {
        self.ingest_rate() * self.rows_per_bundle as f64
    }

    /// Returns consume rate in rows/sec.
    pub fn consume_rows_rate(&self) -> f64 {
        self.consume_rate() * self.rows_per_bundle as f64
    }

    /// Returns ingest rate in MB/sec.
    pub fn ingest_mb_rate(&self) -> f64 {
        self.ingest_rate() * self.bundle_size_bytes as f64 / 1024.0 / 1024.0
    }

    /// Returns consume rate in MB/sec.
    pub fn consume_mb_rate(&self) -> f64 {
        self.consume_rate() * self.bundle_size_bytes as f64 / 1024.0 / 1024.0
    }

    /// Returns memory growth.
    pub fn memory_growth_mb(&self) -> f64 {
        self.current_memory_mb - self.initial_memory_mb
    }

    /// Prints final summary using println! (for TUI mode).
    pub fn print_final_summary_stdout(&self, leak_threshold_mb: f64, data_dir: &str) {
        let elapsed = self.elapsed();
        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;

        println!();
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║           Steady-State Final Summary                       ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!("Data directory: {}", data_dir);
        println!("Total runtime: {:02}:{:02}:{:02}", hours, minutes, seconds);
        println!("Total bundles ingested: {}", self.total_ingested);
        println!("Total bundles consumed: {}", self.total_consumed);
        println!("Total segments written: {}", self.total_segments_written);
        println!("Total segments cleaned: {}", self.total_cleaned);
        println!("Final cleanup time: {} ms", self.cleanup_duration_ms);
        println!();
        println!("Throughput:");
        println!(
            "  Ingest:  {:.0} bundles/s, {:.0} rows/s, {:.1} MB/s",
            self.ingest_rate(),
            self.ingest_rows_rate(),
            self.ingest_mb_rate()
        );
        println!(
            "  Consume: {:.0} bundles/s, {:.0} rows/s, {:.1} MB/s",
            self.consume_rate(),
            self.consume_rows_rate(),
            self.consume_mb_rate()
        );
        println!();
        println!("Memory Analysis:");
        println!("  Initial: {:.2} MB", self.initial_memory_mb);
        println!("  Final:   {:.2} MB", self.current_memory_mb);
        println!("  Peak:    {:.2} MB", self.peak_memory_mb);
        println!("  Growth:  {:.2} MB", self.memory_growth_mb());
        println!();
        println!("Disk Analysis:");
        println!(
            "  Initial: {:.2} MB",
            self.initial_disk_bytes as f64 / 1024.0 / 1024.0
        );
        println!(
            "  Final:   {:.2} MB",
            self.current_disk_bytes as f64 / 1024.0 / 1024.0
        );
        println!(
            "  Peak:    {:.2} MB",
            self.peak_disk_bytes as f64 / 1024.0 / 1024.0
        );

        // Check for memory leak
        let growth = self.memory_growth_mb();
        if growth > leak_threshold_mb {
            println!();
            println!(
                "⚠️  Memory growth ({:.2} MB) exceeds threshold ({:.0} MB)",
                growth, leak_threshold_mb
            );
        } else {
            println!();
            println!("✓ Memory growth within threshold");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("10s").unwrap(), Duration::from_secs(10));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("24h").unwrap(), Duration::from_secs(86400));
        assert_eq!(parse_duration("2d").unwrap(), Duration::from_secs(172800));
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(parse_duration("60").unwrap(), Duration::from_secs(60)); // default to seconds
    }

    #[test]
    fn test_leak_detection() {
        let mut stats = StressStats::new();
        stats.set_initial_memory(100.0);

        // No leak: stable memory
        for i in 0..10 {
            stats.iterations = i;
            stats.record_memory(100.0 + (i as f64 * 0.1)); // small fluctuation
        }
        assert!(!stats.detect_memory_leak(50.0));

        // Leak: growing memory
        let mut stats2 = StressStats::new();
        stats2.set_initial_memory(100.0);
        for i in 0..10 {
            stats2.iterations = i;
            stats2.record_memory(100.0 + (i as f64 * 20.0)); // 20MB per iteration
        }
        assert!(stats2.detect_memory_leak(50.0));
    }
}
