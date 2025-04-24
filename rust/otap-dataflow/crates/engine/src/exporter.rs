// SPDX-License-Identifier: Apache-2.0

//! Set of traits and structures used to implement exporters.
//!
//! An exporter is an egress node that sends data from a pipeline to external systems, performing
//! the necessary conversions from the internal format to the format required by the external system.
//!
//! Exporters can operate in various ways, including:
//!
//! 1. Sending telemetry data to remote endpoints via network protocols,
//! 2. Writing data to files or databases,
//! 3. Pushing data to message queues or event buses,
//! 4. Or any other method of exporting telemetry data to external systems.
//!
//! # Lifecycle
//!
//! 1. The exporter is instantiated and configured
//! 2. The `start` method is called, which begins the exporter's operation
//! 3. The exporter processes both internal control messages and pipeline data
//! 4. The exporter shuts down when it receives a `Shutdown` control message or encounters a fatal error
//!
//! # Thread Safety
//!
//! Note that this trait uses `#[async_trait(?Send)]`, meaning implementations
//! are not required to be thread-safe. To ensure scalability, the pipeline engine will start
//! multiple instances of the same pipeline in parallel, each with its own exporter instance.
//!
//! Through the `Mode` type parameter, exporters can be configured to be either thread-local (`LocalMode`)
//! or thread-safe (`SendableMode`). This allows you to choose the appropriate threading model based on
//! your exporter's requirements and performance considerations.

use crate::error::Error;
use crate::message::{ControlMsg, Message};
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_channel::mpsc;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// A trait for egress exporters.
#[async_trait(?Send)]
pub trait Exporter<PData, EF = EffectHandler<PData>>
where
    PData: Clone,
    EF: EffectHandlerTrait<PData>
{
    /// Starts the exporter and begins exporting incoming data.
    ///
    /// The pipeline engine will call this function to start the exporter in a separate task.
    /// Exporters are assigned their own dedicated task at pipeline initialization because their
    /// primary function involves interacting with the external world, and the pipeline has no
    /// prior knowledge of when these interactions will occur.
    ///
    /// The `Box<Self>` signature indicates that when this method is called, the exporter takes
    /// exclusive ownership of its instance. This approach is necessary because an exporter cannot
    /// yield control back to the pipeline engine - it must independently manage its outputs and
    /// processing timing. The only way the pipeline engine can interact with the exporter after
    /// starting it is through the control message channel.
    ///
    /// Exporters are expected to process both internal control messages and pipeline data messages,
    /// prioritizing control messages over data messages.
    ///
    /// # Parameters
    ///
    /// - `msg_chan`: A channel to receive pdata or control messages.
    /// - `effect_handler`: A handler to perform side effects such as network operations.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if an unrecoverable error occurs.
    ///
    /// # Cancellation Safety
    ///
    /// This method should be cancellation safe and clean up any resources when dropped.
    async fn start(
        self: Box<Self>,
        msg_chan: MessageChannel<PData>,
        effect_handler: EF,
    ) -> Result<(), Error<PData>>;
}

/// Handles side effects for the exporter.
///
/// The `PData` type parameter represents the type of message the exporter will consume.
///
/// # Thread Safety Options
///
/// - `EffectHandler<PData>`: For thread-local (!Send) exporters. Uses `Rc` internally and is
///   the default effect handler.
/// - `SendableEffectHandler<PData>`: For thread-safe (Send) exporters. Uses `Arc` internally and
///   supports sending across thread boundaries.
///
/// Note for implementers: Effect handler implementations are designed to be cloned so the cost of
/// cloning should be minimal.
pub trait EffectHandlerTrait<PData: Clone> {
    /// Returns the name of the exporter associated with this handler.
    fn exporter_name(&self) -> &str;
}

/// A `!Send` implementation of the EffectHandlerTrait.
pub struct EffectHandler<PData> {
    /// The name of the exporter.
    exporter_name: Rc<str>,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the exporter
    /// will consume.
    _pd: PhantomData<PData>,
}

impl<PData> Clone for EffectHandler<PData> {
    fn clone(&self) -> Self {
        EffectHandler {
            exporter_name: self.exporter_name.clone(),
            _pd: PhantomData,
        }
    }
}

/// Implementation for the !Send EffectHandler
impl<Msg> EffectHandler<Msg> {
    /// Creates a new local (!Send) `EffectHandler` with the given exporter name.
    /// This is the default and preferred mode for this project.
    ///
    /// Use this constructor when your exporter doesn't need to be sent across threads or
    /// when it uses components that aren't `Send`.
    pub fn new<S: AsRef<str>>(exporter_name: S) -> Self {
        EffectHandler {
            exporter_name: Rc::from(exporter_name.as_ref()),
            _pd: PhantomData,
        }
    }
}

impl<PData: Clone> EffectHandlerTrait<PData> for EffectHandler<PData> {
    /// Returns the name of the exporter associated with this handler.
    #[must_use]
    fn exporter_name(&self) -> &str {
        &self.exporter_name
    }
}


/// A `Send` implementation of the EffectHandlerTrait.
pub struct SendableEffectHandler<PData> {
    /// The name of the exporter.
    exporter_name: Arc<str>,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the exporter
    /// will consume.
    _pd: PhantomData<PData>,
}

impl<PData> Clone for SendableEffectHandler<PData> {
    fn clone(&self) -> Self {
        SendableEffectHandler {
            exporter_name: self.exporter_name.clone(),
            _pd: PhantomData,
        }
    }
}

/// Implementation for the Send EffectHandler
impl<Msg> SendableEffectHandler<Msg> {
    /// Creates a new "sendable" effect handler with the given exporter name.
    pub fn new<S: AsRef<str>>(exporter_name: S) -> Self {
        SendableEffectHandler {
            exporter_name: Arc::from(exporter_name.as_ref()),
            _pd: PhantomData,
        }
    }
}

impl<PData: Clone> EffectHandlerTrait<PData> for SendableEffectHandler<PData> {
    /// Returns the name of the exporter associated with this handler.
    #[must_use]
    fn exporter_name(&self) -> &str {
        &self.exporter_name
    }
}

enum ExporterWrapper<PData> {
    NonSendable {
        effect_handler: EffectHandler<PData>,
        exporter: Box<dyn Exporter<PData, EffectHandler<PData>>>,
    },
    Sendable {
        effect_handler: SendableEffectHandler<PData>,
        exporter: Box<dyn Exporter<PData, SendableEffectHandler<PData>>>,
    }
}

impl<PData: Clone> ExporterWrapper<PData> {
    async fn start(self, message_channel: MessageChannel<PData>) -> Result<(), Error<PData>> {
        match self {
            ExporterWrapper::NonSendable { effect_handler, exporter } => {
                exporter.start(message_channel, effect_handler).await
            },
            ExporterWrapper::Sendable { effect_handler, exporter } => {
                exporter.start(message_channel, effect_handler).await
            },
        }
    }

    // Create method for local handlers
    fn create<E>(exporter: E, name: &str) -> Self
    where
        E: Exporter<PData, EffectHandler<PData>> + 'static
    {
        // Use static dispatch based on the type parameter
        ExporterWrapper::NonSendable {
            effect_handler: EffectHandler { exporter_name: Rc::from(name), _pd: PhantomData },
            exporter: Box::new(exporter),
        }
    }

    // Create method for sendable handlers
    fn create_sendable<E>(exporter: E, name: &str) -> Self
    where
        E: Exporter<PData, SendableEffectHandler<PData>> + 'static
    {
        // Use static dispatch based on the type parameter
        ExporterWrapper::Sendable {
            effect_handler: SendableEffectHandler { exporter_name: Arc::from(name), _pd: PhantomData },
            exporter: Box::new(exporter),
        }
    }
}

/// A channel for receiving control and pdata messages.
pub struct MessageChannel<PData> {
    control_rx: mpsc::Receiver<ControlMsg>,
    pdata_rx: mpsc::Receiver<PData>,
}

impl<PData> MessageChannel<PData> {
    /// Creates a new `MessageChannel` with the given control and data receivers.
    #[must_use]
    pub fn new(control_rx: mpsc::Receiver<ControlMsg>, pdata_rx: mpsc::Receiver<PData>) -> Self {
        MessageChannel {
            control_rx,
            pdata_rx,
        }
    }

    /// Asynchronously receives the next message to process.
    /// Control messages are prioritized over pdata messages.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if both channels are closed.
    pub async fn recv(&mut self) -> Result<Message<PData>, RecvError> {
        tokio::select! {
            biased;     // Instruct the select macro to poll the futures in the order they appear
                        // from top to bottom.

            // Prioritize control messages explicitly
            control_res = self.control_rx.recv() => {
                control_res.map(|ctrl| Message::Control(ctrl))
            }

            pdata_res = self.pdata_rx.recv() => {
                pdata_res.map(|pdata| Message::PData(pdata))
            }
        }
    }
}

// Note for reviewers: More methods will be added in future PRs.

#[cfg(test)]
mod tests {
    use crate::exporter::{EffectHandler, EffectHandlerTrait, Error, Exporter, MessageChannel, SendableEffectHandler};
    use crate::message::{ControlMsg, Message};
    use crate::testing::exporter::ExporterTestRuntime;
    use crate::testing::{exec_in_send_env, CtrMsgCounters, TestMsg};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::time::Duration;

    /// A test exporter that counts how many `TimerTick` and Message events it processes.
    struct RegularExporter {
        /// Counter for different message types
        counter: CtrMsgCounters,
    }

    impl RegularExporter {
        /// Creates a new test exporter with the given counter.
        pub fn new(counter: CtrMsgCounters) -> Self {
            RegularExporter { counter }
        }
    }

    #[async_trait(?Send)]
    impl Exporter<TestMsg> for RegularExporter {
        async fn start(
            self: Box<Self>,
            mut msg_chan: MessageChannel<TestMsg>,
            effect_handler: EffectHandler<TestMsg>,
        ) -> Result<(), Error<TestMsg>> {
            // Loop until a Shutdown event is received.
            loop {
                match msg_chan.recv().await? {
                    Message::Control(ControlMsg::TimerTick { .. }) => {
                        self.counter.increment_timer_tick();
                    }
                    Message::Control(ControlMsg::Config { .. }) => {
                        self.counter.increment_config();
                    }
                    Message::Control(ControlMsg::Shutdown { .. }) => {
                        self.counter.increment_shutdown();
                        break;
                    }
                    Message::PData(_message) => {
                        self.counter.increment_message();
                    }
                    _ => {
                        return Err(Error::ExporterError {
                            exporter: effect_handler.exporter_name().to_owned(),
                            error: "Unknown control message".to_owned(),
                        });
                    }
                }
            }
            Ok(())
        }
    }

    /// A test of an exporter requiring a sendable effect handler that counts how many `TimerTick`
    /// and Message events it processes.
    struct ExporterWithSendableEffectHandler {
        /// Counter for different message types
        counter: CtrMsgCounters,
    }

    impl ExporterWithSendableEffectHandler {
        /// Creates a new test exporter with the given counter.
        pub fn new(counter: CtrMsgCounters) -> Self {
            ExporterWithSendableEffectHandler { counter }
        }
    }

    #[async_trait(?Send)]
    impl Exporter<TestMsg, SendableEffectHandler<TestMsg>> for ExporterWithSendableEffectHandler {
        async fn start(
            self: Box<Self>,
            mut msg_chan: MessageChannel<TestMsg>,
            effect_handler: SendableEffectHandler<TestMsg>,
        ) -> Result<(), Error<TestMsg>> {
            // Loop until a Shutdown event is received.
            loop {
                match msg_chan.recv().await? {
                    Message::Control(ControlMsg::TimerTick { .. }) => {
                        self.counter.increment_timer_tick();
                    }
                    Message::Control(ControlMsg::Config { .. }) => {
                        self.counter.increment_config();
                    }
                    Message::Control(ControlMsg::Shutdown { .. }) => {
                        self.counter.increment_shutdown();
                        break;
                    }
                    Message::PData(_message) => {
                        self.counter.increment_message();
                        exec_in_send_env(|| {
                            _ = effect_handler.exporter_name();
                        });
                    }
                    _ => {
                        return Err(Error::ExporterError {
                            exporter: effect_handler.exporter_name().to_owned(),
                            error: "Unknown control message".to_owned(),
                        });
                    }
                }
            }
            Ok(())
        }
    }

    #[test]
    fn test_exporter() {
        let mut test_runtime = ExporterTestRuntime::new(10);
        let exporter = RegularExporter::new(test_runtime.counters());

        test_runtime.start_exporter(exporter);
        test_runtime.start_test(|ctx| async move {
            // Send 3 TimerTick events.
            for _ in 0..3 {
                ctx.send_timer_tick()
                    .await
                    .expect("Failed to send TimerTick");
                ctx.sleep(Duration::from_millis(50)).await;
            }

            // Send a Config event.
            ctx.send_config(Value::Null)
                .await
                .expect("Failed to send Config");

            // Send a data message
            ctx.send_data(TestMsg("Hello Exporter".into()))
                .await
                .expect("Failed to send data message");

            // Allow some time for processing
            ctx.sleep(Duration::from_millis(100)).await;

            // Send shutdown
            ctx.send_shutdown("test complete")
                .await
                .expect("Failed to send Shutdown");
        });

        // Get a clone of the counters before moving test_runtime into validate
        let counters = test_runtime.counters();

        test_runtime.validate(|_ctx| async move {
            counters.assert(
                3, // timer tick
                1, // message
                1, // config
                1, // shutdown
            );
        });
    }

    #[test]
    fn test_sendable_exporter() {
        let mut test_runtime = ExporterTestRuntime::new(10);
        let exporter = ExporterWithSendableEffectHandler::new(test_runtime.counters());

        test_runtime.start_exporter_with_send_effect_handler(exporter);
        test_runtime.start_test(|ctx| async move {
            // Send 3 TimerTick events.
            for _ in 0..3 {
                ctx.send_timer_tick()
                    .await
                    .expect("Failed to send TimerTick");
                ctx.sleep(Duration::from_millis(50)).await;
            }

            // Send a Config event.
            ctx.send_config(Value::Null)
                .await
                .expect("Failed to send Config");

            // Send a data message
            ctx.send_data(TestMsg("Hello Exporter".into()))
                .await
                .expect("Failed to send data message");

            // Allow some time for processing
            ctx.sleep(Duration::from_millis(100)).await;

            // Send shutdown
            ctx.send_shutdown("test complete")
                .await
                .expect("Failed to send Shutdown");
        });

        // Get a clone of the counters before moving test_runtime into validate
        let counters = test_runtime.counters();

        test_runtime.validate(|_ctx| async move {
            counters.assert(
                3, // timer tick
                1, // message
                1, // config
                1, // shutdown
            );
        });
    }
}
