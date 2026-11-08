// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Prometheus scrape receiver backed by `expfmt::arrow::ArrowDecoder`.
//!
//! Scrapes configured HTTP `/metrics` endpoints (static URLs and optional headless DNS service
//! endpoints) in a one-core source pipeline and decodes Prometheus text (`0.0.4` / `1.0.0`) and
//! OpenMetrics (`1.0.0`) payloads directly into OTAP Arrow record batches on the core's local
//! runtime without intermediate protobuf materialization.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = PROMETHEUS_RECEIVER_URN,
    target = "otel.receiver.prometheus",
);

use async_trait::async_trait;
use expfmt::arrow::{ArrowDecoder, DecodeOptions, OtapMetricRecords};
use expfmt::format::{Format, FormatType, response_format};
use linkme::distributed_slice;
use otel_arrow_dfe_channel::error::{RecvError, SendError};
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::MessageSourceLocalEffectHandlerExtension;
use otel_arrow_dfe_engine::ReceiverFactory;
use otel_arrow_dfe_engine::config::ReceiverConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::error::{Error, ReceiverErrorKind, TypedError};
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::OTAP_RECEIVER_FACTORIES;
use otel_arrow_dfe_otap::metrics::{ReceiverPullCancellation, ReceiverPullMetrics};
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::otap::transform::{
    AttributesTransform, LiteralValue, UpsertTransform, apply_attribute_transform,
};
use otel_arrow_dfe_pdata::otap::{Metrics, OtapArrowRecords, OtapBatchStore};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::metrics::MetricSet;
use otel_arrow_dfe_telemetry_macros::metric_set;
use reqwest::Url;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::Value;
use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::{Instant as StdInstant, SystemTime, UNIX_EPOCH};
use tokio::net::lookup_host;
use tokio::time::{Instant, sleep_until};

mod config;

pub use config::Config;
use config::{normalize_target_url, validate_config};

/// The URN for the Prometheus scrape receiver.
pub const PROMETHEUS_RECEIVER_URN: &str = "urn:otel:receiver:prometheus";

/// Telemetry metrics for the Prometheus scrape receiver.
#[metric_set(name = "receiver.prometheus")]
#[derive(Debug, Default, Clone)]
pub struct PrometheusReceiverMetrics {
    /// Number of target scrapes started.
    #[metric(unit = "{scrape}")]
    pub scrapes_started: Counter<u64>,
    /// Number of target scrapes that succeeded.
    #[metric(unit = "{scrape}")]
    pub scrapes_completed: Counter<u64>,
    /// Number of target scrapes that failed HTTP fetch or exposition decoding.
    #[metric(unit = "{scrape}")]
    pub scrapes_failed: Counter<u64>,
    /// Number of scheduled scrape intervals skipped because a scrape cycle overran `scrape_interval`.
    #[metric(unit = "{scrape}")]
    pub scrapes_overrun: Counter<u64>,
    /// Number of DNS service discovery lookup failures (errors, timeouts, or empty resolutions).
    #[metric(unit = "{error}")]
    pub dns_lookups_failed: Counter<u64>,
    /// Total number of metric data points decoded across all scrapes.
    #[metric(unit = "{data_point}")]
    pub data_points_decoded: Counter<u64>,
    /// Number of OTAP batches sent downstream.
    #[metric(unit = "{batch}")]
    pub batches_sent: Counter<u64>,
    /// Number of downstream send failures.
    #[metric(unit = "{error}")]
    pub send_failures: Counter<u64>,
    /// Per-target wall-clock scrape + decode duration.
    #[metric(unit = "ns")]
    pub scrape_duration_ns: Mmsc,
}

/// Default cap for a single `dns_sd_targets` lookup (bounded by `Config::scrape_timeout`).
const DEFAULT_DNS_LOOKUP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

// TODO(bwplotka): Replace static and DNS target resolution with scrape.rs discovery once
// https://github.com/prometheus/scrape.rs/issues/36 lands.
/// Discovers and resolves configured static and DNS scrape target URLs.
///
/// Kept separate from [`TargetScraper`] so future shared work coordination can supply per-core
/// receiver instances with assigned targets while reusing the local scrape and decode path.
struct TargetDiscovery {
    static_urls: Vec<Url>,
    dns_sd_targets: Vec<String>,
    metrics_path: String,
    dns_lookup_timeout: std::time::Duration,
}

impl TargetDiscovery {
    fn from_config(config: &Config) -> Result<Self, otel_arrow_dfe_config::error::Error> {
        let mut static_urls = Vec::with_capacity(config.targets.len());
        for target in &config.targets {
            static_urls.push(normalize_target_url(target, &config.metrics_path)?);
        }
        Ok(Self {
            static_urls,
            dns_sd_targets: config
                .dns_sd_targets
                .iter()
                .map(|target| target.trim().to_owned())
                .collect(),
            metrics_path: config.metrics_path.clone(),
            dns_lookup_timeout: config.scrape_timeout.min(DEFAULT_DNS_LOOKUP_TIMEOUT),
        })
    }

    #[cfg(test)]
    async fn resolve_targets(&self) -> Vec<Url> {
        self.resolve_targets_counted().await.0
    }

    async fn resolve_targets_counted(&self) -> (Vec<Url>, u64) {
        let mut all_urls =
            Vec::with_capacity(self.static_urls.len() + self.dns_sd_targets.len() * 4);
        all_urls.extend(self.static_urls.iter().cloned());
        let mut dns_failures = 0_u64;

        for dns_target in &self.dns_sd_targets {
            match tokio::time::timeout(self.dns_lookup_timeout, lookup_host(dns_target.as_str()))
                .await
            {
                Ok(Ok(addrs)) => {
                    let mut sorted_addrs: Vec<_> = addrs.collect();
                    sorted_addrs.sort_unstable();
                    sorted_addrs.dedup();
                    if sorted_addrs.is_empty() {
                        dns_failures += 1;
                        otel_warn!(
                            "prometheus_receiver.dns_lookup_empty",
                            target = dns_target.as_str(),
                            message = "dns_sd_targets entry resolved to zero addresses"
                        );
                    }
                    for addr in sorted_addrs {
                        let candidate = format!("http://{addr}{}", self.metrics_path);
                        match Url::parse(&candidate) {
                            Ok(url) => all_urls.push(url),
                            Err(err) => {
                                dns_failures += 1;
                                otel_warn!(
                                    "prometheus_receiver.dns_invalid_target_url",
                                    target = dns_target.as_str(),
                                    candidate = candidate.as_str(),
                                    error = err.to_string(),
                                    message = "failed to build scrape URL for resolved DNS address"
                                );
                            }
                        }
                    }
                }
                Ok(Err(err)) => {
                    dns_failures += 1;
                    otel_warn!(
                        "prometheus_receiver.dns_lookup_failed",
                        target = dns_target.as_str(),
                        error = err.to_string(),
                        message = "failed to resolve dns_sd_targets entry"
                    );
                }
                Err(_) => {
                    dns_failures += 1;
                    otel_warn!(
                        "prometheus_receiver.dns_lookup_timeout",
                        target = dns_target.as_str(),
                        timeout_ms =
                            u64::try_from(self.dns_lookup_timeout.as_millis()).unwrap_or(u64::MAX),
                        message = "timed out resolving dns_sd_targets entry"
                    );
                }
            }
        }

        (all_urls, dns_failures)
    }
}

#[derive(Default)]
struct ReceiverTelemetry {
    metrics: Option<MetricSet<PrometheusReceiverMetrics>>,
    pull: ReceiverPullMetrics,
}

/// Prometheus scrape receiver.
pub struct PrometheusReceiver {
    config: Config,
    discovery: TargetDiscovery,
    telemetry: ReceiverTelemetry,
}

#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Declares the Prometheus scrape receiver as a local receiver factory.
pub static PROMETHEUS_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: PROMETHEUS_RECEIVER_URN,
    create:
        |pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         receiver_config: &ReceiverConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            create_prometheus_receiver(pipeline, node, node_config, receiver_config)
        },
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: validate_prometheus_config,
};

fn create_prometheus_receiver(
    pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
) -> Result<ReceiverWrapper<OtapPdata>, otel_arrow_dfe_config::error::Error> {
    if pipeline.num_cores() > 1 {
        // See https://github.com/open-telemetry/otel-arrow/issues/4001
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "prometheus receiver must run in a one-core source pipeline; use \
                receiver:prometheus -> exporter:topic and fan out downstream"
                .to_owned(),
        });
    }
    let mut receiver = PrometheusReceiver::from_config(&node_config.config)?;
    receiver.telemetry.metrics = Some(PrometheusReceiverMetrics::register(&pipeline));
    receiver.telemetry.pull = ReceiverPullMetrics::register(&pipeline);
    Ok(ReceiverWrapper::local(
        receiver,
        node,
        node_config,
        receiver_config,
    ))
}

fn validate_prometheus_config(config: &Value) -> Result<(), otel_arrow_dfe_config::error::Error> {
    let parsed: Config = serde_json::from_value(config.clone()).map_err(|err| {
        otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: err.to_string(),
        }
    })?;
    validate_config(&parsed)
}

impl PrometheusReceiver {
    /// Creates a new `PrometheusReceiver`.
    pub fn new(config: Config) -> Result<Self, otel_arrow_dfe_config::error::Error> {
        validate_config(&config)?;
        let discovery = TargetDiscovery::from_config(&config)?;
        Ok(Self {
            config,
            discovery,
            telemetry: ReceiverTelemetry::default(),
        })
    }

    /// Creates a `PrometheusReceiver` from a raw JSON configuration value.
    pub fn from_config(config: &Value) -> Result<Self, otel_arrow_dfe_config::error::Error> {
        let parsed: Config = serde_json::from_value(config.clone()).map_err(|err| {
            otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: err.to_string(),
            }
        })?;
        Self::new(parsed)
    }
}

fn elapsed_nanos(start: StdInstant) -> f64 {
    start.elapsed().as_secs_f64() * 1e9
}

fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|dur| i64::try_from(dur.as_millis()).ok())
        .unwrap_or(0)
}

fn terminal_state(deadline: StdInstant, telemetry: &mut ReceiverTelemetry) -> TerminalState {
    let mut snapshots = telemetry.pull.terminal_snapshots();
    if let Some(metrics) = &telemetry.metrics {
        snapshots.push(metrics.snapshot());
    }
    TerminalState::new(deadline, snapshots)
}

// TODO(bwplotka): Upstream scraper Accept header construction, text 1.0.0 handling, and fallback
// Content-Type resolution to expfmt::format in https://github.com/prometheus/scrape.rs/issues/34.
/// Resolves the HTTP `Accept` header value advertised during scrapes.
pub(crate) fn build_accept_header(enable_open_metrics: bool) -> &'static str {
    if enable_open_metrics {
        "application/openmetrics-text;version=1.0.0;escaping=allow-utf-8;q=0.6,application/openmetrics-text;version=0.0.1;q=0.5,text/plain;version=1.0.0;escaping=allow-utf-8;q=0.4,text/plain;version=0.0.4;q=0.3"
    } else {
        "text/plain;version=1.0.0;escaping=allow-utf-8;q=0.5,text/plain;version=0.0.4;q=0.4"
    }
}

// TODO(bwplotka): Replace fallback Content-Type resolution with expfmt::format scraper
// negotiation once https://github.com/prometheus/scrape.rs/issues/34 lands.
/// Determines the exposition [`Format`] from an HTTP `Content-Type` header.
///
/// Falls back to OpenMetrics 1.0 when the media type starts with `application/openmetrics-text`
/// and to Prometheus text (`0.0.4`) when the header is missing or omits version parameters.
pub(crate) fn resolve_response_format(content_type: Option<&str>) -> Format {
    let Some(raw) = content_type else {
        return Format::TEXT;
    };
    let fmt = response_format(raw);
    if fmt.format_type() != FormatType::Unknown {
        return fmt;
    }
    let media_type = raw.split(';').next().unwrap_or("").trim();
    if media_type.eq_ignore_ascii_case("application/openmetrics-text") {
        Format::OPEN_METRICS_1_0_0
    } else {
        Format::TEXT
    }
}

// TODO(bwplotka): Construct OtapArrowRecords::Metrics through otel-arrow-dfe-pdata-views
// MetricsView once https://github.com/prometheus/scrape.rs/issues/28 lands.
/// Converts a decoded [`OtapMetricRecords`] batch into [`OtapArrowRecords::Metrics`].
///
/// When `honor_timestamps` is `false`, sample timestamps on metric data point tables are
/// overwritten with `scrape_timestamp_ms` so endpoint-provided timestamps are ignored.
#[cfg(test)]
pub(crate) fn records_to_otap(
    records: &OtapMetricRecords,
    honor_timestamps: bool,
    scrape_timestamp_ms: i64,
) -> Result<OtapArrowRecords, otel_arrow_dfe_pdata::error::Error> {
    records_to_otap_with_resource(records, honor_timestamps, scrape_timestamp_ms, None, None)
}

/// Converts a decoded [`OtapMetricRecords`] batch into [`OtapArrowRecords::Metrics`], optionally
/// populating missing cumulative `start_time_unix_nano` columns and target `ResourceAttrs`.
pub(crate) fn records_to_otap_with_resource(
    records: &OtapMetricRecords,
    honor_timestamps: bool,
    scrape_timestamp_ms: i64,
    default_start_timestamp_ns: Option<i64>,
    resource_transform: Option<&AttributesTransform>,
) -> Result<OtapArrowRecords, otel_arrow_dfe_pdata::error::Error> {
    let mut store = Metrics::new();
    let scrape_timestamp_ns = scrape_timestamp_ms.saturating_mul(1_000_000);
    let univariate_batch = records.get(expfmt::arrow::MetricPayloadType::UnivariateMetrics);
    for (payload_type, batch) in records.iter() {
        if let Ok(wire_type) = ArrowPayloadType::try_from(payload_type.as_i32()) {
            let mut batch = if !honor_timestamps && is_metric_data_point_table(wire_type) {
                // NOTE: `DecodeOptions::with_timestamp_ms` only populates untimestamped samples,
                // so `honor_timestamps: false` must replace the `time_unix_nano` column while
                // keeping `honor_timestamps: true` zero-allocation via `RecordBatch::clone`.
                override_time_unix_nano(batch, scrape_timestamp_ns)?
            } else {
                batch.clone()
            };
            if let Some(start_ts_ns) = default_start_timestamp_ns
                && is_metric_data_point_table(wire_type)
            {
                let clamped_start_ns =
                    start_ts_ns.min(scrape_timestamp_ns.saturating_sub(1_000_000));
                batch = ensure_cumulative_start_time_unix_nano(
                    wire_type,
                    &batch,
                    univariate_batch,
                    clamped_start_ns,
                )?;
            }
            store.set(wire_type, batch)?;
        }
    }
    let mut otap = OtapArrowRecords::Metrics(store);
    if let Some(transform) = resource_transform {
        let _ = apply_attribute_transform(
            &mut otap,
            ArrowPayloadType::ResourceAttrs,
            transform,
            false,
        )?;
    }
    Ok(otap)
}

/// Extracts `<host>:<port>` (or `<host>`) from a scrape target URL for `service.instance.id`.
pub(crate) fn target_instance_id(target_url: &Url) -> String {
    match (target_url.host_str(), target_url.port()) {
        (Some(host), Some(port)) => format!("{host}:{port}"),
        (Some(host), None) => host.to_owned(),
        _ => target_url.as_str().to_owned(),
    }
}

/// Builds an [`AttributesTransform`] that upserts `service.name` and `service.instance.id` on
/// scraped target batches.
pub(crate) fn build_target_resource_transform(
    job_name: &str,
    target_url: &Url,
) -> AttributesTransform {
    AttributesTransform::default().with_upsert(UpsertTransform::new(BTreeMap::from([
        (
            "service.name".into(),
            LiteralValue::Str(job_name.to_owned()),
        ),
        (
            "service.instance.id".into(),
            LiteralValue::Str(target_instance_id(target_url)),
        ),
    ])))
}

const fn is_metric_data_point_table(payload_type: ArrowPayloadType) -> bool {
    matches!(
        payload_type,
        ArrowPayloadType::NumberDataPoints
            | ArrowPayloadType::SummaryDataPoints
            | ArrowPayloadType::HistogramDataPoints
            | ArrowPayloadType::ExpHistogramDataPoints
    )
}

/// Populates `start_time_unix_nano` on cumulative metric data points when the exposition payload
/// omits `_created` timestamps (such as Prometheus text `0.0.4`).
fn ensure_cumulative_start_time_unix_nano(
    payload_type: ArrowPayloadType,
    batch: &arrow::record_batch::RecordBatch,
    univariate_batch: Option<&arrow::record_batch::RecordBatch>,
    start_timestamp_ns: i64,
) -> Result<arrow::record_batch::RecordBatch, otel_arrow_dfe_pdata::error::Error> {
    if batch.num_rows() == 0 {
        return Ok(batch.clone());
    }
    let schema = batch.schema();
    if schema
        .index_of(otel_arrow_dfe_pdata::schema::consts::START_TIME_UNIX_NANO)
        .is_ok()
    {
        return Ok(batch.clone());
    }
    let Ok(parent_idx) = schema.index_of(otel_arrow_dfe_pdata::schema::consts::PARENT_ID) else {
        return Ok(batch.clone());
    };
    let tz: Option<Arc<str>> = schema
        .index_of(otel_arrow_dfe_pdata::schema::consts::TIME_UNIX_NANO)
        .ok()
        .and_then(|idx| match schema.field(idx).data_type() {
            arrow::datatypes::DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, tz) => {
                tz.clone()
            }
            _ => None,
        })
        .or_else(|| Some("UTC".into()));

    let (start_col, nullable): (arrow::array::ArrayRef, bool) =
        if payload_type == ArrowPayloadType::NumberDataPoints {
            // MetricType::Sum == 2; Gauge == 1. Only cumulative Sum rows receive a start timestamp.
            let metric_types = univariate_batch
                .and_then(|ub| ub.column_by_name(otel_arrow_dfe_pdata::schema::consts::METRIC_TYPE))
                .and_then(|col| col.as_any().downcast_ref::<arrow::array::UInt8Array>());
            let parent_ids = batch
                .column(parent_idx)
                .as_any()
                .downcast_ref::<arrow::array::UInt16Array>();
            if let (Some(metric_types), Some(parent_ids)) = (metric_types, parent_ids) {
                let mut any_sum = false;
                let mut all_sum = true;
                for i in 0..parent_ids.len() {
                    let pid = parent_ids.value(i) as usize;
                    let is_sum = pid < metric_types.len() && metric_types.value(pid) == 2;
                    any_sum |= is_sum;
                    all_sum &= is_sum;
                }
                if !any_sum {
                    return Ok(batch.clone());
                }
                if all_sum {
                    let mut arr = arrow::array::TimestampNanosecondArray::from_value(
                        start_timestamp_ns,
                        batch.num_rows(),
                    );
                    if let Some(ref tz) = tz {
                        arr = arr.with_timezone(Arc::clone(tz));
                    }
                    (Arc::new(arr), false)
                } else {
                    let mut arr: arrow::array::TimestampNanosecondArray = (0..parent_ids.len())
                        .map(|i| {
                            let pid = parent_ids.value(i) as usize;
                            (pid < metric_types.len() && metric_types.value(pid) == 2)
                                .then_some(start_timestamp_ns)
                        })
                        .collect();
                    if let Some(ref tz) = tz {
                        arr = arr.with_timezone(Arc::clone(tz));
                    }
                    (Arc::new(arr), true)
                }
            } else {
                return Ok(batch.clone());
            }
        } else {
            let mut arr = arrow::array::TimestampNanosecondArray::from_value(
                start_timestamp_ns,
                batch.num_rows(),
            );
            if let Some(ref tz) = tz {
                arr = arr.with_timezone(Arc::clone(tz));
            }
            (Arc::new(arr), false)
        };

    let insert_idx = parent_idx + 1;
    let mut fields: Vec<arrow::datatypes::FieldRef> = schema.fields().iter().cloned().collect();
    fields.insert(
        insert_idx,
        Arc::new(arrow::datatypes::Field::new(
            otel_arrow_dfe_pdata::schema::consts::START_TIME_UNIX_NANO,
            arrow::datatypes::DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, tz),
            nullable,
        )),
    );
    let mut columns = batch.columns().to_vec();
    columns.insert(insert_idx, start_col);
    let new_schema = Arc::new(arrow::datatypes::Schema::new_with_metadata(
        fields,
        schema.metadata().clone(),
    ));
    arrow::record_batch::RecordBatch::try_new(new_schema, columns).map_err(|err| {
        otel_arrow_dfe_pdata::error::Error::UnexpectedRecordBatchState {
            reason: err.to_string(),
        }
    })
}

// TODO(bwplotka): Remove post-decode timestamp column replacement once
// DecodeOptions::with_honor_timestamps lands in https://github.com/prometheus/scrape.rs/issues/33.
fn override_time_unix_nano(
    batch: &arrow::record_batch::RecordBatch,
    scrape_timestamp_ns: i64,
) -> Result<arrow::record_batch::RecordBatch, otel_arrow_dfe_pdata::error::Error> {
    if batch.num_rows() == 0 {
        return Ok(batch.clone());
    }
    let schema = batch.schema();
    let Ok(col_idx) = schema.index_of(otel_arrow_dfe_pdata::schema::consts::TIME_UNIX_NANO) else {
        return Ok(batch.clone());
    };
    let tz = match schema.field(col_idx).data_type() {
        arrow::datatypes::DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, tz) => {
            tz.clone()
        }
        _ => None,
    };
    let mut ts_col =
        arrow::array::TimestampNanosecondArray::from_value(scrape_timestamp_ns, batch.num_rows());
    if let Some(tz) = tz {
        ts_col = ts_col.with_timezone(tz);
    }
    let mut columns = batch.columns().to_vec();
    columns[col_idx] = Arc::new(ts_col);
    arrow::record_batch::RecordBatch::try_new(schema, columns).map_err(|err| {
        otel_arrow_dfe_pdata::error::Error::UnexpectedRecordBatchState {
            reason: err.to_string(),
        }
    })
}

/// Local HTTP scraper and `ArrowDecoder` state for assigned targets.
struct TargetScraper {
    client: reqwest::Client,
    job_name: String,
    accept_header: &'static str,
    honor_timestamps: bool,
    max_data_points_per_batch: usize,
    max_body_bytes: usize,
    start_timestamp_ns: i64,
    decoder: ArrowDecoder,
}

impl TargetScraper {
    fn new(config: &Config, receiver_id: NodeId) -> Result<Self, Error> {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let client = reqwest::Client::builder()
            .timeout(config.scrape_timeout)
            .build()
            .map_err(|err| Error::ReceiverError {
                receiver: receiver_id,
                kind: ReceiverErrorKind::Configuration,
                error: format!("failed to build HTTP scrape client: {err}"),
                source_detail: String::new(),
            })?;
        // Subtract 1s from scraper creation time so the first scrape at `initial_delay = 0s`
        // strictly satisfies `start_time_unix_nano < time_unix_nano` on cumulative metrics.
        let start_timestamp_ns = current_timestamp_ms()
            .saturating_sub(1_000)
            .saturating_mul(1_000_000);
        Ok(Self {
            client,
            job_name: config.job_name.clone(),
            accept_header: build_accept_header(config.enable_open_metrics),
            honor_timestamps: config.honor_timestamps,
            max_data_points_per_batch: config.max_data_points_per_batch,
            max_body_bytes: config.max_body_bytes,
            start_timestamp_ns,
            decoder: ArrowDecoder::new(),
        })
    }

    async fn scrape_target(
        &mut self,
        target_url: &Url,
        ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
        effect_handler: &local::EffectHandler<OtapPdata>,
        telemetry: &mut ReceiverTelemetry,
    ) -> ControlFlow<Result<TerminalState, Error>> {
        let started = StdInstant::now();
        if let Some(metrics) = telemetry.metrics.as_mut() {
            metrics.scrapes_started.add(1);
        }
        let pull_attempt = telemetry.pull.processing();
        let pull_cancel = pull_attempt.cancellation_handle();

        // TODO(bwplotka): Delegate single-target HTTP fetch, compression negotiation, and scrape
        // headers to scrape.rs once https://github.com/prometheus/scrape.rs/issues/36 lands.
        let send_res = match poll_with_control(
            self.client
                .get(target_url.clone())
                .header(ACCEPT, self.accept_header)
                .send(),
            ctrl_msg_recv,
            effect_handler,
            telemetry,
            Some(&pull_cancel),
        )
        .await
        {
            ControlFlow::Continue(res) => res,
            ControlFlow::Break(exit) => return ControlFlow::Break(exit),
        };
        let mut response = match send_res {
            Ok(resp) => resp,
            Err(err) => {
                if let Some(metrics) = telemetry.metrics.as_mut() {
                    metrics.scrapes_failed.add(1);
                    metrics.scrape_duration_ns.record(elapsed_nanos(started));
                }
                otel_warn!(
                    "prometheus_receiver.http_error",
                    target = target_url.as_str(),
                    error = err.to_string(),
                    message = "HTTP scrape request failed"
                );
                return ControlFlow::Continue(());
            }
        };

        if !response.status().is_success() {
            if let Some(metrics) = telemetry.metrics.as_mut() {
                metrics.scrapes_failed.add(1);
                metrics.scrape_duration_ns.record(elapsed_nanos(started));
            }
            otel_warn!(
                "prometheus_receiver.bad_status",
                target = target_url.as_str(),
                status = response.status().as_u16(),
                message = "target returned non-2xx HTTP status"
            );
            return ControlFlow::Continue(());
        }

        let content_length = response.content_length();
        if let Some(len) = content_length
            && len > u64::try_from(self.max_body_bytes).unwrap_or(u64::MAX)
        {
            if let Some(metrics) = telemetry.metrics.as_mut() {
                metrics.scrapes_failed.add(1);
                metrics.scrape_duration_ns.record(elapsed_nanos(started));
            }
            otel_warn!(
                "prometheus_receiver.body_too_large",
                target = target_url.as_str(),
                content_length = len,
                max_body_bytes = u64::try_from(self.max_body_bytes).unwrap_or(u64::MAX),
                message = "scrape response Content-Length exceeds max_body_bytes"
            );
            return ControlFlow::Continue(());
        }

        let format = resolve_response_format(
            response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|val| val.to_str().ok()),
        );

        let initial_cap = content_length
            .and_then(|len| usize::try_from(len).ok())
            .map_or(0, |len| len.min(self.max_body_bytes));
        let mut body = Vec::with_capacity(initial_cap);
        loop {
            let chunk_res = match poll_with_control(
                response.chunk(),
                ctrl_msg_recv,
                effect_handler,
                telemetry,
                Some(&pull_cancel),
            )
            .await
            {
                ControlFlow::Continue(res) => res,
                ControlFlow::Break(exit) => return ControlFlow::Break(exit),
            };
            match chunk_res {
                Ok(Some(chunk)) => {
                    if body.len().saturating_add(chunk.len()) > self.max_body_bytes {
                        if let Some(metrics) = telemetry.metrics.as_mut() {
                            metrics.scrapes_failed.add(1);
                            metrics.scrape_duration_ns.record(elapsed_nanos(started));
                        }
                        otel_warn!(
                            "prometheus_receiver.body_too_large",
                            target = target_url.as_str(),
                            max_body_bytes = u64::try_from(self.max_body_bytes).unwrap_or(u64::MAX),
                            message = "scrape response body exceeds max_body_bytes"
                        );
                        return ControlFlow::Continue(());
                    }
                    body.extend_from_slice(&chunk);
                }
                Ok(None) => break,
                Err(err) => {
                    if let Some(metrics) = telemetry.metrics.as_mut() {
                        metrics.scrapes_failed.add(1);
                        metrics.scrape_duration_ns.record(elapsed_nanos(started));
                    }
                    otel_warn!(
                        "prometheus_receiver.body_error",
                        target = target_url.as_str(),
                        error = err.to_string(),
                        message = "failed to read scrape response body"
                    );
                    return ControlFlow::Continue(());
                }
            }
        }

        let scrape_timestamp_ms = current_timestamp_ms();
        let decode_opts = DecodeOptions::new()
            .with_timestamp_ms(scrape_timestamp_ms)
            .with_max_data_points_per_batch(self.max_data_points_per_batch);
        let resource_transform = build_target_resource_transform(&self.job_name, target_url);
        let mut batches = Vec::new();

        // TODO(bwplotka): Switch to cooperative incremental decoding so large payloads yield to
        // the local runtime once https://github.com/prometheus/scrape.rs/issues/27 lands.
        if self.max_data_points_per_batch == 0 {
            match self.decoder.decode(body.as_slice(), &format, &decode_opts) {
                Ok(records) => {
                    if let Some(metrics) = telemetry.metrics.as_mut() {
                        metrics
                            .data_points_decoded
                            .add(records.num_data_points() as u64);
                    }
                    if !records.is_empty() {
                        match records_to_otap_with_resource(
                            &records,
                            self.honor_timestamps,
                            scrape_timestamp_ms,
                            Some(self.start_timestamp_ns),
                            Some(&resource_transform),
                        ) {
                            Ok(otap) => {
                                batches.push(OtapPdata::new(Context::default(), otap.into()));
                            }
                            Err(err) => {
                                if let Some(metrics) = telemetry.metrics.as_mut() {
                                    metrics.scrapes_failed.add(1);
                                    metrics.scrape_duration_ns.record(elapsed_nanos(started));
                                }
                                otel_warn!(
                                    "prometheus_receiver.otap_conversion_error",
                                    target = target_url.as_str(),
                                    error = err.to_string(),
                                    message = "failed to convert decoded Arrow batch into OtapArrowRecords"
                                );
                                return ControlFlow::Continue(());
                            }
                        }
                    }
                }
                Err(err) => {
                    if let Some(metrics) = telemetry.metrics.as_mut() {
                        metrics.scrapes_failed.add(1);
                        metrics.scrape_duration_ns.record(elapsed_nanos(started));
                    }
                    otel_warn!(
                        "prometheus_receiver.decode_error",
                        target = target_url.as_str(),
                        error = err.to_string(),
                        message = "failed to decode exposition payload"
                    );
                    return ControlFlow::Continue(());
                }
            }
        } else {
            for chunk_res in self
                .decoder
                .decode_stream(body.as_slice(), &format, &decode_opts)
            {
                match chunk_res {
                    Ok(records) => {
                        if let Some(metrics) = telemetry.metrics.as_mut() {
                            metrics
                                .data_points_decoded
                                .add(records.num_data_points() as u64);
                        }
                        if !records.is_empty() {
                            match records_to_otap_with_resource(
                                &records,
                                self.honor_timestamps,
                                scrape_timestamp_ms,
                                Some(self.start_timestamp_ns),
                                Some(&resource_transform),
                            ) {
                                Ok(otap) => {
                                    batches.push(OtapPdata::new(Context::default(), otap.into()));
                                }
                                Err(err) => {
                                    if let Some(metrics) = telemetry.metrics.as_mut() {
                                        metrics.scrapes_failed.add(1);
                                        metrics.scrape_duration_ns.record(elapsed_nanos(started));
                                    }
                                    otel_warn!(
                                        "prometheus_receiver.otap_conversion_error",
                                        target = target_url.as_str(),
                                        error = err.to_string(),
                                        message = "failed to convert decoded Arrow batch into OtapArrowRecords"
                                    );
                                    return ControlFlow::Continue(());
                                }
                            }
                        }
                    }
                    Err(err) => {
                        if let Some(metrics) = telemetry.metrics.as_mut() {
                            metrics.scrapes_failed.add(1);
                            metrics.scrape_duration_ns.record(elapsed_nanos(started));
                        }
                        otel_warn!(
                            "prometheus_receiver.decode_error",
                            target = target_url.as_str(),
                            error = err.to_string(),
                            message = "failed to decode streaming exposition chunk"
                        );
                        return ControlFlow::Continue(());
                    }
                }
                tokio::task::consume_budget().await;
            }
        }

        if let Some(metrics) = telemetry.metrics.as_mut() {
            metrics.scrapes_completed.add(1);
            metrics.scrape_duration_ns.record(elapsed_nanos(started));
        }
        pull_attempt.success();

        let mut drain_deadline: Option<StdInstant> = None;
        for pdata in batches {
            match send_otap_batch(
                pdata,
                &mut drain_deadline,
                ctrl_msg_recv,
                effect_handler,
                telemetry,
            )
            .await
            {
                SendOutcome::Sent => {}
                SendOutcome::Failed => break,
                SendOutcome::DrainTimedOut(deadline) => {
                    drain_deadline = Some(deadline);
                    break;
                }
                SendOutcome::Exit(exit) => return ControlFlow::Break(exit),
            }
        }

        if let Some(deadline) = drain_deadline {
            if let Err(err) = effect_handler.notify_receiver_drained().await {
                return ControlFlow::Break(Err(err));
            }
            return ControlFlow::Break(Ok(terminal_state(deadline, telemetry)));
        }
        ControlFlow::Continue(())
    }
}

async fn handle_control_msg(
    msg: Result<NodeControlMsg<OtapPdata>, RecvError>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    telemetry: &mut ReceiverTelemetry,
    cancel_on_exit: Option<&ReceiverPullCancellation>,
) -> ControlFlow<Result<TerminalState, Error>> {
    match msg {
        Ok(NodeControlMsg::CollectTelemetry {
            mut metrics_reporter,
        }) => {
            if let Some(metrics) = telemetry.metrics.as_mut() {
                let _ = metrics_reporter.report(metrics);
            }
            let _ = telemetry.pull.report(&mut metrics_reporter);
            ControlFlow::Continue(())
        }
        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
            otel_info!("prometheus_receiver.drain_ingress");
            if let Some(cancel) = cancel_on_exit {
                cancel.cancel();
            }
            if let Err(err) = effect_handler.notify_receiver_drained().await {
                return ControlFlow::Break(Err(err));
            }
            ControlFlow::Break(Ok(terminal_state(deadline, telemetry)))
        }
        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
            otel_info!("prometheus_receiver.shutdown");
            if let Some(cancel) = cancel_on_exit {
                cancel.cancel();
            }
            ControlFlow::Break(Ok(terminal_state(deadline, telemetry)))
        }
        Err(err) => {
            if let Some(cancel) = cancel_on_exit {
                cancel.cancel();
            }
            ControlFlow::Break(Err(Error::ChannelRecvError(err)))
        }
        Ok(_) => ControlFlow::Continue(()),
    }
}

async fn poll_with_control<F, T>(
    fut: F,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    telemetry: &mut ReceiverTelemetry,
    cancel_on_exit: Option<&ReceiverPullCancellation>,
) -> ControlFlow<Result<TerminalState, Error>, T>
where
    F: Future<Output = T>,
{
    let mut fut = std::pin::pin!(fut);
    loop {
        tokio::select! {
            biased;

            msg = ctrl_msg_recv.recv() => {
                if let ControlFlow::Break(exit) =
                    handle_control_msg(msg, effect_handler, telemetry, cancel_on_exit).await
                {
                    return ControlFlow::Break(exit);
                }
            }

            output = &mut fut => return ControlFlow::Continue(output),
        }
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for PrometheusReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let PrometheusReceiver {
            config,
            discovery,
            mut telemetry,
        } = *self;

        let mut scraper = TargetScraper::new(&config, effect_handler.receiver_id())?;
        let mut next_due = Instant::now() + config.initial_delay;

        loop {
            tokio::select! {
                biased;

                msg = ctrl_msg_recv.recv() => {
                    if let ControlFlow::Break(exit) =
                        handle_control_msg(msg, &effect_handler, &mut telemetry, None).await
                    {
                        return exit;
                    }
                }

                _ = sleep_until(next_due) => {
                    let now = Instant::now();
                    let elapsed = now.saturating_duration_since(next_due);
                    let missed_ticks = elapsed.as_nanos() / config.scrape_interval.as_nanos() + 1;
                    if missed_ticks > 1 {
                        let skipped = u64::try_from(missed_ticks - 1).unwrap_or(u64::MAX);
                        if let Some(metrics) = telemetry.metrics.as_mut() {
                            metrics.scrapes_overrun.add(skipped);
                        }
                        otel_warn!(
                            "prometheus_receiver.scrape_cycle_overrun",
                            skipped_ticks = skipped,
                            scrape_interval_ms =
                                u64::try_from(config.scrape_interval.as_millis()).unwrap_or(u64::MAX),
                            message = "scrape cycle overran scrape_interval; skipping missed intervals"
                        );
                    }
                    let advance = config
                        .scrape_interval
                        .saturating_mul(u32::try_from(missed_ticks).unwrap_or(u32::MAX));
                    next_due += advance;
                    if next_due <= now {
                        next_due = now + config.scrape_interval;
                    }

                    let (targets, dns_failures) = match poll_with_control(
                        discovery.resolve_targets_counted(),
                        &mut ctrl_msg_recv,
                        &effect_handler,
                        &mut telemetry,
                        None,
                    )
                    .await
                    {
                        ControlFlow::Continue(resolved) => resolved,
                        ControlFlow::Break(exit) => return exit,
                    };
                    if dns_failures > 0
                        && let Some(metrics) = telemetry.metrics.as_mut()
                    {
                        metrics.dns_lookups_failed.add(dns_failures);
                    }
                    for target_url in &targets {
                        if let ControlFlow::Break(exit) = scraper
                            .scrape_target(
                                target_url,
                                &mut ctrl_msg_recv,
                                &effect_handler,
                                &mut telemetry,
                            )
                            .await
                        {
                            return exit;
                        }
                        tokio::task::consume_budget().await;
                    }
                }
            }
        }
    }
}

enum SendOutcome {
    Sent,
    Failed,
    DrainTimedOut(StdInstant),
    Exit(Result<TerminalState, Error>),
}

async fn send_otap_batch(
    pdata: OtapPdata,
    drain_deadline: &mut Option<StdInstant>,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    telemetry: &mut ReceiverTelemetry,
) -> SendOutcome {
    if let Some(deadline) = *drain_deadline
        && StdInstant::now() >= deadline
    {
        return SendOutcome::DrainTimedOut(deadline);
    }
    let send_result = match effect_handler.try_send_message_with_source_node(pdata) {
        Ok(()) => Ok(()),
        Err(TypedError::ChannelSendError(SendError::Full(pdata))) => {
            // NOTE: Downstream consumers run on the same single-threaded LocalSet, so when the
            // channel fills during multi-chunk decoding we must await capacity to let them poll
            // while still servicing control messages and respecting any active drain deadline.
            let mut send = Box::pin(effect_handler.send_message_with_source_node(pdata));
            loop {
                tokio::select! {
                    biased;

                    _ = async {
                        if let Some(deadline) = *drain_deadline {
                            sleep_until(Instant::from_std(deadline)).await;
                        } else {
                            std::future::pending::<()>().await;
                        }
                    }, if drain_deadline.is_some() => {
                        let deadline = drain_deadline.expect("drain_deadline checked Some");
                        return SendOutcome::DrainTimedOut(deadline);
                    }

                    msg = ctrl_msg_recv.recv() => {
                        match msg {
                            Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                                otel_info!("prometheus_receiver.drain_ingress");
                                *drain_deadline = Some(
                                    drain_deadline.map_or(deadline, |prev| prev.min(deadline)),
                                );
                            }
                            other => {
                                if let ControlFlow::Break(exit) =
                                    handle_control_msg(other, effect_handler, telemetry, None).await
                                {
                                    return SendOutcome::Exit(exit);
                                }
                            }
                        }
                    }

                    result = send.as_mut() => break result,
                }
            }
        }
        Err(other) => Err(other),
    };

    match send_result {
        Ok(()) => {
            if let Some(metrics) = telemetry.metrics.as_mut() {
                metrics.batches_sent.add(1);
            }
            SendOutcome::Sent
        }
        Err(err) => {
            if let Some(metrics) = telemetry.metrics.as_mut() {
                metrics.send_failures.add(1);
            }
            otel_warn!(
                "prometheus_receiver.send_error",
                error = err.to_string(),
                message = "failed to send scraped metrics batch downstream"
            );
            SendOutcome::Failed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::control::{RuntimeControlMsg, runtime_ctrl_msg_channel};
    use otel_arrow_dfe_engine::local::receiver::Receiver as _;
    use otel_arrow_dfe_engine::message::Sender;
    use otel_arrow_dfe_engine::testing::{
        setup_test_runtime, test_node, test_pipeline_runtime_services,
    };
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::testing::round_trip::otap_to_otlp;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    struct TestReceiverHarness {
        ctrl_chan: local::ControlChannel<OtapPdata>,
        ctrl_tx: otel_arrow_dfe_channel::mpsc::Sender<NodeControlMsg<OtapPdata>>,
        effect_handler: local::EffectHandler<OtapPdata>,
        out_rx: otel_arrow_dfe_channel::mpsc::Receiver<OtapPdata>,
        runtime_ctrl_rx: otel_arrow_dfe_engine::control::RuntimeCtrlMsgReceiver<OtapPdata>,
    }

    fn build_test_harness(channel_capacity: usize) -> TestReceiverHarness {
        let (out_tx, out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(channel_capacity);
        let mut senders = HashMap::new();
        let _ = senders.insert(
            "".into(),
            Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                out_tx,
            )),
        );
        let (pipe_tx, runtime_ctrl_rx) = runtime_ctrl_msg_channel(8);
        let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
        let effect_handler = local::EffectHandler::new(
            test_node("prometheus_test"),
            senders,
            None,
            pipe_tx,
            reporter,
            test_pipeline_runtime_services(),
        );
        let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
        let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
            otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
        );
        let ctrl_chan = local::ControlChannel::new(ctrl_rx);
        TestReceiverHarness {
            ctrl_chan,
            ctrl_tx,
            effect_handler,
            out_rx,
            runtime_ctrl_rx,
        }
    }

    /// Scenario: configuration specifies a zero scrape interval.
    /// Guarantees: validation rejects the configuration before starting the receiver.
    #[test]
    fn rejects_zero_scrape_interval() {
        let config = Config {
            scrape_interval: Duration::ZERO,
            targets: vec!["127.0.0.1:9001".to_owned()],
            ..Config::default()
        };
        assert!(matches!(
            validate_config(&config),
            Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    /// Scenario: configuration specifies a scrape timeout larger than the scrape interval.
    /// Guarantees: validation rejects overlapping scrape timeouts.
    #[test]
    fn rejects_timeout_exceeding_interval() {
        let config = Config {
            scrape_interval: Duration::from_secs(5),
            scrape_timeout: Duration::from_secs(10),
            targets: vec!["127.0.0.1:9001".to_owned()],
            ..Config::default()
        };
        assert!(matches!(
            validate_config(&config),
            Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    /// Scenario: configuration omits both `targets` and `dns_sd_targets`.
    /// Guarantees: validation rejects target-less receiver configurations.
    #[test]
    fn rejects_empty_targets() {
        let config = Config::default();
        assert!(matches!(
            validate_config(&config),
            Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    /// Scenario: bare `host:port` and full URL targets are normalized.
    /// Guarantees: `normalize_target_url` prepends `http://` and appends `metrics_path` for bare endpoints.
    #[test]
    fn normalizes_bare_host_port_targets() {
        let bare = normalize_target_url("127.0.0.1:9001", "/metrics").expect("valid bare target");
        assert_eq!(bare.as_str(), "http://127.0.0.1:9001/metrics");

        let full = normalize_target_url("http://localhost:9090/custom", "/metrics")
            .expect("valid full url");
        assert_eq!(full.as_str(), "http://localhost:9090/custom");
    }

    /// Scenario: the receiver factory is invoked on a pipeline with more than one core allocated.
    /// Guarantees: `create_prometheus_receiver` rejects multi-core source pipelines with
    /// `InvalidUserConfig` so targets cannot be scraped multiple times without coordination.
    #[test]
    fn factory_rejects_multi_core_pipeline() {
        let registry = otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new();
        let controller = otel_arrow_dfe_engine::context::ControllerContext::new(registry);
        let pipeline =
            controller.pipeline_context_with("test-group".into(), "test-pipeline".into(), 0, 2, 0);
        let mut node_config = NodeUserConfig::new_receiver_config(PROMETHEUS_RECEIVER_URN);
        node_config.config = serde_json::json!({
            "targets": ["127.0.0.1:9001"],
        });
        let receiver_config = ReceiverConfig::new("prometheus");

        let result = create_prometheus_receiver(
            pipeline,
            NodeId {
                index: 0,
                name: "prometheus".into(),
            },
            Arc::new(node_config),
            &receiver_config,
        );

        let Err(err) = result else {
            panic!("prometheus receiver must reject multi-core source pipelines");
        };
        assert!(
            err.to_string().contains("one-core source pipeline"),
            "unexpected error: {err}"
        );
    }

    /// Scenario: a one-core source pipeline is placed on a non-zero physical CPU ID (e.g. CPU 4).
    /// Guarantees: factory construction succeeds and `TargetDiscovery` retains every configured
    /// static and DNS target without filtering targets by OS CPU ID.
    #[tokio::test]
    async fn single_core_on_non_zero_cpu_id_retains_all_targets() {
        let registry = otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new();
        let controller = otel_arrow_dfe_engine::context::ControllerContext::new(registry);
        let pipeline =
            controller.pipeline_context_with("test-group".into(), "test-pipeline".into(), 4, 1, 0);
        let raw_config = serde_json::json!({
            "targets": ["127.0.0.1:9001", "127.0.0.1:9002"],
            "dns_sd_targets": ["127.0.0.1:9003"],
        });
        let mut node_config = NodeUserConfig::new_receiver_config(PROMETHEUS_RECEIVER_URN);
        node_config.config = raw_config.clone();
        let receiver_config = ReceiverConfig::new("prometheus");

        let _ = create_prometheus_receiver(
            pipeline,
            NodeId {
                index: 0,
                name: "prometheus".into(),
            },
            Arc::new(node_config),
            &receiver_config,
        )
        .expect("single-core pipeline on CPU 4 must be accepted");

        let receiver = PrometheusReceiver::from_config(&raw_config).expect("valid config");
        let resolved: Vec<String> = receiver
            .discovery
            .resolve_targets()
            .await
            .into_iter()
            .map(|url| url.to_string())
            .collect();
        assert_eq!(
            resolved,
            vec![
                "http://127.0.0.1:9001/metrics",
                "http://127.0.0.1:9002/metrics",
                "http://127.0.0.1:9003/metrics",
            ]
        );
    }

    /// Scenario: an HTTP target serves an `avalanche`-style payload with counters, gauges,
    /// classic histograms, and summaries.
    /// Guarantees: `ArrowDecoder` + `records_to_otap` produces valid `OtapArrowRecords` that
    /// convert cleanly into OTLP metrics (`Sum`, `Gauge`, `Histogram`, and `Summary`).
    #[tokio::test]
    async fn scrapes_and_converts_avalanche_payload_to_otap_and_otlp() {
        let payload = concat!(
            "# HELP avalanche_metric_mmmmm_0_0 Counter metric.\n",
            "# TYPE avalanche_metric_mmmmm_0_0 counter\n",
            "avalanche_metric_mmmmm_0_0{series_id=\"0\",cycle_id=\"0\"} 42\n",
            "# HELP avalanche_gauge_0 Gauge metric.\n",
            "# TYPE avalanche_gauge_0 gauge\n",
            "avalanche_gauge_0{series_id=\"0\"} 12.5\n",
            "# HELP avalanche_histogram_0 Latency histogram.\n",
            "# TYPE avalanche_histogram_0 histogram\n",
            "avalanche_histogram_0_bucket{series_id=\"0\",le=\"0.1\"} 3\n",
            "avalanche_histogram_0_bucket{series_id=\"0\",le=\"+Inf\"} 5\n",
            "avalanche_histogram_0_sum{series_id=\"0\"} 1.2\n",
            "avalanche_histogram_0_count{series_id=\"0\"} 5\n",
            "# HELP avalanche_summary_0 Request summary.\n",
            "# TYPE avalanche_summary_0 summary\n",
            "avalanche_summary_0{series_id=\"0\",quantile=\"0.5\"} 0.2\n",
            "avalanche_summary_0{series_id=\"0\",quantile=\"0.99\"} 0.8\n",
            "avalanche_summary_0_sum{series_id=\"0\"} 2.4\n",
            "avalanche_summary_0_count{series_id=\"0\"} 10\n",
        );

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local test listener");
        let addr = listener.local_addr().expect("listener address");

        let server = tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    payload.len(),
                    payload
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });

        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let client = reqwest::Client::new();
        let url = format!("http://{addr}/metrics");
        let resp = client
            .get(&url)
            .header(ACCEPT, build_accept_header(true))
            .send()
            .await
            .expect("HTTP GET succeeds");
        let fmt = resolve_response_format(
            resp.headers()
                .get(CONTENT_TYPE)
                .and_then(|val| val.to_str().ok()),
        );
        let body = resp.bytes().await.expect("body reads");
        let _ = server.await;

        let mut decoder = ArrowDecoder::new();
        let opts = DecodeOptions::new().with_timestamp_ms(1_700_000_000_000);
        let decoded = decoder
            .decode(body.as_ref(), &fmt, &opts)
            .expect("decode succeeds");
        assert_eq!(decoded.num_metrics(), 4);
        assert_eq!(decoded.num_data_points(), 4);

        let otap =
            records_to_otap(&decoded, true, 1_700_000_000_000).expect("records_to_otap succeeds");
        let otlp = otap_to_otlp(&otap);
        let OtlpProtoMessage::Metrics(metrics_data) = otlp else {
            panic!("expected OtlpProtoMessage::Metrics");
        };
        let total_metrics: usize = metrics_data
            .resource_metrics
            .iter()
            .flat_map(|rm| &rm.scope_metrics)
            .map(|sm| sm.metrics.len())
            .sum();
        assert_eq!(total_metrics, 4);
    }

    /// Scenario: a target serves four metrics with `max_data_points_per_batch = 1` while the
    /// downstream channel has capacity 1 on the same single-threaded `LocalSet`.
    /// Guarantees: chunk sending awaits downstream capacity instead of dropping batches so all
    /// four chunks arrive in full before shutdown.
    #[test]
    fn awaits_downstream_capacity_across_streamed_chunks() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let payload = concat!(
                "# TYPE test_metric_0 counter\ntest_metric_0 1\n",
                "# TYPE test_metric_1 counter\ntest_metric_1 2\n",
                "# TYPE test_metric_2 counter\ntest_metric_2 3\n",
                "# TYPE test_metric_3 counter\ntest_metric_3 4\n",
            );

            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind local test listener");
            let addr = listener.local_addr().expect("listener address");

            let server = tokio::task::spawn_local(async move {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let mut buf = [0u8; 1024];
                    let _ = socket.read(&mut buf).await;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        payload.len(),
                        payload
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                }
            });

            let receiver = PrometheusReceiver::new(Config {
                scrape_interval: Duration::from_secs(60),
                scrape_timeout: Duration::from_secs(5),
                initial_delay: Duration::ZERO,
                max_data_points_per_batch: 1,
                targets: vec![addr.to_string()],
                ..Config::default()
            })
            .expect("valid receiver config");

            let TestReceiverHarness {
                ctrl_chan,
                ctrl_tx,
                effect_handler,
                out_rx,
                runtime_ctrl_rx: _runtime_ctrl_rx,
            } = build_test_harness(1);

            let handle = tokio::task::spawn_local(async move {
                Box::new(receiver).start(ctrl_chan, effect_handler).await
            });

            let mut received_points = 0usize;
            for _ in 0..4 {
                let mut pdata = tokio::time::timeout(Duration::from_secs(2), out_rx.recv())
                    .await
                    .expect("chunk arrives before timeout")
                    .expect("downstream channel stays open");
                received_points += pdata.num_items();
            }
            assert_eq!(received_points, 4);

            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "test complete".to_owned(),
                })
                .expect("send shutdown control message");

            let _ = tokio::time::timeout(Duration::from_secs(2), handle)
                .await
                .expect("receiver stops before timeout")
                .expect("receiver task joins")
                .expect("receiver exits cleanly");
            let _ = server.await;
        }));
    }

    /// Scenario: `DrainIngress` arrives while the receiver is parked waiting for capacity on a
    /// full downstream channel during a multi-chunk scrape.
    /// Guarantees: pending and remaining chunks of the already-fetched scrape are sent until the
    /// drain deadline before `ReceiverDrained` is notified, and if the channel stays full past
    /// the deadline, the receiver stops waiting at the deadline and exits cleanly.
    #[test]
    fn handles_drain_ingress_while_blocked_on_full_downstream_channel() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let payload = concat!(
                "# TYPE blocked_metric_0 counter\nblocked_metric_0 10\n",
                "# TYPE blocked_metric_1 counter\nblocked_metric_1 20\n",
                "# TYPE blocked_metric_2 counter\nblocked_metric_2 30\n",
            );

            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind local test listener");
            let addr = listener.local_addr().expect("listener address");

            let server = tokio::task::spawn_local(async move {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let mut buf = [0u8; 1024];
                    let _ = socket.read(&mut buf).await;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        payload.len(),
                        payload
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                }
            });

            let receiver = PrometheusReceiver::new(Config {
                scrape_interval: Duration::from_secs(60),
                scrape_timeout: Duration::from_secs(5),
                initial_delay: Duration::ZERO,
                max_data_points_per_batch: 1,
                targets: vec![addr.to_string()],
                ..Config::default()
            })
            .expect("valid receiver config");

            let TestReceiverHarness {
                ctrl_chan,
                ctrl_tx,
                effect_handler,
                out_rx,
                mut runtime_ctrl_rx,
            } = build_test_harness(1);

            let handle = tokio::task::spawn_local(async move {
                Box::new(receiver).start(ctrl_chan, effect_handler).await
            });

            // Wait until the first chunk fills the capacity-1 channel; the receiver's send loop
            // immediately enters `send_message_with_source_node` for the second chunk in the same
            // poll without needing extra yield steps.
            tokio::time::timeout(Duration::from_secs(2), async {
                while out_rx.is_empty() {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .expect("first chunk fills downstream channel");

            ctrl_tx
                .send(NodeControlMsg::DrainIngress {
                    deadline: StdInstant::now() + Duration::from_secs(2),
                    reason: "test ingress drain".to_owned(),
                })
                .expect("send drain ingress control message");
            tokio::task::yield_now().await;

            // Drain all three chunks from the admitted scrape before the deadline expires.
            let mut received_points = 0usize;
            for _ in 0..3 {
                let mut pdata = tokio::time::timeout(Duration::from_secs(2), out_rx.recv())
                    .await
                    .expect("admitted chunk arrives before drain deadline")
                    .expect("downstream channel stays open");
                received_points += pdata.num_items();
            }
            assert_eq!(received_points, 3);

            let _ = tokio::time::timeout(Duration::from_secs(2), handle)
                .await
                .expect("receiver drains before timeout")
                .expect("receiver task joins")
                .expect("receiver exits cleanly on drain");

            let ctrl_event = tokio::time::timeout(Duration::from_secs(1), runtime_ctrl_rx.recv())
                .await
                .expect("runtime control message arrives")
                .expect("runtime control channel open");
            assert!(matches!(ctrl_event, RuntimeControlMsg::ReceiverDrained { .. }));
            let _ = server.await;
        }));
    }

    /// Scenario: an exposition payload supplies explicit per-sample timestamps across counter,
    /// gauge, classic histogram, and summary metrics while `honor_timestamps` is toggled between
    /// `true` and `false` (including through `PrometheusReceiver`).
    /// Guarantees: `honor_timestamps = true` preserves the endpoint's wire timestamps, whereas
    /// `honor_timestamps = false` overwrites every metric data point timestamp with the scrape
    /// timestamp.
    #[test]
    fn applies_honor_timestamps_setting_across_metric_types() {
        use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric::Data;

        let wire_ts_ms = 1_600_000_123_456_i64;
        let scrape_ts_ms = 1_700_000_999_000_i64;
        let payload = format!(
            concat!(
                "# TYPE ts_counter_total counter\n",
                "ts_counter_total{{job=\"app\"}} 7 {ts}\n",
                "# TYPE ts_gauge gauge\n",
                "ts_gauge{{job=\"app\"}} 3.5 {ts}\n",
                "# TYPE ts_hist histogram\n",
                "ts_hist_bucket{{le=\"1\"}} 2 {ts}\n",
                "ts_hist_bucket{{le=\"+Inf\"}} 4 {ts}\n",
                "ts_hist_sum 5.0 {ts}\n",
                "ts_hist_count 4 {ts}\n",
                "# TYPE ts_summary summary\n",
                "ts_summary{{quantile=\"0.5\"}} 1.1 {ts}\n",
                "ts_summary_sum 2.2 {ts}\n",
                "ts_summary_count 2 {ts}\n",
            ),
            ts = wire_ts_ms,
        );

        let mut decoder = ArrowDecoder::new();
        let opts = DecodeOptions::new().with_timestamp_ms(scrape_ts_ms);
        let decoded = decoder
            .decode(payload.as_bytes(), &Format::TEXT, &opts)
            .expect("decode succeeds");

        for (honor_timestamps, expected_ts_ns) in [
            (true, (wire_ts_ms as u64) * 1_000_000),
            (false, (scrape_ts_ms as u64) * 1_000_000),
        ] {
            let otap = records_to_otap(&decoded, honor_timestamps, scrape_ts_ms)
                .expect("records_to_otap succeeds");
            let OtlpProtoMessage::Metrics(metrics_data) = otap_to_otlp(&otap) else {
                panic!("expected OtlpProtoMessage::Metrics");
            };
            let mut observed = Vec::new();
            for metric in metrics_data
                .resource_metrics
                .iter()
                .flat_map(|rm| &rm.scope_metrics)
                .flat_map(|sm| &sm.metrics)
            {
                match metric.data.as_ref().expect("metric data") {
                    Data::Sum(sum) => {
                        observed.extend(sum.data_points.iter().map(|dp| dp.time_unix_nano))
                    }
                    Data::Gauge(gauge) => {
                        observed.extend(gauge.data_points.iter().map(|dp| dp.time_unix_nano))
                    }
                    Data::Histogram(hist) => {
                        observed.extend(hist.data_points.iter().map(|dp| dp.time_unix_nano))
                    }
                    Data::Summary(summary) => {
                        observed.extend(summary.data_points.iter().map(|dp| dp.time_unix_nano))
                    }
                    other => panic!("unexpected metric data variant: {other:?}"),
                }
            }
            assert_eq!(observed.len(), 4, "honor_timestamps={honor_timestamps}");
            for ts_ns in observed {
                assert_eq!(
                    ts_ns, expected_ts_ns,
                    "honor_timestamps={honor_timestamps} produced unexpected timestamp"
                );
            }
        }

        // Also exercise the live receiver loop with `honor_timestamps: false` to verify config
        // plumbing into `TargetScraper`.
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind local test listener");
            let addr = listener.local_addr().expect("listener address");

            let server = tokio::task::spawn_local(async move {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let mut buf = [0u8; 1024];
                    let _ = socket.read(&mut buf).await;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        payload.len(),
                        payload
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                }
            });

            let before_scrape_ns = (current_timestamp_ms() as u64) * 1_000_000;
            let receiver = PrometheusReceiver::new(Config {
                scrape_interval: Duration::from_secs(60),
                scrape_timeout: Duration::from_secs(5),
                initial_delay: Duration::ZERO,
                honor_timestamps: false,
                targets: vec![addr.to_string()],
                ..Config::default()
            })
            .expect("valid receiver config");

            let TestReceiverHarness {
                ctrl_chan,
                ctrl_tx,
                effect_handler,
                out_rx,
                runtime_ctrl_rx: _runtime_ctrl_rx,
            } = build_test_harness(4);

            let handle = tokio::task::spawn_local(async move {
                Box::new(receiver).start(ctrl_chan, effect_handler).await
            });

            let pdata = tokio::time::timeout(Duration::from_secs(2), out_rx.recv())
                .await
                .expect("batch arrives before timeout")
                .expect("downstream channel open");
            let otap = pdata
                .payload_ref()
                .otap_ref()
                .expect("OTAP arrow records");
            let OtlpProtoMessage::Metrics(metrics_data) = otap_to_otlp(otap) else {
                panic!("expected OtlpProtoMessage::Metrics");
            };
            for metric in metrics_data
                .resource_metrics
                .iter()
                .flat_map(|rm| &rm.scope_metrics)
                .flat_map(|sm| &sm.metrics)
            {
                if let Some(Data::Sum(sum)) = &metric.data {
                    for dp in &sum.data_points {
                        assert!(
                            dp.time_unix_nano >= before_scrape_ns,
                            "expected scrape timestamp >= {before_scrape_ns}, got {}",
                            dp.time_unix_nano
                        );
                    }
                }
            }

            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "done".to_owned(),
                })
                .expect("send shutdown");
            let _ = tokio::time::timeout(Duration::from_secs(2), handle)
                .await
                .expect("receiver stops")
                .expect("task joins")
                .expect("clean exit");
            let _ = server.await;
        }));
    }

    /// Scenario: `PrometheusReceiver` scrapes a target with `job_name = "avalanche"` and
    /// downstream pipeline processors upsert cluster and GCP resource attributes.
    /// Guarantees: `service.name`, `service.instance.id`, and downstream resource attributes are
    /// attached to the decoded `ResourceMetrics` batch, and cumulative metrics receive a valid
    /// `start_time_unix_nano < time_unix_nano`.
    #[test]
    fn attaches_target_resource_attributes_and_supports_downstream_resource_upserts() {
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value as AnyVal;
        use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric::Data;

        let payload = concat!(
            "# TYPE avalanche_counter_total counter\n",
            "avalanche_counter_total{series_id=\"0\"} 10\n",
            "# TYPE avalanche_gauge gauge\n",
            "avalanche_gauge{series_id=\"0\"} 3.5\n",
        );

        let mut decoder = ArrowDecoder::new();
        let scrape_ts_ms = 1_700_000_005_000_i64;
        let start_ts_ns = 1_700_000_000_000_000_000_i64;
        let opts = DecodeOptions::new().with_timestamp_ms(scrape_ts_ms);
        let decoded = decoder
            .decode(payload.as_bytes(), &Format::TEXT, &opts)
            .expect("decode succeeds");

        let target_url = Url::parse("http://10.52.2.15:9001/metrics").expect("valid url");
        let target_transform = build_target_resource_transform("avalanche", &target_url);
        let mut otap = records_to_otap_with_resource(
            &decoded,
            true,
            scrape_ts_ms,
            Some(start_ts_ns),
            Some(&target_transform),
        )
        .expect("records_to_otap_with_resource succeeds");

        let downstream_transform =
            AttributesTransform::default().with_upsert(UpsertTransform::new(BTreeMap::from([
                (
                    "gcp.project_id".into(),
                    LiteralValue::Str("gpe-test-1".into()),
                ),
                (
                    "k8s.namespace.name".into(),
                    LiteralValue::Str("otap-rust".into()),
                ),
            ])));
        let _ = apply_attribute_transform(
            &mut otap,
            ArrowPayloadType::ResourceAttrs,
            &downstream_transform,
            false,
        )
        .expect("downstream resource attribute upsert succeeds");

        let OtlpProtoMessage::Metrics(metrics_data) = otap_to_otlp(&otap) else {
            panic!("expected OtlpProtoMessage::Metrics");
        };
        assert_eq!(metrics_data.resource_metrics.len(), 1);
        let res = metrics_data.resource_metrics[0]
            .resource
            .as_ref()
            .expect("resource present");
        let attrs: BTreeMap<String, String> = res
            .attributes
            .iter()
            .filter_map(|kv| match kv.value.as_ref()?.value.as_ref()? {
                AnyVal::StringValue(s) => Some((kv.key.clone(), s.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(
            attrs.get("service.name").map(String::as_str),
            Some("avalanche")
        );
        assert_eq!(
            attrs.get("service.instance.id").map(String::as_str),
            Some("10.52.2.15:9001")
        );
        assert_eq!(
            attrs.get("gcp.project_id").map(String::as_str),
            Some("gpe-test-1")
        );
        assert_eq!(
            attrs.get("k8s.namespace.name").map(String::as_str),
            Some("otap-rust")
        );

        for metric in metrics_data.resource_metrics[0]
            .scope_metrics
            .iter()
            .flat_map(|sm| &sm.metrics)
        {
            match metric.data.as_ref().expect("metric data") {
                Data::Sum(sum) => {
                    assert_eq!(sum.data_points[0].start_time_unix_nano, start_ts_ns as u64);
                }
                Data::Gauge(gauge) => {
                    assert_eq!(gauge.data_points[0].start_time_unix_nano, 0);
                }
                other => panic!("unexpected metric data: {other:?}"),
            }
        }
    }

    /// Scenario: `CollectTelemetry` and `DrainIngress` arrive while an HTTP target scrape is
    /// stalled waiting for headers with a long `scrape_timeout`.
    /// Guarantees: `CollectTelemetry` is serviced without dropping the in-flight HTTP request, and
    /// `DrainIngress` cancels the in-flight HTTP request immediately without waiting for
    /// `scrape_timeout`.
    #[test]
    fn services_control_messages_during_slow_http_scrape() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind local test listener");
            let addr = listener.local_addr().expect("listener address");
            let (conn_accepted_tx, conn_accepted_rx) = tokio::sync::oneshot::channel();
            let (_hold_tx, hold_rx) = tokio::sync::oneshot::channel::<()>();

            let server = tokio::task::spawn_local(async move {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let mut buf = [0u8; 1024];
                    let _ = socket.read(&mut buf).await;
                    let _ = conn_accepted_tx.send(());
                    let _ = hold_rx.await;
                }
            });

            let receiver = PrometheusReceiver::new(Config {
                scrape_interval: Duration::from_secs(60),
                scrape_timeout: Duration::from_secs(30),
                initial_delay: Duration::ZERO,
                targets: vec![addr.to_string()],
                ..Config::default()
            })
            .expect("valid receiver config");

            let TestReceiverHarness {
                ctrl_chan,
                ctrl_tx,
                effect_handler,
                out_rx: _out_rx,
                mut runtime_ctrl_rx,
            } = build_test_harness(4);

            let handle = tokio::task::spawn_local(async move {
                Box::new(receiver).start(ctrl_chan, effect_handler).await
            });

            tokio::time::timeout(Duration::from_secs(2), conn_accepted_rx)
                .await
                .expect("HTTP request reaches server")
                .expect("server signals connection accepted");

            let (_telemetry_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(4);
            ctrl_tx
                .send(NodeControlMsg::CollectTelemetry { metrics_reporter })
                .expect("send CollectTelemetry while HTTP scrape is in flight");

            ctrl_tx
                .send(NodeControlMsg::DrainIngress {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "drain during slow scrape".to_owned(),
                })
                .expect("send DrainIngress while HTTP scrape is in flight");

            let _ = tokio::time::timeout(Duration::from_secs(2), handle)
                .await
                .expect("receiver exits without waiting for 30s scrape_timeout")
                .expect("receiver task joins")
                .expect("receiver exits cleanly");

            let ctrl_event = tokio::time::timeout(Duration::from_secs(1), runtime_ctrl_rx.recv())
                .await
                .expect("runtime control message arrives")
                .expect("runtime control channel open");
            assert!(matches!(
                ctrl_event,
                RuntimeControlMsg::ReceiverDrained { .. }
            ));
            server.abort();
        }));
    }

    /// Scenario: configuration specifies zero `max_body_bytes` or uses default batching and limit settings.
    /// Guarantees: validation rejects zero `max_body_bytes`, and default configuration bounds
    /// batch size to `expfmt::arrow::DEFAULT_MAX_DATA_POINTS_PER_BATCH` and body size to 16 MiB.
    #[test]
    fn rejects_zero_max_body_bytes() {
        let default_cfg = Config::default();
        assert_eq!(
            default_cfg.max_data_points_per_batch,
            expfmt::arrow::DEFAULT_MAX_DATA_POINTS_PER_BATCH
        );
        assert_eq!(default_cfg.max_body_bytes, 16 * 1024 * 1024);

        let zero_body = Config {
            max_body_bytes: 0,
            targets: vec!["127.0.0.1:9001".to_owned()],
            ..Config::default()
        };
        assert!(matches!(
            validate_config(&zero_body),
            Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig { .. })
        ));
    }

    /// Scenario: a target sequentially returns HTTP 500, an oversized `Content-Length` response,
    /// an oversized streaming body without `Content-Length`, and a malformed exposition payload.
    /// Guarantees: each failure increments `scrapes_failed` without incrementing
    /// `scrapes_completed` or `send_failures`, and no batches are emitted downstream.
    #[test]
    fn records_failures_for_bad_status_oversized_body_and_decode_error() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind local test listener");
            let addr = listener.local_addr().expect("listener address");

            let oversized_body =
                "# TYPE big_counter counter\nbig_counter{label=\"0123456789abcdef\"} 1\n";
            let responses = vec![
                "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_owned(),
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    oversized_body.len(),
                    oversized_body
                ),
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nConnection: close\r\n\r\n{}",
                    oversized_body
                ),
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: 23\r\nConnection: close\r\n\r\nnot_a_valid_metric ===\n"
                    .to_owned(),
            ];

            let server = tokio::task::spawn_local(async move {
                for raw_resp in responses {
                    if let Ok((mut socket, _)) = listener.accept().await {
                        let mut buf = [0u8; 1024];
                        let _ = socket.read(&mut buf).await;
                        let _ = socket.write_all(raw_resp.as_bytes()).await;
                    }
                }
            });

            let config = Config {
                scrape_interval: Duration::from_secs(60),
                scrape_timeout: Duration::from_secs(5),
                max_body_bytes: 32,
                targets: vec![addr.to_string()],
                ..Config::default()
            };
            let target_url =
                normalize_target_url(&addr.to_string(), &config.metrics_path).expect("target url");
            let mut scraper =
                TargetScraper::new(&config, test_node("prometheus_test")).expect("build scraper");

            let registry = otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new();
            let controller = otel_arrow_dfe_engine::context::ControllerContext::new(registry);
            let pipeline =
                controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
            let mut telemetry = ReceiverTelemetry {
                metrics: Some(PrometheusReceiverMetrics::register(&pipeline)),
                pull: ReceiverPullMetrics::register(&pipeline),
            };

            let TestReceiverHarness {
                mut ctrl_chan,
                ctrl_tx: _ctrl_tx,
                effect_handler,
                out_rx,
                runtime_ctrl_rx: _runtime_ctrl_rx,
            } = build_test_harness(4);

            for _ in 0..4 {
                let flow = scraper
                    .scrape_target(&target_url, &mut ctrl_chan, &effect_handler, &mut telemetry)
                    .await;
                assert!(matches!(flow, ControlFlow::Continue(())));
            }

            let metrics = telemetry.metrics.as_ref().expect("metrics registered");
            assert_eq!(metrics.scrapes_started.get(), 4);
            assert_eq!(metrics.scrapes_failed.get(), 4);
            assert_eq!(metrics.scrapes_completed.get(), 0);
            assert_eq!(metrics.batches_sent.get(), 0);
            assert_eq!(metrics.send_failures.get(), 0);
            assert!(out_rx.is_empty());

            let _ = server.await;
        }));
    }

    /// Scenario: `TargetDiscovery::resolve_targets_counted` resolves a valid static target alongside
    /// an unresolvable `.invalid` DNS service discovery target.
    /// Guarantees: the static target is preserved while the failed DNS lookup increments the
    /// returned DNS failure count.
    #[tokio::test]
    async fn counts_failed_and_empty_dns_target_lookups() {
        let config = Config {
            targets: vec!["127.0.0.1:9001".to_owned()],
            dns_sd_targets: vec!["nonexistent.prometheus.target.invalid:9001".to_owned()],
            ..Config::default()
        };
        let discovery = TargetDiscovery::from_config(&config).expect("valid discovery config");
        let (urls, dns_failures) = discovery.resolve_targets_counted().await;
        assert_eq!(
            urls.iter().map(Url::as_str).collect::<Vec<_>>(),
            vec!["http://127.0.0.1:9001/metrics"]
        );
        assert_eq!(dns_failures, 1);
    }
}
