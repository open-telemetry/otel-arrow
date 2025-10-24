// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Testing utilities for processors.
//!
//! These utilities are designed to make testing processors simpler by abstracting away common
//! setup and lifecycle management.

use crate::config::ProcessorConfig;
use crate::control::pipeline_ctrl_msg_channel;
use crate::error::Error;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::{Message, Receiver, Sender};
use crate::node::{NodeWithPDataReceiver, NodeWithPDataSender};
use crate::processor::{ProcessorWrapper, ProcessorWrapperRuntime};
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::testing::{CtrlMsgCounters, setup_test_runtime, test_node};
use otap_df_telemetry::MetricsSystem;
use otap_df_telemetry::registry::MetricsRegistryHandle;
use otap_df_telemetry::reporter::MetricsReporter;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::task::{JoinHandle, LocalSet};
use tokio::time::sleep;

/// Context used during the test phase of a test.
pub struct TestContext<PData> {
    runtime: ProcessorWrapperRuntime<PData>,
    output_receiver: Option<Receiver<PData>>,
}

/// Context used during the validation phase of a test.
pub struct ValidateContext {
    counters: CtrlMsgCounters,
}

impl<PData> TestContext<PData> {
    /// Creates a new TestContext from a ProcessorWrapperRuntime.
    #[must_use]
    pub fn new(runtime: ProcessorWrapperRuntime<PData>) -> Self {
        Self {
            runtime,
            output_receiver: None,
        }
    }

    /// Processes a new message.
    pub async fn process(&mut self, msg: Message<PData>) -> Result<(), Error> {
        match &mut self.runtime {
            ProcessorWrapperRuntime::Local {
                processor,
                effect_handler,
                ..
            } => processor.process(msg, effect_handler).await,
            ProcessorWrapperRuntime::Shared {
                processor,
                effect_handler,
                ..
            } => processor.process(msg, effect_handler).await,
        }
    }

    /// Drains and returns all messages from the test output receiver.
    pub async fn drain_pdata(&mut self) -> Vec<PData> {
        let mut emitted = Vec::new();

        if let Some(receiver) = &mut self.output_receiver {
            match receiver {
                Receiver::Local(local_receiver) => {
                    while let Ok(msg) = local_receiver.try_recv() {
                        emitted.push(msg);
                    }
                }
                Receiver::Shared(shared_receiver) => {
                    while let Ok(msg) = shared_receiver.try_recv() {
                        emitted.push(msg);
                    }
                }
            }
        }

        emitted
    }

    /// Sleeps for the specified duration.
    pub async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }

    /// Sets the pipeline control message sender on the effect handler.
    /// This is needed for processor ACK/NACK handling.
    pub fn set_pipeline_ctrl_sender(
        &mut self,
        pipeline_ctrl_sender: crate::control::PipelineCtrlMsgSender<PData>,
    ) {
        match &mut self.runtime {
            ProcessorWrapperRuntime::Local { effect_handler, .. } => {
                effect_handler
                    .core
                    .set_pipeline_ctrl_msg_sender(pipeline_ctrl_sender);
            }
            ProcessorWrapperRuntime::Shared { effect_handler, .. } => {
                effect_handler
                    .core
                    .set_pipeline_ctrl_msg_sender(pipeline_ctrl_sender);
            }
        }
    }
}

impl ValidateContext {
    /// Returns the control message counters.
    #[must_use]
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }
}

/// A test runtime for simplifying processor tests.
///
/// This structure encapsulates the common setup logic needed for testing processors,
/// including channel creation, processor instantiation, and task management.
pub struct TestRuntime<PData> {
    /// The configuration for the processor
    config: ProcessorConfig,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    /// Message counter for tracking processed messages
    counter: CtrlMsgCounters,

    metrics_system: MetricsSystem,

    _pd: PhantomData<PData>,
}

/// Data and operations for the test phase of a processor.
pub struct TestPhase<PData> {
    rt: tokio::runtime::Runtime,
    local_tasks: LocalSet,
    processor: ProcessorWrapper<PData>,
    counters: CtrlMsgCounters,
    output_receiver: Option<Receiver<PData>>,
    metrics_system: MetricsSystem,
}

/// Data and operations for the validation phase of a processor.
pub struct ValidationPhase {
    rt: tokio::runtime::Runtime,
    local_tasks: LocalSet,
    counters: CtrlMsgCounters,
    metrics_collection_handle: JoinHandle<Result<(), otap_df_telemetry::error::Error>>,
}

impl<PData: Clone + Debug + 'static> Default for TestRuntime<PData> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PData: Clone + Debug + 'static> TestRuntime<PData> {
    /// Creates a new test runtime with default configuration.
    #[must_use]
    pub fn new() -> Self {
        let metrics_system = MetricsSystem::default();
        let config = ProcessorConfig::new("test_processor");
        let (rt, local_tasks) = setup_test_runtime();

        Self {
            config,
            rt,
            local_tasks,
            counter: CtrlMsgCounters::new(),
            metrics_system,
            _pd: PhantomData,
        }
    }

    /// Returns the current receiver configuration.
    pub fn config(&self) -> &ProcessorConfig {
        &self.config
    }

    /// Returns a handle to the metrics registry.
    pub fn metrics_registry(&self) -> MetricsRegistryHandle {
        self.metrics_system.registry()
    }

    /// Returns a metrics reporter for use in the processor runtime.
    pub fn metrics_reporter(&self) -> MetricsReporter {
        self.metrics_system.reporter()
    }

    /// Returns the message counter.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counter.clone()
    }

    /// Initializes the test runtime with a processor using a non-sendable effect handler.
    pub fn set_processor(self, mut processor: ProcessorWrapper<PData>) -> TestPhase<PData> {
        // Set up test channels for the processor
        let (pdata_sender, pdata_receiver) = match &processor {
            ProcessorWrapper::Local { .. } => {
                let (sender, receiver) = otap_df_channel::mpsc::Channel::new(100);
                (
                    Sender::Local(LocalSender::MpscSender(sender)),
                    Receiver::Local(LocalReceiver::MpscReceiver(receiver)),
                )
            }
            ProcessorWrapper::Shared { .. } => {
                let (sender, receiver) = tokio::sync::mpsc::channel(100);
                (
                    Sender::Shared(SharedSender::MpscSender(sender)),
                    Receiver::Shared(SharedReceiver::MpscReceiver(receiver)),
                )
            }
        };

        // Set the output sender for the processor
        let _ = processor.set_pdata_sender(
            test_node(self.config().name.clone()),
            "out".into(),
            pdata_sender,
        );

        // Set a dummy input receiver (not used in these tests since we call process directly)
        // We need this because prepare_runtime expects both to be set
        let dummy_receiver = match &processor {
            ProcessorWrapper::Local { .. } => {
                let (_, receiver) = otap_df_channel::mpsc::Channel::new(1);
                Receiver::Local(LocalReceiver::MpscReceiver(receiver))
            }
            ProcessorWrapper::Shared { .. } => {
                let (_, receiver) = tokio::sync::mpsc::channel(1);
                Receiver::Shared(SharedReceiver::MpscReceiver(receiver))
            }
        };
        let _ = processor.set_pdata_receiver(test_node(self.config().name.clone()), dummy_receiver);

        TestPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            processor,
            counters: self.counter,
            output_receiver: Some(pdata_receiver),
            metrics_system: self.metrics_system,
        }
    }
}

impl<PData: Debug + 'static> TestPhase<PData> {
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<F, Fut>(self, f: F) -> ValidationPhase
    where
        F: FnOnce(TestContext<PData>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let metrics_reporter = self.metrics_system.reporter();
        // Spawn metrics collection loop
        let metrics_collection_handle = self.rt.spawn(self.metrics_system.run_collection_loop());

        // The entire scenario is run to completion before the validation phase
        self.rt.block_on(async move {
            let mut runtime = self
                .processor
                .prepare_runtime(metrics_reporter)
                .await
                .expect("Failed to prepare runtime");

            let (pipeline_ctrl_msg_tx, _pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);
            match runtime {
                ProcessorWrapperRuntime::Local {
                    ref mut effect_handler,
                    ..
                } => {
                    effect_handler
                        .core
                        .set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);
                }
                ProcessorWrapperRuntime::Shared {
                    ref mut effect_handler,
                    ..
                } => {
                    effect_handler
                        .core
                        .set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);
                }
            }
            let mut context = TestContext::new(runtime);
            context.output_receiver = self.output_receiver;
            f(context).await;
        });

        // Prepare for next phase
        ValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
            metrics_collection_handle,
        }
    }
}

impl ValidationPhase {
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
        F: FnOnce(ValidateContext) -> Fut,
        Fut: Future<Output = T>,
    {
        let context = ValidateContext {
            counters: self.counters,
        };

        // First run all the spawned tasks to completion
        self.rt.block_on(self.local_tasks);

        // Then run the validation future with the test context
        let result = self.rt.block_on(future_fn(context));
        // Finally, ensure the metrics collection loop is properly shut down
        self.metrics_collection_handle.abort();
        result
    }
}
