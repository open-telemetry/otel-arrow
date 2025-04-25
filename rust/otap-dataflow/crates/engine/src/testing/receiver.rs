// SPDX-License-Identifier: Apache-2.0

//! Utility functions for testing receivers.
//!
//! This module provides specialized utilities for testing receiver components:
//!
//! - `ReceiverTestContext`: Provides a context for interacting with receivers during tests
//! - `ReceiverTestRuntime`: Configures and manages a single-threaded tokio runtime for receiver tests
//!
//! These utilities are designed to make testing receivers simpler by abstracting away common
//! setup and lifecycle management.

use crate::message::ControlMsg;
use crate::receiver::{ControlMsgChannel, Receiver};
use crate::testing::{CtrMsgCounters, TestMsg, create_test_channel, setup_test_runtime};
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use serde_json::Value;
use std::time::Duration;
use tokio::task::LocalSet;
use tokio::time::sleep;

/// A context object for receiver tests that provides functionality for interacting with receivers.
pub struct ReceiverTestContext {
    /// Sender for control messages
    control_tx: mpsc::Sender<ControlMsg>,
    /// Receiver for messages sent by the receiver
    pdata_rx: Option<mpsc::Receiver<TestMsg>>,
}

impl ReceiverTestContext {
    /// Sends a timer tick control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_timer_tick(&self) -> Result<(), SendError<ControlMsg>> {
        self.control_tx.send_async(ControlMsg::TimerTick {}).await
    }

    /// Sends a config control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_config(&self, config: Value) -> Result<(), SendError<ControlMsg>> {
        self.control_tx
            .send_async(ControlMsg::Config { config })
            .await
    }

    /// Sends a shutdown control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_shutdown(&self, reason: &str) -> Result<(), SendError<ControlMsg>> {
        self.control_tx
            .send_async(ControlMsg::Shutdown {
                reason: reason.to_owned(),
            })
            .await
    }

    /// Sleeps for the specified duration.
    pub async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }

    /// Returns a reference to the control message sender.
    pub fn pdata_rx(&mut self) -> Option<mpsc::Receiver<TestMsg>> {
        self.pdata_rx.take()
    }
}

/// A test runtime for simplifying receiver tests.
///
/// This structure encapsulates the common setup logic needed for testing receivers,
/// including channel creation, receiver instantiation, and task management.
pub struct ReceiverTestRuntime {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    /// Sender for control messages
    control_tx: mpsc::Sender<ControlMsg>,
    /// Receiver for control messages
    control_rx: Option<mpsc::Receiver<ControlMsg>>,

    /// Sender for messages sent by the receiver
    pdata_tx: mpsc::Sender<TestMsg>,
    /// Receiver for messages sent by the receiver
    pdata_rx: Option<mpsc::Receiver<TestMsg>>,

    /// Message counter for tracking processed messages
    counter: CtrMsgCounters,

    /// Join handle for starting the receiver task
    start_receiver_handle: Option<tokio::task::JoinHandle<()>>,

    /// Join handle for starting the test task
    start_test_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ReceiverTestRuntime {
    /// Creates a new test runtime with channels of the specified capacity.
    pub fn new(channel_capacity: usize) -> Self {
        let (rt, local_tasks) = setup_test_runtime();
        let (control_tx, control_rx) = create_test_channel(channel_capacity);
        let (pdata_tx, pdata_rx) = create_test_channel(channel_capacity);

        Self {
            rt,
            local_tasks,
            control_tx,
            control_rx: Some(control_rx),
            pdata_tx,
            pdata_rx: Some(pdata_rx),
            counter: CtrMsgCounters::new(),
            start_receiver_handle: None,
            start_test_handle: None,
        }
    }

    /// Returns the message counter.
    pub fn counters(&self) -> CtrMsgCounters {
        self.counter.clone()
    }

    /// Starts a receiver with the configured channels.
    pub fn start_receiver<R>(&mut self, receiver: R)
    where
        R: Receiver<PData = TestMsg> + 'static,
    {
        let control_rx = self
            .control_rx
            .take()
            .expect("Control channel not initialized");
        let ctrl_msg_chan = ControlMsgChannel::new(control_rx);
        let pdata_tx = self.pdata_tx.clone();

        let receiver = Box::new(receiver);
        self.start_receiver_handle = Some(self.local_tasks.spawn_local(async move {
            receiver
                .start(
                    ctrl_msg_chan,
                    crate::receiver::EffectHandler::new("test_receiver", pdata_tx),
                )
                .await
                .expect("Receiver event loop failed");
        }));
    }

    /// Starts the test scenario by executing the provided function with the test context.
    pub fn start_test<F, Fut>(&mut self, f: F)
    where
        F: FnOnce(ReceiverTestContext) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let context = ReceiverTestContext {
            control_tx: self.control_tx.clone(),
            pdata_rx: None,
        };
        self.start_test_handle = Some(self.local_tasks.spawn_local(async move {
            f(context).await;
        }));
    }

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
    pub fn validate<F, Fut, T>(mut self, future_fn: F) -> T
    where
        F: FnOnce(ReceiverTestContext) -> Fut,
        Fut: Future<Output = T>,
    {
        // First run all the spawned tasks to completion
        self.rt.block_on(self.local_tasks);

        let context = ReceiverTestContext {
            control_tx: self.control_tx.clone(),
            pdata_rx: self.pdata_rx.take(),
        };

        let start_receiver_handle = self
            .start_receiver_handle
            .take()
            .expect("Receiver task not started");
        self.rt
            .block_on(start_receiver_handle)
            .expect("Receiver task failed");

        let start_test_handle = self
            .start_test_handle
            .take()
            .expect("Test task not started");
        self.rt
            .block_on(start_test_handle)
            .expect("Test task failed");

        // Then run the validation future with the test context
        self.rt.block_on(future_fn(context))
    }
}
