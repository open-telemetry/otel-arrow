// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for the host metrics receiver.

use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

fn default_collection_interval() -> Duration {
    Duration::from_secs(10)
}

fn default_root_path() -> PathBuf {
    PathBuf::from("/")
}

/// Configuration for the host metrics receiver.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Collection interval.
    #[serde(default = "default_collection_interval", with = "humantime_serde")]
    pub collection_interval: Duration,

    /// Delay before the first scrape.
    #[serde(default, with = "humantime_serde")]
    pub initial_delay: Duration,

    /// Optional legacy host root path. Prefer `host_view.root_path`.
    #[serde(default)]
    pub root_path: Option<PathBuf>,

    /// Host filesystem view.
    #[serde(default)]
    pub host_view: HostViewConfig,

    /// Metric family configuration.
    #[serde(default)]
    pub families: FamiliesConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            collection_interval: default_collection_interval(),
            initial_delay: Duration::ZERO,
            root_path: None,
            host_view: HostViewConfig::default(),
            families: FamiliesConfig::default(),
        }
    }
}

/// Host filesystem view configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct HostViewConfig {
    /// Root path for the observed host filesystem.
    #[serde(default = "default_root_path")]
    pub root_path: PathBuf,
    /// Startup validation mode.
    pub validation: HostViewValidationMode,
}

impl Default for HostViewConfig {
    fn default() -> Self {
        Self {
            root_path: default_root_path(),
            validation: HostViewValidationMode::FailSelected,
        }
    }
}

/// Host view startup validation mode.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostViewValidationMode {
    /// Fail startup if selected sources are unavailable.
    #[default]
    FailSelected,
    /// Start and disable unavailable selected sources.
    WarnSelected,
    /// Skip startup validation.
    None,
}

/// Metric family configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FamiliesConfig {
    /// CPU metrics.
    pub cpu: CpuFamilyConfig,
    /// Memory metrics.
    pub memory: MemoryFamilyConfig,
    /// Paging metrics.
    pub paging: FamilyConfig,
    /// System metrics.
    pub system: FamilyConfig,
    /// Disk metrics.
    pub disk: DiskFamilyConfig,
    /// Filesystem metrics.
    pub filesystem: FilesystemFamilyConfig,
    /// Network metrics.
    pub network: NetworkFamilyConfig,
    /// Process summary metrics.
    pub processes: ProcessesFamilyConfig,
}

impl FamiliesConfig {
    fn enabled_count(&self) -> usize {
        usize::from(self.cpu.enabled)
            + usize::from(self.memory.enabled)
            + usize::from(self.paging.enabled)
            + usize::from(self.system.enabled)
            + usize::from(self.disk.enabled)
            + usize::from(self.filesystem.enabled)
            + usize::from(self.network.enabled)
            + usize::from(self.processes.enabled)
    }
}

/// CPU family config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CpuFamilyConfig {
    /// Enable CPU metrics.
    pub enabled: bool,
    /// Family collection interval. Defaults to top-level `collection_interval`.
    #[serde(default, with = "humantime_serde::option")]
    pub interval: Option<Duration>,
    /// Emit per-logical-CPU time series. Not supported in v1.
    pub per_cpu: bool,
    /// Emit aggregate CPU utilization derived from CPU time deltas.
    pub utilization: bool,
}

impl Default for CpuFamilyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: None,
            per_cpu: false,
            utilization: false,
        }
    }
}

/// Common family config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FamilyConfig {
    /// Enable this family.
    pub enabled: bool,
    /// Family collection interval. Defaults to top-level `collection_interval`.
    #[serde(default, with = "humantime_serde::option")]
    pub interval: Option<Duration>,
}

impl Default for FamilyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: None,
        }
    }
}

/// Memory family config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MemoryFamilyConfig {
    /// Enable memory metrics.
    pub enabled: bool,
    /// Family collection interval. Defaults to top-level `collection_interval`.
    #[serde(default, with = "humantime_serde::option")]
    pub interval: Option<Duration>,
    /// Enable memory limit metrics.
    pub limit: bool,
    /// Enable Linux shared memory metric.
    pub shared: bool,
    /// Enable Linux hugepage metrics.
    pub hugepages: bool,
}

impl Default for MemoryFamilyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: None,
            limit: false,
            shared: false,
            hugepages: false,
        }
    }
}

/// Disk family config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiskFamilyConfig {
    /// Enable disk metrics.
    pub enabled: bool,
    /// Family collection interval. Defaults to top-level `collection_interval`.
    #[serde(default, with = "humantime_serde::option")]
    pub interval: Option<Duration>,
    /// Enable disk limit metrics.
    pub limit: bool,
    /// Device include filter.
    pub include: Option<DeviceFilterConfig>,
    /// Device exclude filter.
    pub exclude: Option<DeviceFilterConfig>,
}

impl Default for DiskFamilyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: None,
            limit: false,
            include: None,
            exclude: None,
        }
    }
}

/// Filesystem family config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FilesystemFamilyConfig {
    /// Enable filesystem metrics.
    pub enabled: bool,
    /// Family collection interval. Defaults to top-level `collection_interval`.
    #[serde(default, with = "humantime_serde::option")]
    pub interval: Option<Duration>,
    /// Include virtual filesystems.
    pub include_virtual_filesystems: bool,
    /// Include remote and userspace filesystems.
    pub include_remote_filesystems: bool,
    /// Enable filesystem limit metrics.
    pub limit: bool,
    /// Device include filter.
    pub include_devices: Option<DeviceFilterConfig>,
    /// Device exclude filter.
    pub exclude_devices: Option<DeviceFilterConfig>,
    /// Filesystem type include filter.
    pub include_fs_types: Option<FilesystemTypeFilterConfig>,
    /// Filesystem type exclude filter.
    pub exclude_fs_types: Option<FilesystemTypeFilterConfig>,
    /// Mount point include filter.
    pub include_mount_points: Option<MountPointFilterConfig>,
    /// Mount point exclude filter.
    pub exclude_mount_points: Option<MountPointFilterConfig>,
}

impl Default for FilesystemFamilyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: None,
            include_virtual_filesystems: false,
            include_remote_filesystems: false,
            limit: false,
            include_devices: None,
            exclude_devices: None,
            include_fs_types: None,
            exclude_fs_types: None,
            include_mount_points: None,
            exclude_mount_points: None,
        }
    }
}

/// Filesystem type filter config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FilesystemTypeFilterConfig {
    /// Filesystem types.
    pub fs_types: Vec<String>,
    /// Match type.
    pub match_type: MatchType,
}

impl Default for FilesystemTypeFilterConfig {
    fn default() -> Self {
        Self {
            fs_types: Vec::new(),
            match_type: MatchType::Strict,
        }
    }
}

/// Mount point filter config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MountPointFilterConfig {
    /// Mount points.
    pub mount_points: Vec<String>,
    /// Match type.
    pub match_type: MatchType,
}

impl Default for MountPointFilterConfig {
    fn default() -> Self {
        Self {
            mount_points: Vec::new(),
            match_type: MatchType::Strict,
        }
    }
}

/// Network family config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkFamilyConfig {
    /// Enable network metrics.
    pub enabled: bool,
    /// Family collection interval. Defaults to top-level `collection_interval`.
    #[serde(default, with = "humantime_serde::option")]
    pub interval: Option<Duration>,
    /// Interface include filter.
    pub include: Option<InterfaceFilterConfig>,
    /// Interface exclude filter.
    pub exclude: Option<InterfaceFilterConfig>,
    /// Connection count is not supported in v1.
    pub include_connection_count: bool,
}

impl Default for NetworkFamilyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: None,
            include: None,
            exclude: None,
            include_connection_count: false,
        }
    }
}

/// Process family config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProcessesFamilyConfig {
    /// Enable process summary metrics.
    pub enabled: bool,
    /// Family collection interval. Defaults to top-level `collection_interval`.
    #[serde(default, with = "humantime_serde::option")]
    pub interval: Option<Duration>,
    /// Only `summary` is supported in v1.
    pub mode: ProcessMode,
}

impl Default for ProcessesFamilyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: None,
            mode: ProcessMode::Summary,
        }
    }
}

/// Process collection mode.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessMode {
    /// Aggregate host process summary.
    #[default]
    Summary,
}

/// Disk device filter.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceFilterConfig {
    /// Device names.
    pub devices: Vec<String>,
    /// Match type.
    #[serde(default)]
    pub match_type: MatchType,
}

/// Network interface filter.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceFilterConfig {
    /// Interface names.
    pub interfaces: Vec<String>,
    /// Match type.
    #[serde(default)]
    pub match_type: MatchType,
}

/// Filter match type.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// Exact string match.
    #[default]
    Strict,
    /// Glob match with `*` and `?`.
    Glob,
    /// Regular expression match.
    Regexp,
}

#[derive(Clone)]
pub(super) struct RuntimeConfig {
    pub(super) root_path: PathBuf,
    pub(super) validation: HostViewValidationMode,
    pub(super) initial_delay: Duration,
    pub(super) cpu_utilization: bool,
    pub(super) memory_limit: bool,
    pub(super) memory_shared: bool,
    pub(super) memory_hugepages: bool,
    pub(super) families: RuntimeFamilies,
}

#[derive(Clone)]
pub(super) struct RuntimeFamilies {
    pub(super) cpu: RuntimeFamily,
    pub(super) memory: RuntimeFamily,
    pub(super) paging: RuntimeFamily,
    pub(super) system: RuntimeFamily,
    pub(super) disk: RuntimeDiskFamily,
    pub(super) filesystem: RuntimeFilesystemFamily,
    pub(super) network: RuntimeNetworkFamily,
    pub(super) processes: RuntimeFamily,
}

#[derive(Clone)]
pub(super) struct RuntimeFamily {
    pub(super) enabled: bool,
    pub(super) interval: Duration,
}

#[derive(Clone)]
pub(super) struct RuntimeDiskFamily {
    pub(super) enabled: bool,
    pub(super) interval: Duration,
    pub(super) limit: bool,
    pub(super) include: Option<CompiledFilter>,
    pub(super) exclude: Option<CompiledFilter>,
}

#[derive(Clone)]
pub(super) struct RuntimeFilesystemFamily {
    pub(super) enabled: bool,
    pub(super) interval: Duration,
    pub(super) include_virtual_filesystems: bool,
    pub(super) include_remote_filesystems: bool,
    pub(super) limit: bool,
    pub(super) include_devices: Option<CompiledFilter>,
    pub(super) exclude_devices: Option<CompiledFilter>,
    pub(super) include_fs_types: Option<CompiledFilter>,
    pub(super) exclude_fs_types: Option<CompiledFilter>,
    pub(super) include_mount_points: Option<CompiledFilter>,
    pub(super) exclude_mount_points: Option<CompiledFilter>,
}

#[derive(Clone)]
pub(super) struct RuntimeNetworkFamily {
    pub(super) enabled: bool,
    pub(super) interval: Duration,
    pub(super) include: Option<CompiledFilter>,
    pub(super) exclude: Option<CompiledFilter>,
}

#[derive(Clone)]
pub(crate) struct CompiledFilter {
    match_type: MatchType,
    values: Vec<String>,
    regex_set: Option<RegexSet>,
}

impl CompiledFilter {
    pub(super) fn compile(
        match_type: MatchType,
        values: Vec<String>,
    ) -> Result<Option<Self>, otap_df_config::error::Error> {
        if values.is_empty() {
            return Ok(None);
        }
        let regex_set = if match_type == MatchType::Regexp {
            Some(RegexSet::new(&values).map_err(|err| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!("invalid host metrics regexp filter: {err}"),
                }
            })?)
        } else {
            None
        };
        Ok(Some(Self {
            match_type,
            values,
            regex_set,
        }))
    }

    pub(crate) fn matches(&self, value: &str) -> bool {
        match self.match_type {
            MatchType::Strict => self.values.iter().any(|candidate| candidate == value),
            MatchType::Glob => self
                .values
                .iter()
                .any(|candidate| glob_matches(candidate.as_bytes(), value.as_bytes())),
            MatchType::Regexp => self
                .regex_set
                .as_ref()
                .is_some_and(|regex_set| regex_set.is_match(value)),
        }
    }
}

fn glob_matches(pattern: &[u8], value: &[u8]) -> bool {
    let (mut p, mut v) = (0, 0);
    let mut star = None;
    let mut star_value = 0;

    while v < value.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == value[v]) {
            p += 1;
            v += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            star_value = v;
        } else if let Some(star_pos) = star {
            p = star_pos + 1;
            star_value += 1;
            v = star_value;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

pub(super) fn validate_config(config: &Config) -> Result<(), otap_df_config::error::Error> {
    if config.collection_interval.is_zero() {
        return Err(otap_df_config::error::Error::InvalidUserConfig {
            error: "collection_interval must be greater than zero".to_owned(),
        });
    }
    if config.families.enabled_count() == 0 {
        return Err(otap_df_config::error::Error::InvalidUserConfig {
            error: "at least one host metrics family must be enabled".to_owned(),
        });
    }
    if config.families.network.include_connection_count {
        return Err(otap_df_config::error::Error::InvalidUserConfig {
            error: "network include_connection_count is not supported in v1".to_owned(),
        });
    }
    if config.families.cpu.per_cpu {
        return Err(otap_df_config::error::Error::InvalidUserConfig {
            error: "cpu per_cpu is not supported in v1".to_owned(),
        });
    }
    validate_family_interval(
        "cpu",
        config.families.cpu.enabled,
        config.families.cpu.interval,
    )?;
    validate_family_interval(
        "memory",
        config.families.memory.enabled,
        config.families.memory.interval,
    )?;
    validate_family_interval(
        "paging",
        config.families.paging.enabled,
        config.families.paging.interval,
    )?;
    validate_family_interval(
        "system",
        config.families.system.enabled,
        config.families.system.interval,
    )?;
    validate_family_interval(
        "disk",
        config.families.disk.enabled,
        config.families.disk.interval,
    )?;
    validate_family_interval(
        "filesystem",
        config.families.filesystem.enabled,
        config.families.filesystem.interval,
    )?;
    validate_family_interval(
        "network",
        config.families.network.enabled,
        config.families.network.interval,
    )?;
    validate_family_interval(
        "processes",
        config.families.processes.enabled,
        config.families.processes.interval,
    )?;
    let _ = normalized_root_path(Some(effective_root_path(config)?))?;
    Ok(())
}

pub(super) fn effective_root_path(config: &Config) -> Result<&Path, otap_df_config::error::Error> {
    if let Some(root_path) = config.root_path.as_deref() {
        let host_view_root = config.host_view.root_path.as_path();
        if host_view_root != Path::new("/") && root_path != host_view_root {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "root_path and host_view.root_path cannot both be set to different values"
                    .to_owned(),
            });
        }
        Ok(root_path)
    } else {
        Ok(config.host_view.root_path.as_path())
    }
}

fn validate_family_interval(
    family: &'static str,
    enabled: bool,
    interval: Option<Duration>,
) -> Result<(), otap_df_config::error::Error> {
    if enabled && interval.is_some_and(|interval| interval.is_zero()) {
        return Err(otap_df_config::error::Error::InvalidUserConfig {
            error: format!("{family} interval must be greater than zero"),
        });
    }
    Ok(())
}

impl TryFrom<Config> for RuntimeConfig {
    type Error = otap_df_config::error::Error;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        validate_config(&config)?;
        let root_path = normalized_root_path(Some(effective_root_path(&config)?))?;
        let disk_include = config
            .families
            .disk
            .include
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.devices))
            .transpose()?
            .flatten();
        let disk_exclude = config
            .families
            .disk
            .exclude
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.devices))
            .transpose()?
            .flatten();
        let filesystem_include_devices = config
            .families
            .filesystem
            .include_devices
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.devices))
            .transpose()?
            .flatten();
        let filesystem_exclude_devices = config
            .families
            .filesystem
            .exclude_devices
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.devices))
            .transpose()?
            .flatten();
        let filesystem_include_fs_types = config
            .families
            .filesystem
            .include_fs_types
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.fs_types))
            .transpose()?
            .flatten();
        let filesystem_exclude_fs_types = config
            .families
            .filesystem
            .exclude_fs_types
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.fs_types))
            .transpose()?
            .flatten();
        let filesystem_include_mount_points = config
            .families
            .filesystem
            .include_mount_points
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.mount_points))
            .transpose()?
            .flatten();
        let filesystem_exclude_mount_points = config
            .families
            .filesystem
            .exclude_mount_points
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.mount_points))
            .transpose()?
            .flatten();
        let network_include = config
            .families
            .network
            .include
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.interfaces))
            .transpose()?
            .flatten();
        let network_exclude = config
            .families
            .network
            .exclude
            .map(|filter| CompiledFilter::compile(filter.match_type, filter.interfaces))
            .transpose()?
            .flatten();

        Ok(Self {
            root_path,
            validation: config.host_view.validation,
            initial_delay: config.initial_delay,
            cpu_utilization: config.families.cpu.utilization,
            memory_limit: config.families.memory.limit,
            memory_shared: config.families.memory.shared,
            memory_hugepages: config.families.memory.hugepages,
            families: RuntimeFamilies {
                cpu: RuntimeFamily::new_cpu(&config.families.cpu, config.collection_interval),
                memory: RuntimeFamily::new_memory(
                    &config.families.memory,
                    config.collection_interval,
                ),
                paging: RuntimeFamily::new(&config.families.paging, config.collection_interval),
                system: RuntimeFamily::new(&config.families.system, config.collection_interval),
                disk: RuntimeDiskFamily {
                    enabled: config.families.disk.enabled,
                    interval: config
                        .families
                        .disk
                        .interval
                        .unwrap_or(config.collection_interval),
                    limit: config.families.disk.limit,
                    include: disk_include,
                    exclude: disk_exclude,
                },
                filesystem: RuntimeFilesystemFamily {
                    enabled: config.families.filesystem.enabled,
                    interval: config
                        .families
                        .filesystem
                        .interval
                        .unwrap_or(config.collection_interval),
                    include_virtual_filesystems: config
                        .families
                        .filesystem
                        .include_virtual_filesystems,
                    include_remote_filesystems: config
                        .families
                        .filesystem
                        .include_remote_filesystems,
                    limit: config.families.filesystem.limit,
                    include_devices: filesystem_include_devices,
                    exclude_devices: filesystem_exclude_devices,
                    include_fs_types: filesystem_include_fs_types,
                    exclude_fs_types: filesystem_exclude_fs_types,
                    include_mount_points: filesystem_include_mount_points,
                    exclude_mount_points: filesystem_exclude_mount_points,
                },
                network: RuntimeNetworkFamily {
                    enabled: config.families.network.enabled,
                    interval: config
                        .families
                        .network
                        .interval
                        .unwrap_or(config.collection_interval),
                    include: network_include,
                    exclude: network_exclude,
                },
                processes: RuntimeFamily {
                    enabled: config.families.processes.enabled,
                    interval: config
                        .families
                        .processes
                        .interval
                        .unwrap_or(config.collection_interval),
                },
            },
        })
    }
}

impl RuntimeFamily {
    fn new(config: &FamilyConfig, default_interval: Duration) -> Self {
        Self {
            enabled: config.enabled,
            interval: config.interval.unwrap_or(default_interval),
        }
    }

    fn new_cpu(config: &CpuFamilyConfig, default_interval: Duration) -> Self {
        Self {
            enabled: config.enabled,
            interval: config.interval.unwrap_or(default_interval),
        }
    }

    fn new_memory(config: &MemoryFamilyConfig, default_interval: Duration) -> Self {
        Self {
            enabled: config.enabled,
            interval: config.interval.unwrap_or(default_interval),
        }
    }
}

pub(super) fn normalized_root_path(
    root_path: Option<&Path>,
) -> Result<PathBuf, otap_df_config::error::Error> {
    let path = root_path.unwrap_or_else(|| Path::new("/"));
    let path_text = path.to_string_lossy();
    if !path.is_absolute() && !path_text.starts_with('/') {
        return Err(otap_df_config::error::Error::InvalidUserConfig {
            error: format!("root_path must be absolute: {}", path.display()),
        });
    }

    let mut normalized = PathBuf::from("/");
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::CurDir | Component::ParentDir => {
                return Err(otap_df_config::error::Error::InvalidUserConfig {
                    error: format!(
                        "root_path must not contain . or .. components: {}",
                        path.display()
                    ),
                });
            }
            Component::Prefix(_) => {
                return Err(otap_df_config::error::Error::InvalidUserConfig {
                    error: format!("root_path must be a Unix absolute path: {}", path.display()),
                });
            }
        }
    }
    Ok(normalized)
}
