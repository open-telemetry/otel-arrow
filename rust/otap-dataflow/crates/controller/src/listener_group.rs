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
use otap_df_telemetry::otel_warn;
use std::net::SocketAddr;

// TODO: Replace this conservative extraction with receiver/factory-declared
// bind identities so listener-group planning does not hardcode receiver URNs
// and config shapes in the controller.
const KNOWN_RECEIVER_URNS: &[&str] = &[
    "urn:otel:receiver:otlp",
    "urn:otel:receiver:otap",
    "urn:otel:receiver:syslog_cef",
];

fn parse_listening_addr(value: &serde_json::Value) -> Option<Result<SocketAddr, String>> {
    // Listener groups only describe concrete socket bind identities. Hostnames
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
    addresses: &mut Vec<(ListenerProtocol, SocketAddr)>,
    node_id: &str,
    receiver_urn: &str,
    config_path: &str,
    value: &serde_json::Value,
    protocol: ListenerProtocol,
) {
    match parse_listening_addr(value) {
        Some(Ok(addr)) => addresses.push((protocol, addr)),
        Some(Err(error)) => {
            otel_warn!(
                "controller.listener_group.addr_skipped",
                node_id = node_id,
                receiver_urn = receiver_urn,
                config_path = config_path,
                error = error.as_str()
            );
        }
        None => {}
    }
}

fn listener_addresses(
    node_id: &str,
    receiver_urn: &str,
    config: &serde_json::Value,
) -> Vec<(ListenerProtocol, SocketAddr)> {
    let mut addresses = Vec::new();

    push_listening_addr(
        &mut addresses,
        node_id,
        receiver_urn,
        "listening_addr",
        config,
        ListenerProtocol::Tcp,
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
                    ListenerProtocol::Tcp,
                );
            }
        }
    }

    if let Some(protocol) = config.get("protocol") {
        for (key, listener_protocol) in [
            ("tcp", ListenerProtocol::Tcp),
            ("udp", ListenerProtocol::Udp),
        ] {
            if let Some(protocol_config) = protocol.get(key) {
                push_listening_addr(
                    &mut addresses,
                    node_id,
                    receiver_urn,
                    &format!("protocol.{key}.listening_addr"),
                    protocol_config,
                    listener_protocol,
                );
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
            numa_node_id: core.known_numa_node_id,
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

        for (protocol, bind_address) in
            listener_addresses(node_id.as_ref(), node_cfg.r#type.as_str(), &node_cfg.config)
        {
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

    plans.sort_by(|left, right| left.key.cmp(&right.key));
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
            policies: ResolvedPolicies::default(),
            role: ResolvedPipelineRole::Regular,
        }
    }

    /// Scenario: an OTLP receiver exposes both gRPC and HTTP listening
    /// addresses.
    /// Guarantees: listener planning emits one TCP listener-group plan for
    /// each configured OTLP bind address.
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

    /// Scenario: an OTAP receiver exposes a top-level listening address.
    /// Guarantees: listener planning recognizes the OTAP bind identity as a
    /// TCP listener group.
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

    /// Scenario: multiple listener-capable receiver nodes are discovered in a
    /// pipeline config.
    /// Guarantees: listener-group plans are returned in deterministic key
    /// order independent of config map iteration.
    #[test]
    fn listener_group_plans_are_sorted_by_key() {
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

        let snapshot = snapshot_for_pipeline(&pipeline, &placement(), 0);

        assert_eq!(snapshot.plans.len(), 2);
        assert_eq!(snapshot.plans[0].key.receiver_node_id.as_ref(), "a_otlp");
        assert_eq!(snapshot.plans[1].key.receiver_node_id.as_ref(), "z_otap");
    }

    /// Scenario: a syslog CEF receiver exposes a UDP listening address.
    /// Guarantees: listener planning preserves the UDP protocol and per-core
    /// NUMA metadata for expected members.
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

        let snapshot = snapshot_for_pipeline(&pipeline, &placement(), 0);

        assert!(snapshot.plans.is_empty());
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

        let snapshot = snapshot_for_pipeline(&pipeline, &placement, 0);

        assert_eq!(snapshot.plans[0].expected_members[0].numa_node_id, None);
        assert_eq!(snapshot.plans[0].expected_members[1].numa_node_id, None);
    }

    /// Scenario: listener planning receives partial topology where one
    /// selected core has no NUMA mapping.
    /// Guarantees: mapped cores retain their NUMA node and unmapped cores remain
    /// unknown in listener metadata.
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

        let snapshot = snapshot_for_pipeline(&pipeline, &placement, 0);

        assert_eq!(snapshot.plans[0].expected_members[0].numa_node_id, Some(0));
        assert_eq!(snapshot.plans[0].expected_members[1].numa_node_id, None);
    }
}
