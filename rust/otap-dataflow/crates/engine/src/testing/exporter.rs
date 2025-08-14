// SPDX-License-Identifier: Apache-2.0

//! Test utilities for exporters.
//!
//! These utilities are designed to make testing exporters simpler by abstracting away common
//! setup and lifecycle management.

use crate::config::ExporterConfig;
use crate::control::{
    Controllable, NodeControlMsg, PipelineCtrlMsgReceiver, pipeline_ctrl_msg_channel,
};
use crate::error::Error;
use crate::exporter::ExporterWrapper;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::{Receiver, Sender};
use crate::node::{NodeDefs, NodeType, NodeUnique, NodeWithPDataReceiver};
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::testing::{CtrlMsgCounters, create_not_send_channel, setup_test_runtime};
use otap_df_channel::error::SendError;
use serde_json::Value;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::task::LocalSet;
use tokio::time::sleep;

/// A context object that holds transmitters for use in test tasks.
pub struct TestContext<PData> {
    /// Sender for control messages
    control_tx: Sender<NodeControlMsg>,
    /// Sender for pipeline data
    pdata_tx: Sender<PData>,
    /// Message counter for tracking processed messages
    counters: CtrlMsgCounters,
}

impl<PData> Clone for TestContext<PData> {
    fn clone(&self) -> Self {
        Self {
            control_tx: self.control_tx.clone(),
            pdata_tx: self.pdata_tx.clone(),
            counters: self.counters.clone(),
        }
    }
}

impl<PData> TestContext<PData> {
    /// Creates a new TestContext with the given transmitters.
    #[must_use]
    pub fn new(
        control_tx: Sender<NodeControlMsg>,
        pdata_tx: Sender<PData>,
        counters: CtrlMsgCounters,
    ) -> Self {
        Self {
            control_tx,
            pdata_tx,
            counters,
        }
    }

    /// Returns the control message counters.
    #[must_use]
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }

    /// Sends a timer tick control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_timer_tick(&self) -> Result<(), SendError<NodeControlMsg>> {
        self.control_tx.send(NodeControlMsg::TimerTick {}).await
    }

    /// Sends a config control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_config(&self, config: Value) -> Result<(), SendError<NodeControlMsg>> {
        self.control_tx
            .send(NodeControlMsg::Config { config })
            .await
    }

    /// Sends a shutdown control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_shutdown(
        &self,
        deadline: Duration,
        reason: &str,
    ) -> Result<(), SendError<NodeControlMsg>> {
        self.control_tx
            .send(NodeControlMsg::Shutdown {
                deadline,
                reason: reason.to_owned(),
            })
            .await
    }

    /// Sends a data message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_pdata(&self, content: PData) -> Result<(), SendError<PData>> {
        self.pdata_tx.send(content).await
    }

    /// Sleeps for the specified duration.
    pub async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }
}

/// A test runtime for simplifying exporter tests.
///
/// This structure encapsulates the common setup logic needed for testing exporters,
/// including channel creation, exporter instantiation, and task management.
pub struct TestRuntime<PData> {
    /// The configuration for the exporter
    config: ExporterConfig,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    /// node defined for the test
    node: NodeUnique,

    /// Message counter for tracking processed messages
    counter: CtrlMsgCounters,

    _pd: PhantomData<PData>,
}

/// Data and operations for the test phase of an exporter.
pub struct TestPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    counters: CtrlMsgCounters,

    control_sender: Sender<NodeControlMsg>,
    pdata_sender: Sender<PData>,

    /// Join handle for the starting the exporter task
    run_exporter_handle: tokio::task::JoinHandle<Result<(), Error<PData>>>,

    pipeline_ctrl_msg_receiver: PipelineCtrlMsgReceiver,
}

/// Data and operations for the validation phase of an exporter.
pub struct ValidationPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    context: TestContext<PData>,

    /// Join handle for the running the exporter task
    run_exporter_handle: tokio::task::JoinHandle<Result<(), Error<PData>>>,

    // ToDo implement support for pipeline control messages in a future PR.
    #[allow(unused_variables)]
    #[allow(dead_code)]
    pipeline_ctrl_msg_receiver: PipelineCtrlMsgReceiver,
}

impl<PData: Clone + Debug + 'static> TestRuntime<PData> {
    /// Creates a new test runtime with channels of the specified capacity.
    #[must_use]
    pub fn new() -> Self {
        let config = ExporterConfig::new("test_exporter");
        let (rt, local_tasks) = setup_test_runtime();
        let counter = CtrlMsgCounters::new();
        let node = NodeDefs::<()>::default()
            .next(config.name.clone(), NodeType::Exporter)
            .expect("valid test config");

        Self {
            config,
            rt,
            local_tasks,
            counter,
            node,
            _pd: PhantomData,
        }
    }

    /// Returns the current exporter configuration.
    pub fn config(&self) -> &ExporterConfig {
        &self.config
    }

    /// Returns the message counter.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counter.clone()
    }

    /// Returns the test node identifier corresponding with config.name.
    pub fn test_node(&self) -> NodeUnique {
        self.node.clone()
    }

    /// Sets the exporter for the test runtime and returns the test phase.
    pub fn set_exporter(self, mut exporter: ExporterWrapper<PData>) -> TestPhase<PData> {
        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = match &exporter {
            ExporterWrapper::Local { .. } => {
                let (pdata_tx, pdata_rx) =
                    create_not_send_channel(self.config.control_channel.capacity);
                (
                    Sender::Local(LocalSender::MpscSender(pdata_tx)),
                    Receiver::Local(LocalReceiver::MpscReceiver(pdata_rx)),
                )
            }
            ExporterWrapper::Shared { .. } => {
                let (pdata_tx, pdata_rx) =
                    tokio::sync::mpsc::channel(self.config.control_channel.capacity);
                (
                    Sender::Shared(SharedSender::MpscSender(pdata_tx)),
                    Receiver::Shared(SharedReceiver::MpscReceiver(pdata_rx)),
                )
            }
        };
        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);

        exporter
            .set_pdata_receiver(self.test_node(), pdata_rx)
            .expect("Failed to set PData receiver");
        let run_exporter_handle = self
            .local_tasks
            .spawn_local(async move { exporter.start(pipeline_ctrl_msg_tx).await });
        TestPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counter.clone(),
            control_sender,
            pdata_sender: pdata_tx,
            run_exporter_handle,
            pipeline_ctrl_msg_receiver: pipeline_ctrl_msg_rx,
        }
    }
}

impl<PData: Clone + Debug + 'static> Default for TestRuntime<PData> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PData: Debug + 'static> TestPhase<PData> {
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<F, Fut>(self, f: F) -> ValidationPhase<PData>
    where
        F: FnOnce(TestContext<PData>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let context = self.create_context();
        let ctx_test = context.clone();
        self.rt.block_on(async move {
            f(ctx_test).await;
        });

        ValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            context,
            run_exporter_handle: self.run_exporter_handle,
            pipeline_ctrl_msg_receiver: self.pipeline_ctrl_msg_receiver,
        }
    }

    /// Creates a new context with the current transmitters
    fn create_context(&self) -> TestContext<PData> {
        TestContext::new(
            self.control_sender.clone(),
            self.pdata_sender.clone(),
            self.counters.clone(),
        )
    }
}

impl<PData> ValidationPhase<PData> {
    /// Runs all spawned tasks to completion and executes the provided future to validate test
    /// expectations.
    ///
    /// # Type Parameters
    ///
    /// * `F` - A function that creates a future with access to the test context.
    /// * `Fut` - The future type returned by the function.
    /// * `T` - The output type of the future.
    ///
    /// # Returns
    ///
    /// The result of the provided future.
    pub fn run_validation<F, Fut, T>(mut self, future_fn: F) -> T
    where
        F: FnOnce(TestContext<PData>, Result<(), Error<PData>>) -> Fut,
        Fut: Future<Output = T>,
    {
        // First run all the spawned tasks to completion
        let local_tasks = std::mem::take(&mut self.local_tasks);
        self.rt.block_on(local_tasks);

        let result = self
            .rt
            .block_on(self.run_exporter_handle)
            .expect("failed to join exporter task handle");

        // Then run the validation future with the test context
        self.rt.block_on(future_fn(self.context, result))
    }
}
