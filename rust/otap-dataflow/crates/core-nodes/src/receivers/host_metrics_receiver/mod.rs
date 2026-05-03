// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Host metrics receiver.

use async_trait::async_trait;
use bytes::Bytes;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error, ReceiverErrorKind, TypedError};
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_telemetry::instrument::{Counter, Mmsc};
use otap_df_telemetry::metrics::{MetricSet, MetricSetSnapshot};
use otap_df_telemetry::{otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
use prost::Message as _;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant as StdInstant};
use tokio::time::{Instant, sleep_until};

mod procfs;

use procfs::{HostSnapshot, ProcfsConfig, ProcfsFamilies, ProcfsSource};

/// The URN for the host metrics receiver.
pub const HOST_METRICS_RECEIVER_URN: &str = "urn:otel:receiver:host_metrics";

/// Telemetry metrics for the host metrics receiver.
#[metric_set(name = "host_metrics.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct HostMetricsReceiverMetrics {
    /// Number of scrape ticks started.
    #[metric(unit = "{scrape}")]
    pub scrapes_started: Counter<u64>,
    /// Number of scrape ticks that built and sent a metrics batch.
    #[metric(unit = "{scrape}")]
    pub scrapes_completed: Counter<u64>,
    /// Number of fatal scrape failures.
    #[metric(unit = "{scrape}")]
    pub scrapes_failed: Counter<u64>,
    /// Number of source read errors skipped because other families succeeded.
    #[metric(unit = "{error}")]
    pub partial_errors: Counter<u64>,
    /// Number of source read errors seen during scrapes.
    #[metric(unit = "{error}")]
    pub source_read_errors: Counter<u64>,
    /// Number of due metric families processed.
    #[metric(unit = "{family}")]
    pub families_scraped: Counter<u64>,
    /// Wall-clock scrape duration.
    #[metric(unit = "ns")]
    pub scrape_duration_ns: Mmsc,
    /// Delay between scheduled and actual scrape start.
    #[metric(unit = "ns")]
    pub scrape_lag_ns: Mmsc,
    /// Number of batches sent downstream.
    #[metric(unit = "{batch}")]
    pub batches_sent: Counter<u64>,
    /// Number of downstream send failures.
    #[metric(unit = "{error}")]
    pub send_failures: Counter<u64>,
}

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
    pub memory: FamilyConfig,
    /// Paging metrics.
    pub paging: FamilyConfig,
    /// System metrics.
    pub system: FamilyConfig,
    /// Disk metrics.
    pub disk: DiskFamilyConfig,
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
struct RuntimeConfig {
    root_path: PathBuf,
    validation: HostViewValidationMode,
    initial_delay: Duration,
    cpu_utilization: bool,
    families: RuntimeFamilies,
}

#[derive(Clone)]
struct RuntimeFamilies {
    cpu: RuntimeFamily,
    memory: RuntimeFamily,
    paging: RuntimeFamily,
    system: RuntimeFamily,
    disk: RuntimeDiskFamily,
    network: RuntimeNetworkFamily,
    processes: RuntimeFamily,
}

#[derive(Clone)]
struct RuntimeFamily {
    enabled: bool,
    interval: Duration,
}

#[derive(Clone)]
struct RuntimeDiskFamily {
    enabled: bool,
    interval: Duration,
    limit: bool,
    include: Option<CompiledFilter>,
    exclude: Option<CompiledFilter>,
}

#[derive(Clone)]
struct RuntimeNetworkFamily {
    enabled: bool,
    interval: Duration,
    include: Option<CompiledFilter>,
    exclude: Option<CompiledFilter>,
}

#[derive(Clone)]
pub(crate) struct CompiledFilter {
    match_type: MatchType,
    values: Vec<String>,
    regexes: Vec<Regex>,
}

impl CompiledFilter {
    fn compile(
        match_type: MatchType,
        values: Vec<String>,
    ) -> Result<Option<Self>, otap_df_config::error::Error> {
        if values.is_empty() {
            return Ok(None);
        }
        let regexes = if match_type == MatchType::Regexp {
            let mut regexes = Vec::with_capacity(values.len());
            for value in &values {
                regexes.push(Regex::new(value).map_err(|err| {
                    otap_df_config::error::Error::InvalidUserConfig {
                        error: format!("invalid host metrics regexp filter {value:?}: {err}"),
                    }
                })?);
            }
            regexes
        } else {
            Vec::new()
        };
        Ok(Some(Self {
            match_type,
            values,
            regexes,
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
                .regexes
                .iter()
                .any(|candidate| candidate.is_match(value)),
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

/// Host metrics receiver.
pub struct HostMetricsReceiver {
    config: RuntimeConfig,
    _lease: HostMetricsLease,
    metrics: Option<MetricSet<HostMetricsReceiverMetrics>>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Declares the host metrics receiver as a local receiver factory.
pub static HOST_METRICS_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: HOST_METRICS_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        validate_supported_platform()?;
        if pipeline.num_cores() > 1 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "host-wide collection must run in a one-core source pipeline; use receiver:host_metrics -> exporter:topic and fan out downstream".to_owned(),
            });
        }
        let mut receiver = HostMetricsReceiver::from_config(&node_config.config)?;
        receiver.metrics = Some(pipeline.register_metrics::<HostMetricsReceiverMetrics>());
        Ok(ReceiverWrapper::local(
            receiver,
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |config| {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        validate_config(&config)
    },
};

impl HostMetricsReceiver {
    /// Creates a new host metrics receiver.
    pub fn new(config: Config) -> Result<Self, otap_df_config::error::Error> {
        validate_supported_platform()?;
        let root_path = normalized_root_path(Some(effective_root_path(&config)?))?;
        let lease = HostMetricsLease::acquire(root_path)?;
        let config = RuntimeConfig::try_from(config)?;
        Ok(Self {
            config,
            _lease: lease,
            metrics: None,
        })
    }

    /// Creates a host metrics receiver from JSON config.
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        validate_config(&config)?;
        Self::new(config)
    }
}

fn validate_config(config: &Config) -> Result<(), otap_df_config::error::Error> {
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

fn validate_supported_platform() -> Result<(), otap_df_config::error::Error> {
    if cfg!(target_os = "linux") {
        Ok(())
    } else {
        Err(otap_df_config::error::Error::InvalidUserConfig {
            error: "host_metrics receiver is supported only on Linux".to_owned(),
        })
    }
}

fn effective_root_path(config: &Config) -> Result<&Path, otap_df_config::error::Error> {
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

fn duration_nanos(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1e9
}

fn elapsed_nanos(start: StdInstant) -> f64 {
    duration_nanos(start.elapsed())
}

fn terminal_state(
    deadline: StdInstant,
    metrics: &Option<MetricSet<HostMetricsReceiverMetrics>>,
) -> TerminalState {
    if let Some(metrics) = metrics {
        TerminalState::new(deadline, [metrics.snapshot()])
    } else {
        TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, [])
    }
}

fn due_family_count(due: ProcfsFamilies) -> u64 {
    u64::from(due.cpu)
        + u64::from(due.memory)
        + u64::from(due.paging)
        + u64::from(due.system)
        + u64::from(due.disk)
        + u64::from(due.network)
        + u64::from(due.processes)
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
            families: RuntimeFamilies {
                cpu: RuntimeFamily::new_cpu(&config.families.cpu, config.collection_interval),
                memory: RuntimeFamily::new(&config.families.memory, config.collection_interval),
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScheduledFamilyKind {
    Cpu,
    Memory,
    Paging,
    System,
    Disk,
    Network,
    Processes,
}

struct ScheduledFamily {
    kind: ScheduledFamilyKind,
    interval: Duration,
    next_due: Instant,
}

struct FamilyScheduler {
    entries: Vec<ScheduledFamily>,
}

impl FamilyScheduler {
    fn new(config: &RuntimeConfig, now: Instant) -> Self {
        let first_due = now + config.initial_delay;
        let mut entries = Vec::with_capacity(7);
        push_scheduled(
            &mut entries,
            ScheduledFamilyKind::Cpu,
            &config.families.cpu,
            first_due,
        );
        push_scheduled(
            &mut entries,
            ScheduledFamilyKind::Memory,
            &config.families.memory,
            first_due,
        );
        push_scheduled(
            &mut entries,
            ScheduledFamilyKind::Paging,
            &config.families.paging,
            first_due,
        );
        push_scheduled(
            &mut entries,
            ScheduledFamilyKind::System,
            &config.families.system,
            first_due,
        );
        if config.families.disk.enabled {
            entries.push(ScheduledFamily {
                kind: ScheduledFamilyKind::Disk,
                interval: config.families.disk.interval,
                next_due: first_due,
            });
        }
        if config.families.network.enabled {
            entries.push(ScheduledFamily {
                kind: ScheduledFamilyKind::Network,
                interval: config.families.network.interval,
                next_due: first_due,
            });
        }
        push_scheduled(
            &mut entries,
            ScheduledFamilyKind::Processes,
            &config.families.processes,
            first_due,
        );
        Self { entries }
    }

    fn next_due(&self) -> Instant {
        self.entries
            .iter()
            .map(|entry| entry.next_due)
            .min()
            .expect("scheduler has at least one enabled family")
    }

    fn mark_due(&mut self, now: Instant) -> ProcfsFamilies {
        let mut due = ProcfsFamilies::default();
        for entry in &mut self.entries {
            if entry.next_due <= now {
                match entry.kind {
                    ScheduledFamilyKind::Cpu => due.cpu = true,
                    ScheduledFamilyKind::Memory => due.memory = true,
                    ScheduledFamilyKind::Paging => due.paging = true,
                    ScheduledFamilyKind::System => due.system = true,
                    ScheduledFamilyKind::Disk => due.disk = true,
                    ScheduledFamilyKind::Network => due.network = true,
                    ScheduledFamilyKind::Processes => due.processes = true,
                }
                while entry.next_due <= now {
                    entry.next_due += entry.interval;
                }
            }
        }
        due
    }
}

fn push_scheduled(
    entries: &mut Vec<ScheduledFamily>,
    kind: ScheduledFamilyKind,
    family: &RuntimeFamily,
    first_due: Instant,
) {
    if family.enabled {
        entries.push(ScheduledFamily {
            kind,
            interval: family.interval,
            next_due: first_due,
        });
    }
}

static HOST_METRICS_LEASES: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

struct HostMetricsLease {
    key: PathBuf,
}

impl HostMetricsLease {
    fn acquire(key: PathBuf) -> Result<Self, otap_df_config::error::Error> {
        let mut leases = HOST_METRICS_LEASES.lock().map_err(|_| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: "host metrics lease registry is unavailable".to_owned(),
            }
        })?;
        if !leases.insert(key.clone()) {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "another host_metrics receiver already collects host root {}",
                    key.display()
                ),
            });
        }
        Ok(Self { key })
    }
}

impl Drop for HostMetricsLease {
    fn drop(&mut self) {
        if let Ok(mut leases) = HOST_METRICS_LEASES.lock() {
            let _ = leases.remove(&self.key);
        }
    }
}

fn normalized_root_path(root_path: Option<&Path>) -> Result<PathBuf, otap_df_config::error::Error> {
    let path = root_path.unwrap_or_else(|| Path::new("/"));
    if !path.is_absolute() {
        return Err(otap_df_config::error::Error::InvalidUserConfig {
            error: format!("root_path must be absolute: {}", path.display()),
        });
    }

    let mut normalized = PathBuf::from("/");
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                let _ = normalized.pop();
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

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for HostMetricsReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let HostMetricsReceiver {
            config,
            _lease,
            mut metrics,
        } = *self;
        let mut source = ProcfsSource::new(
            Some(config.root_path.as_path()),
            ProcfsConfig {
                cpu: config.families.cpu.enabled,
                memory: config.families.memory.enabled,
                paging: config.families.paging.enabled,
                system: config.families.system.enabled,
                disk: config.families.disk.enabled,
                network: config.families.network.enabled,
                processes: config.families.processes.enabled,
                cpu_utilization: config.cpu_utilization,
                disk_limit: config.families.disk.limit,
                disk_include: config.families.disk.include.clone(),
                disk_exclude: config.families.disk.exclude.clone(),
                network_include: config.families.network.include.clone(),
                network_exclude: config.families.network.exclude.clone(),
                validation: config.validation,
            },
        )
        .map_err(|err| Error::ReceiverError {
            receiver: effect_handler.receiver_id(),
            kind: ReceiverErrorKind::Configuration,
            error: format!("failed to validate host metrics procfs sources: {err}"),
            source_detail: String::new(),
        })?;
        let mut scheduler = FamilyScheduler::new(&config, Instant::now());

        let _ = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        loop {
            tokio::select! {
                biased;

                msg = ctrl_msg_recv.recv() => {
                    match msg {
                        Ok(NodeControlMsg::CollectTelemetry { .. }) => {}
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            otel_info!("host_metrics_receiver.drain_ingress");
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(terminal_state(deadline, &metrics));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            otel_info!("host_metrics_receiver.shutdown");
                            return Ok(terminal_state(deadline, &metrics));
                        }
                        Err(e) => return Err(Error::ChannelRecvError(e)),
                        _ => {}
                    }
                }

                _ = sleep_until(scheduler.next_due()) => {
                    let scheduled_due = scheduler.next_due();
                    let now = Instant::now();
                    let due = scheduler.mark_due(now);
                    let scrape_start = StdInstant::now();
                    if let Some(metrics) = metrics.as_mut() {
                        metrics.scrapes_started.add(1);
                        metrics.families_scraped.add(due_family_count(due));
                        metrics.scrape_lag_ns.record(duration_nanos(now.saturating_duration_since(scheduled_due)));
                    }
                    match source.scrape_due(due) {
                        Ok(scrape) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.partial_errors.add(scrape.partial_errors);
                                metrics.source_read_errors.add(scrape.partial_errors);
                            }
                            let pdata = match encode_snapshot(scrape.snapshot) {
                                Ok(pdata) => pdata,
                                Err(err) => {
                                    if let Some(metrics) = metrics.as_mut() {
                                        metrics.scrapes_failed.add(1);
                                        metrics.scrape_duration_ns.record(elapsed_nanos(scrape_start));
                                    }
                                    return Err(Error::ReceiverError {
                                        receiver: effect_handler.receiver_id(),
                                        kind: ReceiverErrorKind::Other,
                                        error: format!("failed to encode host metrics: {err}"),
                                        source_detail: String::new(),
                                    });
                                }
                            };
                            match effect_handler.try_send_message_with_source_node(pdata) {
                                Ok(()) => {
                                    if let Some(metrics) = metrics.as_mut() {
                                        metrics.batches_sent.add(1);
                                        metrics.scrapes_completed.add(1);
                                        metrics.scrape_duration_ns.record(elapsed_nanos(scrape_start));
                                    }
                                }
                                Err(TypedError::ChannelSendError(_)) => {
                                    if let Some(metrics) = metrics.as_mut() {
                                        metrics.send_failures.add(1);
                                        metrics.scrape_duration_ns.record(elapsed_nanos(scrape_start));
                                    }
                                    otel_warn!("host metrics dropped due to downstream backpressure");
                                }
                                Err(other) => {
                                    if let Some(metrics) = metrics.as_mut() {
                                        metrics.send_failures.add(1);
                                        metrics.scrapes_failed.add(1);
                                        metrics.scrape_duration_ns.record(elapsed_nanos(scrape_start));
                                    }
                                    return Err(Error::ReceiverError {
                                        receiver: effect_handler.receiver_id(),
                                        kind: ReceiverErrorKind::Other,
                                        error: format!("failed to send host metrics: {other}"),
                                        source_detail: String::new(),
                                    });
                                }
                            }
                        }
                        Err(err) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.scrapes_failed.add(1);
                                metrics.source_read_errors.add(1);
                                metrics.scrape_duration_ns.record(elapsed_nanos(scrape_start));
                            }
                            return Err(Error::ReceiverError {
                                receiver: effect_handler.receiver_id(),
                                kind: ReceiverErrorKind::Other,
                                error: format!("failed to collect host metrics: {err}"),
                                source_detail: String::new(),
                            });
                        }
                    }
                }
            }
        }
    }
}

fn encode_snapshot(snapshot: HostSnapshot) -> Result<OtapPdata, otap_df_pdata::encode::Error> {
    let request = snapshot.into_export_request();
    let mut buf = Vec::with_capacity(request.encoded_len());
    request
        .encode(&mut buf)
        .expect("encoding metrics request to Vec cannot fail");
    let records: OtapArrowRecords =
        OtlpProtoBytes::ExportMetricsRequest(Bytes::from(buf)).try_into()?;
    Ok(OtapPdata::new(Context::default(), records.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_collection_interval() {
        let mut config = Config::default();
        config.collection_interval = Duration::ZERO;

        assert!(matches!(
            validate_config(&config),
            Err(otap_df_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    #[test]
    fn rejects_all_families_disabled() {
        let config = Config {
            families: FamiliesConfig {
                cpu: CpuFamilyConfig {
                    enabled: false,
                    ..CpuFamilyConfig::default()
                },
                memory: FamilyConfig {
                    enabled: false,
                    ..FamilyConfig::default()
                },
                paging: FamilyConfig {
                    enabled: false,
                    ..FamilyConfig::default()
                },
                system: FamilyConfig {
                    enabled: false,
                    ..FamilyConfig::default()
                },
                disk: DiskFamilyConfig {
                    enabled: false,
                    ..DiskFamilyConfig::default()
                },
                network: NetworkFamilyConfig {
                    enabled: false,
                    ..NetworkFamilyConfig::default()
                },
                processes: ProcessesFamilyConfig {
                    enabled: false,
                    ..ProcessesFamilyConfig::default()
                },
            },
            ..Config::default()
        };

        assert!(matches!(
            validate_config(&config),
            Err(otap_df_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    #[test]
    fn rejects_v1_network_connection_counts() {
        let config = Config {
            families: FamiliesConfig {
                network: NetworkFamilyConfig {
                    include_connection_count: true,
                    ..NetworkFamilyConfig::default()
                },
                ..FamiliesConfig::default()
            },
            ..Config::default()
        };

        assert!(matches!(
            validate_config(&config),
            Err(otap_df_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    #[test]
    fn rejects_zero_enabled_family_interval() {
        let config = Config {
            families: FamiliesConfig {
                cpu: CpuFamilyConfig {
                    interval: Some(Duration::ZERO),
                    ..CpuFamilyConfig::default()
                },
                ..FamiliesConfig::default()
            },
            ..Config::default()
        };

        assert!(matches!(
            validate_config(&config),
            Err(otap_df_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    #[test]
    fn accepts_disabled_cpu_per_cpu_flag() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "families": {
                "cpu": {
                    "per_cpu": false
                }
            }
        }))
        .expect("valid cpu config");

        assert!(!config.families.cpu.per_cpu);
        validate_config(&config).expect("valid config");
    }

    #[test]
    fn accepts_cpu_utilization_opt_in() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "families": {
                "cpu": {
                    "utilization": true
                }
            }
        }))
        .expect("valid cpu config");

        assert!(config.families.cpu.utilization);
        validate_config(&config).expect("valid config");
    }

    #[test]
    fn accepts_disk_limit_opt_in() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "families": {
                "disk": {
                    "limit": true
                }
            }
        }))
        .expect("valid disk config");

        assert!(config.families.disk.limit);
        validate_config(&config).expect("valid config");
    }

    #[test]
    fn rejects_v1_cpu_per_cpu() {
        let config = Config {
            families: FamiliesConfig {
                cpu: CpuFamilyConfig {
                    per_cpu: true,
                    ..CpuFamilyConfig::default()
                },
                ..FamiliesConfig::default()
            },
            ..Config::default()
        };

        assert!(matches!(
            validate_config(&config),
            Err(otap_df_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    #[test]
    fn duplicate_lease_rejects_same_root_until_drop() {
        let root = PathBuf::from("/tmp/otap-host-metrics-lease-test");
        let lease = HostMetricsLease::acquire(root.clone()).expect("first lease");

        assert!(matches!(
            HostMetricsLease::acquire(root.clone()),
            Err(otap_df_config::error::Error::InvalidUserConfig { .. })
        ));

        drop(lease);
        let _lease = HostMetricsLease::acquire(root).expect("lease released on drop");
    }

    #[test]
    fn rejects_conflicting_root_paths() {
        let config = Config {
            root_path: Some(PathBuf::from("/host")),
            host_view: HostViewConfig {
                root_path: PathBuf::from("/container"),
                ..HostViewConfig::default()
            },
            ..Config::default()
        };

        assert!(matches!(
            validate_config(&config),
            Err(otap_df_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    #[test]
    fn accepts_host_view_validation_modes() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "host_view": {
                "validation": "warn_selected"
            }
        }))
        .expect("valid host view config");

        assert_eq!(
            config.host_view.validation,
            HostViewValidationMode::WarnSelected
        );
    }

    #[test]
    fn scheduler_honors_initial_delay_and_family_intervals() {
        let config = Config {
            collection_interval: Duration::from_secs(10),
            initial_delay: Duration::from_secs(1),
            families: FamiliesConfig {
                cpu: CpuFamilyConfig {
                    interval: Some(Duration::from_secs(5)),
                    ..CpuFamilyConfig::default()
                },
                ..FamiliesConfig::default()
            },
            ..Config::default()
        };
        let config = RuntimeConfig::try_from(config).expect("valid config");
        let now = Instant::now();
        let mut scheduler = FamilyScheduler::new(&config, now);

        assert_eq!(scheduler.next_due(), now + Duration::from_secs(1));
        assert_eq!(
            scheduler.mark_due(now),
            ProcfsFamilies::default(),
            "nothing is due before initial_delay"
        );

        let first_due = scheduler.mark_due(now + Duration::from_secs(1));
        assert!(first_due.cpu);
        assert!(first_due.memory);
        assert!(first_due.disk);

        let second_due = scheduler.mark_due(now + Duration::from_secs(6));
        assert!(second_due.cpu);
        assert!(!second_due.memory);
        assert!(!second_due.disk);
    }

    #[test]
    fn glob_filter_matches_without_regex_allocations() {
        let filter = CompiledFilter::compile(MatchType::Glob, vec!["loop*".to_owned()])
            .expect("valid filter")
            .expect("non-empty filter");

        assert!(filter.matches("loop0"));
        assert!(!filter.matches("nvme0n1"));
    }

    #[test]
    fn regexp_filter_is_compiled_once() {
        let filter = CompiledFilter::compile(MatchType::Regexp, vec![r"^eth[0-9]+$".to_owned()])
            .expect("valid filter")
            .expect("non-empty filter");

        assert!(filter.matches("eth0"));
        assert!(!filter.matches("lo"));
    }
}
