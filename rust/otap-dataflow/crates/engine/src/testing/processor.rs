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

use crate::processor::{EffectHandler, Processor};
use crate::testing::{TestMsg, create_test_channel, setup_test_runtime};
use otap_df_channel::mpsc;
use tokio::task::LocalSet;

/// A test runtime for simplifying processor tests.
///
/// This structure encapsulates the common setup logic needed for testing processors,
/// including channel creation, processor instantiation, and task management.
pub struct ProcessorTestRuntime<P> {
    processor: Option<P>,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    /// Sender for pipeline data
    pdata_tx: mpsc::Sender<TestMsg>,
    /// Receiver for pipeline data
    pdata_rx: Option<mpsc::Receiver<TestMsg>>,
}

impl<P> ProcessorTestRuntime<P>
where
    P: Processor<PData = TestMsg> + 'static,
{
    /// Creates a new test runtime with channels of the specified capacity.
    pub fn new(processor: P, channel_capacity: usize) -> Self {
        let (rt, local_tasks) = setup_test_runtime();
        let (pdata_tx, pdata_rx) = create_test_channel(channel_capacity);

        Self {
            processor: Some(processor),
            rt,
            local_tasks,
            pdata_tx,
            pdata_rx: Some(pdata_rx),
        }
    }

    /// Spawns a local task with a TestContext that provides access to transmitters.
    pub fn start_test<F, Fut>(&mut self, f: F)
    where
        F: FnOnce(P, EffectHandler<P::PData>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let processor = self.processor.take().expect("Processor not set");
        let effect_handler = EffectHandler::new("test_processor", self.pdata_tx.clone());

        let _ = self.local_tasks.spawn_local(async move {
            f(processor, effect_handler).await;
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
