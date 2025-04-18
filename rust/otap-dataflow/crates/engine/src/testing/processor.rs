// SPDX-License-Identifier: Apache-2.0

//! Testing utilities for processors.
//!
//! This module provides specialized utilities for testing processor components:
//!
//! - `ProcessorTestContext`: Provides a context for interacting with processors during tests
//! - `ProcessorTestRuntime`: Configures and manages a single-threaded tokio runtime for processor tests
//!
//! These utilities are designed to make testing processors simpler by abstracting away common
//! setup and lifecycle management.

use crate::error::Error;
use crate::message::Message;
use crate::processor::{EffectHandler, Processor};
use crate::testing::{create_test_channel, setup_test_runtime};
use otap_df_channel::mpsc;
use std::future::Future;
use tokio::task::LocalSet;

/// A context object.
pub struct ProcessorTestContext<P>
where
    P: Processor,
{
    processor: P,
    pdata_rx: mpsc::Receiver<P::PData>,
    effect_handler: EffectHandler<P::PData>,
}

impl<P> ProcessorTestContext<P>
where
    P: Processor,
{
    /// Creates a new TestContext with the given transmitters.
    pub fn new(processor: P, channel_capacity: usize) -> Self {
        let (pdata_tx, pdata_rx) = create_test_channel(channel_capacity);
        let effect_handler = EffectHandler::new("test_processor", pdata_tx);
        Self {
            processor,
            pdata_rx,
            effect_handler,
        }
    }

    /// Calls the processor's process method with the given message.
    pub async fn process(&mut self, msg: Message<P::PData>) -> Result<(), Error<P::PData>> {
        self.processor.process(msg, &mut self.effect_handler).await
    }

    /// Returns the last emitted message from the processor.
    pub async fn emitted_pdata(&mut self) -> Vec<P::PData> {
        let mut emitted = Vec::new();
        while let Ok(msg) = self.pdata_rx.try_recv() {
            emitted.push(msg);
        }
        emitted
    }
}

/// A test runtime for simplifying processor tests.
///
/// This structure encapsulates the common setup logic needed for testing processors,
/// including channel creation, processor instantiation, and task management.
pub struct ProcessorTestRuntime<P>
where
    P: Processor,
{
    processor: Option<P>,
    channel_capacity: usize,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,
}

impl<P> ProcessorTestRuntime<P>
where
    P: Processor + 'static,
{
    /// Creates a new test runtime with channels of the specified capacity.
    pub fn new(processor: P, channel_capacity: usize) -> Self {
        let (rt, local_tasks) = setup_test_runtime();

        Self {
            processor: Some(processor),
            channel_capacity,
            rt,
            local_tasks,
        }
    }

    /// Spawns a local task with a TestContext that provides access to transmitters.
    pub fn start_test<F, Fut>(&mut self, f: F)
    where
        F: FnOnce(ProcessorTestContext<P>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let processor = self.processor.take().expect("Processor not set");
        let context = ProcessorTestContext::new(processor, self.channel_capacity);

        let _ = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
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
    pub fn validate<F, Fut, T>(self, future_fn: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        // First run all the spawned tasks to completion
        self.rt.block_on(self.local_tasks);

        // Then run the validation future with the test context
        self.rt.block_on(future_fn())
    }
}
