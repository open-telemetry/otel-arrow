// SPDX-License-Identifier: Apache-2.0

//! Set of traits and structures used to implement exporters.
//!
//! An exporter is an egress node that sends data from a pipeline to external systems, performing
//! the necessary conversions from the internal pdata format to the format required by the external
//! system.
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
//! 3. The exporter processes both internal control messages and pipeline data (pdata)
//! 4. The exporter shuts down when it receives a `Shutdown` control message or encounters a fatal
//!    error
//!
//! # Thread Safety
//!
//! Note that this trait uses `#[async_trait(?Send)]`, meaning implementations are not required to
//! be thread-safe. If you need to implement an exporter that requires `Send`, you can use the
//! [`SendEffectHandler`] type. The default effect handler is `!Send` (see
//! [`NotSendEffectHandler`]).
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own exporter instance.

use crate::error::Error;
use crate::message::{ControlMsg, Message};
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_channel::mpsc;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use crate::config::ExporterConfig;

/// A trait for egress exporters.
#[async_trait(?Send)]
pub trait Exporter<PData, EF = NotSendEffectHandler<PData>>
where
    EF: EffectHandlerTrait<PData>,
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
    /// yield control back to the pipeline engine. It must independently manage its outputs and
    /// processing timing. The only way the pipeline engine can interact with the exporter after
    /// starting it is through the control message channel.
    ///
    /// Exporters are expected to process both internal control messages and pipeline data messages,
    /// prioritizing control messages over data messages. This priorization guaranty is ensured by
    /// the `MessageChannel` implementation.
    ///
    /// # Parameters
    ///
    /// - `msg_chan`: A channel to receive pdata or control messages. Control messages are
    ///   prioritized over pdata messages.
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
/// 2 implementations are provided:
///
/// - [`NotSendEffectHandler<PData>`]: For thread-local (!Send) exporters. Uses `Rc` internally.
///   It's the default and preferred effect handler.
/// - [`SendEffectHandler<PData>`]: For thread-safe (Send) exporters. Uses `Arc` internally and
///   supports sending across thread boundaries.
///
/// Note for implementers: Effect handler implementations are designed to be cloned so the cost of
/// cloning should be minimal.
pub trait EffectHandlerTrait<PData> {
    /// Returns the name of the exporter associated with this handler.
    fn exporter_name(&self) -> &str;

    // More methods will be added in the future as needed.
}

/// A `!Send` implementation of the EffectHandlerTrait.
#[derive(Clone)]
pub struct NotSendEffectHandler<PData> {
    /// The name of the exporter.
    exporter_name: Rc<str>,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the exporter
    /// will consume.
    _pd: PhantomData<PData>,
}

/// Implementation for the `!Send` effect handler.
impl<PData> NotSendEffectHandler<PData> {
    /// Creates a new local (!Send) `EffectHandler` with the given exporter name.
    /// This is the default and preferred effect handler for this project.
    ///
    /// Use this constructor when your exporter doesn't need to be sent across threads or
    /// when it uses components that aren't `Send`.
    pub fn new<S: AsRef<str>>(exporter_name: S) -> Self {
        NotSendEffectHandler {
            exporter_name: Rc::from(exporter_name.as_ref()),
            _pd: PhantomData,
        }
    }
}

impl<PData> EffectHandlerTrait<PData> for NotSendEffectHandler<PData> {
    /// Returns the name of the exporter associated with this handler.
    #[must_use]
    fn exporter_name(&self) -> &str {
        &self.exporter_name
    }
}

/// A `Send` implementation of the EffectHandlerTrait.
#[derive(Clone)]
pub struct SendEffectHandler<PData> {
    /// The name of the exporter.
    exporter_name: Arc<str>,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the exporter
    /// will consume.
    _pd: PhantomData<PData>,
}

/// Implementation for the `Send` effect handler.
impl<PData> SendEffectHandler<PData> {
    /// Creates a new "sendable" effect handler with the given exporter name.
    pub fn new<S: AsRef<str>>(exporter_name: S) -> Self {
        SendEffectHandler {
            exporter_name: Arc::from(exporter_name.as_ref()),
            _pd: PhantomData,
        }
    }
}

impl<PData> EffectHandlerTrait<PData> for SendEffectHandler<PData> {
    /// Returns the name of the exporter associated with this handler.
    #[must_use]
    fn exporter_name(&self) -> &str {
        &self.exporter_name
    }
}

/// A wrapper for the exporter that allows for both `Send` and `!Send` effect handlers.
///
/// Note: This is useful for creating a single interface for the exporter regardless of the effect
/// handler type. This is the only type that the pipeline engine will use in order to be agnostic to
/// the effect handler type.
pub enum ExporterWrapper<PData> {
    /// A receiver with a `!Send` effect handler.
    NotSend {
        /// The exporter instance.
        exporter: Box<dyn Exporter<PData, NotSendEffectHandler<PData>>>,
        /// The effect handler instance for the exporter.
        effect_handler: NotSendEffectHandler<PData>,
    },
    /// A receiver with a `Send` effect handler.
    Send {
        /// The exporter instance.
        exporter: Box<dyn Exporter<PData, SendEffectHandler<PData>>>,
        /// The effect handler instance for the exporter.
        effect_handler: SendEffectHandler<PData>,
    },
}

impl<PData> ExporterWrapper<PData> {
    /// Creates a new `ExporterWrapper` with the given exporter and `!Send` effect handler.
    pub fn with_not_send<E>(exporter: E, config: &ExporterConfig) -> Self
    where
        E: Exporter<PData, NotSendEffectHandler<PData>> + 'static,
    {
        ExporterWrapper::NotSend {
            exporter: Box::new(exporter),
            effect_handler: NotSendEffectHandler::new(&config.name),
        }
    }

    /// Creates a new `ExporterWrapper` with the given exporter and `Send` effect handler.
    pub fn with_send<E>(exporter: E, config: &ExporterConfig) -> Self
    where
        E: Exporter<PData, SendEffectHandler<PData>> + 'static,
    {
        ExporterWrapper::Send {
            exporter: Box::new(exporter),
            effect_handler: SendEffectHandler::new(&config.name),
        }
    }

    /// Starts the exporter and begins exporting incoming data.
    pub(crate) async fn start(
        self,
        message_channel: MessageChannel<PData>,
    ) -> Result<(), Error<PData>> {
        match self {
            ExporterWrapper::NotSend {
                effect_handler,
                exporter,
            } => exporter.start(message_channel, effect_handler).await,
            ExporterWrapper::Send {
                effect_handler,
                exporter,
            } => exporter.start(message_channel, effect_handler).await,
        }
    }
}

/// A channel for receiving control and pdata messages.
///
/// Note: Control messages are prioritized over pdata messages.
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

#[cfg(test)]
mod tests {
    use crate::exporter::{EffectHandlerTrait, Error, Exporter, ExporterWrapper, MessageChannel, NotSendEffectHandler, SendEffectHandler};
    use crate::message::{ControlMsg, Message};
    use crate::testing::exporter::ExporterTestContext;
    use crate::testing::exporter::ExporterTestRuntime;
    use crate::testing::{CtrlMsgCounters, TestMsg, exec_in_send_env};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::future::Future;
    use std::time::Duration;

    /// A generic test exporter that counts message events
    /// Works with any effect handler that implements EffectHandlerTrait
    pub struct GenericTestExporter<EF> {
        /// Counter for different message types
        pub counter: CtrlMsgCounters,
        /// Optional callback for testing sendable effect handlers
        pub test_send_ef: Option<fn(&EF)>,
    }

    impl<EF> GenericTestExporter<EF> {
        /// Creates a new test node with the given counter
        pub fn without_send_test(counter: CtrlMsgCounters) -> Self {
            GenericTestExporter {
                counter,
                test_send_ef: None,
            }
        }

        /// Creates a new test node with a callback for PData messages
        pub fn with_send_test(counter: CtrlMsgCounters, callback: fn(&EF)) -> Self {
            GenericTestExporter {
                counter,
                test_send_ef: Some(callback),
            }
        }
    }

    #[async_trait(?Send)]
    impl<EF> Exporter<TestMsg, EF> for GenericTestExporter<EF>
    where
        EF: EffectHandlerTrait<TestMsg> + Clone,
    {
        async fn start(
            self: Box<Self>,
            mut msg_chan: MessageChannel<TestMsg>,
            effect_handler: EF,
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
                        // Execute optional callback if present
                        if let Some(callback) = self.test_send_ef {
                            callback(&effect_handler);
                        }
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

    /// A type alias for a test exporter with regular effect handler
    type ExporterWithNotSendEffectHandler = GenericTestExporter<NotSendEffectHandler<TestMsg>>;

    /// A type alias for a test exporter with sendable effect handler
    type ExporterWithSendEffectHandler = GenericTestExporter<SendEffectHandler<TestMsg>>;

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    fn scenario()
    -> impl FnOnce(ExporterTestContext<TestMsg>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
    {
        |ctx| {
            Box::pin(async move {
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
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure() -> impl FnOnce(ExporterTestContext<TestMsg>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
    {
        |ctx| {
            Box::pin(async move {
                ctx.counters().assert(
                    3, // timer tick
                    1, // message
                    1, // config
                    1, // shutdown
                );
            })
        }
    }

    #[test]
    fn test_exporter_with_not_send_effect_handler() {
        let test_runtime = ExporterTestRuntime::new();
        let exporter = ExporterWrapper::with_not_send(
            ExporterWithNotSendEffectHandler::without_send_test(test_runtime.counters()),
            test_runtime.config()
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure());
    }

    #[test]
    fn test_exporter_with_send_effect_handler() {
        let test_runtime = ExporterTestRuntime::new();
        let exporter = ExporterWrapper::with_send(
            ExporterWithSendEffectHandler::with_send_test(
                test_runtime.counters(),
                |effect_handler| {
                    exec_in_send_env(|| {
                        _ = effect_handler.exporter_name();
                    });
                },
            ),
            test_runtime.config()
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure());
    }
}
