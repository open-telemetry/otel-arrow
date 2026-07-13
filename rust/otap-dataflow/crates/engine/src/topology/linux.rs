// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux NUMA topology discovery.

use super::{NumaTopology, NumaTopologyProvider, TopologyCompleteness, parse_cpu_list};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Linux sysfs/cgroup backed NUMA topology provider.
#[derive(Clone, Debug)]
pub struct LinuxNumaTopologyProvider {
    node_root: PathBuf,
    cgroup_root: PathBuf,
    affinity_reader: AffinityReader,
}

type AffinityReader = fn() -> io::Result<BTreeSet<u32>>;

impl Default for LinuxNumaTopologyProvider {
    fn default() -> Self {
        Self {
            node_root: PathBuf::from("/sys/devices/system/node"),
            cgroup_root: PathBuf::from("/sys/fs/cgroup"),
            affinity_reader: read_sched_affinity,
        }
    }
}

impl LinuxNumaTopologyProvider {
    /// Creates a provider using the host's standard Linux topology paths.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Discovers topology from a synthetic sysfs root without process filtering.
    pub(crate) fn from_sysfs_root_for_test(root: &Path) -> NumaTopology {
        let DiscoveryResult {
            cpu_to_node,
            partial,
        } = discover_sysfs_topology(root);
        let completeness = if partial {
            TopologyCompleteness::Partial
        } else {
            TopologyCompleteness::Complete
        };
        NumaTopology::new(cpu_to_node, completeness)
    }

    #[cfg(test)]
    fn for_test(node_root: PathBuf, cgroup_root: PathBuf, affinity_reader: AffinityReader) -> Self {
        Self {
            node_root,
            cgroup_root,
            affinity_reader,
        }
    }
}

impl NumaTopologyProvider for LinuxNumaTopologyProvider {
    fn discover(&self) -> NumaTopology {
        let DiscoveryResult {
            mut cpu_to_node,
            mut partial,
        } = discover_sysfs_topology(&self.node_root);

        let allowed = match self.allowed_cpus() {
            AllowedCpuDiscovery::Known { cpus, degraded } => {
                partial |= degraded;
                Some(cpus)
            }
            AllowedCpuDiscovery::Unavailable => {
                partial = true;
                None
            }
        };

        let visible_cpus = if let Some(allowed) = allowed {
            cpu_to_node.retain(|cpu, _| allowed.contains(cpu));
            allowed
        } else {
            cpu_to_node.keys().copied().collect()
        };

        let completeness = if partial {
            TopologyCompleteness::Partial
        } else {
            TopologyCompleteness::Complete
        };
        NumaTopology::with_visible_cpus(cpu_to_node, visible_cpus, completeness)
    }
}

impl LinuxNumaTopologyProvider {
    fn allowed_cpus(&self) -> AllowedCpuDiscovery {
        let mut partial = false;
        let mut allowed = match (self.affinity_reader)() {
            Ok(cpus) => cpus,
            Err(_) => {
                partial = true;
                BTreeSet::new()
            }
        };

        match read_cgroup_v2_effective_cpus(&self.cgroup_root) {
            CgroupCpuDiscovery::Known(cpus) => {
                if allowed.is_empty() {
                    allowed = cpus;
                } else {
                    allowed = allowed.intersection(&cpus).copied().collect();
                }
            }
            CgroupCpuDiscovery::Unavailable => {}
            CgroupCpuDiscovery::Degraded => {
                partial = true;
            }
        }

        if allowed.is_empty() && partial {
            AllowedCpuDiscovery::Unavailable
        } else {
            AllowedCpuDiscovery::Known {
                cpus: allowed,
                degraded: partial,
            }
        }
    }
}

#[derive(Debug)]
struct DiscoveryResult {
    cpu_to_node: BTreeMap<u32, u32>,
    partial: bool,
}

fn discover_sysfs_topology(root: &Path) -> DiscoveryResult {
    let Ok(entries) = fs::read_dir(root) else {
        return DiscoveryResult {
            cpu_to_node: BTreeMap::new(),
            partial: true,
        };
    };

    let mut partial = false;
    let mut cpu_to_node = BTreeMap::new();
    let mut duplicate_cpus = BTreeSet::new();
    for entry in entries {
        let Ok(entry) = entry else {
            partial = true;
            continue;
        };
        let path = entry.path();
        let Some(node_id) = parse_node_dir(&path) else {
            continue;
        };
        let cpulist_path = path.join("cpulist");
        let cpulist = match fs::read_to_string(&cpulist_path) {
            Ok(cpulist) => cpulist,
            Err(_) => {
                partial = true;
                continue;
            }
        };
        if cpulist.trim().is_empty() {
            continue;
        }
        let cpus = match parse_cpu_list(&cpulist) {
            Ok(cpus) => cpus,
            Err(_) => {
                partial = true;
                continue;
            }
        };
        for cpu in cpus {
            if duplicate_cpus.contains(&cpu) {
                continue;
            }
            if cpu_to_node.insert(cpu, node_id).is_some() {
                partial = true;
                _ = duplicate_cpus.insert(cpu);
                _ = cpu_to_node.remove(&cpu);
            }
        }
    }

    DiscoveryResult {
        cpu_to_node,
        partial,
    }
}

fn parse_node_dir(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_str()?;
    let node_id = name.strip_prefix("node")?;
    node_id.parse::<u32>().ok()
}

enum AllowedCpuDiscovery {
    Known { cpus: BTreeSet<u32>, degraded: bool },
    Unavailable,
}

enum CgroupCpuDiscovery {
    Known(BTreeSet<u32>),
    Unavailable,
    Degraded,
}

fn read_cgroup_v2_effective_cpus(cgroup_root: &Path) -> CgroupCpuDiscovery {
    let root_cpuset_path = cgroup_root.join("cpuset.cpus.effective");
    let cpuset_path = current_cgroup_v2_relative_path()
        .map(|relative| cgroup_root.join(relative).join("cpuset.cpus.effective"))
        .unwrap_or_else(|| root_cpuset_path.clone());

    match fs::read_to_string(cpuset_path) {
        Ok(raw) => match parse_cpu_list(&raw) {
            Ok(cpus) => CgroupCpuDiscovery::Known(cpus),
            Err(_) => CgroupCpuDiscovery::Degraded,
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound && root_cpuset_path.exists() => {
            match fs::read_to_string(root_cpuset_path) {
                Ok(raw) => match parse_cpu_list(&raw) {
                    Ok(cpus) => CgroupCpuDiscovery::Known(cpus),
                    Err(_) => CgroupCpuDiscovery::Degraded,
                },
                Err(_) => CgroupCpuDiscovery::Degraded,
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => CgroupCpuDiscovery::Unavailable,
        Err(_) => CgroupCpuDiscovery::Degraded,
    }
}

fn current_cgroup_v2_relative_path() -> Option<PathBuf> {
    let raw = fs::read_to_string("/proc/self/cgroup").ok()?;
    for line in raw.lines() {
        let mut parts = line.splitn(3, ':');
        let _hierarchy = parts.next()?;
        let controllers = parts.next()?;
        let path = parts.next()?;
        if controllers.is_empty() {
            return Some(PathBuf::from(path.trim_start_matches('/')));
        }
    }
    None
}

fn read_sched_affinity() -> io::Result<BTreeSet<u32>> {
    read_sched_affinity_impl()
}

#[cfg(target_os = "linux")]
fn read_sched_affinity_impl() -> io::Result<BTreeSet<u32>> {
    use nix::sched::{CpuSet, sched_getaffinity};
    use nix::unistd::Pid;

    let set = sched_getaffinity(Pid::from_raw(0)).map_err(io::Error::other)?;
    let mut cpus = BTreeSet::new();
    for cpu in 0..CpuSet::count() {
        if set.is_set(cpu).map_err(io::Error::other)? {
            let Ok(cpu) = u32::try_from(cpu) else {
                break;
            };
            _ = cpus.insert(cpu);
        }
    }
    Ok(cpus)
}

#[cfg(not(target_os = "linux"))]
fn read_sched_affinity_impl() -> io::Result<BTreeSet<u32>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "sched_getaffinity is Linux-only",
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn affinity_0_to_3() -> io::Result<BTreeSet<u32>> {
        Ok(BTreeSet::from([0, 1, 2, 3]))
    }

    fn affinity_4_to_7() -> io::Result<BTreeSet<u32>> {
        Ok(BTreeSet::from([4, 5, 6, 7]))
    }

    fn affinity_error() -> io::Result<BTreeSet<u32>> {
        Err(io::Error::other("mock affinity failure"))
    }

    fn write_node(root: &Path, node: u32, cpus: &str) {
        let dir = root.join(format!("node{node}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("cpulist"), cpus).unwrap();
    }

    #[test]
    fn discovers_synthetic_sysfs_and_intersects_affinity() {
        let sysfs = tempfile::tempdir().unwrap();
        let cgroup = tempfile::tempdir().unwrap();
        write_node(sysfs.path(), 0, "0-3");
        write_node(sysfs.path(), 1, "4-7");

        let provider = LinuxNumaTopologyProvider::for_test(
            sysfs.path().to_path_buf(),
            cgroup.path().to_path_buf(),
            affinity_4_to_7,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Complete);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([4, 5, 6, 7]));
        assert_eq!(topology.visible_nodes(), &BTreeSet::from([1]));
        assert_eq!(topology.numa_node(4), Some(1));
        assert_eq!(topology.numa_node(0), None);
    }

    #[test]
    fn intersects_cgroup_effective_cpuset_when_present() {
        let sysfs = tempfile::tempdir().unwrap();
        let cgroup = tempfile::tempdir().unwrap();
        write_node(sysfs.path(), 0, "0-3");
        write_node(sysfs.path(), 1, "4-7");
        fs::write(cgroup.path().join("cpuset.cpus.effective"), "2-5").unwrap();

        let provider = LinuxNumaTopologyProvider::for_test(
            sysfs.path().to_path_buf(),
            cgroup.path().to_path_buf(),
            affinity_0_to_3,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Complete);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([2, 3]));
        assert_eq!(topology.visible_nodes(), &BTreeSet::from([0]));
    }

    #[test]
    fn marks_partial_on_bad_node_and_keeps_good_mapping() {
        let sysfs = tempfile::tempdir().unwrap();
        let cgroup = tempfile::tempdir().unwrap();
        write_node(sysfs.path(), 0, "0-3");
        write_node(sysfs.path(), 1, "not-a-cpulist");

        let provider = LinuxNumaTopologyProvider::for_test(
            sysfs.path().to_path_buf(),
            cgroup.path().to_path_buf(),
            affinity_0_to_3,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Partial);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([0, 1, 2, 3]));
        assert_eq!(topology.numa_node_or_zero(99), 0);
    }

    #[test]
    fn partial_sysfs_keeps_allowed_unmapped_cpus_visible() {
        let sysfs = tempfile::tempdir().unwrap();
        let cgroup = tempfile::tempdir().unwrap();
        write_node(sysfs.path(), 0, "0-1");
        write_node(sysfs.path(), 1, "not-a-cpulist");

        let provider = LinuxNumaTopologyProvider::for_test(
            sysfs.path().to_path_buf(),
            cgroup.path().to_path_buf(),
            affinity_0_to_3,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Partial);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([0, 1, 2, 3]));
        assert_eq!(topology.visible_nodes(), &BTreeSet::from([0]));
        assert_eq!(topology.numa_node(2), None);
    }

    #[test]
    fn missing_sysfs_keeps_allowed_cpus_visible_without_numa_mapping() {
        let sysfs = tempfile::tempdir().unwrap();
        let cgroup = tempfile::tempdir().unwrap();

        let provider = LinuxNumaTopologyProvider::for_test(
            sysfs.path().join("missing-node-root"),
            cgroup.path().to_path_buf(),
            affinity_0_to_3,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Unknown);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([0, 1, 2, 3]));
        assert!(topology.visible_nodes().is_empty());
        assert_eq!(topology.numa_node(0), None);
    }

    #[test]
    fn duplicate_cpu_mappings_are_degraded_and_unmapped() {
        let sysfs = tempfile::tempdir().unwrap();
        let cgroup = tempfile::tempdir().unwrap();
        write_node(sysfs.path(), 0, "0-2");
        write_node(sysfs.path(), 1, "2-3");

        let provider = LinuxNumaTopologyProvider::for_test(
            sysfs.path().to_path_buf(),
            cgroup.path().to_path_buf(),
            affinity_0_to_3,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Partial);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([0, 1, 2, 3]));
        assert_eq!(topology.numa_node(0), Some(0));
        assert_eq!(topology.numa_node(2), None);
        assert_eq!(topology.numa_node(3), Some(1));
    }

    #[test]
    fn ignores_empty_cpu_less_node_cpulist() {
        let sysfs = tempfile::tempdir().unwrap();
        let cgroup = tempfile::tempdir().unwrap();
        write_node(sysfs.path(), 0, "0-3");
        write_node(sysfs.path(), 1, "");

        let provider = LinuxNumaTopologyProvider::for_test(
            sysfs.path().to_path_buf(),
            cgroup.path().to_path_buf(),
            affinity_0_to_3,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Complete);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([0, 1, 2, 3]));
        assert_eq!(topology.visible_nodes(), &BTreeSet::from([0]));
    }

    #[test]
    fn marks_partial_when_affinity_degrades() {
        let sysfs = tempfile::tempdir().unwrap();
        let cgroup = tempfile::tempdir().unwrap();
        write_node(sysfs.path(), 0, "0-3");

        let provider = LinuxNumaTopologyProvider::for_test(
            sysfs.path().to_path_buf(),
            cgroup.path().to_path_buf(),
            affinity_error,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Partial);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([0, 1, 2, 3]));
    }

    #[test]
    fn marks_partial_when_affinity_degrades_but_cgroup_is_known() {
        let sysfs = tempfile::tempdir().unwrap();
        let cgroup = tempfile::tempdir().unwrap();
        write_node(sysfs.path(), 0, "0-3");
        fs::write(cgroup.path().join("cpuset.cpus.effective"), "1-2").unwrap();

        let provider = LinuxNumaTopologyProvider::for_test(
            sysfs.path().to_path_buf(),
            cgroup.path().to_path_buf(),
            affinity_error,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Partial);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([1, 2]));
    }

    #[test]
    fn unknown_when_sysfs_and_allowed_cpus_are_unavailable() {
        let cgroup = tempfile::tempdir().unwrap();
        let provider = LinuxNumaTopologyProvider::for_test(
            PathBuf::from("/this/path/should/not/exist"),
            cgroup.path().to_path_buf(),
            affinity_error,
        );
        let topology = provider.discover();

        assert_eq!(topology.completeness(), TopologyCompleteness::Unknown);
        assert!(topology.visible_cpus().is_empty());
    }
}
