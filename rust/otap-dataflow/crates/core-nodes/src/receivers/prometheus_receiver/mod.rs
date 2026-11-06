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
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::otap::{Metrics, OtapArrowRecords, OtapBatchStore};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::metrics::{MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry_macros::metric_set;
use reqwest::Url;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::Value;
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

/// Discovers and resolves configured static and DNS scrape target URLs.
///
/// Kept separate from [`TargetScraper`] so future shared work coordination can supply per-core
/// receiver instances with assigned targets while reusing the local scrape and decode path.
struct TargetDiscovery {
    static_urls: Vec<Url>,
    dns_sd_targets: Vec<String>,
    metrics_path: String,
}

impl TargetDiscovery {
    fn from_config(config: &Config) -> Result<Self, otel_arrow_dfe_config::error::Error> {
        let mut static_urls = Vec::with_capacity(config.targets.len());
        for target in &config.targets {
            static_urls.push(normalize_target_url(target, &config.metrics_path)?);
        }
        Ok(Self {
            static_urls,
            dns_sd_targets: config.dns_sd_targets.clone(),
            metrics_path: config.metrics_path.clone(),
        })
    }

    async fn resolve_targets(&self) -> Vec<Url> {
        let mut all_urls =
            Vec::with_capacity(self.static_urls.len() + self.dns_sd_targets.len() * 4);
        all_urls.extend(self.static_urls.iter().cloned());

        for dns_target in &self.dns_sd_targets {
            match lookup_host(dns_target.as_str()).await {
                Ok(addrs) => {
                    let mut sorted_addrs: Vec<_> = addrs.collect();
                    sorted_addrs.sort_unstable();
                    sorted_addrs.dedup();
                    for addr in sorted_addrs {
                        if let Ok(url) = Url::parse(&format!("http://{addr}{}", self.metrics_path))
                        {
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
    }
}

/// Prometheus scrape receiver.
pub struct PrometheusReceiver {
    config: Config,
    discovery: TargetDiscovery,
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
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "prometheus receiver must run in a one-core source pipeline; use \
                receiver:prometheus -> exporter:topic and fan out downstream"
                .to_owned(),
        });
    }
    let mut receiver = PrometheusReceiver::from_config(&node_config.config)?;
    receiver.metrics = Some(PrometheusReceiverMetrics::register(&pipeline));
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
            metrics: None,
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

/// Local HTTP scraper and `ArrowDecoder` state for assigned targets.
struct TargetScraper {
    client: reqwest::Client,
    accept_header: &'static str,
    max_data_points_per_batch: usize,
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
        Ok(Self {
            client,
            accept_header: build_accept_header(config.enable_open_metrics),
            max_data_points_per_batch: config.max_data_points_per_batch,
            decoder: ArrowDecoder::new(),
        })
    }

    async fn scrape_target(
        &mut self,
        target_url: &Url,
        ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
        effect_handler: &local::EffectHandler<OtapPdata>,
        metrics: &mut Option<MetricSet<PrometheusReceiverMetrics>>,
    ) -> ControlFlow<Result<TerminalState, Error>> {
        let started = StdInstant::now();
        if let Some(metrics) = metrics.as_mut() {
            metrics.scrapes_started.add(1);
        }

        let response = match self
            .client
            .get(target_url.clone())
            .header(ACCEPT, self.accept_header)
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
                return ControlFlow::Continue(());
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
            return ControlFlow::Continue(());
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
                return ControlFlow::Continue(());
            }
        };

        let decode_opts = DecodeOptions::new()
            .with_timestamp_ms(current_timestamp_ms())
            .with_max_data_points_per_batch(self.max_data_points_per_batch);

        let mut scrape_ok = true;

        if self.max_data_points_per_batch == 0 {
            match self.decoder.decode(body.as_ref(), &format, &decode_opts) {
                Ok(records) => {
                    if let Some(metrics) = metrics.as_mut() {
                        metrics
                            .data_points_decoded
                            .add(records.num_data_points() as u64);
                    }
                    if !records.is_empty() {
                        match send_decoded_records(&records, ctrl_msg_recv, effect_handler, metrics)
                            .await
                        {
                            SendOutcome::Sent => {}
                            SendOutcome::Failed => {
                                scrape_ok = false;
                            }
                            SendOutcome::Exit(exit) => return ControlFlow::Break(exit),
                        }
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
            for chunk_res in self
                .decoder
                .decode_stream(body.as_ref(), &format, &decode_opts)
            {
                match chunk_res {
                    Ok(records) => {
                        if let Some(metrics) = metrics.as_mut() {
                            metrics
                                .data_points_decoded
                                .add(records.num_data_points() as u64);
                        }
                        if !records.is_empty() {
                            match send_decoded_records(
                                &records,
                                ctrl_msg_recv,
                                effect_handler,
                                metrics,
                            )
                            .await
                            {
                                SendOutcome::Sent => {}
                                SendOutcome::Failed => {
                                    scrape_ok = false;
                                    break;
                                }
                                SendOutcome::Exit(exit) => return ControlFlow::Break(exit),
                            }
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
            if scrape_ok {
                metrics.scrapes_completed.add(1);
            } else {
                metrics.scrapes_failed.add(1);
            }
            metrics.scrape_duration_ns.record(elapsed_nanos(started));
        }
        ControlFlow::Continue(())
    }
}

async fn handle_control_msg(
    msg: Result<NodeControlMsg<OtapPdata>, RecvError>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &mut Option<MetricSet<PrometheusReceiverMetrics>>,
) -> ControlFlow<Result<TerminalState, Error>> {
    match msg {
        Ok(NodeControlMsg::CollectTelemetry {
            mut metrics_reporter,
        }) => {
            if let Some(metrics) = metrics.as_mut() {
                let _ = metrics_reporter.report(metrics);
            }
            ControlFlow::Continue(())
        }
        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
            otel_info!("prometheus_receiver.drain_ingress");
            if let Err(err) = effect_handler.notify_receiver_drained().await {
                return ControlFlow::Break(Err(err));
            }
            ControlFlow::Break(Ok(terminal_state(deadline, metrics)))
        }
        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
            otel_info!("prometheus_receiver.shutdown");
            ControlFlow::Break(Ok(terminal_state(deadline, metrics)))
        }
        Err(err) => ControlFlow::Break(Err(Error::ChannelRecvError(err))),
        Ok(_) => ControlFlow::Continue(()),
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
            mut metrics,
        } = *self;

        let mut scraper = TargetScraper::new(&config, effect_handler.receiver_id())?;
        let mut next_due = Instant::now() + config.initial_delay;

        loop {
            tokio::select! {
                biased;

                msg = ctrl_msg_recv.recv() => {
                    if let ControlFlow::Break(exit) =
                        handle_control_msg(msg, &effect_handler, &mut metrics).await
                    {
                        return exit;
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

                    let targets = discovery.resolve_targets().await;
                    for target_url in &targets {
                        if let ControlFlow::Break(exit) = scraper
                            .scrape_target(
                                target_url,
                                &mut ctrl_msg_recv,
                                &effect_handler,
                                &mut metrics,
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
    Exit(Result<TerminalState, Error>),
}

async fn send_decoded_records(
    records: &OtapMetricRecords,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &mut Option<MetricSet<PrometheusReceiverMetrics>>,
) -> SendOutcome {
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
            return SendOutcome::Failed;
        }
    };
    let pdata = OtapPdata::new(Context::default(), otap.into());
    let send_result = match effect_handler.try_send_message_with_source_node(pdata) {
        Ok(()) => Ok(()),
        Err(TypedError::ChannelSendError(SendError::Full(pdata))) => {
            // NOTE: Downstream consumers run on the same single-threaded LocalSet, so when the
            // channel fills during multi-chunk decoding we must await capacity to let them poll
            // while still servicing control messages.
            let mut send = Box::pin(effect_handler.send_message_with_source_node(pdata));
            loop {
                tokio::select! {
                    biased;

                    msg = ctrl_msg_recv.recv() => {
                        if let ControlFlow::Break(exit) =
                            handle_control_msg(msg, effect_handler, metrics).await
                        {
                            return SendOutcome::Exit(exit);
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
            if let Some(metrics) = metrics.as_mut() {
                metrics.batches_sent.add(1);
            }
            SendOutcome::Sent
        }
        Err(err) => {
            if let Some(metrics) = metrics.as_mut() {
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
    /// Guarantees: control messages are still serviced while blocked on backpressure, ingress
    /// drain is notified to the runtime, and the receiver terminates cleanly.
    #[test]
    fn handles_drain_ingress_while_blocked_on_full_downstream_channel() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let payload = concat!(
                "# TYPE blocked_metric_0 counter\nblocked_metric_0 10\n",
                "# TYPE blocked_metric_1 counter\nblocked_metric_1 20\n",
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

            // Wait until the first chunk fills the capacity-1 channel, then yield once more so
            // the second chunk parks on `send_message_with_source_node` without draining `out_rx`.
            tokio::time::timeout(Duration::from_secs(2), async {
                while out_rx.is_empty() {
                    tokio::task::yield_now().await;
                }
                tokio::task::yield_now().await;
            })
            .await
            .expect("first chunk fills downstream channel");

            ctrl_tx
                .send(NodeControlMsg::DrainIngress {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "test ingress drain".to_owned(),
                })
                .expect("send drain ingress control message");

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
}
