// SPDX-License-Identifier: Apache-2.0

//! Test utilities for receivers.
//!
//! This module provides specialized utilities for testing receiver components:
//!
//! - `ReceiverTestContext`: Provides a context for interacting with receivers during tests
//! - `ReceiverTestRuntime`: Configures and manages a single-threaded tokio runtime for receiver tests
//!
//! These utilities are designed to make testing receivers simpler by abstracting away common
//! setup and lifecycle management.

use std::fmt::Debug;
use std::marker::PhantomData;
use crate::message::ControlMsg;
use crate::receiver::{ControlMsgChannel, NotSendableEffectHandler, Receiver, SendableEffectHandler};
use crate::testing::{CtrlMsgCounters, create_not_send_channel, setup_test_runtime};
use otap_df_channel::mpsc;
use serde_json::Value;
use std::time::Duration;
use tokio::task::LocalSet;
use tokio::time::sleep;
use otap_df_channel::error::RecvError;
use crate::error::Error;

/// Context used during the test phase of a test.
pub struct TestContext {
    /// Sender for control messages
    control_sender: mpsc::Sender<ControlMsg>,
}

/// Context used during the validation phase of a test (!Send context).
pub struct NotSendValidateContext<PData> {
    pdata_receiver: mpsc::Receiver<PData>,
    counters : CtrlMsgCounters,
}

/// Context used during the validation phase of a test (Send context).
pub struct SendValidateContext<PData> {
    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
    counters : CtrlMsgCounters,
}

impl TestContext {
    /// Sends a timer tick control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_timer_tick(&self) -> Result<(), Error<ControlMsg>> {
        self.control_sender.send_async(ControlMsg::TimerTick {})
            .await.map_err(|e| Error::ChannelSendError(e))
    }

    /// Sends a config control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_config(&self, config: Value) -> Result<(), Error<ControlMsg>> {
        self.control_sender
            .send_async(ControlMsg::Config { config })
            .await.map_err(|e| Error::ChannelSendError(e))
    }

    /// Sends a shutdown control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_shutdown(&self, reason: &str) -> Result<(), Error<ControlMsg>> {
        self.control_sender
            .send_async(ControlMsg::Shutdown {
                reason: reason.to_owned(),
            })
            .await.map_err(|e| Error::ChannelSendError(e))
    }

    /// Sleeps for the specified duration.
    pub async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }
}

impl<PData> NotSendValidateContext<PData> {
    /// Receives a pdata message produced by the receiver.
    pub async fn recv(&mut self) -> Result<PData, Error<PData>> {
        self.pdata_receiver.recv().await.map_err(|e| Error::ChannelRecvError(e))
    }

    /// Returns the control message counters.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }
}

impl<PData> SendValidateContext<PData> {
    /// Receives a pdata message produced by the receiver.
    pub async fn recv(&mut self) -> Result<PData, Error<PData>> {
        self.pdata_receiver.recv().await.ok_or(Error::ChannelRecvError(RecvError::Closed))
    }

    /// Returns the control message counters.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }
}

/// A test runtime for simplifying receiver tests.
///
/// This structure encapsulates the common setup logic needed for testing receivers,
/// including channel creation, receiver instantiation, and task management.
pub struct TestRuntime<PData> {
    channel_capacity: usize,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    /// Sender for control messages
    control_tx: mpsc::Sender<ControlMsg>,
    /// Receiver for control messages
    control_rx: Option<mpsc::Receiver<ControlMsg>>,

    /// Message counter for tracking processed messages
    counter: CtrlMsgCounters,

    _pd: PhantomData<PData>
}

/// Data and operations for the test phase of a receiver (not sendable effect handler).
pub struct NonSendableTestPhase<PData> {
    name: String,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    ctrl_msg_chan: ControlMsgChannel,
    receiver: Box<dyn Receiver<PData, NotSendableEffectHandler<PData>>>,
    counters: CtrlMsgCounters,

    control_sender: mpsc::Sender<ControlMsg>,
    pdata_sender: mpsc::Sender<PData>,
    pdata_receiver: mpsc::Receiver<PData>,
}

/// Data and operations for the validation phase of a receiver (not sendable effect handler).
pub struct NotSendableValidationPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    counters: CtrlMsgCounters,

    pdata_receiver: mpsc::Receiver<PData>,
}

/// Data and operations for the validation phase of a receiver (sendable effect handler).
pub struct SendableTestPhase<PData> {
    name: String,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    ctrl_msg_chan: ControlMsgChannel,
    receiver: Box<dyn Receiver<PData, SendableEffectHandler<PData>>>,
    counters: CtrlMsgCounters,

    control_sender: mpsc::Sender<ControlMsg>,
    pdata_sender: tokio::sync::mpsc::Sender<PData>,
    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
}

/// Data and operations for the validation phase of a receiver (sendable effect handler).
pub struct SendableValidationPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    counters: CtrlMsgCounters,

    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
}

impl<PData: Clone + Debug + 'static> TestRuntime<PData> {
    /// Creates a new test runtime with channels of the specified capacity.
    pub fn new(channel_capacity: usize) -> Self {
        let (rt, local_tasks) = setup_test_runtime();
        let (control_tx, control_rx) = create_not_send_channel(channel_capacity);

        Self {
            channel_capacity,
            rt,
            local_tasks,
            control_tx,
            control_rx: Some(control_rx),
            counter: CtrlMsgCounters::new(),
            _pd: PhantomData,
        }
    }

    /// Returns the message counter.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counter.clone()
    }

    /// Initializes the test runtime with a receiver using a non-sendable effect handler.
    pub fn receiver_with_non_send_effect_handler<R>(mut self, receiver: R, name: &str) -> NonSendableTestPhase<PData>
    where
        R: Receiver<PData, NotSendableEffectHandler<PData>> + 'static,
    {
        let control_rx = self
            .control_rx
            .take()
            .expect("Control channel not initialized");
        let (pdata_sender, pdata_receiver) = mpsc::Channel::new(self.channel_capacity);

        NonSendableTestPhase {
            name: name.to_owned(),
            rt: self.rt,
            local_tasks: self.local_tasks,
            receiver: Box::new(receiver),
            ctrl_msg_chan: ControlMsgChannel::new(control_rx),
            control_sender: self.control_tx.clone(),
            counters: self.counter,
            pdata_sender,
            pdata_receiver,
        }
    }

    /// Initializes the test runtime with a receiver using a sendable effect handler.
    pub fn receiver_with_send_effect_handler<R>(mut self, receiver: R, name: &str) -> SendableTestPhase<PData>
    where
        R: Receiver<PData, SendableEffectHandler<PData>> + 'static,
    {
        let control_rx = self
            .control_rx
            .take()
            .expect("Control channel not initialized");
        let (pdata_sender, pdata_receiver) = tokio::sync::mpsc::channel(self.channel_capacity);

        SendableTestPhase {
            name: name.to_owned(),
            rt: self.rt,
            local_tasks: self.local_tasks,
            receiver: Box::new(receiver),
            ctrl_msg_chan: ControlMsgChannel::new(control_rx),
            control_sender: self.control_tx.clone(),
            counters: self.counter,
            pdata_sender,
            pdata_receiver,
        }
    }
}

impl<PData: Debug + 'static> NonSendableTestPhase<PData> {
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<F, Fut>(self, f: F) -> NotSendableValidationPhase<PData>
    where
        F: FnOnce(TestContext) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let _ = self.local_tasks.spawn_local(async move {
            self.receiver
                .start(
                    self.ctrl_msg_chan,
                    NotSendableEffectHandler::new(
                        self.name,
                        self.pdata_sender
                    ),
                )
                .await
                .expect("Receiver event loop failed");
        });

        let context = TestContext {
            control_sender: self.control_sender.clone(),
        };
        let _ = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
        NotSendableValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
            pdata_receiver: self.pdata_receiver,
        }
    }
}

impl<PData: Debug + 'static> SendableTestPhase<PData> {
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<F, Fut>(self, f: F) -> SendableValidationPhase<PData>
    where
        F: FnOnce(TestContext) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let _ = self.local_tasks.spawn_local(async move {
            self.receiver
                .start(
                    self.ctrl_msg_chan,
                    SendableEffectHandler::new(
                        self.name,
                        self.pdata_sender
                    ),
                )
                .await
                .expect("Receiver event loop failed");
        });

        let context = TestContext {
            control_sender: self.control_sender.clone(),
        };
        let _ = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
        SendableValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
            pdata_receiver: self.pdata_receiver,
        }
    }
}

impl<PData> NotSendableValidationPhase<PData> {
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
    pub fn validate<F, Fut, T>(self, future_fn: F) -> T
    where
        F: FnOnce(NotSendValidateContext<PData>) -> Fut,
        Fut: Future<Output = T>,
    {
        let context = NotSendValidateContext {
            pdata_receiver: self.pdata_receiver,
            counters: self.counters
        };

        // First run all the spawned tasks to completion
        self.rt.block_on(self.local_tasks);


        // Then run the validation future with the test context
        self.rt.block_on(future_fn(context))
    }
}

impl<PData> SendableValidationPhase<PData> {
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
    pub fn validate<F, Fut, T>(self, future_fn: F) -> T
    where
        F: FnOnce(SendValidateContext<PData>) -> Fut,
        Fut: Future<Output = T>,
    {
        let context = SendValidateContext {
            pdata_receiver: self.pdata_receiver,
            counters: self.counters
        };

        // First run all the spawned tasks to completion
        self.rt.block_on(self.local_tasks);


        // Then run the validation future with the test context
        self.rt.block_on(future_fn(context))
    }
}