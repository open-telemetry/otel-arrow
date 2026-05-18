// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux procfs-backed host metric source.

mod paths;
mod projection;
mod readings;

use crate::receivers::host_metrics_receiver::{CompiledFilter, HostViewValidationMode};
use paths::{PathKind, ProcfsPaths};
use projection::{CounterTracker, HostScrape, host_arch};
pub(crate) use projection::{HostResource, HostSnapshot};
use readings::*;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::{Duration, Instant};

const NANOS_PER_SEC: u64 = 1_000_000_000;
const BYTES_PER_KIB: u64 = 1024;
const DISKSTAT_SECTOR_BYTES: u64 = 512;
const FILESYSTEM_STAT_TIMEOUT: Duration = Duration::from_millis(100);
const FILESYSTEM_SCRAPE_TIMEOUT: Duration = Duration::from_secs(1);
const COUNTER_KEY_SEPARATOR: char = '\x1f';

/// Procfs-backed source for host metrics.
pub struct ProcfsSource {
    paths: ProcfsPaths,
    config: ProcfsConfig,
    buf: String,
    clk_tck: f64,
    previous_cpu: Option<CpuTimes>,
    filesystem_worker: FilesystemStatWorker,
    counter_tracker: CounterTracker,
    boot_time_unix_nano: Option<u64>,
    fallback_start_time_unix_nano: u64,
    resource: Option<HostResource>,
}

/// Procfs collection config.
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
    /// Startup validation mode.
    pub validation: HostViewValidationMode,
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
            clk_tck: clock_ticks_per_second(),
            previous_cpu: None,
            filesystem_worker: FilesystemStatWorker::new()?,
            counter_tracker: CounterTracker::default(),
            boot_time_unix_nano: None,
            fallback_start_time_unix_nano: now_unix_nano(),
            resource: None,
        };
        source.apply_startup_validation()?;
        Ok(source)
    }

    /// Collects one host snapshot for the due family set.
    pub async fn scrape_due(&mut self, due: ProcfsFamilies) -> io::Result<HostScrape> {
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
            || due.processes;
        let needs_stat =
            due.cpu || due.processes || (needs_start_time && self.boot_time_unix_nano.is_none());
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

        tokio::task::consume_budget().await;

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

        tokio::task::consume_budget().await;

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

        tokio::task::consume_budget().await;

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

        tokio::task::consume_budget().await;

        let resource = self.read_resource().clone();
        let counter_starts = self.counter_tracker.snapshot(
            start_time_unix_nano,
            now_unix_nano,
            due.cpu.then_some(stat.cpu).flatten().as_ref(),
            paging.as_ref(),
            due.processes.then_some(stat.processes).as_ref(),
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
            cpu: due.cpu.then_some(stat.cpu).flatten(),
            cpu_utilization,
            cpuinfo,
            memory,
            uptime_seconds,
            paging,
            swaps,
            processes: due.processes.then_some(stat.processes),
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

    fn validate_selected_paths(&self) -> io::Result<()> {
        if self.config.cpu
            || self.config.memory
            || self.config.paging
            || self.config.disk
            || self.config.filesystem
            || self.config.network
            || self.config.processes
        {
            let _ = File::open(self.paths.path(PathKind::Stat))?;
        }
        if self.config.cpu {
            let _ = File::open(self.paths.path(PathKind::Cpuinfo))?;
        }
        if self.config.memory {
            let _ = File::open(self.paths.path(PathKind::Meminfo))?;
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
        if (self.config.cpu || self.config.system || self.config.processes)
            && !self.source_available(PathKind::Stat)
        {
            self.config.cpu = false;
            self.config.system = false;
            self.config.processes = false;
        }
        if self.config.cpu && !self.source_available(PathKind::Cpuinfo) {
            self.config.cpu = false;
        }
        if self.config.memory && !self.source_available(PathKind::Meminfo) {
            self.config.memory = false;
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

#[cfg(test)]
mod tests;
