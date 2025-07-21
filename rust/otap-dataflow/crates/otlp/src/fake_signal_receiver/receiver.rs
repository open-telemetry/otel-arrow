// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP receiver node
//!
//! ToDo: implement Ack and Nack control message, wait for receiver node to receive a Ack control message then the service can send a response back
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//!

use crate::FAKE_SIGNAL_RECEIVERS;

use crate::fake_signal_receiver::config::{Config, OTLPSignal, ScenarioStep, SignalConfig};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_engine::error::Error;
use otap_df_engine::message::ControlMsg;
use otap_df_engine::shared::{SharedReceiverFactory, receiver as shared};
use serde_json::Value;
use tokio::time::{Duration, sleep};

/// A Receiver that listens for OTLP messages
pub struct FakeSignalReceiver {
    config: Config,
}

/// Declares the Fake Signal receiver as a shared receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(FAKE_SIGNAL_RECEIVERS)]
pub static FAKE_SIGNAL_RECEIVER: SharedReceiverFactory<OTLPSignal> = SharedReceiverFactory {
    name: "urn:otel:fake:signal:receiver",
    create: |config: &Value| Box::new(FakeSignalReceiver::from_config(config)),
};

impl FakeSignalReceiver {
    /// creates a new FakeSignalReceiver
    #[must_use]
    pub fn new(config: Config) -> Self {
        FakeSignalReceiver { config }
    }

    /// Creates a new FakeSignalReceiver from a configuration object
    #[must_use]
    pub fn from_config(config: &Value) -> Self {
        let config: Config =
            serde_json::from_value(config.clone()).unwrap_or_else(|_| Config::new(vec![]));
        FakeSignalReceiver { config }
    }
}

// Use the async_trait due to the need for thread safety because of tonic requiring Send and Sync traits
// The Shared version of the receiver allows us to implement a Receiver that requires the effect handler to be Send and Sync
#[async_trait]
impl shared::Receiver<OTLPSignal> for FakeSignalReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel,
        effect_handler: shared::EffectHandler<OTLPSignal>,
    ) -> Result<(), Error<OTLPSignal>> {
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
                _ = run_scenario(self.config.get_steps(), effect_handler.clone()) => {
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
    effect_handler: shared::EffectHandler<OTLPSignal>,
) {
    // loop through each step

    for step in steps {
        // create batches if specified
        let batches = step.get_batches() as usize;
        for _ in 0..batches {
            let signal = match step.get_config() {
                SignalConfig::Metric(config) => OTLPSignal::Metric(config.get_signal()),
                SignalConfig::Log(config) => OTLPSignal::Log(config.get_signal()),
                SignalConfig::Span(config) => OTLPSignal::Span(config.get_signal()),
            };
            _ = effect_handler.send_message(signal).await;
            // if there is a delay set between batches sleep for that amount before created the next signal in the batch
            sleep(Duration::from_millis(step.get_delay_between_batch())).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fake_signal_receiver::{
        config::{
            Config, LogConfig, MetricConfig, MetricType, OTLPSignal, ScenarioStep, SignalConfig,
            SpanConfig,
        },
        receiver::FakeSignalReceiver,
    };
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use otel_arrow_rust::proto::opentelemetry::metrics::v1::metric::Data;
    use std::future::Future;
    use std::pin::Pin;
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
    -> impl FnOnce(NotSendValidateContext<OTLPSignal>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // check that messages have been sent through the effect_handler

                // read from the effect handler
                let metric_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");
                let trace_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");
                let log_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                // Assert that the message received is what the test client sent.
                match metric_received {
                    OTLPSignal::Metric(metric) => {
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
                    OTLPSignal::Span(span) => {
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
                    OTLPSignal::Log(log) => {
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
        let receiver =
            ReceiverWrapper::shared(FakeSignalReceiver::new(config), test_runtime.config());

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario())
            .run_validation(validation_procedure());
    }
}
