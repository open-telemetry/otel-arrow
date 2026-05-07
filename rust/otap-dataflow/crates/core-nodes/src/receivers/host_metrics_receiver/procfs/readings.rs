// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::receivers::host_metrics_receiver::CompiledFilter;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{BYTES_PER_KIB, DISKSTAT_SECTOR_BYTES, NANOS_PER_SEC};

#[derive(Copy, Clone, Default)]
pub(super) struct CpuTimes {
    pub(super) user: f64,
    pub(super) nice: f64,
    pub(super) system: f64,
    pub(super) idle: f64,
    pub(super) wait: f64,
    pub(super) interrupt: f64,
    pub(super) steal: f64,
}

#[derive(Clone, Default)]
pub(super) struct CpuInfo {
    pub(super) logical_count: u64,
    pub(super) physical_count: u64,
    pub(super) frequencies_hz: Vec<f64>,
}

#[derive(Copy, Clone, Default)]
pub(super) struct StatSnapshot {
    pub(super) boot_time_unix_nano: u64,
    pub(super) cpu: Option<CpuTimes>,
    pub(super) processes: ProcessStats,
}

#[derive(Copy, Clone, Default)]
pub(super) struct MemoryStats {
    pub(super) total: u64,
    pub(super) used: u64,
    pub(super) free: u64,
    pub(super) available: u64,
    pub(super) has_available: bool,
    pub(super) cached: u64,
    pub(super) buffered: u64,
    pub(super) shared: u64,
    pub(super) slab_reclaimable: u64,
    pub(super) slab_unreclaimable: u64,
    pub(super) hugepages: HugepageStats,
}

#[derive(Copy, Clone, Default)]
pub(super) struct HugepageStats {
    pub(super) total: u64,
    pub(super) free: u64,
    pub(super) reserved: u64,
    pub(super) surplus: u64,
    pub(super) page_size_bytes: u64,
}

#[derive(Copy, Clone, Default)]
pub(super) struct PagingStats {
    pub(super) minor_faults: u64,
    pub(super) major_faults: u64,
    pub(super) page_in: u64,
    pub(super) page_out: u64,
    pub(super) swap_in: u64,
    pub(super) swap_out: u64,
}

#[derive(Default)]
pub(super) struct SwapStats {
    pub(super) name: String,
    pub(super) size: u64,
    pub(super) used: u64,
    pub(super) free: u64,
}

#[derive(Copy, Clone, Default)]
pub(super) struct ProcessStats {
    pub(super) running: u64,
    pub(super) blocked: u64,
    pub(super) created: u64,
}

#[derive(Default)]
pub(super) struct DiskStats {
    pub(super) name: String,
    pub(super) limit_bytes: Option<u64>,
    pub(super) read_bytes: u64,
    pub(super) write_bytes: u64,
    pub(super) read_ops: u64,
    pub(super) write_ops: u64,
    pub(super) read_merged: u64,
    pub(super) write_merged: u64,
    pub(super) read_time_seconds: f64,
    pub(super) write_time_seconds: f64,
    pub(super) io_time_seconds: f64,
}

#[derive(Default)]
pub(super) struct FilesystemStats {
    pub(super) device: String,
    pub(super) mountpoint: String,
    pub(super) fs_type: String,
    pub(super) mode: &'static str,
    pub(super) used: u64,
    pub(super) free: u64,
    pub(super) reserved: u64,
    pub(super) limit_bytes: Option<u64>,
}

/// Dedicated worker thread for `statvfs` calls.
///
/// Intentionally not using `tokio::fs` / `spawn_blocking`: `statvfs` can block
/// indefinitely on unhealthy remote or FUSE mounts, and Tokio cannot cancel an
/// in-flight blocking task. With `spawn_blocking`, repeated scrapes during a
/// hang could accumulate stuck tasks on Tokio's global blocking pool, affecting
/// unrelated callers.
///
/// The dedicated thread plus `sync_channel(1)` caps the blast radius at one
/// worker thread and at most one queued request per receiver. Callers still use
/// a per-mount timeout, and once the queue is full, later scrapes fail fast
/// instead of creating more blocking work. This fits the dfengine
/// thread-per-core / core-locality model better than offloading to Tokio's
/// shared blocking pool.
pub(super) struct FilesystemStatWorker {
    tx: mpsc::SyncSender<FilesystemStatRequest>,
}

pub(super) struct FilesystemStatRequest {
    path: PathBuf,
    response: mpsc::Sender<io::Result<FilesystemStat>>,
}

pub(super) struct FilesystemStat {
    pub(super) total_bytes: u64,
    pub(super) free_bytes: u64,
    pub(super) available_bytes: u64,
}

impl FilesystemStatWorker {
    pub(super) fn new() -> io::Result<Self> {
        let (tx, rx) = mpsc::sync_channel::<FilesystemStatRequest>(1);
        let _handle = std::thread::Builder::new()
            .name("host-metrics-statvfs".to_owned())
            .spawn(move || {
                while let Ok(request) = rx.recv() {
                    let result = statvfs_bytes(&request.path);
                    let _ = request.response.send(result);
                }
            })
            .map_err(io::Error::other)?;
        Ok(Self { tx })
    }

    #[cfg(test)]
    pub(super) fn disconnected_for_test() -> Self {
        let (tx, rx) = mpsc::sync_channel::<FilesystemStatRequest>(1);
        drop(rx);
        Self { tx }
    }

    pub(super) fn statvfs(&self, path: PathBuf, timeout: Duration) -> io::Result<FilesystemStat> {
        let (response, rx) = mpsc::channel();
        self.tx
            .try_send(FilesystemStatRequest { path, response })
            .map_err(|err| match err {
                mpsc::TrySendError::Full(_) => {
                    io::Error::new(io::ErrorKind::TimedOut, "statvfs worker is busy")
                }
                mpsc::TrySendError::Disconnected(_) => {
                    io::Error::new(io::ErrorKind::BrokenPipe, "statvfs worker stopped")
                }
            })?;
        rx.recv_timeout(timeout).map_err(|err| match err {
            mpsc::RecvTimeoutError::Timeout => {
                io::Error::new(io::ErrorKind::TimedOut, "statvfs timed out")
            }
            mpsc::RecvTimeoutError::Disconnected => {
                io::Error::new(io::ErrorKind::BrokenPipe, "statvfs worker stopped")
            }
        })?
    }
}

fn statvfs_bytes(path: &Path) -> io::Result<FilesystemStat> {
    let stat = nix::sys::statvfs::statvfs(path).map_err(io::Error::other)?;
    let block_size = stat.fragment_size();
    Ok(FilesystemStat {
        total_bytes: stat.blocks().saturating_mul(block_size),
        free_bytes: stat.blocks_free().saturating_mul(block_size),
        available_bytes: stat.blocks_available().saturating_mul(block_size),
    })
}

#[derive(Default)]
pub(super) struct NetworkStats {
    pub(super) name: String,
    pub(super) rx_bytes: u64,
    pub(super) tx_bytes: u64,
    pub(super) rx_packets: u64,
    pub(super) tx_packets: u64,
    pub(super) rx_errors: u64,
    pub(super) tx_errors: u64,
    pub(super) rx_dropped: u64,
    pub(super) tx_dropped: u64,
}

pub(super) fn parse_stat(input: &str, clk_tck: f64) -> StatSnapshot {
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

pub(super) fn parse_cpu_total(input: &str, clk_tck: f64) -> Option<CpuTimes> {
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

pub(super) fn cpu_utilization(previous: CpuTimes, current: CpuTimes) -> Option<CpuTimes> {
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

pub(super) fn counter_delta(previous: f64, current: f64) -> Option<f64> {
    (current >= previous).then_some(current - previous)
}

pub(super) fn parse_cpuinfo(input: &str) -> CpuInfo {
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

pub(super) fn parse_meminfo(input: &str) -> Option<MemoryStats> {
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
    let has_available = available.is_some();
    let available =
        available.unwrap_or_else(|| free.saturating_add(buffers).saturating_add(cached));
    Some(MemoryStats {
        total,
        used: total.saturating_sub(available),
        free,
        available,
        has_available,
        cached,
        buffered: buffers,
        shared,
        slab_reclaimable,
        slab_unreclaimable,
        hugepages,
    })
}

pub(super) fn parse_uptime(input: &str) -> Option<f64> {
    input.split_whitespace().next()?.parse().ok()
}

pub(super) fn parse_vmstat(input: &str) -> PagingStats {
    let mut total_faults = 0;
    let mut major_faults = 0;
    let mut page_in = 0;
    let mut page_out = 0;
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
            "pgpgin" => page_in = value,
            "pgpgout" => page_out = value,
            "pswpin" => swap_in = value,
            "pswpout" => swap_out = value,
            _ => {}
        }
    }

    PagingStats {
        minor_faults: total_faults.saturating_sub(major_faults),
        major_faults,
        page_in,
        page_out,
        swap_in,
        swap_out,
    }
}

pub(super) fn parse_swaps(input: &str) -> Vec<SwapStats> {
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

pub(super) fn parse_diskstats(
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

pub(super) struct FilesystemMount {
    pub(super) device: String,
    pub(super) mountpoint: String,
    pub(super) fs_type: String,
    pub(super) mode: &'static str,
    pub(super) emit_limit: bool,
}

#[derive(Clone, Copy, Default)]
pub(super) struct FilesystemFilters<'a> {
    pub(super) include_devices: Option<&'a CompiledFilter>,
    pub(super) exclude_devices: Option<&'a CompiledFilter>,
    pub(super) include_fs_types: Option<&'a CompiledFilter>,
    pub(super) exclude_fs_types: Option<&'a CompiledFilter>,
    pub(super) include_mount_points: Option<&'a CompiledFilter>,
    pub(super) exclude_mount_points: Option<&'a CompiledFilter>,
}

pub(super) fn parse_mountinfo(
    input: &str,
    include_virtual_filesystems: bool,
    include_remote_filesystems: bool,
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
        if !include_virtual_filesystems && is_virtual_filesystem_type(fs_type) {
            continue;
        }
        if !include_remote_filesystems && is_remote_filesystem_type(fs_type) {
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

pub(super) fn filesystem_mode(options: &str) -> &'static str {
    if options.split(',').any(|option| option == "ro") {
        "ro"
    } else {
        "rw"
    }
}

pub(super) fn is_remote_filesystem_type(fs_type: &str) -> bool {
    if fs_type == "fuse" || fs_type == "fuseblk" || fs_type.starts_with("fuse.") {
        return true;
    }
    matches!(fs_type, "nfs" | "nfs4" | "cifs" | "smb3" | "9p")
}

pub(super) fn is_virtual_filesystem_type(fs_type: &str) -> bool {
    matches!(
        fs_type,
        "autofs"
            | "bpf"
            | "binfmt_misc"
            | "cgroup"
            | "cgroup2"
            | "configfs"
            | "debugfs"
            | "devpts"
            | "devtmpfs"
            | "efivarfs"
            | "fusectl"
            | "hugetlbfs"
            | "mqueue"
            | "nsfs"
            | "overlay"
            | "proc"
            | "pstore"
            | "ramfs"
            | "rpc_pipefs"
            | "securityfs"
            | "selinuxfs"
            | "squashfs"
            | "sysfs"
            | "tmpfs"
            | "tracefs"
    )
}

pub(super) fn unescape_mountinfo(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut escaped = None;
    for idx in 0..bytes.len() {
        if bytes[idx] == b'\\' && idx + 4 <= bytes.len() {
            escaped = Some(idx);
            break;
        }
    }
    let Some(first_escape) = escaped else {
        return input.to_owned();
    };

    let mut output = Vec::with_capacity(input.len());
    output.extend_from_slice(&bytes[..first_escape]);
    let mut idx = first_escape;
    while idx < bytes.len() {
        if bytes[idx] == b'\\' && idx + 4 <= bytes.len() {
            let octal = &input[idx + 1..idx + 4];
            if let Ok(value) = u8::from_str_radix(octal, 8) {
                output.push(value);
                idx += 4;
                continue;
            }
        }
        output.push(bytes[idx]);
        idx += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

pub(super) fn parse_netdev(
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

pub(super) fn filter_allows(
    value: &str,
    include: Option<&CompiledFilter>,
    exclude: Option<&CompiledFilter>,
) -> bool {
    include.is_none_or(|filter| filter.matches(value))
        && !exclude.is_some_and(|filter| filter.matches(value))
}

pub(super) fn record_partial_error(
    partial_errors: &mut u64,
    first_error: &mut Option<io::Error>,
    err: io::Error,
) {
    *partial_errors = partial_errors.saturating_add(1);
    if first_error.is_none() {
        *first_error = Some(err);
    }
}

pub(super) fn frequency_hz_i64(value: f64) -> i64 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    if value >= i64::MAX as f64 {
        return i64::MAX;
    }
    value.round() as i64
}

pub(super) fn parse_u64(input: &str) -> u64 {
    input.parse().unwrap_or_default()
}

pub(super) fn ticks_to_seconds(ticks: u64, clk_tck: f64) -> f64 {
    ticks as f64 / clk_tck
}

pub(super) fn millis_to_seconds(ms: u64) -> f64 {
    ms as f64 / 1_000.0
}

#[allow(unsafe_code)]
pub(super) fn clock_ticks_per_second() -> f64 {
    // SAFETY: _SC_CLK_TCK is a valid sysconf name; the call has no side effects.
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks > 0 { ticks as f64 } else { 100.0 }
}

pub(super) fn now_unix_nano() -> u64 {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return 0;
    };
    duration.as_secs().saturating_mul(NANOS_PER_SEC) + u64::from(duration.subsec_nanos())
}

pub(super) fn saturating_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
