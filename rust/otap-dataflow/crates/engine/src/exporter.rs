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

use crate::NodeName;
use crate::error::Error;
use crate::message::{ControlMsg, Message};
use crate::receiver::{LocalMode, SendableMode, ThreadMode};
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_channel::mpsc;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// A trait for egress exporters.
#[async_trait(?Send)]
pub trait Exporter {
    /// The type of messages handled by the exporter.
    type PData;

    /// The threading mode used by this exporter
    type Mode: ThreadMode;

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
        msg_chan: MessageChannel<Self::PData>,
        effect_handler: EffectHandler<Self::PData, Self::Mode>,
    ) -> Result<(), Error<Self::PData>>;
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

/// Handles side effects for the exporter.
///
/// The `Msg` type parameter represents the type of message the exporter will consume.
///
/// Note for implementers: The `EffectHandler` is designed to be cloned and shared across tasks
/// so the cost of cloning should be minimal.
pub struct EffectHandler<Msg, Mode: ThreadMode = LocalMode> {
    /// The name of the exporter.
    exporter_name: Mode::NameRef,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the exporter
    /// will consume.
    pd: PhantomData<Msg>,

    /// Marker for the thread mode.
    _mode: PhantomData<Mode>,
}

impl<Msg, Mode: ThreadMode> Clone for EffectHandler<Msg, Mode> {
    fn clone(&self) -> Self {
        EffectHandler {
            exporter_name: self.exporter_name.clone(),
            pd: self.pd,
            _mode: PhantomData,
        }
    }
}

// Implementation for any mode
impl<Msg, Mode: ThreadMode> EffectHandler<Msg, Mode> {
    /// Returns the name of the exporter associated with this handler.
    #[must_use]
    pub fn exporter_name(&self) -> NodeName {
        // Convert to NodeName (Rc<str>) to maintain compatibility with existing API
        Rc::from(self.exporter_name.as_ref())
    }
}

// Implementation specific to LocalMode (default, non-Send)
impl<Msg> EffectHandler<Msg, LocalMode> {
    /// Creates a new local (non-Send) `EffectHandler` with the given exporter name.
    /// This is the default mode that maintains backward compatibility.
    pub fn new<S: AsRef<str>>(exporter_name: S) -> Self {
        EffectHandler {
            exporter_name: Rc::from(exporter_name.as_ref()),
            pd: PhantomData,
            _mode: PhantomData,
        }
    }
}

// Implementation for SendableMode (Send)
impl<Msg: Send + 'static> EffectHandler<Msg, SendableMode> {
    /// Creates a new thread-safe (Send) `EffectHandler` with the given exporter name.
    /// Use this when you need an EffectHandler that can be sent across thread boundaries.
    pub fn new_sendable<S: AsRef<str>>(exporter_name: S) -> Self {
        EffectHandler {
            exporter_name: Arc::from(exporter_name.as_ref()),
            pd: PhantomData,
            _mode: PhantomData,
        }
    }
}

// Note for reviewers: More methods will be added in future PRs.

#[cfg(test)]
mod tests {
    use crate::exporter::{EffectHandler, Error, Exporter, MessageChannel};
    use crate::message::{ControlMsg, Message};
    use crate::receiver::LocalMode;
    use crate::testing::exporter::ExporterTestRuntime;
    use crate::testing::{CtrMsgCounters, TestMsg};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::time::Duration;

    /// A test exporter that counts how many `TimerTick` and Message events it processes.
    struct TestExporter {
        /// Counter for different message types
        counter: CtrMsgCounters,
    }

    impl TestExporter {
        /// Creates a new test exporter with the given counter.
        pub fn new(counter: CtrMsgCounters) -> Self {
            TestExporter { counter }
        }
    }

    #[async_trait(?Send)]
    impl Exporter for TestExporter {
        type PData = TestMsg;
        type Mode = LocalMode;

        async fn start(
            self: Box<Self>,
            mut msg_chan: MessageChannel<Self::PData>,
            effect_handler: EffectHandler<Self::PData, Self::Mode>,
        ) -> Result<(), Error<Self::PData>> {
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
                            exporter: effect_handler.exporter_name(),
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
        let exporter = TestExporter::new(test_runtime.counters());

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
            ctx.send_data("Hello Exporter")
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
