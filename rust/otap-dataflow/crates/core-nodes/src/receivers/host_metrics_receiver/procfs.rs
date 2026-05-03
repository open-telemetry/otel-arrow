// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux procfs-backed host metric source.

use crate::receivers::host_metrics_receiver::{CompiledFilter, HostViewValidationMode};
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue, any_value,
};
use otap_df_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    metric, number_data_point,
};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const NANOS_PER_SEC: u64 = 1_000_000_000;
const BYTES_PER_KIB: u64 = 1024;
const DISKSTAT_SECTOR_BYTES: u64 = 512;
const FILESYSTEM_STAT_TIMEOUT: Duration = Duration::from_millis(100);

/// Procfs-backed source for host metrics.
pub struct ProcfsSource {
    paths: ProcfsPaths,
    config: ProcfsConfig,
    buf: String,
    clk_tck: f64,
    previous_cpu: Option<CpuTimes>,
    filesystem_worker: FilesystemStatWorker,
    counter_tracker: CounterTracker,
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

impl ProcfsSource {
    /// Creates a procfs source rooted at `/` or at a host root bind mount.
    pub fn new(root_path: Option<&Path>, config: ProcfsConfig) -> io::Result<Self> {
        let mut source = Self {
            paths: ProcfsPaths::new(root_path),
            config,
            buf: String::with_capacity(16 * 1024),
            clk_tck: clock_ticks_per_second(),
            previous_cpu: None,
            filesystem_worker: FilesystemStatWorker::new(),
            counter_tracker: CounterTracker::default(),
        };
        source.apply_startup_validation()?;
        Ok(source)
    }

    /// Collects one host snapshot for the due family set.
    pub fn scrape_due(&mut self, due: ProcfsFamilies) -> io::Result<HostScrape> {
        let due = ProcfsFamilies {
            cpu: due.cpu && self.config.cpu,
            memory: due.memory && self.config.memory,
            paging: due.paging && self.config.paging,
            system: due.system && self.config.system,
            disk: due.disk && self.config.disk,
            filesystem: due.filesystem && self.config.filesystem,
            network: due.network && self.config.network,
            processes: due.processes && self.config.processes,
        };
        let now_unix_nano = now_unix_nano();
        let clk_tck = self.clk_tck;
        let mut partial_errors = 0;
        let mut first_error = None;
        let needs_stat = due.cpu || due.system || due.processes;
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
                    disks
                }
                Err(err) => {
                    record_partial_error(&mut partial_errors, &mut first_error, err);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        let networks = if due.network {
            let network_include = self.config.network_include.clone();
            let network_exclude = self.config.network_exclude.clone();
            match self.read_path(PathKind::NetDev) {
                Ok(netdev) => {
                    parse_netdev(netdev, network_include.as_ref(), network_exclude.as_ref())
                }
                Err(err) => {
                    record_partial_error(&mut partial_errors, &mut first_error, err);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        let filesystems = if due.filesystem {
            let include_virtual = self.config.filesystem_include_virtual;
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
                    let mounts = parse_mountinfo(mountinfo, include_virtual, emit_limit, filters);
                    self.read_filesystems(mounts)
                }
                Err(err) => {
                    record_partial_error(&mut partial_errors, &mut first_error, err);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        let resource = self.read_resource();
        let counter_starts = self.counter_tracker.snapshot(
            stat.boot_time_unix_nano,
            now_unix_nano,
            stat.cpu.as_ref(),
            paging.as_ref(),
            due.processes.then_some(stat.processes).as_ref(),
            &disks,
            &networks,
        );

        let snapshot = HostSnapshot {
            now_unix_nano,
            start_time_unix_nano: stat.boot_time_unix_nano,
            counter_starts,
            memory_limit: self.config.memory_limit,
            memory_shared: self.config.memory_shared,
            memory_hugepages: self.config.memory_hugepages,
            cpu: stat.cpu,
            cpu_utilization,
            cpuinfo,
            memory,
            uptime_seconds,
            paging,
            swaps,
            processes: due.processes.then_some(stat.processes),
            disks,
            filesystems,
            networks,
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
        if self.config.cpu || self.config.system || self.config.processes {
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

    fn read_filesystems(&mut self, mounts: Vec<FilesystemMount>) -> Vec<FilesystemStats> {
        let mut filesystems = Vec::with_capacity(mounts.len());
        for mount in mounts {
            let path = self.paths.host_path(&mount.mountpoint);
            let Ok(stat) = self
                .filesystem_worker
                .statvfs(path, FILESYSTEM_STAT_TIMEOUT)
            else {
                continue;
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

    fn read_resource(&mut self) -> HostResource {
        HostResource {
            host_id: self
                .read_trimmed_optional(PathKind::MachineId)
                .or_else(|| self.read_trimmed_optional(PathKind::DbusMachineId)),
            host_name: self.read_trimmed_optional(PathKind::Hostname),
            host_arch: host_arch(),
        }
    }

    fn read_trimmed_optional(&mut self, kind: PathKind) -> Option<String> {
        self.read_path(kind)
            .ok()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }
}

#[derive(Clone, Debug)]
struct ProcfsPaths {
    root: PathBuf,
    stat: PathBuf,
    cpuinfo: PathBuf,
    meminfo: PathBuf,
    uptime: PathBuf,
    vmstat: PathBuf,
    swaps: PathBuf,
    diskstats: PathBuf,
    mountinfo: PathBuf,
    sys_block: PathBuf,
    net_dev: PathBuf,
    machine_id: PathBuf,
    dbus_machine_id: PathBuf,
    hostname: PathBuf,
}

impl ProcfsPaths {
    fn new(root_path: Option<&Path>) -> Self {
        let root = root_path.unwrap_or_else(|| Path::new("/"));
        let host_root = root_path.is_some_and(|path| path != Path::new("/"));
        Self {
            root: root.to_path_buf(),
            stat: root.join("proc/stat"),
            cpuinfo: root.join("proc/cpuinfo"),
            meminfo: root.join("proc/meminfo"),
            uptime: root.join("proc/uptime"),
            vmstat: root.join("proc/vmstat"),
            swaps: root.join("proc/swaps"),
            diskstats: root.join("proc/diskstats"),
            mountinfo: if host_root {
                root.join("proc/1/mountinfo")
            } else {
                root.join("proc/self/mountinfo")
            },
            sys_block: root.join("sys/block"),
            machine_id: root.join("etc/machine-id"),
            dbus_machine_id: root.join("var/lib/dbus/machine-id"),
            hostname: root.join("proc/sys/kernel/hostname"),
            net_dev: if host_root {
                root.join("proc/1/net/dev")
            } else {
                root.join("proc/net/dev")
            },
        }
    }

    fn path(&self, kind: PathKind) -> &Path {
        match kind {
            PathKind::Stat => &self.stat,
            PathKind::Cpuinfo => &self.cpuinfo,
            PathKind::Meminfo => &self.meminfo,
            PathKind::Uptime => &self.uptime,
            PathKind::Vmstat => &self.vmstat,
            PathKind::Swaps => &self.swaps,
            PathKind::Diskstats => &self.diskstats,
            PathKind::Mountinfo => &self.mountinfo,
            PathKind::NetDev => &self.net_dev,
            PathKind::MachineId => &self.machine_id,
            PathKind::DbusMachineId => &self.dbus_machine_id,
            PathKind::Hostname => &self.hostname,
        }
    }

    fn host_path(&self, host_absolute_path: &str) -> PathBuf {
        let relative = host_absolute_path
            .strip_prefix('/')
            .unwrap_or(host_absolute_path);
        self.root.join(relative)
    }
}

#[derive(Copy, Clone)]
enum PathKind {
    Stat,
    Cpuinfo,
    Meminfo,
    Uptime,
    Vmstat,
    Swaps,
    Diskstats,
    Mountinfo,
    NetDev,
    MachineId,
    DbusMachineId,
    Hostname,
}

/// Result of one host metrics scrape.
pub struct HostScrape {
    /// Collected host snapshot.
    pub snapshot: HostSnapshot,
    /// Number of source read errors skipped because other families succeeded.
    pub partial_errors: u64,
}

/// One host metrics snapshot.
#[derive(Default)]
pub struct HostSnapshot {
    now_unix_nano: u64,
    start_time_unix_nano: u64,
    counter_starts: CounterStarts,
    memory_limit: bool,
    memory_shared: bool,
    memory_hugepages: bool,
    cpu: Option<CpuTimes>,
    cpu_utilization: Option<CpuTimes>,
    cpuinfo: CpuInfo,
    memory: Option<MemoryStats>,
    uptime_seconds: Option<f64>,
    paging: Option<PagingStats>,
    swaps: Vec<SwapStats>,
    processes: Option<ProcessStats>,
    disks: Vec<DiskStats>,
    filesystems: Vec<FilesystemStats>,
    networks: Vec<NetworkStats>,
    resource: HostResource,
}

impl HostSnapshot {
    fn has_metrics(&self) -> bool {
        self.cpu.is_some()
            || self.cpu_utilization.is_some()
            || self.cpuinfo.logical_count != 0
            || self.cpuinfo.physical_count != 0
            || !self.cpuinfo.frequencies_hz.is_empty()
            || self.memory.is_some()
            || self.uptime_seconds.is_some()
            || self.paging.is_some()
            || !self.swaps.is_empty()
            || self.processes.is_some()
            || !self.disks.is_empty()
            || !self.filesystems.is_empty()
            || !self.networks.is_empty()
    }

    /// Converts a snapshot into an OTLP metrics request.
    pub fn into_export_request(self) -> ExportMetricsServiceRequest {
        let mut metrics = Vec::with_capacity(64);
        let now = self.now_unix_nano;
        let start = self.start_time_unix_nano;
        let counter_starts = &self.counter_starts;

        if let Some(cpu) = self.cpu {
            push_sum_f64(
                &mut metrics,
                "system.cpu.time",
                "s",
                start,
                now,
                counter_starts,
                &[
                    ("user", cpu.user),
                    ("nice", cpu.nice),
                    ("system", cpu.system),
                    ("idle", cpu.idle),
                    ("iowait", cpu.wait),
                    ("interrupt", cpu.interrupt),
                    ("steal", cpu.steal),
                ],
                "cpu.mode",
            );
        }
        if let Some(cpu) = self.cpu_utilization {
            push_gauge_f64_by_attr(
                &mut metrics,
                "system.cpu.utilization",
                "1",
                now,
                &[
                    ("user", cpu.user),
                    ("nice", cpu.nice),
                    ("system", cpu.system),
                    ("idle", cpu.idle),
                    ("iowait", cpu.wait),
                    ("interrupt", cpu.interrupt),
                    ("steal", cpu.steal),
                ],
                "cpu.mode",
            );
        }

        if self.cpuinfo.logical_count != 0 {
            push_updown_single_u64(
                &mut metrics,
                "system.cpu.logical.count",
                "{cpu}",
                start,
                now,
                self.cpuinfo.logical_count,
            );
        }
        if self.cpuinfo.physical_count != 0 {
            push_updown_single_u64(
                &mut metrics,
                "system.cpu.physical.count",
                "{cpu}",
                start,
                now,
                self.cpuinfo.physical_count,
            );
        }
        push_cpu_frequency(&mut metrics, now, &self.cpuinfo.frequencies_hz);

        if let Some(memory) = self.memory {
            push_updown_u64(
                &mut metrics,
                "system.memory.usage",
                "By",
                start,
                now,
                &[
                    ("used", memory.used),
                    ("free", memory.free),
                    ("cached", memory.cached),
                    ("buffers", memory.buffered),
                ],
                "system.memory.state",
            );
            push_gauge_ratio(
                &mut metrics,
                "system.memory.utilization",
                "1",
                now,
                memory.total,
                &[
                    ("used", memory.used),
                    ("free", memory.free),
                    ("cached", memory.cached),
                    ("buffers", memory.buffered),
                ],
                "system.memory.state",
            );
            push_updown_single_u64(
                &mut metrics,
                "system.memory.linux.available",
                "By",
                start,
                now,
                memory.available,
            );
            push_updown_u64(
                &mut metrics,
                "system.memory.linux.slab.usage",
                "By",
                start,
                now,
                &[
                    ("reclaimable", memory.slab_reclaimable),
                    ("unreclaimable", memory.slab_unreclaimable),
                ],
                "system.memory.linux.slab.state",
            );
            if self.memory_limit {
                push_updown_single_u64(
                    &mut metrics,
                    "system.memory.limit",
                    "By",
                    start,
                    now,
                    memory.total,
                );
            }
            if self.memory_shared {
                push_updown_single_u64(
                    &mut metrics,
                    "system.memory.linux.shared",
                    "By",
                    start,
                    now,
                    memory.shared,
                );
            }
            if self.memory_hugepages {
                push_hugepage_metrics(&mut metrics, start, now, &memory.hugepages);
            }
        }

        if let Some(uptime_seconds) = self.uptime_seconds {
            push_gauge_f64(&mut metrics, "system.uptime", "s", now, uptime_seconds);
        }

        if let Some(paging) = self.paging {
            push_sum_u64(
                &mut metrics,
                "system.paging.faults",
                "{fault}",
                start,
                now,
                counter_starts,
                &[
                    ("minor", paging.minor_faults),
                    ("major", paging.major_faults),
                ],
                "system.paging.fault.type",
            );
            push_sum_u64(
                &mut metrics,
                "system.paging.operations",
                "{operation}",
                start,
                now,
                counter_starts,
                &[("in", paging.swap_in), ("out", paging.swap_out)],
                "system.paging.direction",
            );
        }

        for swap in self.swaps {
            push_updown_u64_with_device(
                &mut metrics,
                "system.paging.usage",
                "By",
                start,
                now,
                &swap.name,
                &[("used", swap.used), ("free", swap.free)],
                "system.paging.state",
            );
            push_gauge_ratio_with_device(
                &mut metrics,
                "system.paging.utilization",
                "1",
                now,
                &swap.name,
                swap.size,
                &[("used", swap.used), ("free", swap.free)],
                "system.paging.state",
            );
        }

        if let Some(processes) = self.processes {
            push_updown_u64(
                &mut metrics,
                "system.process.count",
                "{process}",
                start,
                now,
                &[
                    ("running", processes.running),
                    ("blocked", processes.blocked),
                ],
                "process.state",
            );
            push_sum_single_u64(
                &mut metrics,
                "system.process.created",
                "{process}",
                start,
                now,
                counter_starts,
                processes.created,
            );
        }

        for disk in self.disks {
            if let Some(limit_bytes) = disk.limit_bytes {
                push_updown_single_u64_with_device(
                    &mut metrics,
                    "system.disk.limit",
                    "By",
                    start,
                    now,
                    &disk.name,
                    limit_bytes,
                );
            }
            push_disk_sum(
                &mut metrics,
                "system.disk.io",
                "By",
                start,
                now,
                counter_starts,
                &disk,
                DiskProjection::Bytes,
            );
            push_disk_sum(
                &mut metrics,
                "system.disk.operations",
                "{operation}",
                start,
                now,
                counter_starts,
                &disk,
                DiskProjection::Operations,
            );
            push_disk_sum(
                &mut metrics,
                "system.disk.io_time",
                "s",
                start,
                now,
                counter_starts,
                &disk,
                DiskProjection::IoTime,
            );
            push_disk_sum(
                &mut metrics,
                "system.disk.operation_time",
                "s",
                start,
                now,
                counter_starts,
                &disk,
                DiskProjection::OperationTime,
            );
            push_disk_sum(
                &mut metrics,
                "system.disk.merged",
                "{operation}",
                start,
                now,
                counter_starts,
                &disk,
                DiskProjection::Merged,
            );
        }

        for filesystem in self.filesystems {
            push_filesystem_usage(&mut metrics, start, now, &filesystem);
            push_filesystem_utilization(&mut metrics, now, &filesystem);
            if let Some(limit_bytes) = filesystem.limit_bytes {
                push_filesystem_limit(&mut metrics, start, now, &filesystem, limit_bytes);
            }
        }

        for network in self.networks {
            push_network_sum(
                &mut metrics,
                "system.network.io",
                "By",
                start,
                now,
                counter_starts,
                &network,
                NetworkProjection::Bytes,
            );
            push_network_sum(
                &mut metrics,
                "system.network.packet.count",
                "{packet}",
                start,
                now,
                counter_starts,
                &network,
                NetworkProjection::Packets,
            );
            push_network_sum(
                &mut metrics,
                "system.network.packet.dropped",
                "{packet}",
                start,
                now,
                counter_starts,
                &network,
                NetworkProjection::Dropped,
            );
            push_network_sum(
                &mut metrics,
                "system.network.errors",
                "{error}",
                start,
                now,
                counter_starts,
                &network,
                NetworkProjection::Errors,
            );
        }

        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: self.resource.into_attributes(),
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "otap-df-core-nodes/host-metrics".to_owned(),
                        version: env!("CARGO_PKG_VERSION").to_owned(),
                        attributes: Vec::new(),
                        dropped_attributes_count: 0,
                    }),
                    metrics,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }
}

#[derive(Default)]
struct HostResource {
    host_id: Option<String>,
    host_name: Option<String>,
    host_arch: Option<&'static str>,
}

impl HostResource {
    fn into_attributes(self) -> Vec<KeyValue> {
        let mut attributes = Vec::with_capacity(4);
        attributes.push(kv_str("os.type", "linux"));
        if let Some(host_id) = self.host_id {
            attributes.push(kv_str("host.id", &host_id));
        }
        if let Some(host_name) = self.host_name {
            attributes.push(kv_str("host.name", &host_name));
        }
        if let Some(host_arch) = self.host_arch {
            attributes.push(kv_str("host.arch", host_arch));
        }
        attributes
    }
}

#[derive(Default)]
struct CounterTracker {
    states: HashMap<String, CounterState>,
}

struct CounterState {
    previous: f64,
    start_time_unix_nano: u64,
}

#[derive(Default)]
struct CounterStarts {
    entries: Vec<(String, u64)>,
}

impl CounterStarts {
    fn get(&self, metric: &'static str, series: &str, default_start: u64) -> u64 {
        self.entries
            .iter()
            .find_map(|(key, start)| counter_key_matches(key, metric, series).then_some(*start))
            .unwrap_or(default_start)
    }

    fn get_joined(
        &self,
        metric: &'static str,
        first: &str,
        second: &'static str,
        default_start: u64,
    ) -> u64 {
        self.entries
            .iter()
            .find_map(|(key, start)| {
                counter_key_matches_joined(key, metric, first, second).then_some(*start)
            })
            .unwrap_or(default_start)
    }
}

impl CounterTracker {
    fn snapshot(
        &mut self,
        default_start: u64,
        now: u64,
        cpu: Option<&CpuTimes>,
        paging: Option<&PagingStats>,
        processes: Option<&ProcessStats>,
        disks: &[DiskStats],
        networks: &[NetworkStats],
    ) -> CounterStarts {
        let mut starts = CounterStarts::default();
        if let Some(cpu) = cpu {
            self.observe_all(
                "system.cpu.time",
                default_start,
                now,
                &[
                    ("user", cpu.user),
                    ("nice", cpu.nice),
                    ("system", cpu.system),
                    ("idle", cpu.idle),
                    ("iowait", cpu.wait),
                    ("interrupt", cpu.interrupt),
                    ("steal", cpu.steal),
                ],
                &mut starts,
            );
        }
        if let Some(paging) = paging {
            self.observe_all(
                "system.paging.faults",
                default_start,
                now,
                &[
                    ("minor", paging.minor_faults as f64),
                    ("major", paging.major_faults as f64),
                ],
                &mut starts,
            );
            self.observe_all(
                "system.paging.operations",
                default_start,
                now,
                &[
                    ("in", paging.swap_in as f64),
                    ("out", paging.swap_out as f64),
                ],
                &mut starts,
            );
        }
        if let Some(processes) = processes {
            self.observe(
                "system.process.created",
                "",
                processes.created as f64,
                default_start,
                now,
                &mut starts,
            );
        }
        for disk in disks {
            self.observe_disk_all(
                "system.disk.io",
                default_start,
                now,
                &disk.name,
                &[
                    ("read", disk.read_bytes as f64),
                    ("write", disk.write_bytes as f64),
                ],
                &mut starts,
            );
            self.observe_disk_all(
                "system.disk.operations",
                default_start,
                now,
                &disk.name,
                &[
                    ("read", disk.read_ops as f64),
                    ("write", disk.write_ops as f64),
                ],
                &mut starts,
            );
            self.observe(
                "system.disk.io_time",
                &disk.name,
                disk.io_time_seconds,
                default_start,
                now,
                &mut starts,
            );
            self.observe_disk_all(
                "system.disk.operation_time",
                default_start,
                now,
                &disk.name,
                &[
                    ("read", disk.read_time_seconds),
                    ("write", disk.write_time_seconds),
                ],
                &mut starts,
            );
            self.observe_disk_all(
                "system.disk.merged",
                default_start,
                now,
                &disk.name,
                &[
                    ("read", disk.read_merged as f64),
                    ("write", disk.write_merged as f64),
                ],
                &mut starts,
            );
        }
        for network in networks {
            self.observe_network(
                "system.network.io",
                default_start,
                now,
                network,
                network.rx_bytes,
                network.tx_bytes,
                &mut starts,
            );
            self.observe_network(
                "system.network.packet.count",
                default_start,
                now,
                network,
                network.rx_packets,
                network.tx_packets,
                &mut starts,
            );
            self.observe_network(
                "system.network.packet.dropped",
                default_start,
                now,
                network,
                network.rx_dropped,
                network.tx_dropped,
                &mut starts,
            );
            self.observe_network(
                "system.network.errors",
                default_start,
                now,
                network,
                network.rx_errors,
                network.tx_errors,
                &mut starts,
            );
        }
        starts
    }

    fn observe_all(
        &mut self,
        metric: &'static str,
        default_start: u64,
        now: u64,
        values: &[(&str, f64)],
        starts: &mut CounterStarts,
    ) {
        for (series, value) in values {
            self.observe(metric, series, *value, default_start, now, starts);
        }
    }

    fn observe_disk_all(
        &mut self,
        metric: &'static str,
        default_start: u64,
        now: u64,
        device: &str,
        values: &[(&'static str, f64)],
        starts: &mut CounterStarts,
    ) {
        for (direction, value) in values {
            self.observe_joined(
                metric,
                device,
                direction,
                *value,
                default_start,
                now,
                starts,
            );
        }
    }

    fn observe_network(
        &mut self,
        metric: &'static str,
        default_start: u64,
        now: u64,
        network: &NetworkStats,
        rx: u64,
        tx: u64,
        starts: &mut CounterStarts,
    ) {
        self.observe_joined(
            metric,
            &network.name,
            "receive",
            rx as f64,
            default_start,
            now,
            starts,
        );
        self.observe_joined(
            metric,
            &network.name,
            "transmit",
            tx as f64,
            default_start,
            now,
            starts,
        );
    }

    fn observe(
        &mut self,
        metric: &'static str,
        series: &str,
        value: f64,
        default_start: u64,
        now: u64,
        starts: &mut CounterStarts,
    ) {
        self.observe_key(
            counter_key(metric, series),
            value,
            default_start,
            now,
            starts,
        );
    }

    fn observe_joined(
        &mut self,
        metric: &'static str,
        first: &str,
        second: &'static str,
        value: f64,
        default_start: u64,
        now: u64,
        starts: &mut CounterStarts,
    ) {
        self.observe_key(
            counter_key_joined(metric, first, second),
            value,
            default_start,
            now,
            starts,
        );
    }

    fn observe_key(
        &mut self,
        key: String,
        value: f64,
        default_start: u64,
        now: u64,
        starts: &mut CounterStarts,
    ) {
        let state = self.states.entry(key.clone()).or_insert(CounterState {
            previous: value,
            start_time_unix_nano: default_start,
        });
        if state.start_time_unix_nano < default_start {
            state.start_time_unix_nano = default_start;
        } else if value < state.previous {
            state.start_time_unix_nano = now;
        }
        state.previous = value;
        starts.entries.push((key, state.start_time_unix_nano));
    }
}

fn counter_key(metric: &'static str, series: &str) -> String {
    let mut key = String::with_capacity(metric.len() + 1 + series.len());
    key.push_str(metric);
    key.push('|');
    key.push_str(series);
    key
}

fn counter_key_joined(metric: &'static str, first: &str, second: &'static str) -> String {
    let mut key = String::with_capacity(metric.len() + 2 + first.len() + second.len());
    key.push_str(metric);
    key.push('|');
    key.push_str(first);
    key.push('|');
    key.push_str(second);
    key
}

fn counter_key_matches(key: &str, metric: &'static str, series: &str) -> bool {
    key.strip_prefix(metric)
        .and_then(|rest| rest.strip_prefix('|'))
        == Some(series)
}

fn counter_key_matches_joined(
    key: &str,
    metric: &'static str,
    first: &str,
    second: &'static str,
) -> bool {
    let Some(series) = key
        .strip_prefix(metric)
        .and_then(|rest| rest.strip_prefix('|'))
    else {
        return false;
    };
    series
        .strip_prefix(first)
        .and_then(|rest| rest.strip_prefix('|'))
        == Some(second)
}

fn host_arch() -> Option<&'static str> {
    match std::env::consts::ARCH {
        "aarch64" => Some("arm64"),
        "arm" => Some("arm32"),
        "powerpc" => Some("ppc32"),
        "powerpc64" => Some("ppc64"),
        "x86" => Some("x86"),
        "x86_64" => Some("amd64"),
        _ => None,
    }
}

#[derive(Copy, Clone, Default)]
struct CpuTimes {
    user: f64,
    nice: f64,
    system: f64,
    idle: f64,
    wait: f64,
    interrupt: f64,
    steal: f64,
}

#[derive(Clone, Default)]
struct CpuInfo {
    logical_count: u64,
    physical_count: u64,
    frequencies_hz: Vec<f64>,
}

#[derive(Copy, Clone, Default)]
struct StatSnapshot {
    boot_time_unix_nano: u64,
    cpu: Option<CpuTimes>,
    processes: ProcessStats,
}

#[derive(Copy, Clone, Default)]
struct MemoryStats {
    total: u64,
    used: u64,
    free: u64,
    available: u64,
    cached: u64,
    buffered: u64,
    shared: u64,
    slab_reclaimable: u64,
    slab_unreclaimable: u64,
    hugepages: HugepageStats,
}

#[derive(Copy, Clone, Default)]
struct HugepageStats {
    total: u64,
    free: u64,
    reserved: u64,
    surplus: u64,
    page_size_bytes: u64,
}

#[derive(Copy, Clone, Default)]
struct PagingStats {
    minor_faults: u64,
    major_faults: u64,
    swap_in: u64,
    swap_out: u64,
}

#[derive(Default)]
struct SwapStats {
    name: String,
    size: u64,
    used: u64,
    free: u64,
}

#[derive(Copy, Clone, Default)]
struct ProcessStats {
    running: u64,
    blocked: u64,
    created: u64,
}

#[derive(Default)]
struct DiskStats {
    name: String,
    limit_bytes: Option<u64>,
    read_bytes: u64,
    write_bytes: u64,
    read_ops: u64,
    write_ops: u64,
    read_merged: u64,
    write_merged: u64,
    read_time_seconds: f64,
    write_time_seconds: f64,
    io_time_seconds: f64,
}

#[derive(Default)]
struct FilesystemStats {
    device: String,
    mountpoint: String,
    fs_type: String,
    mode: &'static str,
    used: u64,
    free: u64,
    reserved: u64,
    limit_bytes: Option<u64>,
}

struct FilesystemStatWorker {
    tx: mpsc::SyncSender<FilesystemStatRequest>,
}

struct FilesystemStatRequest {
    path: PathBuf,
    response: mpsc::Sender<io::Result<FilesystemStat>>,
}

struct FilesystemStat {
    total_bytes: u64,
    free_bytes: u64,
    available_bytes: u64,
}

impl FilesystemStatWorker {
    fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel::<FilesystemStatRequest>(1);
        let _ = std::thread::Builder::new()
            .name("host-metrics-statvfs".to_owned())
            .spawn(move || {
                while let Ok(request) = rx.recv() {
                    let result = statvfs_bytes(&request.path);
                    let _ = request.response.send(result);
                }
            });
        Self { tx }
    }

    fn statvfs(&self, path: PathBuf, timeout: Duration) -> io::Result<FilesystemStat> {
        let (response, rx) = mpsc::channel();
        self.tx
            .try_send(FilesystemStatRequest { path, response })
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "statvfs worker is busy"))?;
        rx.recv_timeout(timeout)
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "statvfs timed out"))?
    }
}

fn statvfs_bytes(path: &Path) -> io::Result<FilesystemStat> {
    let stat = nix::sys::statvfs::statvfs(path).map_err(io::Error::other)?;
    let block_size = stat.fragment_size();
    Ok(FilesystemStat {
        total_bytes: u64::from(stat.blocks()).saturating_mul(block_size),
        free_bytes: u64::from(stat.blocks_free()).saturating_mul(block_size),
        available_bytes: u64::from(stat.blocks_available()).saturating_mul(block_size),
    })
}

#[derive(Default)]
struct NetworkStats {
    name: String,
    rx_bytes: u64,
    tx_bytes: u64,
    rx_packets: u64,
    tx_packets: u64,
    rx_errors: u64,
    tx_errors: u64,
    rx_dropped: u64,
    tx_dropped: u64,
}

fn parse_stat(input: &str, clk_tck: f64) -> StatSnapshot {
    let mut snapshot = StatSnapshot::default();
    for line in input.lines() {
        if let Some(rest) = line.strip_prefix("cpu ") {
            snapshot.cpu = parse_cpu_total(rest, clk_tck);
        } else if let Some(value) = line.strip_prefix("btime ") {
            snapshot.boot_time_unix_nano = parse_u64(value).saturating_mul(NANOS_PER_SEC);
        } else if let Some(value) = line.strip_prefix("procs_running ") {
            snapshot.processes.running = parse_u64(value);
        } else if let Some(value) = line.strip_prefix("procs_blocked ") {
            snapshot.processes.blocked = parse_u64(value);
        } else if let Some(value) = line.strip_prefix("processes ") {
            snapshot.processes.created = parse_u64(value);
        }
    }
    snapshot
}

fn parse_cpu_total(input: &str, clk_tck: f64) -> Option<CpuTimes> {
    let mut fields = [0_u64; 10];
    let mut count = 0;
    for (idx, token) in input.split_whitespace().take(fields.len()).enumerate() {
        fields[idx] = parse_u64(token);
        count += 1;
    }
    if count < 4 {
        return None;
    }

    let user = fields[0].saturating_sub(fields[8]);
    let nice = fields[1].saturating_sub(fields[9]);
    Some(CpuTimes {
        user: ticks_to_seconds(user, clk_tck),
        nice: ticks_to_seconds(nice, clk_tck),
        system: ticks_to_seconds(fields[2], clk_tck),
        idle: ticks_to_seconds(fields[3], clk_tck),
        wait: ticks_to_seconds(fields[4], clk_tck),
        interrupt: ticks_to_seconds(fields[5].saturating_add(fields[6]), clk_tck),
        steal: ticks_to_seconds(fields[7], clk_tck),
    })
}

fn cpu_utilization(previous: CpuTimes, current: CpuTimes) -> Option<CpuTimes> {
    let user = counter_delta(previous.user, current.user)?;
    let nice = counter_delta(previous.nice, current.nice)?;
    let system = counter_delta(previous.system, current.system)?;
    let idle = counter_delta(previous.idle, current.idle)?;
    let wait = counter_delta(previous.wait, current.wait)?;
    let interrupt = counter_delta(previous.interrupt, current.interrupt)?;
    let steal = counter_delta(previous.steal, current.steal)?;
    let total = user + nice + system + idle + wait + interrupt + steal;
    (total > 0.0).then(|| CpuTimes {
        user: user / total,
        nice: nice / total,
        system: system / total,
        idle: idle / total,
        wait: wait / total,
        interrupt: interrupt / total,
        steal: steal / total,
    })
}

fn counter_delta(previous: f64, current: f64) -> Option<f64> {
    (current >= previous).then_some(current - previous)
}

fn parse_cpuinfo(input: &str) -> CpuInfo {
    let mut logical_count = 0;
    let mut frequencies_hz = Vec::new();
    let mut physical_cores = HashSet::new();
    let mut physical_id = None;
    let mut core_id = None;

    for line in input.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "processor" => {
                logical_count += 1;
                if let (Some(physical), Some(core)) = (physical_id.take(), core_id.take()) {
                    let _ = physical_cores.insert((physical, core));
                }
            }
            "physical id" => physical_id = Some(parse_u64(value)),
            "core id" => core_id = Some(parse_u64(value)),
            "cpu MHz" => {
                if let Ok(mhz) = value.parse::<f64>() {
                    frequencies_hz.push(mhz * 1_000_000.0);
                }
            }
            _ => {}
        }
    }
    if let (Some(physical), Some(core)) = (physical_id, core_id) {
        let _ = physical_cores.insert((physical, core));
    }

    let physical_count = u64::try_from(physical_cores.len())
        .ok()
        .filter(|count| *count != 0)
        .unwrap_or(logical_count);
    CpuInfo {
        logical_count,
        physical_count,
        frequencies_hz,
    }
}

fn parse_meminfo(input: &str) -> Option<MemoryStats> {
    let mut total = 0;
    let mut free = 0;
    let mut available = None;
    let mut buffers = 0;
    let mut cached = 0;
    let mut shared = 0;
    let mut slab_reclaimable = 0;
    let mut slab_unreclaimable = 0;
    let mut hugepages = HugepageStats::default();

    for line in input.lines() {
        let mut fields = line.split_whitespace();
        let Some(key) = fields.next() else {
            continue;
        };
        let raw_value = fields.next().map(parse_u64).unwrap_or_default();
        let value = raw_value * BYTES_PER_KIB;
        match key.trim_end_matches(':') {
            "MemTotal" => total = value,
            "MemFree" => free = value,
            "MemAvailable" => available = Some(value),
            "Buffers" => buffers = value,
            "Cached" => cached = value,
            "Shmem" => shared = value,
            "SReclaimable" => slab_reclaimable = value,
            "SUnreclaim" => slab_unreclaimable = value,
            "HugePages_Total" => hugepages.total = raw_value,
            "HugePages_Free" => hugepages.free = raw_value,
            "HugePages_Rsvd" => hugepages.reserved = raw_value,
            "HugePages_Surp" => hugepages.surplus = raw_value,
            "Hugepagesize" => hugepages.page_size_bytes = value,
            _ => {}
        }
    }

    if total == 0 {
        return None;
    }
    let available =
        available.unwrap_or_else(|| free.saturating_add(buffers).saturating_add(cached));
    Some(MemoryStats {
        total,
        used: total.saturating_sub(available),
        free,
        available,
        cached,
        buffered: buffers,
        shared,
        slab_reclaimable,
        slab_unreclaimable,
        hugepages,
    })
}

fn parse_uptime(input: &str) -> Option<f64> {
    input.split_whitespace().next()?.parse().ok()
}

fn parse_vmstat(input: &str) -> PagingStats {
    let mut total_faults = 0;
    let mut major_faults = 0;
    let mut swap_in = 0;
    let mut swap_out = 0;

    for line in input.lines() {
        let mut fields = line.split_whitespace();
        let Some(key) = fields.next() else {
            continue;
        };
        let value = fields.next().map(parse_u64).unwrap_or_default();
        match key {
            "pgfault" => total_faults = value,
            "pgmajfault" => major_faults = value,
            "pswpin" => swap_in = value,
            "pswpout" => swap_out = value,
            _ => {}
        }
    }

    PagingStats {
        minor_faults: total_faults.saturating_sub(major_faults),
        major_faults,
        swap_in,
        swap_out,
    }
}

fn parse_swaps(input: &str) -> Vec<SwapStats> {
    let mut swaps = Vec::new();
    for line in input.lines().skip(1) {
        let mut fields = line.split_whitespace();
        let Some(name) = fields.next() else {
            continue;
        };
        let _kind = fields.next();
        let Some(size_kib) = fields.next() else {
            continue;
        };
        let Some(used_kib) = fields.next() else {
            continue;
        };
        let size = parse_u64(size_kib).saturating_mul(BYTES_PER_KIB);
        let used = parse_u64(used_kib).saturating_mul(BYTES_PER_KIB);
        swaps.push(SwapStats {
            name: name.to_owned(),
            size,
            used,
            free: size.saturating_sub(used),
        });
    }
    swaps
}

fn parse_diskstats(
    input: &str,
    include: Option<&CompiledFilter>,
    exclude: Option<&CompiledFilter>,
) -> Vec<DiskStats> {
    let mut disks = Vec::new();
    for line in input.lines() {
        let mut fields = line.split_whitespace();
        let _major = fields.next();
        let _minor = fields.next();
        let Some(name) = fields.next() else {
            continue;
        };
        if !filter_allows(name, include, exclude) {
            continue;
        }
        let Some(read_ops) = fields.next() else {
            continue;
        };
        let Some(read_merged) = fields.next() else {
            continue;
        };
        let Some(read_sectors) = fields.next() else {
            continue;
        };
        let Some(read_ms) = fields.next() else {
            continue;
        };
        let Some(write_ops) = fields.next() else {
            continue;
        };
        let Some(write_merged) = fields.next() else {
            continue;
        };
        let Some(write_sectors) = fields.next() else {
            continue;
        };
        let Some(write_ms) = fields.next() else {
            continue;
        };
        let _in_progress = fields.next();
        let Some(io_ms) = fields.next() else {
            continue;
        };
        disks.push(DiskStats {
            name: name.to_owned(),
            limit_bytes: None,
            read_ops: parse_u64(read_ops),
            read_bytes: parse_u64(read_sectors).saturating_mul(DISKSTAT_SECTOR_BYTES),
            write_ops: parse_u64(write_ops),
            write_bytes: parse_u64(write_sectors).saturating_mul(DISKSTAT_SECTOR_BYTES),
            read_merged: parse_u64(read_merged),
            write_merged: parse_u64(write_merged),
            read_time_seconds: millis_to_seconds(parse_u64(read_ms)),
            write_time_seconds: millis_to_seconds(parse_u64(write_ms)),
            io_time_seconds: millis_to_seconds(parse_u64(io_ms)),
        });
    }
    disks
}

struct FilesystemMount {
    device: String,
    mountpoint: String,
    fs_type: String,
    mode: &'static str,
    emit_limit: bool,
}

#[derive(Clone, Copy, Default)]
struct FilesystemFilters<'a> {
    include_devices: Option<&'a CompiledFilter>,
    exclude_devices: Option<&'a CompiledFilter>,
    include_fs_types: Option<&'a CompiledFilter>,
    exclude_fs_types: Option<&'a CompiledFilter>,
    include_mount_points: Option<&'a CompiledFilter>,
    exclude_mount_points: Option<&'a CompiledFilter>,
}

fn parse_mountinfo(
    input: &str,
    include_virtual_filesystems: bool,
    emit_limit: bool,
    filters: FilesystemFilters<'_>,
) -> Vec<FilesystemMount> {
    let mut mounts = Vec::new();
    for line in input.lines() {
        let Some(separator) = line.find(" - ") else {
            continue;
        };
        let mut pre_fields = line[..separator].split_whitespace();
        let _mount_id = pre_fields.next();
        let _parent_id = pre_fields.next();
        let _major_minor = pre_fields.next();
        let _root = pre_fields.next();
        let Some(mountpoint) = pre_fields.next() else {
            continue;
        };
        let Some(options) = pre_fields.next() else {
            continue;
        };

        let mut post_fields = line[separator + 3..].split_whitespace();
        let Some(fs_type) = post_fields.next() else {
            continue;
        };
        let Some(device) = post_fields.next() else {
            continue;
        };
        if !include_virtual_filesystems && is_skipped_filesystem_type(fs_type) {
            continue;
        }
        if !filter_allows(fs_type, filters.include_fs_types, filters.exclude_fs_types) {
            continue;
        }
        let device = unescape_mountinfo(device);
        if !filter_allows(&device, filters.include_devices, filters.exclude_devices) {
            continue;
        }
        let mountpoint = unescape_mountinfo(mountpoint);
        if !filter_allows(
            &mountpoint,
            filters.include_mount_points,
            filters.exclude_mount_points,
        ) {
            continue;
        }
        mounts.push(FilesystemMount {
            device,
            mountpoint,
            fs_type: fs_type.to_owned(),
            mode: filesystem_mode(options),
            emit_limit,
        });
    }
    mounts
}

fn filesystem_mode(options: &str) -> &'static str {
    if options.split(',').any(|option| option == "ro") {
        "ro"
    } else {
        "rw"
    }
}

fn is_skipped_filesystem_type(fs_type: &str) -> bool {
    matches!(
        fs_type,
        "autofs"
            | "bpf"
            | "binfmt_misc"
            | "cgroup"
            | "cgroup2"
            | "debugfs"
            | "devtmpfs"
            | "fusectl"
            | "mqueue"
            | "nsfs"
            | "overlay"
            | "proc"
            | "pstore"
            | "squashfs"
            | "sysfs"
            | "tmpfs"
            | "tracefs"
            | "nfs"
            | "nfs4"
            | "cifs"
            | "smb3"
            | "9p"
    )
}

fn unescape_mountinfo(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut escaped = None;
    for idx in 0..bytes.len() {
        if bytes[idx] == b'\\' && idx + 3 < bytes.len() {
            escaped = Some(idx);
            break;
        }
    }
    let Some(first_escape) = escaped else {
        return input.to_owned();
    };

    let mut output = String::with_capacity(input.len());
    output.push_str(&input[..first_escape]);
    let mut idx = first_escape;
    while idx < bytes.len() {
        if bytes[idx] == b'\\' && idx + 3 < bytes.len() {
            let octal = &input[idx + 1..idx + 4];
            if let Ok(value) = u8::from_str_radix(octal, 8) {
                output.push(value as char);
                idx += 4;
                continue;
            }
        }
        output.push(bytes[idx] as char);
        idx += 1;
    }
    output
}

fn parse_netdev(
    input: &str,
    include: Option<&CompiledFilter>,
    exclude: Option<&CompiledFilter>,
) -> Vec<NetworkStats> {
    let mut interfaces = Vec::new();
    for line in input.lines().skip(2) {
        let Some((name, values)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if !filter_allows(name, include, exclude) {
            continue;
        }
        let mut fields = values.split_whitespace();
        let Some(rx_bytes) = fields.next() else {
            continue;
        };
        let Some(rx_packets) = fields.next() else {
            continue;
        };
        let Some(rx_errors) = fields.next() else {
            continue;
        };
        let Some(rx_dropped) = fields.next() else {
            continue;
        };
        let _rx_fifo = fields.next();
        let _rx_frame = fields.next();
        let _rx_compressed = fields.next();
        let _rx_multicast = fields.next();
        let Some(tx_bytes) = fields.next() else {
            continue;
        };
        let Some(tx_packets) = fields.next() else {
            continue;
        };
        let Some(tx_errors) = fields.next() else {
            continue;
        };
        let Some(tx_dropped) = fields.next() else {
            continue;
        };
        interfaces.push(NetworkStats {
            name: name.to_owned(),
            rx_bytes: parse_u64(rx_bytes),
            rx_packets: parse_u64(rx_packets),
            tx_bytes: parse_u64(tx_bytes),
            tx_packets: parse_u64(tx_packets),
            rx_errors: parse_u64(rx_errors),
            tx_errors: parse_u64(tx_errors),
            rx_dropped: parse_u64(rx_dropped),
            tx_dropped: parse_u64(tx_dropped),
        });
    }
    interfaces
}

fn filter_allows(
    value: &str,
    include: Option<&CompiledFilter>,
    exclude: Option<&CompiledFilter>,
) -> bool {
    include.is_none_or(|filter| filter.matches(value))
        && !exclude.is_some_and(|filter| filter.matches(value))
}

fn record_partial_error(
    partial_errors: &mut u64,
    first_error: &mut Option<io::Error>,
    err: io::Error,
) {
    *partial_errors = partial_errors.saturating_add(1);
    if first_error.is_none() {
        *first_error = Some(err);
    }
}

fn push_gauge_f64(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    now: u64,
    value: f64,
) {
    metrics.push(Metric {
        name: name.to_owned(),
        description: String::new(),
        unit: unit.to_owned(),
        metadata: Vec::new(),
        data: Some(metric::Data::Gauge(Gauge {
            data_points: vec![number_point_f64(Vec::new(), 0, now, value)],
        })),
    });
}

fn push_gauge_f64_by_attr(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    now: u64,
    values: &[(&'static str, f64)],
    attr_name: &'static str,
) {
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_f64(
            vec![kv_str(attr_name, state)],
            0,
            now,
            *value,
        ));
    }
    metrics.push(Metric {
        name: name.to_owned(),
        description: String::new(),
        unit: unit.to_owned(),
        metadata: Vec::new(),
        data: Some(metric::Data::Gauge(Gauge {
            data_points: points,
        })),
    });
}

fn push_updown_u64(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    values: &[(&'static str, u64)],
    attr_name: &'static str,
) {
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_i64(
            vec![kv_str(attr_name, state)],
            start,
            now,
            saturating_i64(*value),
        ));
    }
    push_updown_metric(metrics, name, unit, points);
}

fn push_updown_u64_with_device(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    device: &str,
    values: &[(&'static str, u64)],
    attr_name: &'static str,
) {
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_i64(
            vec![kv_str("system.device", device), kv_str(attr_name, state)],
            start,
            now,
            saturating_i64(*value),
        ));
    }
    push_updown_metric(metrics, name, unit, points);
}

fn push_updown_single_u64(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    value: u64,
) {
    push_updown_metric(
        metrics,
        name,
        unit,
        vec![number_point_i64(
            Vec::new(),
            start,
            now,
            saturating_i64(value),
        )],
    );
}

fn push_updown_single_u64_with_device(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    device: &str,
    value: u64,
) {
    push_updown_metric(
        metrics,
        name,
        unit,
        vec![number_point_i64(
            vec![kv_str("system.device", device)],
            start,
            now,
            saturating_i64(value),
        )],
    );
}

fn push_cpu_frequency(metrics: &mut Vec<Metric>, now: u64, frequencies_hz: &[f64]) {
    if frequencies_hz.is_empty() {
        return;
    }
    let mut points = Vec::with_capacity(frequencies_hz.len());
    for (idx, frequency) in frequencies_hz.iter().enumerate() {
        points.push(number_point_f64(
            vec![kv_str("cpu.logical_number", &idx.to_string())],
            0,
            now,
            *frequency,
        ));
    }
    metrics.push(Metric {
        name: "system.cpu.frequency".to_owned(),
        description: String::new(),
        unit: "Hz".to_owned(),
        metadata: Vec::new(),
        data: Some(metric::Data::Gauge(Gauge {
            data_points: points,
        })),
    });
}

fn push_hugepage_metrics(
    metrics: &mut Vec<Metric>,
    start: u64,
    now: u64,
    hugepages: &HugepageStats,
) {
    push_updown_single_u64(
        metrics,
        "system.memory.linux.hugepages.limit",
        "{page}",
        start,
        now,
        hugepages.total,
    );
    push_updown_single_u64(
        metrics,
        "system.memory.linux.hugepages.page_size",
        "By",
        start,
        now,
        hugepages.page_size_bytes,
    );
    push_updown_single_u64(
        metrics,
        "system.memory.linux.hugepages.reserved",
        "{page}",
        start,
        now,
        hugepages.reserved,
    );
    push_updown_single_u64(
        metrics,
        "system.memory.linux.hugepages.surplus",
        "{page}",
        start,
        now,
        hugepages.surplus,
    );
    let used = hugepages.total.saturating_sub(hugepages.free);
    push_updown_u64(
        metrics,
        "system.memory.linux.hugepages.usage",
        "{page}",
        start,
        now,
        &[("used", used), ("free", hugepages.free)],
        "system.memory.linux.hugepages.state",
    );
    push_gauge_ratio(
        metrics,
        "system.memory.linux.hugepages.utilization",
        "1",
        now,
        hugepages.total,
        &[("used", used), ("free", hugepages.free)],
        "system.memory.linux.hugepages.state",
    );
}

fn push_gauge_ratio(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    now: u64,
    total: u64,
    values: &[(&'static str, u64)],
    attr_name: &'static str,
) {
    if total == 0 {
        return;
    }
    let total = total as f64;
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_f64(
            vec![kv_str(attr_name, state)],
            0,
            now,
            *value as f64 / total,
        ));
    }
    metrics.push(Metric {
        name: name.to_owned(),
        description: String::new(),
        unit: unit.to_owned(),
        metadata: Vec::new(),
        data: Some(metric::Data::Gauge(Gauge {
            data_points: points,
        })),
    });
}

fn push_gauge_ratio_with_device(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    now: u64,
    device: &str,
    total: u64,
    values: &[(&'static str, u64)],
    attr_name: &'static str,
) {
    if total == 0 {
        return;
    }
    let total = total as f64;
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_f64(
            vec![kv_str("system.device", device), kv_str(attr_name, state)],
            0,
            now,
            *value as f64 / total,
        ));
    }
    metrics.push(Metric {
        name: name.to_owned(),
        description: String::new(),
        unit: unit.to_owned(),
        metadata: Vec::new(),
        data: Some(metric::Data::Gauge(Gauge {
            data_points: points,
        })),
    });
}

fn push_sum_f64(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    counter_starts: &CounterStarts,
    values: &[(&'static str, f64)],
    attr_name: &'static str,
) {
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_f64(
            vec![kv_str(attr_name, state)],
            counter_starts.get(name, state, start),
            now,
            *value,
        ));
    }
    push_sum_metric(metrics, name, unit, points);
}

fn push_sum_u64(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    counter_starts: &CounterStarts,
    values: &[(&'static str, u64)],
    attr_name: &'static str,
) {
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_i64(
            vec![kv_str(attr_name, state)],
            counter_starts.get(name, state, start),
            now,
            saturating_i64(*value),
        ));
    }
    push_sum_metric(metrics, name, unit, points);
}

fn push_sum_single_u64(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    counter_starts: &CounterStarts,
    value: u64,
) {
    push_sum_metric(
        metrics,
        name,
        unit,
        vec![number_point_i64(
            Vec::new(),
            counter_starts.get(name, "", start),
            now,
            saturating_i64(value),
        )],
    );
}

fn push_disk_sum(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    counter_starts: &CounterStarts,
    disk: &DiskStats,
    projection: DiskProjection,
) {
    if let DiskProjection::IoTime = projection {
        push_sum_metric(
            metrics,
            name,
            unit,
            vec![number_point_f64(
                vec![kv_str("system.device", &disk.name)],
                counter_starts.get(name, &disk.name, start),
                now,
                disk.io_time_seconds,
            )],
        );
        return;
    }

    let (read, write) = match projection {
        DiskProjection::Bytes => (
            DiskValue::Integer(disk.read_bytes),
            DiskValue::Integer(disk.write_bytes),
        ),
        DiskProjection::Operations => (
            DiskValue::Integer(disk.read_ops),
            DiskValue::Integer(disk.write_ops),
        ),
        DiskProjection::OperationTime => (
            DiskValue::Float(disk.read_time_seconds),
            DiskValue::Float(disk.write_time_seconds),
        ),
        DiskProjection::Merged => (
            DiskValue::Integer(disk.read_merged),
            DiskValue::Integer(disk.write_merged),
        ),
        DiskProjection::IoTime => unreachable!(),
    };
    let points = vec![
        disk_number_point(&disk.name, "read", start, now, counter_starts, name, read),
        disk_number_point(&disk.name, "write", start, now, counter_starts, name, write),
    ];
    push_sum_metric(metrics, name, unit, points);
}

#[derive(Copy, Clone)]
enum DiskProjection {
    Bytes,
    Operations,
    IoTime,
    OperationTime,
    Merged,
}

#[derive(Copy, Clone)]
enum DiskValue {
    Integer(u64),
    Float(f64),
}

fn disk_number_point(
    device: &str,
    direction: &'static str,
    start: u64,
    now: u64,
    counter_starts: &CounterStarts,
    metric: &'static str,
    value: DiskValue,
) -> NumberDataPoint {
    let attributes = vec![
        kv_str("system.device", device),
        kv_str("disk.io.direction", direction),
    ];
    match value {
        DiskValue::Integer(value) => number_point_i64(
            attributes,
            counter_starts.get_joined(metric, device, direction, start),
            now,
            saturating_i64(value),
        ),
        DiskValue::Float(value) => number_point_f64(
            attributes,
            counter_starts.get_joined(metric, device, direction, start),
            now,
            value,
        ),
    }
}

fn push_filesystem_usage(
    metrics: &mut Vec<Metric>,
    start: u64,
    now: u64,
    filesystem: &FilesystemStats,
) {
    let points = vec![
        filesystem_number_point(
            filesystem,
            "used",
            start,
            now,
            FilesystemValue::Integer(filesystem.used),
        ),
        filesystem_number_point(
            filesystem,
            "free",
            start,
            now,
            FilesystemValue::Integer(filesystem.free),
        ),
        filesystem_number_point(
            filesystem,
            "reserved",
            start,
            now,
            FilesystemValue::Integer(filesystem.reserved),
        ),
    ];
    push_updown_metric(metrics, "system.filesystem.usage", "By", points);
}

fn push_filesystem_utilization(metrics: &mut Vec<Metric>, now: u64, filesystem: &FilesystemStats) {
    let total = filesystem
        .used
        .saturating_add(filesystem.free)
        .saturating_add(filesystem.reserved);
    if total == 0 {
        return;
    }
    let total = total as f64;
    let points = vec![
        filesystem_number_point(
            filesystem,
            "used",
            0,
            now,
            FilesystemValue::Float(filesystem.used as f64 / total),
        ),
        filesystem_number_point(
            filesystem,
            "free",
            0,
            now,
            FilesystemValue::Float(filesystem.free as f64 / total),
        ),
        filesystem_number_point(
            filesystem,
            "reserved",
            0,
            now,
            FilesystemValue::Float(filesystem.reserved as f64 / total),
        ),
    ];
    metrics.push(Metric {
        name: "system.filesystem.utilization".to_owned(),
        description: String::new(),
        unit: "1".to_owned(),
        metadata: Vec::new(),
        data: Some(metric::Data::Gauge(Gauge {
            data_points: points,
        })),
    });
}

fn push_filesystem_limit(
    metrics: &mut Vec<Metric>,
    start: u64,
    now: u64,
    filesystem: &FilesystemStats,
    limit_bytes: u64,
) {
    push_updown_metric(
        metrics,
        "system.filesystem.limit",
        "By",
        vec![number_point_i64(
            vec![
                kv_str("system.device", &filesystem.device),
                kv_str("system.filesystem.type", &filesystem.fs_type),
                kv_str("system.filesystem.mode", filesystem.mode),
                kv_str("system.filesystem.mountpoint", &filesystem.mountpoint),
            ],
            start,
            now,
            saturating_i64(limit_bytes),
        )],
    );
}

#[derive(Copy, Clone)]
enum FilesystemValue {
    Integer(u64),
    Float(f64),
}

fn filesystem_number_point(
    filesystem: &FilesystemStats,
    state: &'static str,
    start: u64,
    now: u64,
    value: FilesystemValue,
) -> NumberDataPoint {
    let attributes = vec![
        kv_str("system.device", &filesystem.device),
        kv_str("system.filesystem.state", state),
        kv_str("system.filesystem.type", &filesystem.fs_type),
        kv_str("system.filesystem.mode", filesystem.mode),
        kv_str("system.filesystem.mountpoint", &filesystem.mountpoint),
    ];
    match value {
        FilesystemValue::Integer(value) => {
            number_point_i64(attributes, start, now, saturating_i64(value))
        }
        FilesystemValue::Float(value) => number_point_f64(attributes, start, now, value),
    }
}

fn push_network_sum(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    counter_starts: &CounterStarts,
    network: &NetworkStats,
    projection: NetworkProjection,
) {
    let (rx, tx, interface_attr) = match projection {
        NetworkProjection::Bytes => (network.rx_bytes, network.tx_bytes, "network.interface.name"),
        NetworkProjection::Packets => (network.rx_packets, network.tx_packets, "system.device"),
        NetworkProjection::Dropped => (
            network.rx_dropped,
            network.tx_dropped,
            "network.interface.name",
        ),
        NetworkProjection::Errors => (
            network.rx_errors,
            network.tx_errors,
            "network.interface.name",
        ),
    };
    let points = vec![
        number_point_i64(
            vec![
                kv_str(interface_attr, &network.name),
                kv_str("network.io.direction", "receive"),
            ],
            counter_starts.get_joined(name, &network.name, "receive", start),
            now,
            saturating_i64(rx),
        ),
        number_point_i64(
            vec![
                kv_str(interface_attr, &network.name),
                kv_str("network.io.direction", "transmit"),
            ],
            counter_starts.get_joined(name, &network.name, "transmit", start),
            now,
            saturating_i64(tx),
        ),
    ];
    push_sum_metric(metrics, name, unit, points);
}

#[derive(Copy, Clone)]
enum NetworkProjection {
    Bytes,
    Packets,
    Dropped,
    Errors,
}

fn push_sum_metric(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    points: Vec<NumberDataPoint>,
) {
    push_sum_metric_with_monotonic(metrics, name, unit, points, true);
}

fn push_updown_metric(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    points: Vec<NumberDataPoint>,
) {
    push_sum_metric_with_monotonic(metrics, name, unit, points, false);
}

fn push_sum_metric_with_monotonic(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    points: Vec<NumberDataPoint>,
    is_monotonic: bool,
) {
    metrics.push(Metric {
        name: name.to_owned(),
        description: String::new(),
        unit: unit.to_owned(),
        metadata: Vec::new(),
        data: Some(metric::Data::Sum(Sum {
            data_points: points,
            aggregation_temporality: AggregationTemporality::Cumulative.into(),
            is_monotonic,
        })),
    });
}

fn number_point_f64(
    attributes: Vec<KeyValue>,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    value: f64,
) -> NumberDataPoint {
    NumberDataPoint {
        attributes,
        start_time_unix_nano,
        time_unix_nano,
        exemplars: Vec::new(),
        flags: 0,
        value: Some(number_data_point::Value::AsDouble(value)),
    }
}

fn number_point_i64(
    attributes: Vec<KeyValue>,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    value: i64,
) -> NumberDataPoint {
    NumberDataPoint {
        attributes,
        start_time_unix_nano,
        time_unix_nano,
        exemplars: Vec::new(),
        flags: 0,
        value: Some(number_data_point::Value::AsInt(value)),
    }
}

fn kv_str(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_owned(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_owned())),
        }),
    }
}

fn parse_u64(input: &str) -> u64 {
    input.parse().unwrap_or_default()
}

fn ticks_to_seconds(ticks: u64, clk_tck: f64) -> f64 {
    ticks as f64 / clk_tck
}

fn millis_to_seconds(ms: u64) -> f64 {
    ms as f64 / 1_000.0
}

fn clock_ticks_per_second() -> f64 {
    nix::unistd::sysconf(nix::unistd::SysconfVar::CLK_TCK)
        .ok()
        .flatten()
        .filter(|ticks| *ticks > 0)
        .map(|ticks| ticks as f64)
        .unwrap_or(100.0)
}

fn now_unix_nano() -> u64 {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return 0;
    };
    duration.as_secs().saturating_mul(NANOS_PER_SEC) + u64::from(duration.subsec_nanos())
}

fn saturating_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "dev-tools")]
    use std::collections::{BTreeMap, BTreeSet};
    #[cfg(feature = "dev-tools")]
    use weaver_common::{result::WResult, vdir::VirtualDirectoryPath};
    #[cfg(feature = "dev-tools")]
    use weaver_forge::registry::ResolvedRegistry;
    #[cfg(feature = "dev-tools")]
    use weaver_resolver::SchemaResolver;
    #[cfg(feature = "dev-tools")]
    use weaver_semconv::{
        attribute::{BasicRequirementLevelSpec, RequirementLevel},
        group::{GroupType, InstrumentSpec},
        registry_repo::RegistryRepo,
    };

    #[test]
    fn projection_uses_expected_metric_shapes() {
        let request = projection_fixture_request();

        let resource_metrics = request.resource_metrics.first().expect("resource metrics");
        let resource = resource_metrics.resource.as_ref().expect("resource");
        assert_has_attr(&resource.attributes, "os.type", "linux");
        assert_has_attr(&resource.attributes, "host.id", "host-id");
        assert_has_attr(&resource.attributes, "host.name", "host-name");
        assert_has_attr(&resource.attributes, "host.arch", "amd64");

        let metrics = &resource_metrics.scope_metrics[0].metrics;
        assert_metric_shape(metrics, "system.cpu.time", "s", Some(true));
        assert_first_point_attr(metrics, "system.cpu.time", "cpu.mode", "user");
        assert_sum_point_attr(metrics, "system.cpu.time", "cpu.mode", "iowait");
        assert_metric_shape(metrics, "system.cpu.utilization", "1", None);
        assert_first_point_attr(metrics, "system.cpu.utilization", "cpu.mode", "user");
        assert_metric_shape(metrics, "system.cpu.logical.count", "{cpu}", Some(false));
        assert_metric_shape(metrics, "system.cpu.physical.count", "{cpu}", Some(false));
        assert_metric_shape(metrics, "system.cpu.frequency", "Hz", None);
        assert_first_point_attr(metrics, "system.cpu.frequency", "cpu.logical_number", "0");
        assert_metric_shape(metrics, "system.memory.usage", "By", Some(false));
        assert_first_point_attr(
            metrics,
            "system.memory.usage",
            "system.memory.state",
            "used",
        );
        assert_metric_shape(metrics, "system.memory.utilization", "1", None);
        assert_metric_shape(metrics, "system.memory.linux.available", "By", Some(false));
        assert_metric_shape(metrics, "system.memory.linux.slab.usage", "By", Some(false));
        assert_metric_shape(metrics, "system.memory.limit", "By", Some(false));
        assert_metric_shape(metrics, "system.memory.linux.shared", "By", Some(false));
        assert_metric_shape(
            metrics,
            "system.memory.linux.hugepages.limit",
            "{page}",
            Some(false),
        );
        assert_metric_shape(
            metrics,
            "system.memory.linux.hugepages.page_size",
            "By",
            Some(false),
        );
        assert_metric_shape(
            metrics,
            "system.memory.linux.hugepages.reserved",
            "{page}",
            Some(false),
        );
        assert_metric_shape(
            metrics,
            "system.memory.linux.hugepages.surplus",
            "{page}",
            Some(false),
        );
        assert_metric_shape(
            metrics,
            "system.memory.linux.hugepages.usage",
            "{page}",
            Some(false),
        );
        assert_first_point_attr(
            metrics,
            "system.memory.linux.hugepages.usage",
            "system.memory.linux.hugepages.state",
            "used",
        );
        assert_metric_shape(
            metrics,
            "system.memory.linux.hugepages.utilization",
            "1",
            None,
        );
        assert_metric_shape(metrics, "system.uptime", "s", None);
        assert_metric_shape(metrics, "system.paging.faults", "{fault}", Some(true));
        assert_first_point_attr(
            metrics,
            "system.paging.faults",
            "system.paging.fault.type",
            "minor",
        );
        assert_metric_shape(
            metrics,
            "system.paging.operations",
            "{operation}",
            Some(true),
        );
        assert_metric_shape(metrics, "system.paging.usage", "By", Some(false));
        assert_first_point_attr(metrics, "system.paging.usage", "system.device", "/dev/swap");
        assert_metric_shape(metrics, "system.paging.utilization", "1", None);
        assert_metric_shape(metrics, "system.process.count", "{process}", Some(false));
        assert_metric_shape(metrics, "system.process.created", "{process}", Some(true));
        assert_metric_shape(metrics, "system.disk.io", "By", Some(true));
        assert_first_point_attr(metrics, "system.disk.io", "disk.io.direction", "read");
        assert_metric_shape(metrics, "system.disk.operations", "{operation}", Some(true));
        assert_metric_shape(metrics, "system.disk.io_time", "s", Some(true));
        assert_first_point_attr(metrics, "system.disk.io_time", "system.device", "sda");
        assert_metric_shape(metrics, "system.disk.operation_time", "s", Some(true));
        assert_metric_shape(metrics, "system.disk.merged", "{operation}", Some(true));
        assert_metric_shape(metrics, "system.disk.limit", "By", Some(false));
        assert_first_point_attr(metrics, "system.disk.limit", "system.device", "sda");
        assert_metric_shape(metrics, "system.filesystem.usage", "By", Some(false));
        assert_first_point_attr(
            metrics,
            "system.filesystem.usage",
            "system.filesystem.state",
            "used",
        );
        assert_metric_shape(metrics, "system.filesystem.utilization", "1", None);
        assert_metric_shape(metrics, "system.filesystem.limit", "By", Some(false));
        assert_no_first_point_attr(
            metrics,
            "system.filesystem.limit",
            "system.filesystem.state",
        );
        assert_metric_shape(metrics, "system.network.io", "By", Some(true));
        assert_first_point_attr(
            metrics,
            "system.network.io",
            "network.interface.name",
            "eth0",
        );
        assert_metric_shape(
            metrics,
            "system.network.packet.count",
            "{packet}",
            Some(true),
        );
        assert_first_point_attr(
            metrics,
            "system.network.packet.count",
            "system.device",
            "eth0",
        );
        assert_metric_shape(
            metrics,
            "system.network.packet.dropped",
            "{packet}",
            Some(true),
        );
        assert_first_point_attr(
            metrics,
            "system.network.packet.dropped",
            "network.interface.name",
            "eth0",
        );
        assert_metric_shape(metrics, "system.network.errors", "{error}", Some(true));
    }

    #[cfg(feature = "dev-tools")]
    #[test]
    #[ignore = "dev-only semconv drift check; may access a local or remote semantic-conventions registry"]
    fn emitted_phase1_metric_shapes_match_weaver_semconv() {
        let registry = load_semconv_registry();
        let semconv_shapes = semconv_system_metric_shapes(&registry);
        let emitted_shapes = emitted_phase1_metric_shapes();

        for (name, emitted) in emitted_shapes {
            let semconv = semconv_shapes
                .get(&name)
                .unwrap_or_else(|| panic!("missing semconv metric {name}"));

            assert_eq!(emitted.unit, semconv.unit, "unit mismatch for {name}");
            assert_eq!(
                emitted.monotonic, semconv.monotonic,
                "instrument/temporality mismatch for {name}"
            );

            for attr in &semconv.attributes {
                if is_intentional_semconv_attribute_gap(name.as_str(), attr.as_str()) {
                    continue;
                }
                assert!(
                    emitted.attributes.contains(attr),
                    "missing semconv attribute {attr} on {name}"
                );
            }
        }
    }

    #[test]
    fn projection_uses_counter_start_overrides_for_reset_series() {
        let request = HostSnapshot {
            now_unix_nano: 2_000,
            start_time_unix_nano: 1_000,
            counter_starts: CounterStarts {
                entries: vec![(counter_key("system.process.created", ""), 1_500)],
            },
            processes: Some(ProcessStats {
                created: 99,
                ..ProcessStats::default()
            }),
            ..HostSnapshot::default()
        }
        .into_export_request();

        let metrics = &request.resource_metrics[0].scope_metrics[0].metrics;
        assert_first_sum_point_start(metrics, "system.process.created", 1_500);
    }

    #[test]
    fn counter_tracker_rebaselines_reset_series_only() {
        let mut tracker = CounterTracker::default();
        let disks = vec![DiskStats {
            name: "sda".to_owned(),
            read_bytes: 100,
            write_bytes: 200,
            ..DiskStats::default()
        }];
        let starts = tracker.snapshot(10, 20, None, None, None, &disks, &[]);

        assert_eq!(starts.get_joined("system.disk.io", "sda", "read", 10), 10);
        assert_eq!(starts.get_joined("system.disk.io", "sda", "write", 10), 10);

        let disks = vec![DiskStats {
            name: "sda".to_owned(),
            read_bytes: 50,
            write_bytes: 250,
            ..DiskStats::default()
        }];
        let starts = tracker.snapshot(10, 30, None, None, None, &disks, &[]);

        assert_eq!(starts.get_joined("system.disk.io", "sda", "read", 10), 30);
        assert_eq!(starts.get_joined("system.disk.io", "sda", "write", 10), 10);
    }

    #[test]
    fn scrape_due_emits_successful_families_after_partial_read_error() {
        let root = tempfile::tempdir().expect("tempdir");
        let proc = root.path().join("proc");
        std::fs::create_dir(&proc).expect("proc dir");
        std::fs::write(
            proc.join("meminfo"),
            "MemTotal: 1000 kB\nMemFree: 100 kB\nMemAvailable: 200 kB\n",
        )
        .expect("meminfo");
        let mut source = ProcfsSource::new(
            Some(root.path()),
            ProcfsConfig {
                cpu: false,
                memory: true,
                paging: false,
                system: false,
                disk: true,
                filesystem: false,
                network: false,
                processes: false,
                cpu_utilization: false,
                memory_limit: false,
                memory_shared: false,
                memory_hugepages: false,
                disk_limit: false,
                filesystem_include_virtual: false,
                filesystem_limit: false,
                filesystem_include_devices: None,
                filesystem_exclude_devices: None,
                filesystem_include_fs_types: None,
                filesystem_exclude_fs_types: None,
                filesystem_include_mount_points: None,
                filesystem_exclude_mount_points: None,
                disk_include: None,
                disk_exclude: None,
                network_include: None,
                network_exclude: None,
                validation: HostViewValidationMode::None,
            },
        )
        .expect("source");

        let scrape = source
            .scrape_due(ProcfsFamilies {
                memory: true,
                disk: true,
                ..ProcfsFamilies::default()
            })
            .expect("partial scrape");

        assert_eq!(scrape.partial_errors, 1);
        assert!(scrape.snapshot.memory.is_some());
        assert!(scrape.snapshot.disks.is_empty());
    }

    #[test]
    fn scrape_due_fails_when_all_due_families_fail() {
        let root = tempfile::tempdir().expect("tempdir");
        let mut source = ProcfsSource::new(
            Some(root.path()),
            ProcfsConfig {
                cpu: false,
                memory: true,
                paging: false,
                system: false,
                disk: false,
                filesystem: false,
                network: false,
                processes: false,
                cpu_utilization: false,
                memory_limit: false,
                memory_shared: false,
                memory_hugepages: false,
                disk_limit: false,
                filesystem_include_virtual: false,
                filesystem_limit: false,
                filesystem_include_devices: None,
                filesystem_exclude_devices: None,
                filesystem_include_fs_types: None,
                filesystem_exclude_fs_types: None,
                filesystem_include_mount_points: None,
                filesystem_exclude_mount_points: None,
                disk_include: None,
                disk_exclude: None,
                network_include: None,
                network_exclude: None,
                validation: HostViewValidationMode::None,
            },
        )
        .expect("source");

        assert!(
            source
                .scrape_due(ProcfsFamilies {
                    memory: true,
                    ..ProcfsFamilies::default()
                })
                .is_err()
        );
    }

    #[test]
    fn scrape_due_reads_opt_in_disk_limit_from_sysfs() {
        let root = tempfile::tempdir().expect("tempdir");
        let proc = root.path().join("proc");
        let sys_sda = root.path().join("sys/block/sda");
        std::fs::create_dir(&proc).expect("proc dir");
        std::fs::create_dir_all(&sys_sda).expect("sys block dir");
        std::fs::write(
            proc.join("diskstats"),
            "8 0 sda 1 0 2 3 4 0 5 6 0 0 0 0 0 0 0 0\n",
        )
        .expect("diskstats");
        std::fs::write(sys_sda.join("size"), "4096\n").expect("disk size");
        let mut source = ProcfsSource::new(
            Some(root.path()),
            ProcfsConfig {
                cpu: false,
                memory: false,
                paging: false,
                system: false,
                disk: true,
                filesystem: false,
                network: false,
                processes: false,
                cpu_utilization: false,
                memory_limit: false,
                memory_shared: false,
                memory_hugepages: false,
                disk_limit: true,
                filesystem_include_virtual: false,
                filesystem_limit: false,
                filesystem_include_devices: None,
                filesystem_exclude_devices: None,
                filesystem_include_fs_types: None,
                filesystem_exclude_fs_types: None,
                filesystem_include_mount_points: None,
                filesystem_exclude_mount_points: None,
                disk_include: None,
                disk_exclude: None,
                network_include: None,
                network_exclude: None,
                validation: HostViewValidationMode::None,
            },
        )
        .expect("source");

        let scrape = source
            .scrape_due(ProcfsFamilies {
                disk: true,
                ..ProcfsFamilies::default()
            })
            .expect("disk scrape");

        assert_eq!(scrape.snapshot.disks.len(), 1);
        assert_eq!(
            scrape.snapshot.disks[0].limit_bytes,
            Some(4096 * DISKSTAT_SECTOR_BYTES)
        );
    }

    #[test]
    fn scrape_due_reads_filesystem_usage_from_mountinfo() {
        let root = tempfile::tempdir().expect("tempdir");
        let proc_one = root.path().join("proc/1");
        std::fs::create_dir_all(&proc_one).expect("proc one dir");
        std::fs::write(
            proc_one.join("mountinfo"),
            "36 25 8:1 / / rw,relatime - ext4 /dev/sda1 rw\n",
        )
        .expect("mountinfo");
        let mut source = ProcfsSource::new(
            Some(root.path()),
            ProcfsConfig {
                cpu: false,
                memory: false,
                paging: false,
                system: false,
                disk: false,
                filesystem: true,
                network: false,
                processes: false,
                cpu_utilization: false,
                memory_limit: false,
                memory_shared: false,
                memory_hugepages: false,
                disk_limit: false,
                filesystem_include_virtual: false,
                filesystem_limit: true,
                filesystem_include_devices: None,
                filesystem_exclude_devices: None,
                filesystem_include_fs_types: None,
                filesystem_exclude_fs_types: None,
                filesystem_include_mount_points: None,
                filesystem_exclude_mount_points: None,
                disk_include: None,
                disk_exclude: None,
                network_include: None,
                network_exclude: None,
                validation: HostViewValidationMode::None,
            },
        )
        .expect("source");

        let scrape = source
            .scrape_due(ProcfsFamilies {
                filesystem: true,
                ..ProcfsFamilies::default()
            })
            .expect("filesystem scrape");

        assert_eq!(scrape.snapshot.filesystems.len(), 1);
        assert_eq!(scrape.snapshot.filesystems[0].device, "/dev/sda1");
        assert_eq!(scrape.snapshot.filesystems[0].mountpoint, "/");
        assert_eq!(scrape.snapshot.filesystems[0].fs_type, "ext4");
        assert!(scrape.snapshot.filesystems[0].limit_bytes.is_some());
    }

    #[test]
    fn cpu_parser_accepts_missing_newer_fields() {
        let cpu = parse_cpu_total("10 20 30 40", 10.0).expect("cpu row");
        assert_eq!(cpu.user, 1.0);
        assert_eq!(cpu.nice, 2.0);
        assert_eq!(cpu.system, 3.0);
        assert_eq!(cpu.idle, 4.0);
        assert_eq!(cpu.steal, 0.0);
    }

    #[test]
    fn cpu_parser_removes_guest_from_user_and_nice() {
        let cpu = parse_cpu_total("100 50 30 40 5 2 3 7 10 4", 10.0).expect("cpu row");
        assert_eq!(cpu.user, 9.0);
        assert_eq!(cpu.nice, 4.6);
        assert_eq!(cpu.interrupt, 0.5);
    }

    #[test]
    fn cpu_utilization_uses_counter_deltas() {
        let utilization = cpu_utilization(
            CpuTimes {
                user: 1.0,
                idle: 1.0,
                ..CpuTimes::default()
            },
            CpuTimes {
                user: 3.0,
                idle: 2.0,
                ..CpuTimes::default()
            },
        )
        .expect("utilization");

        assert_eq!(utilization.user, 2.0 / 3.0);
        assert_eq!(utilization.idle, 1.0 / 3.0);
    }

    #[test]
    fn cpu_utilization_skips_counter_resets() {
        assert!(
            cpu_utilization(
                CpuTimes {
                    user: 2.0,
                    ..CpuTimes::default()
                },
                CpuTimes {
                    user: 1.0,
                    ..CpuTimes::default()
                },
            )
            .is_none()
        );
    }

    #[test]
    fn clock_ticks_per_second_uses_positive_system_value() {
        assert!(clock_ticks_per_second() > 0.0);
    }

    #[test]
    fn memavailable_fallback_uses_free_buffers_cached() {
        let memory =
            parse_meminfo("MemTotal: 1000 kB\nMemFree: 100 kB\nBuffers: 20 kB\nCached: 30 kB\n")
                .expect("memory");
        assert_eq!(memory.available, 150 * BYTES_PER_KIB);
        assert_eq!(memory.used, 850 * BYTES_PER_KIB);
    }

    #[test]
    fn meminfo_parser_reads_shared_memory() {
        let memory =
            parse_meminfo("MemTotal: 1000 kB\nMemFree: 100 kB\nShmem: 12 kB\n").expect("memory");
        assert_eq!(memory.shared, 12 * BYTES_PER_KIB);
    }

    #[test]
    fn meminfo_parser_reads_hugepage_stats() {
        let memory = parse_meminfo(
            "MemTotal: 1000 kB\n\
             MemFree: 100 kB\n\
             HugePages_Total: 8\n\
             HugePages_Free: 3\n\
             HugePages_Rsvd: 2\n\
             HugePages_Surp: 1\n\
             Hugepagesize: 2048 kB\n",
        )
        .expect("memory");

        assert_eq!(memory.hugepages.total, 8);
        assert_eq!(memory.hugepages.free, 3);
        assert_eq!(memory.hugepages.reserved, 2);
        assert_eq!(memory.hugepages.surplus, 1);
        assert_eq!(memory.hugepages.page_size_bytes, 2048 * BYTES_PER_KIB);
    }

    #[test]
    fn uptime_parser_reads_first_field() {
        assert_eq!(parse_uptime("123.45 67.89"), Some(123.45));
    }

    #[test]
    fn vmstat_parser_derives_minor_faults() {
        let paging = parse_vmstat("pgfault 100\npgmajfault 7\npswpin 3\npswpout 4\n");
        assert_eq!(paging.minor_faults, 93);
        assert_eq!(paging.major_faults, 7);
        assert_eq!(paging.swap_in, 3);
        assert_eq!(paging.swap_out, 4);
    }

    #[test]
    fn swaps_parser_reads_device_usage() {
        let swaps =
            parse_swaps("Filename Type Size Used Priority\n/dev/sda2 partition 200 50 -2\n");
        assert_eq!(swaps.len(), 1);
        assert_eq!(swaps[0].name, "/dev/sda2");
        assert_eq!(swaps[0].used, 50 * BYTES_PER_KIB);
        assert_eq!(swaps[0].free, 150 * BYTES_PER_KIB);
    }

    #[test]
    fn diskstats_parser_accepts_flush_columns() {
        let disks = parse_diskstats("8 0 sda 1 0 2 3 4 0 5 6 0 0 0 0 0 0 0 0\n", None, None);
        assert_eq!(disks.len(), 1);
        assert_eq!(disks[0].name, "sda");
        assert_eq!(disks[0].read_bytes, 1024);
        assert_eq!(disks[0].write_bytes, 2560);
    }

    #[test]
    fn diskstats_parser_applies_filters_before_parsing_values() {
        let exclude = CompiledFilter::compile(
            crate::receivers::host_metrics_receiver::MatchType::Glob,
            vec!["loop*".to_owned()],
        )
        .expect("valid")
        .expect("filter");
        let disks = parse_diskstats(
            "7 0 loop0 broken row\n8 0 sda 1 0 2 3 4 0 5 6 0 0 0 0 0 0 0 0\n",
            None,
            Some(&exclude),
        );

        assert_eq!(disks.len(), 1);
        assert_eq!(disks[0].name, "sda");
    }

    #[test]
    fn mountinfo_parser_skips_virtual_filesystems_by_default() {
        let mounts = parse_mountinfo(
            "36 25 8:1 / / rw,relatime - ext4 /dev/sda1 rw\n37 25 0:32 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw\n",
            false,
            true,
            FilesystemFilters::default(),
        );

        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].device, "/dev/sda1");
        assert_eq!(mounts[0].mountpoint, "/");
        assert_eq!(mounts[0].fs_type, "ext4");
        assert_eq!(mounts[0].mode, "rw");
        assert!(mounts[0].emit_limit);
    }

    #[test]
    fn mountinfo_parser_unescapes_paths() {
        let mounts = parse_mountinfo(
            "36 25 8:1 / /mnt/data\\040disk rw,relatime - ext4 /dev/disk\\040one rw\n",
            false,
            false,
            FilesystemFilters::default(),
        );

        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].device, "/dev/disk one");
        assert_eq!(mounts[0].mountpoint, "/mnt/data disk");
    }

    #[test]
    fn mountinfo_parser_applies_filesystem_filters() {
        let include_mounts = CompiledFilter::compile(
            crate::receivers::host_metrics_receiver::MatchType::Glob,
            vec!["/data*".to_owned()],
        )
        .expect("valid")
        .expect("filter");
        let exclude_fs_types = CompiledFilter::compile(
            crate::receivers::host_metrics_receiver::MatchType::Strict,
            vec!["xfs".to_owned()],
        )
        .expect("valid")
        .expect("filter");
        let mounts = parse_mountinfo(
            "36 25 8:1 / / rw,relatime - ext4 /dev/sda1 rw\n37 25 8:2 / /data rw,relatime - ext4 /dev/sdb1 rw\n38 25 8:3 / /data2 rw,relatime - xfs /dev/sdc1 rw\n",
            false,
            false,
            FilesystemFilters {
                include_mount_points: Some(&include_mounts),
                exclude_fs_types: Some(&exclude_fs_types),
                ..FilesystemFilters::default()
            },
        );

        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].device, "/dev/sdb1");
        assert_eq!(mounts[0].mountpoint, "/data");
    }

    #[test]
    fn netdev_parser_reads_device_counters() {
        let interfaces = parse_netdev(
            "Inter-| Receive | Transmit\n face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed\n eth0: 10 2 0 0 0 0 0 0 30 4 0 0 0 0 0 0\n",
            None,
            None,
        );
        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].name, "eth0");
        assert_eq!(interfaces[0].rx_bytes, 10);
        assert_eq!(interfaces[0].tx_packets, 4);
    }

    #[test]
    fn netdev_parser_applies_interface_filters() {
        let include = CompiledFilter::compile(
            crate::receivers::host_metrics_receiver::MatchType::Strict,
            vec!["eth0".to_owned()],
        )
        .expect("valid")
        .expect("filter");
        let interfaces = parse_netdev(
            "Inter-| Receive | Transmit\n face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed\n lo: 1 1 0 0 0 0 0 0 1 1 0 0 0 0 0 0\n eth0: 10 2 3 4 0 0 0 0 30 4 5 6 0 0 0 0\n",
            Some(&include),
            None,
        );

        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].name, "eth0");
        assert_eq!(interfaces[0].rx_errors, 3);
        assert_eq!(interfaces[0].tx_dropped, 6);
    }

    #[test]
    fn root_path_uses_host_pid_one_netdev() {
        let paths = ProcfsPaths::new(Some(Path::new("/host")));
        assert_eq!(paths.net_dev, PathBuf::from("/host/proc/1/net/dev"));
        assert_eq!(paths.mountinfo, PathBuf::from("/host/proc/1/mountinfo"));
    }

    #[test]
    fn root_slash_uses_current_proc_netdev() {
        let paths = ProcfsPaths::new(Some(Path::new("/")));
        assert_eq!(paths.net_dev, PathBuf::from("/proc/net/dev"));
        assert_eq!(paths.mountinfo, PathBuf::from("/proc/self/mountinfo"));
    }

    #[test]
    fn host_arch_uses_semconv_values() {
        if let Some(arch) = host_arch() {
            assert!(matches!(
                arch,
                "amd64" | "arm32" | "arm64" | "ppc32" | "ppc64" | "x86"
            ));
        }
    }

    #[cfg(feature = "dev-tools")]
    #[derive(Debug)]
    struct MetricShape {
        unit: String,
        monotonic: Option<bool>,
        attributes: BTreeSet<String>,
    }

    #[cfg(feature = "dev-tools")]
    fn load_semconv_registry() -> ResolvedRegistry {
        let registry_path = std::env::var("OTAP_HOST_METRICS_SEMCONV_REGISTRY")
            .map(|path| {
                path.parse::<VirtualDirectoryPath>()
                    .expect("valid OTAP_HOST_METRICS_SEMCONV_REGISTRY")
            })
            .unwrap_or_else(|_| VirtualDirectoryPath::GitRepo {
                url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
                sub_folder: Some("model".to_owned()),
                refspec: None,
            });

        let registry_repo =
            RegistryRepo::try_new("main", &registry_path).expect("semantic convention registry");
        let registry = match SchemaResolver::load_semconv_repository(registry_repo, false) {
            WResult::Ok(registry) | WResult::OkWithNFEs(registry, _) => registry,
            WResult::FatalErr(err) => panic!("failed to load semantic convention registry: {err}"),
        };
        let resolved_schema = match SchemaResolver::resolve(registry, true) {
            WResult::Ok(schema) | WResult::OkWithNFEs(schema, _) => schema,
            WResult::FatalErr(err) => {
                panic!("failed to resolve semantic convention registry: {err}");
            }
        };

        ResolvedRegistry::try_from_resolved_registry(
            &resolved_schema.registry,
            resolved_schema.catalog(),
        )
        .expect("resolved semantic convention registry")
    }

    #[cfg(feature = "dev-tools")]
    fn semconv_system_metric_shapes(registry: &ResolvedRegistry) -> BTreeMap<String, MetricShape> {
        registry
            .groups
            .iter()
            .filter(|group| group.r#type == GroupType::Metric)
            .filter_map(|group| {
                let name = group.metric_name.as_ref()?;
                if !name.starts_with("system.") {
                    return None;
                }

                let monotonic = match group.instrument.as_ref()? {
                    InstrumentSpec::Counter => Some(true),
                    InstrumentSpec::UpDownCounter => Some(false),
                    InstrumentSpec::Gauge | InstrumentSpec::Histogram => None,
                };
                let attributes = group
                    .attributes
                    .iter()
                    .filter(|attr| !is_opt_in_requirement(&attr.requirement_level))
                    .map(|attr| attr.name.clone())
                    .collect();

                Some((
                    name.clone(),
                    MetricShape {
                        unit: group.unit.clone().unwrap_or_default(),
                        monotonic,
                        attributes,
                    },
                ))
            })
            .collect()
    }

    #[cfg(feature = "dev-tools")]
    fn is_opt_in_requirement(requirement_level: &RequirementLevel) -> bool {
        matches!(
            requirement_level,
            RequirementLevel::Basic(BasicRequirementLevelSpec::OptIn)
                | RequirementLevel::OptIn { .. }
        )
    }

    #[cfg(feature = "dev-tools")]
    fn emitted_phase1_metric_shapes() -> BTreeMap<String, MetricShape> {
        let metrics = projection_fixture_metrics();
        metrics
            .iter()
            .map(|metric| {
                let (monotonic, points) = match metric.data.as_ref().expect("metric data") {
                    metric::Data::Sum(sum) => (Some(sum.is_monotonic), &sum.data_points),
                    metric::Data::Gauge(gauge) => (None, &gauge.data_points),
                    _ => panic!("unsupported metric data for {}", metric.name),
                };
                let attributes = points
                    .iter()
                    .flat_map(|point| point.attributes.iter())
                    .map(|attr| attr.key.clone())
                    .collect();
                (
                    metric.name.clone(),
                    MetricShape {
                        unit: metric.unit.clone(),
                        monotonic,
                        attributes,
                    },
                )
            })
            .collect()
    }

    #[cfg(feature = "dev-tools")]
    fn is_intentional_semconv_attribute_gap(name: &str, attr: &str) -> bool {
        matches!(
            (name, attr),
            ("system.paging.operations", "system.paging.fault.type")
        )
    }

    fn projection_fixture_request() -> ExportMetricsServiceRequest {
        HostSnapshot {
            now_unix_nano: 2_000,
            start_time_unix_nano: 1_000,
            counter_starts: CounterStarts::default(),
            memory_limit: true,
            memory_shared: true,
            memory_hugepages: true,
            cpu: Some(CpuTimes {
                user: 1.0,
                nice: 2.0,
                system: 3.0,
                idle: 4.0,
                wait: 5.0,
                interrupt: 6.0,
                steal: 7.0,
            }),
            cpu_utilization: Some(CpuTimes {
                user: 0.1,
                nice: 0.1,
                system: 0.2,
                idle: 0.3,
                wait: 0.1,
                interrupt: 0.1,
                steal: 0.1,
            }),
            cpuinfo: CpuInfo {
                logical_count: 2,
                physical_count: 1,
                frequencies_hz: vec![2_400_000_000.0],
            },
            memory: Some(MemoryStats {
                total: 100,
                used: 80,
                free: 10,
                available: 20,
                cached: 5,
                buffered: 5,
                shared: 7,
                slab_reclaimable: 3,
                slab_unreclaimable: 2,
                hugepages: HugepageStats {
                    total: 10,
                    free: 4,
                    reserved: 2,
                    surplus: 1,
                    page_size_bytes: 2 * BYTES_PER_KIB,
                },
            }),
            uptime_seconds: Some(42.0),
            paging: Some(PagingStats {
                minor_faults: 9,
                major_faults: 1,
                swap_in: 2,
                swap_out: 3,
            }),
            swaps: vec![SwapStats {
                name: "/dev/swap".to_owned(),
                size: 100,
                used: 25,
                free: 75,
            }],
            processes: Some(ProcessStats {
                running: 4,
                blocked: 1,
                created: 99,
            }),
            disks: vec![DiskStats {
                name: "sda".to_owned(),
                limit_bytes: Some(123),
                read_bytes: 10,
                write_bytes: 20,
                read_ops: 1,
                write_ops: 2,
                read_merged: 3,
                write_merged: 4,
                read_time_seconds: 0.5,
                write_time_seconds: 0.6,
                io_time_seconds: 0.7,
            }],
            filesystems: vec![FilesystemStats {
                device: "/dev/sda1".to_owned(),
                mountpoint: "/".to_owned(),
                fs_type: "ext4".to_owned(),
                mode: "rw",
                used: 60,
                free: 30,
                reserved: 10,
                limit_bytes: Some(100),
            }],
            networks: vec![NetworkStats {
                name: "eth0".to_owned(),
                rx_bytes: 10,
                tx_bytes: 20,
                rx_packets: 1,
                tx_packets: 2,
                rx_errors: 3,
                tx_errors: 4,
                rx_dropped: 5,
                tx_dropped: 6,
            }],
            resource: HostResource {
                host_id: Some("host-id".to_owned()),
                host_name: Some("host-name".to_owned()),
                host_arch: Some("amd64"),
            },
        }
        .into_export_request()
    }

    #[cfg(feature = "dev-tools")]
    fn projection_fixture_metrics() -> Vec<Metric> {
        projection_fixture_request()
            .resource_metrics
            .into_iter()
            .next()
            .expect("resource metrics")
            .scope_metrics
            .into_iter()
            .next()
            .expect("scope metrics")
            .metrics
    }

    fn assert_metric_shape(
        metrics: &[Metric],
        name: &'static str,
        unit: &'static str,
        monotonic_sum: Option<bool>,
    ) {
        let metric = metric_by_name(metrics, name);
        assert_eq!(metric.unit, unit);
        match metric.data.as_ref().expect("metric data") {
            metric::Data::Sum(sum) => {
                let expected_monotonic =
                    monotonic_sum.unwrap_or_else(|| panic!("{name} should be a gauge"));
                assert_eq!(
                    sum.aggregation_temporality,
                    AggregationTemporality::Cumulative as i32
                );
                assert_eq!(sum.is_monotonic, expected_monotonic);
                assert!(
                    sum.data_points
                        .iter()
                        .all(|point| point.start_time_unix_nano == 1_000)
                );
            }
            metric::Data::Gauge(gauge) => {
                assert!(monotonic_sum.is_none(), "{name} should be a cumulative sum");
                assert!(
                    gauge
                        .data_points
                        .iter()
                        .all(|point| point.start_time_unix_nano == 0)
                );
            }
            _ => panic!("unexpected data kind for {name}"),
        }
    }

    fn assert_first_point_attr(
        metrics: &[Metric],
        name: &'static str,
        key: &'static str,
        value: &'static str,
    ) {
        let metric = metric_by_name(metrics, name);
        let point = match metric.data.as_ref().expect("metric data") {
            metric::Data::Sum(sum) => sum.data_points.first(),
            metric::Data::Gauge(gauge) => gauge.data_points.first(),
            _ => None,
        }
        .expect("data point");
        assert_has_attr(&point.attributes, key, value);
    }

    fn assert_sum_point_attr(
        metrics: &[Metric],
        name: &'static str,
        key: &'static str,
        value: &'static str,
    ) {
        let metric = metric_by_name(metrics, name);
        let metric::Data::Sum(sum) = metric.data.as_ref().expect("metric data") else {
            panic!("{name} should be a cumulative sum");
        };
        assert!(
            sum.data_points
                .iter()
                .any(|point| has_attr(&point.attributes, key, value)),
            "missing point attribute {key}={value}"
        );
    }

    fn assert_no_first_point_attr(metrics: &[Metric], name: &'static str, key: &'static str) {
        let metric = metric_by_name(metrics, name);
        let point = match metric.data.as_ref().expect("metric data") {
            metric::Data::Sum(sum) => sum.data_points.first(),
            metric::Data::Gauge(gauge) => gauge.data_points.first(),
            _ => None,
        }
        .expect("data point");
        assert!(
            !point.attributes.iter().any(|attr| attr.key == key),
            "unexpected attribute {key}"
        );
    }

    fn assert_first_sum_point_start(metrics: &[Metric], name: &'static str, expected_start: u64) {
        let metric = metric_by_name(metrics, name);
        let metric::Data::Sum(sum) = metric.data.as_ref().expect("metric data") else {
            panic!("{name} should be a cumulative sum");
        };
        let point = sum.data_points.first().expect("data point");
        assert_eq!(point.start_time_unix_nano, expected_start);
    }

    fn metric_by_name<'a>(metrics: &'a [Metric], name: &'static str) -> &'a Metric {
        metrics
            .iter()
            .find(|metric| metric.name == name)
            .unwrap_or_else(|| panic!("missing metric {name}"))
    }

    fn assert_has_attr(attributes: &[KeyValue], key: &'static str, value: &'static str) {
        assert!(
            has_attr(attributes, key, value),
            "missing attribute {key}={value}"
        );
    }

    fn has_attr(attributes: &[KeyValue], key: &'static str, value: &'static str) -> bool {
        attributes.iter().any(|attr| {
            attr.key == key
                && matches!(
                    attr.value.as_ref().and_then(|value| value.value.as_ref()),
                    Some(any_value::Value::StringValue(actual)) if actual == value
                )
        })
    }
}
