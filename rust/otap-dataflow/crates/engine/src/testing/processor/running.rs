// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deterministically driven production processor and completion-dispatcher futures.

use std::{cell::RefCell, collections::HashMap, pin::Pin, rc::Rc, time::Duration};

use crate::{
    Interests, ReceivedAtNode, Unwindable,
    context::PipelineContext,
    control::{
        ControlSenders, Controllable, NodeControlMsg, PipelineCompletionMsgSender,
        pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
    },
    entity_context::{NodeTaskContext, NodeTelemetryHandle, instrument_with_node_context},
    error::Error,
    flow_metrics::build_flow_metric_state,
    local::message::{LocalReceiver, LocalSender},
    message::{Receiver, Sender},
    node::{Node, NodeId, NodeType, NodeWithPDataReceiver, NodeWithPDataSender},
    pipeline_ctrl::{
        NodeMetricHandles, PipelineCompletionMsgDispatcher, snapshot_node_metrics_with_handles,
    },
    processor::{FlowMetricHook, ProcessorWrapper},
};
use otel_arrow_dfe_channel::mpsc;
use otel_arrow_dfe_config::{PortName, policy::TelemetryPolicy};
use otel_arrow_dfe_telemetry::{metrics::MetricSetSnapshot, reporter::MetricsReporter};

/// A flow endpoint that collects supplied durations without adding wall-clock time.
pub struct FlowEndProbe<PData> {
    /// Handler configured as a duration-measuring flow endpoint.
    pub handler: crate::local::processor::EffectHandler<PData>,
    measurements: flume::Receiver<MetricSetSnapshot>,
}

impl<PData> FlowEndProbe<PData> {
    /// Instrument an existing handler without changing its routes or completion sender.
    pub fn new(
        mut handler: crate::local::processor::EffectHandler<PData>,
        context: &PipelineContext,
    ) -> Self {
        let (measurements, reporter) = MetricsReporter::create_new_and_receiver(8);
        handler.core.metrics_reporter = reporter;
        handler.set_flow_roles(
            false,
            true,
            None,
            None,
            None,
            Some(crate::flow_metrics::FlowDurationMetricSet::Basic(
                crate::flow_metrics::FlowDurationBasicMetrics::register(context),
            )),
            None,
            None,
            None,
            None,
            true,
            false,
        );
        Self {
            handler,
            measurements,
        }
    }

    /// Flush accumulated flow metrics into snapshots.
    pub fn collect(&mut self) -> Vec<MetricSetSnapshot> {
        self.handler.report_flow_metrics();
        self.measurements.try_iter().collect()
    }
}

/// Production processor and completion dispatcher with node/flow telemetry.
///
/// Poll both futures explicitly. Keep `input` open until expected completions
/// arrive; input-close draining is not supported.
pub struct RunningProcessor<PData> {
    /// Production processor run loop.
    pub runner: Pin<Box<dyn Future<Output = Result<(), Error>>>>,
    /// Production return dispatcher.
    pub dispatcher: Pin<Box<dyn Future<Output = Result<(), Error>>>>,
    /// Open upstream data channel.
    pub input: mpsc::Sender<PData>,
    /// All ports share this observation channel to preserve dispatch order.
    pub output: mpsc::Receiver<PData>,
    /// Outcomes routed to upstream node index 1.
    pub outcomes: mpsc::Receiver<NodeControlMsg<PData>>,
    /// Downstream completion sender.
    pub completion: PipelineCompletionMsgSender<PData>,
    /// Flow and component metric snapshots.
    pub measurements: flume::Receiver<MetricSetSnapshot>,
    node_metrics: Rc<RefCell<Vec<Option<NodeMetricHandles>>>>,
    _pipeline_scope: crate::entity_context::PipelineEntityScope,
}

impl<PData: ReceivedAtNode + Unwindable + FlowMetricHook + 'static> RunningProcessor<PData> {
    /// Wire a factory-created processor at index zero to a synthetic upstream at index one.
    ///
    /// `context` must have the processor's node identity.
    pub fn new(
        mut processor: ProcessorWrapper<PData>,
        context: PipelineContext,
        ports: &[PortName],
        interests: Interests,
        policy: TelemetryPolicy,
    ) -> Self {
        let node = processor.node_id();
        assert_eq!(node.index, 0);
        let pipeline_scope = crate::entity_context::set_pipeline_entity_key(
            context.metrics_registry(),
            context.register_pipeline_entity(),
        );
        let (input, input_rx) = mpsc::Channel::new(8);
        let (output_tx, output) = mpsc::Channel::new(8);
        processor
            .set_pdata_receiver(node.clone(), Receiver::Local(LocalReceiver::mpsc(input_rx)))
            .expect("wire processor test input");
        for port in ports {
            processor
                .set_pdata_sender(
                    node.clone(),
                    port.clone(),
                    Sender::Local(LocalSender::mpsc(output_tx.clone())),
                )
                .expect("wire processor test output");
        }
        processor.set_source_tagging(crate::effect_handler::SourceTagging::Enabled);

        let handle =
            NodeTelemetryHandle::new(context.metrics_registry(), context.register_node_entity());
        use crate::attributes::{ChannelImplementation, ChannelKind, ChannelMode, ChannelType};
        let input_key = context.register_node_channel_entity(
            "input".into(),
            "input".into(),
            ChannelKind::Pdata,
            ChannelMode::Local,
            ChannelType::Mpsc,
            ChannelImplementation::Internal,
        );
        handle.set_input_channel_key(input_key);
        for port in ports {
            let key = context.register_node_channel_entity(
                port.to_string().into(),
                port.to_string().into(),
                ChannelKind::Pdata,
                ChannelMode::Local,
                ChannelType::Mpsc,
                ChannelImplementation::Internal,
            );
            handle.add_output_channel_key(port.clone(), key);
        }
        let node_metrics = Rc::new(RefCell::new(vec![Some(
            crate::runtime_pipeline::make_node_metric_handles(
                &Some(handle.clone()),
                &context,
                true,
                true,
                interests,
                None,
            ),
        )]));
        let task_context = NodeTaskContext::new(
            Some(handle.entity_key()),
            Some(handle.clone()),
            Some(input_key),
            handle.output_channel_keys(),
        );
        let mut controls = ControlSenders::new();
        controls.register(
            node.clone(),
            NodeType::Processor,
            processor.control_sender(),
        );
        let (outcome_tx, outcomes) = mpsc::Channel::new(8);
        controls.register(
            NodeId {
                index: 1,
                name: "upstream".into(),
            },
            NodeType::Receiver,
            Sender::Local(LocalSender::mpsc(outcome_tx)),
        );
        let (completion, completion_rx) = pipeline_completion_msg_channel(32);
        let (measurements, reporter) = MetricsReporter::create_new_and_receiver(256);
        let dispatcher = PipelineCompletionMsgDispatcher::new(
            context.clone(),
            completion_rx,
            controls,
            node_metrics.clone(),
            reporter.clone(),
            Duration::from_secs(60),
            policy.clone(),
            Default::default(),
        );
        let mut flow = build_flow_metric_state(
            &policy,
            &HashMap::from([(node.name.to_string(), node.index)]),
            &std::collections::HashSet::from([node.index]),
            &context,
            &[],
        )
        .expect("valid processor test flow policy");
        let active = flow.is_active();
        let timing = flow.needs_timing();
        let start = flow.start_nodes.get(&node.index).copied();
        let end = flow.end_nodes.get(&node.index).copied();
        let dropped = processor
            .runtime_requirements()
            .makes_drop_decisions
            .then(|| flow.decision_candidates.get(&node.index))
            .flatten()
            .map(|candidate| {
                let mut attrs = candidate.attrs.clone();
                attrs.decision = node.name.to_string().into();
                let entity = context.metrics_registry().register_entity(attrs);
                crate::flow_metrics::FlowDroppedItemsMetrics::register(
                    &context.metric_set_registrar_for_entity(entity),
                )
            });
        let completion_tx = completion.clone();
        let runner = async move {
            let (runtime_tx, _runtime_rx) = runtime_ctrl_msg_channel(8);
            let (_trigger, signal) = crate::forced_shutdown::ForcedShutdownTrigger::pair();
            processor
                .start_with_completion_metrics(
                    runtime_tx,
                    completion_tx,
                    reporter,
                    interests,
                    None,
                    start.is_some(),
                    end.is_some(),
                    start.and_then(|i| flow.input_message_metrics[i].take()),
                    start.and_then(|i| flow.input_items_metrics[i].take()),
                    start.and_then(|i| flow.input_size_metrics[i].take()),
                    end.and_then(|i| flow.duration_metrics[i].take()),
                    end.and_then(|i| flow.output_items_metrics[i].take()),
                    end.and_then(|i| flow.output_message_metrics[i].take()),
                    end.and_then(|i| flow.output_size_metrics[i].take()),
                    dropped,
                    active,
                    timing,
                    Default::default(),
                    signal,
                    super::super::test_pipeline_runtime_services(),
                )
                .await
        };
        Self {
            runner: Box::pin(instrument_with_node_context(task_context, runner)),
            dispatcher: Box::pin(dispatcher.run()),
            input,
            output,
            outcomes,
            completion,
            measurements,
            node_metrics,
            _pipeline_scope: pipeline_scope,
        }
    }

    /// Collect completed node outcomes after driving the dispatcher to pending.
    #[must_use]
    pub fn node_snapshots(&self) -> Vec<MetricSetSnapshot> {
        snapshot_node_metrics_with_handles(&self.node_metrics)
    }
}
