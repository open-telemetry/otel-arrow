// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test utilities for receivers.
//!
//! These utilities are designed to make testing receivers simpler by abstracting away common
//! setup and lifecycle management.

use crate::config::ReceiverConfig;
use crate::control::{
    Controllable, NodeControlMsg, PipelineCtrlMsgReceiver, pipeline_ctrl_msg_channel,
};
use crate::error::Error;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::{Receiver, Sender};
use crate::node::NodeWithPDataSender;
use crate::receiver::ReceiverWrapper;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::testing::{CtrlMsgCounters, setup_test_runtime};
use otap_df_channel::error::RecvError;
use otap_df_telemetry::reporter::MetricsReporter;
use serde_json::Value;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::time::{Duration, Instant};
use tokio::task::LocalSet;
use tokio::time::sleep;

/// Context used during the test phase of a test.
pub struct TestContext<PData> {
    /// Sender for control messages
    control_sender: Sender<NodeControlMsg<PData>>,
}

/// Context used during the validation phase of a test (!Send context).
pub struct NotSendValidateContext<PData> {
    pdata_receiver: Receiver<PData>,
    counters: CtrlMsgCounters,
    control_sender: Sender<NodeControlMsg<PData>>,
}

/// Context used during the validation phase of a test (Send context).
pub struct SendValidateContext<PData> {
    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
    counters: CtrlMsgCounters,
}

impl<PData> TestContext<PData> {
    /// Sends a control message to the receiver.
    pub async fn send_control_msg(&self, msg: NodeControlMsg<PData>) -> Result<(), Error> {
        self.control_sender
            .send(msg)
            .await
            .map_err(|e| Error::PipelineControlMsgError {
                error: e.to_string(),
            })
    }

    /// Sends a timer tick control message.
    pub async fn send_timer_tick(&self) -> Result<(), Error> {
        self.send_control_msg(NodeControlMsg::TimerTick {}).await
    }

    /// Sends a config control message.
    pub async fn send_config(&self, config: Value) -> Result<(), Error> {
        self.send_control_msg(NodeControlMsg::Config { config })
            .await
    }

    /// Sends a shutdown control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_shutdown(&self, deadline: Instant, reason: &str) -> Result<(), Error> {
        self.send_control_msg(NodeControlMsg::Shutdown {
            deadline,
            reason: reason.to_owned(),
        })
        .await
    }

    /// Sleeps for the specified duration.
    pub async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }
}

impl<PData> NotSendValidateContext<PData> {
    /// Receives a pdata message produced by the receiver.
    pub async fn recv(&mut self) -> Result<PData, RecvError> {
        self.pdata_receiver.recv().await
    }

    /// Returns the control message counters.
    #[must_use]
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }

    /// Sends a control message to the receiver (e.g., Ack, Nack).
    ///
    /// This is useful for injecting control messages during concurrent validation,
    /// such as sending Ack/Nack messages in response to received pdata.
    pub async fn send_control_msg(&self, msg: NodeControlMsg<PData>) -> Result<(), Error> {
        self.control_sender
            .send(msg)
            .await
            .map_err(|e| Error::PipelineControlMsgError {
                error: e.to_string(),
            })
    }
}

impl<PData> SendValidateContext<PData> {
    /// Receives a pdata message produced by the receiver.
    pub async fn recv(&mut self) -> Result<PData, Error> {
        self.pdata_receiver
            .recv()
            .await
            .ok_or(Error::ChannelRecvError(RecvError::Closed))
    }

    /// Returns the control message counters.
    #[must_use]
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }
}

/// A test runtime for simplifying receiver tests.
///
/// This structure encapsulates the common setup logic needed for testing receivers,
/// including channel creation, receiver instantiation, and task management.
pub struct TestRuntime<PData> {
    /// The configuration for the receiver
    config: ReceiverConfig,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    /// Message counter for tracking processed messages
    counter: CtrlMsgCounters,

    _pd: PhantomData<PData>,
}

/// Data and operations for the test phase of a receiver.
pub struct TestPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    control_sender: Sender<NodeControlMsg<PData>>,
    receiver: ReceiverWrapper<PData>,
    counters: CtrlMsgCounters,
}

/// Data and operations for the validation phase of a receiver.
pub struct ValidationPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    counters: CtrlMsgCounters,

    pdata_receiver: Receiver<PData>,

    /// Control sender for injecting control messages during validation
    control_sender: Sender<NodeControlMsg<PData>>,

    /// Join handle for the running the receiver task
    run_receiver_handle: tokio::task::JoinHandle<()>,

    /// Join handle for the running the test task
    run_test_handle: tokio::task::JoinHandle<()>,

    // ToDo implement support for pipeline control messages in a future PR.
    #[allow(unused_variables)]
    #[allow(dead_code)]
    pipeline_ctrl_msg_receiver: PipelineCtrlMsgReceiver<PData>,
}

impl<PData: Clone + Debug + 'static> TestRuntime<PData> {
    /// Creates a new test runtime with channels of the specified capacity.
    #[must_use]
    pub fn new() -> Self {
        let config = ReceiverConfig::new("test_receiver");
        let (rt, local_tasks) = setup_test_runtime();

        Self {
            config,
            rt,
            local_tasks,
            counter: CtrlMsgCounters::new(),
            _pd: PhantomData,
        }
    }

    /// Returns the current receiver configuration.
    pub fn config(&self) -> &ReceiverConfig {
        &self.config
    }

    /// Returns the message counter.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counter.clone()
    }

    /// Sets the receiver for the test runtime and returns a test phase.
    pub fn set_receiver(self, receiver: ReceiverWrapper<PData>) -> TestPhase<PData> {
        let control_sender = receiver.control_sender();
        TestPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            receiver,
            control_sender,
            counters: self.counter,
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
    pub fn run_test<F, Fut>(mut self, f: F) -> ValidationPhase<PData>
    where
        F: FnOnce(TestContext<PData>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let (node_id, pdata_sender, pdata_receiver) = match &self.receiver {
            ReceiverWrapper::Local {
                node_id,
                runtime_config,
                ..
            } => {
                let (sender, receiver) = otap_df_channel::mpsc::Channel::new(
                    runtime_config.output_pdata_channel.capacity,
                );
                (
                    node_id.clone(),
                    Sender::Local(LocalSender::MpscSender(sender)),
                    Receiver::Local(LocalReceiver::MpscReceiver(receiver)),
                )
            }
            ReceiverWrapper::Shared {
                node_id,
                runtime_config,
                ..
            } => {
                let (sender, receiver) =
                    tokio::sync::mpsc::channel(runtime_config.output_pdata_channel.capacity);
                (
                    node_id.clone(),
                    Sender::Shared(SharedSender::MpscSender(sender)),
                    Receiver::Shared(SharedReceiver::MpscReceiver(receiver)),
                )
            }
        };
        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);

        self.receiver
            .set_pdata_sender(node_id, "".into(), pdata_sender)
            .expect("Failed to set pdata sender");

        let control_sender_for_validation = self.control_sender.clone();

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let final_metrics_reporter = metrics_reporter.clone();

        let run_receiver_handle = self.local_tasks.spawn_local(async move {
            let terminal_state = self
                .receiver
                .start(pipeline_ctrl_msg_tx, metrics_reporter)
                .await
                .expect("Receiver event loop failed");

            for snapshot in terminal_state.into_metrics() {
                let _ = final_metrics_reporter.try_report_snapshot(snapshot);
            }
        });

        let control_sender_for_test = self.control_sender.clone();
        let context = TestContext {
            control_sender: control_sender_for_test,
        };
        let run_test_handle = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
        ValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
            pdata_receiver,
            control_sender: control_sender_for_validation,
            run_receiver_handle,
            run_test_handle,
            pipeline_ctrl_msg_receiver: pipeline_ctrl_msg_rx,
        }
    }
}

impl<PData> ValidationPhase<PData> {
    /// Runs all spawned tasks to completion, then executes the validation sequentially.
    ///
    /// This is the traditional approach where validation runs after the test scenario
    /// completes. Use this when the validation needs to check final state after all
    /// test operations are done.
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
    pub fn run_validation<F, Fut, T>(self, future_fn: F) -> T
    where
        F: FnOnce(NotSendValidateContext<PData>) -> Fut,
        Fut: Future<Output = T>,
    {
        let ValidationPhase {
            rt,
            local_tasks,
            counters,
            pdata_receiver,
            run_receiver_handle,
            run_test_handle,
            pipeline_ctrl_msg_receiver: _,
            control_sender,
        } = self;

        let context = NotSendValidateContext {
            pdata_receiver,
            counters,
            control_sender,
        };

        // First run all the spawned tasks to completion
        rt.block_on(local_tasks);

        rt.block_on(run_receiver_handle)
            .expect("Receiver task failed");

        rt.block_on(run_test_handle).expect("Test task failed");

        // Then run the validation future with the test context
        rt.block_on(future_fn(context))
    }

    /// Runs validation concurrently with the test scenario.
    ///
    /// This is useful when the validation needs to interact with the test scenario
    /// in real-time, such as sending Ack/Nack messages while the scenario is running.
    /// Use this when the validation must respond to messages as they arrive, not just
    /// check final state.
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
    pub fn run_validation_concurrent<F, Fut, T>(self, future_fn: F) -> T
    where
        F: FnOnce(NotSendValidateContext<PData>) -> Fut + 'static,
        Fut: Future<Output = T> + 'static,
        T: 'static,
    {
        let context = NotSendValidateContext {
            pdata_receiver: self.pdata_receiver,
            counters: self.counters,
            control_sender: self.control_sender,
        };

        // Spawn the validation task to run concurrently with test scenario
        let validation_handle = self.local_tasks.spawn_local(future_fn(context));

        // Run all spawned tasks concurrently until completion
        self.rt.block_on(self.local_tasks);

        // Wait for receiver and test to complete
        self.rt
            .block_on(self.run_receiver_handle)
            .expect("Receiver task failed");

        self.rt
            .block_on(self.run_test_handle)
            .expect("Test task failed");

        // Return the validation result
        self.rt
            .block_on(validation_handle)
            .expect("Validation task failed")
    }
}
