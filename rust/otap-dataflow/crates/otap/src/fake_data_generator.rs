// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A fake data generator receiver.
//! Note: This receiver will be replaced in the future with a more sophisticated implementation.
//!

use crate::pdata::{OtapPdata, OtlpProtoBytes};
use crate::{OTAP_RECEIVER_FACTORIES, pdata};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::{ReceiverFactory, control::NodeControlMsg};
use otap_df_otlp::fake_signal_receiver::config::{Config, OTLPSignal};
use otap_df_otlp::fake_signal_receiver::fake_signal::{
    fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
};
use prost::{EncodeError, Message};
use serde_json::Value;
use std::sync::Arc;
use tokio::time::{Duration, Instant, sleep};
use weaver_forge::registry::ResolvedRegistry;

/// The URN for the fake data generator receiver
pub const OTAP_FAKE_DATA_GENERATOR_URN: &str = "urn:otel:otap:fake_data_generator";

/// A Receiver that generates fake OTAP data for testing purposes.
pub struct FakeGeneratorReceiver {
    /// Configuration for the fake data generator
    config: Config,
}

/// Declares the fake data generator as a local receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTAP_FAKE_DATA_GENERATOR: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTAP_FAKE_DATA_GENERATOR_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            FakeGeneratorReceiver::from_config(&node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
};

impl FakeGeneratorReceiver {
    /// creates a new FakeSignalReceiver
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Creates a new fake data generator from a configuration object
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(FakeGeneratorReceiver { config })
    }
}

/// Implement the Receiver trait for the FakeGeneratorReceiver
#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for FakeGeneratorReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        //start event loop
        let traffic_config = self.config.get_traffic_config();
        let registry = self
            .config
            .get_registry()
            .map_err(|err| Error::ReceiverError {
                receiver: effect_handler.receiver_id(),
                error: err,
            })?;

        let (metric_count, trace_count, log_count) = traffic_config.calculate_signal_count();
        let max_signal_count = traffic_config.get_max_signal_count();
        let signals_per_second = traffic_config.get_signal_rate();
        let max_batch_size = traffic_config.get_max_batch_size();
        let mut signal_count: u64 = 0;
        let one_second_duration = Duration::from_secs(1);
        loop {
            let wait_till = Instant::now() + one_second_duration;
            tokio::select! {
                biased; //prioritize ctrl_msg over all other blocks
                // Process internal event
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::Shutdown {..}) => {
                            // ToDo: add proper deadline function
                            break;
                        },
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // unknown control message do nothing
                        }
                    }
                }
                // generate and send signal based on provided configuration
                signal_status = generate_signal(effect_handler.clone(), max_signal_count, &mut signal_count, max_batch_size, metric_count, trace_count, log_count, &registry), if max_signal_count.is_none_or(|max| max > signal_count) => {
                    // if signals per second is set then we should rate limit
                    match signal_status {
                        Ok(_) => {
                            if signals_per_second.is_some() {
                                // check if need to sleep
                                let remaining_time = wait_till - Instant::now();
                                if remaining_time.as_secs_f64() > 0.0 {
                                    sleep(remaining_time).await;
                                }
                                // ToDo: Handle negative time, not able to keep up with specified rate limit
                            }
                        }
                        Err(e) => {
                            return Err(Error::ReceiverError {
                                receiver: effect_handler.receiver_id(),
                                error: e.to_string()
                            });
                        }
                    }
                }


            }
        }
        //Exit event loop
        Ok(())
    }
}

/// generate and send signals
async fn generate_signal(
    effect_handler: local::EffectHandler<OtapPdata>,
    max_signal_count: Option<u64>,
    signal_count: &mut u64,
    max_batch_size: usize,
    metric_count: usize,
    trace_count: usize,
    log_count: usize,
    registry: &ResolvedRegistry,
) -> Result<(), Error> {
    // nothing to send
    if max_batch_size == 0 {
        return Ok(());
    }

    let metric_count_split = metric_count / max_batch_size;
    let metric_count_remainder = metric_count % max_batch_size;
    let trace_count_split = trace_count / max_batch_size;
    let trace_count_remainder = trace_count % max_batch_size;
    let log_count_split = log_count / max_batch_size;
    let log_count_remainder = log_count % max_batch_size;

    if let Some(max_count) = max_signal_count {
        // don't generate signals if we reached max signal
        let mut current_count = *signal_count;
        if current_count >= max_count {
            return Ok(());
        }
        // update the counts here to allow us to reach the max_signal_count

        for _ in 0..metric_count_split {
            if max_count >= current_count + max_batch_size as u64 {
                effect_handler
                    .send_message(
                        OTLPSignal::Metrics(fake_otlp_metrics(max_batch_size, registry))
                            .try_into()?,
                    )
                    .await?;
                current_count += max_batch_size as u64;
            } else {
                // generate last remaining signals
                let remaining_count: usize =
                    (max_count - current_count)
                        .try_into()
                        .map_err(|_| Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            error: "failed to convert u64 to usize".to_string(),
                        })?;
                effect_handler
                    .send_message(
                        OTLPSignal::Metrics(fake_otlp_metrics(remaining_count, registry))
                            .try_into()?,
                    )
                    .await?;

                // no more signals we have reached the max
                *signal_count = max_count;
                return Ok(());
            }
        }
        if metric_count_remainder > 0 && max_count >= current_count + metric_count_remainder as u64
        {
            effect_handler
                .send_message(
                    OTLPSignal::Metrics(fake_otlp_metrics(metric_count_remainder, registry))
                        .try_into()?,
                )
                .await?;
            current_count += metric_count_remainder as u64;
        }

        // generate and send traces
        for _ in 0..trace_count_split {
            if max_count >= current_count + max_batch_size as u64 {
                effect_handler
                    .send_message(
                        OTLPSignal::Traces(fake_otlp_traces(max_batch_size, registry))
                            .try_into()?,
                    )
                    .await?;
                current_count += max_batch_size as u64;
            } else {
                let remaining_count: usize =
                    (max_count - current_count)
                        .try_into()
                        .map_err(|_| Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            error: "failed to convert u64 to usize".to_string(),
                        })?;
                effect_handler
                    .send_message(
                        OTLPSignal::Traces(fake_otlp_traces(remaining_count, registry))
                            .try_into()?,
                    )
                    .await?;
                // no more signals we have reached the max
                *signal_count = max_count;
                return Ok(());
            }
        }
        if trace_count_remainder > 0 && max_count >= current_count + trace_count_remainder as u64 {
            effect_handler
                .send_message(
                    OTLPSignal::Traces(fake_otlp_traces(trace_count_remainder, registry))
                        .try_into()?,
                )
                .await?;
            current_count += trace_count_remainder as u64;
        }

        // generate and send logs
        for _ in 0..log_count_split {
            if max_count >= current_count + max_batch_size as u64 {
                effect_handler
                    .send_message(
                        OTLPSignal::Logs(fake_otlp_logs(max_batch_size, registry)).try_into()?,
                    )
                    .await?;
                current_count += max_batch_size as u64;
            } else {
                let remaining_count: usize =
                    (max_count - current_count)
                        .try_into()
                        .map_err(|_| Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            error: "failed to convert u64 to usize".to_string(),
                        })?;
                effect_handler
                    .send_message(
                        OTLPSignal::Logs(fake_otlp_logs(remaining_count, registry)).try_into()?,
                    )
                    .await?;
                // no more signals we have reached the max
                *signal_count = max_count;
                return Ok(());
            }
        }
        if log_count_remainder > 0 && max_count >= current_count + log_count_remainder as u64 {
            effect_handler
                .send_message(
                    OTLPSignal::Logs(fake_otlp_logs(log_count_remainder, registry)).try_into()?,
                )
                .await?;
            current_count += log_count_remainder as u64;
        }

        *signal_count = current_count;
    } else {
        // generate and send metric
        for _ in 0..metric_count_split {
            effect_handler
                .send_message(
                    OTLPSignal::Metrics(fake_otlp_metrics(max_batch_size, registry)).try_into()?,
                )
                .await?;
        }
        if metric_count_remainder > 0 {
            effect_handler
                .send_message(
                    OTLPSignal::Metrics(fake_otlp_metrics(metric_count_remainder, registry))
                        .try_into()?,
                )
                .await?;
        }

        // generate and send traces
        for _ in 0..trace_count_split {
            effect_handler
                .send_message(
                    OTLPSignal::Traces(fake_otlp_traces(max_batch_size, registry)).try_into()?,
                )
                .await?;
        }
        if trace_count_remainder > 0 {
            effect_handler
                .send_message(
                    OTLPSignal::Traces(fake_otlp_traces(trace_count_remainder, registry))
                        .try_into()?,
                )
                .await?;
        }

        // generate and send logs
        for _ in 0..log_count_split {
            effect_handler
                .send_message(
                    OTLPSignal::Logs(fake_otlp_logs(max_batch_size, registry)).try_into()?,
                )
                .await?;
        }
        if log_count_remainder > 0 {
            effect_handler
                .send_message(
                    OTLPSignal::Logs(fake_otlp_logs(log_count_remainder, registry)).try_into()?,
                )
                .await?;
        }
    }

    Ok(())
}
impl TryFrom<OTLPSignal> for OtapPdata {
    type Error = Error;

    fn try_from(value: OTLPSignal) -> Result<Self, Self::Error> {
        let map_error = |e: EncodeError| {
            Error::from(pdata::error::Error::ConversionError {
                error: format!("error encoding protobuf: {e}"),
            })
        };
        let mut bytes = vec![];
        Ok(match value {
            OTLPSignal::Logs(logs_data) => {
                logs_data.encode(&mut bytes).map_err(map_error)?;
                OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into())
            }
            OTLPSignal::Metrics(metrics_data) => {
                metrics_data.encode(&mut bytes).map_err(map_error)?;
                OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(bytes).into())
            }
            OTLPSignal::Traces(trace_data) => {
                trace_data.encode(&mut bytes).map_err(map_error)?;
                OtapPdata::new_default(OtlpProtoBytes::ExportTracesRequest(bytes).into())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otap_df_otlp::fake_signal_receiver::config::{Config, OTLPSignal, TrafficConfig};
    use otel_arrow_rust::proto::opentelemetry::logs::v1::LogsData;
    use otel_arrow_rust::proto::opentelemetry::metrics::v1::MetricsData;
    use otel_arrow_rust::proto::opentelemetry::metrics::v1::metric::Data;
    use otel_arrow_rust::proto::opentelemetry::trace::v1::TracesData;
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

    impl From<OtapPdata> for OTLPSignal {
        fn from(value: OtapPdata) -> Self {
            let otlp_bytes: OtlpProtoBytes = value
                .try_into()
                .map(|(_, v)| v)
                .expect("can convert signal to otlp bytes");
            match otlp_bytes {
                OtlpProtoBytes::ExportLogsRequest(bytes) => {
                    Self::Logs(LogsData::decode(bytes.as_ref()).expect("can decode bytes"))
                }
                OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                    Self::Metrics(MetricsData::decode(bytes.as_ref()).expect("can decode bytes"))
                }
                OtlpProtoBytes::ExportTracesRequest(bytes) => {
                    Self::Traces(TracesData::decode(bytes.as_ref()).expect("can decode bytes"))
                }
            }
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
                ctx.send_shutdown(Duration::from_millis(0), "Test")
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
                    match received_signal.into() {
                        OTLPSignal::Metrics(metric) => {
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
                        OTLPSignal::Traces(span) => {
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
                        OTLPSignal::Logs(log) => {
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
        let registry = config.get_registry().expect("failed to get registry");

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(config),
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
                    match received_signal.into() {
                        OTLPSignal::Metrics(metric) => {
                            // loop and check count
                            for resource in metric.resource_metrics.iter() {
                                for scope in resource.scope_metrics.iter() {
                                    received_messages += scope.metrics.len();
                                    assert!(scope.metrics.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OTLPSignal::Traces(span) => {
                            for resource in span.resource_spans.iter() {
                                for scope in resource.scope_spans.iter() {
                                    received_messages += scope.spans.len();
                                    assert!(scope.spans.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OTLPSignal::Logs(log) => {
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
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(config),
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
                    match received_signal.into() {
                        OTLPSignal::Metrics(metric) => {
                            // loop and check count
                            for resource in metric.resource_metrics.iter() {
                                for scope in resource.scope_metrics.iter() {
                                    received_messages += scope.metrics.len();
                                    assert!(scope.metrics.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OTLPSignal::Traces(span) => {
                            for resource in span.resource_spans.iter() {
                                for scope in resource.scope_spans.iter() {
                                    received_messages += scope.spans.len();
                                    assert!(scope.spans.len() <= MAX_BATCH);
                                }
                            }
                        }
                        OTLPSignal::Logs(log) => {
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
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(config),
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
}
