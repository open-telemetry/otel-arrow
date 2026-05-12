// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Host metrics receiver.
//!
//! Most implementation is Linux-only (`#[cfg(target_os = "linux")]`).
//! Structs and filters defined here are dead on other platforms by design.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

#[cfg(target_os = "linux")]
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
#[cfg(target_os = "linux")]
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
#[cfg(target_os = "linux")]
use otap_df_engine::control::NodeControlMsg;
#[cfg(target_os = "linux")]
use otap_df_engine::error::{Error, ReceiverErrorKind, TypedError};
#[cfg(target_os = "linux")]
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
#[cfg(target_os = "linux")]
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
#[cfg(target_os = "linux")]
use otap_df_otap::pdata::Context;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::instrument::{Counter, Mmsc};
use otap_df_telemetry::metrics::MetricSet;
#[cfg(target_os = "linux")]
use otap_df_telemetry::metrics::MetricSetSnapshot;
#[cfg(target_os = "linux")]
use otap_df_telemetry::{otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
#[cfg(target_os = "linux")]
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};
#[cfg(any(target_os = "linux", test))]
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::time::Instant as StdInstant;
#[cfg(target_os = "linux")]
use tokio::time::{Instant, sleep_until};

mod config;
#[cfg(target_os = "linux")]
mod otap_builder;
#[cfg(target_os = "linux")]
mod procfs;
#[cfg(target_os = "linux")]
mod semconv;

#[cfg(target_os = "linux")]
use procfs::{HostSnapshot, ProcfsConfig, ProcfsFamilies, ProcfsSource};

#[cfg(any(target_os = "linux", test))]
pub(crate) use config::CompiledFilter;
use config::RuntimeConfig;
#[cfg(any(target_os = "linux", test))]
use config::validate_config;
pub use config::{
    Config, CpuFamilyConfig, DeviceFilterConfig, DiskFamilyConfig, FamiliesConfig, FamilyConfig,
    FilesystemFamilyConfig, FilesystemTypeFilterConfig, HostViewConfig, HostViewValidationMode,
    InterfaceFilterConfig, MatchType, MemoryFamilyConfig, MountPointFilterConfig,
    NetworkFamilyConfig, ProcessMode, ProcessesFamilyConfig,
};
#[cfg(target_os = "linux")]
use config::{RuntimeFamily, effective_root_path, normalized_root_path};

/// The URN for the host metrics receiver.
pub const HOST_METRICS_RECEIVER_URN: &str = "urn:otel:receiver:host_metrics";

/// Telemetry metrics for the host metrics receiver.
#[metric_set(name = "receiver.host_metrics")]
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
    // TODO: Decide whether fixed per-family error counters are needed here.
    // Metric-level attributes are not supported by the internal telemetry API today.
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
    create: create_host_metrics_receiver,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: validate_host_metrics_config,
};

#[cfg(target_os = "linux")]
fn create_host_metrics_receiver(
    pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
) -> Result<ReceiverWrapper<OtapPdata>, otap_df_config::error::Error> {
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
}

#[cfg(not(target_os = "linux"))]
fn create_host_metrics_receiver(
    _pipeline: PipelineContext,
    _node: NodeId,
    _node_config: Arc<NodeUserConfig>,
    _receiver_config: &ReceiverConfig,
) -> Result<ReceiverWrapper<OtapPdata>, otap_df_config::error::Error> {
    Err(unsupported_platform_error())
}

#[cfg(target_os = "linux")]
fn validate_host_metrics_config(config: &Value) -> Result<(), otap_df_config::error::Error> {
    let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
        otap_df_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        }
    })?;
    RuntimeConfig::try_from(config).map(|_| ())
}

#[cfg(not(target_os = "linux"))]
fn validate_host_metrics_config(
    _config: &serde_json::Value,
) -> Result<(), otap_df_config::error::Error> {
    Err(unsupported_platform_error())
}

#[cfg(target_os = "linux")]
impl HostMetricsReceiver {
    /// Creates a new host metrics receiver.
    pub fn new(config: Config) -> Result<Self, otap_df_config::error::Error> {
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

#[cfg(not(target_os = "linux"))]
fn unsupported_platform_error() -> otap_df_config::error::Error {
    otap_df_config::error::Error::InvalidUserConfig {
        error: "host_metrics receiver is supported only on Linux".to_owned(),
    }
}

#[cfg(target_os = "linux")]
fn duration_nanos(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1e9
}

#[cfg(target_os = "linux")]
fn elapsed_nanos(start: StdInstant) -> f64 {
    duration_nanos(start.elapsed())
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn due_family_count(due: ProcfsFamilies) -> u64 {
    u64::from(due.cpu)
        + u64::from(due.memory)
        + u64::from(due.paging)
        + u64::from(due.system)
        + u64::from(due.disk)
        + u64::from(due.filesystem)
        + u64::from(due.network)
        + u64::from(due.processes)
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScheduledFamilyKind {
    Cpu,
    Memory,
    Paging,
    System,
    Disk,
    Filesystem,
    Network,
    Processes,
}

#[cfg(target_os = "linux")]
struct ScheduledFamily {
    kind: ScheduledFamilyKind,
    interval: Duration,
    next_due: Instant,
}

#[cfg(target_os = "linux")]
struct FamilyScheduler {
    entries: Vec<ScheduledFamily>,
}

#[cfg(target_os = "linux")]
impl FamilyScheduler {
    fn new(config: &RuntimeConfig, now: Instant) -> Self {
        let first_due = now + config.initial_delay;
        let mut entries = Vec::with_capacity(8);
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
        if config.families.filesystem.enabled {
            entries.push(ScheduledFamily {
                kind: ScheduledFamilyKind::Filesystem,
                interval: config.families.filesystem.interval,
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

    fn next_due(&self, now: Instant) -> Instant {
        self.entries
            .iter()
            .map(|entry| entry.next_due)
            .min()
            .unwrap_or(now)
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
                    ScheduledFamilyKind::Filesystem => due.filesystem = true,
                    ScheduledFamilyKind::Network => due.network = true,
                    ScheduledFamilyKind::Processes => due.processes = true,
                }
                let elapsed = now.duration_since(entry.next_due);
                let missed_ticks = elapsed.as_nanos() / entry.interval.as_nanos() + 1;
                let advance = entry
                    .interval
                    .saturating_mul(u32::try_from(missed_ticks).unwrap_or(u32::MAX));
                entry.next_due += advance;
                if entry.next_due <= now {
                    entry.next_due = now + entry.interval;
                }
            }
        }
        due
    }
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
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
                filesystem: config.families.filesystem.enabled,
                network: config.families.network.enabled,
                processes: config.families.processes.enabled,
                cpu_utilization: config.cpu_utilization,
                memory_limit: config.memory_limit,
                memory_shared: config.memory_shared,
                memory_hugepages: config.memory_hugepages,
                disk_limit: config.families.disk.limit,
                filesystem_include_virtual: config.families.filesystem.include_virtual_filesystems,
                filesystem_include_remote: config.families.filesystem.include_remote_filesystems,
                filesystem_limit: config.families.filesystem.limit,
                disk_include: config.families.disk.include.clone(),
                disk_exclude: config.families.disk.exclude.clone(),
                filesystem_include_devices: config.families.filesystem.include_devices.clone(),
                filesystem_exclude_devices: config.families.filesystem.exclude_devices.clone(),
                filesystem_include_fs_types: config.families.filesystem.include_fs_types.clone(),
                filesystem_exclude_fs_types: config.families.filesystem.exclude_fs_types.clone(),
                filesystem_include_mount_points: config
                    .families
                    .filesystem
                    .include_mount_points
                    .clone(),
                filesystem_exclude_mount_points: config
                    .families
                    .filesystem
                    .exclude_mount_points
                    .clone(),
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
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            if let Some(metrics) = metrics.as_mut() {
                                let _ = metrics_reporter.report(metrics);
                            }
                        }
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

                _ = sleep_until(scheduler.next_due(Instant::now())) => {
                    let scheduled_due = scheduler.next_due(Instant::now());
                    let now = Instant::now();
                    let due = scheduler.mark_due(now);
                    let scrape_start = StdInstant::now();
                    if let Some(metrics) = metrics.as_mut() {
                        metrics.scrapes_started.add(1);
                        metrics.families_scraped.add(due_family_count(due));
                        metrics.scrape_lag_ns.record(duration_nanos(now.saturating_duration_since(scheduled_due)));
                    }
                    match source.scrape_due(due).await {
                        Ok(scrape) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.partial_errors.add(scrape.partial_errors);
                                metrics.source_read_errors.add(scrape.partial_errors);
                            }
                            tokio::task::consume_budget().await;
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
                            tokio::task::consume_budget().await;
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
                            otel_warn!(
                                "host metrics scrape failed; receiver will retry",
                                error = err.to_string()
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn encode_snapshot(snapshot: HostSnapshot) -> Result<OtapPdata, arrow::error::ArrowError> {
    let records = snapshot.into_otap_records()?;
    Ok(OtapPdata::new(Context::default(), records.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_collection_interval() {
        let config = Config {
            collection_interval: Duration::ZERO,
            ..Config::default()
        };

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
                memory: MemoryFamilyConfig {
                    enabled: false,
                    ..MemoryFamilyConfig::default()
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
                filesystem: FilesystemFamilyConfig {
                    enabled: false,
                    ..FilesystemFamilyConfig::default()
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
    fn accepts_memory_opt_ins() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "families": {
                "memory": {
                    "limit": true,
                    "shared": true,
                    "hugepages": true
                }
            }
        }))
        .expect("valid memory config");

        assert!(config.families.memory.limit);
        assert!(config.families.memory.shared);
        assert!(config.families.memory.hugepages);
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
    fn accepts_filesystem_options() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "families": {
                "filesystem": {
                    "interval": "5m",
                    "include_virtual_filesystems": true,
                    "include_remote_filesystems": true,
                    "limit": true,
                    "exclude_fs_types": {
                        "fs_types": ["tmpfs"],
                        "match_type": "strict"
                    },
                    "include_mount_points": {
                        "mount_points": ["/", "/data*"],
                        "match_type": "glob"
                    }
                }
            }
        }))
        .expect("valid filesystem config");

        assert_eq!(
            config.families.filesystem.interval,
            Some(Duration::from_secs(300))
        );
        assert!(config.families.filesystem.include_virtual_filesystems);
        assert!(config.families.filesystem.include_remote_filesystems);
        assert!(config.families.filesystem.limit);
        assert_eq!(
            config
                .families
                .filesystem
                .exclude_fs_types
                .as_ref()
                .expect("exclude fs types")
                .fs_types,
            vec!["tmpfs".to_owned()]
        );
        assert_eq!(
            config
                .families
                .filesystem
                .include_mount_points
                .as_ref()
                .expect("include mount points")
                .mount_points,
            vec!["/".to_owned(), "/data*".to_owned()]
        );
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

    #[cfg(target_os = "linux")]
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

        assert_eq!(scheduler.next_due(now), now + Duration::from_secs(1));
        assert_eq!(
            scheduler.mark_due(now),
            ProcfsFamilies::default(),
            "nothing is due before initial_delay"
        );

        let first_due = scheduler.mark_due(now + Duration::from_secs(1));
        assert!(first_due.cpu);
        assert!(first_due.memory);
        assert!(first_due.disk);
        assert!(first_due.filesystem);

        let second_due = scheduler.mark_due(now + Duration::from_secs(6));
        assert!(second_due.cpu);
        assert!(!second_due.memory);
        assert!(!second_due.disk);
        assert!(!second_due.filesystem);
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

    #[test]
    fn factory_validation_rejects_invalid_regex_filter() {
        let config = serde_json::json!({
            "families": {
                "disk": {
                    "include": {
                        "devices": ["["],
                        "match_type": "regexp"
                    }
                }
            }
        });

        assert!(matches!(
            (HOST_METRICS_RECEIVER.validate_config)(&config),
            Err(otap_df_config::error::Error::InvalidUserConfig { .. })
        ));
    }
}
