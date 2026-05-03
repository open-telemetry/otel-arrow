// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux procfs-backed host metric source.

use crate::receivers::host_metrics_receiver::CompiledFilter;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue, any_value,
};
use otap_df_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    metric, number_data_point,
};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const NANOS_PER_SEC: u64 = 1_000_000_000;
const BYTES_PER_KIB: u64 = 1024;
const DISKSTAT_SECTOR_BYTES: u64 = 512;

/// Procfs-backed source for host metrics.
pub struct ProcfsSource {
    paths: ProcfsPaths,
    config: ProcfsConfig,
    buf: String,
    clk_tck: f64,
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
    /// Network metrics.
    pub network: bool,
    /// Process summary metrics.
    pub processes: bool,
    /// Disk include filter.
    pub disk_include: Option<CompiledFilter>,
    /// Disk exclude filter.
    pub disk_exclude: Option<CompiledFilter>,
    /// Network include filter.
    pub network_include: Option<CompiledFilter>,
    /// Network exclude filter.
    pub network_exclude: Option<CompiledFilter>,
}

impl ProcfsSource {
    /// Creates a procfs source rooted at `/` or at a host root bind mount.
    pub fn new(root_path: Option<&Path>, config: ProcfsConfig) -> io::Result<Self> {
        let source = Self {
            paths: ProcfsPaths::new(root_path),
            config,
            buf: String::with_capacity(16 * 1024),
            clk_tck: clock_ticks_per_second(),
        };
        source.validate_selected_paths()?;
        Ok(source)
    }

    /// Collects one host snapshot.
    pub fn scrape(&mut self) -> io::Result<HostSnapshot> {
        let now_unix_nano = now_unix_nano();
        let clk_tck = self.clk_tck;
        let needs_stat = self.config.cpu || self.config.system || self.config.processes;
        let stat = if needs_stat {
            let proc_stat = self.read_path(PathKind::Stat)?;
            parse_stat(proc_stat, clk_tck)
        } else {
            StatSnapshot::default()
        };

        let cpuinfo = if self.config.cpu {
            let cpuinfo = self.read_path(PathKind::Cpuinfo)?;
            parse_cpuinfo(cpuinfo)
        } else {
            CpuInfo::default()
        };

        let memory = if self.config.memory {
            let meminfo = self.read_path(PathKind::Meminfo)?;
            parse_meminfo(meminfo)
        } else {
            None
        };

        let uptime_seconds = if self.config.system {
            let uptime = self.read_path(PathKind::Uptime)?;
            parse_uptime(uptime)
        } else {
            None
        };

        let paging = if self.config.paging {
            let vmstat = self.read_path(PathKind::Vmstat)?;
            Some(parse_vmstat(vmstat))
        } else {
            None
        };

        let swaps = if self.config.paging {
            let swaps = self.read_path(PathKind::Swaps)?;
            parse_swaps(swaps)
        } else {
            Vec::new()
        };

        let disks = if self.config.disk {
            let disk_include = self.config.disk_include.clone();
            let disk_exclude = self.config.disk_exclude.clone();
            let diskstats = self.read_path(PathKind::Diskstats)?;
            parse_diskstats(diskstats, disk_include.as_ref(), disk_exclude.as_ref())
        } else {
            Vec::new()
        };

        let networks = if self.config.network {
            let network_include = self.config.network_include.clone();
            let network_exclude = self.config.network_exclude.clone();
            let netdev = self.read_path(PathKind::NetDev)?;
            parse_netdev(netdev, network_include.as_ref(), network_exclude.as_ref())
        } else {
            Vec::new()
        };

        let resource = self.read_resource();

        Ok(HostSnapshot {
            now_unix_nano,
            start_time_unix_nano: stat.boot_time_unix_nano,
            cpu: stat.cpu,
            cpuinfo,
            memory,
            uptime_seconds,
            paging,
            swaps,
            processes: self.config.processes.then_some(stat.processes),
            disks,
            networks,
            resource,
        })
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
        if self.config.network {
            let _ = File::open(self.paths.path(PathKind::NetDev))?;
        }
        Ok(())
    }

    fn read_path(&mut self, kind: PathKind) -> io::Result<&str> {
        self.buf.clear();
        let mut file = File::open(self.paths.path(kind))?;
        let _ = file.read_to_string(&mut self.buf)?;
        Ok(self.buf.as_str())
    }

    fn read_resource(&mut self) -> HostResource {
        HostResource {
            host_id: self
                .read_trimmed_optional(PathKind::MachineId)
                .or_else(|| self.read_trimmed_optional(PathKind::DbusMachineId)),
            host_name: self.read_trimmed_optional(PathKind::Hostname),
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
    stat: PathBuf,
    cpuinfo: PathBuf,
    meminfo: PathBuf,
    uptime: PathBuf,
    vmstat: PathBuf,
    swaps: PathBuf,
    diskstats: PathBuf,
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
            stat: root.join("proc/stat"),
            cpuinfo: root.join("proc/cpuinfo"),
            meminfo: root.join("proc/meminfo"),
            uptime: root.join("proc/uptime"),
            vmstat: root.join("proc/vmstat"),
            swaps: root.join("proc/swaps"),
            diskstats: root.join("proc/diskstats"),
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
            PathKind::NetDev => &self.net_dev,
            PathKind::MachineId => &self.machine_id,
            PathKind::DbusMachineId => &self.dbus_machine_id,
            PathKind::Hostname => &self.hostname,
        }
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
    NetDev,
    MachineId,
    DbusMachineId,
    Hostname,
}

/// One host metrics snapshot.
#[derive(Default)]
pub struct HostSnapshot {
    now_unix_nano: u64,
    start_time_unix_nano: u64,
    cpu: Option<CpuTimes>,
    cpuinfo: CpuInfo,
    memory: Option<MemoryStats>,
    uptime_seconds: Option<f64>,
    paging: Option<PagingStats>,
    swaps: Vec<SwapStats>,
    processes: Option<ProcessStats>,
    disks: Vec<DiskStats>,
    networks: Vec<NetworkStats>,
    resource: HostResource,
}

impl HostSnapshot {
    /// Converts a snapshot into an OTLP metrics request.
    pub fn into_export_request(self) -> ExportMetricsServiceRequest {
        let mut metrics = Vec::with_capacity(64);
        let now = self.now_unix_nano;
        let start = self.start_time_unix_nano;

        if let Some(cpu) = self.cpu {
            push_sum_f64(
                &mut metrics,
                "system.cpu.time",
                "s",
                start,
                now,
                &[
                    ("user", cpu.user),
                    ("nice", cpu.nice),
                    ("system", cpu.system),
                    ("idle", cpu.idle),
                    ("wait", cpu.wait),
                    ("interrupt", cpu.interrupt),
                    ("steal", cpu.steal),
                ],
                "cpu.mode",
            );
        }

        if self.cpuinfo.logical_count != 0 {
            push_gauge_single_u64(
                &mut metrics,
                "system.cpu.logical.count",
                "{cpu}",
                now,
                self.cpuinfo.logical_count,
            );
        }
        if self.cpuinfo.physical_count != 0 {
            push_gauge_single_u64(
                &mut metrics,
                "system.cpu.physical.count",
                "{cpu}",
                now,
                self.cpuinfo.physical_count,
            );
        }
        push_cpu_frequency(&mut metrics, now, &self.cpuinfo.frequencies_hz);

        if let Some(memory) = self.memory {
            push_gauge_u64(
                &mut metrics,
                "system.memory.usage",
                "By",
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
            push_gauge_single_u64(
                &mut metrics,
                "system.memory.linux.available",
                "By",
                now,
                memory.available,
            );
            push_gauge_u64(
                &mut metrics,
                "system.memory.linux.slab.usage",
                "By",
                now,
                &[
                    ("reclaimable", memory.slab_reclaimable),
                    ("unreclaimable", memory.slab_unreclaimable),
                ],
                "system.memory.linux.slab.state",
            );
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
                &[("in", paging.swap_in), ("out", paging.swap_out)],
                "system.paging.direction",
            );
        }

        for swap in self.swaps {
            push_gauge_u64_with_device(
                &mut metrics,
                "system.paging.usage",
                "By",
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
            push_gauge_u64(
                &mut metrics,
                "system.process.count",
                "{process}",
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
                processes.created,
            );
        }

        for disk in self.disks {
            push_disk_sum(
                &mut metrics,
                "system.disk.io",
                "By",
                start,
                now,
                &disk,
                DiskProjection::Bytes,
            );
            push_disk_sum(
                &mut metrics,
                "system.disk.operations",
                "{operation}",
                start,
                now,
                &disk,
                DiskProjection::Operations,
            );
            push_disk_sum(
                &mut metrics,
                "system.disk.io_time",
                "s",
                start,
                now,
                &disk,
                DiskProjection::IoTime,
            );
            push_disk_sum(
                &mut metrics,
                "system.disk.operation_time",
                "s",
                start,
                now,
                &disk,
                DiskProjection::OperationTime,
            );
            push_disk_sum(
                &mut metrics,
                "system.disk.merged",
                "{operation}",
                start,
                now,
                &disk,
                DiskProjection::Merged,
            );
        }

        for network in self.networks {
            push_network_sum(
                &mut metrics,
                "system.network.io",
                "By",
                start,
                now,
                &network,
                NetworkProjection::Bytes,
            );
            push_network_sum(
                &mut metrics,
                "system.network.packet.count",
                "{packet}",
                start,
                now,
                &network,
                NetworkProjection::Packets,
            );
            push_network_sum(
                &mut metrics,
                "system.network.packet.dropped",
                "{packet}",
                start,
                now,
                &network,
                NetworkProjection::Dropped,
            );
            push_network_sum(
                &mut metrics,
                "system.network.errors",
                "{error}",
                start,
                now,
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
        attributes
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
    slab_reclaimable: u64,
    slab_unreclaimable: u64,
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
    let mut slab_reclaimable = 0;
    let mut slab_unreclaimable = 0;

    for line in input.lines() {
        let mut fields = line.split_whitespace();
        let Some(key) = fields.next() else {
            continue;
        };
        let value = fields.next().map(parse_u64).unwrap_or_default() * BYTES_PER_KIB;
        match key.trim_end_matches(':') {
            "MemTotal" => total = value,
            "MemFree" => free = value,
            "MemAvailable" => available = Some(value),
            "Buffers" => buffers = value,
            "Cached" => cached = value,
            "SReclaimable" => slab_reclaimable = value,
            "SUnreclaim" => slab_unreclaimable = value,
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
        slab_reclaimable,
        slab_unreclaimable,
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

fn push_gauge_u64(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    now: u64,
    values: &[(&'static str, u64)],
    attr_name: &'static str,
) {
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_i64(
            vec![kv_str(attr_name, state)],
            0,
            now,
            saturating_i64(*value),
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

fn push_gauge_u64_with_device(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    now: u64,
    device: &str,
    values: &[(&'static str, u64)],
    attr_name: &'static str,
) {
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_i64(
            vec![kv_str("system.device", device), kv_str(attr_name, state)],
            0,
            now,
            saturating_i64(*value),
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

fn push_gauge_single_u64(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    now: u64,
    value: u64,
) {
    metrics.push(Metric {
        name: name.to_owned(),
        description: String::new(),
        unit: unit.to_owned(),
        metadata: Vec::new(),
        data: Some(metric::Data::Gauge(Gauge {
            data_points: vec![number_point_i64(Vec::new(), 0, now, saturating_i64(value))],
        })),
    });
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
    values: &[(&'static str, f64)],
    attr_name: &'static str,
) {
    let mut points = Vec::with_capacity(values.len());
    for (state, value) in values {
        points.push(number_point_f64(
            vec![kv_str(attr_name, state)],
            start,
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
    push_sum_metric(metrics, name, unit, points);
}

fn push_sum_single_u64(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
    value: u64,
) {
    push_sum_metric(
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

fn push_disk_sum(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
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
                start,
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
        disk_number_point(&disk.name, "read", start, now, read),
        disk_number_point(&disk.name, "write", start, now, write),
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
    value: DiskValue,
) -> NumberDataPoint {
    let attributes = vec![
        kv_str("system.device", device),
        kv_str("disk.io.direction", direction),
    ];
    match value {
        DiskValue::Integer(value) => {
            number_point_i64(attributes, start, now, saturating_i64(value))
        }
        DiskValue::Float(value) => number_point_f64(attributes, start, now, value),
    }
}

fn push_network_sum(
    metrics: &mut Vec<Metric>,
    name: &'static str,
    unit: &'static str,
    start: u64,
    now: u64,
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
            start,
            now,
            saturating_i64(rx),
        ),
        number_point_i64(
            vec![
                kv_str(interface_attr, &network.name),
                kv_str("network.io.direction", "transmit"),
            ],
            start,
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
    metrics.push(Metric {
        name: name.to_owned(),
        description: String::new(),
        unit: unit.to_owned(),
        metadata: Vec::new(),
        data: Some(metric::Data::Sum(Sum {
            data_points: points,
            aggregation_temporality: AggregationTemporality::Cumulative.into(),
            is_monotonic: true,
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
    // Linux exposes CPU counters in USER_HZ. 100 is the common Linux value and
    // keeps this receiver dependency-light until a platform helper is added.
    100.0
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
    fn memavailable_fallback_uses_free_buffers_cached() {
        let memory =
            parse_meminfo("MemTotal: 1000 kB\nMemFree: 100 kB\nBuffers: 20 kB\nCached: 30 kB\n")
                .expect("memory");
        assert_eq!(memory.available, 150 * BYTES_PER_KIB);
        assert_eq!(memory.used, 850 * BYTES_PER_KIB);
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
    }

    #[test]
    fn root_slash_uses_current_proc_netdev() {
        let paths = ProcfsPaths::new(Some(Path::new("/")));
        assert_eq!(paths.net_dev, PathBuf::from("/proc/net/dev"));
    }
}
