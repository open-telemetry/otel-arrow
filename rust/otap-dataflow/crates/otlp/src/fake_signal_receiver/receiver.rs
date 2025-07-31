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
    use std::future::Future;
    use std::pin::Pin;
    use std::rc::Rc;
    use tokio::time::{Duration, sleep, timeout};
    use weaver_forge::registry::ResolvedRegistry;

    const RESOURCE_COUNT: usize = 1;
    const SCOPE_COUNT: usize = 1;
    const BATCH_COUNT: u64 = 1;
    const DELAY: u64 = 0;

    const RESOLVED_REGISTRY_JSON: &str = r#"
    {
  "registry_url": "",
  "groups": [
    {
      "id": "metric.system.network.dropped",
      "type": "metric",
      "brief": "Count of packets that are dropped or discarded even though there was no error.",
      "note": "Measured as:\n\n- Linux: the `drop` column in `/proc/dev/net` ([source](https://web.archive.org/web/20180321091318/http://www.onlamp.com/pub/a/linux/2000/11/16/LinuxAdmin.html))\n- Windows: [`InDiscards`/`OutDiscards`](https://docs.microsoft.com/windows/win32/api/netioapi/ns-netioapi-mib_if_row2)\n  from [`GetIfEntry2`](https://docs.microsoft.com/windows/win32/api/netioapi/nf-netioapi-getifentry2)\n",
      "stability": "development",
      "attributes": [
        {
          "name": "network.io.direction",
          "type": {
            "members": [
              {
                "id": "transmit",
                "value": "transmit",
                "brief": null,
                "note": null,
                "stability": "development",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "receive",
                "value": "receive",
                "brief": null,
                "note": null,
                "stability": "development",
                "deprecated": null,
                "annotations": null
              }
            ]
          },
          "brief": "The network IO operation direction.",
          "examples": [
            "transmit"
          ],
          "requirement_level": "recommended",
          "stability": "development"
        },
        {
          "name": "network.interface.name",
          "type": "string",
          "brief": "The network interface name.",
          "examples": [
            "lo",
            "eth0"
          ],
          "requirement_level": "recommended",
          "stability": "development"
        }
      ],
      "span_kind": null,
      "events": [],
      "metric_name": "system.network.dropped",
      "instrument": "counter",
      "unit": "{packet}",
      "name": null,
      "lineage": {
        "provenance": {
          "registry_id": "main",
          "path": "https://github.com/open-telemetry/semantic-conventions.git[model]/system/metrics.yaml"
        },
        "attributes": {
          "network.interface.name": {
            "source_group": "registry.network",
            "inherited_fields": [
              "brief",
              "examples",
              "note",
              "requirement_level",
              "stability"
            ]
          },
          "network.io.direction": {
            "source_group": "registry.network",
            "inherited_fields": [
              "brief",
              "examples",
              "note",
              "requirement_level",
              "stability"
            ]
          }
        }
      },
      "entity_associations": [
        "host"
      ],
      "annotations": {
        "code_generation": {
          "metric_value_type": "int"
        }
      }
    },
    {
      "id": "span.rpc.client",
      "type": "span",
      "brief": "This span represents an outgoing Remote Procedure Call (RPC).",
      "note": "Remote procedure calls can only be represented with these semantic conventions\nwhen the names of the called service and method are known and available.\n\n**Span name:** refer to the [Span Name](#span-name) section.\n\n**Span kind** MUST be `CLIENT`.\n",
      "stability": "development",
      "attributes": [
        {
          "name": "rpc.method",
          "type": "string",
          "brief": "The name of the (logical) method being called, must be equal to the $method part in the span name.",
          "examples": "exampleMethod",
          "requirement_level": "recommended",
          "note": "This is the logical name of the method from the RPC interface perspective, which can be different from the name of any implementing method/function. The `code.function.name` attribute may be used to store the latter (e.g., method actually executing the call on the server side, RPC client stub method on the client side).\n",
          "stability": "development"
        },
        {
          "name": "rpc.service",
          "type": "string",
          "brief": "The full (logical) name of the service being called, including its package name, if applicable.",
          "examples": "myservice.EchoService",
          "requirement_level": "recommended",
          "note": "This is the logical name of the service from the RPC interface perspective, which can be different from the name of any implementing class. The `code.namespace` attribute may be used to store the latter (despite the attribute name, it may include a class name; e.g., class with method actually executing the call on the server side, RPC client stub class on the client side).\n",
          "stability": "development"
        },
        {
          "name": "rpc.system",
          "type": {
            "members": [
              {
                "id": "grpc",
                "value": "grpc",
                "brief": "gRPC",
                "note": null,
                "stability": "development",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "java_rmi",
                "value": "java_rmi",
                "brief": "Java RMI",
                "note": null,
                "stability": "development",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "dotnet_wcf",
                "value": "dotnet_wcf",
                "brief": ".NET WCF",
                "note": null,
                "stability": "development",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "apache_dubbo",
                "value": "apache_dubbo",
                "brief": "Apache Dubbo",
                "note": null,
                "stability": "development",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "connect_rpc",
                "value": "connect_rpc",
                "brief": "Connect RPC",
                "note": null,
                "stability": "development",
                "deprecated": null,
                "annotations": null
              }
            ]
          },
          "brief": "A string identifying the remoting system. See below for a list of well-known identifiers.",
          "requirement_level": "required",
          "stability": "development"
        },
        {
          "name": "network.peer.address",
          "type": "string",
          "brief": "Peer address of the network connection - IP address or Unix domain socket name.",
          "examples": [
            "10.1.2.80",
            "/tmp/my.sock"
          ],
          "requirement_level": "recommended",
          "stability": "stable"
        },
        {
          "name": "network.transport",
          "type": {
            "members": [
              {
                "id": "tcp",
                "value": "tcp",
                "brief": "TCP",
                "note": null,
                "stability": "stable",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "udp",
                "value": "udp",
                "brief": "UDP",
                "note": null,
                "stability": "stable",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "pipe",
                "value": "pipe",
                "brief": "Named or anonymous pipe.",
                "note": null,
                "stability": "stable",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "unix",
                "value": "unix",
                "brief": "Unix domain socket",
                "note": null,
                "stability": "stable",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "quic",
                "value": "quic",
                "brief": "QUIC",
                "note": null,
                "stability": "stable",
                "deprecated": null,
                "annotations": null
              }
            ]
          },
          "brief": "[OSI transport layer](https://wikipedia.org/wiki/Transport_layer) or [inter-process communication method](https://wikipedia.org/wiki/Inter-process_communication).\n",
          "examples": [
            "tcp",
            "udp"
          ],
          "requirement_level": "recommended",
          "note": "The value SHOULD be normalized to lowercase.\n\nConsider always setting the transport when setting a port number, since\na port number is ambiguous without knowing the transport. For example\ndifferent processes could be listening on TCP port 12345 and UDP port 12345.\n",
          "stability": "stable"
        },
        {
          "name": "network.type",
          "type": {
            "members": [
              {
                "id": "ipv4",
                "value": "ipv4",
                "brief": "IPv4",
                "note": null,
                "stability": "stable",
                "deprecated": null,
                "annotations": null
              },
              {
                "id": "ipv6",
                "value": "ipv6",
                "brief": "IPv6",
                "note": null,
                "stability": "stable",
                "deprecated": null,
                "annotations": null
              }
            ]
          },
          "brief": "[OSI network layer](https://wikipedia.org/wiki/Network_layer) or non-OSI equivalent.",
          "examples": [
            "ipv4",
            "ipv6"
          ],
          "requirement_level": "recommended",
          "note": "The value SHOULD be normalized to lowercase.",
          "stability": "stable"
        },
        {
          "name": "network.peer.port",
          "type": "int",
          "brief": "Peer port number of the network connection.",
          "examples": [
            65123
          ],
          "requirement_level": {
            "recommended": "If `network.peer.address` is set."
          },
          "stability": "stable"
        },
        {
          "name": "server.address",
          "type": "string",
          "brief": "RPC server [host name](https://grpc.github.io/grpc/core/md_doc_naming.html).\n",
          "examples": [
            "example.com",
            "10.1.2.80",
            "/tmp/my.sock"
          ],
          "requirement_level": "required",
          "note": "May contain server IP address, DNS name, or local socket name. When host component is an IP address, instrumentations SHOULD NOT do a reverse proxy lookup to obtain DNS name and SHOULD set `server.address` to the IP address provided in the host component.\n",
          "stability": "stable"
        },
        {
          "name": "server.port",
          "type": "int",
          "brief": "Server port number.",
          "examples": [
            80,
            8080,
            443
          ],
          "requirement_level": {
            "conditionally_required": "if the port is supported by the network transport used for communication."
          },
          "note": "When observed from the client side, and when communicating through an intermediary, `server.port` SHOULD represent the server port behind any intermediaries, for example proxies, if it's available.\n",
          "stability": "stable"
        }
      ],
      "span_kind": "client",
      "events": [
        "rpc.message"
      ],
      "metric_name": null,
      "instrument": null,
      "unit": null,
      "name": null,
      "lineage": {
        "provenance": {
          "registry_id": "main",
          "path": "https://github.com/open-telemetry/semantic-conventions.git[model]/rpc/spans.yaml"
        },
        "attributes": {
          "network.peer.address": {
            "source_group": "registry.network",
            "inherited_fields": [
              "brief",
              "examples",
              "note",
              "stability"
            ],
            "locally_overridden_fields": [
              "requirement_level"
            ]
          },
          "network.peer.port": {
            "source_group": "registry.network",
            "inherited_fields": [
              "brief",
              "examples",
              "note",
              "stability"
            ],
            "locally_overridden_fields": [
              "requirement_level"
            ]
          },
          "network.transport": {
            "source_group": "registry.network",
            "inherited_fields": [
              "brief",
              "examples",
              "note",
              "stability"
            ],
            "locally_overridden_fields": [
              "requirement_level"
            ]
          },
          "network.type": {
            "source_group": "registry.network",
            "inherited_fields": [
              "brief",
              "examples",
              "note",
              "stability"
            ],
            "locally_overridden_fields": [
              "requirement_level"
            ]
          },
          "rpc.method": {
            "source_group": "registry.rpc",
            "inherited_fields": [
              "brief",
              "examples",
              "note",
              "stability"
            ],
            "locally_overridden_fields": [
              "requirement_level"
            ]
          },
          "rpc.service": {
            "source_group": "registry.rpc",
            "inherited_fields": [
              "brief",
              "examples",
              "note",
              "stability"
            ],
            "locally_overridden_fields": [
              "requirement_level"
            ]
          },
          "rpc.system": {
            "source_group": "registry.rpc",
            "inherited_fields": [
              "brief",
              "note",
              "stability"
            ],
            "locally_overridden_fields": [
              "requirement_level"
            ]
          },
          "server.address": {
            "source_group": "registry.server",
            "inherited_fields": [
              "examples",
              "stability"
            ],
            "locally_overridden_fields": [
              "brief",
              "note",
              "requirement_level"
            ]
          },
          "server.port": {
            "source_group": "registry.server",
            "inherited_fields": [
              "brief",
              "examples",
              "note",
              "stability"
            ],
            "locally_overridden_fields": [
              "requirement_level"
            ]
          }
        }
      }
    },
    {
      "id": "event.session.end",
      "type": "event",
      "brief": "Indicates that a session has ended.\n",
      "note": "For instrumentation that tracks user behavior during user sessions, a `session.end` event SHOULD be emitted every time a session ends. When a session ends and continues as a new session, this event SHOULD be emitted prior to the `session.start` event.\n",
      "stability": "development",
      "attributes": [
        {
          "name": "session.id",
          "type": "string",
          "brief": "The ID of the session being ended.",
          "examples": "00112233-4455-6677-8899-aabbccddeeff",
          "requirement_level": "required",
          "stability": "development"
        }
      ],
      "span_kind": null,
      "events": [],
      "metric_name": null,
      "instrument": null,
      "unit": null,
      "name": "session.end",
      "lineage": {
        "provenance": {
          "registry_id": "main",
          "path": "https://github.com/open-telemetry/semantic-conventions.git[model]/session/events.yaml"
        },
        "attributes": {
          "session.id": {
            "source_group": "registry.session",
            "inherited_fields": [
              "examples",
              "note",
              "stability"
            ],
            "locally_overridden_fields": [
              "brief",
              "requirement_level"
            ]
          }
        }
      }
    }
  ]
}
"#;

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
        "rpc.system",
        "network.peer.address",
        "network.transport",
        "network.type",
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
                                    if metric.name == METRIC_NAME {
                                        metric_seen = true;
                                        assert!(metric.description.as_str() == METRIC_DESC);
                                        assert!(metric.unit.as_str() == METRIC_UNIT);
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
                                            _ => unreachable!(),
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
                                    if span.name == SPAN_NAME {
                                        span_seen = true;
                                        let keys: Vec<&str> = span
                                            .attributes
                                            .iter()
                                            .map(|attribute| attribute.key.as_str())
                                            .collect();
                                        assert!(keys == SPAN_ATTR.to_vec());
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
                                    if log_record.event_name == LOG_NAME {
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

        let registry: ResolvedRegistry = serde_json::from_str(RESOLVED_REGISTRY_JSON).unwrap();

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

        let config_string = serde_json::to_string(&config).unwrap();
        println!("{}", config_string);

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
