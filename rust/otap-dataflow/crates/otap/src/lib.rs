// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::pdata::OtapPdata;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;

/// Code for encoding OTAP batch from pdata view
pub mod encoder;
/// gRPC service implementation
pub mod grpc;
/// Implementation of OTAP Exporter that implements the exporter trait
pub mod otap_exporter;
/// Implementation of OTAP Receiver that implements the receiver trait
pub mod otap_receiver;

/// This receiver receives OTLP bytes from the grpc service request and
/// produce for the pipeline OTAP PData
pub mod otlp_receiver;

/// Implementation of OTLP exporter that implements the exporter trait
pub mod otlp_exporter;

/// Generated protobuf files
pub mod proto;

pub mod pdata;

pub mod parquet_exporter;

pub mod perf_exporter;

pub mod fake_data_generator;
/// testing utilities
#[cfg(test)]
mod mock;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata)]
static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();

#[cfg(test)]
mod tests {
    use crate::OTAP_PIPELINE_FACTORY;
    use crate::fake_data_generator::OTAP_FAKE_DATA_GENERATOR_URN;
    use crate::perf_exporter::exporter::OTAP_PERF_EXPORTER_URN;
    use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};
    use serde_json::json;

    #[test]
    fn test_mini_pipeline() {
        let config = PipelineConfigBuilder::new()
            .add_receiver(
                "receiver",
                OTAP_FAKE_DATA_GENERATOR_URN,
                Some(json!({
                    "steps": [
                        {
                            "batches_to_generate": 100,
                            "signal_type": {
                                "Logs": {
                                    "resource_count": 1,
                                    "scope_count": 1
                                }
                            }
                        }
                    ],
                    "resolved_registry": {
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
                })),
            )
            .add_exporter(
                "exporter",
                OTAP_PERF_EXPORTER_URN,
                Some(json!({
                    "disk_usage": false,
                    "io_usage": false
                })),
            )
            // ToDo(LQ): Check the validity of the outport.
            .broadcast("receiver", "out_port", ["exporter"])
            .build(PipelineType::Otap, "pgroup", "pipeline")
            .expect("Failed to build pipeline config");

        let runtime_pipeline = OTAP_PIPELINE_FACTORY
            .build(config)
            .expect("Failed to create runtime pipeline");
        assert_eq!(
            runtime_pipeline.node_count(),
            2,
            "Expected 2 nodes in the pipeline"
        );

        runtime_pipeline.start().expect("Failed to start pipeline");
    }
}
