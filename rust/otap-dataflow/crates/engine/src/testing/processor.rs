// SPDX-License-Identifier: Apache-2.0

//! Testing utilities for processors.
//!
//! These utilities are designed to make testing processors simpler by abstracting away common
//! setup and lifecycle management.

use std::fmt::Debug;
use crate::error::Error;
use crate::message::Message;
use crate::processor::{NotSendEffectHandler, Processor, SendEffectHandler};
use crate::testing::{create_not_send_channel, create_send_channel, setup_test_runtime, CtrlMsgCounters};
use otap_df_channel::mpsc;
use std::future::Future;
use std::marker::PhantomData;
use std::time::Duration;
use async_trait::async_trait;
use tokio::task::LocalSet;
use tokio::time::sleep;
use otap_df_channel::error::RecvError;

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

/// Context used during the validation phase of a test (!Send context).
pub struct NotSendValidateContext<PData> {
    pdata_receiver: mpsc::Receiver<PData>,
    counters: CtrlMsgCounters,
}

/// Context used during the validation phase of a test (Send context).
pub struct SendValidateContext<PData> {
    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
    counters: CtrlMsgCounters,
}

impl<PData, P> NotSendTestContext<PData, P>
where
    P: Processor<PData, NotSendEffectHandler<PData>>
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

impl<PData,P> TestContext<PData> for NotSendTestContext<PData, P>
where
    P: Processor<PData, NotSendEffectHandler<PData>>
{
    /// Calls the processor's process method with the given message.
    async fn process(&mut self, msg: Message<PData>) -> Result<(), Error<PData>> {
        self.processor.process(msg, &mut self.effect_handler).await
    }

    /// Drains and returns all pdata messages emitted by the processor via the effect handler.
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
    P: Processor<PData, SendEffectHandler<PData>>
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

impl<PData,P> TestContext<PData> for SendTestContext<PData, P>
where
    P: Processor<PData, SendEffectHandler<PData>>
{
    /// Calls the processor's process method with the given message.
    async fn process(&mut self, msg: Message<PData>) -> Result<(), Error<PData>> {
        self.processor.process(msg, &mut self.effect_handler).await
    }

    /// Drains and returns all pdata messages emitted by the processor via the effect handler.
    async fn drain_pdata(&mut self) -> Vec<PData> {
        let mut emitted = Vec::new();
        while let Ok(msg) = self.pdata_receiver.try_recv() {
            emitted.push(msg);
        }
        emitted
    }
}

impl<PData> NotSendValidateContext<PData> {
    /// Receives a pdata message produced by the receiver.
    pub async fn recv(&mut self) -> Result<PData, Error<PData>> {
        self.pdata_receiver
            .recv()
            .await
            .map_err(|e| Error::ChannelRecvError(e))
    }

    /// Drains and returns all pdata messages emitted by the processor via the effect handler.
    pub async fn drain_pdata(&mut self) -> Vec<PData> {
        let mut emitted = Vec::new();
        while let Ok(msg) = self.pdata_receiver.try_recv() {
            emitted.push(msg);
        }
        emitted
    }

    /// Returns the control message counters.
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

    /// Drains and returns all pdata messages emitted by the processor via the effect handler.
    pub async fn drain_pdata(&mut self) -> Vec<PData> {
        let mut emitted = Vec::new();
        while let Ok(msg) = self.pdata_receiver.try_recv() {
            emitted.push(msg);
        }
        emitted
    }

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
pub struct NonSendTestPhase<PData> {
    name: String,
    channel_capacity: usize,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    processor: Option<Box<dyn Processor<PData, NotSendEffectHandler<PData>>>>,
    counters: CtrlMsgCounters,

    pdata_sender: mpsc::Sender<PData>,
    pdata_receiver: mpsc::Receiver<PData>,
}

/// Data and operations for the validation phase of a processor (not sendable effect handler).
pub struct NotSendValidationPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    counters: CtrlMsgCounters,

    pdata_receiver: mpsc::Receiver<PData>,
}

/// Data and operations for the validation phase of a processor (sendable effect handler).
pub struct SendTestPhase<PData> {
    name: String,
    channel_capacity: usize,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    processor: Option<Box<dyn Processor<PData, SendEffectHandler<PData>>>>,
    counters: CtrlMsgCounters,

    pdata_sender: tokio::sync::mpsc::Sender<PData>,
    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
}

/// Data and operations for the validation phase of a processor (sendable effect handler).
pub struct SendValidationPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    counters: CtrlMsgCounters,

    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
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
        mut self,
        processor: P,
        name: &str,
    ) -> NonSendTestPhase<PData>
    where
        P: Processor<PData, NotSendEffectHandler<PData>> + 'static,
    {
        let (pdata_sender, pdata_receiver) = mpsc::Channel::new(self.channel_capacity);

        NonSendTestPhase {
            channel_capacity: self.channel_capacity,
            name: name.to_owned(),
            rt: self.rt,
            local_tasks: self.local_tasks,
            processor: Some(Box::new(processor)),
            counters: self.counter,
            pdata_sender,
            pdata_receiver,
        }
    }

    /// Initializes the test runtime with a processor using a sendable effect handler.
    pub fn processor_with_send_effect_handler<P>(
        mut self,
        processor: P,
        name: &str,
    ) -> SendTestPhase<PData>
    where
        P: Processor<PData, SendEffectHandler<PData>> + 'static,
    {
        let (pdata_sender, pdata_receiver) = tokio::sync::mpsc::channel(self.channel_capacity);

        SendTestPhase {
            channel_capacity: self.channel_capacity,
            name: name.to_owned(),
            rt: self.rt,
            local_tasks: self.local_tasks,
            processor: Some(Box::new(processor)),
            counters: self.counter,
            pdata_sender,
            pdata_receiver,
        }
    }
}

impl<PData: Debug + 'static> NonSendTestPhase<PData> {
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<P, F, Fut>(mut self, f: F) -> NotSendValidationPhase<PData>
    where
        F: FnOnce(NotSendTestContext<PData, P>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
        P: Processor<PData, NotSendEffectHandler<PData>>,
    {
        let processor = self.processor.take().expect("Processor not set");
        let context = NotSendTestContext::new(processor, &self.name, self.channel_capacity);
        let _ = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
        NotSendValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
            pdata_receiver: self.pdata_receiver,
        }
    }
}

impl<PData: Debug + 'static> SendTestPhase<PData> {
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<P, F, Fut>(mut self, f: F) -> SendValidationPhase<PData>
    where
        F: FnOnce(SendTestContext<PData, P>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
        P: Processor<PData, NotSendEffectHandler<PData>>,
    {
        let processor = self.processor.take().expect("Processor not set");
        let context = SendTestContext::new(processor, &self.name, self.channel_capacity);
        let _ = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
        SendValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
            pdata_receiver: self.pdata_receiver,
        }
    }
}

impl<PData> NotSendValidationPhase<PData> {
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
        F: FnOnce(NotSendValidateContext<PData>) -> Fut,
        Fut: Future<Output = T>,
    {
        let context = NotSendValidateContext {
            pdata_receiver: self.pdata_receiver,
            counters: self.counters,
        };

        // First run all the spawned tasks to completion
        self.rt.block_on(self.local_tasks);

        // Then run the validation future with the test context
        self.rt.block_on(future_fn(context))
    }
}

impl<PData> SendValidationPhase<PData> {
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
        F: FnOnce(SendValidateContext<PData>) -> Fut,
        Fut: Future<Output = T>,
    {
        let context = SendValidateContext {
            pdata_receiver: self.pdata_receiver,
            counters: self.counters,
        };

        // First run all the spawned tasks to completion
        self.rt.block_on(self.local_tasks);

        // Then run the validation future with the test context
        self.rt.block_on(future_fn(context))
    }
}
