// SPDX-License-Identifier: Apache-2.0

//! A fake data generator receiver.
//! Note: This receiver will be replaced in the future with a more sophisticated implementation.
//!

use std::rc::Rc;
use std::time::Duration;

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
use otap_df_otlp::fake_signal_receiver::config::{Config, ScenarioStep, SignalConfig};
use otap_df_otlp::grpc::OTLPData;
use prost::{EncodeError, Message};
use serde_json::Value;
use tokio::time::sleep;

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
                _ = run_scenario(&self.config.get_steps(), effect_handler.clone()) => {
                    // do nothing
                }

            }
        }
        //Exit event loop
        Ok(())
    }
}

/// Run the configured scenario steps
async fn run_scenario(
    steps: &Vec<ScenarioStep>,
    effect_handler: local::EffectHandler<OtapPdata>,
) -> Result<(), Error<OtapPdata>> {
    // loop through each step

    for step in steps {
        // create batches if specified
        let batches = step.get_batches_to_generate() as usize;
        for _ in 0..batches {
            let signal = match step.get_config() {
                SignalConfig::Metric(config) => OTLPData::Metrics(config.get_signal()),
                SignalConfig::Log(config) => OTLPData::Logs(config.get_signal()),
                SignalConfig::Span(config) => OTLPData::Traces(config.get_signal()),
            };
            _ = effect_handler.send_message(signal.try_into()?).await;
            // if there is a delay set between batches sleep for that amount before created the next signal in the batch
            sleep(Duration::from_millis(step.get_delay_between_batches_ms())).await;
        }
    }

    Ok(())
}

impl TryFrom<OTLPData> for OtapPdata {
    type Error = Error<OtapPdata>;

    fn try_from(value: OTLPData) -> Result<Self, Self::Error> {
        let map_error = |e: EncodeError| {
            Error::from(pdata::error::Error::ConversionError {
                error: format!("error encoding protobuf: {e}"),
            })
        };
        let mut bytes = vec![];
        Ok(match value {
            OTLPData::Logs(logs_data) => {
                logs_data.encode(&mut bytes).map_err(map_error)?;
                OtlpProtoBytes::ExportLogsRequest(bytes).into()
            }
            OTLPData::Metrics(metrics_data) => {
                metrics_data.encode(&mut bytes).map_err(map_error)?;
                OtlpProtoBytes::ExportMetricsRequest(bytes).into()
            }
            OTLPData::Traces(trace_data) => {
                trace_data.encode(&mut bytes).map_err(map_error)?;
                OtlpProtoBytes::ExportTracesRequest(bytes).into()
            }
            _ => {
                return Err(Error::from(pdata::error::Error::ConversionError {
                    error: "unsupported signal type".into(),
                }));
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
    use otap_df_otlp::ExportTraceServiceRequest;
    use otap_df_otlp::fake_signal_receiver::config::{
        Config, LogConfig, MetricConfig, MetricType, ScenarioStep, SignalConfig, SpanConfig,
    };
    use otap_df_otlp::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_otlp::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_otlp::proto::opentelemetry::metrics::v1::metric::Data;
    use std::future::Future;
    use std::pin::Pin;
    use std::rc::Rc;
    use tokio::time::{Duration, sleep, timeout};

    const RESOURCE_COUNT: usize = 1;
    const SPAN_COUNT: usize = 1;
    const METRIC_COUNT: usize = 1;
    const LOG_COUNT: usize = 1;
    const DATAPOINT_COUNT: usize = 1;
    const ATTRIBUTE_COUNT: usize = 1;
    const EVENT_COUNT: usize = 1;
    const LINK_COUNT: usize = 1;
    const SCOPE_COUNT: usize = 1;
    const BATCH_COUNT: u64 = 1;
    const DELAY: u64 = 0;

    impl From<OtapPdata> for OTLPData {
        fn from(value: OtapPdata) -> Self {
            let otlp_bytes: OtlpProtoBytes =
                value.try_into().expect("can convert signal to otlp bytes");
            match otlp_bytes {
                OtlpProtoBytes::ExportLogsRequest(bytes) => Self::Logs(
                    ExportLogsServiceRequest::decode(bytes.as_ref()).expect("can decode bytes"),
                ),
                OtlpProtoBytes::ExportMetricsRequest(bytes) => Self::Metrics(
                    ExportMetricsServiceRequest::decode(bytes.as_ref()).expect("can decode bytes"),
                ),
                OtlpProtoBytes::ExportTracesRequest(bytes) => Self::Traces(
                    ExportTraceServiceRequest::decode(bytes.as_ref()).expect("can decode bytes"),
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
                let metric_received: OTLPData = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received")
                    .into();
                let trace_received: OTLPData = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received")
                    .into();
                let log_received: OTLPData = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received")
                    .into();

                // Assert that the message received is what the test client sent.
                match metric_received {
                    OTLPData::Metrics(metric) => {
                        // loop and check count
                        let resource_count = metric.resource_metrics.len();
                        assert!(resource_count == RESOURCE_COUNT);
                        for resource in metric.resource_metrics.iter() {
                            let scope_count = resource.scope_metrics.len();
                            assert!(scope_count == SCOPE_COUNT);
                            for scope in resource.scope_metrics.iter() {
                                let metric_count = scope.metrics.len();
                                assert!(metric_count == METRIC_COUNT);
                                for metric_data in scope.metrics.iter() {
                                    if let Some(data) = &metric_data.data {
                                        if let Data::Gauge(gauge) = data {
                                            let datapoint_count = gauge.data_points.len();
                                            assert!(datapoint_count == DATAPOINT_COUNT);
                                            for datapoint in gauge.data_points.iter() {
                                                let attribute_count = datapoint.attributes.len();
                                                assert!(attribute_count == ATTRIBUTE_COUNT);
                                            }
                                        } else {
                                            unreachable!("Wrong MetricType received");
                                        }
                                    } else {
                                        unreachable!("Option should not be None");
                                    }
                                }
                            }
                        }
                    }
                    _ => unreachable!("Signal should have been a Metric type"),
                }

                match trace_received {
                    OTLPData::Traces(span) => {
                        let resource_count = span.resource_spans.len();
                        assert!(resource_count == RESOURCE_COUNT);
                        for resource in span.resource_spans.iter() {
                            let scope_count = resource.scope_spans.len();
                            assert!(scope_count == SCOPE_COUNT);
                            for scope in resource.scope_spans.iter() {
                                let span_count = scope.spans.len();
                                assert!(span_count == SPAN_COUNT);
                                for span_data in scope.spans.iter() {
                                    let event_count = span_data.events.len();
                                    let link_count = span_data.links.len();
                                    let attribute_count = span_data.attributes.len();
                                    assert!(link_count == LINK_COUNT);
                                    assert!(event_count == EVENT_COUNT);
                                    assert!(attribute_count == ATTRIBUTE_COUNT);
                                }
                            }
                        }
                    }
                    _ => unreachable!("Signal should have been a Span type"),
                }

                match log_received {
                    OTLPData::Logs(log) => {
                        let resource_count = log.resource_logs.len();
                        assert!(resource_count == RESOURCE_COUNT);
                        for resource in log.resource_logs.iter() {
                            let scope_count = resource.scope_logs.len();
                            assert!(scope_count == SCOPE_COUNT);
                            for scope in resource.scope_logs.iter() {
                                let log_count = scope.log_records.len();
                                assert!(log_count == LOG_COUNT);
                            }
                        }
                    }
                    _ => unreachable!("Signal should have been a Log type"),
                }
            })
        }
    }

    #[test]
    fn test_fake_signal_receiver() {
        let test_runtime = TestRuntime::new();

        let mut steps = vec![];
        let metric_config = MetricConfig::new(
            RESOURCE_COUNT,
            SCOPE_COUNT,
            METRIC_COUNT,
            DATAPOINT_COUNT,
            MetricType::Gauge,
            ATTRIBUTE_COUNT,
        );
        let trace_config = SpanConfig::new(
            RESOURCE_COUNT,
            SCOPE_COUNT,
            SPAN_COUNT,
            EVENT_COUNT,
            LINK_COUNT,
            ATTRIBUTE_COUNT,
        );

        let log_config = LogConfig::new(RESOURCE_COUNT, SCOPE_COUNT, LOG_COUNT, ATTRIBUTE_COUNT);

        steps.push(ScenarioStep::new(
            SignalConfig::Metric(metric_config),
            BATCH_COUNT,
            DELAY,
        ));

        steps.push(ScenarioStep::new(
            SignalConfig::Span(trace_config),
            BATCH_COUNT,
            DELAY,
        ));
        steps.push(ScenarioStep::new(
            SignalConfig::Log(log_config),
            BATCH_COUNT,
            DELAY,
        ));
        let config = Config::new(steps);

        // create our receiver
        let node_config = Rc::new(NodeUserConfig::new_receiver_config(
            OTAP_FAKE_DATA_GENERATOR_URN,
        ));
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
