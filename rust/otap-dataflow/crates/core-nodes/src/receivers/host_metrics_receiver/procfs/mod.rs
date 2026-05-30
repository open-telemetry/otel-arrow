// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux procfs-backed host metric source.

mod paths;
mod projection;
mod readings;

use crate::receivers::host_metrics_receiver::{
    CompiledFilter, HostViewValidationMode, ProcessLabelsConfig, ProcessMetricsConfig,
};
use paths::{PathKind, ProcfsPaths};
use projection::{CounterTracker, host_arch};
pub(crate) use projection::{HostResource, HostScrape, HostSnapshot};
use readings::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

const NANOS_PER_SEC: u64 = 1_000_000_000;
const BYTES_PER_KIB: u64 = 1024;
const DISKSTAT_SECTOR_BYTES: u64 = 512;
const FILESYSTEM_STAT_TIMEOUT: Duration = Duration::from_millis(100);
const FILESYSTEM_SCRAPE_TIMEOUT: Duration = Duration::from_secs(1);
const COUNTER_KEY_SEPARATOR: char = '\x1f';

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(super) struct ProcessKey {
    pub(super) pid: u32,
    pub(super) start_time_unix_nano: u64,
}

#[derive(Clone, Copy)]
struct ProcessCpuSample {
    total_cpu_seconds: f64,
    observed_unix_nano: u64,
}

struct ProcessCommand {
    command: String,
    executable_name: String,
}

impl ProcessCommand {
    fn from_comm(comm: &str) -> Self {
        Self {
            command: comm.to_owned(),
            executable_name: comm.to_owned(),
        }
    }
}

/// Procfs-backed source for host metrics.
pub struct ProcfsSource {
    paths: ProcfsPaths,
    config: ProcfsConfig,
    buf: String,
    process_text_buf: String,
    process_bytes_buf: Vec<u8>,
    process_file_path: PathBuf,
    clk_tck: f64,
    process_cpu_count: f64,
    previous_cpu: Option<CpuTimes>,
    filesystem_worker: FilesystemStatWorker,
    counter_tracker: CounterTracker,
    previous_process_cpu: HashMap<ProcessKey, ProcessCpuSample>,
    boot_time_unix_nano: Option<u64>,
    fallback_start_time_unix_nano: u64,
    resource: Option<HostResource>,
}

/// Procfs collection config.
#[derive(Clone)]
pub struct ProcfsConfig {
    /// CPU metrics.
    pub cpu: bool,
    /// Memory metrics.
    pub memory: bool,
    /// Paging metrics.
    pub paging: bool,
    /// System metrics.
    pub system: bool,
    /// Disk metrics.
    pub disk: bool,
    /// Filesystem metrics.
    pub filesystem: bool,
    /// Network metrics.
    pub network: bool,
    /// Process summary metrics.
    pub processes: bool,
    /// Linux load average metrics.
    pub load: bool,
    /// Per-process metrics.
    pub per_processes: bool,
    /// Derived aggregate CPU utilization.
    pub cpu_utilization: bool,
    /// Emit memory limit metric.
    pub memory_limit: bool,
    /// Emit Linux shared memory metric.
    pub memory_shared: bool,
    /// Emit Linux hugepage metrics.
    pub memory_hugepages: bool,
    /// Derived disk limit from sysfs block device size.
    pub disk_limit: bool,
    /// Include virtual filesystems.
    pub filesystem_include_virtual: bool,
    /// Include remote and userspace filesystems.
    pub filesystem_include_remote: bool,
    /// Emit filesystem limit metric.
    pub filesystem_limit: bool,
    /// Disk include filter.
    pub disk_include: Option<CompiledFilter>,
    /// Disk exclude filter.
    pub disk_exclude: Option<CompiledFilter>,
    /// Filesystem device include filter.
    pub filesystem_include_devices: Option<CompiledFilter>,
    /// Filesystem device exclude filter.
    pub filesystem_exclude_devices: Option<CompiledFilter>,
    /// Filesystem type include filter.
    pub filesystem_include_fs_types: Option<CompiledFilter>,
    /// Filesystem type exclude filter.
    pub filesystem_exclude_fs_types: Option<CompiledFilter>,
    /// Filesystem mount point include filter.
    pub filesystem_include_mount_points: Option<CompiledFilter>,
    /// Filesystem mount point exclude filter.
    pub filesystem_exclude_mount_points: Option<CompiledFilter>,
    /// Network include filter.
    pub network_include: Option<CompiledFilter>,
    /// Network exclude filter.
    pub network_exclude: Option<CompiledFilter>,
    /// Process include filter.
    pub process_include: Option<CompiledFilter>,
    /// Process exclude filter.
    pub process_exclude: Option<CompiledFilter>,
    /// Per-process cardinality limit.
    pub process_max_processes: usize,
    /// Per-process label controls.
    pub process_labels: ProcessLabelsConfig,
    /// Per-process metric controls.
    pub process_metrics: ProcessMetricsConfig,
    /// Startup validation mode.
    pub validation: HostViewValidationMode,
}

/// Dedicated worker for procfs/sysfs host inspection.
///
/// The pipeline task owns scheduling, control handling, projection, encoding,
/// and downstream send. This worker owns `ProcfsSource` and performs the
/// synchronous host reads so the current-thread runtime is not blocked by host
/// filesystem traversal.
pub struct ProcfsScrapeWorker {
    tx: mpsc::SyncSender<ScrapeWorkerRequest>,
    accepted: Arc<AtomicBool>,
}

/// Error returned when a scrape cannot be started.
#[derive(Debug)]
pub enum StartScrapeError {
    /// A previous scrape is still running.
    Busy,
    /// The worker has stopped.
    Stopped,
}

struct ScrapeWorkerRequest {
    due: ProcfsFamilies,
    response: oneshot::Sender<io::Result<HostScrape>>,
}

struct AcceptedScrapeGuard {
    accepted: Arc<AtomicBool>,
}

impl AcceptedScrapeGuard {
    fn new(accepted: Arc<AtomicBool>) -> Self {
        Self { accepted }
    }
}

impl Drop for AcceptedScrapeGuard {
    fn drop(&mut self) {
        self.accepted.store(false, Ordering::Release);
    }
}

impl ProcfsScrapeWorker {
    /// Starts a worker rooted at `/` or at a host root bind mount.
    pub fn new(root_path: Option<PathBuf>, config: ProcfsConfig) -> io::Result<Self> {
        let mut source = ProcfsSource::new(root_path.as_deref(), config)?;
        let accepted = Arc::new(AtomicBool::new(false));
        let worker_accepted = Arc::clone(&accepted);
        let (tx, rx) = mpsc::sync_channel::<ScrapeWorkerRequest>(1);
        let _handle = std::thread::Builder::new()
            .name("host-metrics-scraper".to_owned())
            .spawn(move || {
                while let Ok(request) = rx.recv() {
                    let _accepted = AcceptedScrapeGuard::new(Arc::clone(&worker_accepted));
                    let result = source.scrape_due_blocking(request.due);
                    let _ = request.response.send(result);
                }
            })?;
        Ok(Self { tx, accepted })
    }

    /// Starts one scrape if the worker is idle.
    pub fn try_start_scrape(
        &self,
        due: ProcfsFamilies,
    ) -> Result<oneshot::Receiver<io::Result<HostScrape>>, StartScrapeError> {
        if self
            .accepted
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(StartScrapeError::Busy);
        }
        let (response, rx) = oneshot::channel();
        self.tx
            .try_send(ScrapeWorkerRequest { due, response })
            .map_err(|err| match err {
                mpsc::TrySendError::Full(_) => StartScrapeError::Busy,
                mpsc::TrySendError::Disconnected(_) => {
                    self.accepted.store(false, Ordering::Release);
                    StartScrapeError::Stopped
                }
            })?;
        Ok(rx)
    }
}

/// Families due for one scrape.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProcfsFamilies {
    /// CPU metrics.
    pub cpu: bool,
    /// Memory metrics.
    pub memory: bool,
    /// Paging metrics.
    pub paging: bool,
    /// System metrics.
    pub system: bool,
    /// Disk metrics.
    pub disk: bool,
    /// Filesystem metrics.
    pub filesystem: bool,
    /// Network metrics.
    pub network: bool,
    /// Process summary metrics.
    pub processes: bool,
    /// Linux load average metrics.
    pub load: bool,
    /// Per-process metrics.
    pub per_processes: bool,
}

impl ProcfsFamilies {
    fn enabled_by(self, config: &ProcfsConfig) -> Self {
        Self {
            cpu: self.cpu && config.cpu,
            memory: self.memory && config.memory,
            paging: self.paging && config.paging,
            system: self.system && config.system,
            disk: self.disk && config.disk,
            filesystem: self.filesystem && config.filesystem,
            network: self.network && config.network,
            processes: self.processes && config.processes,
            load: self.load && config.load,
            per_processes: self.per_processes && config.per_processes,
        }
    }
}

impl ProcfsSource {
    /// Creates a procfs source rooted at `/` or at a host root bind mount.
    pub fn new(root_path: Option<&Path>, config: ProcfsConfig) -> io::Result<Self> {
        let mut source = Self {
            paths: ProcfsPaths::new(root_path),
            config,
            buf: String::with_capacity(16 * 1024),
            process_text_buf: String::with_capacity(1024),
            process_bytes_buf: Vec::with_capacity(512),
            process_file_path: PathBuf::new(),
            clk_tck: clock_ticks_per_second(),
            process_cpu_count: available_process_cpu_count(),
            previous_cpu: None,
            filesystem_worker: FilesystemStatWorker::new()?,
            counter_tracker: CounterTracker::default(),
            previous_process_cpu: HashMap::new(),
            boot_time_unix_nano: None,
            fallback_start_time_unix_nano: now_unix_nano(),
            resource: None,
        };
        source.apply_startup_validation()?;
        Ok(source)
    }

    /// Collects one host snapshot for the due family set on a blocking worker.
    pub fn scrape_due_blocking(&mut self, due: ProcfsFamilies) -> io::Result<HostScrape> {
        let due = due.enabled_by(&self.config);
        let now_unix_nano = now_unix_nano();
        let clk_tck = self.clk_tck;
        let mut partial_errors = 0;
        let mut first_error = None;
        let needs_start_time = due.cpu
            || due.memory
            || due.paging
            || due.disk
            || due.filesystem
            || due.network
            || due.processes
            || due.per_processes;
        let needs_stat = due.cpu
            || due.processes
            || due.per_processes
            || (needs_start_time && self.boot_time_unix_nano.is_none());
        let stat = match needs_stat
            .then(|| self.read_path(PathKind::Stat))
            .transpose()
        {
            Ok(Some(proc_stat)) => parse_stat(proc_stat, clk_tck),
            Ok(None) => StatSnapshot::default(),
            Err(err) => {
                record_partial_error(&mut partial_errors, &mut first_error, err);
                StatSnapshot::default()
            }
        };
        if stat.boot_time_unix_nano != 0 {
            self.boot_time_unix_nano = Some(stat.boot_time_unix_nano);
        }
        let start_time_unix_nano = self
            .boot_time_unix_nano
            .unwrap_or(self.fallback_start_time_unix_nano);
        let cpu_utilization = if due.cpu && self.config.cpu_utilization {
            let utilization = stat.cpu.and_then(|current| {
                self.previous_cpu
                    .and_then(|previous| cpu_utilization(previous, current))
            });
            self.previous_cpu = stat.cpu;
            utilization
        } else {
            None
        };

        let cpuinfo = match due
            .cpu
            .then(|| self.read_path(PathKind::Cpuinfo))
            .transpose()
        {
            Ok(Some(cpuinfo)) => parse_cpuinfo(cpuinfo),
            Ok(None) => CpuInfo::default(),
            Err(err) => {
                record_partial_error(&mut partial_errors, &mut first_error, err);
                CpuInfo::default()
            }
        };

        let load = match due
            .load
            .then(|| self.read_path(PathKind::Loadavg))
            .transpose()
        {
            Ok(Some(loadavg)) => parse_loadavg(loadavg),
            Ok(None) => None,
            Err(err) => {
                record_partial_error(&mut partial_errors, &mut first_error, err);
                None
            }
        };

        let memory = match due
            .memory
            .then(|| self.read_path(PathKind::Meminfo))
            .transpose()
        {
            Ok(Some(meminfo)) => parse_meminfo(meminfo),
            Ok(None) => None,
            Err(err) => {
                record_partial_error(&mut partial_errors, &mut first_error, err);
                None
            }
        };

        let uptime_seconds = match due
            .system
            .then(|| self.read_path(PathKind::Uptime))
            .transpose()
        {
            Ok(Some(uptime)) => parse_uptime(uptime),
            Ok(None) => None,
            Err(err) => {
                record_partial_error(&mut partial_errors, &mut first_error, err);
                None
            }
        };

        let paging = match due
            .paging
            .then(|| self.read_path(PathKind::Vmstat))
            .transpose()
        {
            Ok(Some(vmstat)) => Some(parse_vmstat(vmstat)),
            Ok(None) => None,
            Err(err) => {
                record_partial_error(&mut partial_errors, &mut first_error, err);
                None
            }
        };

        let swaps = match due
            .paging
            .then(|| self.read_path(PathKind::Swaps))
            .transpose()
        {
            Ok(Some(swaps)) => parse_swaps(swaps),
            Ok(None) => Vec::new(),
            Err(err) => {
                record_partial_error(&mut partial_errors, &mut first_error, err);
                Vec::new()
            }
        };

        let disks = if due.disk {
            let disk_include = self.config.disk_include.clone();
            let disk_exclude = self.config.disk_exclude.clone();
            match self.read_path(PathKind::Diskstats) {
                Ok(diskstats) => {
                    let mut disks =
                        parse_diskstats(diskstats, disk_include.as_ref(), disk_exclude.as_ref());
                    if self.config.disk_limit {
                        for disk in &mut disks {
                            disk.limit_bytes = self.read_disk_limit_bytes(&disk.name).ok();
                        }
                    }
                    Some(disks)
                }
                Err(err) => {
                    record_partial_error(&mut partial_errors, &mut first_error, err);
                    None
                }
            }
        } else {
            None
        };

        let networks = if due.network {
            let network_include = self.config.network_include.clone();
            let network_exclude = self.config.network_exclude.clone();
            match self.read_path(PathKind::NetDev) {
                Ok(netdev) => Some(parse_netdev(
                    netdev,
                    network_include.as_ref(),
                    network_exclude.as_ref(),
                )),
                Err(err) => {
                    record_partial_error(&mut partial_errors, &mut first_error, err);
                    None
                }
            }
        } else {
            None
        };

        let filesystems = if due.filesystem {
            let include_virtual = self.config.filesystem_include_virtual;
            let include_remote = self.config.filesystem_include_remote;
            let emit_limit = self.config.filesystem_limit;
            let include_devices = self.config.filesystem_include_devices.clone();
            let exclude_devices = self.config.filesystem_exclude_devices.clone();
            let include_fs_types = self.config.filesystem_include_fs_types.clone();
            let exclude_fs_types = self.config.filesystem_exclude_fs_types.clone();
            let include_mount_points = self.config.filesystem_include_mount_points.clone();
            let exclude_mount_points = self.config.filesystem_exclude_mount_points.clone();
            match self.read_path(PathKind::Mountinfo) {
                Ok(mountinfo) => {
                    let filters = FilesystemFilters {
                        include_devices: include_devices.as_ref(),
                        exclude_devices: exclude_devices.as_ref(),
                        include_fs_types: include_fs_types.as_ref(),
                        exclude_fs_types: exclude_fs_types.as_ref(),
                        include_mount_points: include_mount_points.as_ref(),
                        exclude_mount_points: exclude_mount_points.as_ref(),
                    };
                    let mounts = parse_mountinfo(
                        mountinfo,
                        include_virtual,
                        include_remote,
                        emit_limit,
                        filters,
                    );
                    self.read_filesystems(mounts, &mut partial_errors, &mut first_error)
                }
                Err(err) => {
                    record_partial_error(&mut partial_errors, &mut first_error, err);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        let per_processes = if due.per_processes {
            self.read_processes(
                start_time_unix_nano,
                now_unix_nano,
                &mut partial_errors,
                &mut first_error,
            )
        } else {
            Vec::new()
        };

        let resource = self.read_resource().clone();
        let counter_starts = self.counter_tracker.snapshot(
            start_time_unix_nano,
            now_unix_nano,
            due.cpu.then_some(stat.cpu).flatten().as_ref(),
            paging.as_ref(),
            due.processes.then_some(stat.processes).as_ref(),
            due.per_processes.then_some(per_processes.as_slice()),
            disks.as_deref(),
            networks.as_deref(),
        );

        let snapshot = HostSnapshot {
            now_unix_nano,
            start_time_unix_nano,
            counter_starts,
            memory_limit: self.config.memory_limit,
            memory_shared: self.config.memory_shared,
            memory_hugepages: self.config.memory_hugepages,
            process_metrics: self.config.process_metrics,
            cpu: due.cpu.then_some(stat.cpu).flatten(),
            cpu_utilization,
            cpuinfo,
            load,
            memory,
            uptime_seconds,
            paging,
            swaps,
            processes: due.processes.then_some(stat.processes),
            per_processes,
            disks: disks.unwrap_or_default(),
            filesystems,
            networks: networks.unwrap_or_default(),
            resource,
        };
        if !snapshot.has_metrics() {
            return Err(first_error
                .unwrap_or_else(|| io::Error::other("host metrics scrape produced no metrics")));
        }
        Ok(HostScrape {
            snapshot,
            partial_errors,
        })
    }

    fn apply_startup_validation(&mut self) -> io::Result<()> {
        match self.config.validation {
            HostViewValidationMode::None => Ok(()),
            HostViewValidationMode::FailSelected => self.validate_selected_paths(),
            HostViewValidationMode::WarnSelected => {
                self.disable_unavailable_sources();
                Ok(())
            }
        }
    }

    fn read_processes(
        &mut self,
        host_start_time_unix_nano: u64,
        now_unix_nano: u64,
        partial_errors: &mut u64,
        first_error: &mut Option<io::Error>,
    ) -> Vec<ProcessMetrics> {
        let mut processes = Vec::new();
        let proc_dir = self.paths.proc_path();
        let entries = match fs::read_dir(proc_dir) {
            Ok(entries) => entries,
            Err(err) => {
                record_partial_error(partial_errors, first_error, err);
                return processes;
            }
        };
        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<u32>().ok())
            else {
                continue;
            };
            match self.read_process(
                entry.path(),
                pid,
                host_start_time_unix_nano,
                now_unix_nano,
                partial_errors,
                first_error,
            ) {
                Ok(Some(process)) => processes.push(process),
                Ok(None) => {}
                Err(err) => record_partial_error(partial_errors, first_error, err),
            }
        }
        processes.sort_by(|a, b| {
            b.total_cpu_seconds()
                .total_cmp(&a.total_cpu_seconds())
                .then_with(|| b.memory_usage_bytes.cmp(&a.memory_usage_bytes))
                .then_with(|| a.key.pid.cmp(&b.key.pid))
        });
        processes.truncate(self.config.process_max_processes);
        self.previous_process_cpu
            .retain(|key, _| processes.iter().any(|process| process.key == *key));
        processes
    }

    fn read_process(
        &mut self,
        process_dir: PathBuf,
        pid: u32,
        host_start_time_unix_nano: u64,
        now_unix_nano: u64,
        partial_errors: &mut u64,
        first_error: &mut Option<io::Error>,
    ) -> io::Result<Option<ProcessMetrics>> {
        let clk_tck = self.clk_tck;
        let stat = match self.read_process_file_text(&process_dir, "stat") {
            Ok(stat) => parse_process_stat(stat, pid, clk_tck),
            Err(err) if Self::is_expected_process_read_error(&err) => return Ok(None),
            Err(err) => return Err(err),
        };
        let Some(stat) = stat else {
            return Ok(None);
        };
        let process_command = self
            .read_process_command(&process_dir)
            .unwrap_or_else(|| ProcessCommand::from_comm(&stat.command));
        if !process_filter_allows(
            &process_command,
            self.config.process_include.as_ref(),
            self.config.process_exclude.as_ref(),
        ) {
            return Ok(None);
        }
        let start_offset_nano =
            (stat.start_time_ticks as f64 / self.clk_tck * NANOS_PER_SEC as f64) as u64;
        let start_time_unix_nano = host_start_time_unix_nano.saturating_add(start_offset_nano);
        let key = ProcessKey {
            pid: stat.pid,
            start_time_unix_nano,
        };
        let total_cpu_seconds = stat.user_cpu_seconds + stat.system_cpu_seconds;
        let previous = self.previous_process_cpu.insert(
            key,
            ProcessCpuSample {
                total_cpu_seconds,
                observed_unix_nano: now_unix_nano,
            },
        );
        let cpu_utilization = previous.and_then(|previous| {
            let cpu_delta = counter_delta(previous.total_cpu_seconds, total_cpu_seconds)?;
            let elapsed = now_unix_nano.saturating_sub(previous.observed_unix_nano) as f64
                / NANOS_PER_SEC as f64;
            (elapsed > 0.0).then_some(cpu_delta / elapsed / self.process_cpu_count)
        });
        let io = if self.config.process_metrics.disk_io {
            match self.read_process_file_text(&process_dir, "io") {
                Ok(content) => Some(parse_process_io(content)),
                Err(err) if Self::is_expected_process_read_error(&err) => None,
                Err(err) => {
                    record_partial_error(partial_errors, first_error, err);
                    None
                }
            }
        } else {
            None
        };
        Ok(Some(ProcessMetrics {
            key,
            labels: self.config.process_labels,
            command: process_command.command,
            executable_name: process_command.executable_name,
            parent_pid: stat.parent_pid,
            user_cpu_seconds: stat.user_cpu_seconds,
            system_cpu_seconds: stat.system_cpu_seconds,
            cpu_utilization,
            memory_usage_bytes: stat.resident_pages.saturating_mul(page_size_bytes()),
            memory_virtual_bytes: stat.virtual_memory_bytes,
            read_bytes: io.as_ref().map(|io| io.read_bytes),
            write_bytes: io.as_ref().map(|io| io.write_bytes),
            threads: stat.threads,
            uptime_seconds: now_unix_nano.saturating_sub(start_time_unix_nano) as f64
                / NANOS_PER_SEC as f64,
        }))
    }

    fn read_process_command(&mut self, process_dir: &Path) -> Option<ProcessCommand> {
        self.read_process_file_bytes(process_dir, "cmdline").ok()?;
        let argv0 = self
            .process_bytes_buf
            .split(|byte| *byte == 0)
            .find(|arg| !arg.is_empty())?;
        let command = String::from_utf8_lossy(argv0).into_owned();
        let executable_path = command.trim_end_matches('/');
        let executable_source = if executable_path.is_empty() {
            command.as_str()
        } else {
            executable_path
        };
        let executable_name = Path::new(executable_source)
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or(&command)
            .to_owned();
        Some(ProcessCommand {
            command,
            executable_name,
        })
    }

    fn read_process_file_text(&mut self, process_dir: &Path, file_name: &str) -> io::Result<&str> {
        self.process_text_buf.clear();
        self.process_file_path.clear();
        self.process_file_path.push(process_dir);
        self.process_file_path.push(file_name);
        let mut file = File::open(&self.process_file_path)?;
        let _ = file.read_to_string(&mut self.process_text_buf)?;
        Ok(self.process_text_buf.as_str())
    }

    fn read_process_file_bytes(&mut self, process_dir: &Path, file_name: &str) -> io::Result<()> {
        self.process_bytes_buf.clear();
        self.process_file_path.clear();
        self.process_file_path.push(process_dir);
        self.process_file_path.push(file_name);
        let mut file = File::open(&self.process_file_path)?;
        let _ = file.read_to_end(&mut self.process_bytes_buf)?;
        Ok(())
    }

    fn is_expected_process_read_error(err: &io::Error) -> bool {
        matches!(
            err.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
        )
    }

    fn validate_selected_paths(&self) -> io::Result<()> {
        if self.config.cpu
            || self.config.memory
            || self.config.paging
            || self.config.disk
            || self.config.filesystem
            || self.config.network
            || self.config.processes
            || self.config.per_processes
        {
            let _ = File::open(self.paths.path(PathKind::Stat))?;
        }
        if self.config.cpu {
            let _ = File::open(self.paths.path(PathKind::Cpuinfo))?;
        }
        if self.config.memory {
            let _ = File::open(self.paths.path(PathKind::Meminfo))?;
        }
        if self.config.load {
            let _ = File::open(self.paths.path(PathKind::Loadavg))?;
        }
        if self.config.system {
            let _ = File::open(self.paths.path(PathKind::Uptime))?;
        }
        if self.config.paging {
            let _ = File::open(self.paths.path(PathKind::Vmstat))?;
            let _ = File::open(self.paths.path(PathKind::Swaps))?;
        }
        if self.config.disk {
            let _ = File::open(self.paths.path(PathKind::Diskstats))?;
        }
        if self.config.filesystem {
            let _ = File::open(self.paths.path(PathKind::Mountinfo))?;
        }
        if self.config.network {
            let _ = File::open(self.paths.path(PathKind::NetDev))?;
        }
        Ok(())
    }

    fn disable_unavailable_sources(&mut self) {
        if (self.config.cpu
            || self.config.system
            || self.config.processes
            || self.config.per_processes)
            && !self.source_available(PathKind::Stat)
        {
            self.config.cpu = false;
            self.config.system = false;
            self.config.processes = false;
            self.config.per_processes = false;
        }
        if self.config.cpu && !self.source_available(PathKind::Cpuinfo) {
            self.config.cpu = false;
        }
        if self.config.memory && !self.source_available(PathKind::Meminfo) {
            self.config.memory = false;
        }
        if self.config.load && !self.source_available(PathKind::Loadavg) {
            self.config.load = false;
        }
        if self.config.system && !self.source_available(PathKind::Uptime) {
            self.config.system = false;
        }
        if self.config.paging
            && (!self.source_available(PathKind::Vmstat) || !self.source_available(PathKind::Swaps))
        {
            self.config.paging = false;
        }
        if self.config.disk && !self.source_available(PathKind::Diskstats) {
            self.config.disk = false;
        }
        if self.config.filesystem && !self.source_available(PathKind::Mountinfo) {
            self.config.filesystem = false;
        }
        if self.config.network && !self.source_available(PathKind::NetDev) {
            self.config.network = false;
        }
    }

    fn source_available(&self, kind: PathKind) -> bool {
        File::open(self.paths.path(kind)).is_ok()
    }

    fn read_path(&mut self, kind: PathKind) -> io::Result<&str> {
        self.buf.clear();
        let mut file = File::open(self.paths.path(kind))?;
        let _ = file.read_to_string(&mut self.buf)?;
        Ok(self.buf.as_str())
    }

    fn read_disk_limit_bytes(&mut self, disk_name: &str) -> io::Result<u64> {
        self.buf.clear();
        let mut file = File::open(self.paths.sys_block.join(disk_name).join("size"))?;
        let _ = file.read_to_string(&mut self.buf)?;
        let sectors = parse_u64(self.buf.trim());
        Ok(sectors.saturating_mul(DISKSTAT_SECTOR_BYTES))
    }

    fn read_filesystems(
        &mut self,
        mounts: Vec<FilesystemMount>,
        partial_errors: &mut u64,
        first_error: &mut Option<io::Error>,
    ) -> Vec<FilesystemStats> {
        let started = Instant::now();
        let mut filesystems = Vec::with_capacity(mounts.len());
        for mount in mounts {
            let Some(remaining) = FILESYSTEM_SCRAPE_TIMEOUT.checked_sub(started.elapsed()) else {
                record_partial_error(
                    partial_errors,
                    first_error,
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        "filesystem scrape budget exhausted",
                    ),
                );
                break;
            };
            let path = self.paths.host_path(&mount.mountpoint);
            let stat = match self
                .filesystem_worker
                .statvfs(path, remaining.min(FILESYSTEM_STAT_TIMEOUT))
            {
                Ok(stat) => stat,
                Err(err) => {
                    record_partial_error(partial_errors, first_error, err);
                    continue;
                }
            };
            let free = stat.available_bytes;
            let reserved = stat.free_bytes.saturating_sub(stat.available_bytes);
            let used = stat.total_bytes.saturating_sub(stat.free_bytes);
            filesystems.push(FilesystemStats {
                device: mount.device,
                mountpoint: mount.mountpoint,
                fs_type: mount.fs_type,
                mode: mount.mode,
                used,
                free,
                reserved,
                limit_bytes: mount.emit_limit.then_some(stat.total_bytes),
            });
        }
        filesystems
    }

    fn read_resource(&mut self) -> &HostResource {
        if self.resource.is_none() {
            let host_id = self
                .read_trimmed_optional(PathKind::MachineId)
                .or_else(|| self.read_trimmed_optional(PathKind::DbusMachineId));
            let host_name = self.read_trimmed_optional(PathKind::Hostname);
            self.resource = Some(HostResource {
                host_id,
                host_name,
                host_arch: host_arch(),
            });
        }
        self.resource.as_ref().expect("resource is initialized")
    }

    fn read_trimmed_optional(&mut self, kind: PathKind) -> Option<String> {
        self.read_path(kind)
            .ok()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }
}

fn process_filter_allows(
    command: &ProcessCommand,
    include: Option<&CompiledFilter>,
    exclude: Option<&CompiledFilter>,
) -> bool {
    let matches = |filter: &CompiledFilter| {
        filter.matches(&command.command) || filter.matches(&command.executable_name)
    };
    include.is_none_or(matches) && !exclude.is_some_and(matches)
}

#[cfg(test)]
mod tests;
