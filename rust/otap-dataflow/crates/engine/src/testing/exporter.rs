// SPDX-License-Identifier: Apache-2.0

//! Test utilities for exporters.
//!
//! This module provides specialized utilities for testing exporter components:
//!
//! - `ExporterTestContext`: Provides a context for interacting with exporters during tests
//! - `ExporterTestRuntime`: Configures and manages a single-threaded tokio runtime for exporter tests
//!
//! These utilities are designed to make testing exporters simpler by abstracting away common
//! setup and lifecycle management.

use crate::exporter::{EffectHandler, Exporter, MessageChannel};
use crate::message::ControlMsg;
use crate::testing::{CtrMsgCounters, TestMsg, create_test_channel, setup_test_runtime};
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use serde_json::Value;
use std::time::Duration;
use tokio::task::LocalSet;
use tokio::time::sleep;

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
    counter: CtrMsgCounters,

    /// Join handle for the starting the exporter task
    start_exporter_handle: Option<tokio::task::JoinHandle<()>>,

    /// Join handle for the starting the test task
    start_test_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ExporterTestRuntime {
    /// Creates a new test runtime with channels of the specified capacity.
    pub fn new(channel_capacity: usize) -> Self {
        let (rt, local_tasks) = setup_test_runtime();
        let counter = CtrMsgCounters::new();
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
            start_exporter_handle: None,
            start_test_handle: None,
        }
    }

    /// Returns the message counter.
    pub fn counters(&self) -> CtrMsgCounters {
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
                .expect("Control channel not initialized"),
            self.pdata_rx.take().expect("PData channel not initialized"),
        );

        let boxed_exporter = Box::new(exporter);

        self.start_exporter_handle = Some(self.local_tasks.spawn_local(async move {
            boxed_exporter
                .start(msg_chan, EffectHandler::new("test_exporter"))
                .await
                .expect("Exporter event loop failed");
        }));
    }

    /// Spawns a local task with a TestContext that provides access to transmitters.
    pub fn start_test<F, Fut>(&mut self, f: F)
    where
        F: FnOnce(ExporterTestContext) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let context = ExporterTestContext::new(self.control_tx.clone(), self.pdata_tx.clone());
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
        F: FnOnce(ExporterTestContext) -> Fut,
        Fut: Future<Output = T>,
    {
        // First run all the spawned tasks to completion
        self.rt.block_on(self.local_tasks);
        
        let start_exporter_handle = self
            .start_exporter_handle
            .take()
            .expect("Exporter task not started");

        let start_test_handle = self
            .start_test_handle
            .take()
            .expect("Test task not started");

        self.rt.block_on(start_exporter_handle).expect("Exporter task failed");
        self.rt.block_on(start_test_handle).expect("Test task failed");

        let context = ExporterTestContext::new(self.control_tx.clone(), self.pdata_tx.clone());

        // Then run the validation future with the test context
        self.rt.block_on(future_fn(context))
    }
}
