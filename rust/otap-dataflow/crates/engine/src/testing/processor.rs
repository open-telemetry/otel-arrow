// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Testing utilities for processors.
//!
//! These utilities are designed to make testing processors simpler by abstracting away common
//! setup and lifecycle management.

use crate::config::ProcessorConfig;
use crate::error::Error;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::{Message, Receiver, Sender};
use crate::node::{NodeWithPDataReceiver, NodeWithPDataSender};
use crate::processor::{ProcessorWrapper, ProcessorWrapperRuntime};
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::testing::{CtrlMsgCounters, setup_test_runtime, test_node};
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::task::LocalSet;
use tokio::time::sleep;

/// Context used during the test phase of a test.
pub struct TestContext<PData: crate::PipelineData> {
    runtime: ProcessorWrapperRuntime<PData>,
    output_receiver: Option<Receiver<PData>>,
}

/// Context used during the validation phase of a test.
pub struct ValidateContext {
    counters: CtrlMsgCounters,
}

impl<PData: crate::PipelineData> TestContext<PData> {
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

    _pd: PhantomData<PData>,
}

/// Data and operations for the test phase of a processor.
pub struct TestPhase<PData: crate::PipelineData> {
    rt: tokio::runtime::Runtime,
    local_tasks: LocalSet,
    processor: ProcessorWrapper<PData>,
    counters: CtrlMsgCounters,
    output_receiver: Option<Receiver<PData>>,
}

/// Data and operations for the validation phase of a processor.
pub struct ValidationPhase {
    rt: tokio::runtime::Runtime,
    local_tasks: LocalSet,
    counters: CtrlMsgCounters,
}

impl<PData: crate::PipelineData + Clone + Debug + 'static> TestRuntime<PData> {
    /// Creates a new test runtime with channels of the specified capacity.
    #[must_use]
    pub fn new() -> Self {
        let config = ProcessorConfig::new("test_processor");
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
    pub fn config(&self) -> &ProcessorConfig {
        &self.config
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
        }
    }
}

impl<PData: crate::PipelineData + Clone + Debug + 'static> Default for TestRuntime<PData> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PData: crate::PipelineData + Debug + 'static> TestPhase<PData> {
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<F, Fut>(self, f: F) -> ValidationPhase
    where
        F: FnOnce(TestContext<PData>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        // The entire scenario is run to completion before the validation phase
        self.rt.block_on(async move {
            let runtime = self
                .processor
                .prepare_runtime()
                .await
                .expect("Failed to prepare runtime");
            let mut context = TestContext::new(runtime);
            context.output_receiver = self.output_receiver;
            f(context).await;
        });

        // Prepare for next phase
        ValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
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
        self.rt.block_on(future_fn(context))
    }
}
