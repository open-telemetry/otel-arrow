// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Statistics and utility functions for the stress test harness.

#![allow(clippy::print_stdout)]

use std::path::Path;
use std::time::{Duration, Instant};

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
    /// Total segments cleaned up (NOTE: derived from written - active, may differ from explicit tracking)
    pub total_cleaned: u64,
    /// Total segments written
    pub total_segments_written: u64,
    /// Currently active segments (from actual segment store count)
    pub active_segments: u64,
    /// Total backpressure events (StorageAtCapacity errors)
    pub backpressure_count: u64,
    /// Total segments force-dropped (DropOldest policy)
    pub force_dropped_segments: u64,
    /// Total bundles lost due to force-dropped segments
    pub force_dropped_bundles: u64,
    /// Final cleanup duration in milliseconds
    pub cleanup_duration_ms: u64,
    /// Current buffered bundles (ingested - consumed - dropped)
    pub buffered: u64,
    /// Number of subscribers per engine (used for buffered calculation)
    /// Each subscriber only consumes from its own engine's bundles.
    pub subscribers: usize,
    /// Rows per bundle (for calculating rows/sec)
    pub rows_per_bundle: usize,
    /// Approximate bytes per bundle (for calculating MB/s)
    pub bundle_size_bytes: usize,
    /// Previous ingested count (for live rate calculation)
    prev_ingested: u64,
    /// Previous consumed count (for live rate calculation)
    prev_consumed: u64,
    /// Timestamp of previous sample (for live rate calculation)
    prev_sample_time: Option<Instant>,
    /// Live ingest rate (bundles/sec over last sample interval)
    live_ingest_rate: f64,
    /// Live consume rate (bundles/sec over last sample interval)
    live_consume_rate: f64,
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
        active_segments: u64,
        segments_written: u64,
        backpressure: u64,
        force_dropped_segments: u64,
        force_dropped_bundles: u64,
    ) {
        // Calculate live rates from deltas
        let now = Instant::now();
        if let Some(prev_time) = self.prev_sample_time {
            let dt = now.duration_since(prev_time).as_secs_f64();
            if dt > 0.0 {
                let ingested_delta = ingested.saturating_sub(self.prev_ingested);
                let consumed_delta = consumed.saturating_sub(self.prev_consumed);
                self.live_ingest_rate = ingested_delta as f64 / dt;
                self.live_consume_rate = consumed_delta as f64 / dt;
            }
        }
        // Store for next sample
        self.prev_ingested = ingested;
        self.prev_consumed = consumed;
        self.prev_sample_time = Some(now);

        self.total_ingested = ingested;
        self.total_consumed = consumed;
        self.active_segments = active_segments;
        self.total_segments_written = segments_written;
        // Derive cleaned from written - active (accurate regardless of cleanup source)
        self.total_cleaned = segments_written.saturating_sub(active_segments);
        self.backpressure_count = backpressure;
        self.force_dropped_segments = force_dropped_segments;
        self.force_dropped_bundles = force_dropped_bundles;
        // Buffered = bundles sitting in Quiver's buffer
        // = ingested - consumed - dropped
        // (dropped bundles can never be consumed, so they don't count as buffered)
        if self.subscribers > 0 {
            let effective_consumed = consumed / self.subscribers as u64;
            self.buffered = ingested
                .saturating_sub(effective_consumed)
                .saturating_sub(force_dropped_bundles);
        }
    }

    /// Returns average ingest rate since start (bundles/sec).
    pub fn ingest_rate(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_ingested as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Returns average consume rate since start (bundles/sec).
    pub fn consume_rate(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_consumed as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Returns live ingest rate (bundles/sec over last sample interval).
    pub fn live_ingest_rate(&self) -> f64 {
        self.live_ingest_rate
    }

    /// Returns live consume rate (bundles/sec over last sample interval).
    pub fn live_consume_rate(&self) -> f64 {
        self.live_consume_rate
    }

    /// Returns live consume rate in rows/sec.
    pub fn live_consume_rows_rate(&self) -> f64 {
        self.live_consume_rate * self.rows_per_bundle as f64
    }

    /// Returns live consume rate in MB/sec.
    pub fn live_consume_mb_rate(&self) -> f64 {
        self.live_consume_rate * self.bundle_size_bytes as f64 / 1024.0 / 1024.0
    }

    /// Returns average ingest rate in rows/sec.
    pub fn ingest_rows_rate(&self) -> f64 {
        self.ingest_rate() * self.rows_per_bundle as f64
    }

    /// Returns average consume rate in rows/sec.
    pub fn consume_rows_rate(&self) -> f64 {
        self.consume_rate() * self.rows_per_bundle as f64
    }

    /// Returns average ingest rate in MB/sec.
    pub fn ingest_mb_rate(&self) -> f64 {
        self.ingest_rate() * self.bundle_size_bytes as f64 / 1024.0 / 1024.0
    }

    /// Returns average consume rate in MB/sec.
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
        println!(
            "Dropped: {} segments / {} bundles (DropOldest policy)",
            self.force_dropped_segments, self.force_dropped_bundles
        );
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
}
