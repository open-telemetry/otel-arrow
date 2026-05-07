// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(super) struct ProcfsPaths {
    root: PathBuf,
    stat: PathBuf,
    cpuinfo: PathBuf,
    meminfo: PathBuf,
    uptime: PathBuf,
    vmstat: PathBuf,
    swaps: PathBuf,
    diskstats: PathBuf,
    pub(super) mountinfo: PathBuf,
    pub(super) sys_block: PathBuf,
    pub(super) net_dev: PathBuf,
    machine_id: PathBuf,
    dbus_machine_id: PathBuf,
    hostname: PathBuf,
}

impl ProcfsPaths {
    pub(super) fn new(root_path: Option<&Path>) -> Self {
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

    pub(super) fn path(&self, kind: PathKind) -> &Path {
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

    pub(super) fn host_path(&self, host_absolute_path: &str) -> PathBuf {
        let relative = host_absolute_path
            .strip_prefix('/')
            .unwrap_or(host_absolute_path);
        self.root.join(relative)
    }
}

#[derive(Copy, Clone)]
pub(super) enum PathKind {
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
