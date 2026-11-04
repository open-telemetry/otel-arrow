// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Prometheus scrape receiver backed by `expfmt::arrow::ArrowDecoder`.
//!
//! Scrapes configured HTTP `/metrics` endpoints (static URLs and optional headless DNS service
//! endpoints) and decodes Prometheus text (`0.0.4` / `1.0.0`) and OpenMetrics (`1.0.0`) payloads
//! directly into OTAP Arrow record batches on each core's local runtime without intermediate
//! protobuf materialization.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = PROMETHEUS_RECEIVER_URN,
    target = "otel.receiver.prometheus",
);

use async_trait::async_trait;
use expfmt::arrow::{ArrowDecoder, DecodeOptions, OtapMetricRecords};
use expfmt::format::{Format, FormatType, response_format};
use linkme::distributed_slice;
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
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::otap::{Metrics, OtapArrowRecords, OtapBatchStore};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::metrics::{MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry_macros::metric_set;
use reqwest::Url;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::Value;
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

/// Prometheus scrape receiver.
pub struct PrometheusReceiver {
    config: Config,
    static_urls: Vec<Url>,
    core_id: usize,
    num_cores: usize,
    metrics: Option<MetricSet<PrometheusReceiverMetrics>>,
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
            let mut receiver = PrometheusReceiver::from_config(
                &node_config.config,
                pipeline.core_id(),
                pipeline.num_cores().max(1),
            )?;
            receiver.metrics = Some(PrometheusReceiverMetrics::register(&pipeline));
            Ok(ReceiverWrapper::local(
                receiver,
                node,
                node_config,
                receiver_config,
            ))
        },
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: validate_prometheus_config,
};

fn validate_prometheus_config(config: &Value) -> Result<(), otel_arrow_dfe_config::error::Error> {
    let parsed: Config = serde_json::from_value(config.clone()).map_err(|err| {
        otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: err.to_string(),
        }
    })?;
    validate_config(&parsed)
}

impl PrometheusReceiver {
    /// Creates a new `PrometheusReceiver` for the given core shard.
    pub fn new(
        config: Config,
        core_id: usize,
        num_cores: usize,
    ) -> Result<Self, otel_arrow_dfe_config::error::Error> {
        validate_config(&config)?;
        let mut static_urls = Vec::with_capacity(config.targets.len());
        for target in &config.targets {
            static_urls.push(normalize_target_url(target, &config.metrics_path)?);
        }
        Ok(Self {
            config,
            static_urls,
            core_id,
            num_cores: num_cores.max(1),
            metrics: None,
        })
    }

    /// Creates a `PrometheusReceiver` from a raw JSON configuration value.
    pub fn from_config(
        config: &Value,
        core_id: usize,
        num_cores: usize,
    ) -> Result<Self, otel_arrow_dfe_config::error::Error> {
        let parsed: Config = serde_json::from_value(config.clone()).map_err(|err| {
            otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: err.to_string(),
            }
        })?;
        Self::new(parsed, core_id, num_cores)
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

fn terminal_state(
    deadline: StdInstant,
    metrics: &Option<MetricSet<PrometheusReceiverMetrics>>,
) -> TerminalState {
    if let Some(metrics) = metrics {
        TerminalState::new(deadline, [metrics.snapshot()])
    } else {
        TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, [])
    }
}

/// Resolves the HTTP `Accept` header value advertised during scrapes.
pub(crate) fn build_accept_header(enable_open_metrics: bool) -> &'static str {
    if enable_open_metrics {
        "application/openmetrics-text;version=1.0.0;escaping=allow-utf-8;q=0.6,application/openmetrics-text;version=0.0.1;q=0.5,text/plain;version=1.0.0;escaping=allow-utf-8;q=0.4,text/plain;version=0.0.4;q=0.3"
    } else {
        "text/plain;version=1.0.0;escaping=allow-utf-8;q=0.5,text/plain;version=0.0.4;q=0.4"
    }
}

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

/// Converts a decoded [`OtapMetricRecords`] batch into [`OtapArrowRecords::Metrics`].
pub(crate) fn records_to_otap(
    records: &OtapMetricRecords,
) -> Result<OtapArrowRecords, otel_arrow_dfe_pdata::error::Error> {
    let mut store = Metrics::new();
    for (payload_type, batch) in records.iter() {
        if let Ok(wire_type) = ArrowPayloadType::try_from(payload_type.as_i32()) {
            store.set(wire_type, batch.clone())?;
        }
    }
    Ok(OtapArrowRecords::Metrics(store))
}

async fn resolve_core_targets(
    static_urls: &[Url],
    dns_sd_targets: &[String],
    metrics_path: &str,
    core_id: usize,
    num_cores: usize,
) -> Vec<Url> {
    let mut all_urls = Vec::with_capacity(static_urls.len() + dns_sd_targets.len() * 4);
    all_urls.extend(static_urls.iter().cloned());

    for dns_target in dns_sd_targets {
        match lookup_host(dns_target.as_str()).await {
            Ok(addrs) => {
                let mut sorted_addrs: Vec<_> = addrs.collect();
                sorted_addrs.sort_unstable();
                sorted_addrs.dedup();
                for addr in sorted_addrs {
                    if let Ok(url) = Url::parse(&format!("http://{addr}{metrics_path}")) {
                        all_urls.push(url);
                    }
                }
            }
            Err(err) => {
                otel_warn!(
                    "prometheus_receiver.dns_lookup_failed",
                    target = dns_target.as_str(),
                    error = err.to_string(),
                    message = "failed to resolve dns_sd_targets entry"
                );
            }
        }
    }

    all_urls
        .into_iter()
        .enumerate()
        .filter_map(|(idx, url)| (idx % num_cores == core_id).then_some(url))
        .collect()
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
            static_urls,
            core_id,
            num_cores,
            mut metrics,
        } = *self;

        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let client = reqwest::Client::builder()
            .timeout(config.scrape_timeout)
            .build()
            .map_err(|err| Error::ReceiverError {
                receiver: effect_handler.receiver_id(),
                kind: ReceiverErrorKind::Configuration,
                error: format!("failed to build HTTP scrape client: {err}"),
                source_detail: String::new(),
            })?;

        let accept_header = build_accept_header(config.enable_open_metrics);
        let mut decoder = ArrowDecoder::new();
        let mut next_due = Instant::now() + config.initial_delay;

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
                            otel_info!("prometheus_receiver.drain_ingress");
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(terminal_state(deadline, &metrics));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            otel_info!("prometheus_receiver.shutdown");
                            return Ok(terminal_state(deadline, &metrics));
                        }
                        Err(err) => return Err(Error::ChannelRecvError(err)),
                        _ => {}
                    }
                }

                _ = sleep_until(next_due) => {
                    let now = Instant::now();
                    let elapsed = now.saturating_duration_since(next_due);
                    let missed_ticks = elapsed.as_nanos() / config.scrape_interval.as_nanos() + 1;
                    let advance = config
                        .scrape_interval
                        .saturating_mul(u32::try_from(missed_ticks).unwrap_or(u32::MAX));
                    next_due += advance;
                    if next_due <= now {
                        next_due = now + config.scrape_interval;
                    }

                    let targets = resolve_core_targets(
                        &static_urls,
                        &config.dns_sd_targets,
                        &config.metrics_path,
                        core_id,
                        num_cores,
                    )
                    .await;

                    for target_url in targets {
                        let started = StdInstant::now();
                        if let Some(metrics) = metrics.as_mut() {
                            metrics.scrapes_started.add(1);
                        }

                        let response = match client
                            .get(target_url.clone())
                            .header(ACCEPT, accept_header)
                            .send()
                            .await
                        {
                            Ok(resp) => resp,
                            Err(err) => {
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.scrapes_failed.add(1);
                                    metrics.scrape_duration_ns.record(elapsed_nanos(started));
                                }
                                otel_warn!(
                                    "prometheus_receiver.http_error",
                                    target = target_url.as_str(),
                                    error = err.to_string(),
                                    message = "HTTP scrape request failed"
                                );
                                continue;
                            }
                        };

                        if !response.status().is_success() {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.scrapes_failed.add(1);
                                metrics.scrape_duration_ns.record(elapsed_nanos(started));
                            }
                            otel_warn!(
                                "prometheus_receiver.bad_status",
                                target = target_url.as_str(),
                                status = response.status().as_u16(),
                                message = "target returned non-2xx HTTP status"
                            );
                            continue;
                        }

                        let format = resolve_response_format(
                            response
                                .headers()
                                .get(CONTENT_TYPE)
                                .and_then(|val| val.to_str().ok()),
                        );

                        let body = match response.bytes().await {
                            Ok(bytes) => bytes,
                            Err(err) => {
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.scrapes_failed.add(1);
                                    metrics.scrape_duration_ns.record(elapsed_nanos(started));
                                }
                                otel_warn!(
                                    "prometheus_receiver.body_error",
                                    target = target_url.as_str(),
                                    error = err.to_string(),
                                    message = "failed to read scrape response body"
                                );
                                continue;
                            }
                        };

                        let decode_opts = DecodeOptions::new()
                            .with_timestamp_ms(current_timestamp_ms())
                            .with_max_data_points_per_batch(config.max_data_points_per_batch);

                        let mut scrape_ok = true;
                        let mut decoded_points = 0u64;

                        if config.max_data_points_per_batch == 0 {
                            match decoder.decode(body.as_ref(), &format, &decode_opts) {
                                Ok(records) => {
                                    decoded_points += records.num_data_points() as u64;
                                    if !records.is_empty()
                                        && !send_decoded_records(
                                            &records,
                                            &effect_handler,
                                            &mut metrics,
                                        )
                                    {
                                        scrape_ok = false;
                                    }
                                }
                                Err(err) => {
                                    scrape_ok = false;
                                    otel_warn!(
                                        "prometheus_receiver.decode_error",
                                        target = target_url.as_str(),
                                        error = err.to_string(),
                                        message = "failed to decode exposition payload"
                                    );
                                }
                            }
                        } else {
                            for chunk_res in
                                decoder.decode_stream(body.as_ref(), &format, &decode_opts)
                            {
                                match chunk_res {
                                    Ok(records) => {
                                        decoded_points += records.num_data_points() as u64;
                                        if !records.is_empty()
                                            && !send_decoded_records(
                                                &records,
                                                &effect_handler,
                                                &mut metrics,
                                            )
                                        {
                                            scrape_ok = false;
                                            break;
                                        }
                                    }
                                    Err(err) => {
                                        scrape_ok = false;
                                        otel_warn!(
                                            "prometheus_receiver.decode_error",
                                            target = target_url.as_str(),
                                            error = err.to_string(),
                                            message = "failed to decode streaming exposition chunk"
                                        );
                                        break;
                                    }
                                }
                            }
                        }

                        if let Some(metrics) = metrics.as_mut() {
                            metrics.data_points_decoded.add(decoded_points);
                            if scrape_ok {
                                metrics.scrapes_completed.add(1);
                            } else {
                                metrics.scrapes_failed.add(1);
                            }
                            metrics.scrape_duration_ns.record(elapsed_nanos(started));
                        }

                        tokio::task::consume_budget().await;
                    }
                }
            }
        }
    }
}

fn send_decoded_records(
    records: &OtapMetricRecords,
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &mut Option<MetricSet<PrometheusReceiverMetrics>>,
) -> bool {
    let otap = match records_to_otap(records) {
        Ok(otap) => otap,
        Err(err) => {
            if let Some(metrics) = metrics.as_mut() {
                metrics.send_failures.add(1);
            }
            otel_warn!(
                "prometheus_receiver.otap_conversion_error",
                error = err.to_string(),
                message = "failed to convert decoded Arrow batch into OtapArrowRecords"
            );
            return false;
        }
    };
    let pdata = OtapPdata::new(Context::default(), otap.into());
    match effect_handler.try_send_message_with_source_node(pdata) {
        Ok(()) => {
            if let Some(metrics) = metrics.as_mut() {
                metrics.batches_sent.add(1);
            }
            true
        }
        Err(TypedError::ChannelSendError(_)) => {
            if let Some(metrics) = metrics.as_mut() {
                metrics.send_failures.add(1);
            }
            otel_warn!(
                "prometheus_receiver.dropped_backpressure",
                message = "scraped metrics batch dropped due to downstream backpressure"
            );
            false
        }
        Err(other) => {
            if let Some(metrics) = metrics.as_mut() {
                metrics.send_failures.add(1);
            }
            otel_warn!(
                "prometheus_receiver.send_error",
                error = other.to_string(),
                message = "failed to send scraped metrics batch downstream"
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::testing::round_trip::otap_to_otlp;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

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

        let otap = records_to_otap(&decoded).expect("records_to_otap succeeds");
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
}
