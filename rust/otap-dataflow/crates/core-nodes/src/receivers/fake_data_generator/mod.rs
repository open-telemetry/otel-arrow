// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A fake data generator receiver.
//! Note: This receiver will be replaced in the future with a more sophisticated implementation.
//!

use crate::receivers::fake_data_generator::config::Config;
use async_trait::async_trait;
use linkme::distributed_slice;
use metrics::FakeSignalReceiverMetrics;
use otap_df_channel::error::{RecvError, SendError};
use otap_df_config::node::NodeUserConfig;
use otap_df_config::transport_headers::{TransportHeader, TransportHeaders};
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::CallData;
use otap_df_engine::error::{Error, ReceiverErrorKind, TypedError};
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{
    Interests, ProducerEffectHandlerExtension, ReceiverFactory, control::NodeControlMsg,
};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapPayload;
#[cfg(test)]
use otap_df_pdata::TryIntoWithOptions;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_debug, otel_info, otel_warn};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant as StdInstant;
use tokio::time::{Duration, Interval, MissedTickBehavior, interval};

use self::producer::{GenerateError, TrafficProducer};

pub mod attributes;
/// allows the user to configure their fake signal receiver
pub mod config;
/// provides the fake signal with fake data
pub mod fake_data;
/// fake signal metrics implementation
pub mod metrics;
/// Signal generation abstractions
pub mod producer;
/// generates signals based on OTel semantic conventions registry
pub mod semconv_signal;
/// Static hardcoded signal generators for lightweight load testing
pub mod static_signal;

/// The URN for the fake data generator receiver
pub const OTAP_FAKE_DATA_GENERATOR_URN: &str = "urn:otel:receiver:traffic_generator";

const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// A Receiver that generates fake OTAP data for testing purposes.
pub struct FakeGeneratorReceiver {
    /// Configuration for the fake data generator
    config: Config,

    /// Metrics for the fake data generator
    metrics: MetricSet<FakeSignalReceiverMetrics>,
}

fn smooth_batch_interval(run_len: usize) -> Option<Duration> {
    if run_len == 0 {
        return None;
    }

    let run_len = run_len as u128;
    let nanos = NANOS_PER_SECOND.div_ceil(run_len);
    u64::try_from(nanos).ok().map(Duration::from_nanos)
}

fn duration_nanos(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1e9
}

fn elapsed_nanos(start: StdInstant) -> f64 {
    duration_nanos(start.elapsed())
}

/// Declares the fake data generator as a local receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTAP_FAKE_DATA_GENERATOR: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTAP_FAKE_DATA_GENERATOR_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            FakeGeneratorReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

impl FakeGeneratorReceiver {
    /// creates a new FakeSignalReceiver
    #[must_use]
    pub fn new(pipeline_ctx: PipelineContext, config: Config) -> Self {
        let metrics = pipeline_ctx.register_metrics::<FakeSignalReceiverMetrics>();
        Self { config, metrics }
    }

    /// Creates a new fake data generator from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        config.get_traffic_config().validate()?;
        Ok(FakeGeneratorReceiver::new(pipeline_ctx, config))
    }

    async fn run_smooth(
        mut self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        handler: &local::EffectHandler<OtapPdata>,
        mut producer: TrafficProducer,
        mut run_ticker: Interval,
        mut batch_ticker: Interval,
        transport_headers: Option<TransportHeaders>,
    ) -> Result<TerminalState, Error> {
        let mut run_produced: u64 = 0;
        let mut next_pdata: Option<OtapPdata> = None;

        loop {
            producer.record_production(run_produced);
            run_produced = 0;

            let Ok(Some(mut current_run)) = producer.next_run() else {
                return wait_for_terminal(ctrl_msg_recv, handler, &mut self.metrics).await;
            };

            self.metrics.smooth_runs_started.inc();
            let mut run_completed = false;

            loop {
                tokio::select! {
                    biased;

                    msg = ctrl_msg_recv.recv() => {
                        if let Some(terminal) = handle_control_msg(msg, handler, &mut self.metrics).await? {
                            return Ok(terminal);
                        }
                    }

                    _ = run_ticker.tick() => {
                        let remaining_batches = current_run.len() + usize::from(next_pdata.is_some());
                        let remaining_items = current_run.remaining_signal_count()
                            + next_pdata.as_ref().map_or(0, |pdata| pdata.num_items() as u64);
                        if remaining_batches > 0 {
                            self.metrics.smooth_runs_behind.inc();
                            self.metrics
                                .smooth_behind_remaining_batches
                                .record(remaining_batches as f64);
                            self.metrics
                                .smooth_behind_remaining_items
                                .record(remaining_items as f64);
                            otel_warn!(
                                "Data generator is falling behind and didn't finish the current run. For highest
                                possible throughput, use production_mode: open",
                                remaining=remaining_batches,
                                remaining_items,
                            );
                        } else if !run_completed {
                            self.metrics.smooth_runs_completed.inc();
                            run_completed = true;
                        }

                        if next_pdata.is_some() {
                            continue;
                        }

                        break;
                    }

                    scheduled = batch_ticker.tick() => {
                        let tick_lateness = tokio::time::Instant::now()
                            .saturating_duration_since(scheduled);
                        self.metrics
                            .smooth_batch_tick_lateness_duration_ns
                            .record(duration_nanos(tick_lateness));

                        let channel_result = match next_pdata.take() {
                            Some(pdata) => {
                                self.metrics.smooth_payload_send_retry.inc();
                                let send_start = StdInstant::now();
                                let result = self.export_pdata(handler, pdata)?;
                                self.metrics
                                    .smooth_payload_send_duration_ns
                                    .record(elapsed_nanos(send_start));
                                result
                            }
                            None => {
                                let generate_start = StdInstant::now();
                                let payload = current_run.next();

                                let Some(payload) = payload else {
                                    if !run_completed {
                                        self.metrics.smooth_runs_completed.inc();
                                        run_completed = true;
                                    }
                                    continue;
                                };
                                self.metrics
                                    .smooth_payload_generate_duration_ns
                                    .record(elapsed_nanos(generate_start));

                                let send_start = StdInstant::now();
                                let result = self.handle_payload(handler, payload, &transport_headers)?;
                                self.metrics
                                    .smooth_payload_send_duration_ns
                                    .record(elapsed_nanos(send_start));
                                result
                            }
                        };

                        match channel_result {
                            Ok(count) => {
                                run_produced += count;
                            }
                            Err(pdata) => {
                                self.metrics.smooth_payload_send_full.inc();
                                next_pdata = Some(pdata);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn run_open(
        mut self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        handler: &local::EffectHandler<OtapPdata>,
        mut producer: TrafficProducer,
        mut run_ticker: Interval,
        transport_headers: Option<TransportHeaders>,
    ) -> Result<TerminalState, Error> {
        let mut run_produced: u64 = 0;
        'start: loop {
            producer.record_production(run_produced);
            run_produced = 0;

            let Ok(Some(mut current_run)) = producer.next_run() else {
                return wait_for_terminal(ctrl_msg_recv, handler, &mut self.metrics).await;
            };

            // First phase is the open export phase where we pump data as fast as
            // possible one chunk at a time while checking for control messages
            // in between.
            let mut next_pdata: Option<OtapPdata> = None;
            loop {
                // In the first select statement, we try to drain the entire run
                tokio::select! {
                    biased;

                    msg = ctrl_msg_recv.recv() => {
                        if let Some(terminal) = handle_control_msg(msg, handler, &mut self.metrics).await? {
                            return Ok(terminal);
                        }
                    }

                    _ = run_ticker.tick() => {
                        otel_debug!(
                            "Data generator is falling behind and didn't finish the current run.",
                            remaining=current_run.len(),
                        );

                        continue 'start;
                    }

                    _ = std::future::ready(()) => {
                        let channel_result = match next_pdata.take() {
                            Some(pdata) => {
                                self.export_pdata(handler, pdata)?
                            }
                            None => {
                                let Some(payload) = current_run.next() else {
                                    break;
                                };

                               self.handle_payload(handler, payload, &transport_headers)?
                            }
                        };

                        match channel_result {
                            Ok(count) => {
                                run_produced += count;
                            }
                            Err(pdata) => {
                                next_pdata = Some(pdata);
                                tokio::task::yield_now().await;
                            }
                        }
                    }
                }
            }

            loop {
                // The second phase starts once we exhaust a traffic run. At this point
                // we only process control messages while we wait for a tick at which point
                // we start the next run.
                tokio::select! {
                    biased;

                    msg = ctrl_msg_recv.recv() => {
                        if let Some(terminal) = handle_control_msg(msg, handler, &mut self.metrics).await? {
                            return Ok(terminal);
                        }
                    }

                    _ = run_ticker.tick() => {
                        continue 'start;
                    }
                }
            }
        }
    }

    // There are two failure modes here
    fn handle_payload(
        &mut self,
        handler: &local::EffectHandler<OtapPdata>,
        payload: Result<OtapPayload, GenerateError>,
        transport_headers: &Option<TransportHeaders>,
    ) -> Result<Result<u64, OtapPdata>, Error> {
        let payload = match payload {
            Ok(payload) => payload,
            Err(e) => {
                return Err(Error::ReceiverError {
                    receiver: handler.receiver_id(),
                    kind: ReceiverErrorKind::Other,
                    error: format!("Failed to generate data: {}", e),
                    source_detail: String::new(),
                });
            }
        };

        let mut pdata = OtapPdata::new_todo_context(payload);
        if let Some(headers) = transport_headers {
            pdata.set_transport_headers(headers.clone());
        }
        if self.config.enable_ack_nack() {
            handler.subscribe_to(
                Interests::ACKS | Interests::NACKS,
                CallData::default(),
                &mut pdata,
            );
        }

        self.export_pdata(handler, pdata)
    }

    fn export_pdata(
        &mut self,
        handler: &local::EffectHandler<OtapPdata>,
        pdata: OtapPdata,
    ) -> Result<Result<u64, OtapPdata>, Error> {
        let signal = pdata.signal_type();
        let count = pdata.num_items() as u64;
        match handler.try_send_message_with_source_node(pdata) {
            Ok(()) => {
                match signal {
                    otap_df_config::SignalType::Traces => self.metrics.spans_produced.add(count),
                    otap_df_config::SignalType::Metrics => self.metrics.metrics_produced.add(count),
                    otap_df_config::SignalType::Logs => self.metrics.logs_produced.add(count),
                };
                Ok(Ok(count))
            }
            Err(e) => {
                let TypedError::ChannelSendError(SendError::Full(pdata)) = e else {
                    return Err(Error::ReceiverError {
                        receiver: handler.receiver_id(),
                        kind: ReceiverErrorKind::Other,
                        error: format!("Failed to generate data: {}", e),
                        source_detail: String::new(),
                    });
                };

                Ok(Err(pdata))
            }
        }
    }
}

/// Builds transport headers from the user-configured map.
///
/// Keys with `Some(value)` produce fixed header values.
/// Keys with `None` produce a random value once at startup: a 16-char
/// random alphabetical string for text headers, or 16 raw random bytes
/// for binary headers (keys ending in `-bin`).
///
/// Returns `None` when the config map is empty (zero overhead).
fn build_transport_headers(
    config_headers: &HashMap<String, Option<String>>,
) -> Option<TransportHeaders> {
    if config_headers.is_empty() {
        return None;
    }
    let mut headers = TransportHeaders::with_capacity(config_headers.len());
    for (key, value) in config_headers {
        // Infer the value kind from the key name, matching the convention
        // used by the header capture policy: keys ending in `-bin` are
        // treated as binary (the gRPC binary metadata convention).
        if key.ends_with("-bin") {
            let resolved_value = match value {
                Some(v) => v.as_bytes().to_vec(),
                None => {
                    let mut buf = [0u8; 16];
                    rand::RngExt::fill(&mut rand::rng(), &mut buf);
                    buf.to_vec()
                }
            };
            headers.push(TransportHeader::binary(
                key.clone(),
                key.clone(),
                resolved_value,
            ));
        } else {
            let resolved_value = match value {
                Some(v) => v.as_bytes().to_vec(),
                None => {
                    // Generate bytes in ASCII printable range (space..tilde)
                    let mut rng = rand::rng();
                    (0..16)
                        .map(|_| rand::RngExt::random_range(&mut rng, 32u8..127))
                        .collect()
                }
            };
            headers.push(TransportHeader::text(
                key.clone(),
                key.clone(),
                resolved_value,
            ));
        }
    }
    Some(headers)
}

/// Waits for a terminal control message after the producer has finished.
///
/// This is used when `max_signal_count` has been reached and the receiver has
/// no more data to produce, but must remain alive for graceful shutdown.
async fn wait_for_terminal(
    mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
    handler: &local::EffectHandler<OtapPdata>,
    metrics: &mut MetricSet<FakeSignalReceiverMetrics>,
) -> Result<TerminalState, Error> {
    loop {
        let msg = ctrl_msg_recv.recv().await;
        if let Some(terminal) = handle_control_msg(msg, handler, metrics).await? {
            return Ok(terminal);
        }
    }
}

/// Handle a control message received on the control channel.
///
/// Returns `Ok(Some(terminal_state))` when the receiver should exit,
/// `Ok(None)` when it should continue the event loop, or `Err` on a
/// channel error.
async fn handle_control_msg(
    ctrl_msg: Result<NodeControlMsg<OtapPdata>, RecvError>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &mut MetricSet<FakeSignalReceiverMetrics>,
) -> Result<Option<TerminalState>, Error> {
    match ctrl_msg {
        Ok(NodeControlMsg::CollectTelemetry {
            mut metrics_reporter,
        }) => {
            _ = metrics_reporter.report(metrics);
            Ok(None)
        }
        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
            otel_info!("fake_data_generator.drain_ingress");
            effect_handler.notify_receiver_drained().await?;
            Ok(Some(TerminalState::new(deadline, [metrics.snapshot()])))
        }
        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
            otel_info!("fake_data_generator.shutdown");
            Ok(Some(TerminalState::new(deadline, [metrics.snapshot()])))
        }
        Err(e) => Err(Error::ChannelRecvError(e)),
        _ => Ok(None),
    }
}

/// Implement the Receiver trait for the FakeGeneratorReceiver
#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for FakeGeneratorReceiver {
    async fn start(
        mut self: Box<Self>,
        ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let producer =
            TrafficProducer::from_config(&self.config).map_err(|e| Error::ReceiverError {
                receiver: effect_handler.receiver_id(),
                kind: ReceiverErrorKind::Configuration,
                error: format!("Failed to generate producer: {}", e),
                source_detail: String::new(),
            })?;

        let transport_headers = build_transport_headers(self.config.transport_headers());

        let _ = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        let run_len = producer.run_len();

        // We consume one tick here because it's always immediately ready and would
        // make us think we're lagging;
        let mut run_ticker = interval(Duration::from_secs(1));
        run_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        _ = run_ticker.tick().await;

        match self.config.get_traffic_config().production_mode {
            config::ProductionMode::Smooth => {
                if let Some(batch_duration) = smooth_batch_interval(run_len) {
                    self.metrics.smooth_run_batches.set(run_len as u64);
                    self.metrics
                        .smooth_batch_interval_ns
                        .set(batch_duration.as_nanos() as u64);
                    let mut batch_ticker = interval(batch_duration);
                    batch_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
                    self.run_smooth(
                        ctrl_msg_recv,
                        &effect_handler,
                        producer,
                        run_ticker,
                        batch_ticker,
                        transport_headers,
                    )
                    .await
                } else {
                    otel_warn!(
                        "Falling back to Open production mode because smooth batch interval is zero"
                    );
                    self.run_open(
                        ctrl_msg_recv,
                        &effect_handler,
                        producer,
                        run_ticker,
                        transport_headers,
                    )
                    .await
                }
            }
            config::ProductionMode::Open => {
                self.run_open(
                    ctrl_msg_recv,
                    &effect_handler,
                    producer,
                    run_ticker,
                    transport_headers,
                )
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::config::{DataSource, GenerationStrategy};
    use super::*;

    use crate::receivers::fake_data_generator::config::{Config, TrafficConfig};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_config::transport_headers::ValueKind;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogsData;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::MetricsData;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data;
    use otap_df_pdata::proto::opentelemetry::trace::v1::TracesData;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message;
    use std::future::Future;
    use std::pin::Pin;
    use tokio::time::{Duration, sleep};

    use std::collections::HashSet;
    use weaver_common::vdir::VirtualDirectoryPath;
    use weaver_forge::registry::ResolvedRegistry;

    const RESOURCE_COUNT: usize = 1;
    const SCOPE_COUNT: usize = 1;
    const MESSAGE_COUNT: usize = 1;
    const RUN_TILL_SHUTDOWN: u64 = 999;
    const MESSAGE_PER_SECOND: usize = 3;
    const MAX_SIGNALS: u64 = 3;
    const MAX_BATCH: usize = 30;

    #[test]
    fn test_smooth_batch_interval_uses_sub_millisecond_precision() {
        let interval = smooth_batch_interval(2000).expect("interval should exist");

        assert_eq!(interval, Duration::from_micros(500));
    }

    #[test]
    fn test_smooth_batch_interval_does_not_overdrive_run() {
        let interval = smooth_batch_interval(88).expect("interval should exist");

        assert!(interval * 88 >= Duration::from_secs(1));
        assert!(interval * 87 < Duration::from_secs(1));
    }

    /// Convert OtapPdata signal to OtlpProtoMessage for testing purposes.
    fn pdata_to_otlp_message(value: OtapPdata) -> OtlpProtoMessage {
        let otlp_bytes: OtlpProtoBytes = value
            .payload()
            .try_into_with_default()
            .expect("can convert signal to otlp bytes");
        match otlp_bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                OtlpProtoMessage::Logs(LogsData::decode(bytes.as_ref()).expect("can decode bytes"))
            }
            OtlpProtoBytes::ExportMetricsRequest(bytes) => OtlpProtoMessage::Metrics(
                MetricsData::decode(bytes.as_ref()).expect("can decode bytes"),
            ),
            OtlpProtoBytes::ExportTracesRequest(bytes) => OtlpProtoMessage::Traces(
                TracesData::decode(bytes.as_ref()).expect("can decode bytes"),
            ),
        }
    }

    /// Test closure that simulates a typical receiver scenario.
    fn scenario() -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // no scenario to run here as scenario is already defined in the configuration
                // wait for the scenario to finish running
                sleep(Duration::from_millis(RUN_TILL_SHUTDOWN)).await;
                // send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(std::time::Instant::now(), "Test")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure(
        resolved_registry: ResolvedRegistry,
    ) -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // check that messages have been sent through the effect_handler
                while let Ok(received_signal) = ctx.recv().await {
                    match pdata_to_otlp_message(received_signal) {
                        OtlpProtoMessage::Metrics(metric) => {
                            // loop and check count
                            let resource_count = metric.resource_metrics.len();
                            assert!(resource_count == RESOURCE_COUNT);
                            for resource in metric.resource_metrics.iter() {
                                let scope_count = resource.scope_metrics.len();
                                assert!(scope_count == SCOPE_COUNT);
                                for scope in resource.scope_metrics.iter() {
                                    let metric_count = scope.metrics.len();
                                    assert!(metric_count == MESSAGE_COUNT);
                                    for metric in scope.metrics.iter() {
                                        let metric_definition = resolved_registry
                                            .groups
                                            .iter()
                                            .find(|group| {
                                                group.metric_name == Some(metric.name.clone())
                                            })
                                            .expect("metric not found in registry");
                                        assert_eq!(metric.description, metric_definition.brief);
                                        assert_eq!(
                                            Some(metric.unit.clone()),
                                            metric_definition.unit
                                        );

                                        let keys_metric_definition: HashSet<&str> =
                                            metric_definition
                                                .attributes
                                                .iter()
                                                .map(|attribute| attribute.name.as_str())
                                                .collect();

                                        match metric.data.as_ref().expect("metric has no data") {
                                            Data::Sum(sum) => {
                                                for datapoints in sum.data_points.iter() {
                                                    let keys: HashSet<&str> = datapoints
                                                        .attributes
                                                        .iter()
                                                        .map(|attribute| attribute.key.as_str())
                                                        .collect();

                                                    assert_eq!(keys, keys_metric_definition);
                                                }
                                            }
                                            Data::Gauge(gauge) => {
                                                for datapoints in gauge.data_points.iter() {
                                                    let keys: HashSet<&str> = datapoints
                                                        .attributes
                                                        .iter()
                                                        .map(|attribute| attribute.key.as_str())
                                                        .collect();

                                                    assert_eq!(keys, keys_metric_definition);
                                                }
                                            }
                                            Data::Histogram(histogram) => {
                                                for datapoints in histogram.data_points.iter() {
                                                    let keys: HashSet<&str> = datapoints
                                                        .attributes
                                                        .iter()
                                                        .map(|attribute| attribute.key.as_str())
                                                        .collect();

                                                    assert_eq!(keys, keys_metric_definition);
                                                }
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                }
                            }
                        }
                        OtlpProtoMessage::Traces(span) => {
                            let resource_count = span.resource_spans.len();
                            assert!(resource_count == RESOURCE_COUNT);
                            for resource in span.resource_spans.iter() {
                                let scope_count = resource.scope_spans.len();
                                assert!(scope_count == SCOPE_COUNT);
                                for scope in resource.scope_spans.iter() {
                                    let span_count = scope.spans.len();
                                    assert!(span_count == MESSAGE_COUNT);
                                    for span in scope.spans.iter() {
                                        let span_definition = resolved_registry
                                            .groups
                                            .iter()
                                            .find(|group| group.id == span.name)
                                            .expect("span not found in registry");
                                        let keys: HashSet<&str> = span
                                            .attributes
                                            .iter()
                                            .map(|attribute| attribute.key.as_str())
                                            .collect();
                                        let keys_span_definition: HashSet<&str> = span_definition
                                            .attributes
                                            .iter()
                                            .map(|attribute| attribute.name.as_str())
                                            .collect();

                                        assert_eq!(keys, keys_span_definition);

                                        let events: HashSet<&str> = span
                                            .events
                                            .iter()
                                            .map(|event| event.name.as_str())
                                            .collect();

                                        let events_span_definition: HashSet<&str> = span_definition
                                            .events
                                            .iter()
                                            .map(|event_name| event_name.as_str())
                                            .collect();
                                        assert_eq!(events, events_span_definition);
                                    }
                                }
                            }
                        }
                        OtlpProtoMessage::Logs(log) => {
                            let resource_count = log.resource_logs.len();
                            assert!(resource_count == RESOURCE_COUNT);
                            for resource in log.resource_logs.iter() {
                                let scope_count = resource.scope_logs.len();
                                assert!(scope_count == SCOPE_COUNT);
                                for scope in resource.scope_logs.iter() {
                                    let log_record_count = scope.log_records.len();
                                    assert!(log_record_count == MESSAGE_COUNT);
                                    for log_record in scope.log_records.iter() {
                                        let log_record_definition = resolved_registry
                                            .groups
                                            .iter()
                                            .find(|group| {
                                                group.name == Some(log_record.event_name.clone())
                                            })
                                            .expect("metric not found in registry");
                                        let keys: HashSet<&str> = log_record
                                            .attributes
                                            .iter()
                                            .map(|attribute| attribute.key.as_str())
                                            .collect();
                                        let keys_log_record_definition: HashSet<&str> =
                                            log_record_definition
                                                .attributes
                                                .iter()
                                                .map(|attribute| attribute.name.as_str())
                                                .collect();

                                        assert_eq!(keys, keys_log_record_definition);
                                    }
                                }
                            }
                        }
                    }
                }
            })
        }
    }

    #[test]
    fn test_fake_signal_receiver() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(Some(MESSAGE_PER_SECOND), None, MAX_BATCH, 1, 1, 1);
        let config = Config::new(traffic_config, registry_path);
        let registry = config
            .get_registry()
            .expect("failed to get registry")
            .expect("registry should be Some for SemanticConventions data source");

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure(registry));
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure_message_rate()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let mut received_messages = 0;

                while let Ok(received_signal) = ctx.recv().await {
                    match pdata_to_otlp_message(received_signal) {
                        OtlpProtoMessage::Metrics(metric) => {
                            // loop and check count
                            for resource in metric.resource_metrics.iter() {
                                for scope in resource.scope_metrics.iter() {
                                    received_messages += scope.metrics.len();
                                    assert!(scope.metrics.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OtlpProtoMessage::Traces(span) => {
                            for resource in span.resource_spans.iter() {
                                for scope in resource.scope_spans.iter() {
                                    received_messages += scope.spans.len();
                                    assert!(scope.spans.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OtlpProtoMessage::Logs(log) => {
                            for resource in log.resource_logs.iter() {
                                for scope in resource.scope_logs.iter() {
                                    received_messages += scope.log_records.len();
                                    assert!(scope.log_records.len() <= MAX_BATCH);
                                }
                            }
                        }
                    }
                }

                // Allow 1 to 2x (observed)
                assert!(received_messages >= MESSAGE_PER_SECOND);
                assert!(received_messages <= 2 * MESSAGE_PER_SECOND);
            })
        }
    }

    #[test]
    fn test_fake_signal_receiver_message_rate_only() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(Some(MESSAGE_PER_SECOND), None, MAX_BATCH, 1, 0, 0);
        let config = Config::new(traffic_config, registry_path);

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver"),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure_message_rate());
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure_max_signal()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let mut received_messages = 0;

                while let Ok(received_signal) = ctx.recv().await {
                    match pdata_to_otlp_message(received_signal) {
                        OtlpProtoMessage::Metrics(metric) => {
                            // loop and check count
                            for resource in metric.resource_metrics.iter() {
                                for scope in resource.scope_metrics.iter() {
                                    received_messages += scope.metrics.len();
                                    assert!(scope.metrics.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OtlpProtoMessage::Traces(span) => {
                            for resource in span.resource_spans.iter() {
                                for scope in resource.scope_spans.iter() {
                                    received_messages += scope.spans.len();
                                    assert!(scope.spans.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OtlpProtoMessage::Logs(log) => {
                            for resource in log.resource_logs.iter() {
                                for scope in resource.scope_logs.iter() {
                                    received_messages += scope.log_records.len();
                                    assert!(scope.log_records.len() <= MAX_BATCH);
                                }
                            }
                        }
                    }
                }

                assert!(received_messages as u64 == MAX_SIGNALS);
            })
        }
    }
    #[test]
    fn test_fake_signal_receiver_max_signal_count_only() {
        let test_runtime = TestRuntime::new();
        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(None, Some(MAX_SIGNALS), MAX_BATCH, 1, 0, 0);
        let config = Config::new(traffic_config, registry_path);

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver"),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure_max_signal());
    }

    /// Validation closure for PreGenerated strategy test
    fn validation_procedure_pregenerated()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let mut received_messages = 0;

                while let Ok(received_signal) = ctx.recv().await {
                    match pdata_to_otlp_message(received_signal) {
                        OtlpProtoMessage::Metrics(metric) => {
                            for resource in metric.resource_metrics.iter() {
                                for scope in resource.scope_metrics.iter() {
                                    received_messages += scope.metrics.len();
                                }
                            }
                        }
                        OtlpProtoMessage::Traces(span) => {
                            for resource in span.resource_spans.iter() {
                                for scope in resource.scope_spans.iter() {
                                    received_messages += scope.spans.len();
                                }
                            }
                        }
                        OtlpProtoMessage::Logs(log) => {
                            for resource in log.resource_logs.iter() {
                                for scope in resource.scope_logs.iter() {
                                    received_messages += scope.log_records.len();
                                }
                            }
                        }
                    }
                }

                // Should have received at least some messages
                assert!(
                    received_messages > 0,
                    "Should receive messages from pre-generated cache"
                );
            })
        }
    }

    #[test]
    fn test_fake_signal_receiver_static_pregenerated() {
        let test_runtime = TestRuntime::new();

        // Use Static data source with PreGenerated strategy
        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(Some(MESSAGE_PER_SECOND), None, MAX_BATCH, 1, 1, 1);
        let config = Config::new(traffic_config, registry_path)
            .with_data_source(DataSource::Static)
            .with_generation_strategy(GenerationStrategy::PreGenerated);

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_pregenerated"),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure_pregenerated());
    }

    /// Regression test: verifies that the receiver handles DrainIngress
    /// promptly instead of stalling until the drain deadline expires.
    /// Without proper DrainIngress handling the receiver would sleep
    /// through the entire rate-limit interval, causing DrainDeadlineReached.
    #[test]
    fn test_drain_ingress_exits_promptly() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        // signals_per_second=1 means the receiver sleeps ~1s between sends.
        // DrainIngress must interrupt that sleep and exit promptly.
        let traffic_config = TrafficConfig::new(Some(1), None, 1, 0, 0, 1);
        let config =
            Config::new(traffic_config, registry_path).with_data_source(DataSource::Static);

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_drain"),
            node_config,
            test_runtime.config(),
        );

        let drain_scenario =
            move |ctx: TestContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async move {
                    // Let the receiver start and enter its rate-limit sleep.
                    sleep(Duration::from_millis(200)).await;
                    let deadline = std::time::Instant::now() + Duration::from_secs(5);
                    ctx.send_control_msg(NodeControlMsg::DrainIngress {
                        deadline,
                        reason: "test drain".to_owned(),
                    })
                    .await
                    .expect("Failed to send DrainIngress");
                })
            };

        let drain_validation =
            |_ctx: NotSendValidateContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async {})
            };

        test_runtime
            .set_receiver(receiver)
            .run_test(drain_scenario)
            .run_validation(drain_validation);
    }

    /// Scenario: receiver-first shutdown reaches the pre-generated hot path
    /// while it is sending many batches in one iteration.
    /// Guarantees: the generated send loop yields often enough for the outer
    /// control select to observe `DrainIngress` promptly instead of timing out
    /// behind a long uncapped send burst.
    #[test]
    fn test_drain_ingress_exits_promptly_during_high_throughput_send_loop() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(Some(1000), None, 1, 0, 0, 1);
        let config = Config::new(traffic_config, registry_path)
            .with_data_source(DataSource::Static)
            .with_generation_strategy(GenerationStrategy::PreGenerated);

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_hot_drain"),
            node_config,
            test_runtime.config(),
        );

        let drain_scenario =
            move |ctx: TestContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async move {
                    sleep(Duration::from_millis(200)).await;
                    let deadline = std::time::Instant::now() + Duration::from_secs(5);
                    ctx.send_control_msg(NodeControlMsg::DrainIngress {
                        deadline,
                        reason: "test hot drain".to_owned(),
                    })
                    .await
                    .expect("Failed to send DrainIngress");
                })
            };

        let drain_validation =
            |_ctx: NotSendValidateContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async {})
            };

        test_runtime
            .set_receiver(receiver)
            .run_test(drain_scenario)
            .run_validation(drain_validation);
    }

    /// Regression test: verifies that a non-terminal control message
    /// (CollectTelemetry) arriving during the rate-limit sleep does NOT
    /// break the sleep early – the receiver should still respect the
    /// original wait_till deadline.
    #[test]
    fn test_non_terminal_ctrl_msg_does_not_break_rate_limit_sleep() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        // signals_per_second=1 with a single log per iteration means the
        // receiver will sleep ~1s between sends. If the non-terminal control
        // message breaks the sleep, we'd see more than 2 batches in 1.5s.
        let traffic_config = TrafficConfig::new(Some(1), None, 1, 0, 0, 1);
        let config =
            Config::new(traffic_config, registry_path).with_data_source(DataSource::Static);

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_ctrl_sleep"),
            node_config,
            test_runtime.config(),
        );

        let ctrl_scenario =
            move |ctx: TestContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async move {
                    // Let the receiver start and enter its rate-limit sleep.
                    sleep(Duration::from_millis(200)).await;
                    // Fire a CollectTelemetry message mid-sleep. This is a
                    // non-terminal control message and must NOT break the
                    // rate-limit sleep.
                    let (_rx, metrics_reporter) =
                        otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(1);
                    ctx.send_control_msg(NodeControlMsg::CollectTelemetry { metrics_reporter })
                        .await
                        .expect("Failed to send CollectTelemetry");

                    // Wait long enough for the first sleep to expire plus a
                    // small margin, but NOT long enough for a third iteration.
                    sleep(Duration::from_millis(1300)).await;

                    ctx.send_shutdown(std::time::Instant::now(), "Test")
                        .await
                        .expect("Failed to send Shutdown");
                })
            };

        let ctrl_validation =
            |mut ctx: NotSendValidateContext<OtapPdata>| -> Pin<Box<dyn Future<Output = ()>>> {
                Box::pin(async move {
                    let mut received_batches: u64 = 0;

                    while let Ok(_received_signal) = ctx.recv().await {
                        received_batches += 1;
                    }

                    // With 1 signal/sec and ~1.5s total runtime we expect at
                    // most 2 batches. If the non-terminal control message
                    // broke the sleep, we would see 3+.
                    assert!(
                        received_batches <= 2,
                        "Non-terminal control message broke the rate-limit sleep: \
                         expected at most 2 batches, got {received_batches}"
                    );
                })
            };

        test_runtime
            .set_receiver(receiver)
            .run_test(ctrl_scenario)
            .run_validation(ctrl_validation);
    }

    /// Verifies that pdata messages contain transport headers with fixed values
    /// when `transport_headers` is configured with explicit values.
    #[test]
    fn test_fake_data_transport_headers_fixed_value() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(
            Some(MESSAGE_PER_SECOND),
            Some(MAX_SIGNALS),
            MAX_BATCH,
            0,
            0,
            1,
        );
        let config = Config::new(traffic_config, registry_path)
            .with_data_source(DataSource::Static)
            .with_generation_strategy(GenerationStrategy::Fresh)
            .with_transport_headers(HashMap::from([(
                "x-tenant-id".to_string(),
                Some("acme".to_string()),
            )]));

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_transport_headers"),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                sleep(Duration::from_millis(RUN_TILL_SHUTDOWN)).await;
                ctx.send_shutdown(std::time::Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                let pdata = ctx.recv().await.expect("should receive at least one pdata");

                let headers = pdata
                    .transport_headers()
                    .expect("pdata should have transport headers");
                let tenant: Vec<_> = headers.find_by_name("x-tenant-id").collect();
                assert_eq!(
                    tenant.len(),
                    1,
                    "should have exactly one x-tenant-id header"
                );
                assert_eq!(
                    tenant[0].value_as_str(),
                    Some("acme"),
                    "fixed header value should be 'acme'"
                );
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation(validation);
    }

    /// Verifies that pdata messages contain transport headers with random values
    /// when `transport_headers` is configured with null values.
    #[test]
    fn test_fake_data_transport_headers_random_value() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(
            Some(MESSAGE_PER_SECOND),
            Some(MAX_SIGNALS),
            MAX_BATCH,
            0,
            0,
            1,
        );
        let config = Config::new(traffic_config, registry_path)
            .with_data_source(DataSource::Static)
            .with_generation_strategy(GenerationStrategy::Fresh)
            .with_transport_headers(HashMap::from([("x-request-id".to_string(), None)]));

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_random_headers"),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                sleep(Duration::from_millis(RUN_TILL_SHUTDOWN)).await;
                ctx.send_shutdown(std::time::Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                let pdata = ctx.recv().await.expect("should receive at least one pdata");

                let headers = pdata
                    .transport_headers()
                    .expect("pdata should have transport headers");
                let request_id: Vec<_> = headers.find_by_name("x-request-id").collect();
                assert_eq!(
                    request_id.len(),
                    1,
                    "should have exactly one x-request-id header"
                );
                assert_eq!(
                    request_id[0].value.len(),
                    16,
                    "random value should be 16 bytes"
                );
                assert_eq!(
                    request_id[0].value_kind,
                    ValueKind::Text,
                    "non-bin key should produce a Text header"
                );
                assert!(
                    request_id[0].value_as_str().is_some(),
                    "text header random value should be valid UTF-8 (printable)"
                );
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation(validation);
    }

    /// Verifies that pdata messages contain binary transport headers with random
    /// values when `transport_headers` is configured with null values and keys
    /// ending in `-bin`.
    #[test]
    fn test_fake_data_transport_headers_binary_random_value() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(
            Some(MESSAGE_PER_SECOND),
            Some(MAX_SIGNALS),
            MAX_BATCH,
            0,
            0,
            1,
        );
        let config = Config::new(traffic_config, registry_path)
            .with_data_source(DataSource::Static)
            .with_generation_strategy(GenerationStrategy::Fresh)
            .with_transport_headers(HashMap::from([("x-trace-bin".to_string(), None)]));

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_binary_headers"),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                sleep(Duration::from_millis(RUN_TILL_SHUTDOWN)).await;
                ctx.send_shutdown(std::time::Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                let pdata = ctx.recv().await.expect("should receive at least one pdata");

                let headers = pdata
                    .transport_headers()
                    .expect("pdata should have transport headers");
                let trace_bin: Vec<_> = headers.find_by_name("x-trace-bin").collect();
                assert_eq!(
                    trace_bin.len(),
                    1,
                    "should have exactly one x-trace-bin header"
                );
                assert_eq!(
                    trace_bin[0].value.len(),
                    16,
                    "random binary value should be 16 bytes"
                );
                assert_eq!(
                    trace_bin[0].value_kind,
                    ValueKind::Binary,
                    "-bin key should produce a Binary header"
                );
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation(validation);
    }

    /// Verifies that pdata messages do NOT have transport headers when
    /// `transport_headers` is absent from the config.
    #[test]
    fn test_fake_data_transport_headers_empty_by_default() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(
            Some(MESSAGE_PER_SECOND),
            Some(MAX_SIGNALS),
            MAX_BATCH,
            0,
            0,
            1,
        );
        let config = Config::new(traffic_config, registry_path)
            .with_data_source(DataSource::Static)
            .with_generation_strategy(GenerationStrategy::Fresh);

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(pipeline_ctx, config),
            test_node("fake_receiver_no_headers"),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                sleep(Duration::from_millis(RUN_TILL_SHUTDOWN)).await;
                ctx.send_shutdown(std::time::Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                let pdata = ctx.recv().await.expect("should receive at least one pdata");

                assert!(
                    pdata.transport_headers().is_none(),
                    "pdata should NOT have transport headers when config has no transport_headers"
                );
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation(validation);
    }
}
