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
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_channel::mpsc;
use std::rc::Rc;

/// A trait for egress exporters.
#[async_trait(?Send)]
pub trait Exporter {
    /// The type of messages handled by the exporter.
    type Msg;

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
        msg_chan: MessageChannel<Self::Msg>,
        effect_handler: EffectHandler<Self::Msg>,
    ) -> Result<(), Error<Self::Msg>>;
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
pub struct EffectHandler<Msg> {
    /// The name of the exporter.
    exporter_name: NodeName,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the exporter
    /// will consume.
    pd: std::marker::PhantomData<Msg>,
}

impl<Msg> Clone for EffectHandler<Msg> {
    fn clone(&self) -> Self {
        EffectHandler {
            exporter_name: self.exporter_name.clone(),
            pd: self.pd,
        }
    }
}

impl<Msg> EffectHandler<Msg> {
    /// Creates a new `EffectHandler` with the given exporter name.
    pub fn new<S: AsRef<str>>(exporter_name: S) -> Self {
        EffectHandler {
            exporter_name: Rc::from(exporter_name.as_ref()),
            pd: std::marker::PhantomData,
        }
    }

    /// Returns the name of the exporter associated with this handler.
    #[must_use]
    pub fn exporter_name(&self) -> &str {
        &self.exporter_name
    }

    // Note for reviewers: More methods will be added in future PRs.
}

#[cfg(test)]
mod tests {
    use crate::exporter::{EffectHandler, Error, Exporter, MessageChannel};
    use crate::message::{ControlMsg, Message};
    use async_trait::async_trait;
    use otap_df_channel::mpsc;
    use serde_json::Value;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;
    use tokio::runtime::Builder;
    use tokio::task::LocalSet;
    use tokio::time::sleep;

    /// A test message.
    #[derive(Debug, PartialEq)]
    struct TestMsg(String);

    /// A test exporter that counts how many `TimerTick` and Message events it processes.
    struct TestExporter {
        timer_tick_count: Rc<RefCell<usize>>,
        message_count: Rc<RefCell<usize>>,
        config_count: Rc<RefCell<usize>>,
    }

    #[async_trait(?Send)]
    impl Exporter for TestExporter {
        type Msg = TestMsg;

        async fn start(
            self: Box<Self>,
            mut msg_chan: MessageChannel<Self::Msg>,
            _effect_handler: EffectHandler<Self::Msg>,
        ) -> Result<(), Error<Self::Msg>> {
            // Loop until a Shutdown event is received.
            loop {
                match msg_chan.recv().await? {
                    Message::Control(ControlMsg::TimerTick { .. }) => {
                        println!("Exporter received TimerTick event.");
                        *self.timer_tick_count.borrow_mut() += 1;
                    }
                    Message::Control(ControlMsg::Config { .. }) => {
                        println!("Exporter received Config event.");
                        *self.config_count.borrow_mut() += 1;
                    }
                    Message::PData(message) => {
                        println!("Exporter received Message event: {message:?}");
                        *self.message_count.borrow_mut() += 1;
                    }
                    Message::Control(ControlMsg::Shutdown { .. }) => {
                        println!("Exporter received Shutdown event.");
                        break;
                    }
                    _ => {
                        println!("Exporter received unknown message.");
                    }
                }
            }
            println!("Exporter event loop terminated.");
            Ok(())
        }
    }

    #[test]
    fn test_exporter() {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        let local_tasks = LocalSet::new();

        // Create shared counters to keep track of events.
        let timer_tick_count = Rc::new(RefCell::new(0));
        let message_count = Rc::new(RefCell::new(0));
        let config_count = Rc::new(RefCell::new(0));

        // Create an MPSC channel for events.
        let (control_tx, control_rx) = mpsc::Channel::new(10);
        let (pdata_tx, pdata_rx) = mpsc::Channel::new(10);
        let msg_chan = MessageChannel::new(control_rx, pdata_rx);

        // Create the exporter instance.
        let exporter = Box::new(TestExporter {
            timer_tick_count: timer_tick_count.clone(),
            message_count: message_count.clone(),
            config_count: config_count.clone(),
        });

        // Spawn the exporter's event loop.
        _ = local_tasks.spawn_local(async move {
            exporter
                .start(msg_chan, EffectHandler::new("test_exporter"))
                .await
                .expect("Exporter event loop failed");
        });

        // Spawn a task to simulate sending events to the exporter.
        _ = local_tasks.spawn_local(async move {
            // Send 3 TimerTick events.
            for _ in 0..3 {
                let result = control_tx.send_async(ControlMsg::TimerTick {}).await;
                assert!(result.is_ok(), "Failed to send TimerTick event");
                sleep(Duration::from_millis(50)).await;
            }

            // Send a Config event.
            let result = control_tx
                .send_async(ControlMsg::Config {
                    config: Value::Null,
                })
                .await;
            assert!(result.is_ok(), "Failed to send Config event");

            // Send a Message event.
            let test_msg = TestMsg("Hello exporter".to_string());
            let result = pdata_tx.send_async(test_msg).await;
            assert!(result.is_ok(), "Failed to send Message event");
            sleep(Duration::from_millis(50)).await;
            // Finally, send a Shutdown event to terminate the event loop.
            let result = control_tx
                .send_async(ControlMsg::Shutdown {
                    reason: "end of test".to_owned(),
                })
                .await;
            assert!(result.is_ok(), "Failed to send Shutdown event");
        });

        // Run all tasks.
        rt.block_on(local_tasks);

        // After the event loop completes, assert that the expected number of events was processed.
        assert_eq!(*timer_tick_count.borrow(), 3, "Expected 3 TimerTick events");
        assert_eq!(*config_count.borrow(), 1, "Expected 1 Config event");
        assert_eq!(*message_count.borrow(), 1, "Expected 1 Message event");
    }
}
