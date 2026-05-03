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
use otap_df_telemetry::metrics::MetricSetSnapshot;
use otap_df_telemetry::{otel_info, otel_warn};
use prost::Message as _;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tokio::time::interval;

mod procfs;

use procfs::{HostSnapshot, ProcfsConfig, ProcfsSource};

/// The URN for the host metrics receiver.
pub const HOST_METRICS_RECEIVER_URN: &str = "urn:otel:receiver:host_metrics";

fn default_collection_interval() -> Duration {
    Duration::from_secs(10)
}

/// Configuration for the host metrics receiver.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Collection interval.
    #[serde(default = "default_collection_interval", with = "humantime_serde")]
    pub collection_interval: Duration,

    /// Optional host root path. In Kubernetes this is commonly `/host`.
    #[serde(default)]
    pub root_path: Option<PathBuf>,

    /// Metric family configuration.
    #[serde(default)]
    pub families: FamiliesConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            collection_interval: default_collection_interval(),
            root_path: None,
            families: FamiliesConfig::default(),
        }
    }
}

/// Metric family configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FamiliesConfig {
    /// CPU metrics.
    pub cpu: FamilyConfig,
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

/// Common family config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FamilyConfig {
    /// Enable this family.
    pub enabled: bool,
}

impl Default for FamilyConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Disk family config.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiskFamilyConfig {
    /// Enable disk metrics.
    pub enabled: bool,
    /// Device include filter.
    pub include: Option<DeviceFilterConfig>,
    /// Device exclude filter.
    pub exclude: Option<DeviceFilterConfig>,
}

impl Default for DiskFamilyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
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
    /// Only `summary` is supported in v1.
    pub mode: ProcessMode,
}

impl Default for ProcessesFamilyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: ProcessMode::Summary,
        }
    }
}

/// Process collection mode.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessMode {
    /// Aggregate host process summary.
    Summary,
}

impl Default for ProcessMode {
    fn default() -> Self {
        Self::Summary
    }
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
    root_path: Option<PathBuf>,
    collection_interval: Duration,
    families: RuntimeFamilies,
}

#[derive(Clone)]
struct RuntimeFamilies {
    cpu: bool,
    memory: bool,
    paging: bool,
    system: bool,
    disk: RuntimeDiskFamily,
    network: RuntimeNetworkFamily,
    processes: bool,
}

#[derive(Clone)]
struct RuntimeDiskFamily {
    enabled: bool,
    include: Option<CompiledFilter>,
    exclude: Option<CompiledFilter>,
}

#[derive(Clone)]
struct RuntimeNetworkFamily {
    enabled: bool,
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
        if pipeline.num_cores() > 1 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "host-wide collection must run in a one-core source pipeline; use receiver:host_metrics -> exporter:topic and fan out downstream".to_owned(),
            });
        }
        Ok(ReceiverWrapper::local(
            HostMetricsReceiver::from_config(&node_config.config)?,
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
        let root_path = normalized_root_path(config.root_path.as_deref())?;
        let lease = HostMetricsLease::acquire(root_path)?;
        let config = RuntimeConfig::try_from(config)?;
        Ok(Self {
            config,
            _lease: lease,
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
    let _ = normalized_root_path(config.root_path.as_deref())?;
    Ok(())
}

impl TryFrom<Config> for RuntimeConfig {
    type Error = otap_df_config::error::Error;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        validate_config(&config)?;
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
            root_path: config.root_path,
            collection_interval: config.collection_interval,
            families: RuntimeFamilies {
                cpu: config.families.cpu.enabled,
                memory: config.families.memory.enabled,
                paging: config.families.paging.enabled,
                system: config.families.system.enabled,
                disk: RuntimeDiskFamily {
                    enabled: config.families.disk.enabled,
                    include: disk_include,
                    exclude: disk_exclude,
                },
                network: RuntimeNetworkFamily {
                    enabled: config.families.network.enabled,
                    include: network_include,
                    exclude: network_exclude,
                },
                processes: config.families.processes.enabled,
            },
        })
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
        let mut source = ProcfsSource::new(
            self.config.root_path.as_deref(),
            ProcfsConfig {
                cpu: self.config.families.cpu,
                memory: self.config.families.memory,
                paging: self.config.families.paging,
                system: self.config.families.system,
                disk: self.config.families.disk.enabled,
                network: self.config.families.network.enabled,
                processes: self.config.families.processes,
                disk_include: self.config.families.disk.include.clone(),
                disk_exclude: self.config.families.disk.exclude.clone(),
                network_include: self.config.families.network.include.clone(),
                network_exclude: self.config.families.network.exclude.clone(),
            },
        )
        .map_err(|err| Error::ReceiverError {
            receiver: effect_handler.receiver_id(),
            kind: ReceiverErrorKind::Configuration,
            error: format!("failed to validate host metrics procfs sources: {err}"),
            source_detail: String::new(),
        })?;
        let mut ticker = interval(self.config.collection_interval);

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
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            otel_info!("host_metrics_receiver.shutdown");
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Err(e) => return Err(Error::ChannelRecvError(e)),
                        _ => {}
                    }
                }

                _ = ticker.tick() => {
                    match source.scrape() {
                        Ok(snapshot) => {
                            let pdata = encode_snapshot(snapshot).map_err(|err| Error::ReceiverError {
                                receiver: effect_handler.receiver_id(),
                                kind: ReceiverErrorKind::Other,
                                error: format!("failed to encode host metrics: {err}"),
                                source_detail: String::new(),
                            })?;
                            if let Err(err) = effect_handler.try_send_message_with_source_node(pdata) {
                                match err {
                                    TypedError::ChannelSendError(_) => {
                                        otel_warn!("host metrics dropped due to downstream backpressure");
                                    }
                                    other => {
                                        return Err(Error::ReceiverError {
                                            receiver: effect_handler.receiver_id(),
                                            kind: ReceiverErrorKind::Other,
                                            error: format!("failed to send host metrics: {other}"),
                                            source_detail: String::new(),
                                        });
                                    }
                                }
                            }
                        }
                        Err(err) => {
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
                cpu: FamilyConfig { enabled: false },
                memory: FamilyConfig { enabled: false },
                paging: FamilyConfig { enabled: false },
                system: FamilyConfig { enabled: false },
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
