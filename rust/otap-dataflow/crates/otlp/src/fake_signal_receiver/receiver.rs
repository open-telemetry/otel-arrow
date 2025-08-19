// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP receiver node
//!
//! ToDo: implement Ack and Nack control message, wait for receiver node to receive a Ack control message then the service can send a response back
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//!

// use crate::FAKE_SIGNAL_RECEIVERS;

use crate::fake_signal_receiver::config::{Config, OTLPSignal};
use crate::fake_signal_receiver::fake_signal::{
    fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
};
use async_trait::async_trait;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use serde_json::Value;
use tokio::time::{Duration, Instant, sleep};
use weaver_forge::registry::ResolvedRegistry;

/// The URN for the fake signal receiver
pub const FAKE_SIGNAL_RECEIVER_URN: &str = "urn:otel:fake:signal:receiver";

/// A Receiver that listens for OTLP messages
pub struct FakeSignalReceiver {
    config: Config,
}

// ToDo: The fake signal receiver pdata type is not the same as the other OTLP nodes which are based on the OTLPSignal type. We must unify this in the future.
// /// Declares the Fake Signal receiver as a local receiver factory
// ///
// /// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
// /// This macro is part of the `linkme` crate which is considered safe and well maintained.
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

        let mut signal_count: u64 = 0;

        let one_second_duration = Duration::from_secs(1);
        loop {
            // wait one second from now to generate more signals
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
                signal_status = generate_signal(effect_handler.clone(), max_signal_count, &mut signal_count, metric_count, trace_count, log_count, &registry), if max_signal_count.is_none_or(|max| max > signal_count) => {
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
    effect_handler: local::EffectHandler<OTLPSignal>,
    max_signal_count: Option<u64>,
    signal_count: &mut u64,
    metric_count: usize,
    trace_count: usize,
    log_count: usize,
    registry: &ResolvedRegistry,
) -> Result<(), Error<OTLPSignal>> {
    // ToDo: will need to check in if loop if generating signals will cause the signal count to be more than max
    // this will allow us to generate some but not all signals
    if let Some(max_count) = max_signal_count {
        // don't generate signals if we reached max signal
        let mut current_count = *signal_count;
        if current_count >= max_count {
            return Ok(());
        }
        // update the counts here to allow us to reach the max_signal_count

        if metric_count > 0 && max_count >= current_count + metric_count as u64 {
            effect_handler
                .send_message(OTLPSignal::Metrics(fake_otlp_metrics(
                    metric_count,
                    registry,
                )))
                .await?;
            current_count += metric_count as u64;
        }

        // generate and send traces
        if trace_count > 0 && max_count >= current_count + metric_count as u64 {
            effect_handler
                .send_message(OTLPSignal::Traces(fake_otlp_traces(trace_count, registry)))
                .await?;
            current_count += trace_count as u64;
        }

        // generate and send logs
        if log_count > 0 && max_count >= current_count + metric_count as u64 {
            effect_handler
                .send_message(OTLPSignal::Logs(fake_otlp_logs(log_count, registry)))
                .await?;
            current_count += log_count as u64;
        }

        *signal_count = current_count;
    } else {
        // generate and send metric
        if metric_count > 0 {
            effect_handler
                .send_message(OTLPSignal::Metrics(fake_otlp_metrics(
                    metric_count,
                    registry,
                )))
                .await?;
        }

        // generate and send traces
        if trace_count > 0 {
            effect_handler
                .send_message(OTLPSignal::Traces(fake_otlp_traces(trace_count, registry)))
                .await?;
        }

        // generate and send logs
        if log_count > 0 {
            effect_handler
                .send_message(OTLPSignal::Logs(fake_otlp_logs(log_count, registry)))
                .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::fake_signal_receiver::{
        config::{Config, OTLPSignal, TrafficConfig},
        receiver::{FAKE_SIGNAL_RECEIVER_URN, FakeSignalReceiver},
    };
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otel_arrow_rust::proto::opentelemetry::metrics::v1::metric::Data;
    use std::collections::HashSet;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::time::{Duration, sleep};
    use weaver_common::vdir::VirtualDirectoryPath;
    use weaver_forge::registry::ResolvedRegistry;

    const RESOURCE_COUNT: usize = 1;
    const SCOPE_COUNT: usize = 1;
    const MESSAGE_COUNT: usize = 1;
    const RUN_TILL_SHUTDOWN: u64 = 999;
    const MESSAGE_PER_SECOND: usize = 3;
    const MAX_SIGNALS: u64 = 3;

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
    ) -> impl FnOnce(NotSendValidateContext<OTLPSignal>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // check that messages have been sent through the effect_handler
                while let Ok(received_signal) = ctx.recv().await {
                    match received_signal {
                        OTLPSignal::Metrics(metric) => {
                            // loop and check count
                            let resource_count = metric.resource_metrics.len();
                            assert_eq!(resource_count, RESOURCE_COUNT);
                            for resource in metric.resource_metrics.iter() {
                                let scope_count = resource.scope_metrics.len();
                                assert_eq!(scope_count, SCOPE_COUNT);
                                for scope in resource.scope_metrics.iter() {
                                    let metric_count = scope.metrics.len();
                                    assert_eq!(metric_count, MESSAGE_COUNT);
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
                            assert_eq!(resource_count, RESOURCE_COUNT);
                            for resource in span.resource_spans.iter() {
                                let scope_count = resource.scope_spans.len();
                                assert_eq!(scope_count, SCOPE_COUNT);
                                for scope in resource.scope_spans.iter() {
                                    let span_count = scope.spans.len();
                                    assert_eq!(span_count, MESSAGE_COUNT);
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
                            assert_eq!(resource_count, RESOURCE_COUNT);
                            for resource in log.resource_logs.iter() {
                                let scope_count = resource.scope_logs.len();
                                assert_eq!(scope_count, SCOPE_COUNT);
                                for scope in resource.scope_logs.iter() {
                                    let log_record_count = scope.log_records.len();
                                    assert_eq!(log_record_count, MESSAGE_COUNT);
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

        let traffic_config = TrafficConfig::new(Some(MESSAGE_PER_SECOND), None, 1, 1, 1);
        let config = Config::new(traffic_config, registry_path);
        let registry = config.get_registry().expect("failed to get registry");

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            FAKE_SIGNAL_RECEIVER_URN,
        ));
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeSignalReceiver::new(config),
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
    -> impl FnOnce(NotSendValidateContext<OTLPSignal>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let mut received_messages = 0;

                while let Ok(received_signal) = ctx.recv().await {
                    match received_signal {
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

        let traffic_config = TrafficConfig::new(Some(MESSAGE_PER_SECOND), None, 1, 0, 0);
        let config = Config::new(traffic_config, registry_path);

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            FAKE_SIGNAL_RECEIVER_URN,
        ));
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeSignalReceiver::new(config),
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
    -> impl FnOnce(NotSendValidateContext<OTLPSignal>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let mut received_messages = 0;
                while let Ok(received_signal) = ctx.recv().await {
                    match received_signal {
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
                assert_eq!(received_messages as u64, MAX_SIGNALS);
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

        let traffic_config = TrafficConfig::new(None, Some(MAX_SIGNALS), 1, 0, 0);
        let config = Config::new(traffic_config, registry_path);

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            FAKE_SIGNAL_RECEIVER_URN,
        ));
        // create our receiver
        let receiver = ReceiverWrapper::local(
            FakeSignalReceiver::new(config),
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
