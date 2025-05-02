// SPDX-License-Identifier: Apache-2.0

//! Test utilities for exporters.
//!
//! These utilities are designed to make testing exporters simpler by abstracting away common
//! setup and lifecycle management.

use crate::config::ExporterConfig;
use crate::exporter::{ExporterWrapper, MessageChannel};
use crate::message::ControlMsg;
use crate::testing::{CtrlMsgCounters, create_not_send_channel, setup_test_runtime};
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
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
    control_tx: mpsc::Sender<ControlMsg>,
    /// Sender for pipeline data
    pdata_tx: mpsc::Sender<PData>,
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
    pub fn new(
        control_tx: mpsc::Sender<ControlMsg>,
        pdata_tx: mpsc::Sender<PData>,
        counters: CtrlMsgCounters,
    ) -> Self {
        Self {
            control_tx,
            pdata_tx,
            counters,
        }
    }

    /// Returns the control message counters.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
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
    pub async fn send_shutdown(
        &self,
        deadline: Duration,
        reason: &str,
    ) -> Result<(), SendError<ControlMsg>> {
        self.control_tx
            .send_async(ControlMsg::Shutdown {
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
        self.pdata_tx.send_async(content).await
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

    /// Sender for control messages
    control_tx: mpsc::Sender<ControlMsg>,
    /// Receiver for control messages
    control_rx: Option<mpsc::Receiver<ControlMsg>>,

    /// Sender for pdata messages
    pdata_tx: mpsc::Sender<PData>,
    /// Receiver for pdata messages
    pdata_rx: Option<mpsc::Receiver<PData>>,

    /// Message counter for tracking processed messages
    counter: CtrlMsgCounters,
}

/// Data and operations for the test phase of an exporter.
pub struct TestPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    counters: CtrlMsgCounters,

    control_sender: mpsc::Sender<ControlMsg>,
    pdata_sender: mpsc::Sender<PData>,
}

/// Data and operations for the validation phase of an exporter.
pub struct ValidationPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    context: TestContext<PData>,
    _pd: PhantomData<PData>,
}

impl<PData: Clone + Debug + 'static> TestRuntime<PData> {
    /// Creates a new test runtime with channels of the specified capacity.
    pub fn new() -> Self {
        let config = ExporterConfig::new("test_exporter");
        let (rt, local_tasks) = setup_test_runtime();
        let counter = CtrlMsgCounters::new();
        let (control_tx, control_rx) = create_not_send_channel(config.control_channel.capacity);
        let (pdata_tx, pdata_rx) = create_not_send_channel(config.input_pdata_channel.capacity);

        Self {
            config,
            rt,
            local_tasks,
            control_tx,
            control_rx: Some(control_rx),
            pdata_tx,
            pdata_rx: Some(pdata_rx),
            counter,
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

    /// Sets the exporter for the test runtime and returns the test phase.
    pub fn set_exporter(mut self, exporter: ExporterWrapper<PData>) -> TestPhase<PData> {
        let msg_chan = MessageChannel::new(
            self.control_rx
                .take()
                .expect("Control channel not initialized"),
            self.pdata_rx.take().expect("PData channel not initialized"),
        );

        let _ = self.local_tasks.spawn_local(async move {
            exporter
                .start(msg_chan)
                .await
                .expect("Exporter event loop failed");
        });
        TestPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counter.clone(),
            control_sender: self.control_tx,
            pdata_sender: self.pdata_tx,
        }
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
            _pd: PhantomData,
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
        F: FnOnce(TestContext<PData>) -> Fut,
        Fut: Future<Output = T>,
    {
        // First run all the spawned tasks to completion
        let local_tasks = std::mem::take(&mut self.local_tasks);
        self.rt.block_on(local_tasks);
        // Then run the validation future with the test context
        self.rt.block_on(future_fn(self.context))
    }
}
