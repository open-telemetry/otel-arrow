// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::receivers::host_metrics_receiver::otap_builder::HostMetricsArrowBuilder;
use crate::receivers::host_metrics_receiver::semconv::{attr, metric};
use std::collections::{HashMap, HashSet};

use super::COUNTER_KEY_SEPARATOR;
use super::readings::{
    CpuInfo, CpuTimes, DiskStats, FilesystemStats, HugepageStats, MemoryStats, NetworkStats,
    PagingStats, ProcessStats, SwapStats, frequency_hz_i64, saturating_i64,
};

/// Result of one host metrics scrape.
pub(crate) struct HostScrape {
    /// Collected host snapshot.
    pub snapshot: HostSnapshot,
    /// Number of source read errors skipped because other families succeeded.
    pub partial_errors: u64,
}

/// One host metrics snapshot.
#[derive(Default)]
pub(crate) struct HostSnapshot {
    pub(super) now_unix_nano: u64,
    pub(super) start_time_unix_nano: u64,
    pub(super) counter_starts: CounterStarts,
    pub(super) memory_limit: bool,
    pub(super) memory_shared: bool,
    pub(super) memory_hugepages: bool,
    pub(super) cpu: Option<CpuTimes>,
    pub(super) cpu_utilization: Option<CpuTimes>,
    pub(super) cpuinfo: CpuInfo,
    pub(super) memory: Option<MemoryStats>,
    pub(super) uptime_seconds: Option<f64>,
    pub(super) paging: Option<PagingStats>,
    pub(super) swaps: Vec<SwapStats>,
    pub(super) processes: Option<ProcessStats>,
    pub(super) disks: Vec<DiskStats>,
    pub(super) filesystems: Vec<FilesystemStats>,
    pub(super) networks: Vec<NetworkStats>,
    pub(super) resource: HostResource,
}

impl HostSnapshot {
    pub(super) fn has_metrics(&self) -> bool {
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

    /// Converts a snapshot directly into an OTAP Arrow metrics batch.
    pub fn into_otap_records(
        self,
    ) -> Result<otap_df_pdata::otap::OtapArrowRecords, arrow::error::ArrowError> {
        let mut b = HostMetricsArrowBuilder::new();
        b.append_resource(&self.resource);
        project_snapshot(&self, &mut b);
        b.finish()
    }
}

#[derive(Clone, Default)]
pub(crate) struct HostResource {
    pub(crate) host_id: Option<String>,
    pub(crate) host_name: Option<String>,
    pub(crate) host_arch: Option<&'static str>,
}

pub(super) fn project_snapshot(snap: &HostSnapshot, b: &mut HostMetricsArrowBuilder) {
    let now = snap.now_unix_nano;
    let start = snap.start_time_unix_nano;
    let cs = &snap.counter_starts;

    // ── CPU ──────────────────────────────────────────────────────────────────
    if let Some(cpu) = snap.cpu {
        let m = b.begin_counter_f64(metric::CPU_TIME, "s");
        for (mode, value) in [
            ("user", cpu.user),
            ("nice", cpu.nice),
            ("system", cpu.system),
            ("idle", cpu.idle),
            ("iowait", cpu.wait),
            ("interrupt", cpu.interrupt),
            ("steal", cpu.steal),
        ] {
            b.append_f64_sum_dp(m, cs.get(metric::CPU_TIME, mode, start), now, value, |w| {
                w.str(attr::CPU_MODE, mode);
            });
        }
    }
    if let Some(cpu) = snap.cpu_utilization {
        let m = b.begin_gauge_f64(metric::CPU_UTILIZATION, "1");
        for (mode, value) in [
            ("user", cpu.user),
            ("nice", cpu.nice),
            ("system", cpu.system),
            ("idle", cpu.idle),
            ("iowait", cpu.wait),
            ("interrupt", cpu.interrupt),
            ("steal", cpu.steal),
        ] {
            b.append_f64_gauge_dp(m, now, value, |w| {
                w.str(attr::CPU_MODE, mode);
            });
        }
    }
    if snap.cpuinfo.logical_count != 0 {
        let m = b.begin_updown_i64(metric::CPU_LOGICAL_COUNT, "{cpu}");
        b.append_i64_sum_dp(
            m,
            start,
            now,
            saturating_i64(snap.cpuinfo.logical_count),
            |_| {},
        );
    }
    if snap.cpuinfo.physical_count != 0 {
        let m = b.begin_updown_i64(metric::CPU_PHYSICAL_COUNT, "{cpu}");
        b.append_i64_sum_dp(
            m,
            start,
            now,
            saturating_i64(snap.cpuinfo.physical_count),
            |_| {},
        );
    }
    if !snap.cpuinfo.frequencies_hz.is_empty() {
        let m = b.begin_gauge_i64(metric::CPU_FREQUENCY, "Hz");
        for (idx, &freq) in snap.cpuinfo.frequencies_hz.iter().enumerate() {
            let logical = i64::try_from(idx).unwrap_or(i64::MAX);
            b.append_i64_gauge_dp(m, now, frequency_hz_i64(freq), |w| {
                w.int(attr::CPU_LOGICAL_NUMBER, logical);
            });
        }
    }

    // ── Memory ───────────────────────────────────────────────────────────────
    if let Some(memory) = snap.memory {
        let m = b.begin_updown_i64(metric::MEMORY_USAGE, "By");
        for (state, value) in [
            ("used", memory.used),
            ("free", memory.free),
            ("cached", memory.cached),
            ("buffers", memory.buffered),
        ] {
            b.append_i64_sum_dp(m, start, now, saturating_i64(value), |w| {
                w.str(attr::SYSTEM_MEMORY_STATE, state);
            });
        }
        if memory.total > 0 {
            let m = b.begin_gauge_f64(metric::MEMORY_UTILIZATION, "1");
            let total = memory.total as f64;
            for (state, value) in [
                ("used", memory.used),
                ("free", memory.free),
                ("cached", memory.cached),
                ("buffers", memory.buffered),
            ] {
                b.append_f64_gauge_dp(m, now, value as f64 / total, |w| {
                    w.str(attr::SYSTEM_MEMORY_STATE, state);
                });
            }
        }
        if memory.has_available {
            let m = b.begin_updown_i64(metric::MEMORY_LINUX_AVAILABLE, "By");
            b.append_i64_sum_dp(m, start, now, saturating_i64(memory.available), |_| {});
        }
        let m = b.begin_updown_i64(metric::MEMORY_LINUX_SLAB_USAGE, "By");
        for (state, value) in [
            ("reclaimable", memory.slab_reclaimable),
            ("unreclaimable", memory.slab_unreclaimable),
        ] {
            b.append_i64_sum_dp(m, start, now, saturating_i64(value), |w| {
                w.str(attr::SYSTEM_MEMORY_LINUX_SLAB_STATE, state);
            });
        }
        if snap.memory_limit {
            let m = b.begin_updown_i64(metric::MEMORY_LIMIT, "By");
            b.append_i64_sum_dp(m, start, now, saturating_i64(memory.total), |_| {});
        }
        if snap.memory_shared {
            let m = b.begin_updown_i64(metric::MEMORY_LINUX_SHARED, "By");
            b.append_i64_sum_dp(m, start, now, saturating_i64(memory.shared), |_| {});
        }
        if snap.memory_hugepages {
            project_hugepages(b, start, now, &memory.hugepages);
        }
    }

    // ── System / uptime ──────────────────────────────────────────────────────
    if let Some(uptime) = snap.uptime_seconds {
        let m = b.begin_gauge_f64(metric::UPTIME, "s");
        b.append_f64_gauge_dp(m, now, uptime, |_| {});
    }

    // ── Paging ───────────────────────────────────────────────────────────────
    if let Some(paging) = snap.paging {
        let m = b.begin_counter_i64(metric::PAGING_FAULTS, "{fault}");
        for (fault_type, value) in [
            ("minor", paging.minor_faults),
            ("major", paging.major_faults),
        ] {
            b.append_i64_sum_dp(
                m,
                cs.get(metric::PAGING_FAULTS, fault_type, start),
                now,
                saturating_i64(value),
                |w| {
                    w.str(attr::SYSTEM_PAGING_FAULT_TYPE, fault_type);
                },
            );
        }
        let m = b.begin_counter_i64(metric::PAGING_OPERATIONS, "{operation}");
        // Linux exposes swap operations and page-in/page-out counters separately.
        // Semconv requires both direction and fault.type for this metric, so the
        // receiver keeps the phase-1 mapping explicit here.
        for (direction, fault_type, value) in [
            ("in", "major", paging.swap_in),
            ("out", "major", paging.swap_out),
            ("in", "minor", paging.page_in),
            ("out", "minor", paging.page_out),
        ] {
            b.append_i64_sum_dp(
                m,
                cs.get_joined(metric::PAGING_OPERATIONS, direction, fault_type, start),
                now,
                saturating_i64(value),
                |w| {
                    w.str(attr::SYSTEM_PAGING_DIRECTION, direction);
                    w.str(attr::SYSTEM_PAGING_FAULT_TYPE, fault_type);
                },
            );
        }
    }
    if !snap.swaps.is_empty() {
        let m = b.begin_updown_i64(metric::PAGING_USAGE, "By");
        for swap in &snap.swaps {
            for (state, value) in [("used", swap.used), ("free", swap.free)] {
                b.append_i64_sum_dp(m, start, now, saturating_i64(value), |w| {
                    w.str(attr::SYSTEM_DEVICE, &swap.name);
                    w.str(attr::SYSTEM_PAGING_STATE, state);
                });
            }
        }
    }
    if snap.swaps.iter().any(|swap| swap.size > 0) {
        let m = b.begin_gauge_f64(metric::PAGING_UTILIZATION, "1");
        for swap in &snap.swaps {
            let size = swap.size;
            if size == 0 {
                continue;
            }
            let total = size as f64;
            for (state, value) in [("used", swap.used), ("free", swap.free)] {
                b.append_f64_gauge_dp(m, now, value as f64 / total, |w| {
                    w.str(attr::SYSTEM_DEVICE, &swap.name);
                    w.str(attr::SYSTEM_PAGING_STATE, state);
                });
            }
        }
    }

    // ── Processes ────────────────────────────────────────────────────────────
    if let Some(processes) = snap.processes {
        let m = b.begin_updown_i64(metric::PROCESS_COUNT, "{process}");
        b.append_i64_sum_dp(m, start, now, saturating_i64(processes.running), |w| {
            w.str(attr::PROCESS_STATE, "running");
        });
        // /proc/stat procs_blocked has no registered process.state value.
        // Do not map it to sleeping; Linux blocked tasks are not the same state.
        let m = b.begin_counter_i64(metric::PROCESS_CREATED, "{process}");
        b.append_i64_sum_dp(
            m,
            cs.get(metric::PROCESS_CREATED, "", start),
            now,
            saturating_i64(processes.created),
            |_| {},
        );
    }

    // ── Disk ─────────────────────────────────────────────────────────────────
    if snap.disks.iter().any(|disk| disk.limit_bytes.is_some()) {
        let m = b.begin_updown_i64(metric::DISK_LIMIT, "By");
        for disk in &snap.disks {
            let Some(limit_bytes) = disk.limit_bytes else {
                continue;
            };
            b.append_i64_sum_dp(m, start, now, saturating_i64(limit_bytes), |w| {
                w.str(attr::SYSTEM_DEVICE, &disk.name);
            });
        }
    }
    if !snap.disks.is_empty() {
        let m = b.begin_counter_i64(metric::DISK_IO, "By");
        for disk in &snap.disks {
            for (dir, value) in [("read", disk.read_bytes), ("write", disk.write_bytes)] {
                b.append_i64_sum_dp(
                    m,
                    cs.get_joined(metric::DISK_IO, &disk.name, dir, start),
                    now,
                    saturating_i64(value),
                    |w| {
                        w.str(attr::SYSTEM_DEVICE, &disk.name);
                        w.str(attr::DISK_IO_DIRECTION, dir);
                    },
                );
            }
        }
        let m = b.begin_counter_i64(metric::DISK_OPERATIONS, "{operation}");
        for disk in &snap.disks {
            for (dir, value) in [("read", disk.read_ops), ("write", disk.write_ops)] {
                b.append_i64_sum_dp(
                    m,
                    cs.get_joined(metric::DISK_OPERATIONS, &disk.name, dir, start),
                    now,
                    saturating_i64(value),
                    |w| {
                        w.str(attr::SYSTEM_DEVICE, &disk.name);
                        w.str(attr::DISK_IO_DIRECTION, dir);
                    },
                );
            }
        }
        let m = b.begin_counter_f64(metric::DISK_IO_TIME, "s");
        for disk in &snap.disks {
            b.append_f64_sum_dp(
                m,
                cs.get(metric::DISK_IO_TIME, &disk.name, start),
                now,
                disk.io_time_seconds,
                |w| {
                    w.str(attr::SYSTEM_DEVICE, &disk.name);
                },
            );
        }
        let m = b.begin_counter_f64(metric::DISK_OPERATION_TIME, "s");
        for disk in &snap.disks {
            for (dir, value) in [
                ("read", disk.read_time_seconds),
                ("write", disk.write_time_seconds),
            ] {
                b.append_f64_sum_dp(
                    m,
                    cs.get_joined(metric::DISK_OPERATION_TIME, &disk.name, dir, start),
                    now,
                    value,
                    |w| {
                        w.str(attr::SYSTEM_DEVICE, &disk.name);
                        w.str(attr::DISK_IO_DIRECTION, dir);
                    },
                );
            }
        }
        let m = b.begin_counter_i64(metric::DISK_MERGED, "{operation}");
        for disk in &snap.disks {
            for (dir, value) in [("read", disk.read_merged), ("write", disk.write_merged)] {
                b.append_i64_sum_dp(
                    m,
                    cs.get_joined(metric::DISK_MERGED, &disk.name, dir, start),
                    now,
                    saturating_i64(value),
                    |w| {
                        w.str(attr::SYSTEM_DEVICE, &disk.name);
                        w.str(attr::DISK_IO_DIRECTION, dir);
                    },
                );
            }
        }
    }

    // ── Filesystem ───────────────────────────────────────────────────────────
    if !snap.filesystems.is_empty() {
        let m = b.begin_updown_i64(metric::FILESYSTEM_USAGE, "By");
        for fs in &snap.filesystems {
            for (state, value) in [
                ("used", fs.used),
                ("free", fs.free),
                ("reserved", fs.reserved),
            ] {
                b.append_i64_sum_dp(m, start, now, saturating_i64(value), |w| {
                    w.str(attr::SYSTEM_DEVICE, &fs.device);
                    w.str(attr::SYSTEM_FILESYSTEM_STATE, state);
                    w.str(attr::SYSTEM_FILESYSTEM_TYPE, &fs.fs_type);
                    w.str(attr::SYSTEM_FILESYSTEM_MODE, fs.mode);
                    w.str(attr::SYSTEM_FILESYSTEM_MOUNTPOINT, &fs.mountpoint);
                });
            }
        }
    }
    if snap
        .filesystems
        .iter()
        .any(|fs| fs.used.saturating_add(fs.free).saturating_add(fs.reserved) > 0)
    {
        let m = b.begin_gauge_f64(metric::FILESYSTEM_UTILIZATION, "1");
        for fs in &snap.filesystems {
            let total = fs.used.saturating_add(fs.free).saturating_add(fs.reserved);
            if total > 0 {
                let total_f = total as f64;
                for (state, value) in [
                    ("used", fs.used),
                    ("free", fs.free),
                    ("reserved", fs.reserved),
                ] {
                    b.append_f64_gauge_dp(m, now, value as f64 / total_f, |w| {
                        w.str(attr::SYSTEM_DEVICE, &fs.device);
                        w.str(attr::SYSTEM_FILESYSTEM_STATE, state);
                        w.str(attr::SYSTEM_FILESYSTEM_TYPE, &fs.fs_type);
                        w.str(attr::SYSTEM_FILESYSTEM_MODE, fs.mode);
                        w.str(attr::SYSTEM_FILESYSTEM_MOUNTPOINT, &fs.mountpoint);
                    });
                }
            }
        }
    }
    if snap.filesystems.iter().any(|fs| fs.limit_bytes.is_some()) {
        let m = b.begin_updown_i64(metric::FILESYSTEM_LIMIT, "By");
        for fs in &snap.filesystems {
            let Some(limit_bytes) = fs.limit_bytes else {
                continue;
            };
            b.append_i64_sum_dp(m, start, now, saturating_i64(limit_bytes), |w| {
                w.str(attr::SYSTEM_DEVICE, &fs.device);
                w.str(attr::SYSTEM_FILESYSTEM_TYPE, &fs.fs_type);
                w.str(attr::SYSTEM_FILESYSTEM_MODE, fs.mode);
                w.str(attr::SYSTEM_FILESYSTEM_MOUNTPOINT, &fs.mountpoint);
            });
        }
    }

    // ── Network ──────────────────────────────────────────────────────────────
    if !snap.networks.is_empty() {
        let m = b.begin_counter_i64(metric::NETWORK_IO, "By");
        for net in &snap.networks {
            for (dir, value) in [("receive", net.rx_bytes), ("transmit", net.tx_bytes)] {
                b.append_i64_sum_dp(
                    m,
                    cs.get_joined(metric::NETWORK_IO, &net.name, dir, start),
                    now,
                    saturating_i64(value),
                    |w| {
                        w.str(attr::NETWORK_INTERFACE_NAME, &net.name);
                        w.str(attr::NETWORK_IO_DIRECTION, dir);
                    },
                );
            }
        }
        let m = b.begin_counter_i64(metric::NETWORK_PACKET_COUNT, "{packet}");
        for net in &snap.networks {
            for (dir, value) in [("receive", net.rx_packets), ("transmit", net.tx_packets)] {
                b.append_i64_sum_dp(
                    m,
                    cs.get_joined(metric::NETWORK_PACKET_COUNT, &net.name, dir, start),
                    now,
                    saturating_i64(value),
                    |w| {
                        // Semconv uses system.device here, while sibling network
                        // metrics use network.interface.name.
                        w.str(attr::SYSTEM_DEVICE, &net.name);
                        w.str(attr::NETWORK_IO_DIRECTION, dir);
                    },
                );
            }
        }
        let m = b.begin_counter_i64(metric::NETWORK_PACKET_DROPPED, "{packet}");
        for net in &snap.networks {
            for (dir, value) in [("receive", net.rx_dropped), ("transmit", net.tx_dropped)] {
                b.append_i64_sum_dp(
                    m,
                    cs.get_joined(metric::NETWORK_PACKET_DROPPED, &net.name, dir, start),
                    now,
                    saturating_i64(value),
                    |w| {
                        w.str(attr::NETWORK_INTERFACE_NAME, &net.name);
                        w.str(attr::NETWORK_IO_DIRECTION, dir);
                    },
                );
            }
        }
        let m = b.begin_counter_i64(metric::NETWORK_ERRORS, "{error}");
        for net in &snap.networks {
            for (dir, value) in [("receive", net.rx_errors), ("transmit", net.tx_errors)] {
                b.append_i64_sum_dp(
                    m,
                    cs.get_joined(metric::NETWORK_ERRORS, &net.name, dir, start),
                    now,
                    saturating_i64(value),
                    |w| {
                        w.str(attr::NETWORK_INTERFACE_NAME, &net.name);
                        w.str(attr::NETWORK_IO_DIRECTION, dir);
                    },
                );
            }
        }
    }
}

pub(super) fn project_hugepages(
    b: &mut HostMetricsArrowBuilder,
    start: u64,
    now: u64,
    hugepages: &HugepageStats,
) {
    let m = b.begin_updown_i64(metric::MEMORY_LINUX_HUGEPAGES_LIMIT, "{page}");
    b.append_i64_sum_dp(m, start, now, saturating_i64(hugepages.total), |_| {});
    let m = b.begin_updown_i64(metric::MEMORY_LINUX_HUGEPAGES_PAGE_SIZE, "By");
    b.append_i64_sum_dp(
        m,
        start,
        now,
        saturating_i64(hugepages.page_size_bytes),
        |_| {},
    );
    let m = b.begin_updown_i64(metric::MEMORY_LINUX_HUGEPAGES_RESERVED, "{page}");
    b.append_i64_sum_dp(m, start, now, saturating_i64(hugepages.reserved), |_| {});
    let m = b.begin_updown_i64(metric::MEMORY_LINUX_HUGEPAGES_SURPLUS, "{page}");
    b.append_i64_sum_dp(m, start, now, saturating_i64(hugepages.surplus), |_| {});
    let used = hugepages.total.saturating_sub(hugepages.free);
    let m = b.begin_updown_i64(metric::MEMORY_LINUX_HUGEPAGES_USAGE, "{page}");
    for (state, value) in [("used", used), ("free", hugepages.free)] {
        b.append_i64_sum_dp(m, start, now, saturating_i64(value), |w| {
            w.str(attr::SYSTEM_MEMORY_LINUX_HUGEPAGES_STATE, state);
        });
    }
    if hugepages.total > 0 {
        let total = hugepages.total as f64;
        let m = b.begin_gauge_f64(metric::MEMORY_LINUX_HUGEPAGES_UTILIZATION, "1");
        for (state, value) in [("used", used), ("free", hugepages.free)] {
            b.append_f64_gauge_dp(m, now, value as f64 / total, |w| {
                w.str(attr::SYSTEM_MEMORY_LINUX_HUGEPAGES_STATE, state);
            });
        }
    }
}

#[derive(Default)]
pub(super) struct CounterTracker {
    states: HashMap<String, CounterState>,
}

pub(super) struct CounterState {
    previous: f64,
    start_time_unix_nano: u64,
}

#[derive(Default)]
pub(super) struct CounterStarts {
    pub(super) entries: Vec<(String, u64)>,
}

impl CounterStarts {
    fn get(&self, metric: &'static str, series: &str, default_start: u64) -> u64 {
        self.entries
            .iter()
            .find_map(|(key, start)| counter_key_matches(key, metric, series).then_some(*start))
            .unwrap_or(default_start)
    }

    pub(super) fn get_joined(
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
    pub(super) fn snapshot(
        &mut self,
        default_start: u64,
        now: u64,
        cpu: Option<&CpuTimes>,
        paging: Option<&PagingStats>,
        processes: Option<&ProcessStats>,
        disks: Option<&[DiskStats]>,
        networks: Option<&[NetworkStats]>,
    ) -> CounterStarts {
        let mut starts = CounterStarts::default();
        if let Some(cpu) = cpu {
            self.observe_all(
                metric::CPU_TIME,
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
                metric::PAGING_FAULTS,
                default_start,
                now,
                &[
                    ("minor", paging.minor_faults as f64),
                    ("major", paging.major_faults as f64),
                ],
                &mut starts,
            );
            for (direction, fault_type, value) in [
                ("in", "major", paging.swap_in),
                ("out", "major", paging.swap_out),
                ("in", "minor", paging.page_in),
                ("out", "minor", paging.page_out),
            ] {
                self.observe_joined(
                    metric::PAGING_OPERATIONS,
                    direction,
                    fault_type,
                    value as f64,
                    default_start,
                    now,
                    &mut starts,
                );
            }
        }
        if let Some(processes) = processes {
            self.observe(
                metric::PROCESS_CREATED,
                "",
                processes.created as f64,
                default_start,
                now,
                &mut starts,
            );
        }
        if let Some(disks) = disks {
            let first_disk_entry = starts.entries.len();
            for disk in disks {
                self.observe_disk_all(
                    metric::DISK_IO,
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
                    metric::DISK_OPERATIONS,
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
                    metric::DISK_IO_TIME,
                    &disk.name,
                    disk.io_time_seconds,
                    default_start,
                    now,
                    &mut starts,
                );
                self.observe_disk_all(
                    metric::DISK_OPERATION_TIME,
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
                    metric::DISK_MERGED,
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
            self.prune_scraped_family(
                &starts.entries[first_disk_entry..],
                &[
                    metric::DISK_IO,
                    metric::DISK_OPERATIONS,
                    metric::DISK_IO_TIME,
                    metric::DISK_OPERATION_TIME,
                    metric::DISK_MERGED,
                ],
            );
        }
        if let Some(networks) = networks {
            let first_network_entry = starts.entries.len();
            for network in networks {
                self.observe_network(
                    metric::NETWORK_IO,
                    default_start,
                    now,
                    network,
                    network.rx_bytes,
                    network.tx_bytes,
                    &mut starts,
                );
                self.observe_network(
                    metric::NETWORK_PACKET_COUNT,
                    default_start,
                    now,
                    network,
                    network.rx_packets,
                    network.tx_packets,
                    &mut starts,
                );
                self.observe_network(
                    metric::NETWORK_PACKET_DROPPED,
                    default_start,
                    now,
                    network,
                    network.rx_dropped,
                    network.tx_dropped,
                    &mut starts,
                );
                self.observe_network(
                    metric::NETWORK_ERRORS,
                    default_start,
                    now,
                    network,
                    network.rx_errors,
                    network.tx_errors,
                    &mut starts,
                );
            }
            self.prune_scraped_family(
                &starts.entries[first_network_entry..],
                &[
                    metric::NETWORK_IO,
                    metric::NETWORK_PACKET_COUNT,
                    metric::NETWORK_PACKET_DROPPED,
                    metric::NETWORK_ERRORS,
                ],
            );
        }
        starts
    }

    fn prune_scraped_family(
        &mut self,
        current_entries: &[(String, u64)],
        metrics: &[&'static str],
    ) {
        let current_keys = current_entries
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<HashSet<_>>();
        self.states.retain(|key, _| {
            !metrics
                .iter()
                .any(|metric| counter_key_is_metric(key, metric))
                || current_keys.contains(key.as_str())
        });
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

pub(super) fn counter_key(metric: &'static str, series: &str) -> String {
    let mut key = String::with_capacity(metric.len() + 1 + series.len());
    key.push_str(metric);
    key.push(COUNTER_KEY_SEPARATOR);
    key.push_str(series);
    key
}

pub(super) fn counter_key_joined(
    metric: &'static str,
    first: &str,
    second: &'static str,
) -> String {
    let mut key = String::with_capacity(metric.len() + 2 + first.len() + second.len());
    key.push_str(metric);
    key.push(COUNTER_KEY_SEPARATOR);
    key.push_str(first);
    key.push(COUNTER_KEY_SEPARATOR);
    key.push_str(second);
    key
}

pub(super) fn counter_key_matches(key: &str, metric: &'static str, series: &str) -> bool {
    key.strip_prefix(metric)
        .and_then(|rest| rest.strip_prefix(COUNTER_KEY_SEPARATOR))
        == Some(series)
}

pub(super) fn counter_key_matches_joined(
    key: &str,
    metric: &'static str,
    first: &str,
    second: &'static str,
) -> bool {
    let Some(series) = key
        .strip_prefix(metric)
        .and_then(|rest| rest.strip_prefix(COUNTER_KEY_SEPARATOR))
    else {
        return false;
    };
    series
        .strip_prefix(first)
        .and_then(|rest| rest.strip_prefix(COUNTER_KEY_SEPARATOR))
        == Some(second)
}

fn counter_key_is_metric(key: &str, metric: &'static str) -> bool {
    key.strip_prefix(metric)
        .is_some_and(|rest| rest.starts_with(COUNTER_KEY_SEPARATOR))
}

pub(super) fn host_arch() -> Option<&'static str> {
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
