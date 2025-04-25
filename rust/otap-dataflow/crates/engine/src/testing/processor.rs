// SPDX-License-Identifier: Apache-2.0

//! Testing utilities for processors.
//!
//! These utilities are designed to make testing processors simpler by abstracting away common
//! setup and lifecycle management.

use crate::error::Error;
use crate::message::Message;
use crate::processor::{NotSendEffectHandler, Processor, SendEffectHandler};
use crate::testing::{
    CtrlMsgCounters, create_not_send_channel, create_send_channel, setup_test_runtime,
};
use async_trait::async_trait;
use otap_df_channel::mpsc;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::task::LocalSet;
use tokio::time::sleep;

/// The interface exposed to the test phase of a processor.
#[async_trait(?Send)]
pub trait TestContext<PData> {
    /// Calls the processor's process method with the given message.
    async fn process(&mut self, msg: Message<PData>) -> Result<(), Error<PData>>;

    /// Drains and returns all pdata messages emitted by the processor via the effect handler.
    async fn drain_pdata(&mut self) -> Vec<PData>;

    /// Sleeps for the specified duration.
    async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }
}

/// Context used during the test phase of a test (!Send).
pub struct NotSendTestContext<PData, P>
where
    P: Processor<PData, NotSendEffectHandler<PData>>,
{
    processor: P,
    pdata_receiver: mpsc::Receiver<PData>,
    effect_handler: NotSendEffectHandler<PData>,
}

/// Context used during the test phase of a test (!Send).
pub struct SendTestContext<PData, P>
where
    P: Processor<PData, SendEffectHandler<PData>>,
{
    processor: P,
    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
    effect_handler: SendEffectHandler<PData>,
}

/// Context used during the validation phase of a test.
pub struct ValidateContext {
    counters: CtrlMsgCounters,
}

impl<PData, P> NotSendTestContext<PData, P>
where
    P: Processor<PData, NotSendEffectHandler<PData>>,
{
    /// Creates a new NotSendTestContext.
    pub fn new(processor: P, processor_name: &str, channel_capacity: usize) -> Self {
        let (pdata_sender, pdata_receiver) = create_not_send_channel(channel_capacity);
        let effect_handler = NotSendEffectHandler::new(processor_name, pdata_sender);
        Self {
            processor,
            pdata_receiver,
            effect_handler,
        }
    }
}

#[async_trait(?Send)]
impl<PData, P> TestContext<PData> for NotSendTestContext<PData, P>
where
    P: Processor<PData, NotSendEffectHandler<PData>> + 'static,
{
    async fn process(&mut self, msg: Message<PData>) -> Result<(), Error<PData>> {
        self.processor.process(msg, &mut self.effect_handler).await
    }

    async fn drain_pdata(&mut self) -> Vec<PData> {
        let mut emitted = Vec::new();
        while let Ok(msg) = self.pdata_receiver.try_recv() {
            emitted.push(msg);
        }
        emitted
    }
}

impl<PData, P> SendTestContext<PData, P>
where
    P: Processor<PData, SendEffectHandler<PData>>,
{
    /// Creates a new SendTestContext.
    pub fn new(processor: P, processor_name: &str, channel_capacity: usize) -> Self {
        let (pdata_sender, pdata_receiver) = create_send_channel(channel_capacity);
        let effect_handler = SendEffectHandler::new(processor_name, pdata_sender);
        Self {
            processor,
            pdata_receiver,
            effect_handler,
        }
    }
}

#[async_trait(?Send)]
impl<PData, P> TestContext<PData> for SendTestContext<PData, P>
where
    P: Processor<PData, SendEffectHandler<PData>> + 'static,
{
    async fn process(&mut self, msg: Message<PData>) -> Result<(), Error<PData>> {
        self.processor.process(msg, &mut self.effect_handler).await
    }

    async fn drain_pdata(&mut self) -> Vec<PData> {
        let mut emitted = Vec::new();
        while let Ok(msg) = self.pdata_receiver.try_recv() {
            emitted.push(msg);
        }
        emitted
    }
}

impl ValidateContext {
    /// Returns the control message counters.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }
}

/// A test runtime for simplifying processor tests.
///
/// This structure encapsulates the common setup logic needed for testing processors,
/// including channel creation, processor instantiation, and task management.
pub struct TestRuntime<PData> {
    channel_capacity: usize,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    /// Message counter for tracking processed messages
    counter: CtrlMsgCounters,

    _pd: PhantomData<PData>,
}

/// Data and operations for the test phase of a processor (not sendable effect handler).
pub struct NotSendTestPhase<PData, P>
where
    P: Processor<PData, NotSendEffectHandler<PData>>,
{
    name: String,
    channel_capacity: usize,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    processor: Option<P>,
    counters: CtrlMsgCounters,
    _pd: PhantomData<PData>,
}

/// Data and operations for the validation phase of a processor.
pub struct ValidationPhase {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    counters: CtrlMsgCounters,
}

/// Data and operations for the validation phase of a processor (sendable effect handler).
pub struct SendTestPhase<PData, P>
where
    P: Processor<PData, SendEffectHandler<PData>>,
{
    name: String,
    channel_capacity: usize,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    processor: Option<P>,
    counters: CtrlMsgCounters,
    _pd: PhantomData<PData>,
}

impl<PData: Clone + Debug + 'static> TestRuntime<PData> {
    /// Creates a new test runtime with channels of the specified capacity.
    pub fn new(channel_capacity: usize) -> Self {
        let (rt, local_tasks) = setup_test_runtime();

        Self {
            channel_capacity,
            rt,
            local_tasks,
            counter: CtrlMsgCounters::new(),
            _pd: PhantomData,
        }
    }

    /// Returns the message counter.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counter.clone()
    }

    /// Initializes the test runtime with a processor using a non-sendable effect handler.
    pub fn processor_with_non_send_effect_handler<P>(
        self,
        processor: P,
        name: &str,
    ) -> NotSendTestPhase<PData, P>
    where
        P: Processor<PData, NotSendEffectHandler<PData>> + 'static,
    {
        NotSendTestPhase {
            channel_capacity: self.channel_capacity,
            name: name.to_owned(),
            rt: self.rt,
            local_tasks: self.local_tasks,
            processor: Some(processor),
            counters: self.counter,
            _pd: PhantomData,
        }
    }

    /// Initializes the test runtime with a processor using a sendable effect handler.
    pub fn processor_with_send_effect_handler<P>(
        self,
        processor: P,
        name: &str,
    ) -> SendTestPhase<PData, P>
    where
        P: Processor<PData, SendEffectHandler<PData>> + 'static,
    {
        SendTestPhase {
            channel_capacity: self.channel_capacity,
            name: name.to_owned(),
            rt: self.rt,
            local_tasks: self.local_tasks,
            processor: Some(processor),
            counters: self.counter,
            _pd: PhantomData,
        }
    }
}

impl<PData: Debug + 'static, P> NotSendTestPhase<PData, P>
where
    P: Processor<PData, NotSendEffectHandler<PData>> + 'static,
{
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<F, Fut>(self, f: F) -> ValidationPhase
    where
        F: FnOnce(NotSendTestContext<PData, P>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let processor = self.processor.expect("Processor not set");
        let context = NotSendTestContext::new(processor, &self.name, self.channel_capacity);
        let _ = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
        ValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
        }
    }
}

impl<PData: Debug + 'static, P> SendTestPhase<PData, P>
where
    P: Processor<PData, SendEffectHandler<PData>> + 'static,
{
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<F, Fut>(self, f: F) -> ValidationPhase
    where
        F: FnOnce(SendTestContext<PData, P>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let processor = self.processor.expect("Processor not set");
        let context = SendTestContext::new(processor, &self.name, self.channel_capacity);
        let _ = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
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
