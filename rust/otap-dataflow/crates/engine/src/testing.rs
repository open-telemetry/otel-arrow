// SPDX-License-Identifier: Apache-2.0

//! Common testing utilities for engine components.
//!
//! This module provides shared testing constructs used across tests for receivers,
//! processors, and exporters.

use crate::exporter::{EffectHandler, Exporter, MessageChannel};
use crate::message::ControlMsg;
use crate::receiver;
use crate::receiver::{ControlMsgChannel, Receiver};
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use serde_json::Value;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::task::LocalSet;
use tokio::time::sleep;

/// A test message type used in component tests.
#[derive(Debug, PartialEq, Clone)]
pub struct TestMsg(pub String);

impl TestMsg {
    /// Creates a new test message with the given content.
    pub fn new<S: Into<String>>(content: S) -> Self {
        TestMsg(content.into())
    }
}

/// A context object that holds transmitters for use in test tasks.
pub struct ExporterTestContext {
    /// Sender for control messages
    control_tx: mpsc::Sender<ControlMsg>,
    /// Sender for pipeline data
    pdata_tx: mpsc::Sender<TestMsg>,
}

impl ExporterTestContext {
    /// Creates a new TestContext with the given transmitters.
    pub fn new(control_tx: mpsc::Sender<ControlMsg>, pdata_tx: mpsc::Sender<TestMsg>) -> Self {
        Self {
            control_tx,
            pdata_tx,
        }
    }

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

    /// Sends a data message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_data<S: Into<String>>(&self, content: S) -> Result<(), SendError<TestMsg>> {
        self.pdata_tx.send_async(TestMsg::new(content)).await
    }

    /// Sleeps for the specified duration.
    pub async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }
}

/// Creates a single-threaded runtime with a local task set for testing components.
pub fn setup_test_runtime() -> (tokio::runtime::Runtime, LocalSet) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let local_tasks = LocalSet::new();
    (rt, local_tasks)
}

/// A counter for tracking the number of messages processed.
///
/// Uses Rc<RefCell<usize>> to allow sharing between components and test code.
#[derive(Clone)]
pub struct MessageCounter {
    pub timer_tick_count: Rc<RefCell<usize>>,
    pub message_count: Rc<RefCell<usize>>,
    pub config_count: Rc<RefCell<usize>>,
    pub shutdown_count: Rc<RefCell<usize>>,
}

impl MessageCounter {
    /// Creates a new message counter with all counts initialized to zero.
    pub fn new() -> Self {
        MessageCounter {
            timer_tick_count: Rc::new(RefCell::new(0)),
            message_count: Rc::new(RefCell::new(0)),
            config_count: Rc::new(RefCell::new(0)),
            shutdown_count: Rc::new(RefCell::new(0)),
        }
    }

    /// Increments the timer tick count.
    pub fn increment_timer_tick(&self) {
        *self.timer_tick_count.borrow_mut() += 1;
    }

    /// Increments the message count.
    pub fn increment_message(&self) {
        *self.message_count.borrow_mut() += 1;
    }

    /// Increments the config count.
    pub fn increment_config(&self) {
        *self.config_count.borrow_mut() += 1;
    }

    /// Increments the shutdown count.
    pub fn increment_shutdown(&self) {
        *self.shutdown_count.borrow_mut() += 1;
    }

    /// Gets the current timer tick count.
    pub fn get_timer_tick_count(&self) -> usize {
        *self.timer_tick_count.borrow()
    }

    /// Gets the current message count.
    pub fn get_message_count(&self) -> usize {
        *self.message_count.borrow()
    }

    /// Gets the current config count.
    pub fn get_config_count(&self) -> usize {
        *self.config_count.borrow()
    }

    /// Gets the current shutdown count.
    pub fn get_shutdown_count(&self) -> usize {
        *self.shutdown_count.borrow()
    }
}

/// Helper to create MPSC channels with a specific capacity.
///
/// This function creates a sender-receiver pair with the given capacity.
pub fn create_test_channel<T>(capacity: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::Channel::new(capacity)
}

/// A test runtime for simplifying exporter tests.
///
/// This structure encapsulates the common setup logic needed for testing exporters,
/// including channel creation, exporter instantiation, and task management.
pub struct ExporterTestRuntime {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    /// Sender for control messages
    control_tx: mpsc::Sender<ControlMsg>,
    /// Receiver for control messages
    control_rx: Option<mpsc::Receiver<ControlMsg>>,

    /// Sender for pipeline data
    pdata_tx: mpsc::Sender<TestMsg>,
    /// Receiver for pipeline data
    pdata_rx: Option<mpsc::Receiver<TestMsg>>,

    /// Message counter for tracking processed messages
    counter: MessageCounter,
}

impl ExporterTestRuntime {
    /// Creates a new test runtime with channels of the specified capacity.
    pub fn new(channel_capacity: usize) -> Self {
        let (rt, local_tasks) = setup_test_runtime();
        let counter = MessageCounter::new();
        let (control_tx, control_rx) = create_test_channel(channel_capacity);
        let (pdata_tx, pdata_rx) = create_test_channel(channel_capacity);

        Self {
            rt,
            local_tasks,
            control_tx,
            control_rx: Some(control_rx),
            pdata_tx,
            pdata_rx: Some(pdata_rx),
            counter,
        }
    }

    /// Returns the message counter.
    pub fn counters(&self) -> MessageCounter {
        self.counter.clone()
    }

    /// Starts an exporter with the configured channels.
    pub fn start_exporter<E>(&mut self, exporter: E)
    where
        E: Exporter<PData = TestMsg> + 'static,
    {
        let msg_chan = MessageChannel::new(
            self.control_rx
                .take()
                .expect("Control channel not, initialized"),
            self.pdata_rx
                .take()
                .expect("PData channel not, initialized"),
        );

        let boxed_exporter = Box::new(exporter);

        let _ = self.local_tasks.spawn_local(async move {
            boxed_exporter
                .start(msg_chan, EffectHandler::new("test_exporter"))
                .await
                .expect("Exporter event loop failed");
        });
    }

    /// Spawns a local task with a TestContext that provides access to transmitters.
    pub fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce(ExporterTestContext) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let context = ExporterTestContext::new(self.control_tx.clone(), self.pdata_tx.clone());
        let _ = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
    }

    /// Runs all spawned tasks to completion.
    pub fn run(self) -> MessageCounter {
        let counters = self.counters();
        self.rt.block_on(self.local_tasks);
        counters
    }
}

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
        }
    }

    /// Starts a receiver with the configured channels.
    pub fn start_receiver<R>(&mut self, receiver: R)
    where
        R: Receiver<PData = TestMsg> + 'static,
    {
        let control_rx = self
            .control_rx
            .take()
            .expect("Control channel not, initialized");
        let ctrl_msg_chan = ControlMsgChannel::new(control_rx);
        let pdata_tx = self.pdata_tx.clone();

        let receiver = Box::new(receiver);
        let _ = self.local_tasks.spawn_local(async move {
            receiver
                .start(
                    ctrl_msg_chan,
                    receiver::EffectHandler::new("test_receiver", pdata_tx),
                )
                .await
                .expect("Receiver event loop failed");
        });
    }

    /// Spawns a local task with access to the test context.
    pub fn spawn_with_context<F, Fut>(&mut self, f: F)
    where
        F: FnOnce(ReceiverTestContext) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let context = ReceiverTestContext {
            control_tx: self.control_tx.clone(),
            pdata_rx: None,
        };
        let _ = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
    }

    /// Runs all spawned tasks to completion and executes the provided future to verify test expectations.
    ///
    /// This method is particularly useful when you need to check that a message was received or
    /// to perform other post-run validations.
    ///
    /// # Type Parameters
    ///
    /// * `F` - A function that creates a future with access to the test context.
    /// * `T` - The output type of the future.
    ///
    /// # Returns
    ///
    /// The result of the provided future.
    pub fn run_until<F, Fut, T>(mut self, future_fn: F) -> T
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

        // Then run the validation future with the test context
        self.rt.block_on(future_fn(context))
    }
}
