// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP Dataflow Engine Controller
//!
//! This controller is responsible for deploying, managing, and monitoring pipeline groups
//! within the current process.
//!
//! Each pipeline configuration declares its CPU requirements through
//! `policies.resources.core_allocation`.
//! Based on this policy, the controller allocates CPU cores and spawns one dedicated
//! thread per assigned core. Threads are pinned to distinct CPU cores, following a
//! strict thread-per-core model.
//!
//! A pipeline deployed on `n` cores results in `n` worker threads. Hot data paths are
//! fully contained within each thread to maximize CPU cache locality and minimize
//! cross-thread contention. Inter-thread communication is restricted to control
//! messages and internal telemetry only.
//!
//! By default, pipelines are expected to run on dedicated CPU cores. It is possible
//! to deploy multiple pipeline configurations on the same cores, primarily for
//! consolidation, testing, or transitional deployments. This comes at the cost of
//! reduced efficiency, especially cache locality. Even in this mode, pipeline
//! instances run in independent threads and do not share mutable data structures.
//!
//! Pipelines do not perform implicit work stealing, dynamic scheduling, or automatic
//! load balancing across threads. Any form of cross-pipeline or cross-thread data
//! exchange must be explicitly modeled.
//!
//! In the future, controller-managed named channels will be introduced as the
//! recommended mechanism to implement explicit load balancing and routing schemes
//! within the engine. These channels will complement the existing SO_REUSEPORT-based
//! load balancing mechanism already supported at the receiver level on operating
//! systems that provide it.
//!
//! Pipelines can be gracefully shut down by sending control messages through their
//! control channels.
//!
//! Future work includes:
//! - TODO: Complete status and health checks for pipelines
//! - TODO: Auto-restart threads in case of panic
//! - TODO: Live pipeline updates
//! - TODO: Better resource control

use crate::error::Error;
use crate::thread_task::spawn_thread_local_task;
use core_affinity::CoreId;
use otap_df_config::engine::{
    OtelDataflowSpec, ResolvedPipelineConfig, ResolvedPipelineRole,
    SYSTEM_OBSERVABILITY_PIPELINE_ID, SYSTEM_PIPELINE_GROUP_ID,
};
use otap_df_config::node::{NodeKind, NodeUserConfig};
use otap_df_config::policy::{ChannelCapacityPolicy, CoreAllocation, TelemetryPolicy};
use otap_df_config::topic::{
    TopicAckPropagationMode, TopicBackendKind, TopicBroadcastOnLagPolicy, TopicImplSelectionPolicy,
    TopicSpec,
};
use otap_df_config::{
    DeployedPipelineKey, PipelineGroupId, PipelineId, PipelineKey, SubscriptionGroupName,
    TopicName, pipeline::PipelineConfig,
};
use otap_df_engine::PipelineFactory;
use otap_df_engine::ReceivedAtNode;
use otap_df_engine::Unwindable;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    PipelineResultMsgReceiver, PipelineResultMsgSender, RuntimeCtrlMsgReceiver,
    RuntimeCtrlMsgSender, pipeline_result_msg_channel, runtime_ctrl_msg_channel,
};
use otap_df_engine::entity_context::{
    node_entity_key, pipeline_entity_key, set_pipeline_entity_key,
};
use otap_df_engine::error::{Error as EngineError, error_summary_from};
use otap_df_engine::topic::{
    InMemoryBackend, PipelineTopicBinding, TopicBroker, TopicOptions, TopicPublishOutcomeConfig,
    TopicSet,
};
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::event::{EngineEvent, ErrorSummary, ObservedEventReporter};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry::{
    InternalTelemetrySettings, InternalTelemetrySystem, TracingSetup, otel_error, otel_info,
    otel_info_span, otel_warn, self_tracing::LogContext,
};
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use std::thread;

/// Error types and helpers for the controller module.
pub mod error;
/// Utilities to spawn async tasks on dedicated threads with graceful shutdown.
pub mod thread_task;

/// Controller for managing pipelines in a thread-per-core model.
///
/// # Thread Safety
/// This struct is designed to be used in multi-threaded contexts. Each pipeline is run on a
/// dedicated thread pinned to a CPU core.
/// Intended for use as a long-lived process controller.
pub struct Controller<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    /// The pipeline factory used to build runtime pipelines.
    pipeline_factory: &'static PipelineFactory<PData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    ParkMainThread,
    ShutdownWhenDone,
}

struct DeclaredTopics<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    broker: TopicBroker<PData>,
    global_names: HashMap<TopicName, TopicName>,
    group_names: HashMap<(PipelineGroupId, TopicName), TopicName>,
    inferred_mode_reports: Vec<InferredTopicModeReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InferredTopicMode {
    Mixed,
    BalancedOnly,
    BroadcastOnly,
}

impl InferredTopicMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::BalancedOnly => "balanced_only",
            Self::BroadcastOnly => "broadcast_only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TopicBackendCapabilities {
    supports_balanced_only: bool,
    supports_broadcast_only: bool,
    supports_mixed: bool,
    supports_broadcast_on_lag_drop_oldest: bool,
    supports_broadcast_on_lag_disconnect: bool,
    supports_ack_propagation_disabled: bool,
    supports_ack_propagation_auto: bool,
}

impl TopicBackendCapabilities {
    const fn supports_mode(self, mode: InferredTopicMode) -> bool {
        match mode {
            InferredTopicMode::BalancedOnly => self.supports_balanced_only,
            InferredTopicMode::BroadcastOnly => self.supports_broadcast_only,
            InferredTopicMode::Mixed => self.supports_mixed,
        }
    }

    const fn supports_broadcast_on_lag(self, policy: TopicBroadcastOnLagPolicy) -> bool {
        match policy {
            TopicBroadcastOnLagPolicy::DropOldest => self.supports_broadcast_on_lag_drop_oldest,
            TopicBroadcastOnLagPolicy::Disconnect => self.supports_broadcast_on_lag_disconnect,
        }
    }

    const fn supports_ack_propagation(self, policy: TopicAckPropagationMode) -> bool {
        match policy {
            TopicAckPropagationMode::Disabled => self.supports_ack_propagation_disabled,
            TopicAckPropagationMode::Auto => self.supports_ack_propagation_auto,
        }
    }
}

const fn broadcast_on_lag_policy_value(policy: TopicBroadcastOnLagPolicy) -> &'static str {
    match policy {
        TopicBroadcastOnLagPolicy::DropOldest => "drop_oldest",
        TopicBroadcastOnLagPolicy::Disconnect => "disconnect",
    }
}

const fn ack_propagation_policy_value(policy: TopicAckPropagationMode) -> &'static str {
    match policy {
        TopicAckPropagationMode::Disabled => "disabled",
        TopicAckPropagationMode::Auto => "auto",
    }
}

#[derive(Debug, Default)]
struct TopicUsageSummary {
    receiver_refs: usize,
    exporter_refs: usize,
    has_broadcast_receivers: bool,
    balanced_groups: HashSet<SubscriptionGroupName>,
    has_unknown_receiver_mode: bool,
}

#[derive(Debug)]
enum TopicReceiverMode {
    Broadcast,
    Balanced(SubscriptionGroupName),
    Unknown,
}

#[derive(Debug, Clone)]
struct InferredTopicModeReport {
    topic: TopicName,
    topology_mode: InferredTopicMode,
    selected_mode: InferredTopicMode,
    selection_policy: TopicImplSelectionPolicy,
    receiver_refs: usize,
    exporter_refs: usize,
    balanced_group_count: usize,
    has_broadcast_receivers: bool,
    has_unknown_receiver_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TopicWiringVertex {
    PipelineNode {
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        node_id: otap_df_config::NodeId,
    },
    Topic {
        declared_name: TopicName,
    },
}

impl TopicWiringVertex {
    fn label(&self) -> String {
        match self {
            Self::PipelineNode {
                pipeline_group_id,
                pipeline_id,
                node_id,
            } => format!(
                "pipeline:{}/{}/{}",
                pipeline_group_id.as_ref(),
                pipeline_id.as_ref(),
                node_id.as_ref()
            ),
            Self::Topic { declared_name } => format!("topic:{}", declared_name.as_ref()),
        }
    }
}

/// Returns the set of entity keys relevant to this context.
fn engine_context() -> LogContext {
    if let Some(node) = node_entity_key() {
        smallvec![node]
    } else if let Some(pipeline) = pipeline_entity_key() {
        smallvec![pipeline]
    } else {
        smallvec![]
    }
}

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable>
    Controller<PData>
{
    /// Creates a new controller with the given pipeline factory.
    pub const fn new(pipeline_factory: &'static PipelineFactory<PData>) -> Self {
        Self { pipeline_factory }
    }

    /// Starts the controller with the given engine configurations.
    pub fn run_forever(&self, engine_config: OtelDataflowSpec) -> Result<(), Error> {
        self.run_with_mode(engine_config, RunMode::ParkMainThread)
    }

    /// Starts the controller with the given engine configurations.
    ///
    /// Runs until pipelines are shut down, then closes telemetry/admin services.
    pub fn run_till_shutdown(&self, engine_config: OtelDataflowSpec) -> Result<(), Error> {
        self.run_with_mode(engine_config, RunMode::ShutdownWhenDone)
    }

    fn map_topic_spec_to_options(
        spec: &TopicSpec,
        inferred_mode: InferredTopicMode,
    ) -> TopicOptions {
        let balanced_capacity = spec.policies.balanced.queue_capacity.max(1);
        let broadcast_capacity = spec.policies.broadcast.queue_capacity.max(1);
        let broadcast_on_lag = spec.policies.broadcast.on_lag;
        match inferred_mode {
            InferredTopicMode::Mixed => TopicOptions::Mixed {
                balanced_capacity,
                broadcast_capacity,
                on_lag: broadcast_on_lag,
            },
            InferredTopicMode::BalancedOnly => TopicOptions::BalancedOnly {
                capacity: balanced_capacity,
            },
            InferredTopicMode::BroadcastOnly => TopicOptions::BroadcastOnly {
                capacity: broadcast_capacity,
                on_lag: broadcast_on_lag,
            },
        }
    }

    fn map_topic_spec_to_publish_outcome_config(spec: &TopicSpec) -> TopicPublishOutcomeConfig {
        TopicPublishOutcomeConfig {
            max_in_flight: spec.policies.ack_propagation.max_in_flight.max(1),
            timeout: spec.policies.ack_propagation.timeout,
        }
    }

    fn build_declared_topic_name_maps(
        config: &OtelDataflowSpec,
    ) -> Result<
        (
            HashMap<TopicName, TopicName>,
            HashMap<(PipelineGroupId, TopicName), TopicName>,
        ),
        Error,
    > {
        let mut global_names = HashMap::new();
        let mut group_names = HashMap::new();

        for topic_name in config.topics.keys() {
            let declared_name =
                Self::parse_topic_name(&format!("global::{}", topic_name.as_ref()))?;
            _ = global_names.insert(topic_name.clone(), declared_name);
        }

        for (group_id, group_cfg) in &config.groups {
            for topic_name in group_cfg.topics.keys() {
                let declared_name = Self::parse_topic_name(&format!(
                    "group::{}::{}",
                    group_id.as_ref(),
                    topic_name.as_ref()
                ))?;
                _ = group_names.insert((group_id.clone(), topic_name.clone()), declared_name);
            }
        }

        Ok((global_names, group_names))
    }

    fn resolve_declared_topic_name(
        pipeline_group_id: &PipelineGroupId,
        topic_name: &TopicName,
        global_names: &HashMap<TopicName, TopicName>,
        group_names: &HashMap<(PipelineGroupId, TopicName), TopicName>,
    ) -> Option<TopicName> {
        group_names
            .get(&(pipeline_group_id.clone(), topic_name.clone()))
            .cloned()
            .or_else(|| global_names.get(topic_name).cloned())
    }

    fn parse_topic_name_from_node_config(node_config: &NodeUserConfig) -> Option<TopicName> {
        let raw_topic = node_config.config.get("topic")?.as_str()?;
        TopicName::parse(raw_topic).ok()
    }

    fn parse_topic_receiver_mode(node_config: &NodeUserConfig) -> TopicReceiverMode {
        let Some(subscription) = node_config.config.get("subscription") else {
            return TopicReceiverMode::Broadcast;
        };
        let Some(subscription) = subscription.as_object() else {
            return TopicReceiverMode::Unknown;
        };
        let Some(mode) = subscription.get("mode").and_then(|value| value.as_str()) else {
            return TopicReceiverMode::Unknown;
        };

        match mode {
            "broadcast" => TopicReceiverMode::Broadcast,
            "balanced" => {
                let Some(raw_group) = subscription.get("group").and_then(|value| value.as_str())
                else {
                    return TopicReceiverMode::Unknown;
                };
                match SubscriptionGroupName::parse(raw_group) {
                    Ok(group) => TopicReceiverMode::Balanced(group),
                    Err(_) => TopicReceiverMode::Unknown,
                }
            }
            _ => TopicReceiverMode::Unknown,
        }
    }

    fn infer_topic_mode(summary: &TopicUsageSummary) -> InferredTopicMode {
        if summary.has_unknown_receiver_mode {
            return InferredTopicMode::Mixed;
        }
        if summary.receiver_refs == 0 {
            return InferredTopicMode::Mixed;
        }
        if summary.has_broadcast_receivers && summary.balanced_groups.is_empty() {
            return InferredTopicMode::BroadcastOnly;
        }
        if !summary.has_broadcast_receivers && summary.balanced_groups.len() == 1 {
            return InferredTopicMode::BalancedOnly;
        }
        InferredTopicMode::Mixed
    }

    fn infer_topic_modes(
        config: &OtelDataflowSpec,
        global_names: &HashMap<TopicName, TopicName>,
        group_names: &HashMap<(PipelineGroupId, TopicName), TopicName>,
    ) -> (
        HashMap<TopicName, InferredTopicMode>,
        Vec<InferredTopicModeReport>,
    ) {
        let mut usage_by_declared_topic = HashMap::<TopicName, TopicUsageSummary>::new();
        for declared_name in global_names.values().chain(group_names.values()) {
            _ = usage_by_declared_topic.insert(declared_name.clone(), TopicUsageSummary::default());
        }

        let mut visit_topic_node =
            |pipeline_group_id: &PipelineGroupId, node_config: &NodeUserConfig| {
                let topic_name = match Self::parse_topic_name_from_node_config(node_config) {
                    Some(topic_name) => topic_name,
                    None => return,
                };
                let Some(declared_topic_name) = Self::resolve_declared_topic_name(
                    pipeline_group_id,
                    &topic_name,
                    global_names,
                    group_names,
                ) else {
                    return;
                };
                let Some(summary) = usage_by_declared_topic.get_mut(&declared_topic_name) else {
                    return;
                };

                match node_config.kind() {
                    NodeKind::Receiver => {
                        summary.receiver_refs += 1;
                        match Self::parse_topic_receiver_mode(node_config) {
                            TopicReceiverMode::Broadcast => {
                                summary.has_broadcast_receivers = true;
                            }
                            TopicReceiverMode::Balanced(group) => {
                                _ = summary.balanced_groups.insert(group);
                            }
                            TopicReceiverMode::Unknown => {
                                summary.has_unknown_receiver_mode = true;
                            }
                        }
                    }
                    NodeKind::Exporter => {
                        summary.exporter_refs += 1;
                    }
                    _ => {}
                }
            };

        for (group_id, group_cfg) in &config.groups {
            for pipeline_cfg in group_cfg.pipelines.values() {
                for (_node_id, node_cfg) in pipeline_cfg.node_iter() {
                    if node_cfg.r#type.id() != "topic" {
                        continue;
                    }
                    visit_topic_node(group_id, node_cfg.as_ref());
                }
            }
        }

        if let Some(observability_pipeline) = config.engine.observability.pipeline.as_ref() {
            let system_group_id: PipelineGroupId = SYSTEM_PIPELINE_GROUP_ID.into();
            for (_node_id, node_cfg) in observability_pipeline.nodes.iter() {
                if node_cfg.r#type.id() != "topic" {
                    continue;
                }
                visit_topic_node(&system_group_id, node_cfg.as_ref());
            }
        }

        let mut inferred_modes = HashMap::with_capacity(usage_by_declared_topic.len());
        let mut inferred_mode_reports = Vec::with_capacity(usage_by_declared_topic.len());
        let mut declared_topics: Vec<_> = usage_by_declared_topic.keys().cloned().collect();
        declared_topics.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));

        for declared_topic in declared_topics {
            let summary = usage_by_declared_topic
                .get(&declared_topic)
                .expect("declared topic must have a usage summary");
            let topology_mode = Self::infer_topic_mode(summary);
            inferred_mode_reports.push(InferredTopicModeReport {
                topic: declared_topic.clone(),
                topology_mode,
                selected_mode: topology_mode,
                selection_policy: TopicImplSelectionPolicy::Auto,
                receiver_refs: summary.receiver_refs,
                exporter_refs: summary.exporter_refs,
                balanced_group_count: summary.balanced_groups.len(),
                has_broadcast_receivers: summary.has_broadcast_receivers,
                has_unknown_receiver_mode: summary.has_unknown_receiver_mode,
            });
            _ = inferred_modes.insert(declared_topic, topology_mode);
        }

        (inferred_modes, inferred_mode_reports)
    }

    fn add_topic_wiring_edge(
        adjacency: &mut HashMap<TopicWiringVertex, Vec<TopicWiringVertex>>,
        from: TopicWiringVertex,
        to: TopicWiringVertex,
    ) {
        adjacency.entry(from.clone()).or_default().push(to.clone());
        let _ = adjacency.entry(to).or_default();
    }

    fn collect_topic_wiring_edges_for_pipeline(
        adjacency: &mut HashMap<TopicWiringVertex, Vec<TopicWiringVertex>>,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
        pipeline: &PipelineConfig,
        global_names: &HashMap<TopicName, TopicName>,
        group_names: &HashMap<(PipelineGroupId, TopicName), TopicName>,
    ) {
        for connection in pipeline.connection_iter() {
            let targets = connection.to_nodes();
            for source in connection.from_sources() {
                let source_vertex = TopicWiringVertex::PipelineNode {
                    pipeline_group_id: pipeline_group_id.clone(),
                    pipeline_id: pipeline_id.clone(),
                    node_id: source.node_id().clone(),
                };
                for target in &targets {
                    let target_vertex = TopicWiringVertex::PipelineNode {
                        pipeline_group_id: pipeline_group_id.clone(),
                        pipeline_id: pipeline_id.clone(),
                        node_id: target.clone(),
                    };
                    Self::add_topic_wiring_edge(adjacency, source_vertex.clone(), target_vertex);
                }
            }
        }

        let mut topic_nodes = pipeline.node_iter().collect::<Vec<_>>();
        topic_nodes.sort_by(|(left, _), (right, _)| left.as_ref().cmp(right.as_ref()));
        for (node_id, node_config) in topic_nodes {
            if node_config.r#type.id() != "topic" {
                continue;
            }
            let Some(topic_name) = Self::parse_topic_name_from_node_config(node_config) else {
                continue;
            };
            let Some(declared_name) = Self::resolve_declared_topic_name(
                pipeline_group_id,
                &topic_name,
                global_names,
                group_names,
            ) else {
                continue;
            };
            let node_vertex = TopicWiringVertex::PipelineNode {
                pipeline_group_id: pipeline_group_id.clone(),
                pipeline_id: pipeline_id.clone(),
                node_id: node_id.clone(),
            };
            let topic_vertex = TopicWiringVertex::Topic { declared_name };
            match node_config.kind() {
                NodeKind::Exporter => {
                    Self::add_topic_wiring_edge(adjacency, node_vertex, topic_vertex);
                }
                NodeKind::Receiver => {
                    Self::add_topic_wiring_edge(adjacency, topic_vertex, node_vertex);
                }
                _ => {}
            }
        }
    }

    fn detect_topic_wiring_cycles(
        adjacency: &HashMap<TopicWiringVertex, Vec<TopicWiringVertex>>,
    ) -> Vec<Vec<TopicWiringVertex>> {
        fn visit(
            node: &TopicWiringVertex,
            adjacency: &HashMap<TopicWiringVertex, Vec<TopicWiringVertex>>,
            visiting: &mut HashSet<TopicWiringVertex>,
            visited: &mut HashSet<TopicWiringVertex>,
            current_path: &mut Vec<TopicWiringVertex>,
            cycles: &mut Vec<Vec<TopicWiringVertex>>,
        ) {
            if visited.contains(node) {
                return;
            }
            if visiting.contains(node) {
                if let Some(pos) = current_path.iter().position(|candidate| candidate == node) {
                    cycles.push(current_path[pos..].to_vec());
                }
                return;
            }

            let _ = visiting.insert(node.clone());
            current_path.push(node.clone());

            if let Some(targets) = adjacency.get(node) {
                for target in targets {
                    visit(target, adjacency, visiting, visited, current_path, cycles);
                }
            }

            let _ = visiting.remove(node);
            let _ = visited.insert(node.clone());
            let _ = current_path.pop();
        }

        let mut nodes = adjacency.keys().cloned().collect::<Vec<_>>();
        nodes.sort_by_key(TopicWiringVertex::label);

        let mut cycles = Vec::new();
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        for node in nodes {
            visit(
                &node,
                adjacency,
                &mut visiting,
                &mut visited,
                &mut current_path,
                &mut cycles,
            );
        }

        cycles
    }

    fn validate_topic_wiring_acyclic(
        config: &OtelDataflowSpec,
        global_names: &HashMap<TopicName, TopicName>,
        group_names: &HashMap<(PipelineGroupId, TopicName), TopicName>,
    ) -> Result<(), Error> {
        let mut adjacency = HashMap::<TopicWiringVertex, Vec<TopicWiringVertex>>::new();

        let mut group_ids = config.groups.keys().cloned().collect::<Vec<_>>();
        group_ids.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));
        for group_id in group_ids {
            let group_cfg = config
                .groups
                .get(&group_id)
                .expect("group collected from config must still exist");
            let mut pipeline_ids = group_cfg.pipelines.keys().cloned().collect::<Vec<_>>();
            pipeline_ids.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));
            for pipeline_id in pipeline_ids {
                let pipeline_cfg = group_cfg
                    .pipelines
                    .get(&pipeline_id)
                    .expect("pipeline collected from config must still exist");
                Self::collect_topic_wiring_edges_for_pipeline(
                    &mut adjacency,
                    &group_id,
                    &pipeline_id,
                    pipeline_cfg,
                    global_names,
                    group_names,
                );
            }
        }

        if let Some(observability_pipeline) = config.engine.observability.pipeline.as_ref() {
            let system_group_id: PipelineGroupId = SYSTEM_PIPELINE_GROUP_ID.into();
            let observability_pipeline_id: PipelineId = SYSTEM_OBSERVABILITY_PIPELINE_ID.into();
            let pipeline_cfg = observability_pipeline.clone().into_pipeline_config();
            Self::collect_topic_wiring_edges_for_pipeline(
                &mut adjacency,
                &system_group_id,
                &observability_pipeline_id,
                &pipeline_cfg,
                global_names,
                group_names,
            );
        }

        if let Some(cycle) = Self::detect_topic_wiring_cycles(&adjacency)
            .into_iter()
            .next()
        {
            let mut cycle_labels = cycle
                .iter()
                .map(TopicWiringVertex::label)
                .collect::<Vec<_>>();
            if let Some(first) = cycle.first() {
                cycle_labels.push(first.label());
            }
            return Err(Error::TopicWiringCycleDetected {
                cycle: cycle_labels,
            });
        }

        Ok(())
    }

    fn emit_topic_mode_reports(reports: &[InferredTopicModeReport]) {
        for report in reports {
            otel_info!(
                "controller.topic_mode_inferred",
                topic = report.topic.as_ref(),
                topology_mode = report.topology_mode.as_str(),
                selected_mode = report.selected_mode.as_str(),
                selection_policy = report.selection_policy.to_string(),
                receiver_refs = report.receiver_refs as u64,
                exporter_refs = report.exporter_refs as u64,
                balanced_group_count = report.balanced_group_count as u64,
                has_broadcast_receivers = report.has_broadcast_receivers,
                has_unknown_receiver_mode = report.has_unknown_receiver_mode,
                message = "Resolved topic mode from topology inference and config selection policy"
            );
        }
    }

    fn apply_topic_impl_selection_policy(
        topology_mode: InferredTopicMode,
        policy: TopicImplSelectionPolicy,
    ) -> InferredTopicMode {
        match policy {
            TopicImplSelectionPolicy::Auto => topology_mode,
            TopicImplSelectionPolicy::ForceMixed => InferredTopicMode::Mixed,
        }
    }

    fn update_topic_mode_report(
        reports: &mut [InferredTopicModeReport],
        topic: &TopicName,
        selection_policy: TopicImplSelectionPolicy,
        selected_mode: InferredTopicMode,
    ) {
        if let Some(report) = reports.iter_mut().find(|report| &report.topic == topic) {
            report.selection_policy = selection_policy;
            report.selected_mode = selected_mode;
        }
    }

    fn topic_backend_capabilities(backend: TopicBackendKind) -> Option<TopicBackendCapabilities> {
        match backend {
            TopicBackendKind::InMemory => Some(TopicBackendCapabilities {
                supports_balanced_only: true,
                supports_broadcast_only: true,
                supports_mixed: true,
                supports_broadcast_on_lag_drop_oldest: true,
                supports_broadcast_on_lag_disconnect: true,
                supports_ack_propagation_disabled: true,
                supports_ack_propagation_auto: true,
            }),
            TopicBackendKind::Quiver => None,
        }
    }

    fn validate_topic_runtime_support_with_capabilities(
        topic: &TopicName,
        backend: TopicBackendKind,
        policies: &otap_df_config::topic::TopicPolicies,
        selected_mode: InferredTopicMode,
        capabilities: TopicBackendCapabilities,
    ) -> Result<(), Error> {
        if !capabilities.supports_mode(selected_mode) {
            return Err(Error::UnsupportedTopicMode {
                topic: topic.clone(),
                backend,
                mode: selected_mode.as_str().to_owned(),
            });
        }

        if matches!(
            selected_mode,
            InferredTopicMode::BroadcastOnly | InferredTopicMode::Mixed
        ) && !capabilities.supports_broadcast_on_lag(policies.broadcast.on_lag)
        {
            return Err(Error::UnsupportedTopicPolicy {
                topic: topic.clone(),
                backend,
                policy: "broadcast.on_lag",
                value: broadcast_on_lag_policy_value(policies.broadcast.on_lag).to_owned(),
            });
        }

        if !capabilities.supports_ack_propagation(policies.ack_propagation.mode) {
            return Err(Error::UnsupportedTopicPolicy {
                topic: topic.clone(),
                backend,
                policy: "ack_propagation",
                value: ack_propagation_policy_value(policies.ack_propagation.mode).to_owned(),
            });
        }

        Ok(())
    }

    fn validate_topic_runtime_support(
        topic: &TopicName,
        spec: &TopicSpec,
        selected_mode: InferredTopicMode,
    ) -> Result<(), Error> {
        let Some(capabilities) = Self::topic_backend_capabilities(spec.backend) else {
            return Err(Error::UnsupportedTopicBackend {
                topic: topic.clone(),
                backend: spec.backend,
            });
        };
        Self::validate_topic_runtime_support_with_capabilities(
            topic,
            spec.backend,
            &spec.policies,
            selected_mode,
            capabilities,
        )
    }

    fn declare_topic(
        broker: &TopicBroker<PData>,
        name: TopicName,
        spec: &TopicSpec,
        inferred_mode: InferredTopicMode,
    ) -> Result<(), Error> {
        Self::validate_topic_runtime_support(&name, spec, inferred_mode)?;
        let opts = Self::map_topic_spec_to_options(spec, inferred_mode);
        match spec.backend {
            TopicBackendKind::InMemory => {
                _ = broker
                    .create_topic(name, opts, InMemoryBackend)
                    .map_err(|e| Error::PipelineRuntimeError {
                        source: Box::new(e),
                    })?;
                Ok(())
            }
            TopicBackendKind::Quiver => unreachable!("unsupported backend must be rejected above"),
        }
    }

    fn parse_topic_name(raw: &str) -> Result<TopicName, Error> {
        TopicName::parse(raw).map_err(|e| Error::PipelineRuntimeError {
            source: Box::new(EngineError::InternalError {
                message: format!("invalid topic name `{raw}`: {e}"),
            }),
        })
    }

    fn declare_topics(config: &OtelDataflowSpec) -> Result<DeclaredTopics<PData>, Error> {
        let broker = TopicBroker::<PData>::new();
        let (global_names, group_names) = Self::build_declared_topic_name_maps(config)?;
        Self::validate_topic_wiring_acyclic(config, &global_names, &group_names)?;
        let (inferred_modes, mut inferred_mode_reports) =
            Self::infer_topic_modes(config, &global_names, &group_names);
        let default_selection_policy = config.engine.topics.impl_selection;

        for (topic_name, spec) in &config.topics {
            let declared_name = global_names
                .get(topic_name)
                .expect("global topic declaration must resolve to a declared topic name")
                .clone();
            let topology_mode = inferred_modes
                .get(&declared_name)
                .copied()
                .unwrap_or(InferredTopicMode::Mixed);
            let selection_policy = spec.impl_selection.unwrap_or(default_selection_policy);
            let selected_mode =
                Self::apply_topic_impl_selection_policy(topology_mode, selection_policy);
            Self::update_topic_mode_report(
                &mut inferred_mode_reports,
                &declared_name,
                selection_policy,
                selected_mode,
            );
            Self::declare_topic(&broker, declared_name, spec, selected_mode)?;
        }

        for (group_id, group_cfg) in &config.groups {
            for (topic_name, spec) in &group_cfg.topics {
                let declared_name = group_names
                    .get(&(group_id.clone(), topic_name.clone()))
                    .expect("group topic declaration must resolve to a declared topic name")
                    .clone();
                let topology_mode = inferred_modes
                    .get(&declared_name)
                    .copied()
                    .unwrap_or(InferredTopicMode::Mixed);
                let selection_policy = spec.impl_selection.unwrap_or(default_selection_policy);
                let selected_mode =
                    Self::apply_topic_impl_selection_policy(topology_mode, selection_policy);
                Self::update_topic_mode_report(
                    &mut inferred_mode_reports,
                    &declared_name,
                    selection_policy,
                    selected_mode,
                );
                Self::declare_topic(&broker, declared_name, spec, selected_mode)?;
            }
        }

        Ok(DeclaredTopics {
            broker,
            global_names,
            group_names,
            inferred_mode_reports,
        })
    }

    fn build_pipeline_topic_set(
        config: &OtelDataflowSpec,
        declared: &DeclaredTopics<PData>,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
        core_id: usize,
    ) -> Result<TopicSet<PData>, Error> {
        let set_name = format!(
            "{}::{}::core-{}",
            pipeline_group_id.as_ref(),
            pipeline_id.as_ref(),
            core_id
        );
        let set = TopicSet::new(set_name);

        for (global_topic_name, topic_spec) in &config.topics {
            if let Some(declared_name) = declared.global_names.get(global_topic_name) {
                let handle = declared
                    .broker
                    .get_topic_required(declared_name)
                    .map_err(|e| Error::PipelineRuntimeError {
                        source: Box::new(e),
                    })?;
                let handle = handle.with_default_publish_outcome_config(
                    Self::map_topic_spec_to_publish_outcome_config(topic_spec),
                );
                let binding = PipelineTopicBinding::from(handle)
                    .with_default_queue_on_full(topic_spec.policies.balanced.on_full.clone())
                    .with_default_ack_propagation_mode(topic_spec.policies.ack_propagation.mode);
                _ = set.insert(global_topic_name.clone(), binding);
            }
        }

        if let Some(group_cfg) = config.groups.get(pipeline_group_id) {
            for (group_topic_name, topic_spec) in &group_cfg.topics {
                if let Some(declared_name) = declared
                    .group_names
                    .get(&(pipeline_group_id.clone(), group_topic_name.clone()))
                {
                    let handle =
                        declared
                            .broker
                            .get_topic_required(declared_name)
                            .map_err(|e| Error::PipelineRuntimeError {
                                source: Box::new(e),
                            })?;
                    let handle = handle.with_default_publish_outcome_config(
                        Self::map_topic_spec_to_publish_outcome_config(topic_spec),
                    );
                    let binding = PipelineTopicBinding::from(handle)
                        .with_default_queue_on_full(topic_spec.policies.balanced.on_full.clone())
                        .with_default_ack_propagation_mode(
                            topic_spec.policies.ack_propagation.mode,
                        );
                    // Group-local declarations override globals with the same local name.
                    _ = set.insert(group_topic_name.clone(), binding);
                }
            }
        }

        Ok(set)
    }

    fn run_with_mode(
        &self,
        engine_config: OtelDataflowSpec,
        run_mode: RunMode,
    ) -> Result<(), Error> {
        let num_pipeline_groups = engine_config.groups.len();
        let resolved_config = engine_config.resolve();
        let (engine, pipelines, observability_pipeline) = resolved_config.into_parts();
        let num_pipelines = pipelines.len();
        let admin_settings = engine.http_admin.clone().unwrap_or_default();
        // Initialize metrics system and observed event store.
        // ToDo A hierarchical metrics system will be implemented to better support hardware with multiple NUMA nodes.
        let telemetry_config = &engine.telemetry;
        otel_info!(
            "controller.start",
            num_pipeline_groups = num_pipeline_groups,
            num_pipelines = num_pipelines
        );

        // Create the shared telemetry registry first - it will be used by both
        // the observed state store and the internal telemetry system.
        let telemetry_registry = TelemetryRegistryHandle::new();

        // Create the observed state store for the telemetry system.
        let obs_state_store =
            ObservedStateStore::new(&engine.observed_state, telemetry_registry.clone());
        let obs_state_handle = obs_state_store.handle();
        let engine_evt_reporter =
            obs_state_store.engine_reporter(engine.observed_state.engine_events);
        let console_async_reporter = telemetry_config
            .logs
            .providers
            .uses_console_async_provider()
            .then(|| obs_state_store.reporter(engine.observed_state.logging_events));

        // Create the telemetry system. The console_async_reporter is passed when any
        // providers use ConsoleAsync. The its_logs_receiver is passed when any
        // providers use the ITS mode.
        let telemetry_system = InternalTelemetrySystem::new(
            telemetry_config,
            telemetry_registry.clone(),
            console_async_reporter,
            engine_context,
        )?;

        let admin_tracing_setup = telemetry_system.admin_tracing_setup();
        let internal_tracing_setup = telemetry_system.internal_tracing_setup();

        let metrics_dispatcher = telemetry_system.dispatcher();
        let metrics_reporter = telemetry_system.reporter();
        let controller_ctx = ControllerContext::new(telemetry_system.registry());
        // Declare all topics up front before any pipeline thread starts.
        let declared_topics = Self::declare_topics(&engine_config)?;

        for pipeline_entry in &pipelines {
            let pipeline_key = PipelineKey::new(
                pipeline_entry.pipeline_group_id.clone(),
                pipeline_entry.pipeline_id.clone(),
            );
            obs_state_store.register_pipeline_health_policy(
                pipeline_key,
                pipeline_entry.policies.health.clone(),
            );
        }

        let pipeline_count = pipelines.len();
        let all_cores =
            core_affinity::get_core_ids().ok_or_else(|| Error::CoreDetectionUnavailable)?;
        let its_core = *all_cores.first().expect("a cpu core");
        let its_key = Self::internal_pipeline_key(its_core);
        if let Some(pipeline) = observability_pipeline.as_ref() {
            obs_state_store.register_pipeline_health_policy(
                PipelineKey::new(
                    its_key.pipeline_group_id.clone(),
                    its_key.pipeline_id.clone(),
                ),
                pipeline.policies.health.clone(),
            );
        }
        let available_core_ids = if pipeline_count == 0 {
            Vec::new()
        } else {
            all_cores
        };
        let planned_core_assignments =
            Self::preflight_pipeline_core_allocations(&pipelines, &available_core_ids)?;

        let internal_pipeline_handle = Self::spawn_internal_pipeline_if_configured(
            its_key.clone(),
            its_core,
            observability_pipeline,
            &engine_config,
            &declared_topics,
            &telemetry_system,
            self.pipeline_factory,
            &controller_ctx,
            &engine_evt_reporter,
            &metrics_reporter,
            internal_tracing_setup,
        )?;

        // TODO: This should be validated somewhere, that engine observability pipeline is
        // defined when ITS is requested. Possibly we could fill in a default.
        let has_internal_pipeline = internal_pipeline_handle.is_some();
        match (
            has_internal_pipeline,
            telemetry_config.logs.providers.uses_its_provider(),
        ) {
            (false, true) => {
                otel_warn!(
                    "ITS provider requested yet engine.observability.pipeline is not defined"
                )
            }
            (true, false) => {
                otel_warn!(
                    "engine.observability.pipeline is defined yet ITS provider is not requested"
                )
            }
            _ => {}
        };

        // Initialize the global subscriber AFTER the internal pipeline has signaled
        // successful startup. This ensures the channel receiver is being consumed
        // before we start sending logs.
        telemetry_system.init_global_subscriber();
        Self::emit_topic_mode_reports(&declared_topics.inferred_mode_reports);

        let internal_collector = telemetry_system.collector();
        let metrics_agg_handle = spawn_thread_local_task(
            "metrics-aggregator",
            admin_tracing_setup.clone(),
            move |cancellation_token| internal_collector.run(cancellation_token),
        )?;

        // Start the metrics dispatcher only if there are metric readers configured.
        let metrics_dispatcher_handle = if telemetry_config.metrics.has_readers() {
            Some(spawn_thread_local_task(
                "metrics-dispatcher",
                admin_tracing_setup.clone(),
                move |cancellation_token| metrics_dispatcher.run_dispatch_loop(cancellation_token),
            )?)
        } else {
            None
        };

        // Start the observed state store background task
        let obs_state_join_handle = spawn_thread_local_task(
            "observed-state-store",
            admin_tracing_setup.clone(),
            move |cancellation_token| obs_state_store.run(cancellation_token),
        )?;

        // Start the engine-wide metrics collection task.
        // This samples engine-level metrics (e.g. RSS) on a fixed interval and
        // reports them once per engine, rather than duplicating across pipelines.
        let engine_entity_key = controller_ctx.register_engine_entity();
        let engine_registry = controller_ctx.telemetry_registry();
        let engine_reporter = metrics_reporter.clone();
        let engine_metrics_handle = spawn_thread_local_task(
            "engine-metrics",
            admin_tracing_setup.clone(),
            move |cancellation_token| async move {
                use otap_df_engine::engine_metrics::EngineMetricsMonitor;
                use std::time::Duration;
                use tokio::time::{MissedTickBehavior, interval};

                // TODO: Make this interval configurable via engine config.
                const ENGINE_METRICS_INTERVAL: Duration = Duration::from_secs(5);

                let mut monitor =
                    EngineMetricsMonitor::new(engine_registry, engine_entity_key, engine_reporter);

                let mut ticker = interval(ENGINE_METRICS_INTERVAL);
                ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

                loop {
                    tokio::select! {
                        _ = cancellation_token.cancelled() => {
                            return Ok::<(), otap_df_telemetry::error::Error>(());
                        }
                        _ = ticker.tick() => {
                            monitor.update();
                            if let Err(err) = monitor.report() {
                                otel_warn!(
                                    "engine.metrics.reporting.fail",
                                    error = err.to_string()
                                );
                            }
                        }
                    }
                }
            },
        )?;

        let mut threads = Vec::new();
        let mut ctrl_msg_senders = Vec::new();

        // TODO: We do not have proper thread::current().id assignment.
        let mut next_thread_id: usize = 1;
        let its_thread_id: usize = 0;

        // Add internal pipeline to threads list if present
        if let Some((thread_name, handle)) = internal_pipeline_handle {
            threads.push((thread_name, its_thread_id, its_key, handle));
        }

        for (pipeline_entry, requested_cores) in pipelines.into_iter().zip(planned_core_assignments)
        {
            let core_allocation = pipeline_entry
                .policies
                .effective_resources()
                .core_allocation
                .to_string();
            let channel_capacity_policy = pipeline_entry.policies.channel_capacity;
            let telemetry_policy = pipeline_entry.policies.telemetry;
            let pipeline_group_id = pipeline_entry.pipeline_group_id;
            let pipeline_id = pipeline_entry.pipeline_id;
            let pipeline = pipeline_entry.pipeline;

            let num_cores = requested_cores.len();
            otel_info!(
                "pipeline.core_allocation",
                pipeline_group_id = pipeline_group_id.as_ref(),
                pipeline_id = pipeline_id.as_ref(),
                num_cores = num_cores,
                core_allocation = core_allocation
            );
            for core_id in requested_cores {
                let pipeline_key = DeployedPipelineKey {
                    pipeline_group_id: pipeline_group_id.clone(),
                    pipeline_id: pipeline_id.clone(),
                    core_id: core_id.id,
                };
                let (runtime_ctrl_msg_tx, runtime_ctrl_msg_rx) =
                    runtime_ctrl_msg_channel(channel_capacity_policy.control.runtime);
                let (pipeline_result_msg_tx, pipeline_result_msg_rx) =
                    pipeline_result_msg_channel(channel_capacity_policy.control.results);
                ctrl_msg_senders.push(runtime_ctrl_msg_tx.clone());

                let pipeline_config = pipeline.clone();
                let pipeline_factory = self.pipeline_factory;
                let thread_id = next_thread_id;
                next_thread_id += 1;
                let mut pipeline_handle = controller_ctx.pipeline_context_with(
                    pipeline_group_id.clone(),
                    pipeline_id.clone(),
                    core_id.id,
                    num_cores,
                    thread_id,
                );
                let topic_set = Self::build_pipeline_topic_set(
                    &engine_config,
                    &declared_topics,
                    &pipeline_group_id,
                    &pipeline_id,
                    core_id.id,
                )?;
                pipeline_handle.set_topic_set(topic_set);
                let metrics_reporter = metrics_reporter.clone();

                let thread_name = format!(
                    "pipeline-{}-{}-core-{}",
                    pipeline_group_id.as_ref(),
                    pipeline_id.as_ref(),
                    core_id.id
                );

                let run_key = pipeline_key.clone();
                let engine_tracing_setup = telemetry_system.engine_tracing_setup();
                let engine_evt_reporter = engine_evt_reporter.clone();
                let effective_channel_capacity_policy = channel_capacity_policy.clone();
                let effective_telemetry_policy = telemetry_policy.clone();
                let handle = thread::Builder::new()
                    .name(thread_name.clone())
                    .spawn(move || {
                        Self::run_pipeline_thread(
                            run_key,
                            core_id,
                            pipeline_config,
                            effective_channel_capacity_policy,
                            effective_telemetry_policy,
                            pipeline_factory,
                            pipeline_handle,
                            engine_evt_reporter,
                            metrics_reporter,
                            runtime_ctrl_msg_tx,
                            runtime_ctrl_msg_rx,
                            pipeline_result_msg_tx,
                            pipeline_result_msg_rx,
                            engine_tracing_setup,
                            None,
                        )
                    })
                    .map_err(|e| Error::ThreadSpawnError {
                        thread_name: thread_name.clone(),
                        source: e,
                    })?;

                threads.push((thread_name, thread_id, pipeline_key, handle));
            }
        }

        // Drop the original metrics sender so only pipeline threads hold references
        drop(metrics_reporter);

        // Start the admin HTTP server
        let admin_server_handle = spawn_thread_local_task(
            "http-admin",
            admin_tracing_setup,
            move |cancellation_token| {
                // Convert the concrete senders to trait objects for the admin crate
                let admin_senders: Vec<Arc<dyn otap_df_engine::control::PipelineAdminSender>> =
                    ctrl_msg_senders
                        .into_iter()
                        .map(|sender| {
                            Arc::new(sender)
                                as Arc<dyn otap_df_engine::control::PipelineAdminSender>
                        })
                        .collect();

                otap_df_admin::run(
                    admin_settings,
                    obs_state_handle,
                    admin_senders,
                    telemetry_registry,
                    cancellation_token,
                )
            },
        )?;

        // Wait for all pipeline threads to finish and collect their results
        let mut results: Vec<Result<(), Error>> = Vec::with_capacity(threads.len());
        for (thread_name, thread_id, pipeline_key, handle) in threads {
            match handle.join() {
                Ok(Ok(_)) => {
                    engine_evt_reporter.report(EngineEvent::drained(pipeline_key, None));
                }
                Ok(Err(e)) => {
                    let err_summary: ErrorSummary = error_summary_from_gen(&e);
                    engine_evt_reporter.report(EngineEvent::pipeline_runtime_error(
                        pipeline_key.clone(),
                        "Pipeline encountered a runtime error.",
                        err_summary,
                    ));
                    results.push(Err(e));
                }
                Err(e) => {
                    let err_summary = ErrorSummary::Pipeline {
                        error_kind: "panic".into(),
                        message: "The pipeline panicked during execution.".into(),
                        source: Some(format!("{e:?}")),
                    };
                    engine_evt_reporter.report(EngineEvent::pipeline_runtime_error(
                        pipeline_key.clone(),
                        "The pipeline panicked during execution.",
                        err_summary,
                    ));
                    // Thread join failed, handle the error
                    let core_id = pipeline_key.core_id;
                    return Err(Error::ThreadPanic {
                        thread_name,
                        thread_id,
                        core_id,
                        panic_message: format!("{e:?}"),
                    });
                }
            }
        }

        // Check if any pipeline threads returned an error
        if let Some(err) = results.into_iter().find_map(Result::err) {
            return Err(err);
        }

        // In standard engine mode we keep the main thread parked after startup.
        if run_mode == RunMode::ParkMainThread {
            thread::park();
        }

        // All pipelines have finished; shut down the admin HTTP server and metric aggregator gracefully.
        engine_metrics_handle.shutdown_and_join()?;
        admin_server_handle.shutdown_and_join()?;
        metrics_agg_handle.shutdown_and_join()?;
        if let Some(handle) = metrics_dispatcher_handle {
            handle.shutdown_and_join()?;
        }
        obs_state_join_handle.shutdown_and_join()?;
        telemetry_system.shutdown_otel()?;

        Ok(())
    }

    /// Selects which CPU cores to use based on the given allocation.
    fn select_cores_for_allocation(
        mut available_core_ids: Vec<CoreId>,
        core_allocation: &CoreAllocation,
    ) -> Result<Vec<CoreId>, Error> {
        available_core_ids.sort_by_key(|c| c.id);

        let max_core_id = available_core_ids.iter().map(|c| c.id).max().unwrap_or(0);
        let num_cores = available_core_ids.len();

        match core_allocation {
            CoreAllocation::AllCores => Ok(available_core_ids),
            CoreAllocation::CoreCount { count } => {
                if *count == 0 {
                    Ok(available_core_ids)
                } else if *count > num_cores {
                    Err(Error::InvalidCoreAllocation {
                        alloc: core_allocation.clone(),
                        message: format!(
                            "Requested {} cores but only {} cores available on this system",
                            count, num_cores
                        ),
                        available: available_core_ids.iter().map(|c| c.id).collect(),
                    })
                } else {
                    Ok(available_core_ids.into_iter().take(*count).collect())
                }
            }
            CoreAllocation::CoreSet { set } => {
                // Validate all ranges first
                for r in set.iter() {
                    if r.start > r.end {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: core_allocation.clone(),
                            message: format!(
                                "Invalid core range: start ({}) is greater than end ({})",
                                r.start, r.end
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                    if r.start > max_core_id {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: core_allocation.clone(),
                            message: format!(
                                "Core ID {} exceeds available cores (system has cores 0-{})",
                                r.start, max_core_id
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                    if r.end > max_core_id {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: core_allocation.clone(),
                            message: format!(
                                "Core ID {} exceeds available cores (system has cores 0-{})",
                                r.end, max_core_id
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                }

                // Check for overlapping ranges
                for (i, r1) in set.iter().enumerate() {
                    for r2 in set.iter().skip(i + 1) {
                        // Two ranges overlap if they share any common cores
                        if r1.start <= r2.end && r2.start <= r1.end {
                            let overlap_start = r1.start.max(r2.start);
                            let overlap_end = r1.end.min(r2.end);
                            return Err(Error::InvalidCoreAllocation {
                                alloc: core_allocation.clone(),
                                message: format!(
                                    "Core ranges overlap: {}-{} and {}-{} share cores {}-{}",
                                    r1.start, r1.end, r2.start, r2.end, overlap_start, overlap_end
                                ),
                                available: available_core_ids.iter().map(|c| c.id).collect(),
                            });
                        }
                    }
                }

                // Filter cores in range
                let selected: Vec<_> = available_core_ids
                    .into_iter()
                    // Naively check if each interval contains the point
                    // This problem is known as the "Interval Stabbing Problem"
                    // and has more efficient but more complex solutions
                    .filter(|c| set.iter().any(|r| r.start <= c.id && c.id <= r.end))
                    .collect();

                if selected.is_empty() {
                    return Err(Error::InvalidCoreAllocation {
                        alloc: core_allocation.clone(),
                        message: "No available cores in the specified ranges".to_owned(),
                        available: core_affinity::get_core_ids()
                            .unwrap_or_default()
                            .iter()
                            .map(|c| c.id)
                            .collect(),
                    });
                }

                Ok(selected)
            }
        }
    }

    /// Pre-resolves core assignments for all regular pipelines.
    ///
    /// This validates the full pipeline set before any pipeline thread is spawned.
    fn preflight_pipeline_core_allocations(
        pipelines: &[ResolvedPipelineConfig],
        available_core_ids: &[CoreId],
    ) -> Result<Vec<Vec<CoreId>>, Error> {
        pipelines
            .iter()
            .map(|pipeline_entry| {
                Self::select_cores_for_allocation(
                    available_core_ids.to_vec(),
                    &pipeline_entry
                        .policies
                        .effective_resources()
                        .core_allocation,
                )
            })
            .collect()
    }

    fn internal_pipeline_key(core_id: CoreId) -> DeployedPipelineKey {
        DeployedPipelineKey {
            pipeline_group_id: SYSTEM_PIPELINE_GROUP_ID.into(),
            pipeline_id: SYSTEM_OBSERVABILITY_PIPELINE_ID.into(),
            core_id: core_id.id,
        }
    }

    /// Spawns the internal telemetry pipeline if engine observability config provides one.
    ///
    /// Returns the thread handle if an internal pipeline was spawned
    /// and waits for it to start, or None.
    #[allow(clippy::too_many_arguments)]
    fn spawn_internal_pipeline_if_configured(
        its_key: DeployedPipelineKey,
        its_core: CoreId,
        observability_pipeline: Option<ResolvedPipelineConfig>,
        config: &OtelDataflowSpec,
        declared_topics: &DeclaredTopics<PData>,
        telemetry_system: &InternalTelemetrySystem,
        pipeline_factory: &'static PipelineFactory<PData>,
        controller_ctx: &ControllerContext,
        engine_evt_reporter: &ObservedEventReporter,
        metrics_reporter: &MetricsReporter,
        tracing_setup: TracingSetup,
    ) -> Result<Option<(String, thread::JoinHandle<Result<Vec<()>, Error>>)>, Error> {
        let (internal_config, channel_capacity_policy, telemetry_policy): (
            PipelineConfig,
            ChannelCapacityPolicy,
            TelemetryPolicy,
        ) = match observability_pipeline {
            Some(config) if config.role == ResolvedPipelineRole::ObservabilityInternal => {
                let channel_capacity_policy = config.policies.channel_capacity;
                let telemetry_policy = config.policies.telemetry;
                (config.pipeline, channel_capacity_policy, telemetry_policy)
            }
            Some(_) => {
                // Note: This path is internal-only and should be filtered by caller.
                return Ok(None);
            }
            _ => {
                // Note: Inconsistent configurations are checked elsewhere.
                // This method is "_if_configured()" for lifetime reasons,
                // so a silent return.
                return Ok(None);
            }
        };

        let its_settings = match telemetry_system.internal_telemetry_settings() {
            None => {
                // Note: An inconsistency warning will be logged by the
                // calling function.
                return Ok(None);
            }
            Some(its_settings) => its_settings,
        };

        let mut internal_pipeline_ctx = controller_ctx.pipeline_context_with(
            its_key.pipeline_group_id.clone(),
            its_key.pipeline_id.clone(),
            its_key.core_id,
            1, // Internal telemetry pipeline runs on a single core
            0, // TODO: we do not have a thread_id
        );
        let topic_set = Self::build_pipeline_topic_set(
            config,
            declared_topics,
            &its_key.pipeline_group_id,
            &its_key.pipeline_id,
            its_key.core_id,
        )?;
        internal_pipeline_ctx.set_topic_set(topic_set);

        // Create control message channel for internal pipeline
        let (internal_ctrl_tx, internal_ctrl_rx) =
            runtime_ctrl_msg_channel(channel_capacity_policy.control.runtime);
        let (internal_return_tx, internal_return_rx) =
            pipeline_result_msg_channel(channel_capacity_policy.control.results);

        // Create a channel to signal startup success/failure
        let (startup_tx, startup_rx) = std_mpsc::sync_channel::<Result<(), EngineError>>(1);

        let thread_name = "internal-pipeline".to_string();
        let internal_evt_reporter = engine_evt_reporter.clone();
        let internal_metrics_reporter = metrics_reporter.clone();
        let internal_channel_capacity_policy = channel_capacity_policy;
        let internal_telemetry_policy = telemetry_policy;

        let handle = thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || {
                Self::run_pipeline_thread(
                    its_key,
                    its_core,
                    internal_config,
                    internal_channel_capacity_policy,
                    internal_telemetry_policy,
                    pipeline_factory,
                    internal_pipeline_ctx,
                    internal_evt_reporter,
                    internal_metrics_reporter,
                    internal_ctrl_tx,
                    internal_ctrl_rx,
                    internal_return_tx,
                    internal_return_rx,
                    tracing_setup,
                    Some((its_settings, startup_tx)),
                )
            })
            .map_err(|e| Error::ThreadSpawnError {
                thread_name: thread_name.clone(),
                source: e,
            })?;

        // Wait for the internal pipeline to signal successful startup
        match startup_rx.recv() {
            Ok(Ok(())) => {
                otel_info!(
                    "internal_pipeline.started",
                    message = "Internal telemetry pipeline started successfully"
                );
            }
            Ok(Err(e)) => {
                // Internal pipeline failed to build - propagate the error
                return Err(Error::PipelineRuntimeError {
                    source: Box::new(e),
                });
            }
            Err(err) => {
                // Channel closed unexpectedly - thread may have panicked
                return Err(Error::PipelineRuntimeError {
                    source: Box::new(err),
                });
            }
        }

        Ok(Some((thread_name, handle)))
    }

    /// Runs a single pipeline in the current thread.
    fn run_pipeline_thread(
        pipeline_key: DeployedPipelineKey,
        core_id: CoreId,
        pipeline_config: PipelineConfig,
        channel_capacity_policy: ChannelCapacityPolicy,
        telemetry_policy: TelemetryPolicy,
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_context: PipelineContext,
        obs_evt_reporter: ObservedEventReporter,
        metrics_reporter: MetricsReporter,
        runtime_ctrl_msg_tx: RuntimeCtrlMsgSender<PData>,
        runtime_ctrl_msg_rx: RuntimeCtrlMsgReceiver<PData>,
        pipeline_result_msg_tx: PipelineResultMsgSender<PData>,
        pipeline_result_msg_rx: PipelineResultMsgReceiver<PData>,
        tracing_setup: TracingSetup,
        internal_telemetry: Option<(
            InternalTelemetrySettings,
            std_mpsc::SyncSender<Result<(), EngineError>>,
        )>,
    ) -> Result<Vec<()>, Error> {
        // Pin thread to specific core. As much as possible, we pin
        // before allocating memory.
        if !core_affinity::set_for_current(core_id) {
            // Continue execution even if pinning fails.
            // This is acceptable because the OS will still schedule the thread, but performance may be less predictable.
            otel_warn!(
                "core_affinity.set_failed",
                message = "Failed to set core affinity for pipeline thread. Performance may be less predictable."
            );
        }

        // Run the pipeline with thread-local tracing subscriber active.
        tracing_setup.with_subscriber(|| {
            // Create a tracing span for this pipeline thread
            // so that all logs within this scope include pipeline context.
            let span = otel_info_span!("pipeline_thread", core.id = core_id.id);
            let _guard = span.enter();

            // The controller creates a pipeline instance into a dedicated thread. The corresponding
            // entity is registered here for proper context tracking and set into thread-local storage
            // in order to be accessible by all components within this thread.
            let pipeline_entity_key = pipeline_context.register_pipeline_entity();
            let _pipeline_entity_guard =
                set_pipeline_entity_key(pipeline_context.metrics_registry(), pipeline_entity_key);

            obs_evt_reporter.report(EngineEvent::admitted(
                pipeline_key.clone(),
                Some("Pipeline admission successful.".to_owned()),
            ));

            // Build the runtime pipeline from the configuration
            let its_settings = internal_telemetry.as_ref().map(|(s, _)| s).cloned();
            let runtime_pipeline = pipeline_factory
                .build(
                    pipeline_context.clone(),
                    pipeline_config.clone(),
                    channel_capacity_policy,
                    telemetry_policy,
                    its_settings,
                )
                .map_err(|e| {
                    if let Some((_, startup_tx)) = internal_telemetry.as_ref() {
                        let _ = startup_tx.send(Err(EngineError::InternalError {
                            message: e.to_string(),
                        }));
                    }
                    otel_error!(
                        "controller.pipeline_build_failed",
                        pipeline_group_id = pipeline_key.pipeline_group_id.as_ref(),
                        pipeline_id = pipeline_key.pipeline_id.as_ref(),
                        core_id = core_id.id,
                        error = %e,
                        message = "Failed to build runtime pipeline from configuration"
                    );
                    Error::PipelineRuntimeError {
                        source: Box::new(e),
                    }
                })?;

            obs_evt_reporter.report(EngineEvent::ready(
                pipeline_key.clone(),
                Some("Pipeline initialization successful.".to_owned()),
            ));

            if let Some((_, startup_tx)) = internal_telemetry.as_ref() {
                let _ = startup_tx.send(Ok(()));
            }

            // Start the pipeline (this will use the current thread's Tokio runtime)
            runtime_pipeline
                .run_forever(
                    pipeline_key,
                    pipeline_context,
                    obs_evt_reporter,
                    metrics_reporter,
                    runtime_ctrl_msg_tx,
                    runtime_ctrl_msg_rx,
                    pipeline_result_msg_tx,
                    pipeline_result_msg_rx,
                )
                .map_err(|e| {
                    otel_error!(
                        "controller.pipeline_runtime_failed",
                        core_id = core_id.id,
                        error = %e,
                        message = "Pipeline terminated with a runtime error"
                    );
                    Error::PipelineRuntimeError {
                        source: Box::new(e),
                    }
                })
        })
    }
}

fn error_summary_from_gen(error: &Error) -> ErrorSummary {
    match error {
        Error::PipelineRuntimeError { source } => {
            if let Some(engine_error) = source.downcast_ref::<EngineError>() {
                error_summary_from(engine_error)
            } else {
                ErrorSummary::Pipeline {
                    error_kind: "runtime".into(),
                    message: source.to_string(),
                    source: None,
                }
            }
        }
        _ => ErrorSummary::Pipeline {
            error_kind: "runtime".into(),
            message: error.to_string(),
            source: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::engine::{ResolvedPipelineConfig, ResolvedPipelineRole};
    use otap_df_config::policy::{CoreRange, Policies, ResourcesPolicy};
    use otap_df_config::topic::{TopicAckPropagationMode, TopicBroadcastOnLagPolicy};

    fn available_core_ids() -> Vec<CoreId> {
        vec![
            CoreId { id: 0 },
            CoreId { id: 1 },
            CoreId { id: 2 },
            CoreId { id: 3 },
            CoreId { id: 4 },
            CoreId { id: 5 },
            CoreId { id: 6 },
            CoreId { id: 7 },
        ]
    }

    fn to_ids(v: &[CoreId]) -> Vec<usize> {
        v.iter().map(|c| c.id).collect()
    }

    fn minimal_pipeline_config() -> PipelineConfig {
        PipelineConfig::from_yaml(
            "g".into(),
            "p".into(),
            r#"
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
        )
        .expect("minimal test pipeline config should parse")
    }

    fn resolved_pipeline_with_core_allocation(
        pipeline_group_id: &str,
        pipeline_id: &str,
        core_allocation: CoreAllocation,
    ) -> ResolvedPipelineConfig {
        let policies = Policies {
            resources: Some(ResourcesPolicy { core_allocation }),
            ..Default::default()
        };
        ResolvedPipelineConfig {
            pipeline_group_id: pipeline_group_id.to_string().into(),
            pipeline_id: pipeline_id.to_string().into(),
            pipeline: minimal_pipeline_config(),
            policies,
            role: ResolvedPipelineRole::Regular,
        }
    }

    fn global_topic_handle(
        declared: &DeclaredTopics<()>,
        topic_name: &str,
    ) -> otap_df_engine::topic::TopicHandle<()> {
        let declared_name = declared
            .global_names
            .get(topic_name)
            .expect("global topic must be declared");
        declared
            .broker
            .get_topic_required(declared_name)
            .expect("declared topic must exist in broker")
    }

    fn group_topic_handle(
        declared: &DeclaredTopics<()>,
        group_id: &str,
        topic_name: &str,
    ) -> otap_df_engine::topic::TopicHandle<()> {
        let key = (
            PipelineGroupId::from(group_id.to_owned()),
            TopicName::parse(topic_name).expect("topic name must parse"),
        );
        let declared_name = declared
            .group_names
            .get(&key)
            .expect("group topic must be declared");
        declared
            .broker
            .get_topic_required(declared_name)
            .expect("declared topic must exist in broker")
    }

    #[test]
    fn select_all_cores_by_default() {
        let core_allocation = CoreAllocation::AllCores;
        let available_core_ids = available_core_ids();
        let expected_core_ids = available_core_ids.clone();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), to_ids(&expected_core_ids));
    }

    #[test]
    fn select_limited_by_num_cores() {
        let core_allocation = CoreAllocation::CoreCount { count: 4 };
        let available_core_ids = available_core_ids();
        let result = Controller::<()>::select_cores_for_allocation(
            available_core_ids.clone(),
            &core_allocation,
        )
        .unwrap();
        assert_eq!(result.len(), 4);
        let expected_ids: Vec<usize> = available_core_ids
            .into_iter()
            .take(4)
            .map(|c| c.id)
            .collect();
        assert_eq!(to_ids(&result), expected_ids);
    }

    #[test]
    fn select_with_valid_single_core_range() {
        let available_core_ids = available_core_ids();
        let first_id = available_core_ids[0].id;
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange {
                start: first_id,
                end: first_id,
            }],
        };
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), vec![first_id]);
    }

    #[test]
    fn select_with_valid_multi_core_range() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 5 },
                CoreRange { start: 6, end: 6 },
            ],
        };
        let available_core_ids = available_core_ids();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), vec![2, 3, 4, 5, 6]);
    }

    #[test]
    fn select_with_inverted_range_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange { start: 2, end: 1 }],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, .. } => {
                assert_eq!(alloc, core_allocation);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_out_of_bounds_range_errors() {
        let start = 100;
        let end = 110;
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange { start, end }],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, .. } => {
                assert_eq!(alloc, core_allocation);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_zero_count_uses_all_cores() {
        let core_allocation = CoreAllocation::CoreCount { count: 0 };
        let available_core_ids = available_core_ids();
        let expected_core_ids = available_core_ids.clone();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), to_ids(&expected_core_ids));
    }

    #[test]
    fn select_with_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 5 },
                CoreRange { start: 4, end: 7 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_fully_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 6 },
                CoreRange { start: 3, end: 5 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_identical_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 3, end: 5 },
                CoreRange { start: 3, end: 5 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_adjacent_ranges_succeeds() {
        // Adjacent but non-overlapping ranges should work
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 3 },
                CoreRange { start: 4, end: 5 },
            ],
        };
        let available_core_ids = available_core_ids();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), vec![2, 3, 4, 5]);
    }

    #[test]
    fn select_with_multiple_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 1, end: 3 },
                CoreRange { start: 2, end: 4 },
                CoreRange { start: 5, end: 6 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn preflight_fails_fast_when_later_pipeline_allocation_is_invalid() {
        let pipelines = vec![
            resolved_pipeline_with_core_allocation(
                "g1",
                "p1",
                CoreAllocation::CoreCount { count: 2 },
            ),
            resolved_pipeline_with_core_allocation(
                "g1",
                "p2",
                CoreAllocation::CoreSet {
                    set: vec![CoreRange {
                        start: 999,
                        end: 999,
                    }],
                },
            ),
        ];

        let err = Controller::<()>::preflight_pipeline_core_allocations(
            &pipelines,
            &available_core_ids(),
        )
        .expect_err("preflight should fail");
        match err {
            Error::InvalidCoreAllocation { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn preflight_succeeds_and_allows_cross_pipeline_core_overlap() {
        let pipelines = vec![
            resolved_pipeline_with_core_allocation(
                "g1",
                "p1",
                CoreAllocation::CoreSet {
                    set: vec![CoreRange { start: 1, end: 2 }],
                },
            ),
            resolved_pipeline_with_core_allocation(
                "g1",
                "p2",
                CoreAllocation::CoreSet {
                    set: vec![CoreRange { start: 2, end: 3 }],
                },
            ),
        ];

        let assignments = Controller::<()>::preflight_pipeline_core_allocations(
            &pipelines,
            &available_core_ids(),
        )
        .expect("preflight should succeed");

        assert_eq!(assignments.len(), 2);
        assert_eq!(to_ids(&assignments[0]), vec![1, 2]);
        assert_eq!(to_ids(&assignments[1]), vec![2, 3]);
    }

    #[test]
    fn declare_topics_accepts_default_and_explicit_in_memory_backend() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  global_default: {}
  global_mem:
    backend: in_memory
groups:
  g1:
    topics:
      local_default: {}
      local_mem:
        backend: in_memory
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");

        assert_eq!(declared.broker.topic_names().len(), 4);
    }

    #[test]
    fn declare_topics_rejects_unimplemented_backend_kind() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  global_quiver:
    backend: quiver
groups:
  g1:
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        match Controller::<()>::declare_topics(&config) {
            Err(Error::UnsupportedTopicBackend { topic, backend }) => {
                assert_eq!(topic.as_ref(), "global::global_quiver");
                assert_eq!(backend, TopicBackendKind::Quiver);
            }
            Ok(_) => panic!("quiver backend should be rejected"),
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_topic_runtime_support_rejects_unsupported_mode_for_backend_capabilities() {
        let topic = TopicName::parse("test_topic").expect("topic name should parse");
        let capabilities = TopicBackendCapabilities {
            supports_balanced_only: true,
            supports_broadcast_only: false,
            supports_mixed: false,
            supports_broadcast_on_lag_drop_oldest: true,
            supports_broadcast_on_lag_disconnect: false,
            supports_ack_propagation_disabled: true,
            supports_ack_propagation_auto: true,
        };

        let err = Controller::<()>::validate_topic_runtime_support_with_capabilities(
            &topic,
            TopicBackendKind::InMemory,
            &TopicSpec::default().policies,
            InferredTopicMode::BroadcastOnly,
            capabilities,
        )
        .expect_err("broadcast_only should be rejected");

        match err {
            Error::UnsupportedTopicMode {
                topic,
                backend,
                mode,
            } => {
                assert_eq!(topic.as_ref(), "test_topic");
                assert_eq!(backend, TopicBackendKind::InMemory);
                assert_eq!(mode, "broadcast_only");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_topic_runtime_support_rejects_unsupported_broadcast_lag_policy() {
        let topic = TopicName::parse("test_topic").expect("topic name should parse");
        let capabilities = TopicBackendCapabilities {
            supports_balanced_only: true,
            supports_broadcast_only: true,
            supports_mixed: true,
            supports_broadcast_on_lag_drop_oldest: true,
            supports_broadcast_on_lag_disconnect: false,
            supports_ack_propagation_disabled: true,
            supports_ack_propagation_auto: true,
        };
        let mut spec = TopicSpec::default();
        spec.policies.broadcast.on_lag = TopicBroadcastOnLagPolicy::Disconnect;

        let err = Controller::<()>::validate_topic_runtime_support_with_capabilities(
            &topic,
            TopicBackendKind::InMemory,
            &spec.policies,
            InferredTopicMode::BroadcastOnly,
            capabilities,
        )
        .expect_err("disconnect lag policy should be rejected");

        match err {
            Error::UnsupportedTopicPolicy {
                topic,
                backend,
                policy,
                value,
            } => {
                assert_eq!(topic.as_ref(), "test_topic");
                assert_eq!(backend, TopicBackendKind::InMemory);
                assert_eq!(policy, "broadcast.on_lag");
                assert_eq!(value, "disconnect");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_topic_runtime_support_rejects_unsupported_ack_propagation_policy() {
        let topic = TopicName::parse("test_topic").expect("topic name should parse");
        let capabilities = TopicBackendCapabilities {
            supports_balanced_only: true,
            supports_broadcast_only: true,
            supports_mixed: true,
            supports_broadcast_on_lag_drop_oldest: true,
            supports_broadcast_on_lag_disconnect: true,
            supports_ack_propagation_disabled: true,
            supports_ack_propagation_auto: false,
        };
        let mut spec = TopicSpec::default();
        spec.policies.ack_propagation.mode = TopicAckPropagationMode::Auto;

        let err = Controller::<()>::validate_topic_runtime_support_with_capabilities(
            &topic,
            TopicBackendKind::InMemory,
            &spec.policies,
            InferredTopicMode::BalancedOnly,
            capabilities,
        )
        .expect_err("ack auto should be rejected");

        match err {
            Error::UnsupportedTopicPolicy {
                topic,
                backend,
                policy,
                value,
            } => {
                assert_eq!(topic.as_ref(), "test_topic");
                assert_eq!(backend, TopicBackendKind::InMemory);
                assert_eq!(policy, "ack_propagation");
                assert_eq!(value, "auto");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn declare_topics_rejects_same_pipeline_topic_wiring_cycle() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  loop: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          loop_receiver:
            type: "urn:otel:receiver:topic"
            config:
              topic: loop
          loop_exporter:
            type: "urn:otel:exporter:topic"
            config:
              topic: loop
        connections:
          - from: loop_receiver
            to: loop_exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let err = Controller::<()>::declare_topics(&config)
            .err()
            .expect("same-pipeline topic feedback loop should be rejected");
        match err {
            Error::TopicWiringCycleDetected { cycle } => {
                assert!(cycle.len() >= 4, "unexpected cycle path: {cycle:?}");
                assert_eq!(cycle.first(), cycle.last());
                assert!(cycle.contains(&"topic:global::loop".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p1/loop_receiver".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p1/loop_exporter".to_owned()));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn declare_topics_rejects_cross_pipeline_topic_wiring_cycle() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  topic_a: {}
  topic_b: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          from_topic_a:
            type: "urn:otel:receiver:topic"
            config:
              topic: topic_a
          to_topic_b:
            type: "urn:otel:exporter:topic"
            config:
              topic: topic_b
        connections:
          - from: from_topic_a
            to: to_topic_b
      p2:
        nodes:
          from_topic_b:
            type: "urn:otel:receiver:topic"
            config:
              topic: topic_b
          to_topic_a:
            type: "urn:otel:exporter:topic"
            config:
              topic: topic_a
        connections:
          - from: from_topic_b
            to: to_topic_a
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let err = Controller::<()>::declare_topics(&config)
            .err()
            .expect("cross-pipeline topic cycle should be rejected");
        match err {
            Error::TopicWiringCycleDetected { cycle } => {
                assert!(cycle.len() >= 6, "unexpected cycle path: {cycle:?}");
                assert_eq!(cycle.first(), cycle.last());
                assert!(cycle.contains(&"topic:global::topic_a".to_owned()));
                assert!(cycle.contains(&"topic:global::topic_b".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p1/from_topic_a".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p1/to_topic_b".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p2/from_topic_b".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p2/to_topic_a".to_owned()));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn declare_topics_infers_balanced_only_for_single_consumer_group() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  balanced_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: balanced_topic
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "balanced_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(matches!(
            topic.subscribe(
                otap_df_engine::topic::SubscriptionMode::Broadcast,
                otap_df_engine::topic::SubscriberOptions::default(),
            ),
            Err(otap_df_engine::error::Error::SubscribeBroadcastNotSupported)
        ));
    }

    #[test]
    fn declare_topics_infers_broadcast_only_when_only_broadcast_receivers_exist() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  broadcast_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: broadcast_topic
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "broadcast_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(matches!(
            topic.subscribe(
                otap_df_engine::topic::SubscriptionMode::Balanced { group: "g1".into() },
                otap_df_engine::topic::SubscriberOptions::default(),
            ),
            Err(otap_df_engine::error::Error::SubscribeBalancedNotSupported)
        ));
    }

    #[test]
    fn declare_topics_keeps_mixed_for_multiple_balanced_groups() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  mixed_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: mixed_topic
              subscription:
                mode: balanced
                group: g1
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
      p2:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: mixed_topic
              subscription:
                mode: balanced
                group: g2
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "mixed_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced { group: "g3".into() },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
    }

    #[test]
    fn declare_topics_defaults_to_mixed_when_topic_has_no_receivers() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  idle_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "idle_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced { group: "g1".into() },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
    }

    #[test]
    fn declare_topics_inference_respects_group_local_topic_shadowing() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  shared: {}
groups:
  g1:
    topics:
      shared: {}
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: shared
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let global_topic = global_topic_handle(&declared, "shared");
        let group_topic = group_topic_handle(&declared, "g1", "shared");

        assert!(
            global_topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(matches!(
            group_topic.subscribe(
                otap_df_engine::topic::SubscriptionMode::Broadcast,
                otap_df_engine::topic::SubscriberOptions::default(),
            ),
            Err(otap_df_engine::error::Error::SubscribeBroadcastNotSupported)
        ));
    }

    #[test]
    fn declare_topics_engine_default_force_mixed_disables_optimization() {
        let yaml = r#"
version: otel_dataflow/v1
engine:
  topics:
    impl_selection: force_mixed
topics:
  balanced_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: balanced_topic
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "balanced_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
    }

    #[test]
    fn declare_topics_topic_override_auto_wins_over_engine_force_mixed() {
        let yaml = r#"
version: otel_dataflow/v1
engine:
  topics:
    impl_selection: force_mixed
topics:
  balanced_topic:
    impl_selection: auto
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: balanced_topic
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "balanced_topic");

        assert!(matches!(
            topic.subscribe(
                otap_df_engine::topic::SubscriptionMode::Broadcast,
                otap_df_engine::topic::SubscriberOptions::default(),
            ),
            Err(otap_df_engine::error::Error::SubscribeBroadcastNotSupported)
        ));
        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
    }

    #[test]
    fn build_pipeline_topic_set_wires_topic_queue_on_full_policy() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  global_drop:
    policies:
      balanced:
        queue_capacity: 8
        on_full: drop_newest
      broadcast:
        queue_capacity: 8
        on_lag: disconnect
      ack_propagation:
        mode: auto
        max_in_flight: 21
        timeout: 45s
groups:
  g1:
    topics:
      local_block:
        policies:
          balanced:
            queue_capacity: 8
            on_full: block
          broadcast:
            queue_capacity: 8
            on_lag: drop_oldest
          ack_propagation:
            mode: disabled
            max_in_flight: 22
            timeout: 46s
      # Same local alias as global to verify group-local override path.
      global_drop:
        policies:
          balanced:
            queue_capacity: 8
            on_full: block
          broadcast:
            queue_capacity: 8
            on_lag: drop_oldest
          ack_propagation:
            mode: disabled
            max_in_flight: 23
            timeout: 47s
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let group_id: PipelineGroupId = "g1".into();
        let pipeline_id: PipelineId = "p1".into();
        let set = Controller::<()>::build_pipeline_topic_set(
            &config,
            &declared,
            &group_id,
            &pipeline_id,
            0,
        )
        .expect("topic set should build");

        let local_block = set
            .get_required(TopicName::from("local_block"))
            .expect("local_block topic must exist");
        assert_eq!(
            local_block.default_queue_on_full(),
            otap_df_config::topic::TopicQueueOnFullPolicy::Block
        );
        assert_eq!(
            local_block.default_ack_propagation_mode(),
            otap_df_config::topic::TopicAckPropagationMode::Disabled
        );
        assert_eq!(
            local_block.broadcast_on_lag_policy(),
            otap_df_config::topic::TopicBroadcastOnLagPolicy::DropOldest
        );
        assert_eq!(
            local_block.default_publish_outcome_config().max_in_flight,
            22
        );
        assert_eq!(
            local_block.default_publish_outcome_config().timeout,
            std::time::Duration::from_secs(46)
        );

        // group-local declaration must override global policy for same local name
        let overridden = set
            .get_required(TopicName::from("global_drop"))
            .expect("overridden topic must exist");
        assert_eq!(
            overridden.default_queue_on_full(),
            otap_df_config::topic::TopicQueueOnFullPolicy::Block
        );
        assert_eq!(
            overridden.default_ack_propagation_mode(),
            otap_df_config::topic::TopicAckPropagationMode::Disabled
        );
        assert_eq!(
            overridden.broadcast_on_lag_policy(),
            otap_df_config::topic::TopicBroadcastOnLagPolicy::DropOldest
        );
        assert_eq!(
            overridden.default_publish_outcome_config().max_in_flight,
            23
        );
        assert_eq!(
            overridden.default_publish_outcome_config().timeout,
            std::time::Duration::from_secs(47)
        );
    }

    #[tokio::test]
    async fn declare_topics_preserves_separate_balanced_and_broadcast_capacities() {
        let yaml = r#"
version: otel_dataflow/v1
engine:
  topics:
    impl_selection: force_mixed
topics:
  mixed_topic:
    policies:
      balanced:
        queue_capacity: 1
      broadcast:
        queue_capacity: 3
        on_lag: disconnect
groups:
  g1:
    pipelines:
      balanced_consumer:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: mixed_topic
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
      broadcast_consumer:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: mixed_topic
              subscription:
                mode: broadcast
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "mixed_topic");

        let mut balanced = topic
            .subscribe(
                otap_df_engine::topic::SubscriptionMode::Balanced {
                    group: "workers".into(),
                },
                otap_df_engine::topic::SubscriberOptions::default(),
            )
            .expect("balanced subscription should succeed");
        let mut broadcast = topic
            .subscribe(
                otap_df_engine::topic::SubscriptionMode::Broadcast,
                otap_df_engine::topic::SubscriberOptions::default(),
            )
            .expect("broadcast subscription should succeed");

        assert_eq!(
            topic
                .try_publish(Arc::new(()))
                .expect("publish should succeed"),
            otap_df_engine::topic::PublishOutcome::Published
        );
        assert_eq!(
            topic
                .try_publish(Arc::new(()))
                .expect("publish should still reach broadcast"),
            otap_df_engine::topic::PublishOutcome::DroppedOnFull
        );
        assert_eq!(
            topic
                .try_publish(Arc::new(()))
                .expect("publish should still reach broadcast"),
            otap_df_engine::topic::PublishOutcome::DroppedOnFull
        );
        assert_eq!(
            topic.broadcast_on_lag_policy(),
            otap_df_config::topic::TopicBroadcastOnLagPolicy::Disconnect
        );
        topic.close();

        let mut balanced_messages = 0usize;
        while let Ok(item) = balanced.recv().await {
            match item {
                otap_df_engine::topic::RecvItem::Message(_) => balanced_messages += 1,
                otap_df_engine::topic::RecvItem::Lagged { missed } => {
                    panic!("unexpected lag for balanced subscription: missed={missed}");
                }
            }
        }
        assert_eq!(balanced_messages, 1);

        let mut broadcast_messages = 0usize;
        while let Ok(item) = broadcast.recv().await {
            match item {
                otap_df_engine::topic::RecvItem::Message(_) => broadcast_messages += 1,
                otap_df_engine::topic::RecvItem::Lagged { missed } => {
                    panic!("unexpected lag with broadcast capacity 3: missed={missed}");
                }
            }
        }
        assert_eq!(broadcast_messages, 3);
    }
}
