// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Live dashboard for stress test monitoring using ratatui.

#[cfg(not(windows))]
use std::fs;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::stats::SteadyStateStats;

/// Reads the syscall counts (reads and writes separately).
/// On Linux: reads from /proc/self/io (syscr, syscw).
/// On Windows: uses GetProcessIoCounters API for read/write operation counts.
/// Returns (read_ops, write_ops) or (0, 0) if unavailable.
#[cfg(not(windows))]
fn read_process_syscalls() -> (u64, u64) {
    let content = match fs::read_to_string("/proc/self/io") {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };

    let mut syscr = 0u64;
    let mut syscw = 0u64;

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("syscr: ") {
            syscr = val.trim().parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("syscw: ") {
            syscw = val.trim().parse().unwrap_or(0);
        }
    }

    (syscr, syscw)
}

#[cfg(windows)]
fn read_process_syscalls() -> (u64, u64) {
    // On Windows, sysinfo's disk_usage() provides I/O byte counts.
    // We convert to estimated operation counts by dividing by typical I/O block size (4KB).
    // This provides roughly comparable metrics to Linux syscr/syscw.
    const IO_BLOCK_SIZE: u64 = 4096;
    let mut sys = System::new();
    let pid = Pid::from_u32(std::process::id());
    let _ = sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    if let Some(proc) = sys.process(pid) {
        let usage = proc.disk_usage();
        // Convert bytes to estimated operations (divide by 4KB block size)
        let read_ops = usage.total_read_bytes / IO_BLOCK_SIZE;
        let write_ops = usage.total_written_bytes / IO_BLOCK_SIZE;
        (read_ops, write_ops)
    } else {
        (0, 0)
    }
}

/// Reads page fault counts.
/// On Linux: reads from /proc/self/stat.
/// On Windows: uses GetProcessMemoryInfo API for page fault count.
/// Returns (minflt, majflt) where:
/// - minflt: minor page faults (page in memory, just needed mapping)
/// - majflt: major page faults (page read from disk - relevant for mmap I/O)
///
/// Note: Windows only provides a single PageFaultCount, returned as (pagefaults, 0).
#[cfg(not(windows))]
fn read_process_page_faults() -> (u64, u64) {
    let content = match fs::read_to_string("/proc/self/stat") {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };

    // /proc/self/stat format: pid (comm) state ... fields
    // Fields after (comm): minflt is field 10, majflt is field 12 (1-indexed)
    // We need to skip past (comm) which may contain spaces
    let start = match content.find(')') {
        Some(pos) => pos + 2, // skip ") "
        None => return (0, 0),
    };

    let fields: Vec<&str> = content[start..].split_whitespace().collect();
    // After (comm), fields are 0-indexed:
    // 0=state, 1=ppid, 2=pgrp, 3=session, 4=tty_nr, 5=tpgid,
    // 6=flags, 7=minflt, 8=cminflt, 9=majflt, 10=cmajflt, ...
    let minflt = fields.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
    let majflt = fields.get(9).and_then(|s| s.parse().ok()).unwrap_or(0);

    (minflt, majflt)
}

#[cfg(windows)]
fn read_process_page_faults() -> (u64, u64) {
    // Page fault counts are not directly available via sysinfo on Windows.
    // Return (0, 0) - the dashboard will show 0 for page faults on Windows.
    // This is acceptable as page faults are less meaningful on Windows for this use case.
    (0, 0)
}

/// Reads RSS (Resident Set Size).
/// On Linux: reads from /proc/self/status.
/// On Windows: uses sysinfo to get process memory (working set).
/// Returns RSS in bytes. This includes all resident memory: heap, stack, and mmap'd pages.
#[cfg(not(windows))]
fn read_process_rss_bytes() -> u64 {
    let content = match fs::read_to_string("/proc/self/status") {
        Ok(c) => c,
        Err(_) => return 0,
    };

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("VmRSS:") {
            // VmRSS is in kB, e.g., "VmRSS:    12345 kB"
            let val = val.trim();
            if let Some(kb_str) = val.strip_suffix(" kB") {
                if let Ok(kb) = kb_str.trim().parse::<u64>() {
                    return kb * 1024; // Convert to bytes
                }
            }
        }
    }
    0
}

#[cfg(windows)]
fn read_process_rss_bytes() -> u64 {
    let mut sys = System::new();
    let pid = Pid::from_u32(std::process::id());
    let _ = sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    // sysinfo's memory() on Windows returns the working set size in bytes
    sys.process(pid).map(|p| p.memory()).unwrap_or(0)
}

/// Live dashboard state for stress testing.
pub struct Dashboard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    start_time: Instant,
    target_duration: Duration,
    /// Maximum history length based on terminal width
    max_history_len: usize,
    /// Recent consumed throughput samples for sparkline (bundles/sec)
    consumed_throughput_history: Vec<u64>,
    /// Recent heap memory samples for sparkline (MB * 10 for precision) - jemalloc allocated
    memory_history: Vec<u64>,
    /// Recent RSS samples for sparkline (MB) - includes mmap'd pages
    rss_history: Vec<u64>,
    /// Recent CPU samples for sparkline (percentage * 10 for precision)
    cpu_history: Vec<u64>,
    /// Recent disk usage samples for sparkline (MB)
    disk_history: Vec<u64>,
    /// Recent segment write rate samples for sparkline (KB/s)
    segment_write_history: Vec<u64>,
    /// Recent WAL write rate samples for sparkline (KB/s)
    wal_write_history: Vec<u64>,
    /// Recent read IOPS samples for sparkline (ops/s from syscr)
    read_iops_history: Vec<u64>,
    /// Recent write IOPS samples for sparkline (ops/s from syscw)
    write_iops_history: Vec<u64>,
    /// Recent minor page fault samples for sparkline (soft faults - page in cache)
    minflt_history: Vec<u64>,
    /// Recent major page fault samples for sparkline (hard faults - disk read)
    majflt_history: Vec<u64>,
    /// Recent buffered samples for sparkline (bundles)
    buffered_history: Vec<u64>,
    /// System info for CPU monitoring
    system: System,
    /// Our process ID for CPU tracking
    pid: Pid,
    /// Last iteration time for throughput calculation
    last_iteration_time: Instant,
    last_iteration_consumed: u64,
    /// Last segment file size for write rate calculation
    last_segment_bytes: u64,
    /// Last WAL file size for write rate calculation
    last_wal_bytes: u64,
    /// Last read syscall count for IOPS calculation
    last_syscr: u64,
    /// Last write syscall count for IOPS calculation
    last_syscw: u64,
    /// Last minor page fault count
    last_minflt: u64,
    /// Last major page fault count
    last_majflt: u64,
    /// Whether the terminal has already been restored.
    restored: bool,
}

impl Dashboard {
    /// Creates and initializes the dashboard.
    pub fn new(target_duration: Duration) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Calculate max history based on terminal width
        // Account for: 2 margin + 2 border chars = 4 chars overhead per column
        // With two-column layout, each sparkline gets half the width
        let term_width = terminal.size()?.width as usize;
        let column_width = term_width / 2;
        let max_history_len = column_width.saturating_sub(4).max(1);

        // Get initial syscall counts
        // Note: WAL and segment bytes are tracked via cumulative engine counters, starting at 0
        let (initial_syscr, initial_syscw) = read_process_syscalls();
        let (initial_minflt, initial_majflt) = read_process_page_faults();

        Ok(Self {
            terminal,
            start_time: Instant::now(),
            target_duration,
            max_history_len,
            consumed_throughput_history: Vec::with_capacity(max_history_len),
            memory_history: Vec::with_capacity(max_history_len),
            rss_history: Vec::with_capacity(max_history_len),
            cpu_history: Vec::with_capacity(max_history_len),
            disk_history: Vec::with_capacity(max_history_len),
            segment_write_history: Vec::with_capacity(max_history_len),
            wal_write_history: Vec::with_capacity(max_history_len),
            read_iops_history: Vec::with_capacity(max_history_len),
            write_iops_history: Vec::with_capacity(max_history_len),
            minflt_history: Vec::with_capacity(max_history_len),
            majflt_history: Vec::with_capacity(max_history_len),
            buffered_history: Vec::with_capacity(max_history_len),
            system: System::new(),
            pid: Pid::from_u32(std::process::id()),
            last_iteration_time: Instant::now(),
            last_iteration_consumed: 0,
            last_segment_bytes: 0, // Cumulative from engine, starts at 0
            last_wal_bytes: 0,     // Cumulative from engine, starts at 0
            last_syscr: initial_syscr,
            last_syscw: initial_syscw,
            last_minflt: initial_minflt,
            last_majflt: initial_majflt,
            restored: false,
        })
    }

    /// Checks if user pressed 'q' to quit.
    pub fn check_quit(&self) -> io::Result<bool> {
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Restores terminal state.
    pub fn cleanup(mut self) -> io::Result<()> {
        self.restore_terminal();
        Ok(())
    }

    /// Best-effort terminal restore (used by both `cleanup` and `Drop`).
    /// Idempotent: subsequent calls after the first are no-ops.
    fn restore_terminal(&mut self) {
        if self.restored {
            return;
        }
        self.restored = true;
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }

    /// Trims all history vectors to max_history_len (for terminal resize).
    fn trim_histories(&mut self) {
        fn trim_vec(v: &mut Vec<u64>, max_len: usize) {
            while v.len() > max_len {
                let _ = v.remove(0);
            }
        }
        trim_vec(&mut self.consumed_throughput_history, self.max_history_len);
        trim_vec(&mut self.memory_history, self.max_history_len);
        trim_vec(&mut self.rss_history, self.max_history_len);
        trim_vec(&mut self.cpu_history, self.max_history_len);
        trim_vec(&mut self.disk_history, self.max_history_len);
        trim_vec(&mut self.segment_write_history, self.max_history_len);
        trim_vec(&mut self.wal_write_history, self.max_history_len);
        trim_vec(&mut self.read_iops_history, self.max_history_len);
        trim_vec(&mut self.write_iops_history, self.max_history_len);
        trim_vec(&mut self.minflt_history, self.max_history_len);
        trim_vec(&mut self.majflt_history, self.max_history_len);
        trim_vec(&mut self.buffered_history, self.max_history_len);
    }

    /// Updates and renders the dashboard for steady-state mode.
    ///
    /// `wal_bytes_written` should be the cumulative WAL bytes from `engine.wal_bytes_written()`.
    /// `segment_bytes_written` should be the cumulative segment bytes from `engine.segment_bytes_written()`.
    pub fn update_steady_state(
        &mut self,
        stats: &SteadyStateStats,
        config: &SteadyStateConfig,
        wal_bytes_written: u64,
        segment_bytes_written: u64,
    ) -> io::Result<()> {
        let elapsed = self.start_time.elapsed();

        // Update max_history_len based on current terminal width BEFORE sampling
        // This ensures we use the new width when deciding whether to trim
        // With two-column layout, each sparkline gets half the width
        if let Ok(size) = self.terminal.size() {
            let column_width = size.width as usize / 2;
            let new_max = column_width.saturating_sub(4).max(1);
            if new_max < self.max_history_len {
                // Terminal shrank - trim histories
                self.max_history_len = new_max;
                self.trim_histories();
            } else if new_max > self.max_history_len {
                // Terminal grew - just update max, histories will grow naturally
                self.max_history_len = new_max;
            }
        }

        // Calculate current consumed throughput
        let now = Instant::now();
        let iteration_elapsed = now.duration_since(self.last_iteration_time);
        let consumed_this_period = stats
            .total_consumed
            .saturating_sub(self.last_iteration_consumed);

        if iteration_elapsed >= Duration::from_secs(1) {
            let consumed_throughput =
                (consumed_this_period as f64 / iteration_elapsed.as_secs_f64()) as u64;
            self.consumed_throughput_history.push(consumed_throughput);
            if self.consumed_throughput_history.len() > self.max_history_len {
                let _ = self.consumed_throughput_history.remove(0);
            }

            // Memory history (store as MB * 10 for better sparkline resolution)
            // This is heap-only memory from jemalloc
            self.memory_history
                .push((stats.current_memory_mb * 10.0) as u64);
            if self.memory_history.len() > self.max_history_len {
                let _ = self.memory_history.remove(0);
            }

            // RSS history (store as MB) - includes mmap'd pages
            let rss_mb = read_process_rss_bytes() / 1024 / 1024;
            self.rss_history.push(rss_mb);
            if self.rss_history.len() > self.max_history_len {
                let _ = self.rss_history.remove(0);
            }

            // CPU history (refresh and get process-specific CPU usage)
            // We use process CPU rather than global_cpu_usage() to show this process's usage
            let _ = self.system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[self.pid]),
                true, // refresh all process info
                ProcessRefreshKind::nothing().with_cpu(),
            );
            let cpu_usage = self
                .system
                .process(self.pid)
                .map(|p| p.cpu_usage())
                .unwrap_or(0.0);
            self.cpu_history.push((cpu_usage * 10.0) as u64); // Store as % * 10
            if self.cpu_history.len() > self.max_history_len {
                let _ = self.cpu_history.remove(0);
            }

            // Disk history (store as MB)
            let disk_mb = stats.current_disk_bytes / 1024 / 1024;
            self.disk_history.push(disk_mb);
            if self.disk_history.len() > self.max_history_len {
                let _ = self.disk_history.remove(0);
            }

            // Segment write rate (KB/s) - based on cumulative bytes from engine
            // This is accurate even when segments are cleaned between samples
            let seg_bytes_this_period =
                segment_bytes_written.saturating_sub(self.last_segment_bytes);
            let seg_rate_kbs =
                (seg_bytes_this_period as f64 / iteration_elapsed.as_secs_f64() / 1024.0) as u64;
            self.segment_write_history.push(seg_rate_kbs);
            if self.segment_write_history.len() > self.max_history_len {
                let _ = self.segment_write_history.remove(0);
            }
            self.last_segment_bytes = segment_bytes_written;

            // WAL write rate (KB/s) - based on cumulative bytes from engine
            // This is accurate even across WAL rotations since the counter never decreases
            let wal_bytes_this_period = wal_bytes_written.saturating_sub(self.last_wal_bytes);
            let wal_rate_kbs =
                (wal_bytes_this_period as f64 / iteration_elapsed.as_secs_f64() / 1024.0) as u64;
            self.wal_write_history.push(wal_rate_kbs);
            if self.wal_write_history.len() > self.max_history_len {
                let _ = self.wal_write_history.remove(0);
            }
            self.last_wal_bytes = wal_bytes_written;

            // Read and Write IOPS from OS-level syscall counters
            let (current_syscr, current_syscw) = read_process_syscalls();
            let read_ops_delta = current_syscr.saturating_sub(self.last_syscr);
            let write_ops_delta = current_syscw.saturating_sub(self.last_syscw);
            let read_iops = (read_ops_delta as f64 / iteration_elapsed.as_secs_f64()) as u64;
            let write_iops = (write_ops_delta as f64 / iteration_elapsed.as_secs_f64()) as u64;
            self.read_iops_history.push(read_iops);
            self.write_iops_history.push(write_iops);
            if self.read_iops_history.len() > self.max_history_len {
                let _ = self.read_iops_history.remove(0);
            }
            if self.write_iops_history.len() > self.max_history_len {
                let _ = self.write_iops_history.remove(0);
            }
            self.last_syscr = current_syscr;
            self.last_syscw = current_syscw;

            // Page faults from /proc/self/stat
            let (current_minflt, current_majflt) = read_process_page_faults();

            // Minor page faults (soft) - pages resolved from memory/cache
            let minflt_delta = current_minflt.saturating_sub(self.last_minflt);
            let minflt_rate = (minflt_delta as f64 / iteration_elapsed.as_secs_f64()) as u64;
            self.minflt_history.push(minflt_rate);
            if self.minflt_history.len() > self.max_history_len {
                let _ = self.minflt_history.remove(0);
            }
            self.last_minflt = current_minflt;

            // Major page faults (hard) - pages read from disk
            let majflt_delta = current_majflt.saturating_sub(self.last_majflt);
            let majflt_rate = (majflt_delta as f64 / iteration_elapsed.as_secs_f64()) as u64;
            self.majflt_history.push(majflt_rate);
            if self.majflt_history.len() > self.max_history_len {
                let _ = self.majflt_history.remove(0);
            }
            self.last_majflt = current_majflt;

            // Buffered history
            self.buffered_history.push(stats.buffered);
            if self.buffered_history.len() > self.max_history_len {
                let _ = self.buffered_history.remove(0);
            }

            self.last_iteration_time = now;
            self.last_iteration_consumed = stats.total_consumed;
        }

        let progress = elapsed.as_secs_f64() / self.target_duration.as_secs_f64();
        let progress_pct = (progress * 100.0).min(100.0);

        // Draw and capture actual frame width for resize handling
        let _ = self.terminal.draw(|frame| {
            render_steady_state_dashboard(
                frame,
                stats,
                elapsed,
                self.target_duration,
                progress_pct,
                &self.consumed_throughput_history,
                &self.memory_history,
                &self.rss_history,
                &self.cpu_history,
                &self.disk_history,
                &self.segment_write_history,
                &self.wal_write_history,
                &self.read_iops_history,
                &self.write_iops_history,
                &self.minflt_history,
                &self.majflt_history,
                &self.buffered_history,
                config,
            );
        });

        Ok(())
    }
}

impl Drop for Dashboard {
    fn drop(&mut self) {
        self.restore_terminal();
    }
}

/// Configuration for steady-state mode dashboard.
#[derive(Clone)]
pub struct SteadyStateConfig {
    pub subscribers: usize,
    pub bundles_per_batch: usize,
    pub rows_per_bundle: usize,
    pub subscriber_delay_ms: u64,
    pub data_dir: String,
}

fn render_throughput_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let max_val = history.iter().max().copied().unwrap_or(1);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(
                    " Consumed Throughput (bundles/sec) │ max: {} ",
                    max_val
                ))
                .borders(Borders::ALL),
        )
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::Cyan))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn render_memory_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64], current: f64) {
    let max_val = history.iter().max().copied().unwrap_or(1);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" Heap (MB) │ current: {:.1} ", current))
                .borders(Borders::ALL),
        )
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::Magenta))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn render_rss_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let current = history.last().copied().unwrap_or(0);
    let max_val = history.iter().max().copied().unwrap_or(1);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" RSS (MB) │ current: {} ", current))
                .borders(Borders::ALL),
        )
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::LightMagenta))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn render_cpu_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    // History stores CPU % * 10, so divide for display
    let current = history.last().copied().unwrap_or(0) as f64 / 10.0;
    let max_val = history.iter().max().copied().unwrap_or(1).max(1000); // Cap at 100% (1000 = 100.0 * 10)

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" CPU │ current: {:.1}% ", current))
                .borders(Borders::ALL),
        )
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::Yellow))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn render_disk_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let current = history.last().copied().unwrap_or(0);
    let max_val = history.iter().max().copied().unwrap_or(1).max(1);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" Disk │ current: {} MB ", current))
                .borders(Borders::ALL),
        )
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn render_segment_write_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let current = history.last().copied().unwrap_or(0);
    let max_val = history.iter().max().copied().unwrap_or(1).max(1);

    // Display in MB/s if >= 1024 KB/s, otherwise KB/s
    let title = if current >= 1024 {
        format!(
            " Segment Writes (.qseg) │ current: {:.1} MB/s ",
            current as f64 / 1024.0
        )
    } else {
        format!(" Segment Writes (.qseg) │ current: {} KB/s ", current)
    };

    let sparkline = Sparkline::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::Cyan))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn render_read_iops_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let current = history.last().copied().unwrap_or(0);
    let max_val = history.iter().max().copied().unwrap_or(1).max(1);

    let title = format!(" Read IOPS │ current: {} ops/s ", current);

    let sparkline = Sparkline::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::LightCyan))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn render_write_iops_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let current = history.last().copied().unwrap_or(0);
    let max_val = history.iter().max().copied().unwrap_or(1).max(1);

    let title = format!(" Write IOPS │ current: {} ops/s ", current);

    let sparkline = Sparkline::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::Cyan))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

/// Render minor (soft) page faults - pages resolved from memory/cache, not disk.
fn render_minor_faults_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let current = history.last().copied().unwrap_or(0);
    let max_val = history.iter().max().copied().unwrap_or(1).max(1);

    // Display in K if >= 1000
    let title = if current >= 1000 {
        format!(
            " Minor Faults (soft) │ current: {:.1}K/s ",
            current as f64 / 1000.0
        )
    } else {
        format!(" Minor Faults (soft) │ current: {}/s ", current)
    };

    let sparkline = Sparkline::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::Magenta))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

/// Render major (hard) page faults - pages that had to be read from disk.
fn render_major_faults_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let current = history.last().copied().unwrap_or(0);
    let max_val = history.iter().max().copied().unwrap_or(1).max(1);

    // Display in K if >= 1000
    let title = if current >= 1000 {
        format!(
            " Major Faults (hard) │ current: {:.1}K/s ",
            current as f64 / 1000.0
        )
    } else {
        format!(" Major Faults (hard) │ current: {}/s ", current)
    };

    let sparkline = Sparkline::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::Red))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn render_wal_write_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let current = history.last().copied().unwrap_or(0);
    let max_val = history.iter().max().copied().unwrap_or(1).max(1);

    // Display in MB/s if >= 1024 KB/s, otherwise KB/s
    let title = if current >= 1024 {
        format!(
            " WAL Writes (quiver.wal*) │ current: {:.1} MB/s ",
            current as f64 / 1024.0
        )
    } else {
        format!(" WAL Writes (quiver.wal*) │ current: {} KB/s ", current)
    };

    let sparkline = Sparkline::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::LightBlue))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn render_buffered_sparkline(frame: &mut Frame<'_>, area: Rect, history: &[u64]) {
    let current = history.last().copied().unwrap_or(0);
    let max_val = history.iter().max().copied().unwrap_or(1).max(1);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" Buffered │ current: {} bundles ", current))
                .borders(Borders::ALL),
        )
        .data(history)
        .max(max_val)
        .style(Style::default().fg(Color::LightRed))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

// =============================================================================
// Steady-State Dashboard Rendering
// =============================================================================

fn render_steady_state_dashboard(
    frame: &mut Frame<'_>,
    stats: &SteadyStateStats,
    elapsed: Duration,
    target_duration: Duration,
    progress_pct: f64,
    consumed_throughput_history: &[u64],
    memory_history: &[u64],
    rss_history: &[u64],
    cpu_history: &[u64],
    disk_history: &[u64],
    segment_write_history: &[u64],
    wal_write_history: &[u64],
    read_iops_history: &[u64],
    write_iops_history: &[u64],
    minflt_history: &[u64],
    majflt_history: &[u64],
    buffered_history: &[u64],
    config: &SteadyStateConfig,
) {
    // Main vertical layout: header, stats, sparklines (two columns), config
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title + progress
            Constraint::Length(10), // Stats panels (extra lines for backpressure + dropped)
            Constraint::Min(24),    // Sparklines area (two columns) - increased for 6 sparklines
            Constraint::Length(3),  // Status/config
        ])
        .split(frame.area());

    // Title and progress bar
    render_steady_state_header(
        frame,
        main_chunks[0],
        elapsed,
        target_duration,
        progress_pct,
    );

    // Stats panels
    render_steady_state_stats(frame, main_chunks[1], stats, elapsed);

    // Split sparklines area into two columns
    let sparkline_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left column: System metrics
            Constraint::Percentage(50), // Right column: I/O metrics
        ])
        .split(main_chunks[2]);

    // Left column: System/resource metrics (6 sparklines)
    let left_sparklines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Throughput
            Constraint::Length(4), // Memory (heap)
            Constraint::Length(4), // RSS (total)
            Constraint::Length(4), // CPU
            Constraint::Length(4), // Disk
            Constraint::Length(4), // Backlog
            Constraint::Min(0),    // Spacer
        ])
        .split(sparkline_columns[0]);

    // Right column: I/O metrics (6 sparklines)
    let right_sparklines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Segment writes
            Constraint::Length(4), // WAL writes
            Constraint::Length(4), // Read IOPS
            Constraint::Length(4), // Write IOPS
            Constraint::Length(4), // Minor page faults
            Constraint::Length(4), // Major page faults
            Constraint::Min(0),    // Spacer
        ])
        .split(sparkline_columns[1]);

    // Left column sparklines
    render_throughput_sparkline(frame, left_sparklines[0], consumed_throughput_history);
    render_memory_sparkline(
        frame,
        left_sparklines[1],
        memory_history,
        stats.current_memory_mb,
    );
    render_rss_sparkline(frame, left_sparklines[2], rss_history);
    render_cpu_sparkline(frame, left_sparklines[3], cpu_history);
    render_disk_sparkline(frame, left_sparklines[4], disk_history);
    render_buffered_sparkline(frame, left_sparklines[5], buffered_history);

    // Right column sparklines
    render_segment_write_sparkline(frame, right_sparklines[0], segment_write_history);
    render_wal_write_sparkline(frame, right_sparklines[1], wal_write_history);
    render_read_iops_sparkline(frame, right_sparklines[2], read_iops_history);
    render_write_iops_sparkline(frame, right_sparklines[3], write_iops_history);
    render_minor_faults_sparkline(frame, right_sparklines[4], minflt_history);
    render_major_faults_sparkline(frame, right_sparklines[5], majflt_history);

    // Configuration/status
    render_steady_state_config(frame, main_chunks[3], config);
}

fn render_steady_state_header(
    frame: &mut Frame<'_>,
    area: Rect,
    elapsed: Duration,
    target: Duration,
    progress_pct: f64,
) {
    let hours = elapsed.as_secs() / 3600;
    let minutes = (elapsed.as_secs() % 3600) / 60;
    let seconds = elapsed.as_secs() % 60;

    let remaining = target.saturating_sub(elapsed);
    let rem_hours = remaining.as_secs() / 3600;
    let rem_mins = (remaining.as_secs() % 3600) / 60;
    let rem_secs = remaining.as_secs() % 60;

    let title = format!(
        " Quiver Steady-State Test │ Elapsed: {:02}:{:02}:{:02} │ Remaining: {:02}:{:02}:{:02} │ Press 'q' to quit ",
        hours, minutes, seconds, rem_hours, rem_mins, rem_secs
    );

    let gauge = Gauge::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(progress_pct as u16)
        .label(format!("{:.1}%", progress_pct));

    frame.render_widget(gauge, area);
}

fn render_steady_state_stats(
    frame: &mut Frame<'_>,
    area: Rect,
    stats: &SteadyStateStats,
    _elapsed: Duration,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(area);

    // Bundles panel
    let ingest_rate = stats.live_bundles_ingest_rate();
    let consume_rate = stats.live_bundles_consume_rate();

    let bundles_text = vec![
        Line::from(vec![
            Span::styled("Ingested: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.total_ingested),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Consumed: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.total_consumed),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::from(vec![
            Span::styled("Buffered: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.buffered),
                Style::default().fg(if stats.buffered > 1000 {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
        ]),
    ];

    let bundles_block = Paragraph::new(bundles_text)
        .block(Block::default().title(" Bundles ").borders(Borders::ALL));
    frame.render_widget(bundles_block, chunks[0]);

    // Segments panel
    let backpressure_color = if stats.backpressure_count > 100 {
        Color::Red
    } else if stats.backpressure_count > 0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let force_dropped_color = if stats.force_dropped_segments > 0 {
        Color::Red
    } else {
        Color::Green
    };

    let segments_text = vec![
        Line::from(vec![
            Span::styled("Written:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.total_segments_written),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("Cleaned:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.total_cleaned),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Active:   ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.active_segments),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Backpres: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.backpressure_count),
                Style::default().fg(backpressure_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Dropped:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.force_dropped_segments),
                Style::default().fg(force_dropped_color),
            ),
        ]),
    ];

    let segments_block = Paragraph::new(segments_text)
        .block(Block::default().title(" Segments ").borders(Borders::ALL));
    frame.render_widget(segments_block, chunks[1]);

    // Rates panel
    let rates_text = vec![
        Line::from(vec![
            Span::styled("Ingest:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0}/s", ingest_rate),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("Consume: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0}/s", consume_rate),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::from(vec![
            Span::styled("Rows:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0}/s", stats.live_consume_rows_rate()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Data:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} MB/s", stats.live_consume_mb_rate()),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let rates_block =
        Paragraph::new(rates_text).block(Block::default().title(" Rates ").borders(Borders::ALL));
    frame.render_widget(rates_block, chunks[2]);

    // Memory stats
    let memory_growth = stats.memory_growth_mb();
    let memory_color = if memory_growth > 100.0 {
        Color::Red
    } else if memory_growth > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let memory_text = vec![
        Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} MB", stats.current_memory_mb),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Initial: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} MB", stats.initial_memory_mb),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::styled("Growth:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:+.1} MB", memory_growth),
                Style::default().fg(memory_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Peak:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} MB", stats.peak_memory_mb),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let memory_block =
        Paragraph::new(memory_text).block(Block::default().title(" Memory ").borders(Borders::ALL));
    frame.render_widget(memory_block, chunks[3]);

    // Disk stats
    let disk_mb = stats.current_disk_bytes as f64 / 1024.0 / 1024.0;
    let initial_disk_mb = stats.initial_disk_bytes as f64 / 1024.0 / 1024.0;
    let peak_disk_mb = stats.peak_disk_bytes as f64 / 1024.0 / 1024.0;
    let disk_growth = disk_mb - initial_disk_mb;

    let disk_text = vec![
        Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} MB", disk_mb),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Initial: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} MB", initial_disk_mb),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::styled("Growth:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:+.1} MB", disk_growth),
                Style::default().fg(if disk_growth > 100.0 {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Peak:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} MB", peak_disk_mb),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let disk_block =
        Paragraph::new(disk_text).block(Block::default().title(" Disk ").borders(Borders::ALL));
    frame.render_widget(disk_block, chunks[4]);
}

fn render_steady_state_config(frame: &mut Frame<'_>, area: Rect, config: &SteadyStateConfig) {
    let delay_str = if config.subscriber_delay_ms > 0 {
        format!("{}ms", config.subscriber_delay_ms)
    } else {
        "none".to_string()
    };

    let config_lines = vec![
        Line::from(format!("Data dir: {}", config.data_dir)),
        Line::from(format!(
            "Mode: Steady-State │ Bundles/batch: {} │ Rows/bundle: {} │ Subscribers: {} │ Delay: {}",
            config.bundles_per_batch, config.rows_per_bundle, config.subscribers, delay_str,
        )),
    ];

    let config_block = Paragraph::new(config_lines)
        .block(
            Block::default()
                .title(" Configuration ")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(config_block, area);
}
