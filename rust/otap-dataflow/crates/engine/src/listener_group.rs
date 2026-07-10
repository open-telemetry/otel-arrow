// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generic listener-group placement contract.
//!
//! This module is intentionally data-only. It describes the controller's planned
//! listener membership for socket-level consumers such as coordinated
//! `SO_REUSEPORT`, eBPF `sk_reuseport`, or future engine-level load balancers,
//! but it does not bind sockets or attach any selector.

use otap_df_config::{NodeId, PipelineGroupId, PipelineId};
use std::borrow::Cow;
use std::net::SocketAddr;

/// Transport protocol used by a listener group.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ListenerProtocol {
    /// TCP listener.
    Tcp,
    /// UDP socket.
    Udp,
}

impl ListenerProtocol {
    /// Returns a stable lower-case identifier suitable for telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::Udp => "udp",
        }
    }
}

/// Stable identity for one logical listener group.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ListenerGroupKey {
    /// Pipeline group id that owns this listener group.
    pub pipeline_group_id: PipelineGroupId,
    /// Pipeline id that owns this listener group.
    pub pipeline_id: PipelineId,
    /// Receiver node id that will host the listeners.
    pub receiver_node_id: NodeId,
    /// Bind address shared by every listener in the group.
    pub bind_address: SocketAddr,
    /// Transport protocol.
    pub protocol: ListenerProtocol,
    /// Optional bind-device identity reserved for future socket integrations.
    pub bind_device: Option<Cow<'static, str>>,
}

impl ListenerGroupKey {
    /// Creates a listener-group key.
    #[must_use]
    pub fn new(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        receiver_node_id: NodeId,
        bind_address: SocketAddr,
        protocol: ListenerProtocol,
    ) -> Self {
        Self {
            pipeline_group_id,
            pipeline_id,
            receiver_node_id,
            bind_address,
            protocol,
            bind_device: None,
        }
    }
}

/// One expected member of a listener group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ListenerGroupMember {
    /// Stable per-listener identifier within the controller plan.
    pub listener_id: u32,
    /// CPU core that owns the receiver pipeline thread for this listener.
    pub core_id: usize,
    /// NUMA node for `core_id`, if topology discovery knew it.
    pub numa_node_id: Option<usize>,
}

/// A planned listener group declared by the controller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListenerGroupPlan {
    /// Stable lookup key.
    pub key: ListenerGroupKey,
    /// One entry per expected listener.
    pub expected_members: Vec<ListenerGroupMember>,
}

impl ListenerGroupPlan {
    /// Returns the member assigned to `core_id`, if present.
    #[must_use]
    pub fn member_for_core(&self, core_id: usize) -> Option<&ListenerGroupMember> {
        self.expected_members
            .iter()
            .find(|member| member.core_id == core_id)
    }
}

/// Controller-resolved listener-group plans for one pipeline deployment.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ListenerGroupSnapshot {
    /// Placement generation used to build this snapshot.
    pub generation: u64,
    /// Planned listener groups for this pipeline.
    pub plans: Vec<ListenerGroupPlan>,
}

impl ListenerGroupSnapshot {
    /// Creates a snapshot from resolved plans.
    #[must_use]
    pub fn new(generation: u64, plans: Vec<ListenerGroupPlan>) -> Self {
        Self { generation, plans }
    }

    /// Creates an empty snapshot for compatibility paths.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns the matching plan for a receiver bind identity, if present.
    #[must_use]
    pub fn plan_for(
        &self,
        receiver_node_id: &str,
        bind_address: SocketAddr,
        protocol: ListenerProtocol,
    ) -> Option<&ListenerGroupPlan> {
        self.plans.iter().find(|plan| {
            plan.key.receiver_node_id.as_ref() == receiver_node_id
                && plan.key.bind_address == bind_address
                && plan.key.protocol == protocol
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_finds_plan_by_receiver_identity() {
        let addr: SocketAddr = "127.0.0.1:4317".parse().unwrap();
        let plan = ListenerGroupPlan {
            key: ListenerGroupKey::new(
                "group".into(),
                "pipeline".into(),
                "otlp".into(),
                addr,
                ListenerProtocol::Tcp,
            ),
            expected_members: vec![ListenerGroupMember {
                listener_id: 0,
                core_id: 3,
                numa_node_id: Some(1),
            }],
        };
        let snapshot = ListenerGroupSnapshot::new(9, vec![plan]);

        let found = snapshot
            .plan_for("otlp", addr, ListenerProtocol::Tcp)
            .expect("plan should be indexed by receiver identity");

        assert_eq!(found.member_for_core(3).unwrap().numa_node_id, Some(1));
        assert!(found.member_for_core(4).is_none());
        assert_eq!(snapshot.generation, 9);
    }
}
