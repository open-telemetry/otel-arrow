// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! NUMA topology discovery abstractions.
//!
//! The controller owns discovery and placement. Runtime workers receive resolved
//! topology metadata instead of reading host state independently.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;
use std::sync::Arc;

#[cfg(any(target_os = "linux", test))]
pub mod linux;

/// Indicates how much usable CPU topology discovery covered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopologyCompleteness {
    /// Discovery completed for every usable CPU visible to the process.
    Complete,
    /// Discovery produced usable data, but at least one source degraded.
    Partial,
    /// Discovery could not produce usable topology data.
    Unknown,
}

/// Provider abstraction for OS-specific NUMA topology discovery.
pub trait NumaTopologyProvider: fmt::Debug + Send + Sync {
    /// Discovers the CPU to NUMA-node mapping visible to this process.
    fn discover(&self) -> NumaTopology;
}

/// CPU to NUMA-node mapping visible to the current engine process.
#[derive(Clone, Debug)]
pub struct NumaTopology {
    inner: Arc<NumaTopologyInner>,
}

#[derive(Debug)]
struct NumaTopologyInner {
    cpu_to_node: BTreeMap<u32, u32>,
    visible_cpus: BTreeSet<u32>,
    visible_nodes: BTreeSet<u32>,
    completeness: TopologyCompleteness,
}

impl Default for NumaTopology {
    fn default() -> Self {
        Self::unknown()
    }
}

impl NumaTopology {
    /// Creates an unknown topology with no visible CPU mapping.
    #[must_use]
    pub fn unknown() -> Self {
        Self::new(BTreeMap::new(), TopologyCompleteness::Unknown)
    }

    /// Detects NUMA topology using the platform default provider.
    #[must_use]
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            DefaultNumaTopologyProvider::default().discover()
        }

        #[cfg(not(target_os = "linux"))]
        {
            DefaultNumaTopologyProvider.discover()
        }
    }

    /// Detects Linux NUMA topology under an arbitrary sysfs root.
    ///
    /// On non-Linux platforms this returns an unknown topology.
    #[must_use]
    pub fn from_sysfs_root(root: &Path) -> Self {
        Self::from_sysfs_root_impl(root)
    }

    #[cfg(any(target_os = "linux", test))]
    fn from_sysfs_root_impl(root: &Path) -> Self {
        linux::LinuxNumaTopologyProvider::from_sysfs_root_for_test(root)
    }

    #[cfg(not(any(target_os = "linux", test)))]
    fn from_sysfs_root_impl(_root: &Path) -> Self {
        Self::unknown()
    }

    /// Builds a topology from in-memory `(node_id, cpulist)` pairs.
    #[must_use]
    pub fn from_node_cpulists(entries: &[(u32, String)]) -> Self {
        let mut cpu_to_node = BTreeMap::new();
        let mut partial = false;
        for (node_id, cpulist) in entries {
            match parse_cpu_list(cpulist) {
                Ok(cpus) => {
                    for cpu in cpus {
                        _ = cpu_to_node.insert(cpu, *node_id);
                    }
                }
                Err(_) => partial = true,
            }
        }
        let completeness = if partial {
            TopologyCompleteness::Partial
        } else {
            TopologyCompleteness::Complete
        };
        Self::new(cpu_to_node, completeness)
    }

    /// Creates a topology from a CPU to NUMA-node map.
    #[must_use]
    pub fn new(cpu_to_node: BTreeMap<u32, u32>, completeness: TopologyCompleteness) -> Self {
        let visible_cpus = cpu_to_node.keys().copied().collect();
        Self::with_visible_cpus(cpu_to_node, visible_cpus, completeness)
    }

    /// Creates a topology with explicit process-visible CPUs.
    ///
    /// `cpu_to_node` may be a partial mapping. CPUs present in `visible_cpus`
    /// but absent from `cpu_to_node` are still usable, but their NUMA node is
    /// unknown.
    #[must_use]
    pub fn with_visible_cpus(
        cpu_to_node: BTreeMap<u32, u32>,
        visible_cpus: BTreeSet<u32>,
        completeness: TopologyCompleteness,
    ) -> Self {
        let visible_nodes = visible_cpus
            .iter()
            .filter_map(|cpu| cpu_to_node.get(cpu).copied())
            .collect();
        let completeness = if cpu_to_node.is_empty() {
            if visible_cpus.is_empty() {
                TopologyCompleteness::Unknown
            } else {
                TopologyCompleteness::Partial
            }
        } else {
            completeness
        };
        Self {
            inner: Arc::new(NumaTopologyInner {
                cpu_to_node,
                visible_cpus,
                visible_nodes,
                completeness,
            }),
        }
    }

    /// Returns the discovered NUMA node for `cpu_id`, if known.
    #[must_use]
    pub fn numa_node(&self, cpu_id: u32) -> Option<u32> {
        self.inner.cpu_to_node.get(&cpu_id).copied()
    }

    /// Returns the NUMA node for telemetry paths that need a concrete value.
    ///
    /// Unknown CPU ids fall back to `0` to preserve the engine's existing
    /// telemetry behavior while discovery is unavailable or partial.
    #[must_use]
    pub fn numa_node_or_zero(&self, cpu_id: u32) -> u32 {
        self.numa_node(cpu_id).unwrap_or(0)
    }

    /// CPUs visible and usable by this process after affinity/cgroup filtering.
    #[must_use]
    pub fn visible_cpus(&self) -> &BTreeSet<u32> {
        &self.inner.visible_cpus
    }

    /// NUMA nodes represented by the visible CPU set.
    #[must_use]
    pub fn visible_nodes(&self) -> &BTreeSet<u32> {
        &self.inner.visible_nodes
    }

    /// Completeness of the discovery result.
    #[must_use]
    pub fn completeness(&self) -> TopologyCompleteness {
        self.inner.completeness
    }

    /// Returns `true` when no CPU mapping is available.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        self.inner.completeness == TopologyCompleteness::Unknown
    }
}

/// Parses a Linux cpulist such as `0-3,8,10-12`.
pub(crate) fn parse_cpu_list(input: &str) -> Result<BTreeSet<u32>, ParseCpuListError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseCpuListError::Empty);
    }

    let mut cpus = BTreeSet::new();
    for raw in input.split(',') {
        let token = raw.trim();
        if token.is_empty() {
            return Err(ParseCpuListError::EmptyToken);
        }
        if let Some((lo, hi)) = token.split_once('-') {
            let lo = lo.trim();
            let hi = hi.trim();
            if lo.is_empty() || hi.is_empty() {
                return Err(ParseCpuListError::BadRange(token.to_string()));
            }
            let lo = lo
                .parse::<u32>()
                .map_err(|_| ParseCpuListError::BadRange(token.to_string()))?;
            let hi = hi
                .parse::<u32>()
                .map_err(|_| ParseCpuListError::BadRange(token.to_string()))?;
            if lo > hi {
                return Err(ParseCpuListError::ReversedRange(lo, hi));
            }
            for cpu in lo..=hi {
                _ = cpus.insert(cpu);
            }
        } else {
            let cpu = token
                .parse::<u32>()
                .map_err(|_| ParseCpuListError::BadCpu(token.to_string()))?;
            _ = cpus.insert(cpu);
        }
    }
    Ok(cpus)
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub(crate) enum ParseCpuListError {
    #[error("empty cpulist")]
    Empty,
    #[error("empty token in cpulist")]
    EmptyToken,
    #[error("invalid range `{0}` in cpulist")]
    BadRange(String),
    #[error("reversed range `{0}-{1}` in cpulist")]
    ReversedRange(u32, u32),
    #[error("invalid cpu id `{0}` in cpulist")]
    BadCpu(String),
}

#[cfg(not(target_os = "linux"))]
#[derive(Debug, Default)]
/// Default topology provider for platforms without an OS backend yet.
pub struct DefaultNumaTopologyProvider;

#[cfg(not(target_os = "linux"))]
impl NumaTopologyProvider for DefaultNumaTopologyProvider {
    fn discover(&self) -> NumaTopology {
        NumaTopology::unknown()
    }
}

#[cfg(target_os = "linux")]
pub use linux::LinuxNumaTopologyProvider as DefaultNumaTopologyProvider;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_cpu_list_accepts_ranges_and_deduplicates() {
        assert_eq!(
            parse_cpu_list("3,1-2,2,8").unwrap(),
            BTreeSet::from([1, 2, 3, 8])
        );
    }

    #[test]
    fn parse_cpu_list_rejects_bad_inputs() {
        assert_eq!(parse_cpu_list("").unwrap_err(), ParseCpuListError::Empty);
        assert_eq!(
            parse_cpu_list("1,,2").unwrap_err(),
            ParseCpuListError::EmptyToken
        );
        assert_eq!(
            parse_cpu_list("4-2").unwrap_err(),
            ParseCpuListError::ReversedRange(4, 2)
        );
    }

    #[test]
    fn topology_unknown_when_empty_even_if_marked_complete() {
        let topology = NumaTopology::new(BTreeMap::new(), TopologyCompleteness::Complete);
        assert_eq!(topology.completeness(), TopologyCompleteness::Unknown);
        assert!(topology.visible_cpus().is_empty());
        assert_eq!(topology.numa_node_or_zero(42), 0);
    }

    #[test]
    fn topology_can_keep_visible_cpus_without_numa_mapping() {
        let topology = NumaTopology::with_visible_cpus(
            BTreeMap::new(),
            BTreeSet::from([2, 3]),
            TopologyCompleteness::Partial,
        );

        assert_eq!(topology.completeness(), TopologyCompleteness::Partial);
        assert_eq!(topology.visible_cpus(), &BTreeSet::from([2, 3]));
        assert!(topology.visible_nodes().is_empty());
        assert_eq!(topology.numa_node(2), None);
    }
}
