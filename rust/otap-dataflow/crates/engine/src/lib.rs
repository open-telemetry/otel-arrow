// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Async Pipeline Engine

use crate::error::{ProcessorErrorKind, ReceiverErrorKind};
use crate::{
    channel_metrics::{CHANNEL_KIND_PDATA, ChannelMetricsRegistry},
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig},
    control::{AckMsg, CallData, NackMsg},
    error::Error,
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
use context::PipelineContext;
pub use linkme::distributed_slice;
use otap_df_config::{
    PortName,
    node::{DispatchStrategy, NodeUserConfig},
    pipeline::PipelineConfig,
};
use otap_df_telemetry::otel_debug;
use std::borrow::Cow;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::{collections::HashMap, sync::OnceLock};

pub mod error;
pub mod exporter;
pub mod message;
pub mod processor;
pub mod receiver;

mod attributes;
mod channel_metrics;
pub mod config;
pub mod context;
pub mod control;
pub mod effect_handler;
pub mod local;
pub mod node;
pub mod pipeline_ctrl;
mod pipeline_metrics;
pub mod runtime_pipeline;
pub mod shared;
pub mod terminal_state;
pub mod testing;

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
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ReceiverFactory<PData> {
    fn clone(&self) -> Self {
        ReceiverFactory {
            name: self.name,
            create: self.create,
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
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ProcessorFactory<PData> {
    fn clone(&self) -> Self {
        ProcessorFactory {
            name: self.name,
            create: self.create,
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
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ExporterFactory<PData> {
    fn clone(&self) -> Self {
        ExporterFactory {
            name: self.name,
            create: self.create,
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
    /// Steps:
    /// - Create all runtime nodes based on the pipeline configuration.
    /// - Analyze both the local vs shared nature of each pair of connected nodes and the nature of
    ///   the hyper-edges between them to determine the best channel type.
    /// - Assign channels to the source nodes and their destination nodes based on the previous
    ///   analysis.
    ///
    /// # Parameters
    /// - `pipeline_ctx`: The pipeline context for this build.
    /// - `config`: The pipeline configuration.
    /// - `logs_receiver`: Optional tuple of (URN, receiver) for internal logs channel.
    ///   When provided, the receiver is injected into any receiver node matching the URN,
    ///   enabling collection of logs from all threads via the channel.
    pub fn build(
        self: &PipelineFactory<PData>,
        pipeline_ctx: PipelineContext,
        config: PipelineConfig,
        logs_receiver: Option<(&str, receiver::LogsReceiver)>,
    ) -> Result<RuntimePipeline<PData>, Error> {
        let mut receivers = Vec::new();
        let mut processors = Vec::new();
        let mut exporters = Vec::new();
        let mut receiver_names = HashMap::new();
        let mut processor_names = HashMap::new();
        let mut exporter_names = HashMap::new();
        let mut node_contexts = HashMap::new();
        let mut nodes = NodeDefs::default();
        let mut channel_metrics = ChannelMetricsRegistry::default();

        let pipeline_group_id = pipeline_ctx.pipeline_group_id();
        let pipeline_id = pipeline_ctx.pipeline_id();
        let core_id = pipeline_ctx.core_id();

        otel_debug!(
            "pipeline.build.start",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id
        );

        let channel_metrics_enabled = config.pipeline_settings().telemetry.channel_metrics;

        // Create runtime nodes based on the pipeline configuration.
        // ToDo(LQ): Collect all errors instead of failing fast to provide better feedback.
        for (name, node_config) in config.node_iter() {
            let pipeline_ctx = pipeline_ctx.with_node_context(
                name.clone(),
                node_config.plugin_urn.clone(),
                node_config.kind,
            );
            let _ = node_contexts.insert(name.clone(), pipeline_ctx.clone());

            match node_config.kind {
                otap_df_config::node::NodeKind::Receiver => {
                    let mut wrapper = self.create_receiver(
                        &pipeline_ctx,
                        &mut receiver_names,
                        &mut nodes,
                        receivers.len(),
                        name.clone(),
                        node_config.clone(),
                    )?;

                    // Inject logs receiver if this is the target node
                    if let Some((target_urn, ref logs_rx)) = logs_receiver {
                        if node_config.plugin_urn.as_ref() == target_urn {
                            wrapper.set_logs_receiver(logs_rx.clone());
                        }
                    }

                    receivers.push(wrapper.with_control_channel_metrics(
                        &pipeline_ctx,
                        &mut channel_metrics,
                        channel_metrics_enabled,
                    ));
                }
                otap_df_config::node::NodeKind::Processor => {
                    let wrapper = self.create_processor(
                        &pipeline_ctx,
                        &mut processor_names,
                        &mut nodes,
                        processors.len(),
                        name.clone(),
                        node_config.clone(),
                    )?;
                    processors.push(wrapper.with_control_channel_metrics(
                        &pipeline_ctx,
                        &mut channel_metrics,
                        channel_metrics_enabled,
                    ));
                }
                otap_df_config::node::NodeKind::Exporter => {
                    let wrapper = self.create_exporter(
                        &pipeline_ctx,
                        &mut exporter_names,
                        &mut nodes,
                        exporters.len(),
                        name.clone(),
                        node_config.clone(),
                    )?;
                    exporters.push(wrapper.with_control_channel_metrics(
                        &pipeline_ctx,
                        &mut channel_metrics,
                        channel_metrics_enabled,
                    ));
                }
                otap_df_config::node::NodeKind::ProcessorChain => {
                    // ToDo(LQ): Implement processor chain optimization to eliminate intermediary channels.
                    return Err(Error::UnsupportedNodeKind {
                        kind: "ProcessorChain".into(),
                    });
                }
            }
        }

        let edges = collect_hyper_edges_runtime(&receivers, &processors);

        let mut pipeline = RuntimePipeline::new(config, receivers, processors, exporters, nodes);

        // First pass: collect all channel assignments to avoid multiple mutable borrows
        struct ChannelAssignment<PData> {
            source_id: NodeId,
            port: PortName,
            sender: Sender<PData>,
            destinations: Vec<(NodeId, Receiver<PData>)>,
        }
        let mut assignments = Vec::new();
        for hyper_edge in edges {
            let source_node_id = hyper_edge.source.clone();
            otel_debug!(
                "hyper_edge.wireup.start",
                pipeline_group_id = pipeline_group_id.as_ref(),
                pipeline_id = pipeline_id.as_ref(),
                core_id = core_id,
                source_node_id = source_node_id.name.as_ref(),
                port = hyper_edge.port.as_ref(),
                dest_node_ids = format!("{:?}", hyper_edge.destinations),
            );

            // Get source node
            let src_node =
                pipeline
                    .get_node(hyper_edge.source.index)
                    .ok_or_else(|| Error::UnknownNode {
                        node: hyper_edge.source.name.clone(),
                    })?;

            // Get destination nodes: note the order of dest_nodes matches hyper_edge.destinations
            // and preserved by select_channel_type(). The zip() below depends on both of these.
            let mut dest_nodes = Vec::with_capacity(hyper_edge.destinations.len());
            let mut dest_contexts = Vec::with_capacity(hyper_edge.destinations.len());
            for name in &hyper_edge.destinations {
                let node = processor_names
                    .get(name)
                    .map(|id| pipeline.get_node(id.index).expect("ok"))
                    .or_else(|| {
                        exporter_names
                            .get(name)
                            .map(|id| pipeline.get_node(id.index).expect("ok"))
                    })
                    .ok_or_else(|| Error::UnknownNode { node: name.clone() })?;
                dest_nodes.push(node);
                let ctx = node_contexts
                    .get(name)
                    .ok_or_else(|| Error::UnknownNode { node: name.clone() })?
                    .clone();
                dest_contexts.push(ctx);
            }

            // Select channel type
            let channel_id = format!("{}:{}", hyper_edge.source.name, hyper_edge.port);
            let source_ctx =
                node_contexts
                    .get(&hyper_edge.source.name)
                    .ok_or_else(|| Error::UnknownNode {
                        node: hyper_edge.source.name.clone(),
                    })?;
            let (pdata_sender, pdata_receivers) = Self::select_channel_type(
                src_node,
                &dest_nodes,
                NonZeroUsize::new(1000).expect("Buffer size must be non-zero"),
                channel_id.into(),
                source_ctx,
                &dest_contexts,
                &mut channel_metrics,
                channel_metrics_enabled,
            )?;

            // Prepare assignments
            let destinations = dest_nodes
                .into_iter()
                .map(|n| n.node_id())
                .zip(pdata_receivers.into_iter())
                .collect();
            assignments.push(ChannelAssignment {
                source_id: hyper_edge.source,
                port: hyper_edge.port,
                sender: pdata_sender,
                destinations,
            });

            otel_debug!(
                "hyper_edge.wireup.complete",
                pipeline_group_id = pipeline_group_id.as_ref(),
                pipeline_id = pipeline_id.as_ref(),
                core_id = core_id,
                source_node_id = source_node_id.name.as_ref(),
            );
        }

        // Second pass: perform all assignments
        for assignment in assignments {
            let src_node = pipeline
                .get_mut_node_with_pdata_sender(assignment.source_id.index)
                .ok_or_else(|| Error::UnknownNode {
                    node: assignment.source_id.name.clone(),
                })?;
            otel_debug!(
                "pdata.sender.set",
                pipeline_group_id = pipeline_group_id.as_ref(),
                pipeline_id = pipeline_id.as_ref(),
                core_id = core_id,
                node_id = assignment.source_id.name.as_ref(),
                port = assignment.port.as_ref(),
            );
            src_node.set_pdata_sender(
                assignment.source_id,
                assignment.port.clone(),
                assignment.sender,
            )?;
            for (dest, receiver) in assignment.destinations {
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
        }
        pipeline.set_channel_metrics(channel_metrics.into_handles());

        otel_debug!(
            "pipeline.build.complete",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id
        );

        Ok(pipeline)
    }

    /// Determines the best channel type from the following parameters:
    /// - Flag specifying if the channel is shared (true) or local (false).
    /// - The number of destinations connected to the channel.
    /// - The dispatch strategy for the channel (not yet supported).
    ///
    /// This function returns a tuple containing the selected sender and one receiver per
    /// destination.
    ///
    /// ToDo (LQ): Support dispatch strategies.
    fn select_channel_type(
        src_node: &dyn Node<PData>,
        dest_nodes: &Vec<&dyn Node<PData>>,
        buffer_size: NonZeroUsize,
        channel_id: Cow<'static, str>,
        source_ctx: &PipelineContext,
        dest_contexts: &[PipelineContext],
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Result<(Sender<PData>, Vec<Receiver<PData>>), Error> {
        let source_is_shared = src_node.is_shared();
        let any_dest_is_shared = dest_nodes.iter().any(|dest| dest.is_shared());
        let use_shared_channels = source_is_shared || any_dest_is_shared;
        let num_destinations = dest_nodes.len();
        debug_assert_eq!(num_destinations, dest_contexts.len());

        let channel_kind = CHANNEL_KIND_PDATA;
        let capacity = buffer_size.get() as u64;

        if channel_metrics_enabled {
            match (use_shared_channels, num_destinations > 1) {
                (true, true) => {
                    let (pdata_sender, pdata_receiver) = flume::bounded(buffer_size.get());
                    let pdata_sender = SharedSender::mpmc_with_metrics(
                        pdata_sender,
                        source_ctx,
                        channel_metrics,
                        channel_id.clone(),
                        channel_kind,
                    );
                    let pdata_receivers = dest_contexts
                        .iter()
                        .map(|ctx| {
                            let receiver = SharedReceiver::mpmc_with_metrics(
                                pdata_receiver.clone(),
                                ctx,
                                channel_metrics,
                                channel_id.clone(),
                                channel_kind,
                                capacity,
                            );
                            Receiver::Shared(receiver)
                        })
                        .collect::<Vec<_>>();
                    Ok((Sender::Shared(pdata_sender), pdata_receivers))
                }
                (true, false) => {
                    let (pdata_sender, pdata_receiver) =
                        tokio::sync::mpsc::channel::<PData>(buffer_size.get());
                    let pdata_sender = SharedSender::mpsc_with_metrics(
                        pdata_sender,
                        source_ctx,
                        channel_metrics,
                        channel_id.clone(),
                        channel_kind,
                    );
                    let ctx = dest_contexts.first().expect("dest_contexts is empty");
                    let pdata_receiver = SharedReceiver::mpsc_with_metrics(
                        pdata_receiver,
                        ctx,
                        channel_metrics,
                        channel_id.clone(),
                        channel_kind,
                        capacity,
                    );
                    Ok((
                        Sender::Shared(pdata_sender),
                        vec![Receiver::Shared(pdata_receiver)],
                    ))
                }
                (false, true) => {
                    // ToDo(LQ): Use a local SPMC channel when available.
                    let (pdata_sender, pdata_receiver) =
                        otap_df_channel::mpmc::Channel::new(buffer_size);
                    let pdata_sender = LocalSender::mpmc_with_metrics(
                        pdata_sender,
                        source_ctx,
                        channel_metrics,
                        channel_id.clone(),
                        channel_kind,
                    );
                    let pdata_receivers = dest_contexts
                        .iter()
                        .map(|ctx| {
                            let receiver = LocalReceiver::mpmc_with_metrics(
                                pdata_receiver.clone(),
                                ctx,
                                channel_metrics,
                                channel_id.clone(),
                                channel_kind,
                                capacity,
                            );
                            Receiver::Local(receiver)
                        })
                        .collect::<Vec<_>>();
                    Ok((Sender::Local(pdata_sender), pdata_receivers))
                }
                (false, false) => {
                    // ToDo(LQ): Use a local SPSC channel when available.
                    let (pdata_sender, pdata_receiver) =
                        otap_df_channel::mpsc::Channel::new(buffer_size.get());
                    let pdata_sender = LocalSender::mpsc_with_metrics(
                        pdata_sender,
                        source_ctx,
                        channel_metrics,
                        channel_id.clone(),
                        channel_kind,
                    );
                    let ctx = dest_contexts.first().expect("dest_contexts is empty");
                    let pdata_receiver = LocalReceiver::mpsc_with_metrics(
                        pdata_receiver,
                        ctx,
                        channel_metrics,
                        channel_id.clone(),
                        channel_kind,
                        capacity,
                    );
                    Ok((
                        Sender::Local(pdata_sender),
                        vec![Receiver::Local(pdata_receiver)],
                    ))
                }
            }
        } else {
            match (use_shared_channels, num_destinations > 1) {
                (true, true) => {
                    let (pdata_sender, pdata_receiver) = flume::bounded(buffer_size.get());
                    let pdata_sender = SharedSender::mpmc(pdata_sender);
                    let pdata_receivers = dest_contexts
                        .iter()
                        .map(|_| Receiver::Shared(SharedReceiver::mpmc(pdata_receiver.clone())))
                        .collect::<Vec<_>>();
                    Ok((Sender::Shared(pdata_sender), pdata_receivers))
                }
                (true, false) => {
                    let (pdata_sender, pdata_receiver) =
                        tokio::sync::mpsc::channel::<PData>(buffer_size.get());
                    Ok((
                        Sender::Shared(SharedSender::mpsc(pdata_sender)),
                        vec![Receiver::Shared(SharedReceiver::mpsc(pdata_receiver))],
                    ))
                }
                (false, true) => {
                    // ToDo(LQ): Use a local SPMC channel when available.
                    let (pdata_sender, pdata_receiver) =
                        otap_df_channel::mpmc::Channel::new(buffer_size);
                    let pdata_sender = LocalSender::mpmc(pdata_sender);
                    let pdata_receivers = dest_contexts
                        .iter()
                        .map(|_| Receiver::Local(LocalReceiver::mpmc(pdata_receiver.clone())))
                        .collect::<Vec<_>>();
                    Ok((Sender::Local(pdata_sender), pdata_receivers))
                }
                (false, false) => {
                    // ToDo(LQ): Use a local SPSC channel when available.
                    let (pdata_sender, pdata_receiver) =
                        otap_df_channel::mpsc::Channel::new(buffer_size.get());
                    Ok((
                        Sender::Local(LocalSender::mpsc(pdata_sender)),
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
        names: &mut HashMap<NodeName, NodeId>,
        nodes: &mut NodeDefs<PData, PipeNode>,
        receiver_index: usize,
        name: NodeName,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<ReceiverWrapper<PData>, Error> {
        let pipeline_group_id = pipeline_ctx.pipeline_group_id();
        let pipeline_id = pipeline_ctx.pipeline_id();
        let core_id = pipeline_ctx.core_id();

        otel_debug!(
            "receiver.create.start",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            node_id = name.as_ref(),
        );

        // Validate plugin URN structure during registration
        otap_df_config::urn::validate_plugin_urn(
            node_config.plugin_urn.as_ref(),
            otap_df_config::node::NodeKind::Receiver,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;

        let factory = self
            .get_receiver_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownReceiver {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let runtime_config = ReceiverConfig::new(name.clone());
        let create = factory.create;

        let node_id = nodes.next(
            name.clone(),
            NodeType::Receiver,
            PipeNode::new(receiver_index),
        )?;
        if names.insert(name.clone(), node_id.clone()).is_some() {
            return Err(Error::ReceiverAlreadyExists { receiver: node_id });
        }

        let receiver = create(
            (*pipeline_ctx).clone(),
            node_id.clone(),
            node_config,
            &runtime_config,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;
        if receiver.user_config().out_ports.is_empty() {
            return Err(Error::ReceiverError {
                receiver: node_id,
                kind: ReceiverErrorKind::Configuration,
                error: "The `out_ports` field is empty. This is either an invalid configuration or a receiver factory that does not provide the correct configuration.".to_owned(),
                source_detail: "".to_string(),
            });
        }

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
        names: &mut HashMap<NodeName, NodeId>,
        nodes: &mut NodeDefs<PData, PipeNode>,
        processor_index: usize,
        name: NodeName,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<ProcessorWrapper<PData>, Error> {
        let pipeline_group_id = pipeline_ctx.pipeline_group_id();
        let pipeline_id = pipeline_ctx.pipeline_id();
        let core_id = pipeline_ctx.core_id();

        otel_debug!(
            "processor.create.start",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            node_id = name.as_ref(),
        );

        // Validate plugin URN structure during registration
        otap_df_config::urn::validate_plugin_urn(
            node_config.plugin_urn.as_ref(),
            otap_df_config::node::NodeKind::Processor,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;

        let factory = self
            .get_processor_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownProcessor {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let processor_config = ProcessorConfig::new(name.clone());
        let create = factory.create;

        let node_id = nodes.next(
            name.clone(),
            NodeType::Processor,
            PipeNode::new(processor_index),
        )?;
        if names.insert(name.clone(), node_id.clone()).is_some() {
            return Err(Error::ProcessorAlreadyExists { processor: node_id });
        }

        let processor = create(
            (*pipeline_ctx).clone(),
            node_id.clone(),
            node_config.clone(),
            &processor_config,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;
        if processor.user_config().out_ports.is_empty() {
            return Err(Error::ProcessorError {
                processor: node_id,
                kind: ProcessorErrorKind::Configuration,
                error: "The `out_ports` field is empty. This is either an invalid configuration or a processor factory that does not provide the correct configuration.".to_owned(),
                source_detail: "".to_string(),
            });
        }

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
        names: &mut HashMap<NodeName, NodeId>,
        nodes: &mut NodeDefs<PData, PipeNode>,
        exporter_index: usize,
        name: NodeName,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<ExporterWrapper<PData>, Error> {
        let pipeline_group_id = pipeline_ctx.pipeline_group_id();
        let pipeline_id = pipeline_ctx.pipeline_id();
        let core_id = pipeline_ctx.core_id();

        otel_debug!(
            "exporter.create.start",
            pipeline_group_id = pipeline_group_id.as_ref(),
            pipeline_id = pipeline_id.as_ref(),
            core_id = core_id,
            node_id = name.as_ref(),
        );

        // Validate plugin URN structure during registration
        otap_df_config::urn::validate_plugin_urn(
            node_config.plugin_urn.as_ref(),
            otap_df_config::node::NodeKind::Exporter,
        )
        .map_err(|e| Error::ConfigError(Box::new(e)))?;

        let factory = self
            .get_exporter_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownExporter {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let exporter_config = ExporterConfig::new(name.clone());
        let create = factory.create;

        let node_id = nodes.next(
            name.clone(),
            NodeType::Exporter,
            PipeNode::new(exporter_index),
        )?;

        if names.insert(name.clone(), node_id.clone()).is_some() {
            return Err(Error::ExporterAlreadyExists { exporter: node_id });
        }
        let exporter = create(
            (*pipeline_ctx).clone(),
            node_id,
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

/// Represents a hyper-edge in the runtime graph, corresponding to a source node's output port,
/// its dispatch strategy, and the set of destination node ids connected to that port.
struct HyperEdgeRuntime {
    source: NodeId,

    // ToDo(LQ): Use port name for telemetry and debugging purposes.
    port: PortName,

    #[allow(dead_code)]
    dispatch_strategy: DispatchStrategy,

    // names are from the configuration, not yet resolved
    destinations: Vec<NodeName>,
}

/// Returns a vector of all hyper-edges in the runtime graph.
///
/// Each item represents a hyper-edge with source node id, port, dispatch strategy, and destination
/// node ids.
fn collect_hyper_edges_runtime<PData>(
    receivers: &[ReceiverWrapper<PData>],
    processors: &[ProcessorWrapper<PData>],
) -> Vec<HyperEdgeRuntime> {
    let mut edges = Vec::new();
    let mut nodes: Vec<&dyn Node<PData>> = Vec::new();
    nodes.extend(receivers.iter().map(|node| node as &dyn Node<PData>));
    nodes.extend(processors.iter().map(|node| node as &dyn Node<PData>));

    for node in nodes {
        let config = node.user_config();
        for (port, hyper_edge_cfg) in &config.out_ports {
            let destinations = hyper_edge_cfg
                .destinations
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            if destinations.is_empty() {
                continue;
            }
            edges.push(HyperEdgeRuntime {
                source: node.node_id(),
                port: port.clone(),
                dispatch_strategy: hyper_edge_cfg.dispatch_strategy.clone(),
                destinations,
            });
        }
    }
    edges
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_interests() {
        assert_eq!(Interests::ACKS | Interests::NACKS, Interests::ACKS_OR_NACKS);
    }
}
