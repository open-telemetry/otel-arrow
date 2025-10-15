// SPDX-License-Identifier: Apache-2.0

//! Test utilities for receivers.
//!
//! These utilities are designed to make testing receivers simpler by abstracting away common
//! setup and lifecycle management.

use crate::config::ReceiverConfig;
use crate::error::Error;
use crate::message::{ControlMsg, Receiver, Sender};
use crate::receiver::ReceiverWrapper;
use crate::testing::{CtrlMsgCounters, setup_test_runtime};
use otap_df_channel::error::RecvError;
use serde_json::Value;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::task::LocalSet;
use tokio::time::sleep;

/// Context used during the test phase of a test.
pub struct TestContext {
    /// Sender for control messages
    control_sender: Sender<ControlMsg>,
}

/// Context used during the validation phase of a test (!Send context).
pub struct NotSendValidateContext<PData> {
    pdata_receiver: Receiver<PData>,
    counters: CtrlMsgCounters,
}

/// Context used during the validation phase of a test (Send context).
pub struct SendValidateContext<PData> {
    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
    counters: CtrlMsgCounters,
}

impl TestContext {
    /// Sends a timer tick control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_timer_tick(&self) -> Result<(), Error<ControlMsg>> {
        self.control_sender
            .send(ControlMsg::TimerTick {})
            .await
            .map_err(Error::ChannelSendError)
    }

    /// Sends a config control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_config(&self, config: Value) -> Result<(), Error<ControlMsg>> {
        self.control_sender
            .send(ControlMsg::Config { config })
            .await
            .map_err(Error::ChannelSendError)
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
    ) -> Result<(), Error<ControlMsg>> {
        self.control_sender
            .send(ControlMsg::Shutdown {
                deadline,
                reason: reason.to_owned(),
            })
            .await
            .map_err(Error::ChannelSendError)
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
}

impl<PData> SendValidateContext<PData> {
    /// Receives a pdata message produced by the receiver.
    pub async fn recv(&mut self) -> Result<PData, Error<PData>> {
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

    control_sender: Sender<ControlMsg>,
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

    /// Join handle for the running the receiver task
    run_receiver_handle: tokio::task::JoinHandle<()>,

    /// Join handle for the running the test task
    run_test_handle: tokio::task::JoinHandle<()>,
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
        F: FnOnce(TestContext) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let pdata_receiver = self.receiver.take_pdata_receiver();
        let run_receiver_handle = self.local_tasks.spawn_local(async move {
            self.receiver
                .start()
                .await
                .expect("Receiver event loop failed");
        });

        let context = TestContext {
            control_sender: self.control_sender,
        };
        let run_test_handle = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
        ValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
            pdata_receiver,
            run_receiver_handle,
            run_test_handle,
        }
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
    pub fn run_validation<F, Fut, T>(self, future_fn: F) -> T
    where
        F: FnOnce(NotSendValidateContext<PData>) -> Fut,
        Fut: Future<Output = T>,
    {
        let context = NotSendValidateContext {
            pdata_receiver: self.pdata_receiver,
            counters: self.counters,
        };

        // First run all the spawned tasks to completion
        self.rt.block_on(self.local_tasks);

        self.rt
            .block_on(self.run_receiver_handle)
            .expect("Receiver task failed");

        self.rt
            .block_on(self.run_test_handle)
            .expect("Test task failed");

        // Then run the validation future with the test context
        self.rt.block_on(future_fn(context))
    }
}
