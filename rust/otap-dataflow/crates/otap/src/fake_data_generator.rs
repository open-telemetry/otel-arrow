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
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
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
    create: |node_config: Arc<NodeUserConfig>, receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            FakeGeneratorReceiver::from_config(&node_config.config)?,
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
        mut ctrl_msg_recv: local::ControlChannel,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error<OtapPdata>> {
        //start event loop
        let registry = self
            .config
            .get_registry()
            .map_err(|err| Error::ReceiverError {
                receiver: effect_handler.receiver_id(),
                error: err,
            })?;

        let (metric_count, trace_count, log_count) =
            self.config.get_traffic_config().calculate_signal_count();

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
                // run scenario based on provided configuration
                _ = generate_signal(effect_handler.clone(), metric_count, trace_count, log_count, &registry) => {
                    // calculate how much time we need to sleep
                    let remaining_time = wait_till - Instant::now();
                    if remaining_time.as_secs_f64() > 0.0 {
                        sleep(remaining_time).await;
                    }
                    // ToDo: handle negative time_till_next where we can't keep up with the specified message_rate
                }

            }
        }
        //Exit event loop
        Ok(())
    }
}

/// Run the configured scenario steps
async fn generate_signal(
    effect_handler: local::EffectHandler<OtapPdata>,
    metric_count: usize,
    trace_count: usize,
    log_count: usize,
    registry: &ResolvedRegistry,
) -> Result<(), Error<OtapPdata>> {
    // generate and send metric
    if metric_count > 0 {
        let signal = OTLPSignal::Metrics(fake_otlp_metrics(metric_count, registry));
        effect_handler.send_message(signal.try_into()?).await?;
    }

    // generate and send traces
    if trace_count > 0 {
        let signal = OTLPSignal::Traces(fake_otlp_traces(trace_count, registry));
        effect_handler.send_message(signal.try_into()?).await?;
    }

    // generate and send logs
    if log_count > 0 {
        let signal = OTLPSignal::Logs(fake_otlp_logs(log_count, registry));
        effect_handler.send_message(signal.try_into()?).await?;
    }
    Ok(())
}

impl TryFrom<OTLPSignal> for OtapPdata {
    type Error = Error<OtapPdata>;

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
                OtlpProtoBytes::ExportLogsRequest(bytes).into()
            }
            OTLPSignal::Metrics(metrics_data) => {
                metrics_data.encode(&mut bytes).map_err(map_error)?;
                OtlpProtoBytes::ExportMetricsRequest(bytes).into()
            }
            OTLPSignal::Traces(trace_data) => {
                trace_data.encode(&mut bytes).map_err(map_error)?;
                OtlpProtoBytes::ExportTracesRequest(bytes).into()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
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

    impl From<OtapPdata> for OTLPSignal {
        fn from(value: OtapPdata) -> Self {
            let otlp_bytes: OtlpProtoBytes =
                value.try_into().expect("can convert signal to otlp bytes");
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
    fn scenario() -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
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

        let traffic_config = TrafficConfig::new(MESSAGE_PER_SECOND, 1, 1, 1);
        let config = Config::new(traffic_config, registry_path);
        let registry = config.get_registry().expect("failed to get registry");

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(config),
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
                                }
                            }
                        }
                        OTLPSignal::Traces(span) => {
                            for resource in span.resource_spans.iter() {
                                for scope in resource.scope_spans.iter() {
                                    received_messages += scope.spans.len();
                                }
                            }
                        }
                        OTLPSignal::Logs(log) => {
                            for resource in log.resource_logs.iter() {
                                for scope in resource.scope_logs.iter() {
                                    received_messages += scope.log_records.len();
                                }
                            }
                        }
                    }
                }

                assert!(received_messages == MESSAGE_PER_SECOND);
            })
        }
    }

    #[test]
    fn test_fake_signal_receiver_message_rate() {
        let test_runtime = TestRuntime::new();

        let registry_path = VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        };

        let traffic_config = TrafficConfig::new(MESSAGE_PER_SECOND, 1, 0, 0);
        let config = Config::new(traffic_config, registry_path);

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeGeneratorReceiver::new(config),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure_message_rate());
    }
}
