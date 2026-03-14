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
use otap_df_config::policy::MemoryLimiterMode;
use otap_df_config::policy::{ChannelCapacityPolicy, CoreAllocation, TelemetryPolicy};
use otap_df_config::topic::{
    TopicAckPropagationMode, TopicBackendKind, TopicBroadcastOnLagPolicy, TopicImplSelectionPolicy,
    TopicSpec,
};
use otap_df_config::transport_headers_policy::TransportHeadersPolicy;
use otap_df_config::{
    DeployedPipelineKey, PipelineGroupId, PipelineId, PipelineKey, SubscriptionGroupName,
    TopicName, pipeline::PipelineConfig,
};
use otap_df_engine::PipelineFactory;
use otap_df_engine::ReceivedAtNode;
use otap_df_engine::Unwindable;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    PipelineAdminSender, PipelineCompletionMsgReceiver, PipelineCompletionMsgSender,
    RuntimeCtrlMsgReceiver, RuntimeCtrlMsgSender, pipeline_completion_msg_channel,
    runtime_ctrl_msg_channel,
};
use otap_df_engine::entity_context::{
    node_entity_key, pipeline_entity_key, set_pipeline_entity_key,
};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::memory_limiter::{
    EffectiveMemoryLimiter, MemoryLimiterTick, MemoryPressureBehaviorConfig, MemoryPressureChanged,
    MemoryPressureLevel,
};
use otap_df_engine::topic::{
    InMemoryBackend, PipelineTopicBinding, TopicBroker, TopicOptions, TopicPublishOutcomeConfig,
    TopicSet,
};
use otap_df_state::store::{ObservedStateHandle, ObservedStateStore};
use otap_df_telemetry::event::{EngineEvent, ErrorSummary, ObservedEventReporter};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry::{
    InternalTelemetrySettings, InternalTelemetrySystem, TracingSetup, otel_error, otel_info,
    otel_info_span, otel_warn, self_tracing::LogContext,
};
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use std::thread;
use std::time::Duration;

/// Error types and helpers for the controller module.
pub mod error;
mod live_control;
/// Reusable startup helpers (validation, CLI overrides, system info).
pub mod startup;
/// Utilities to spawn async tasks on dedicated threads with graceful shutdown.
pub mod thread_task;

use live_control::{
    ControllerRuntime, LaunchedPipelineThread, PanicReport, RuntimeInstanceError,
    RuntimeInstanceExit,
};

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

    /// Validates component-specific configuration for one pipeline before startup or reconfigure.
    fn validate_pipeline_components_with_factory(
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
        pipeline_cfg: &PipelineConfig,
    ) -> Result<(), String> {
        for (node_id, node_cfg) in pipeline_cfg.node_iter() {
            let urn_str = node_cfg.r#type.as_str();
            let validate_config_fn = match node_cfg.kind() {
                NodeKind::Receiver => pipeline_factory
                    .get_receiver_factory_map()
                    .get(urn_str)
                    .map(|factory| factory.validate_config),
                NodeKind::Processor | NodeKind::ProcessorChain => pipeline_factory
                    .get_processor_factory_map()
                    .get(urn_str)
                    .map(|factory| factory.validate_config),
                NodeKind::Exporter => pipeline_factory
                    .get_exporter_factory_map()
                    .get(urn_str)
                    .map(|factory| factory.validate_config),
                NodeKind::Extension => {
                    // Extensions are not yet validated here because PipelineFactory
                    // does not expose an extension factory registry.
                    continue;
                }
            };

            let Some(validate_fn) = validate_config_fn else {
                let kind_name = match node_cfg.kind() {
                    NodeKind::Receiver => "receiver",
                    NodeKind::Processor | NodeKind::ProcessorChain => "processor",
                    NodeKind::Exporter => "exporter",
                    NodeKind::Extension => unreachable!("handled above"),
                };
                return Err(format!(
                    "Unknown {} component `{}` in pipeline_group={} pipeline={} node={}",
                    kind_name,
                    urn_str,
                    pipeline_group_id.as_ref(),
                    pipeline_id.as_ref(),
                    node_id.as_ref()
                ));
            };

            validate_fn(&node_cfg.config).map_err(|err| {
                format!(
                    "Invalid config for component `{}` in pipeline_group={} pipeline={} node={}: {}",
                    urn_str,
                    pipeline_group_id.as_ref(),
                    pipeline_id.as_ref(),
                    node_id.as_ref(),
                    err
                )
            })?;
        }
        Ok(())
    }

    /// Validates every configured pipeline and observability pipeline against registered components.
    fn validate_engine_components_with_factory(
        pipeline_factory: &'static PipelineFactory<PData>,
        engine_cfg: &OtelDataflowSpec,
    ) -> Result<(), String> {
        for (pipeline_group_id, pipeline_group) in &engine_cfg.groups {
            for (pipeline_id, pipeline_cfg) in &pipeline_group.pipelines {
                Self::validate_pipeline_components_with_factory(
                    pipeline_factory,
                    pipeline_group_id,
                    pipeline_id,
                    pipeline_cfg,
                )?;
            }
        }

        if let Some(obs_pipeline) = &engine_cfg.engine.observability.pipeline {
            let obs_group_id: PipelineGroupId = SYSTEM_PIPELINE_GROUP_ID.into();
            let obs_pipeline_id: PipelineId = SYSTEM_OBSERVABILITY_PIPELINE_ID.into();
            let obs_pipeline_config = obs_pipeline.clone().into_pipeline_config();
            Self::validate_pipeline_components_with_factory(
                pipeline_factory,
                &obs_group_id,
                &obs_pipeline_id,
                &obs_pipeline_config,
            )?;
        }

        Ok(())
    }

    /// Validates that every configured node resolves to a registered component and that the
    /// static component-specific configuration validates.
    pub fn validate_engine_components(&self, engine_cfg: &OtelDataflowSpec) -> Result<(), String> {
        Self::validate_engine_components_with_factory(self.pipeline_factory, engine_cfg)
    }

    /// Starts the controller with the given engine configurations.
    pub fn run_forever(&self, engine_config: OtelDataflowSpec) -> Result<(), Error> {
        self.run_with_mode(
            engine_config,
            RunMode::ParkMainThread,
            None::<fn(ObservedStateHandle)>,
        )
    }

    /// Starts the controller and invokes `observer` with an
    /// [`ObservedStateHandle`] as soon as the pipeline state store is ready.
    ///
    /// The callback fires once, before the engine blocks. Use it to obtain
    /// zero-overhead, in-process access to pipeline liveness, readiness, and
    /// health without going through the admin HTTP server.
    pub fn run_forever_with_observer<F>(
        &self,
        engine_config: OtelDataflowSpec,
        observer: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(ObservedStateHandle),
    {
        self.run_with_mode(engine_config, RunMode::ParkMainThread, Some(observer))
    }

    /// Starts the controller with the given engine configurations.
    ///
    /// Runs until pipelines are shut down, then closes telemetry/admin services.
    pub fn run_till_shutdown(&self, engine_config: OtelDataflowSpec) -> Result<(), Error> {
        self.run_with_mode(
            engine_config,
            RunMode::ShutdownWhenDone,
            None::<fn(ObservedStateHandle)>,
        )
    }

    /// Like [`run_till_shutdown`](Self::run_till_shutdown), but invokes
    /// `observer` with an [`ObservedStateHandle`] before blocking.
    pub fn run_till_shutdown_with_observer<F>(
        &self,
        engine_config: OtelDataflowSpec,
        observer: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(ObservedStateHandle),
    {
        self.run_with_mode(engine_config, RunMode::ShutdownWhenDone, Some(observer))
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
    ) -> Result<
        (
            HashMap<TopicName, InferredTopicMode>,
            Vec<InferredTopicModeReport>,
        ),
        Error,
    > {
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

        let mut declared_topics: Vec<_> = usage_by_declared_topic.into_iter().collect();
        declared_topics.sort_by(|(left, _), (right, _)| left.as_ref().cmp(right.as_ref()));

        let mut inferred_modes = HashMap::with_capacity(declared_topics.len());
        let mut inferred_mode_reports = Vec::with_capacity(declared_topics.len());
        for (declared_topic, summary) in declared_topics {
            let topology_mode = Self::infer_topic_mode(&summary);
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

        Ok((inferred_modes, inferred_mode_reports))
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
            let Some(group_cfg) = config.groups.get(&group_id) else {
                return Err(Error::PipelineRuntimeError {
                    source: Box::new(EngineError::InternalError {
                        message: format!(
                            "group `{}` disappeared while validating topic wiring",
                            group_id.as_ref()
                        ),
                    }),
                });
            };
            let mut pipelines = group_cfg.pipelines.iter().collect::<Vec<_>>();
            pipelines.sort_by(|(left, _), (right, _)| left.as_ref().cmp(right.as_ref()));
            for (pipeline_id, pipeline_cfg) in pipelines {
                Self::collect_topic_wiring_edges_for_pipeline(
                    &mut adjacency,
                    &group_id,
                    pipeline_id,
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
            Self::infer_topic_modes(config, &global_names, &group_names)?;
        let default_selection_policy = config.engine.topics.impl_selection;

        for (topic_name, spec) in &config.topics {
            let declared_name = global_names
                .get(topic_name)
                .ok_or_else(|| Error::PipelineRuntimeError {
                    source: Box::new(EngineError::InternalError {
                        message: format!(
                            "missing declared topic name for global topic `{}` during topic declaration",
                            topic_name.as_ref()
                        ),
                    }),
                })?
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
                    .ok_or_else(|| Error::PipelineRuntimeError {
                        source: Box::new(EngineError::InternalError {
                            message: format!(
                                "missing declared topic name for group `{}` topic `{}` during topic declaration",
                                group_id.as_ref(),
                                topic_name.as_ref()
                            ),
                        }),
                    })?
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

    fn run_with_mode<F>(
        &self,
        engine_config: OtelDataflowSpec,
        run_mode: RunMode,
        observer: Option<F>,
    ) -> Result<(), Error>
    where
        F: FnOnce(ObservedStateHandle),
    {
        let num_pipeline_groups = engine_config.groups.len();
        let resolved_config = engine_config.resolve();
        let (engine, pipelines, observability_pipeline) = resolved_config.into_parts();
        let num_pipelines = pipelines.len();
        let admin_settings = engine.http_admin.clone().unwrap_or_default();
        // Initialize metrics system and observed event store.
        // ToDo A hierarchical metrics system will be implemented to better support hardware with multiple NUMA nodes.
        let telemetry_config = &engine.telemetry;
        let telemetry_reporting_interval = engine.telemetry.reporting_interval;
        otel_info!(
            "controller.start",
            num_pipeline_groups = num_pipeline_groups,
            num_pipelines = num_pipelines
        );

        // Create the shared telemetry registry first - it will be used by both
        // the observed state store and the internal telemetry system.
        let telemetry_registry = TelemetryRegistryHandle::new();
        let log_tap_handle = telemetry_config
            .logs
            .tap
            .enabled
            .then(|| otap_df_telemetry::log_tap::build(&telemetry_config.logs.tap));

        // Create the observed state store for the telemetry system.
        let obs_state_store = ObservedStateStore::new_with_log_tap(
            &engine.observed_state,
            telemetry_registry.clone(),
            log_tap_handle.clone(),
        );
        let obs_state_handle = obs_state_store.handle();

        // Notify the caller with a clone of the observed-state handle, if requested.
        if let Some(observer) = observer {
            observer(obs_state_handle.clone());
        }

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
            log_tap_handle.clone(),
        )?;

        let admin_tracing_setup = telemetry_system.admin_tracing_setup();
        let internal_tracing_setup = telemetry_system.internal_tracing_setup();

        let metrics_dispatcher = telemetry_system.dispatcher();
        let metrics_reporter = telemetry_system.reporter();
        let controller_ctx = ControllerContext::new(telemetry_system.registry());
        let memory_pressure_state = controller_ctx.memory_pressure_state();
        let (memory_pressure_tx, _memory_pressure_rx) =
            tokio::sync::watch::channel(MemoryPressureChanged::initial());
        let mut memory_limiter_handle = None;
        if let Some(memory_limiter_policy) = engine_config
            .policies
            .resources()
            .and_then(|resources| resources.memory_limiter.as_ref())
        {
            memory_pressure_state.configure(MemoryPressureBehaviorConfig {
                retry_after_secs: memory_limiter_policy.retry_after_secs,
                fail_readiness_on_hard: memory_limiter_policy.fail_readiness_on_hard,
                mode: memory_limiter_policy.mode,
            });

            let mut limiter = EffectiveMemoryLimiter::from_policy(memory_limiter_policy)
                .map_err(|message| Error::MemoryLimiterError { message })?;
            if memory_limiter_policy.mode == MemoryLimiterMode::ObserveOnly {
                if memory_limiter_policy.purge_on_hard {
                    otel_warn!(
                        "process_memory_limiter.observe_only_ignored_setting",
                        setting = "purge_on_hard",
                        message =
                            "purge_on_hard is ignored when memory_limiter.mode is observe_only"
                    );
                }
            } else if limiter.purge_on_hard() && !limiter.purge_supported() {
                otel_warn!(
                    "process_memory_limiter.purge_unavailable",
                    source = format!("{:?}", memory_limiter_policy.source),
                    message = "purge_on_hard is enabled, but no allocator purge backend is available for this build"
                );
            }
            let initial_tick = limiter
                .tick(&memory_pressure_state)
                .map_err(|message| Error::MemoryLimiterError { message })?;
            let initial_transitioned = initial_tick.transitioned();
            Self::log_memory_limiter_tick(initial_tick);
            let mut transition_generation = 0_u64;
            if initial_transitioned {
                transition_generation += 1;
                let _ = memory_pressure_tx
                    .send(memory_pressure_state.current_update(transition_generation));
            }

            let limiter_state = memory_pressure_state.clone();
            let limiter_updates = memory_pressure_tx.clone();
            memory_limiter_handle = Some(spawn_thread_local_task(
                "process-memory-limiter",
                admin_tracing_setup.clone(),
                move |cancellation_token| async move {
                    use tokio::time::{Instant, MissedTickBehavior, interval_at};

                    let mut ticker = interval_at(
                        Instant::now() + limiter.check_interval(),
                        limiter.check_interval(),
                    );
                    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

                    loop {
                        tokio::select! {
                            _ = cancellation_token.cancelled() => {
                                return Ok::<(), otap_df_telemetry::error::Error>(());
                            }
                            _ = ticker.tick() => {
                                match limiter.tick(&limiter_state) {
                                    Ok(tick) => {
                                        if tick.transitioned() {
                                            transition_generation += 1;
                                            let _ = limiter_updates.send(MemoryPressureChanged {
                                                generation: transition_generation,
                                                level: tick.current_level,
                                                retry_after_secs: limiter_state.retry_after_secs(),
                                                usage_bytes: tick.sample.usage_bytes,
                                            });
                                        }
                                        Self::log_memory_limiter_tick(tick)
                                    }
                                    Err(err) => {
                                        otel_warn!(
                                            "process_memory_limiter.sample_failed",
                                            error = err.as_str()
                                        );
                                    }
                                }
                            }
                        }
                    }
                },
            )?);
        }
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

        let all_cores =
            core_affinity::get_core_ids().ok_or_else(|| Error::CoreDetectionUnavailable)?;
        let its_core = *all_cores
            .first()
            .ok_or_else(|| Error::CoreDetectionUnavailable)?;
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
        let planned_core_assignments =
            Self::preflight_pipeline_core_allocations(&pipelines, &all_cores)?;

        let runtime = Arc::new(ControllerRuntime::new(
            self.pipeline_factory,
            controller_ctx.clone(),
            obs_state_store.clone(),
            obs_state_handle.clone(),
            engine_evt_reporter.clone(),
            metrics_reporter.clone(),
            declared_topics,
            all_cores.clone(),
            telemetry_system.engine_tracing_setup(),
            telemetry_reporting_interval,
            memory_pressure_tx.clone(),
            engine_config.clone(),
        ));

        // Pipeline threads receive only a Weak handle back to the controller runtime. That lets
        // them report their terminal exit without becoming owners that keep the runtime alive
        // during shutdown.
        let internal_pipeline_handle = Self::spawn_internal_pipeline_if_configured(
            Arc::downgrade(&runtime),
            its_key.clone(),
            its_core,
            observability_pipeline,
            &engine_config,
            &telemetry_system,
            self.pipeline_factory,
            &controller_ctx,
            &engine_evt_reporter,
            &metrics_reporter,
            telemetry_reporting_interval,
            &memory_pressure_tx,
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
        Self::emit_topic_mode_reports(&runtime.declared_topics().inferred_mode_reports);

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
        let obs_state_store_runtime = obs_state_store.clone();
        let obs_state_join_handle = spawn_thread_local_task(
            "observed-state-store",
            admin_tracing_setup.clone(),
            move |cancellation_token| obs_state_store_runtime.run(cancellation_token),
        )?;

        // Start the engine-wide metrics collection task.
        // This samples engine-level metrics (e.g. RSS) on a fixed interval and
        // reports them once per engine, rather than duplicating across pipelines.
        let engine_entity_key = controller_ctx.register_engine_entity();
        let engine_registry = controller_ctx.telemetry_registry();
        let engine_reporter = metrics_reporter.clone();
        let engine_metrics_memory_pressure_state = memory_pressure_state.clone();
        let engine_metrics_handle = spawn_thread_local_task(
            "engine-metrics",
            admin_tracing_setup.clone(),
            move |cancellation_token| async move {
                use otap_df_engine::engine_metrics::EngineMetricsMonitor;
                use std::time::Duration;
                use tokio::time::{MissedTickBehavior, interval};

                // TODO: Make this interval configurable via engine config.
                const ENGINE_METRICS_INTERVAL: Duration = Duration::from_secs(5);

                let mut monitor = EngineMetricsMonitor::new(
                    engine_registry,
                    engine_entity_key,
                    engine_reporter,
                    engine_metrics_memory_pressure_state.clone(),
                );

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

        if let Some(launched) = internal_pipeline_handle {
            runtime.register_launched_instance(launched);
        }

        for (pipeline_entry, requested_cores) in pipelines.iter().zip(planned_core_assignments) {
            runtime.register_committed_pipeline(pipeline_entry.clone(), 0);
            let num_cores = requested_cores.len();

            let core_allocation = pipeline_entry
                .policies
                .resources
                .core_allocation
                .to_string();
            otel_info!(
                "pipeline.core_allocation",
                pipeline_group_id = pipeline_entry.pipeline_group_id.as_ref(),
                pipeline_id = pipeline_entry.pipeline_id.as_ref(),
                num_cores = num_cores,
                core_allocation = core_allocation
            );

            for core_id in &requested_cores {
                // Pass a Weak runtime handle into each pipeline thread. The thread upgrades it
                // only when it needs to report Success/Error/Panic on exit, and silently skips
                // that late report if shutdown has already dropped the runtime.
                let launched = Self::launch_pipeline_thread(
                    self.pipeline_factory,
                    DeployedPipelineKey {
                        pipeline_group_id: pipeline_entry.pipeline_group_id.clone(),
                        pipeline_id: pipeline_entry.pipeline_id.clone(),
                        core_id: core_id.id,
                        deployment_generation: 0,
                    },
                    *core_id,
                    num_cores,
                    pipeline_entry.pipeline.clone(),
                    pipeline_entry.policies.channel_capacity.clone(),
                    pipeline_entry.policies.telemetry.clone(),
                    pipeline_entry.policies.transport_headers.clone(),
                    controller_ctx.clone(),
                    metrics_reporter.clone(),
                    engine_evt_reporter.clone(),
                    telemetry_system.engine_tracing_setup(),
                    telemetry_reporting_interval,
                    memory_pressure_tx.clone(),
                    &engine_config,
                    runtime.declared_topics(),
                    Arc::downgrade(&runtime),
                    runtime.next_thread_id(),
                    None,
                )?;
                runtime.register_launched_instance(launched);
            }
        }

        drop(metrics_reporter);

        let control_plane = runtime.control_plane();
        let admin_server_handle = spawn_thread_local_task(
            "http-admin",
            admin_tracing_setup,
            move |cancellation_token| {
                otap_df_admin::run(
                    admin_settings,
                    obs_state_handle,
                    control_plane,
                    telemetry_registry,
                    memory_pressure_state,
                    log_tap_handle,
                    cancellation_token,
                )
            },
        )?;

        if run_mode == RunMode::ShutdownWhenDone {
            runtime.wait_until_all_instances_exit();
        }

        // In standard engine mode we keep the main thread parked after startup.
        if run_mode == RunMode::ParkMainThread {
            thread::park();
        }

        // All pipelines have finished; shut down the admin HTTP server and metric aggregator gracefully.
        engine_metrics_handle.shutdown_and_join()?;
        if let Some(handle) = memory_limiter_handle {
            handle.shutdown_and_join()?;
        }
        admin_server_handle.shutdown_and_join()?;
        metrics_agg_handle.shutdown_and_join()?;
        if let Some(handle) = metrics_dispatcher_handle {
            handle.shutdown_and_join()?;
        }
        obs_state_join_handle.shutdown_and_join()?;
        telemetry_system.shutdown_otel()?;

        if let Some(err) = runtime.take_runtime_error() {
            return Err(err);
        }

        Ok(())
    }

    fn log_memory_limiter_tick(tick: MemoryLimiterTick) {
        if let Some(purge_error) = tick.purge_error.as_deref() {
            otel_warn!(
                "process_memory_limiter.purge_failed",
                source = tick.sample.source.as_str(),
                pre_purge_usage_bytes = tick
                    .pre_purge_usage_bytes
                    .unwrap_or(tick.sample.usage_bytes),
                usage_bytes = tick.sample.usage_bytes,
                error = purge_error
            );
        }

        if let Some(purge_duration) = tick.purge_duration {
            otel_info!(
                "process_memory_limiter.purge",
                source = tick.sample.source.as_str(),
                pre_purge_usage_bytes = tick
                    .pre_purge_usage_bytes
                    .unwrap_or(tick.sample.usage_bytes),
                post_purge_usage_bytes = tick.sample.usage_bytes,
                purge_duration_ms = purge_duration.as_millis() as u64,
                current = format!("{:?}", tick.current_level)
            );
        }

        if !tick.transitioned() {
            return;
        }

        let usage_bytes = tick.sample.usage_bytes;
        let source = tick.sample.source.as_str();
        match tick.current_level {
            MemoryPressureLevel::Hard => {
                otel_warn!(
                    "process_memory_limiter.transition",
                    previous = format!("{:?}", tick.previous_level),
                    current = format!("{:?}", tick.current_level),
                    source = source,
                    usage_bytes = usage_bytes,
                    soft_limit_bytes = tick.soft_limit_bytes,
                    hard_limit_bytes = tick.hard_limit_bytes
                );
            }
            MemoryPressureLevel::Soft => {
                otel_info!(
                    "process_memory_limiter.transition",
                    previous = format!("{:?}", tick.previous_level),
                    current = "Soft",
                    source = source,
                    usage_bytes = usage_bytes,
                    soft_limit_bytes = tick.soft_limit_bytes,
                    hard_limit_bytes = tick.hard_limit_bytes
                );
            }
            MemoryPressureLevel::Normal => {
                otel_info!(
                    "process_memory_limiter.transition",
                    previous = format!("{:?}", tick.previous_level),
                    current = "Normal",
                    source = source,
                    usage_bytes = usage_bytes,
                    soft_limit_bytes = tick.soft_limit_bytes,
                    hard_limit_bytes = tick.hard_limit_bytes
                );
            }
        }
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
                    &pipeline_entry.policies.resources.core_allocation,
                )
            })
            .collect()
    }

    fn internal_pipeline_key(core_id: CoreId) -> DeployedPipelineKey {
        DeployedPipelineKey {
            pipeline_group_id: SYSTEM_PIPELINE_GROUP_ID.into(),
            pipeline_id: SYSTEM_OBSERVABILITY_PIPELINE_ID.into(),
            core_id: core_id.id,
            deployment_generation: 0,
        }
    }

    /// Launches one pipeline OS thread and wires its terminal exit back into the controller.
    ///
    /// The spawned thread owns the actual pipeline execution and maps success, runtime error, or
    /// panic into RuntimeInstanceExit. `runtime` is a Weak handle on purpose: the pipeline thread
    /// should be able to report its exit, but it must not become an owner that prolongs the
    /// controller runtime during shutdown.
    #[allow(clippy::too_many_arguments)]
    fn launch_pipeline_thread(
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_key: DeployedPipelineKey,
        core_id: CoreId,
        num_cores: usize,
        pipeline_config: PipelineConfig,
        channel_capacity_policy: ChannelCapacityPolicy,
        telemetry_policy: TelemetryPolicy,
        transport_headers_policy: Option<TransportHeadersPolicy>,
        controller_ctx: ControllerContext,
        metrics_reporter: MetricsReporter,
        engine_evt_reporter: ObservedEventReporter,
        tracing_setup: TracingSetup,
        telemetry_reporting_interval: Duration,
        memory_pressure_tx: tokio::sync::watch::Sender<MemoryPressureChanged>,
        config: &OtelDataflowSpec,
        declared_topics: &DeclaredTopics<PData>,
        runtime: std::sync::Weak<ControllerRuntime<PData>>,
        thread_id: usize,
        internal_telemetry: Option<(
            InternalTelemetrySettings,
            std_mpsc::SyncSender<Result<(), EngineError>>,
        )>,
    ) -> Result<LaunchedPipelineThread<PData>, Error> {
        let mut pipeline_ctx = controller_ctx.pipeline_context_with_generation(
            pipeline_key.pipeline_group_id.clone(),
            pipeline_key.pipeline_id.clone(),
            pipeline_key.core_id,
            num_cores,
            thread_id,
            pipeline_key.deployment_generation,
        );
        let topic_set = Self::build_pipeline_topic_set(
            config,
            declared_topics,
            &pipeline_key.pipeline_group_id,
            &pipeline_key.pipeline_id,
            pipeline_key.core_id,
        )?;
        pipeline_ctx.set_topic_set(topic_set);
        let (runtime_ctrl_msg_tx, runtime_ctrl_msg_rx) =
            runtime_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
        let (pipeline_completion_msg_tx, pipeline_completion_msg_rx) =
            pipeline_completion_msg_channel(channel_capacity_policy.control.completion);
        let control_sender: Arc<dyn PipelineAdminSender> = Arc::new(runtime_ctrl_msg_tx.clone());
        let memory_pressure_rx = memory_pressure_tx.subscribe();
        let thread_name = format!(
            "pipeline-{}-{}-core-{}-gen-{}",
            pipeline_key.pipeline_group_id.as_ref(),
            pipeline_key.pipeline_id.as_ref(),
            pipeline_key.core_id,
            pipeline_key.deployment_generation
        );
        let run_key = pipeline_key.clone();
        let runtime_key = pipeline_key.clone();
        let runtime_thread_name = thread_name.clone();
        let _handle = thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || {
                let exit = match catch_unwind(AssertUnwindSafe(|| {
                    Self::run_pipeline_thread(
                        run_key,
                        core_id,
                        pipeline_config,
                        channel_capacity_policy,
                        telemetry_policy,
                        transport_headers_policy,
                        telemetry_reporting_interval,
                        pipeline_factory,
                        pipeline_ctx,
                        engine_evt_reporter,
                        metrics_reporter,
                        runtime_ctrl_msg_tx,
                        runtime_ctrl_msg_rx,
                        pipeline_completion_msg_tx,
                        pipeline_completion_msg_rx,
                        memory_pressure_rx,
                        tracing_setup,
                        internal_telemetry,
                    )
                })) {
                    Ok(Ok(_)) => RuntimeInstanceExit::Success,
                    Ok(Err(err)) => {
                        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime(err.to_string()))
                    }
                    Err(panic) => RuntimeInstanceExit::Error(RuntimeInstanceError::from_panic(
                        PanicReport::capture(
                            "runtime thread",
                            panic,
                            Some(runtime_thread_name),
                            Some(thread_id),
                            Some(runtime_key.core_id),
                        ),
                    )),
                };
                if let Some(runtime) = runtime.upgrade() {
                    runtime.note_instance_exit(runtime_key, exit);
                }
                // The controller runtime may already be gone during teardown. In that case there
                // is nothing left to update, so late exit reporting is intentionally best-effort.
            })
            .map_err(|e| Error::ThreadSpawnError {
                thread_name: thread_name.clone(),
                source: e,
            })?;

        Ok(LaunchedPipelineThread {
            pipeline_key,
            control_sender,
            _marker: std::marker::PhantomData,
        })
    }

    /// Spawns the internal telemetry pipeline if engine observability config provides one.
    ///
    /// Returns the thread handle if an internal pipeline was spawned
    /// and waits for it to start, or None.
    #[allow(clippy::too_many_arguments)]
    fn spawn_internal_pipeline_if_configured(
        runtime: std::sync::Weak<ControllerRuntime<PData>>,
        its_key: DeployedPipelineKey,
        its_core: CoreId,
        observability_pipeline: Option<ResolvedPipelineConfig>,
        config: &OtelDataflowSpec,
        telemetry_system: &InternalTelemetrySystem,
        pipeline_factory: &'static PipelineFactory<PData>,
        controller_ctx: &ControllerContext,
        engine_evt_reporter: &ObservedEventReporter,
        metrics_reporter: &MetricsReporter,
        telemetry_reporting_interval: Duration,
        memory_pressure_tx: &tokio::sync::watch::Sender<MemoryPressureChanged>,
        tracing_setup: TracingSetup,
    ) -> Result<Option<LaunchedPipelineThread<PData>>, Error> {
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

        // Create a channel to signal startup success/failure
        let (startup_tx, startup_rx) = std_mpsc::sync_channel::<Result<(), EngineError>>(1);
        let launched = Self::launch_pipeline_thread(
            pipeline_factory,
            its_key,
            its_core,
            1,
            internal_config,
            channel_capacity_policy,
            telemetry_policy,
            None,
            controller_ctx.clone(),
            metrics_reporter.clone(),
            engine_evt_reporter.clone(),
            tracing_setup,
            telemetry_reporting_interval,
            memory_pressure_tx.clone(),
            config,
            runtime
                .upgrade()
                .expect("controller runtime should exist while spawning internal pipeline")
                .declared_topics(),
            runtime,
            0,
            Some((its_settings, startup_tx)),
        )?;

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

        Ok(Some(launched))
    }

    /// Runs a single pipeline in the current thread.
    fn run_pipeline_thread(
        pipeline_key: DeployedPipelineKey,
        core_id: CoreId,
        pipeline_config: PipelineConfig,
        channel_capacity_policy: ChannelCapacityPolicy,
        telemetry_policy: TelemetryPolicy,
        transport_headers_policy: Option<TransportHeadersPolicy>,
        telemetry_reporting_interval: Duration,
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_context: PipelineContext,
        obs_evt_reporter: ObservedEventReporter,
        metrics_reporter: MetricsReporter,
        runtime_ctrl_msg_tx: RuntimeCtrlMsgSender<PData>,
        runtime_ctrl_msg_rx: RuntimeCtrlMsgReceiver<PData>,
        pipeline_completion_msg_tx: PipelineCompletionMsgSender<PData>,
        pipeline_completion_msg_rx: PipelineCompletionMsgReceiver<PData>,
        memory_pressure_rx: tokio::sync::watch::Receiver<MemoryPressureChanged>,
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
                    transport_headers_policy,
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
                    telemetry_reporting_interval,
                    memory_pressure_rx,
                    runtime_ctrl_msg_tx,
                    runtime_ctrl_msg_rx,
                    pipeline_completion_msg_tx,
                    pipeline_completion_msg_rx,
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

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::engine::{ResolvedPipelineConfig, ResolvedPipelineRole};
    use otap_df_config::policy::{CoreRange, ResolvedPolicies, ResourcesPolicy};
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
        ResolvedPipelineConfig {
            pipeline_group_id: pipeline_group_id.to_string().into(),
            pipeline_id: pipeline_id.to_string().into(),
            pipeline: minimal_pipeline_config(),
            policies: ResolvedPolicies {
                resources: ResourcesPolicy {
                    core_allocation,
                    ..Default::default()
                },
                ..Default::default()
            },
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
            Duration::from_secs(46)
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
            Duration::from_secs(47)
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
                .expect("publish should report backpressure once balanced is full"),
            otap_df_engine::topic::PublishOutcome::DroppedOnFull
        );
        assert_eq!(
            topic
                .try_publish(Arc::new(()))
                .expect("publish should keep reporting backpressure while balanced is full"),
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
        assert_eq!(broadcast_messages, 1);
    }
}
