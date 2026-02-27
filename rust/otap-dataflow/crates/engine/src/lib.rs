// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Async Pipeline Engine

use crate::{
    channel_metrics::{
        CHANNEL_IMPL_FLUME, CHANNEL_IMPL_INTERNAL, CHANNEL_IMPL_TOKIO, CHANNEL_KIND_PDATA,
        CHANNEL_MODE_LOCAL, CHANNEL_MODE_SHARED, CHANNEL_TYPE_MPMC, CHANNEL_TYPE_MPSC,
        ChannelMetricsRegistry, ChannelReceiverMetrics, ChannelSenderMetrics,
    },
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig},
    control::{AckMsg, CallData, NackMsg},
    effect_handler::SourceTagging,
    entity_context::{NodeTelemetryGuard, NodeTelemetryHandle, with_node_telemetry_handle},
    error::{Error, TypedError},
    exporter::ExporterWrapper,
    local::message::{LocalReceiver, LocalSender},
    message::{Receiver, Sender},
    node::{Node, NodeDefs, NodeId, NodeName, NodeType},
    processor::ProcessorWrapper,
    receiver::ReceiverWrapper,
    runtime_pipeline::{PipeNode, RuntimePipeline},
    shared::message::{SharedReceiver, SharedSender},
};
use async_trait::async_trait;
use context::NodeNameIndex;
use context::PipelineContext;
pub use linkme::distributed_slice;
use otap_df_config::{
    PipelineGroupId, PipelineId, PortName,
    node::NodeUserConfig,
    pipeline::{DispatchPolicy, PipelineConfig},
    policy::{ChannelCapacityPolicy, TelemetryPolicy},
};
use otap_df_telemetry::INTERNAL_TELEMETRY_RECEIVER_URN;
use otap_df_telemetry::InternalTelemetrySettings;
use otap_df_telemetry::{otel_debug, otel_debug_span, otel_info, otel_warn};
use std::borrow::Cow;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

pub mod error;
pub mod exporter;
pub mod message;
pub mod processor;
pub mod receiver;

mod attributes;
mod channel_metrics;
mod channel_mode;
pub mod config;
pub mod context;
pub mod control;
pub mod effect_handler;
pub mod entity_context;
pub mod local;
pub mod node;
pub mod pipeline_ctrl;
mod pipeline_metrics;
pub mod runtime_pipeline;
pub mod shared;
pub mod terminal_state;
pub mod testing;
pub mod topic;
pub mod wiring_contract;

/// Trait for factory types that expose a name.
///
/// This trait is used to define a common interface for different types of factories
/// that create instances of receivers, processors, or exporters.
pub trait NamedFactory {
    /// Returns the name of the node factory.
    fn name(&self) -> &'static str;
}

/// A factory for creating receivers.
pub struct ReceiverFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new receiver instance.
    pub create: fn(
        pipeline_ctx: PipelineContext,
        node: NodeId,
        node_config: Arc<NodeUserConfig>,
        receiver_config: &ReceiverConfig,
    ) -> Result<ReceiverWrapper<PData>, otap_df_config::error::Error>,
    /// Optional wiring constraints enforced during pipeline build.
    pub wiring_contract: wiring_contract::WiringContract,
    /// Validates the node-specific config statically, without creating the component.
    ///
    /// Use [`otap_df_config::validation::validate_typed_config`] for components with a
    /// typed `Config` struct, or [`otap_df_config::validation::no_config`] for components
    /// that accept no user configuration.
    pub validate_config: fn(config: &serde_json::Value) -> Result<(), otap_df_config::error::Error>,
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ReceiverFactory<PData> {
    fn clone(&self) -> Self {
        ReceiverFactory {
            name: self.name,
            create: self.create,
            wiring_contract: self.wiring_contract,
            validate_config: self.validate_config,
        }
    }
}

impl<PData> NamedFactory for ReceiverFactory<PData> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// A factory for creating processors.
pub struct ProcessorFactory<PData> {
    /// The name of the processor.
    pub name: &'static str,
    /// A function that creates a new processor instance.
    pub create: fn(
        pipeline: PipelineContext,
        node: NodeId,
        node_config: Arc<NodeUserConfig>,
        processor_config: &ProcessorConfig,
    ) -> Result<ProcessorWrapper<PData>, otap_df_config::error::Error>,
    /// Optional wiring constraints enforced during pipeline build.
    pub wiring_contract: wiring_contract::WiringContract,
    /// Validates the node-specific config statically, without creating the component.
    ///
    /// Use [`otap_df_config::validation::validate_typed_config`] for components with a
    /// typed `Config` struct, or [`otap_df_config::validation::no_config`] for components
    /// that accept no user configuration.
    pub validate_config: fn(config: &serde_json::Value) -> Result<(), otap_df_config::error::Error>,
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ProcessorFactory<PData> {
    fn clone(&self) -> Self {
        ProcessorFactory {
            name: self.name,
            create: self.create,
            wiring_contract: self.wiring_contract,
            validate_config: self.validate_config,
        }
    }
}

impl<PData> NamedFactory for ProcessorFactory<PData> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// A factory for creating exporter.
pub struct ExporterFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new exporter instance.
    pub create: fn(
        pipeline: PipelineContext,
        node: NodeId,
        node_config: Arc<NodeUserConfig>,
        exporter_config: &ExporterConfig,
    ) -> Result<ExporterWrapper<PData>, otap_df_config::error::Error>,
    /// Optional wiring constraints enforced during pipeline build.
    pub wiring_contract: wiring_contract::WiringContract,
    /// Validates the node-specific config statically, without creating the component.
    ///
    /// Use [`otap_df_config::validation::validate_typed_config`] for components with a
    /// typed `Config` struct, or [`otap_df_config::validation::no_config`] for components
    /// that accept no user configuration.
    pub validate_config: fn(config: &serde_json::Value) -> Result<(), otap_df_config::error::Error>,
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ExporterFactory<PData> {
    fn clone(&self) -> Self {
        ExporterFactory {
            name: self.name,
            create: self.create,
            wiring_contract: self.wiring_contract,
            validate_config: self.validate_config,
        }
    }
}

impl<PData> NamedFactory for ExporterFactory<PData> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// Returns a map of factory names to factory instances.
pub fn get_factory_map<T>(
    factory_map: &'static OnceLock<HashMap<&'static str, T>>,
    factory_slice: &'static [T],
) -> &'static HashMap<&'static str, T>
where
    T: NamedFactory + Clone,
{
    factory_map.get_or_init(|| {
        factory_slice
            .iter()
            .map(|f| (f.name(), f.clone()))
            .collect::<HashMap<&'static str, T>>()
    })
}

bitflags::bitflags! {
/// An 8-bit flags struct intended to store various intents describing
/// callers in a pipeline, e.g., detail about whether Ack and/or
/// Nack should be delivered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Interests: u8 {
    /// Acks interest
    const ACKS   = 1 << 0;

    /// Nacks interest
    const NACKS  = 1 << 1;

    /// Acks or Nacks interest subset
    const ACKS_OR_NACKS = Self::ACKS.bits() | Self::NACKS.bits();

    /// Return data
    const RETURN_DATA = 1 << 2;
}
}

/// Effect handler extensions for producers specific to data type.
#[async_trait(?Send)]
pub trait ProducerEffectHandlerExtension<PData> {
    /// Subscribe to a set of interests.
    fn subscribe_to(&self, int: Interests, ctx: CallData, data: &mut PData);
}

/// Effect handler extensions for consumers specific to data type.
#[async_trait(?Send)]
pub trait ConsumerEffectHandlerExtension<PData> {
    /// Triggers the next step of work (if any) in Ack processing.
    async fn notify_ack(&self, ack: AckMsg<PData>) -> Result<(), Error>;

    /// Triggers the next step of work (if any) in Nack processing.
    async fn notify_nack(&self, nack: NackMsg<PData>) -> Result<(), Error>;
}

/// Effect handler extension for adding message source
#[async_trait(?Send)]
pub trait MessageSourceLocalEffectHandlerExtension<PData> {
    /// Send data after tagging with the source node.
    async fn send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;
    /// Try to send data after tagging with the source node.
    fn try_send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;
    /// Send data to a specific port after tagging with the source node.
    async fn send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: PData,
    ) -> Result<(), TypedError<PData>>
    where
        P: Into<PortName> + Send + 'static;
    /// Try to send data to a specific port after tagging with the source node.
    fn try_send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: PData,
    ) -> Result<(), TypedError<PData>>
    where
        P: Into<PortName> + Send + 'static;
}

/// Send-friendly variant for use in `Send` contexts (e.g., `tokio::spawn`).
#[async_trait]
pub trait MessageSourceSharedEffectHandlerExtension<PData: Send + 'static> {
    /// Send data after tagging with the source node.
    async fn send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;
    /// Try to send data after tagging with the source node.
    fn try_send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;
    /// Send data to a specific port after tagging with the source node.
    async fn send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: PData,
    ) -> Result<(), TypedError<PData>>
    where
        P: Into<PortName> + Send + 'static;
    /// Try to send data to a specific port after tagging with the source node.
    fn try_send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: PData,
    ) -> Result<(), TypedError<PData>>
    where
        P: Into<PortName> + Send + 'static;
}
/// Builds a pipeline factory for initialization.
///
/// This function is used as a placeholder when declaring a pipeline factory with the
/// `#[factory_registry]` attribute macro. The macro will replace this placeholder with
/// proper lazy initialization using `LazyLock`.
///
/// # Example
/// ```rust,ignore
/// #[factory_registry(MyData)]
/// static FACTORY_REGISTRY: PipelineFactory<MyData> = build_factory();
/// ```
#[must_use]
pub const fn build_factory<PData: 'static + Clone>() -> PipelineFactory<PData> {
    // This function should never actually be called since the macro replaces it entirely.
    // If it is called, that indicates a bug in the macro system.
    panic!(
        "build_registry() should never be called - it's replaced by the #[factory_registry] macro"
    )
}

/// A pipeline factory.
///
/// This factory contains a registry of all the micro-factories for receivers, processors, and
/// exporters, as well as the logic for creating pipelines based on a given configuration.
pub struct PipelineFactory<PData: 'static + Clone> {
    receiver_factory_map: OnceLock<HashMap<&'static str, ReceiverFactory<PData>>>,
    processor_factory_map: OnceLock<HashMap<&'static str, ProcessorFactory<PData>>>,
    exporter_factory_map: OnceLock<HashMap<&'static str, ExporterFactory<PData>>>,
    receiver_factories: &'static [ReceiverFactory<PData>],
    processor_factories: &'static [ProcessorFactory<PData>],
    exporter_factories: &'static [ExporterFactory<PData>],
}

impl<PData: 'static + Clone + Debug> PipelineFactory<PData> {
    /// Creates a new factory registry with the given factory slices.
    #[must_use]
    pub const fn new(
        receiver_factories: &'static [ReceiverFactory<PData>],
        processor_factories: &'static [ProcessorFactory<PData>],
        exporter_factories: &'static [ExporterFactory<PData>],
    ) -> Self {
        Self {
            receiver_factory_map: OnceLock::new(),
            processor_factory_map: OnceLock::new(),
            exporter_factory_map: OnceLock::new(),
            receiver_factories,
            processor_factories,
            exporter_factories,
        }
    }

    /// Gets the receiver factory map, initializing it if necessary.
    pub fn get_receiver_factory_map(&self) -> &HashMap<&'static str, ReceiverFactory<PData>> {
        self.receiver_factory_map.get_or_init(|| {
            self.receiver_factories
                .iter()
                .map(|f| (f.name(), f.clone()))
                .collect::<HashMap<&'static str, ReceiverFactory<PData>>>()
        })
    }

    /// Gets the processor factory map, initializing it if necessary.
    pub fn get_processor_factory_map(&self) -> &HashMap<&'static str, ProcessorFactory<PData>> {
        self.processor_factory_map.get_or_init(|| {
            self.processor_factories
                .iter()
                .map(|f| (f.name(), f.clone()))
                .collect::<HashMap<&'static str, ProcessorFactory<PData>>>()
        })
    }

    /// Gets the exporter factory map, initializing it if necessary.
    pub fn get_exporter_factory_map(&self) -> &HashMap<&'static str, ExporterFactory<PData>> {
        self.exporter_factory_map.get_or_init(|| {
            self.exporter_factories
                .iter()
                .map(|f| (f.name(), f.clone()))
                .collect::<HashMap<&'static str, ExporterFactory<PData>>>()
        })
    }

    /// Builds a runtime pipeline from the given pipeline configuration.
    ///
    /// Main phases:
    /// 1) Create runtime nodes and register telemetry.
    /// 2) Plan hyper edge wiring: resolve destinations, pick channel type (shared/local,
    ///    MPSC/MPMC), create channel endpoints, and register channel metrics.
    /// 3) Apply wiring: attach senders to source ports and receivers to destination nodes,
    ///    then publish collected channel metrics on the pipeline.
    ///
    /// [config] -> [nodes] -> [hyper-edges] -> [wiring plan] -> [pipeline]
    ///
    /// The `internal_telemetry` settings are injected into any receiver with the
    /// `INTERNAL_TELEMETRY_RECEIVER_URN` plugin URN, enabling it to consume logs
    /// from the Internal Telemetry System.
    pub fn build(
        self: &PipelineFactory<PData>,
        mut pipeline_ctx: PipelineContext,
        mut config: PipelineConfig,
        channel_capacity_policy: ChannelCapacityPolicy,
        telemetry_policy: TelemetryPolicy,
        internal_telemetry: Option<InternalTelemetrySettings>,
    ) -> Result<RuntimePipeline<PData>, Error> {
        let mut receivers = Vec::new();
        let mut processors = Vec::new();
        let mut exporters = Vec::new();
        let mut build_state = BuildState::new();

        let pipeline_group_id = pipeline_ctx.pipeline_group_id();
        let pipeline_id = pipeline_ctx.pipeline_id();
        let core_id = pipeline_ctx.core_id();

        let span = otel_debug_span!(
            "pipeline.build",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id
        );
        let _enter = span.enter();

        // Remove unconnected nodes before building the pipeline.
        // Nodes that have no incoming or outgoing connections are filtered out
        // with a warning instead of causing a startup failure.
        let unconnected = config.remove_unconnected_nodes();
        for (node_id, node_kind) in &unconnected {
            let kind: Cow<'static, str> = (*node_kind).into();
            otel_info!(
                "pipeline.build.unconnected_node.removed",
                message = "Removed unconnected node from pipeline.",
                pipeline_group_id = pipeline_group_id.as_ref(),
                pipeline_id = pipeline_id.as_ref(),
                core_id = core_id,
                node_id = node_id.as_ref(),
                node_kind = kind.as_ref(),
            );
        }
        if !unconnected.is_empty() {
            otel_warn!(
                "pipeline.build.unconnected_nodes",
                message = "Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional.",
                pipeline_group_id = pipeline_group_id.as_ref(),
                pipeline_id = pipeline_id.as_ref(),
                core_id = core_id,
                removed_count = unconnected.len(),
            );
        }

        // If every node was removed, the pipeline config is broken — fail early.
        if config.nodes().is_empty() {
            return Err(Error::EmptyPipeline);
        }

        self.validate_connection_wiring_contracts(&config)?;

        let channel_metrics_enabled = telemetry_policy.channel_metrics;

        // First pass: allocate all node IDs from the build_state.
        let mut receiver_count = 0usize;
        let mut processor_count = 0usize;
        let mut exporter_count = 0usize;
        let mut node_ids: HashMap<NodeName, NodeId> = HashMap::new();

        for (name, node_config) in config.node_iter() {
            let (node_type, pipe_node) = match node_config.kind() {
                otap_df_config::node::NodeKind::Receiver => {
                    let pn = PipeNode::new(receiver_count);
                    receiver_count += 1;
                    (NodeType::Receiver, pn)
                }
                otap_df_config::node::NodeKind::Processor => {
                    let pn = PipeNode::new(processor_count);
                    processor_count += 1;
                    (NodeType::Processor, pn)
                }
                otap_df_config::node::NodeKind::Exporter => {
                    let pn = PipeNode::new(exporter_count);
                    exporter_count += 1;
                    (NodeType::Exporter, pn)
                }
                otap_df_config::node::NodeKind::ProcessorChain => {
                    return Err(Error::UnsupportedNodeKind {
                        kind: "ProcessorChain".into(),
                    });
                }
            };
            let node_id = build_state.next_node_id(name.clone(), node_type, pipe_node)?;
            let _ = node_ids.insert(name.clone(), node_id);
        }

        let node_names: NodeNameIndex = Arc::new(
            node_ids
                .iter()
                .map(|(name, id)| (name.clone(), id.clone()))
                .collect(),
        );
        pipeline_ctx.set_node_names(node_names);

        // Second pass: create runtime nodes.  Node IDs were pre-assigned above,
        // so we look them up from `node_ids` instead of calling `next_node_id`.
        // ToDo(LQ): Collect all errors instead of failing fast to provide better feedback.
        for (name, node_config) in config.node_iter() {
            let node_kind = node_config.kind();
            let node_id = node_ids.get(name).expect("allocated in first pass").clone();
            let base_ctx = pipeline_ctx.with_node_context(
                name.clone(),
                node_config.r#type.clone(),
                node_kind,
                node_config.telemetry_attributes.clone(),
            );

            match node_kind {
                otap_df_config::node::NodeKind::Receiver => {
                    // Inject internal telemetry settings into context if this is the ITR node.
                    // The ITR factory will extract these settings during construction.
                    let mut base_ctx = base_ctx;
                    if node_config.r#type.as_ref() == INTERNAL_TELEMETRY_RECEIVER_URN {
                        if let Some(ref settings) = internal_telemetry {
                            base_ctx.set_internal_telemetry(settings.clone());
                        }
                    }

                    let wrapper = self.build_node_wrapper(
                        &mut build_state,
                        &base_ctx,
                        NodeType::Receiver,
                        node_id.clone(),
                        channel_metrics_enabled,
                        || {
                            self.create_receiver(
                                &base_ctx,
                                node_id.clone(),
                                node_config.clone(),
                                channel_capacity_policy.control.node,
                                channel_capacity_policy.pdata,
                            )
                        },
                    )?;
                    receivers.push(wrapper);
                }
                otap_df_config::node::NodeKind::Processor => {
                    let wrapper = self.build_node_wrapper(
                        &mut build_state,
                        &base_ctx,
                        NodeType::Processor,
                        node_id.clone(),
                        channel_metrics_enabled,
                        || {
                            self.create_processor(
                                &base_ctx,
                                node_id.clone(),
                                node_config.clone(),
                                channel_capacity_policy.control.node,
                                channel_capacity_policy.pdata,
                            )
                        },
                    )?;
                    processors.push(wrapper);
                }
                otap_df_config::node::NodeKind::Exporter => {
                    let wrapper = self.build_node_wrapper(
                        &mut build_state,
                        &base_ctx,
                        NodeType::Exporter,
                        node_id.clone(),
                        channel_metrics_enabled,
                        || {
                            self.create_exporter(
                                &base_ctx,
                                node_id.clone(),
                                node_config.clone(),
                                channel_capacity_policy.control.node,
                                channel_capacity_policy.pdata,
                            )
                        },
                    )?;
                    exporters.push(wrapper);
                }
                otap_df_config::node::NodeKind::ProcessorChain => {
                    // ToDo(LQ): Implement processor chain optimization to eliminate intermediary channels.
                    unreachable!("rejected in first pass");
                }
            }
        }

        let edges = collect_hyper_edges_runtime_from_connections(&config, &build_state)?;

        // First pass: plan hyper-edge wiring to avoid multiple mutable borrows
        let buffer_size = NonZeroUsize::new(channel_capacity_policy.pdata)
            .expect("channel_capacity.pdata must be non-zero");
        let nodes = std::mem::take(&mut build_state.nodes);
        let mut pipeline = RuntimePipeline::new(
            config,
            receivers,
            processors,
            exporters,
            nodes,
            telemetry_policy,
        );
        let wirings = edges
            .into_iter()
            .map(|hyper_edge| {
                let resolved = hyper_edge.resolve(&build_state)?;
                resolved.into_wiring(
                    &pipeline,
                    &mut build_state,
                    buffer_size,
                    channel_metrics_enabled,
                    &pipeline_group_id,
                    &pipeline_id,
                    core_id,
                )
            })
            .collect::<Result<Vec<_>, Error>>()?;

        // Second pass: apply hyper-edge wiring
        for wiring in wirings {
            wiring.apply(&mut pipeline, &pipeline_group_id, &pipeline_id, core_id)?;
        }
        pipeline.set_channel_metrics(build_state.channel_metrics.into_handles());

        Ok(pipeline)
    }

    fn validate_connection_wiring_contracts(&self, config: &PipelineConfig) -> Result<(), Error> {
        let mut contracts_by_node: HashMap<NodeName, wiring_contract::WiringContract> =
            HashMap::new();

        for (node_name, node_config) in config.node_iter() {
            let contract = match node_config.kind() {
                otap_df_config::node::NodeKind::Receiver => {
                    let normalized = otap_df_config::node_urn::validate_plugin_urn(
                        node_config.r#type.as_ref(),
                        otap_df_config::node::NodeKind::Receiver,
                    )
                    .map_err(|e| Error::ConfigError(Box::new(e)))?;
                    self.get_receiver_factory_map()
                        .get(normalized.as_str())
                        .ok_or(Error::UnknownReceiver {
                            plugin_urn: normalized,
                        })?
                        .wiring_contract
                }
                otap_df_config::node::NodeKind::Processor => {
                    let normalized = otap_df_config::node_urn::validate_plugin_urn(
                        node_config.r#type.as_ref(),
                        otap_df_config::node::NodeKind::Processor,
                    )
                    .map_err(|e| Error::ConfigError(Box::new(e)))?;
                    self.get_processor_factory_map()
                        .get(normalized.as_str())
                        .ok_or(Error::UnknownProcessor {
                            plugin_urn: normalized,
                        })?
                        .wiring_contract
                }
                otap_df_config::node::NodeKind::Exporter => {
                    let normalized = otap_df_config::node_urn::validate_plugin_urn(
                        node_config.r#type.as_ref(),
                        otap_df_config::node::NodeKind::Exporter,
                    )
                    .map_err(|e| Error::ConfigError(Box::new(e)))?;
                    self.get_exporter_factory_map()
                        .get(normalized.as_str())
                        .ok_or(Error::UnknownExporter {
                            plugin_urn: normalized,
                        })?
                        .wiring_contract
                }
                otap_df_config::node::NodeKind::ProcessorChain => {
                    return Err(Error::UnsupportedNodeKind {
                        kind: "ProcessorChain".into(),
                    });
                }
            };

            _ = contracts_by_node.insert(node_name.as_ref().to_string().into(), contract);
        }

        let mut destinations_by_source_output: HashMap<(NodeName, PortName), HashSet<NodeName>> =
            HashMap::new();
        for connection in config.connection_iter() {
            let mut destinations: Vec<NodeName> = connection
                .to_nodes()
                .into_iter()
                .map(|node_id| node_id.as_ref().to_string().into())
                .collect();
            if destinations.is_empty() {
                continue;
            }
            destinations.sort_unstable_by(|left, right| left.as_ref().cmp(right.as_ref()));
            destinations.dedup_by(|left, right| left.as_ref() == right.as_ref());

            for source in connection.from_sources() {
                let source_name: NodeName = source.node_id().as_ref().to_string().into();
                let source_port = source.resolved_output_port();
                let entry = destinations_by_source_output
                    .entry((source_name, source_port))
                    .or_default();
                entry.extend(destinations.iter().cloned());
            }
        }

        for ((source, output), destination_set) in destinations_by_source_output {
            let Some(contract) = contracts_by_node.get(&source) else {
                return Err(Error::UnknownNode { node: source });
            };
            let mut destinations: Vec<NodeName> = destination_set.into_iter().collect();
            destinations.sort_unstable_by(|left, right| left.as_ref().cmp(right.as_ref()));
            contract.validate_output_destinations(&source, &output, &destinations)?;
        }

        Ok(())
    }

    fn build_node_wrapper<W, F>(
        &self,
        build_state: &mut BuildState<PData>,
        base_ctx: &PipelineContext,
        node_type: NodeType,
        node_id: NodeId,
        channel_metrics_enabled: bool,
        create_wrapper: F,
    ) -> Result<W, Error>
    where
        W: TelemetryWrapped,
        F: FnOnce() -> Result<W, Error>,
    {
        let node_entity_key = base_ctx.register_node_entity();
        let node_telemetry_handle =
            NodeTelemetryHandle::new(base_ctx.metrics_registry(), node_entity_key);
        // Create the guard before any fallible work so failed builds still clean up.
        let mut node_guard = Some(NodeTelemetryGuard::new(node_telemetry_handle.clone()));
        build_state.register_node(
            node_type,
            node_id,
            base_ctx.clone(),
            node_telemetry_handle.clone(),
        )?;
        let wrapper =
            with_node_telemetry_handle(node_telemetry_handle.clone(), || -> Result<W, Error> {
                let wrapper = create_wrapper()?;
                Ok(wrapper.with_control_channel_metrics(
                    base_ctx,
                    &mut build_state.channel_metrics,
                    channel_metrics_enabled,
                ))
            })?;
        Ok(wrapper
            .with_node_telemetry_guard(node_guard.take().expect("node telemetry guard missing")))
    }

    /// Determines the best channel type from the following parameters:
    /// - The number of sources connected to the channel.
    /// - The number of destinations connected to the channel.
    ///
    /// Current behavior:
    /// - multi-destination edges use competing consumers over a shared channel
    ///   (`one_of` semantics).
    /// - broadcast semantics are not yet implemented.
    ///
    /// This function returns a tuple containing one sender per source and one receiver per
    /// destination.
    fn select_channel_type(
        src_nodes: &[&dyn Node<PData>],
        dest_nodes: &[&dyn Node<PData>],
        buffer_size: NonZeroUsize,
        channel_id: Cow<'static, str>,
        source_ports: &[PortName],
        source_contexts: &[PipelineContext],
        source_telemetries: &[NodeTelemetryHandle],
        dest_contexts: &[PipelineContext],
        dest_telemetries: &[NodeTelemetryHandle],
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Result<(Vec<Sender<PData>>, Vec<Receiver<PData>>), Error> {
        let any_source_is_shared = src_nodes.iter().any(|source| source.is_shared());
        let any_dest_is_shared = dest_nodes.iter().any(|dest| dest.is_shared());
        let use_shared_channels = any_source_is_shared || any_dest_is_shared;
        let num_sources = src_nodes.len();
        let num_destinations = dest_nodes.len();
        debug_assert_eq!(num_sources, source_ports.len());
        debug_assert_eq!(num_sources, source_contexts.len());
        debug_assert_eq!(num_sources, source_telemetries.len());
        debug_assert_eq!(num_destinations, dest_contexts.len());
        debug_assert_eq!(num_destinations, dest_telemetries.len());

        let channel_kind = CHANNEL_KIND_PDATA;
        let capacity = buffer_size.get() as u64;

        if channel_metrics_enabled {
            match (use_shared_channels, num_destinations > 1) {
                (true, true) => {
                    let channel_mode = CHANNEL_MODE_SHARED;
                    let channel_type = CHANNEL_TYPE_MPMC;
                    let channel_impl = CHANNEL_IMPL_FLUME;
                    let (pdata_sender, pdata_receiver) = flume::bounded(buffer_size.get());
                    let mut pdata_senders = Vec::with_capacity(num_sources);
                    for ((ctx, telemetry), port) in source_contexts
                        .iter()
                        .zip(source_telemetries.iter())
                        .zip(source_ports.iter())
                    {
                        let sender_entity_key = ctx.register_channel_entity(
                            channel_id.clone(),
                            port.clone(),
                            channel_kind,
                            channel_mode,
                            channel_type,
                            channel_impl,
                        );
                        telemetry.add_output_channel_key(port.clone(), sender_entity_key);
                        let sender_metrics = ctx
                            .register_metric_set_for_entity::<ChannelSenderMetrics>(
                                sender_entity_key,
                            );
                        telemetry.track_metric_set(sender_metrics.metric_set_key());
                        let sender = SharedSender::mpmc_with_metrics(
                            pdata_sender.clone(),
                            channel_metrics,
                            sender_metrics,
                        );
                        pdata_senders.push(Sender::Shared(sender));
                    }
                    let pdata_receivers = dest_contexts
                        .iter()
                        .zip(dest_telemetries.iter())
                        .map(|(ctx, telemetry)| {
                            let receiver_entity_key = ctx.register_channel_entity(
                                channel_id.clone(),
                                "input".into(),
                                channel_kind,
                                channel_mode,
                                channel_type,
                                channel_impl,
                            );
                            telemetry.set_input_channel_key(receiver_entity_key);
                            let receiver_metrics = ctx
                                .register_metric_set_for_entity::<ChannelReceiverMetrics>(
                                    receiver_entity_key,
                                );
                            telemetry.track_metric_set(receiver_metrics.metric_set_key());
                            let receiver = SharedReceiver::mpmc_with_metrics(
                                pdata_receiver.clone(),
                                channel_metrics,
                                receiver_metrics,
                                capacity,
                            );
                            Receiver::Shared(receiver)
                        })
                        .collect::<Vec<_>>();
                    Ok((pdata_senders, pdata_receivers))
                }
                (true, false) => {
                    let channel_mode = CHANNEL_MODE_SHARED;
                    let channel_type = CHANNEL_TYPE_MPSC;
                    let channel_impl = CHANNEL_IMPL_TOKIO;
                    let (pdata_sender, pdata_receiver) =
                        tokio::sync::mpsc::channel::<PData>(buffer_size.get());
                    let mut pdata_senders = Vec::with_capacity(num_sources);
                    for ((ctx, telemetry), port) in source_contexts
                        .iter()
                        .zip(source_telemetries.iter())
                        .zip(source_ports.iter())
                    {
                        let sender_entity_key = ctx.register_channel_entity(
                            channel_id.clone(),
                            port.clone(),
                            channel_kind,
                            channel_mode,
                            channel_type,
                            channel_impl,
                        );
                        telemetry.add_output_channel_key(port.clone(), sender_entity_key);
                        let sender_metrics = ctx
                            .register_metric_set_for_entity::<ChannelSenderMetrics>(
                                sender_entity_key,
                            );
                        telemetry.track_metric_set(sender_metrics.metric_set_key());
                        let sender = SharedSender::mpsc_with_metrics(
                            pdata_sender.clone(),
                            channel_metrics,
                            sender_metrics,
                        );
                        pdata_senders.push(Sender::Shared(sender));
                    }
                    let ctx = dest_contexts.first().expect("dest_contexts is empty");
                    let telemetry = dest_telemetries.first().expect("dest_telemetries is empty");
                    let receiver_entity_key = ctx.register_channel_entity(
                        channel_id.clone(),
                        "input".into(),
                        channel_kind,
                        channel_mode,
                        channel_type,
                        channel_impl,
                    );
                    telemetry.set_input_channel_key(receiver_entity_key);
                    let receiver_metrics = ctx
                        .register_metric_set_for_entity::<ChannelReceiverMetrics>(
                            receiver_entity_key,
                        );
                    telemetry.track_metric_set(receiver_metrics.metric_set_key());
                    let pdata_receiver = SharedReceiver::mpsc_with_metrics(
                        pdata_receiver,
                        channel_metrics,
                        receiver_metrics,
                        capacity,
                    );
                    Ok((pdata_senders, vec![Receiver::Shared(pdata_receiver)]))
                }
                (false, true) => {
                    let channel_mode = CHANNEL_MODE_LOCAL;
                    let channel_type = CHANNEL_TYPE_MPMC;
                    let channel_impl = CHANNEL_IMPL_INTERNAL;
                    // ToDo(LQ): Use a local SPMC channel when available.
                    let (pdata_sender, pdata_receiver) =
                        otap_df_channel::mpmc::Channel::new(buffer_size);
                    let mut pdata_senders = Vec::with_capacity(num_sources);
                    for ((ctx, telemetry), port) in source_contexts
                        .iter()
                        .zip(source_telemetries.iter())
                        .zip(source_ports.iter())
                    {
                        let sender_entity_key = ctx.register_channel_entity(
                            channel_id.clone(),
                            port.clone(),
                            channel_kind,
                            channel_mode,
                            channel_type,
                            channel_impl,
                        );
                        telemetry.add_output_channel_key(port.clone(), sender_entity_key);
                        let sender_metrics = ctx
                            .register_metric_set_for_entity::<ChannelSenderMetrics>(
                                sender_entity_key,
                            );
                        telemetry.track_metric_set(sender_metrics.metric_set_key());
                        let sender = LocalSender::mpmc_with_metrics(
                            pdata_sender.clone(),
                            channel_metrics,
                            sender_metrics,
                        );
                        pdata_senders.push(Sender::Local(sender));
                    }
                    let pdata_receivers = dest_contexts
                        .iter()
                        .zip(dest_telemetries.iter())
                        .map(|(ctx, telemetry)| {
                            let receiver_entity_key = ctx.register_channel_entity(
                                channel_id.clone(),
                                "input".into(),
                                channel_kind,
                                channel_mode,
                                channel_type,
                                channel_impl,
                            );
                            telemetry.set_input_channel_key(receiver_entity_key);
                            let receiver_metrics = ctx
                                .register_metric_set_for_entity::<ChannelReceiverMetrics>(
                                    receiver_entity_key,
                                );
                            telemetry.track_metric_set(receiver_metrics.metric_set_key());
                            let receiver = LocalReceiver::mpmc_with_metrics(
                                pdata_receiver.clone(),
                                channel_metrics,
                                receiver_metrics,
                                capacity,
                            );
                            Receiver::Local(receiver)
                        })
                        .collect::<Vec<_>>();
                    Ok((pdata_senders, pdata_receivers))
                }
                (false, false) => {
                    let channel_mode = CHANNEL_MODE_LOCAL;
                    let channel_type = CHANNEL_TYPE_MPSC;
                    let channel_impl = CHANNEL_IMPL_INTERNAL;
                    // ToDo(LQ): Use a local SPSC channel when available.
                    let (pdata_sender, pdata_receiver) =
                        otap_df_channel::mpsc::Channel::new(buffer_size.get());
                    let mut pdata_senders = Vec::with_capacity(num_sources);
                    for ((ctx, telemetry), port) in source_contexts
                        .iter()
                        .zip(source_telemetries.iter())
                        .zip(source_ports.iter())
                    {
                        let sender_entity_key = ctx.register_channel_entity(
                            channel_id.clone(),
                            port.clone(),
                            channel_kind,
                            channel_mode,
                            channel_type,
                            channel_impl,
                        );
                        telemetry.add_output_channel_key(port.clone(), sender_entity_key);
                        let sender_metrics = ctx
                            .register_metric_set_for_entity::<ChannelSenderMetrics>(
                                sender_entity_key,
                            );
                        telemetry.track_metric_set(sender_metrics.metric_set_key());
                        let sender = LocalSender::mpsc_with_metrics(
                            pdata_sender.clone(),
                            channel_metrics,
                            sender_metrics,
                        );
                        pdata_senders.push(Sender::Local(sender));
                    }
                    let ctx = dest_contexts.first().expect("dest_contexts is empty");
                    let telemetry = dest_telemetries.first().expect("dest_telemetries is empty");
                    let receiver_entity_key = ctx.register_channel_entity(
                        channel_id.clone(),
                        "input".into(),
                        channel_kind,
                        channel_mode,
                        channel_type,
                        channel_impl,
                    );
                    telemetry.set_input_channel_key(receiver_entity_key);
                    let receiver_metrics = ctx
                        .register_metric_set_for_entity::<ChannelReceiverMetrics>(
                            receiver_entity_key,
                        );
                    telemetry.track_metric_set(receiver_metrics.metric_set_key());
                    let pdata_receiver = LocalReceiver::mpsc_with_metrics(
                        pdata_receiver,
                        channel_metrics,
                        receiver_metrics,
                        capacity,
                    );
                    Ok((pdata_senders, vec![Receiver::Local(pdata_receiver)]))
                }
            }
        } else {
            match (use_shared_channels, num_destinations > 1) {
                (true, true) => {
                    let channel_mode = CHANNEL_MODE_SHARED;
                    let channel_type = CHANNEL_TYPE_MPMC;
                    let channel_impl = CHANNEL_IMPL_FLUME;
                    let (pdata_sender, pdata_receiver) = flume::bounded(buffer_size.get());
                    let mut pdata_senders = Vec::with_capacity(num_sources);
                    for ((ctx, telemetry), port) in source_contexts
                        .iter()
                        .zip(source_telemetries.iter())
                        .zip(source_ports.iter())
                    {
                        let sender_entity_key = ctx.register_channel_entity(
                            channel_id.clone(),
                            port.clone(),
                            channel_kind,
                            channel_mode,
                            channel_type,
                            channel_impl,
                        );
                        telemetry.add_output_channel_key(port.clone(), sender_entity_key);
                        let sender = SharedSender::mpmc(pdata_sender.clone());
                        pdata_senders.push(Sender::Shared(sender));
                    }
                    let pdata_receivers = dest_contexts
                        .iter()
                        .zip(dest_telemetries.iter())
                        .map(|(ctx, telemetry)| {
                            let receiver_entity_key = ctx.register_channel_entity(
                                channel_id.clone(),
                                "input".into(),
                                channel_kind,
                                channel_mode,
                                channel_type,
                                channel_impl,
                            );
                            telemetry.set_input_channel_key(receiver_entity_key);
                            Receiver::Shared(SharedReceiver::mpmc(pdata_receiver.clone()))
                        })
                        .collect::<Vec<_>>();
                    Ok((pdata_senders, pdata_receivers))
                }
                (true, false) => {
                    let channel_mode = CHANNEL_MODE_SHARED;
                    let channel_type = CHANNEL_TYPE_MPSC;
                    let channel_impl = CHANNEL_IMPL_TOKIO;
                    let (pdata_sender, pdata_receiver) =
                        tokio::sync::mpsc::channel::<PData>(buffer_size.get());
                    let mut pdata_senders = Vec::with_capacity(num_sources);
                    for ((ctx, telemetry), port) in source_contexts
                        .iter()
                        .zip(source_telemetries.iter())
                        .zip(source_ports.iter())
                    {
                        let sender_entity_key = ctx.register_channel_entity(
                            channel_id.clone(),
                            port.clone(),
                            channel_kind,
                            channel_mode,
                            channel_type,
                            channel_impl,
                        );
                        telemetry.add_output_channel_key(port.clone(), sender_entity_key);
                        let sender = SharedSender::mpsc(pdata_sender.clone());
                        pdata_senders.push(Sender::Shared(sender));
                    }
                    let ctx = dest_contexts.first().expect("dest_contexts is empty");
                    let telemetry = dest_telemetries.first().expect("dest_telemetries is empty");
                    let receiver_entity_key = ctx.register_channel_entity(
                        channel_id.clone(),
                        "input".into(),
                        channel_kind,
                        channel_mode,
                        channel_type,
                        channel_impl,
                    );
                    telemetry.set_input_channel_key(receiver_entity_key);
                    Ok((
                        pdata_senders,
                        vec![Receiver::Shared(SharedReceiver::mpsc(pdata_receiver))],
                    ))
                }
                (false, true) => {
                    let channel_mode = CHANNEL_MODE_LOCAL;
                    let channel_type = CHANNEL_TYPE_MPMC;
                    let channel_impl = CHANNEL_IMPL_INTERNAL;
                    // ToDo(LQ): Use a local SPMC channel when available.
                    let (pdata_sender, pdata_receiver) =
                        otap_df_channel::mpmc::Channel::new(buffer_size);
                    let mut pdata_senders = Vec::with_capacity(num_sources);
                    for ((ctx, telemetry), port) in source_contexts
                        .iter()
                        .zip(source_telemetries.iter())
                        .zip(source_ports.iter())
                    {
                        let sender_entity_key = ctx.register_channel_entity(
                            channel_id.clone(),
                            port.clone(),
                            channel_kind,
                            channel_mode,
                            channel_type,
                            channel_impl,
                        );
                        telemetry.add_output_channel_key(port.clone(), sender_entity_key);
                        let sender = LocalSender::mpmc(pdata_sender.clone());
                        pdata_senders.push(Sender::Local(sender));
                    }
                    let pdata_receivers = dest_contexts
                        .iter()
                        .zip(dest_telemetries.iter())
                        .map(|(ctx, telemetry)| {
                            let receiver_entity_key = ctx.register_channel_entity(
                                channel_id.clone(),
                                "input".into(),
                                channel_kind,
                                channel_mode,
                                channel_type,
                                channel_impl,
                            );
                            telemetry.set_input_channel_key(receiver_entity_key);
                            Receiver::Local(LocalReceiver::mpmc(pdata_receiver.clone()))
                        })
                        .collect::<Vec<_>>();
                    Ok((pdata_senders, pdata_receivers))
                }
                (false, false) => {
                    let channel_mode = CHANNEL_MODE_LOCAL;
                    let channel_type = CHANNEL_TYPE_MPSC;
                    let channel_impl = CHANNEL_IMPL_INTERNAL;
                    // ToDo(LQ): Use a local SPSC channel when available.
                    let (pdata_sender, pdata_receiver) =
                        otap_df_channel::mpsc::Channel::new(buffer_size.get());
                    let mut pdata_senders = Vec::with_capacity(num_sources);
                    for ((ctx, telemetry), port) in source_contexts
                        .iter()
                        .zip(source_telemetries.iter())
                        .zip(source_ports.iter())
                    {
                        let sender_entity_key = ctx.register_channel_entity(
                            channel_id.clone(),
                            port.clone(),
                            channel_kind,
                            channel_mode,
                            channel_type,
                            channel_impl,
                        );
                        telemetry.add_output_channel_key(port.clone(), sender_entity_key);
                        let sender = LocalSender::mpsc(pdata_sender.clone());
                        pdata_senders.push(Sender::Local(sender));
                    }
                    let ctx = dest_contexts.first().expect("dest_contexts is empty");
                    let telemetry = dest_telemetries.first().expect("dest_telemetries is empty");
                    let receiver_entity_key = ctx.register_channel_entity(
                        channel_id.clone(),
                        "input".into(),
                        channel_kind,
                        channel_mode,
                        channel_type,
                        channel_impl,
                    );
                    telemetry.set_input_channel_key(receiver_entity_key);
                    Ok((
                        pdata_senders,
                        vec![Receiver::Local(LocalReceiver::mpsc(pdata_receiver))],
                    ))
                }
            }
        }
    }

    /// Creates a receiver node and adds it to the list of runtime nodes.
    fn create_receiver(
        &self,
        pipeline_ctx: &PipelineContext,
        node_id: NodeId,
        node_config: Arc<NodeUserConfig>,
        control_channel_capacity: usize,
        pdata_channel_capacity: usize,
    ) -> Result<ReceiverWrapper<PData>, Error> {
        let pipeline_group_id = pipeline_ctx.pipeline_group_id();
        let pipeline_id = pipeline_ctx.pipeline_id();
        let core_id = pipeline_ctx.core_id();
        let name = node_id.name.clone();

        otel_debug!(
            "receiver.create.start",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            node_id = name.as_ref(),
        );

        // Validate plugin URN structure during registration
        let normalized = otap_df_config::node_urn::validate_plugin_urn(
            node_config.r#type.as_ref(),
            otap_df_config::node::NodeKind::Receiver,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;

        let factory = self
            .get_receiver_factory_map()
            .get(normalized.as_str())
            .ok_or(Error::UnknownReceiver {
                plugin_urn: normalized,
            })?;
        let runtime_config = ReceiverConfig::with_channel_capacities(
            name.clone(),
            control_channel_capacity,
            pdata_channel_capacity,
        );
        let create = factory.create;

        let receiver = create(
            (*pipeline_ctx).clone(),
            node_id.clone(),
            node_config,
            &runtime_config,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;

        otel_debug!(
            "receiver.create.complete",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            node_id = name.as_ref(),
        );

        Ok(receiver)
    }

    /// Creates a processor node and adds it to the list of runtime nodes.
    fn create_processor(
        &self,
        pipeline_ctx: &PipelineContext,
        node_id: NodeId,
        node_config: Arc<NodeUserConfig>,
        control_channel_capacity: usize,
        pdata_channel_capacity: usize,
    ) -> Result<ProcessorWrapper<PData>, Error> {
        let pipeline_group_id = pipeline_ctx.pipeline_group_id();
        let pipeline_id = pipeline_ctx.pipeline_id();
        let core_id = pipeline_ctx.core_id();
        let name = node_id.name.clone();

        otel_debug!(
            "processor.create.start",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            node_id = name.as_ref(),
        );

        // Validate plugin URN structure during registration
        let normalized = otap_df_config::node_urn::validate_plugin_urn(
            node_config.r#type.as_ref(),
            otap_df_config::node::NodeKind::Processor,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;

        let factory = self
            .get_processor_factory_map()
            .get(normalized.as_str())
            .ok_or(Error::UnknownProcessor {
                plugin_urn: normalized,
            })?;
        let processor_config = ProcessorConfig::with_channel_capacities(
            name.clone(),
            control_channel_capacity,
            pdata_channel_capacity,
        );
        let create = factory.create;

        let processor = create(
            (*pipeline_ctx).clone(),
            node_id.clone(),
            node_config.clone(),
            &processor_config,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;

        otel_debug!(
            "processor.create.complete",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            node_id = name.as_ref(),
        );

        Ok(processor)
    }

    /// Creates an exporter node and adds it to the list of runtime nodes.
    fn create_exporter(
        &self,
        pipeline_ctx: &PipelineContext,
        node_id: NodeId,
        node_config: Arc<NodeUserConfig>,
        control_channel_capacity: usize,
        pdata_channel_capacity: usize,
    ) -> Result<ExporterWrapper<PData>, Error> {
        let pipeline_group_id = pipeline_ctx.pipeline_group_id();
        let pipeline_id = pipeline_ctx.pipeline_id();
        let core_id = pipeline_ctx.core_id();
        let name = node_id.name.clone();

        otel_debug!(
            "exporter.create.start",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            node_id = name.as_ref(),
        );

        // Validate plugin URN structure during registration
        let normalized = otap_df_config::node_urn::validate_plugin_urn(
            node_config.r#type.as_ref(),
            otap_df_config::node::NodeKind::Exporter,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;

        let factory = self
            .get_exporter_factory_map()
            .get(normalized.as_str())
            .ok_or(Error::UnknownExporter {
                plugin_urn: normalized,
            })?;
        let exporter_config = ExporterConfig::with_channel_capacities(
            name.clone(),
            control_channel_capacity,
            pdata_channel_capacity,
        );
        let create = factory.create;

        let exporter = create(
            (*pipeline_ctx).clone(),
            node_id.clone(),
            node_config,
            &exporter_config,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;

        otel_debug!(
            "exporter.create.complete",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            node_id = name.as_ref(),
        );

        Ok(exporter)
    }
}

trait TelemetryWrapped: Sized {
    fn with_control_channel_metrics(
        self,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Self;
    fn with_node_telemetry_guard(self, guard: NodeTelemetryGuard) -> Self;
}

impl<PData> TelemetryWrapped for ReceiverWrapper<PData> {
    fn with_control_channel_metrics(
        self,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Self {
        ReceiverWrapper::with_control_channel_metrics(
            self,
            pipeline_ctx,
            channel_metrics,
            channel_metrics_enabled,
        )
    }

    fn with_node_telemetry_guard(self, guard: NodeTelemetryGuard) -> Self {
        ReceiverWrapper::with_node_telemetry_guard(self, guard)
    }
}

impl<PData> TelemetryWrapped for ProcessorWrapper<PData> {
    fn with_control_channel_metrics(
        self,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Self {
        ProcessorWrapper::with_control_channel_metrics(
            self,
            pipeline_ctx,
            channel_metrics,
            channel_metrics_enabled,
        )
    }

    fn with_node_telemetry_guard(self, guard: NodeTelemetryGuard) -> Self {
        ProcessorWrapper::with_node_telemetry_guard(self, guard)
    }
}

impl<PData> TelemetryWrapped for ExporterWrapper<PData> {
    fn with_control_channel_metrics(
        self,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Self {
        ExporterWrapper::with_control_channel_metrics(
            self,
            pipeline_ctx,
            channel_metrics,
            channel_metrics_enabled,
        )
    }

    fn with_node_telemetry_guard(self, guard: NodeTelemetryGuard) -> Self {
        ExporterWrapper::with_node_telemetry_guard(self, guard)
    }
}

struct NodeRegistration {
    node_id: NodeId,
    node_type: NodeType,
    context: PipelineContext,
    telemetry: NodeTelemetryHandle,
}

struct BuildState<PData> {
    nodes: NodeDefs<PData, PipeNode>,
    registry: HashMap<NodeName, NodeRegistration>,
    channel_metrics: ChannelMetricsRegistry,
}

impl<PData> BuildState<PData> {
    fn new() -> Self {
        Self {
            nodes: NodeDefs::default(),
            registry: HashMap::new(),
            channel_metrics: ChannelMetricsRegistry::default(),
        }
    }

    fn next_node_id(
        &mut self,
        name: NodeName,
        node_type: NodeType,
        inner: PipeNode,
    ) -> Result<NodeId, Error> {
        self.nodes.next(name, node_type, inner)
    }

    fn register_node(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        context: PipelineContext,
        telemetry: NodeTelemetryHandle,
    ) -> Result<(), Error> {
        if self.registry.contains_key(&node_id.name) {
            return Err(match node_type {
                NodeType::Receiver => Error::ReceiverAlreadyExists { receiver: node_id },
                NodeType::Processor => Error::ProcessorAlreadyExists { processor: node_id },
                NodeType::Exporter => Error::ExporterAlreadyExists { exporter: node_id },
            });
        }

        let _ = self.registry.insert(
            node_id.name.clone(),
            NodeRegistration {
                node_id,
                node_type,
                context,
                telemetry,
            },
        );
        Ok(())
    }

    fn registration(&self, name: &NodeName) -> Result<&NodeRegistration, Error> {
        self.registry
            .get(name)
            .ok_or_else(|| Error::UnknownNode { node: name.clone() })
    }

    fn node_context(&self, name: &NodeName) -> Result<PipelineContext, Error> {
        Ok(self.registration(name)?.context.clone())
    }

    fn node_telemetry(&self, name: &NodeName) -> Result<NodeTelemetryHandle, Error> {
        Ok(self.registration(name)?.telemetry.clone())
    }

    fn resolve_destination_id(&self, name: &NodeName) -> Result<NodeId, Error> {
        let registration = self.registration(name)?;
        match registration.node_type {
            NodeType::Processor | NodeType::Exporter => Ok(registration.node_id.clone()),
            NodeType::Receiver => Err(Error::UnknownNode { node: name.clone() }),
        }
    }
}

/// Represents a source endpoint for a hyper-edge in the runtime graph.
struct NodeIdPortName {
    node_id: NodeId,
    port: PortName,
}

/// Represents the channel wiring for a hyper-edge in the runtime graph.
struct HyperEdgeWiring<PData> {
    /// All the source endpoints for this hyper-edge.
    sources: Vec<NodeIdPortName>,
    /// The senders assigned to the sources.
    senders: Vec<Sender<PData>>,
    /// The destinations and their assigned receivers.
    destinations: Vec<(NodeId, Receiver<PData>)>,
}

impl<PData> HyperEdgeWiring<PData>
where
    PData: 'static + Clone + Debug,
{
    fn apply(
        self,
        pipeline: &mut RuntimePipeline<PData>,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
        core_id: usize,
    ) -> Result<(), Error> {
        debug_assert_eq!(self.sources.len(), self.senders.len());

        // When there are multiple sources sharing a channel to the same
        // destination(s), mark each source so it tags outgoing messages with
        // its node id.  This lets the destination distinguish which source
        // sent each message.
        let multi_source = self.sources.len() > 1;

        for (source, sender) in self.sources.into_iter().zip(self.senders.into_iter()) {
            let src_node = pipeline
                .get_mut_node_with_pdata_sender(source.node_id.index)
                .ok_or_else(|| Error::UnknownNode {
                    node: source.node_id.name.clone(),
                })?;
            if multi_source {
                src_node.set_source_tagging(SourceTagging::Enabled);
            }
            otel_debug!(
                "pdata.sender.set",
                pipeline_group_id = pipeline_group_id.as_ref(),
                pipeline_id = pipeline_id.as_ref(),
                core_id = core_id,
                node_id = source.node_id.name.as_ref(),
                port = source.port.as_ref(),
            );
            src_node.set_pdata_sender(source.node_id, source.port, sender)?;
        }
        for (dest, receiver) in self.destinations {
            let dest_node = pipeline
                .get_mut_node_with_pdata_receiver(dest.index)
                .ok_or_else(|| Error::UnknownNode {
                    node: dest.name.clone(),
                })?;
            otel_debug!(
                "pdata.receiver.set",
                pipeline_group_id = pipeline_group_id.as_ref(),
                pipeline_id = pipeline_id.as_ref(),
                core_id = core_id,
                node_id = dest.name.as_ref(),
            );

            dest_node.set_pdata_receiver(dest, receiver)?;
        }
        Ok(())
    }
}

/// Represents a hyper-edge in the runtime graph, corresponding to one or more source ports,
/// its dispatch policy, and the set of destination node ids connected to those ports.
struct HyperEdgeRuntime {
    sources: Vec<NodeIdPortName>,

    #[allow(dead_code)]
    dispatch_policy: DispatchPolicy,

    // names are from the configuration, not yet resolved
    destinations: Vec<NodeName>,
}

/// Represents a hyper-edge with resolved destination node IDs.
struct ResolvedHyperEdgeRuntime {
    sources: Vec<NodeIdPortName>,
    destinations: Vec<NodeId>,
    dispatch_policy: DispatchPolicy,
    source_ids_display: String,
    destination_ids_display: String,
}

#[derive(Hash, PartialEq, Eq)]
struct HyperEdgeKey {
    dispatch_policy: std::mem::Discriminant<DispatchPolicy>,
    destinations: Vec<NodeName>,
}
impl HyperEdgeRuntime {
    fn resolve<PData>(
        self,
        build_state: &BuildState<PData>,
    ) -> Result<ResolvedHyperEdgeRuntime, Error> {
        let destinations = self
            .destinations
            .iter()
            .map(|name| build_state.resolve_destination_id(name))
            .collect::<Result<Vec<_>, Error>>()?;
        let source_ids_display = self
            .sources
            .iter()
            .map(|source| format!("{}:{}", source.node_id.name, source.port))
            .collect::<Vec<_>>()
            .join(", ");
        let destination_ids_display = destinations
            .iter()
            .map(|dest| dest.name.as_ref().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Ok(ResolvedHyperEdgeRuntime {
            sources: self.sources,
            destinations,
            dispatch_policy: self.dispatch_policy,
            source_ids_display,
            destination_ids_display,
        })
    }
}

impl ResolvedHyperEdgeRuntime {
    fn channel_id(&self) -> Cow<'static, str> {
        let sources = self
            .sources
            .iter()
            .map(|source| format!("{}:{}", source.node_id.name, source.port))
            .collect::<Vec<_>>();
        let destinations = self
            .destinations
            .iter()
            .map(|dest| dest.name.as_ref().to_string())
            .collect::<Vec<_>>();
        let signature = format!(
            "src:[{}]|dst:[{}]|dispatch:{}",
            sources.join(","),
            destinations.join(","),
            dispatch_policy_label(&self.dispatch_policy)
        );
        let hash = stable_hash64(&signature);
        format!("hyperedge:{:016x}", hash).into()
    }

    fn into_wiring<PData>(
        self,
        pipeline: &RuntimePipeline<PData>,
        build_state: &mut BuildState<PData>,
        buffer_size: NonZeroUsize,
        channel_metrics_enabled: bool,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
        core_id: usize,
    ) -> Result<HyperEdgeWiring<PData>, Error>
    where
        PData: 'static + Clone + Debug,
    {
        let channel_id = self.channel_id();
        let ResolvedHyperEdgeRuntime {
            sources,
            destinations,
            dispatch_policy: _,
            source_ids_display,
            destination_ids_display,
        } = self;
        let span = otel_debug_span!(
            "hyper_edge.wireup",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            source_ids = source_ids_display,
            dest_ids = destination_ids_display
        );
        let _enter = span.enter();

        let mut source_nodes = Vec::with_capacity(sources.len());
        let mut source_ports = Vec::with_capacity(sources.len());
        let mut source_contexts = Vec::with_capacity(sources.len());
        let mut source_telemetries = Vec::with_capacity(sources.len());
        for source in &sources {
            let src_node =
                pipeline
                    .get_node(source.node_id.index)
                    .ok_or_else(|| Error::UnknownNode {
                        node: source.node_id.name.clone(),
                    })?;
            source_nodes.push(src_node);
            source_ports.push(source.port.clone());
            source_contexts.push(build_state.node_context(&source.node_id.name)?);
            source_telemetries.push(build_state.node_telemetry(&source.node_id.name)?);
        }

        // Get destination nodes: note the order of dest_nodes matches destinations and is
        // preserved by select_channel_type(). The zip() below depends on both of these.
        let mut dest_nodes = Vec::with_capacity(destinations.len());
        let mut dest_contexts = Vec::with_capacity(destinations.len());
        let mut dest_telemetries = Vec::with_capacity(destinations.len());
        for node_id in &destinations {
            let node = pipeline
                .get_node(node_id.index)
                .ok_or_else(|| Error::UnknownNode {
                    node: node_id.name.clone(),
                })?;
            dest_nodes.push(node);
            dest_contexts.push(build_state.node_context(&node_id.name)?);
            dest_telemetries.push(build_state.node_telemetry(&node_id.name)?);
        }

        let (pdata_senders, pdata_receivers) = PipelineFactory::<PData>::select_channel_type(
            &source_nodes,
            &dest_nodes,
            buffer_size,
            channel_id,
            &source_ports,
            &source_contexts,
            &source_telemetries,
            &dest_contexts,
            &dest_telemetries,
            &mut build_state.channel_metrics,
            channel_metrics_enabled,
        )?;

        let destinations = destinations.into_iter().zip(pdata_receivers).collect();
        Ok(HyperEdgeWiring {
            sources,
            senders: pdata_senders,
            destinations,
        })
    }
}

/// Builds hyper-edges directly from the top-level `connections` section.
fn collect_hyper_edges_runtime_from_connections<PData>(
    config: &PipelineConfig,
    build_state: &BuildState<PData>,
) -> Result<Vec<HyperEdgeRuntime>, Error> {
    let mut edges: Vec<HyperEdgeRuntime> = Vec::new();
    let mut edge_index: HashMap<HyperEdgeKey, Vec<usize>> = HashMap::new();

    for connection in config.connection_iter() {
        let mut destinations: Vec<NodeName> = connection
            .to_nodes()
            .into_iter()
            .map(|node_id| node_id.as_ref().to_string().into())
            .collect();
        if destinations.is_empty() {
            continue;
        }
        destinations.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
        destinations.dedup_by(|a, b| a.as_ref() == b.as_ref());

        let mut sources = Vec::new();
        for source in connection.from_sources() {
            let source_name: NodeName = source.node_id().as_ref().to_string().into();
            let registration = build_state.registration(&source_name)?;
            if !matches!(
                registration.node_type,
                NodeType::Receiver | NodeType::Processor
            ) {
                return Err(Error::UnknownNode { node: source_name });
            }
            sources.push(NodeIdPortName {
                node_id: registration.node_id.clone(),
                port: source.resolved_output_port(),
            });
        }
        if sources.is_empty() {
            continue;
        }
        sources.sort_by(|left, right| {
            let left_key = (left.node_id.name.as_ref(), left.port.as_ref());
            let right_key = (right.node_id.name.as_ref(), right.port.as_ref());
            left_key.cmp(&right_key)
        });
        sources.dedup_by(|left, right| {
            left.node_id.index == right.node_id.index && left.port.as_ref() == right.port.as_ref()
        });

        let dispatch_policy = connection.effective_dispatch_policy();
        let key = HyperEdgeKey {
            dispatch_policy: std::mem::discriminant(&dispatch_policy),
            destinations: destinations.clone(),
        };

        let mut match_index = None;
        if let Some(indexes) = edge_index.get(&key) {
            'candidate: for &index in indexes {
                let edge = &edges[index];
                for source in &sources {
                    if edge.sources.iter().any(|existing| {
                        existing.node_id.index == source.node_id.index
                            && existing.port.as_ref() != source.port.as_ref()
                    }) {
                        continue 'candidate;
                    }
                }
                match_index = Some(index);
                break;
            }
        }

        if let Some(index) = match_index {
            edges[index].sources.extend(sources);
        } else {
            edges.push(HyperEdgeRuntime {
                sources,
                dispatch_policy,
                destinations,
            });
            edge_index.entry(key).or_default().push(edges.len() - 1);
        }
    }

    for edge in &mut edges {
        edge.sources.sort_by(|left, right| {
            let left_key = (left.node_id.name.as_ref(), left.port.as_ref());
            let right_key = (right.node_id.name.as_ref(), right.port.as_ref());
            left_key.cmp(&right_key)
        });
        edge.sources.dedup_by(|left, right| {
            left.node_id.index == right.node_id.index && left.port.as_ref() == right.port.as_ref()
        });
    }

    Ok(edges)
}

const fn dispatch_policy_label(policy: &DispatchPolicy) -> &'static str {
    match policy {
        DispatchPolicy::OneOf => "one_of",
        DispatchPolicy::Broadcast => "broadcast",
    }
}

fn stable_hash64(value: &str) -> u64 {
    // FNV-1a 64-bit hash for a deterministic, dependency-free channel id.
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_interests() {
        assert_eq!(Interests::ACKS | Interests::NACKS, Interests::ACKS_OR_NACKS);
    }
}
