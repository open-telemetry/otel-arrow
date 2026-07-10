// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Controller-side listener-group plan extraction.
//!
//! This module is deliberately conservative and data-only: it extracts known
//! receiver bind identities from resolved configuration and combines them with
//! controller-owned placement metadata. It does not bind sockets, enable
//! `SO_REUSEPORT`, or attach any selector.

use crate::placement::PipelinePlacement;
use otap_df_config::engine::ResolvedPipelineConfig;
use otap_df_engine::listener_group::{
    ListenerGroupKey, ListenerGroupMember, ListenerGroupPlan, ListenerGroupSnapshot,
    ListenerProtocol,
};
use std::net::SocketAddr;

const KNOWN_RECEIVER_URNS: &[&str] = &[
    "urn:otel:receiver:otlp",
    "urn:otel:receiver:otap",
    "urn:otel:receiver:syslog_cef",
];

fn parse_listening_addr(value: &serde_json::Value) -> Option<SocketAddr> {
    value
        .get("listening_addr")
        .and_then(serde_json::Value::as_str)
        .and_then(|raw| raw.parse().ok())
}

fn listener_addresses(config: &serde_json::Value) -> Vec<(ListenerProtocol, SocketAddr)> {
    let mut addresses = Vec::new();

    if let Some(addr) = parse_listening_addr(config) {
        addresses.push((ListenerProtocol::Tcp, addr));
    }

    if let Some(protocols) = config.get("protocols") {
        for protocol in ["grpc", "http"] {
            if let Some(addr) = protocols.get(protocol).and_then(parse_listening_addr) {
                addresses.push((ListenerProtocol::Tcp, addr));
            }
        }
    }

    if let Some(protocol) = config.get("protocol") {
        for (key, listener_protocol) in [
            ("tcp", ListenerProtocol::Tcp),
            ("udp", ListenerProtocol::Udp),
        ] {
            if let Some(addr) = protocol.get(key).and_then(parse_listening_addr) {
                addresses.push((listener_protocol, addr));
            }
        }
    }

    addresses.sort();
    addresses.dedup();
    addresses
}

pub(crate) fn snapshot_for_pipeline(
    pipeline: &ResolvedPipelineConfig,
    placement: &PipelinePlacement,
    generation: u64,
) -> ListenerGroupSnapshot {
    let members = placement
        .cores
        .iter()
        .enumerate()
        .map(|(idx, core)| ListenerGroupMember {
            listener_id: idx as u32,
            core_id: core.core_id.id,
            numa_node_id: if core.topology_completeness
                == otap_df_engine::topology::TopologyCompleteness::Unknown
            {
                None
            } else {
                Some(core.numa_node_id)
            },
        })
        .collect::<Vec<_>>();

    if members.is_empty() {
        return ListenerGroupSnapshot::new(generation, Vec::new());
    }

    let mut plans = Vec::new();
    for (node_id, node_cfg) in pipeline.pipeline.node_iter() {
        if !KNOWN_RECEIVER_URNS.contains(&node_cfg.r#type.as_str()) {
            continue;
        }

        for (protocol, bind_address) in listener_addresses(&node_cfg.config) {
            plans.push(ListenerGroupPlan {
                key: ListenerGroupKey::new(
                    pipeline.pipeline_group_id.clone(),
                    pipeline.pipeline_id.clone(),
                    node_id.clone(),
                    bind_address,
                    protocol,
                ),
                expected_members: members.clone(),
            });
        }
    }

    ListenerGroupSnapshot::new(generation, plans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::placement::{CorePlacement, PipelinePlacement};
    use core_affinity::CoreId;
    use otap_df_config::engine::{ResolvedPipelineConfig, ResolvedPipelineRole};
    use otap_df_config::pipeline::PipelineConfig;
    use otap_df_config::policy::ResolvedPolicies;
    use otap_df_engine::topology::TopologyCompleteness;

    fn placement() -> PipelinePlacement {
        PipelinePlacement {
            pipeline_group_id: "pg".into(),
            pipeline_id: "pipe".into(),
            cores: vec![
                CorePlacement {
                    core_id: CoreId { id: 2 },
                    numa_node_id: 0,
                    topology_completeness: TopologyCompleteness::Complete,
                },
                CorePlacement {
                    core_id: CoreId { id: 3 },
                    numa_node_id: 1,
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
            policies: ResolvedPolicies::default(),
            role: ResolvedPipelineRole::Regular,
        }
    }

    #[test]
    fn extracts_otlp_grpc_and_http_listener_groups() {
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

        let snapshot = snapshot_for_pipeline(&pipeline, &placement(), 7);

        assert_eq!(snapshot.generation, 7);
        assert_eq!(snapshot.plans.len(), 2);
        assert!(
            snapshot
                .plan_for(
                    "otlp",
                    "127.0.0.1:4317".parse().unwrap(),
                    ListenerProtocol::Tcp
                )
                .is_some()
        );
        assert!(
            snapshot
                .plan_for(
                    "otlp",
                    "127.0.0.1:4318".parse().unwrap(),
                    ListenerProtocol::Tcp
                )
                .is_some()
        );
        assert_eq!(snapshot.plans[0].expected_members.len(), 2);
    }

    #[test]
    fn extracts_otap_top_level_listener_group() {
        let pipeline = resolved_pipeline(
            r#"
nodes:
  otap:
    type: "urn:otel:receiver:otap"
    config:
      listening_addr: "127.0.0.1:9000"
"#,
        );

        let snapshot = snapshot_for_pipeline(&pipeline, &placement(), 0);

        assert_eq!(snapshot.plans.len(), 1);
        assert_eq!(snapshot.plans[0].key.receiver_node_id.as_ref(), "otap");
        assert_eq!(snapshot.plans[0].key.protocol, ListenerProtocol::Tcp);
    }

    #[test]
    fn extracts_syslog_udp_listener_group() {
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

        let snapshot = snapshot_for_pipeline(&pipeline, &placement(), 0);

        assert_eq!(snapshot.plans.len(), 1);
        assert_eq!(snapshot.plans[0].key.protocol, ListenerProtocol::Udp);
        assert_eq!(snapshot.plans[0].expected_members[1].numa_node_id, Some(1));
    }

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

        let snapshot = snapshot_for_pipeline(&pipeline, &placement(), 0);

        assert!(snapshot.plans.is_empty());
    }

    #[test]
    fn unknown_topology_keeps_numa_nodes_unknown() {
        let mut placement = placement();
        for core in &mut placement.cores {
            core.topology_completeness = TopologyCompleteness::Unknown;
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

        let snapshot = snapshot_for_pipeline(&pipeline, &placement, 0);

        assert_eq!(snapshot.plans[0].expected_members[0].numa_node_id, None);
        assert_eq!(snapshot.plans[0].expected_members[1].numa_node_id, None);
    }
}
