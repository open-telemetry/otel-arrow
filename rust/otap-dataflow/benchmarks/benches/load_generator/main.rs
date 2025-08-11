// SPDX-License-Identifier: Apache-2.0

//! This benchmark tests the performance of the load generator

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fluke_hpack::Encoder;
use mimalloc::MiMalloc;
use otap_df_channel::mpsc;
use otap_df_engine::node::NodeWithPDataSender;
use otap_df_engine::{
    config::ReceiverConfig,
    message::{Receiver, Sender},
    receiver::ReceiverWrapper,
};

use otap_df_otap::fake_data_generator::FakeGeneratorReceiver;
use otap_df_otap::fake_data_generator::OTAP_FAKE_DATA_GENERATOR_URN;
use tokio::time::sleep;

use otap_df_config::node::NodeUserConfig;
use otap_df_engine::control::{Controllable, NodeControlMsg, pipeline_ctrl_msg_channel};
use otap_df_otlp::fake_signal_receiver::config::{Config, LoadConfig, OTLPSignal, TrafficConfig};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task::LocalSet;
use tokio::time::Duration;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use weaver_forge::registry::ResolvedRegistry;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
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

fn bench_load_gen(c: &mut Criterion) {
    // Use a single-threaded Tokio runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    // Pin the current thread to a core
    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    let core = cores.iter().last().expect("no cores found");
    _ = core_affinity::set_for_current(*core);

    let mut group = c.benchmark_group("Receiver");

    for message_rate in [10000, 20000, 30000, 40000] {
        // create data that will be used to benchmark the exporters

        // Benchmark the `start` function
        let _ = group.bench_with_input(
            BenchmarkId::new("load_generator", message_rate),
            &message_rate,
            |b, &message_rate| {
                b.to_async(&rt).iter(|| async {
                    // start perf exporter
                    let registry: ResolvedRegistry =
                        serde_json::from_str(RESOLVED_REGISTRY_JSON).unwrap();
                    let load = LoadConfig::new(1, 1, 3333);

                    let traffic_config = TrafficConfig::new(
                        message_rate,
                        Some(load.clone()),
                        Some(load.clone()),
                        Some(load.clone()),
                    );
                    let config = Config::new(traffic_config, registry);
                    let receiver_config = ReceiverConfig::new("fake_signal_receiver");
                    // create our receiver
                    let node_config = Arc::new(NodeUserConfig::new_receiver_config(
                        OTAP_FAKE_DATA_GENERATOR_URN,
                    ));
                    // create our receiver
                    let mut receiver = ReceiverWrapper::local(
                        FakeGeneratorReceiver::new(config),
                        node_config,
                        &receiver_config,
                    );
                    let control_sender = receiver.control_sender();

                    let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) =
                        pipeline_ctrl_msg_channel(10);
                    let (pdata_tx, pdata_rx) = mpsc::Channel::new(100);
                    let pdata_sender = Sender::new_local_mpsc_sender(pdata_tx);

                    receiver
                        .set_pdata_sender(receiver_config.name, "".into(), pdata_sender)
                        .expect("Failed to set pdata sender");

                    // start the exporter
                    let local = LocalSet::new();
                    let _run_exporter_handle = local.spawn_local(async move {
                        receiver
                            .start(pipeline_ctrl_msg_tx)
                            .await
                            .expect("Receiver event loop failed");
                    });
                    // wait for 1 second
                    sleep(Duration::from_millis(1000)).await;

                    _ = control_sender
                        .send(NodeControlMsg::Shutdown {
                            deadline: Duration::from_millis(1000),
                            reason: "shutdown".to_string(),
                        })
                        .await;
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_load_gen);
criterion_main!(benches);
