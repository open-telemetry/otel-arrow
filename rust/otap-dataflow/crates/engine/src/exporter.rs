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

use crate::config::ExporterConfig;
use crate::error::Error;
use crate::message::{ControlMsg, Message};
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_channel::mpsc;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

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
    pub async fn start(self, message_channel: MessageChannel<PData>) -> Result<(), Error<PData>> {
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
/// Control messages are always prioritized _until_ a `Shutdown` arrives.
/// After a `Shutdown` control message, pdata messages become prioritized
/// (i.e. are drained first) up to the shutdown deadline.
pub struct MessageChannel<PData> {
    control_rx: mpsc::Receiver<ControlMsg>,
    pdata_rx: mpsc::Receiver<PData>,
    /// Once a Shutdown is seen, this is set to `Some(instant)` at which point
    /// no more pdata will be accepted.
    shutting_down_deadline: Option<Instant>,
    /// Holds the ControlMsg::Shutdown until after we’ve drained pdata.
    pending_shutdown: Option<ControlMsg>,
}

impl<PData> MessageChannel<PData> {
    /// Creates a new `MessageChannel` with the given control and data receivers.
    #[must_use]
    pub fn new(control_rx: mpsc::Receiver<ControlMsg>, pdata_rx: mpsc::Receiver<PData>) -> Self {
        MessageChannel {
            control_rx,
            pdata_rx,
            shutting_down_deadline: None,
            pending_shutdown: None,
        }
    }

    /// Asynchronously receives the next message to process.
    ///
    /// Before any `Shutdown`: control messages are always
    /// polled before pdata.
    ///
    /// On a `Shutdown` control msg: returns `Message::Control(Shutdown)`
    /// and records a deadline (`Instant::now() + deadline`).
    ///
    /// After shutdown has started: pdata messages are polled
    /// before control, until the deadline expires, at which point
    /// this method returns an error.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if both channels are closed, or if the
    /// shutdown deadline has passed.
    pub async fn recv(&mut self) -> Result<Message<PData>, RecvError> {
        loop {
            // ——— Draining mode: Shutdown pending ———
            if let Some(dl) = self.shutting_down_deadline {
                // If the deadline has passed, emit the pending Shutdown now.
                if Instant::now() >= dl {
                    let shutdown = self
                        .pending_shutdown
                        .take()
                        .expect("pending_shutdown must exist");
                    self.shutting_down_deadline = None;
                    return Ok(Message::Control(shutdown));
                }

                // Drain pdata first, then timer, then other control msgs
                tokio::select! {
                    biased;

                    // 1) Any pdata?
                    pdata = self.pdata_rx.recv() => match pdata {
                        Ok(d) => return Ok(Message::PData(d)),
                        Err(_) => {
                            // pdata channel closed → emit Shutdown
                            let shutdown = self.pending_shutdown
                                .take()
                                .expect("pending_shutdown must exist");
                            self.shutting_down_deadline = None;
                            return Ok(Message::Control(shutdown));
                        }
                    },

                    // 2) Deadline hit?
                    _ = tokio::time::sleep_until(dl) => {
                        let shutdown = self.pending_shutdown
                            .take()
                            .expect("pending_shutdown must exist");
                        self.shutting_down_deadline = None;
                        return Ok(Message::Control(shutdown));
                    },

                    // 3) Still accept other control messages
                    ctrl = self.control_rx.recv() => {
                        return ctrl.map(Message::Control);
                    }
                }
            }

            // ——— Normal mode: no shutdown yet ———
            tokio::select! {
                biased;

                // A) Control first
                ctrl = self.control_rx.recv() => match ctrl {
                    Ok(ControlMsg::Shutdown { deadline, reason }) if !deadline.is_zero() => {
                        // Begin draining mode, but don’t return Shutdown yet
                        let when = Instant::now() + deadline;
                        self.shutting_down_deadline = Some(when);
                        self.pending_shutdown = Some(ControlMsg::Shutdown { deadline: Duration::from_millis(0), reason });
                        continue; // re-enter the loop into draining mode
                    }
                    Ok(msg) => return Ok(Message::Control(msg)),
                    Err(e)  => return Err(e),
                },

                // B) Then pdata
                pdata = self.pdata_rx.recv() => {
                    return pdata.map(Message::PData);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::exporter::{
        EffectHandlerTrait, Error, Exporter, ExporterWrapper, MessageChannel, NotSendEffectHandler,
        SendEffectHandler,
    };
    use crate::message::{ControlMsg, Message};
    use crate::testing::exporter::TestContext;
    use crate::testing::exporter::TestRuntime;
    use crate::testing::{CtrlMsgCounters, TestMsg, exec_in_send_env};
    use async_trait::async_trait;
    use otap_df_channel::error::RecvError;
    use otap_df_channel::mpsc;
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
    fn scenario() -> impl FnOnce(TestContext<TestMsg>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
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
                ctx.send_pdata(TestMsg("Hello Exporter".into()))
                    .await
                    .expect("Failed to send data message");

                // Send shutdown
                ctx.send_shutdown(Duration::from_millis(200), "test complete")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure()
    -> impl FnOnce(TestContext<TestMsg>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
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
        let test_runtime = TestRuntime::new();
        let exporter = ExporterWrapper::with_not_send(
            ExporterWithNotSendEffectHandler::without_send_test(test_runtime.counters()),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure());
    }

    #[test]
    fn test_exporter_with_send_effect_handler() {
        let test_runtime = TestRuntime::new();
        let exporter = ExporterWrapper::with_send(
            ExporterWithSendEffectHandler::with_send_test(
                test_runtime.counters(),
                |effect_handler| {
                    exec_in_send_env(|| {
                        _ = effect_handler.exporter_name();
                    });
                },
            ),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure());
    }

    #[tokio::test]
    async fn test_control_priority() {
        let (control_tx, control_rx) = mpsc::Channel::<ControlMsg>::new(10);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(10);
        let mut channel = MessageChannel::new(control_rx, pdata_rx);

        control_tx
            .send_async(ControlMsg::Ack { id: 1 })
            .await
            .unwrap();
        pdata_tx.send_async("pdata1".to_string()).await.unwrap();

        // Control message should be received first due to bias
        let msg = channel.recv().await.unwrap();
        assert!(matches!(msg, Message::Control(ControlMsg::Ack { id: 1 })));

        // Then pdata message
        let msg = channel.recv().await.unwrap();
        assert!(matches!(msg, Message::PData(ref s) if s == "pdata1"));

        drop(control_tx);
        drop(pdata_tx);
    }

    #[tokio::test]
    async fn test_shutdown_drain() {
        let (control_tx, control_rx) = mpsc::Channel::<ControlMsg>::new(10);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(10);
        let mut channel = MessageChannel::new(control_rx, pdata_rx);

        // Pre-load pdata
        pdata_tx.send_async("pdata1".to_string()).await.unwrap();
        pdata_tx.send_async("pdata2".to_string()).await.unwrap();

        // Send shutdown with a deadline
        control_tx
            .send_async(ControlMsg::Shutdown {
                deadline: Duration::from_millis(100), // 100ms deadline
                reason: "Test Shutdown".to_string(),
            })
            .await
            .unwrap();

        // Send more pdata *after* shutdown is sent, but before receiver likely gets it
        pdata_tx.send_async("pdata3".to_string()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await; // Give receiver a chance to see shutdown
        pdata_tx
            .send_async("pdata4_during_drain".to_string())
            .await
            .unwrap();

        // --- Start Receiving ---

        // 1. Should receive pdata1 (drain)
        let msg1 = channel.recv().await.unwrap();
        println!("Received: {:?}", msg1);
        assert!(matches!(msg1, Message::PData(ref s) if s == "pdata1"));

        // 2. Should receive pdata2 (drain)
        let msg2 = channel.recv().await.unwrap();
        println!("Received: {:?}", msg2);
        assert!(matches!(msg2, Message::PData(ref s) if s == "pdata2"));

        // 3. Should receive pdata3 (drain)
        let msg3 = channel.recv().await.unwrap();
        println!("Received: {:?}", msg3);
        assert!(matches!(msg3, Message::PData(ref s) if s == "pdata3"));

        // 4. Should receive pdata4 (drain)
        let msg4 = channel.recv().await.unwrap();
        println!("Received: {:?}", msg4);
        assert!(matches!(msg4, Message::PData(ref s) if s == "pdata4_during_drain"));

        // Wait for deadline to likely expire
        tokio::time::sleep(Duration::from_millis(120)).await; // Wait longer than deadline

        // Send pdata *after* deadline
        // This might get buffered but shouldn't be received before the shutdown msg
        let _ = pdata_tx
            .send_async("pdata5_after_deadline".to_string())
            .await;

        // 5. Now, should receive the Shutdown message itself
        let msg5 = channel.recv().await.unwrap();
        println!("Received: {:?}", msg5);
        assert!(matches!(
            msg5,
            Message::Control(ControlMsg::Shutdown { .. })
        ));

        // Optional: Check if post-deadline message is still there (if channel not closed)
        // If pdata_tx is still alive:
        // let msg6 = channel.recv().await;
        // println!("Received after shutdown signal: {:?}", msg6);
        // assert!(matches!(msg6, Ok(Message::PData(ref s)) if s == "pdata5_after_deadline"));

        drop(control_tx);
        drop(pdata_tx); // Close channels

        // 6. Check for RecvError after channels closed
        let msg_err = channel.recv().await;
        println!("Received after close: {:?}", msg_err);
        assert!(matches!(msg_err, Err(RecvError::Closed)));
    }

    #[tokio::test]
    async fn test_shutdown_drain_pdata_closes() {
        let (control_tx, control_rx) = mpsc::Channel::<ControlMsg>::new(10);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(10);
        let mut channel = MessageChannel::new(control_rx, pdata_rx);

        // Pre-load pdata
        pdata_tx.send_async("pdata1".to_string()).await.unwrap();

        // Send shutdown with a long deadline
        control_tx
            .send_async(ControlMsg::Shutdown {
                deadline: Duration::from_secs(5), // Long deadline
                reason: "Test Shutdown PData Closes".to_string(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await; // Give receiver a chance

        // --- Start Receiving ---

        // 1. Should receive pdata1 (drain)
        let msg1 = channel.recv().await.unwrap();
        println!("Received: {:?}", msg1);
        assert!(matches!(msg1, Message::PData(ref s) if s == "pdata1"));

        // Close the pdata channel during drain
        drop(pdata_tx);

        // 2. Now, should receive the Shutdown message because pdata channel closed
        let msg2 = channel.recv().await.unwrap();
        println!("Received: {:?}", msg2);
        assert!(matches!(
            msg2,
            Message::Control(ControlMsg::Shutdown { .. })
        ));

        drop(control_tx);

        // 3. Check for RecvError after channels closed
        let msg_err = channel.recv().await;
        println!("Received after close: {:?}", msg_err);
        assert!(matches!(msg_err, Err(RecvError::Closed)));
    }

    #[tokio::test]
    async fn test_immediate_shutdown() {
        let (control_tx, control_rx) = mpsc::Channel::<ControlMsg>::new(10);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(10);
        let mut channel = MessageChannel::new(control_rx, pdata_rx);

        pdata_tx.send_async("pdata1".to_string()).await.unwrap();
        control_tx
            .send_async(ControlMsg::Shutdown {
                deadline: Duration::from_secs(0), // Immediate deadline
                reason: "Immediate Shutdown".to_string(),
            })
            .await
            .unwrap();

        // Should immediately receive the shutdown message, no draining
        let msg1 = channel.recv().await.unwrap();
        println!("Received: {:?}", msg1);
        assert!(matches!(
            msg1,
            Message::Control(ControlMsg::Shutdown { .. })
        ));

        // Pdata should still be in the channel if not closed
        let msg2 = channel.recv().await.unwrap();
        println!("Received after immediate shutdown signal: {:?}", msg2);
        assert!(matches!(msg2, Message::PData(ref s) if s == "pdata1"));

        drop(control_tx);
        drop(pdata_tx);
    }
}
