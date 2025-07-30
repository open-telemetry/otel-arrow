// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP receiver node
//!
//! ToDo: implement Ack and Nack control message, wait for receiver node to receive a Ack control message then the service can send a response back
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//!

// use crate::FAKE_SIGNAL_RECEIVERS;

use crate::fake_signal_receiver::config::{Config, OTLPSignal, SignalType};
use crate::fake_signal_receiver::fake_signal::{
    fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
};
use async_trait::async_trait;
use otap_df_engine::control::ControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use serde_json::Value;
use tokio::time::{Duration, sleep};

/// The URN for the fake signal receiver
pub const FAKE_SIGNAL_RECEIVER_URN: &str = "urn:otel:fake:signal:receiver";

/// A Receiver that listens for OTLP messages
pub struct FakeSignalReceiver {
    config: Config,
}

// ToDo: The fake signal receiver pdata type is not the same as the other OTLP nodes which are based on the OTLPSignal type. We must unify this in the future.
/// Declares the Fake Signal receiver as a local receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
// #[allow(unsafe_code)]
// #[distributed_slice(LOCAL_RECEIVERS)]
// pub static FAKE_SIGNAL_RECEIVER: LocalReceiverFactory<OTLPSignal> = LocalReceiverFactory {
//     name: "urn:otel:fake:signal:receiver",
//     create: |config: &Value| Box::new(FakeSignalReceiver::from_config(config)),
// };

impl FakeSignalReceiver {
    /// creates a new FakeSignalReceiver
    #[must_use]
    pub fn new(config: Config) -> Self {
        FakeSignalReceiver { config }
    }

    /// Creates a new FakeSignalReceiver from a configuration object
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(FakeSignalReceiver { config })
    }
}

// We use the local version of the receiver here since we don't need to worry about Send and Sync traits
#[async_trait( ? Send)]
impl local::Receiver<OTLPSignal> for FakeSignalReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel,
        effect_handler: local::EffectHandler<OTLPSignal>,
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
                _ = run_scenario(&self.config, effect_handler.clone()) => {
                    // do nothing
                }

            }
        }
        //Exit event loop
        Ok(())
    }
}

/// Run the configured scenario steps
async fn run_scenario(config: &Config, effect_handler: local::EffectHandler<OTLPSignal>) {
    // loop through each step
    let steps = config.get_steps();
    let registry = config.get_registry();
    for step in steps {
        // create batches if specified
        let batches = step.get_batches_to_generate() as usize;
        for _ in 0..batches {
            let signal = match step.get_signal_type() {
                SignalType::Metrics(load) => OTLPSignal::Metrics(fake_otlp_metrics(
                    load.resource_count(),
                    load.scope_count(),
                    registry,
                )),
                SignalType::Logs(load) => OTLPSignal::Logs(fake_otlp_logs(
                    load.resource_count(),
                    load.scope_count(),
                    registry,
                )),
                SignalType::Traces(load) => OTLPSignal::Traces(fake_otlp_traces(
                    load.resource_count(),
                    load.scope_count(),
                    registry,
                )),
            };
            _ = effect_handler.send_message(signal).await;
            // if there is a delay set between batches sleep for that amount before created the next signal in the batch
            sleep(Duration::from_millis(step.get_delay_between_batches_ms())).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fake_signal_receiver::{
        config::{Config, Load, OTLPSignal, ScenarioStep, SignalType},
        receiver::{FAKE_SIGNAL_RECEIVER_URN, FakeSignalReceiver},
    };
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use otel_arrow_rust::proto::opentelemetry::metrics::v1::metric::Data;
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
    const RESOLVED_REGISTRY_FILE: &str = "src/fake_signal_receiver/resolved_registry.yaml";

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
    fn test_fake_signal_receiver() {
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
            FAKE_SIGNAL_RECEIVER_URN,
        ));
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeSignalReceiver::new(config),
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
