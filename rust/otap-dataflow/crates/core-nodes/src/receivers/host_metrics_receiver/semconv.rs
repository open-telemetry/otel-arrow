// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Semantic convention constants used by the host metrics receiver.
// Keep these strings centralized here. If this surface grows, prefer generating
// constants from the semconv registry instead of adding scattered literals.

/// Semconv version targeted by this receiver's projection layer.
pub(crate) const VERSION: &str = "1.41.0";

/// Schema URL emitted with host metric batches.
pub(crate) const SCHEMA_URL: &[u8] = b"https://opentelemetry.io/schemas/1.41.0";

const _: () = {
    let url = SCHEMA_URL;
    let ver = VERSION.as_bytes();
    assert!(url.len() >= ver.len(), "SCHEMA_URL is shorter than VERSION");
    let suffix = url.split_at(url.len() - ver.len()).1;
    let mut i = 0;
    while i < ver.len() {
        assert!(suffix[i] == ver[i], "SCHEMA_URL suffix must match VERSION");
        i += 1;
    }
};

pub(crate) mod metric {
    pub(crate) const CPU_FREQUENCY: &str = "system.cpu.frequency";
    pub(crate) const CPU_LOGICAL_COUNT: &str = "system.cpu.logical.count";
    pub(crate) const CPU_PHYSICAL_COUNT: &str = "system.cpu.physical.count";
    pub(crate) const CPU_TIME: &str = "system.cpu.time";
    pub(crate) const CPU_UTILIZATION: &str = "system.cpu.utilization";
    pub(crate) const DISK_IO: &str = "system.disk.io";
    pub(crate) const DISK_IO_TIME: &str = "system.disk.io_time";
    pub(crate) const DISK_LIMIT: &str = "system.disk.limit";
    pub(crate) const DISK_MERGED: &str = "system.disk.merged";
    pub(crate) const DISK_OPERATION_TIME: &str = "system.disk.operation_time";
    pub(crate) const DISK_OPERATIONS: &str = "system.disk.operations";
    pub(crate) const FILESYSTEM_LIMIT: &str = "system.filesystem.limit";
    pub(crate) const FILESYSTEM_USAGE: &str = "system.filesystem.usage";
    pub(crate) const FILESYSTEM_UTILIZATION: &str = "system.filesystem.utilization";
    pub(crate) const MEMORY_LIMIT: &str = "system.memory.limit";
    pub(crate) const MEMORY_LINUX_AVAILABLE: &str = "system.memory.linux.available";
    pub(crate) const MEMORY_LINUX_HUGEPAGES_LIMIT: &str = "system.memory.linux.hugepages.limit";
    pub(crate) const MEMORY_LINUX_HUGEPAGES_PAGE_SIZE: &str =
        "system.memory.linux.hugepages.page_size";
    pub(crate) const MEMORY_LINUX_HUGEPAGES_RESERVED: &str =
        "system.memory.linux.hugepages.reserved";
    pub(crate) const MEMORY_LINUX_HUGEPAGES_SURPLUS: &str = "system.memory.linux.hugepages.surplus";
    pub(crate) const MEMORY_LINUX_HUGEPAGES_USAGE: &str = "system.memory.linux.hugepages.usage";
    pub(crate) const MEMORY_LINUX_HUGEPAGES_UTILIZATION: &str =
        "system.memory.linux.hugepages.utilization";
    pub(crate) const MEMORY_LINUX_SHARED: &str = "system.memory.linux.shared";
    pub(crate) const MEMORY_LINUX_SLAB_USAGE: &str = "system.memory.linux.slab.usage";
    pub(crate) const MEMORY_USAGE: &str = "system.memory.usage";
    pub(crate) const MEMORY_UTILIZATION: &str = "system.memory.utilization";
    pub(crate) const NETWORK_ERRORS: &str = "system.network.errors";
    pub(crate) const NETWORK_IO: &str = "system.network.io";
    pub(crate) const NETWORK_PACKET_COUNT: &str = "system.network.packet.count";
    pub(crate) const NETWORK_PACKET_DROPPED: &str = "system.network.packet.dropped";
    pub(crate) const PAGING_FAULTS: &str = "system.paging.faults";
    pub(crate) const PAGING_OPERATIONS: &str = "system.paging.operations";
    pub(crate) const PAGING_USAGE: &str = "system.paging.usage";
    pub(crate) const PAGING_UTILIZATION: &str = "system.paging.utilization";
    pub(crate) const PROCESS_COUNT: &str = "system.process.count";
    pub(crate) const PROCESS_CREATED: &str = "system.process.created";
    pub(crate) const UPTIME: &str = "system.uptime";
}

pub(crate) mod attr {
    pub(crate) const CPU_LOGICAL_NUMBER: &str = "cpu.logical_number";
    pub(crate) const CPU_MODE: &str = "cpu.mode";
    pub(crate) const DISK_IO_DIRECTION: &str = "disk.io.direction";
    pub(crate) const HOST_ARCH: &str = "host.arch";
    pub(crate) const HOST_ID: &str = "host.id";
    pub(crate) const HOST_NAME: &str = "host.name";
    pub(crate) const NETWORK_INTERFACE_NAME: &str = "network.interface.name";
    pub(crate) const NETWORK_IO_DIRECTION: &str = "network.io.direction";
    pub(crate) const OS_TYPE: &str = "os.type";
    pub(crate) const PROCESS_STATE: &str = "process.state";
    pub(crate) const SYSTEM_DEVICE: &str = "system.device";
    pub(crate) const SYSTEM_FILESYSTEM_MODE: &str = "system.filesystem.mode";
    pub(crate) const SYSTEM_FILESYSTEM_MOUNTPOINT: &str = "system.filesystem.mountpoint";
    pub(crate) const SYSTEM_FILESYSTEM_STATE: &str = "system.filesystem.state";
    pub(crate) const SYSTEM_FILESYSTEM_TYPE: &str = "system.filesystem.type";
    pub(crate) const SYSTEM_MEMORY_LINUX_HUGEPAGES_STATE: &str =
        "system.memory.linux.hugepages.state";
    pub(crate) const SYSTEM_MEMORY_LINUX_SLAB_STATE: &str = "system.memory.linux.slab.state";
    pub(crate) const SYSTEM_MEMORY_STATE: &str = "system.memory.state";
    pub(crate) const SYSTEM_PAGING_DIRECTION: &str = "system.paging.direction";
    pub(crate) const SYSTEM_PAGING_FAULT_TYPE: &str = "system.paging.fault.type";
    pub(crate) const SYSTEM_PAGING_STATE: &str = "system.paging.state";
}
