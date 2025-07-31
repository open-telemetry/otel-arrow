// SPDX-License-Identifier: Apache-2.0

//! A fake data generator receiver.
//! Note: This receiver will be replaced in the future with a more sophisticated implementation.

use crate::grpc::OtapArrowBytes;
use crate::pdata::{OtapPdata, OtlpProtoBytes};
use crate::{OTAP_RECEIVER_FACTORIES, pdata};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::{ReceiverFactory, control::ControlMsg};
use otap_df_otlp::fake_signal_receiver::config::{Config, OTLPSignal};
use otap_df_otlp::fake_signal_receiver::receiver::generate_signals;
use otap_df_otlp::grpc::OTLPData;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::BatchArrowRecords;
use prost::{EncodeError, Message};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::rc::Rc;

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
    create: |node_config: Rc<NodeUserConfig>, receiver_config: &ReceiverConfig| {
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
        loop {
            tokio::select! {
                biased; //prioritize ctrl_msg over all other blocks
                // Process internal event
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(ControlMsg::Shutdown {..}) => {
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
                _ = run_scenario(&self.config, effect_handler.clone()) => {
                    // do nothing
                }

            }
        }
        //Exit event loop
        Ok(())
    }
}

async fn run_scenario(
    config: &Config,
    effect_handler: local::EffectHandler<OtapPdata>,
) -> Result<(), Error<OtapPdata>> {
    for signal in generate_signals(config) {
        _ = effect_handler.send_message(signal.try_into()?).await;
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
    use otap_df_otlp::fake_signal_receiver::config::{
        Config, Load, OTLPSignal, ScenarioStep, SignalType,
    };
    use otel_arrow_rust::proto::opentelemetry::{
        logs::v1::LogsData,
        metrics::v1::{Metric, MetricsData, metric::Data},
        trace::v1::TracesData,
    };
    use std::fs;
    use std::future::Future;
    use std::pin::Pin;
    use std::rc::Rc;
    use tokio::time::{Duration, sleep, timeout};
    use weaver_forge::registry::ResolvedRegistry;

    const RESOURCE_COUNT: usize = 1;
    const SCOPE_COUNT: usize = 1;
    const BATCH_COUNT: u64 = 1;
    const DELAY: u64 = 0;
    const RESOLVED_REGISTRY_FILE: &str = "../otlp/src/fake_signal_receiver/resolved_registry.yaml";

    // metric signal based on registry we should check matches
    const METRIC_NAME: &str = "system.network.dropped";
    const METRIC_DESC: &str =
        "Count of packets that are dropped or discarded even though there was no error.";
    const METRIC_DATAPOINT_ATTR: [&str; 2] = ["network.io.direction", "network.interface.name"];
    const METRIC_UNIT: &str = "{packet}";

    // span signal based on registry we should check matches
    const SPAN_NAME: &str = "span.rpc.client";
    const SPAN_ATTR: [&str; 9] = [
        "rpc.method",
        "rpc.service",
        "network.peer.address",
        "network.transport",
        "network.type",
        "rpc.system",
        "network.peer.port",
        "server.address",
        "server.port",
    ];
    const SPAN_EVENTS: [&str; 1] = ["rpc.message"];

    // log signal based on registry we should check matches
    const LOG_NAME: &str = "session.end";
    const LOG_ATTR: [&str; 1] = ["session.id"];

    impl From<OtapPdata> for OTLPSignal {
        fn from(value: OtapPdata) -> Self {
            // TODO no unwrap?
            let otlp_bytes: OtlpProtoBytes =
                value.try_into().expect("can convert signal to otlp bytes");
            match otlp_bytes {
                OtlpProtoBytes::ExportLogsRequest(bytes) => {
                    OTLPSignal::Logs(LogsData::decode(bytes.as_ref()).expect("can decode bytes"))
                }
                OtlpProtoBytes::ExportMetricsRequest(bytes) => OTLPSignal::Metrics(
                    MetricsData::decode(bytes.as_ref()).expect("can decode bytes"),
                ),
                OtlpProtoBytes::ExportTracesRequest(bytes) => OTLPSignal::Traces(
                    TracesData::decode(bytes.as_ref()).expect("can decode bytes"),
                ),
            }
        }
    }

    /// Test closure that simulates a typical receiver scenario.
    fn scenario() -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // no scenario to run here as scenario is already defined in the configuration
                // wait for the scenario to finish running
                sleep(Duration::from_millis(1000)).await;
                // send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Duration::from_millis(0), "Test")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // check that messages have been sent through the effect_handler

                // read from the effect handler
                let metric_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received")
                    .into();
                let trace_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received")
                    .into();
                let log_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received")
                    .into();

                // Assert that the message received is what the test client sent.
                match metric_received {
                    OTLPSignal::Metrics(metric) => {
                        // loop and check count
                        let mut metric_seen = false;
                        let resource_count = metric.resource_metrics.len();
                        assert!(resource_count == RESOURCE_COUNT);
                        for resource in metric.resource_metrics.iter() {
                            let scope_count = resource.scope_metrics.len();
                            assert!(scope_count == SCOPE_COUNT);
                            for scope in resource.scope_metrics.iter() {
                                for metric in scope.metrics.iter() {
                                    // check for metric and see if the signal fields match what is defined in the registry
                                    if &metric.name == METRIC_NAME {
                                        metric_seen = true;
                                        assert!(metric.description == METRIC_DESC.to_string());
                                        assert!(metric.unit == METRIC_UNIT.to_string());
                                        match metric.data.as_ref().expect("metric has no data") {
                                            Data::Sum(sum) => {
                                                assert!(sum.is_monotonic);
                                                for datapoints in sum.data_points.iter() {
                                                    let keys: Vec<&str> = datapoints
                                                        .attributes
                                                        .iter()
                                                        .map(|attribute| attribute.key.as_str())
                                                        .collect();
                                                    assert!(keys == METRIC_DATAPOINT_ATTR.to_vec());
                                                }
                                            }
                                            _ => assert!(false),
                                        }
                                    }
                                }
                            }
                        }
                        assert!(metric_seen);
                    }
                    _ => unreachable!("Signal should have been a Metric type"),
                }

                match trace_received {
                    OTLPSignal::Traces(span) => {
                        let mut span_seen = false;
                        let resource_count = span.resource_spans.len();
                        assert!(resource_count == RESOURCE_COUNT);
                        for resource in span.resource_spans.iter() {
                            let scope_count = resource.scope_spans.len();
                            assert!(scope_count == SCOPE_COUNT);
                            for scope in resource.scope_spans.iter() {
                                for span in scope.spans.iter() {
                                    // check for span and see if the signal fields match what is defined in the registry
                                    if &span.name == SPAN_NAME {
                                        span_seen = true;
                                        let keys: Vec<&str> = span
                                            .attributes
                                            .iter()
                                            .map(|attribute| attribute.key.as_str())
                                            .collect();
                                        assert!(keys == SPAN_ATTR);
                                        let events: Vec<&str> = span
                                            .events
                                            .iter()
                                            .map(|event| event.name.as_str())
                                            .collect();
                                        assert!(events == SPAN_EVENTS.to_vec())
                                    }
                                }
                            }
                        }
                        assert!(span_seen);
                    }
                    _ => unreachable!("Signal should have been a Span type"),
                }

                match log_received {
                    OTLPSignal::Logs(log) => {
                        let mut log_seen = false;
                        let resource_count = log.resource_logs.len();
                        assert!(resource_count == RESOURCE_COUNT);
                        for resource in log.resource_logs.iter() {
                            let scope_count = resource.scope_logs.len();
                            assert!(scope_count == SCOPE_COUNT);
                            for scope in resource.scope_logs.iter() {
                                for log_record in scope.log_records.iter() {
                                    // check for log and see if the signal fields match what is defined in the registry
                                    if &log_record.event_name == LOG_NAME {
                                        log_seen = true;
                                        let keys: Vec<&str> = log_record
                                            .attributes
                                            .iter()
                                            .map(|attribute| attribute.key.as_str())
                                            .collect();
                                        assert!(keys == LOG_ATTR.to_vec());
                                    }
                                }
                            }
                        }
                        assert!(log_seen);
                    }
                    _ => unreachable!("Signal should have been a Log type"),
                }
            })
        }
    }

    #[test]
    fn test_fake_signal_generator() {
        let test_runtime = TestRuntime::new();

        let registry_yaml =
            fs::read_to_string(RESOLVED_REGISTRY_FILE).expect("unable to read registry file");
        let registry: ResolvedRegistry = serde_yaml::from_str(registry_yaml.as_ref()).unwrap();

        let mut steps = vec![];

        let load = Load::new(RESOURCE_COUNT, SCOPE_COUNT);

        steps.push(ScenarioStep::new(
            SignalType::Metrics(load.clone()),
            BATCH_COUNT,
            DELAY,
        ));

        steps.push(ScenarioStep::new(
            SignalType::Traces(load.clone()),
            BATCH_COUNT,
            DELAY,
        ));
        steps.push(ScenarioStep::new(
            SignalType::Logs(load.clone()),
            BATCH_COUNT,
            DELAY,
        ));
        let config = Config::new(steps, registry);

        let node_config = Rc::new(NodeUserConfig::new_receiver_config(
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
            .run_validation(validation_procedure());
    }
}
