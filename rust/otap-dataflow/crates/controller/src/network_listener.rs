// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Controller-side network listener plan extraction.
//!
//! This module is deliberately conservative and data-only: it extracts known
//! receiver bind identities from resolved configuration and combines them with
//! controller-owned placement metadata. It does not bind sockets, enable
//! `SO_REUSEPORT`, or attach any selector.

use crate::placement::PipelinePlacement;
use otel_arrow_dfe_config::{LogicalPipelineDeploymentKey, engine::ResolvedPipelineConfig};
use otel_arrow_dfe_engine::network_listener::{
    NetworkEndpoint, NetworkListenerMember, NetworkListenerPlan, NetworkTransport,
};
use otel_arrow_dfe_telemetry::otel_warn;
use std::io;
use std::net::SocketAddr;

// TODO(#3687): Replace controller-owned receiver schema extraction with
// receiver-declared listener identities. Distinguish declared-none from
// undeclared before network plans drive coordination.
const KNOWN_RECEIVER_URNS: &[&str] = &[
    "urn:otel:receiver:otlp",
    "urn:otel:receiver:otap",
    "urn:otel:receiver:syslog_cef",
];

fn parse_listening_addr(value: &serde_json::Value) -> Option<Result<SocketAddr, String>> {
    // Network plans only describe concrete socket bind identities. Hostnames
    // can resolve differently over time, so they are left to future resolver
    // integration instead of being guessed here.
    let raw = value.get("listening_addr")?;
    Some(if let Some(raw) = raw.as_str() {
        raw.parse().map_err(|err| format!("`{raw}`: {err}"))
    } else {
        Err(format!("expected string, got {raw}"))
    })
}

fn push_listening_addr(
    addresses: &mut Vec<(NetworkTransport, SocketAddr)>,
    node_id: &str,
    receiver_urn: &str,
    config_path: &str,
    value: &serde_json::Value,
    protocol: NetworkTransport,
) {
    match parse_listening_addr(value) {
        Some(Ok(addr)) => addresses.push((protocol, addr)),
        Some(Err(error)) => {
            otel_warn!(
                "controller.network_listener.addr_skipped",
                node_id = node_id,
                receiver_urn = receiver_urn,
                config_path = config_path,
                error = error.as_str()
            );
        }
        None => {}
    }
}

fn endpoint_addresses(
    node_id: &str,
    receiver_urn: &str,
    config: &serde_json::Value,
) -> Vec<(NetworkTransport, SocketAddr)> {
    let mut addresses = Vec::new();

    push_listening_addr(
        &mut addresses,
        node_id,
        receiver_urn,
        "listening_addr",
        config,
        NetworkTransport::Tcp,
    );

    if let Some(protocols) = config.get("protocols") {
        for protocol in ["grpc", "http"] {
            if let Some(protocol_config) = protocols.get(protocol) {
                push_listening_addr(
                    &mut addresses,
                    node_id,
                    receiver_urn,
                    &format!("protocols.{protocol}.listening_addr"),
                    protocol_config,
                    NetworkTransport::Tcp,
                );
            }
        }
    }

    if let Some(protocol_config) = config
        .get("protocol")
        .and_then(|protocol| protocol.get("tcp"))
    {
        push_listening_addr(
            &mut addresses,
            node_id,
            receiver_urn,
            "protocol.tcp.listening_addr",
            protocol_config,
            NetworkTransport::Tcp,
        );
    }

    addresses.sort();
    addresses.dedup();
    addresses
}

pub(crate) fn plan_for_pipeline(
    pipeline: &ResolvedPipelineConfig,
    placement: &PipelinePlacement,
    deployment_key: LogicalPipelineDeploymentKey,
) -> io::Result<NetworkListenerPlan> {
    let members = placement
        .cores
        .iter()
        .map(|core| NetworkListenerMember {
            core_id: core.core_id.id,
            numa_node_id: core.known_numa_node_id,
        })
        .collect::<Vec<_>>();

    let mut endpoints = Vec::new();
    for (node_id, node_cfg) in pipeline.pipeline.node_iter() {
        if !KNOWN_RECEIVER_URNS.contains(&node_cfg.r#type.as_str()) {
            continue;
        }

        for (protocol, bind_address) in
            endpoint_addresses(node_id.as_ref(), node_cfg.r#type.as_str(), &node_cfg.config)
        {
            endpoints.push(NetworkEndpoint::new(
                node_id.clone(),
                bind_address,
                protocol,
            ));
        }
    }

    NetworkListenerPlan::new(deployment_key, members, endpoints)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::placement::{CorePlacement, PipelinePlacement};
    use core_affinity::CoreId;
    use otel_arrow_dfe_config::engine::{ResolvedPipelineConfig, ResolvedPipelineRole};
    use otel_arrow_dfe_config::pipeline::PipelineConfig;
    use otel_arrow_dfe_config::policy::ResolvedPolicies;
    use otel_arrow_dfe_engine::topology::TopologyCompleteness;

    fn placement() -> PipelinePlacement {
        PipelinePlacement {
            pipeline_group_id: "pg".into(),
            pipeline_id: "pipe".into(),
            cores: vec![
                CorePlacement {
                    core_id: CoreId { id: 2 },
                    numa_node_id: 0,
                    known_numa_node_id: Some(0),
                    topology_completeness: TopologyCompleteness::Complete,
                },
                CorePlacement {
                    core_id: CoreId { id: 3 },
                    numa_node_id: 1,
                    known_numa_node_id: Some(1),
                    topology_completeness: TopologyCompleteness::Complete,
                },
            ],
        }
    }

    fn resolved_pipeline(yaml: &str) -> ResolvedPipelineConfig {
        ResolvedPipelineConfig {
            pipeline_group_id: "pg".into(),
            pipeline_id: "pipe".into(),
            pipeline: PipelineConfig::from_yaml("pg".into(), "pipe".into(), yaml).unwrap(),
            topic_scope: otel_arrow_dfe_config::topic::TopicScope::GLOBAL,
            policies: ResolvedPolicies::default(),
            role: ResolvedPipelineRole::Regular,
        }
    }

    fn deployment_key(placement_generation: u64) -> LogicalPipelineDeploymentKey {
        LogicalPipelineDeploymentKey::new(
            otel_arrow_dfe_config::PipelineKey::new("pg".into(), "pipe".into()),
            placement_generation,
        )
    }

    /// Scenario: an OTLP receiver exposes both gRPC and HTTP listening
    /// addresses.
    /// Guarantees: network planning emits one TCP endpoint for
    /// each configured OTLP bind address.
    #[test]
    fn extracts_otlp_grpc_and_http_endpoints() {
        let pipeline = resolved_pipeline(
            r#"
nodes:
  otlp:
    type: "urn:otel:receiver:otlp"
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
        http:
          listening_addr: "127.0.0.1:4318"
"#,
        );

        let plan = plan_for_pipeline(&pipeline, &placement(), deployment_key(7)).unwrap();

        assert_eq!(plan.deployment_key.placement_generation(), 7);
        assert_eq!(plan.endpoints.len(), 2);
        assert!(plan.endpoints.iter().any(|endpoint| endpoint
            == &NetworkEndpoint::new(
                "otlp".into(),
                "127.0.0.1:4317".parse().unwrap(),
                NetworkTransport::Tcp
            )));
        assert!(plan.endpoints.iter().any(|endpoint| endpoint
            == &NetworkEndpoint::new(
                "otlp".into(),
                "127.0.0.1:4318".parse().unwrap(),
                NetworkTransport::Tcp
            )));
        assert_eq!(plan.members.len(), 2);
    }

    /// Scenario: an OTAP receiver exposes a top-level listening address.
    /// Guarantees: listener planning recognizes the OTAP bind identity as a
    /// TCP endpoint.
    #[test]
    fn extracts_otap_top_level_endpoint() {
        let pipeline = resolved_pipeline(
            r#"
nodes:
  otap:
    type: "urn:otel:receiver:otap"
    config:
      listening_addr: "127.0.0.1:9000"
"#,
        );

        let plan = plan_for_pipeline(&pipeline, &placement(), deployment_key(0)).unwrap();

        assert_eq!(plan.endpoints.len(), 1);
        assert_eq!(plan.endpoints[0].receiver_node_id.as_ref(), "otap");
        assert_eq!(plan.endpoints[0].transport, NetworkTransport::Tcp);
    }

    /// Scenario: multiple listener-capable receiver nodes are discovered in a
    /// pipeline config.
    /// Guarantees: network endpoints are returned in deterministic key
    /// order independent of config map iteration.
    #[test]
    fn network_endpoints_are_sorted_by_key() {
        let pipeline = resolved_pipeline(
            r#"
nodes:
  z_otap:
    type: "urn:otel:receiver:otap"
    config:
      listening_addr: "127.0.0.1:9000"
  a_otlp:
    type: "urn:otel:receiver:otlp"
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
"#,
        );

        let plan = plan_for_pipeline(&pipeline, &placement(), deployment_key(0)).unwrap();

        assert_eq!(plan.endpoints.len(), 2);
        assert_eq!(plan.endpoints[0].receiver_node_id.as_ref(), "a_otlp");
        assert_eq!(plan.endpoints[1].receiver_node_id.as_ref(), "z_otap");
    }

    /// Scenario: a syslog CEF receiver exposes only a UDP listening address.
    /// Guarantees: the TCP-first network listener plan leaves UDP unmanaged
    /// until a generation-aware UDP accept API is implemented.
    #[test]
    fn ignores_syslog_udp_endpoint() {
        let pipeline = resolved_pipeline(
            r#"
nodes:
  syslog:
    type: "urn:otel:receiver:syslog_cef"
    config:
      protocol:
        udp:
          listening_addr: "127.0.0.1:5140"
"#,
        );

        let plan = plan_for_pipeline(&pipeline, &placement(), deployment_key(0)).unwrap();

        assert!(plan.endpoints.is_empty());
        assert_eq!(plan.members[1].numa_node_id, Some(1));
    }

    /// Scenario: pipeline nodes either use unknown receiver URNs or known
    /// receivers with unsupported config shapes.
    /// Guarantees: listener planning skips unsupported nodes instead of
    /// guessing bind identities.
    #[test]
    fn ignores_unknown_receivers_and_unknown_shapes() {
        let pipeline = resolved_pipeline(
            r#"
nodes:
  unknown:
    type: "urn:otel:receiver:custom"
    config:
      listening_addr: "127.0.0.1:1234"
  otlp:
    type: "urn:otel:receiver:otlp"
    config:
      endpoint: "127.0.0.1:4317"
"#,
        );

        let plan = plan_for_pipeline(&pipeline, &placement(), deployment_key(0)).unwrap();

        assert!(plan.endpoints.is_empty());
    }

    /// Scenario: listener planning receives placement built from unknown NUMA
    /// topology.
    /// Guarantees: listener members keep NUMA node metadata unknown rather than
    /// publishing fallback node zero as discovered topology.
    #[test]
    fn unknown_topology_keeps_numa_nodes_unknown() {
        let mut placement = placement();
        for core in &mut placement.cores {
            core.topology_completeness = TopologyCompleteness::Unknown;
            core.known_numa_node_id = None;
        }
        let pipeline = resolved_pipeline(
            r#"
nodes:
  otap:
    type: "urn:otel:receiver:otap"
    config:
      listening_addr: "127.0.0.1:9000"
"#,
        );

        let plan = plan_for_pipeline(&pipeline, &placement, deployment_key(0)).unwrap();

        assert_eq!(plan.members[0].numa_node_id, None);
        assert_eq!(plan.members[1].numa_node_id, None);
    }

    /// Scenario: listener planning receives partial topology where one
    /// selected core has no NUMA mapping.
    /// Guarantees: mapped cores retain their NUMA node and unmapped cores remain
    /// unknown in network metadata.
    #[test]
    fn partial_topology_keeps_unmapped_member_numa_node_unknown() {
        let mut placement = placement();
        placement.cores[1].topology_completeness = TopologyCompleteness::Partial;
        placement.cores[1].numa_node_id = 0;
        placement.cores[1].known_numa_node_id = None;
        let pipeline = resolved_pipeline(
            r#"
nodes:
  otap:
    type: "urn:otel:receiver:otap"
    config:
      listening_addr: "127.0.0.1:9000"
"#,
        );

        let plan = plan_for_pipeline(&pipeline, &placement, deployment_key(0)).unwrap();

        assert_eq!(plan.members[0].numa_node_id, Some(0));
        assert_eq!(plan.members[1].numa_node_id, None);
    }
}
