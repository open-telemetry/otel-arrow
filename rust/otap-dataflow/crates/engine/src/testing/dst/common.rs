// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::clock;
use crate::context::{ControllerContext, PipelineContext};
use crate::control::{ControlSenders, Frame, NodeControlMsg, RouteData, runtime_ctrl_msg_channel};
use crate::entity_context::set_pipeline_entity_key;
use crate::message::{Receiver, Sender};
use crate::pipeline_ctrl::{NodeMetricHandles, RuntimeCtrlMsgManager};
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::testing::setup_test_runtime;
use crate::{Interests, ReceivedAtNode, Unwindable};
use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_config::policy::TelemetryPolicy;
use otap_df_config::{MetricLevel, PipelineGroupId, PipelineId};
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::InternalTelemetrySystem;
use smallvec::smallvec;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub(super) struct DstPData {
    pub(super) id: u64,
    pub(super) frames: Vec<Frame>,
    pub(super) payload: Option<String>,
    pub(super) received_at: Vec<usize>,
}

impl DstPData {
    pub(super) fn new(id: u64) -> Self {
        Self {
            id,
            frames: Vec::new(),
            payload: Some(format!("payload-{id}")),
            received_at: Vec::new(),
        }
    }

    pub(super) fn with_frames(id: u64, frames: Vec<Frame>) -> Self {
        let mut pdata = Self::new(id);
        pdata.frames = frames;
        pdata
    }
}

impl ReceivedAtNode for DstPData {
    fn received_at_node(&mut self, node_id: usize, _node_interests: Interests) {
        self.received_at.push(node_id);
    }
}

impl Unwindable for DstPData {
    fn has_frames(&self) -> bool {
        !self.frames.is_empty()
    }

    fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop()
    }

    fn drop_payload(&mut self) {
        self.payload = None;
    }
}

pub(super) fn frame(node_id: usize, interests: Interests, tag: u64) -> Frame {
    Frame {
        node_id,
        interests,
        route: RouteData {
            calldata: smallvec![tag.into()],
            entry_time_ns: clock::nanos_since_birth(),
            output_port_index: 0,
        },
    }
}

pub(super) fn create_mock_control_sender<PData>(
    capacity: usize,
) -> (
    Sender<NodeControlMsg<PData>>,
    Receiver<NodeControlMsg<PData>>,
) {
    let (tx, rx) = tokio::sync::mpsc::channel(capacity);
    (
        Sender::Shared(SharedSender::mpsc(tx)),
        Receiver::Shared(SharedReceiver::mpsc(rx)),
    )
}

pub(super) fn empty_node_metric_handles() -> Rc<RefCell<Vec<Option<NodeMetricHandles>>>> {
    Rc::new(RefCell::new(Vec::new()))
}

pub(super) fn build_manager<PData>(
    pipeline_capacity: usize,
    control_senders: ControlSenders<PData>,
) -> (
    RuntimeCtrlMsgManager<PData>,
    crate::control::RuntimeCtrlMsgSender<PData>,
    crate::entity_context::PipelineEntityScope,
    PipelineContext,
) {
    let (pipeline_tx, pipeline_rx) = runtime_ctrl_msg_channel(pipeline_capacity);

    let metrics_system = InternalTelemetrySystem::default();
    let metrics_reporter = metrics_system.reporter();
    let observed_state_store =
        ObservedStateStore::new(&ObservedStateSettings::default(), metrics_system.registry());
    let pipeline_group_id: PipelineGroupId = Default::default();
    let pipeline_id: PipelineId = Default::default();
    let controller_context = ControllerContext::new(metrics_system.registry());
    let pipeline_context = PipelineContext::new(
        controller_context,
        pipeline_group_id.clone(),
        pipeline_id.clone(),
        0,
        1,
        0,
    );

    let pipeline_entity_key = pipeline_context.register_pipeline_entity();
    let pipeline_entity_guard =
        set_pipeline_entity_key(pipeline_context.metrics_registry(), pipeline_entity_key);

    let manager = RuntimeCtrlMsgManager::new(
        otap_df_config::DeployedPipelineKey {
            pipeline_group_id,
            pipeline_id,
            core_id: 0,
        },
        pipeline_context.clone(),
        pipeline_rx,
        control_senders,
        observed_state_store.reporter(SendPolicy::default()),
        metrics_reporter,
        TelemetryPolicy {
            runtime_metrics: MetricLevel::None,
            pipeline_metrics: false,
            tokio_metrics: false,
        },
        Vec::new(),
        empty_node_metric_handles(),
    );

    (
        manager,
        pipeline_tx,
        pipeline_entity_guard,
        pipeline_context,
    )
}

pub(super) async fn yield_cycles(count: usize) {
    for _ in 0..count {
        tokio::task::yield_now().await;
    }
}

pub(super) async fn recv_controls<PData: Clone>(
    receiver: &mut Receiver<NodeControlMsg<PData>>,
) -> Vec<NodeControlMsg<PData>> {
    let mut msgs = Vec::new();
    while let Ok(msg) = receiver.try_recv() {
        msgs.push(msg);
    }
    msgs
}

pub(super) async fn recv_until<PData, F>(
    receiver: &mut Receiver<NodeControlMsg<PData>>,
    timeout_duration: Duration,
    mut done: F,
    timeout_context: &str,
) -> Vec<NodeControlMsg<PData>>
where
    PData: std::fmt::Debug,
    F: FnMut(&[NodeControlMsg<PData>]) -> bool,
{
    let deadline = tokio::time::Instant::now() + timeout_duration;
    let mut msgs = Vec::new();
    while !done(&msgs) {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        match timeout(remaining, receiver.recv()).await {
            Ok(Ok(msg)) => msgs.push(msg),
            Ok(Err(_)) => panic!("{timeout_context}: control receiver closed early"),
            Err(_) => panic!("{timeout_context}: observed msgs={msgs:?}"),
        }
    }
    msgs
}

pub(super) fn setup_dst_runtime() -> (tokio::runtime::Runtime, tokio::task::LocalSet) {
    setup_test_runtime()
}
