// SPDX-License-Identifier: Apache-2.0

//! Exporter wrapper used to provide a unified interface to the pipeline engine that abstracts over
//! the fact that exporter implementations may be `!Send` or `Send`.
//!
//! For more details on the `!Send` implementation of an exporter, see [`local::Exporter`].
//! See [`shared::Exporter`] for the Send implementation.

use crate::config::ExporterConfig;
use crate::control::ControlMsg;
use crate::error::Error;
use crate::local::exporter as local;
use crate::message;
use crate::message::Receiver;
use crate::shared::exporter as shared;

/// A wrapper for the exporter that allows for both `Send` and `!Send` effect handlers.
///
/// Note: This is useful for creating a single interface for the exporter regardless of their
/// 'sendability'.
pub enum ExporterWrapper<PData> {
    /// An exporter with a `!Send` implementation.
    Local {
        /// The exporter instance.
        exporter: Box<dyn local::Exporter<PData>>,
        /// The effect handler instance for the exporter.
        effect_handler: local::EffectHandler<PData>,
    },
    /// An exporter with a `Send` implementation.
    Shared {
        /// The exporter instance.
        exporter: Box<dyn shared::Exporter<PData>>,
        /// The effect handler instance for the exporter.
        effect_handler: shared::EffectHandler<PData>,
    },
}

impl<PData> ExporterWrapper<PData> {
    /// Creates a new local `ExporterWrapper` with the given exporter and configuration (!Send
    /// implementation).
    pub fn local<E>(exporter: E, config: &ExporterConfig) -> Self
    where
        E: local::Exporter<PData> + 'static,
    {
        ExporterWrapper::Local {
            effect_handler: local::EffectHandler::new(config.name.clone()),
            exporter: Box::new(exporter),
        }
    }

    /// Creates a new shared `ExporterWrapper` with the given exporter and configuration (Send
    /// implementation).
    pub fn shared<E>(exporter: E, config: &ExporterConfig) -> Self
    where
        E: shared::Exporter<PData> + 'static,
    {
        ExporterWrapper::Shared {
            effect_handler: shared::EffectHandler::new(config.name.clone()),
            exporter: Box::new(exporter),
        }
    }

    /// Starts the exporter and begins exporting incoming data.
    pub async fn start(
        self,
        control_rx: Receiver<ControlMsg>,
        pdata_rx: Receiver<PData>,
    ) -> Result<(), Error<PData>> {
        match self {
            ExporterWrapper::Local {
                effect_handler,
                exporter,
            } => {
                let message_channel = message::MessageChannel::new(control_rx, pdata_rx);
                exporter.start(message_channel, effect_handler).await
            }
            ExporterWrapper::Shared {
                effect_handler,
                exporter,
            } => {
                if let (Receiver::Shared(control_rx), Receiver::Shared(pdata_rx)) =
                    (control_rx, pdata_rx)
                {
                    let message_channel = shared::MessageChannel::new(control_rx, pdata_rx);
                    exporter.start(message_channel, effect_handler).await
                } else {
                    Err(Error::ExporterError {
                        exporter: effect_handler.exporter_name(),
                        error: "Shared ExporterWrapper requires shared channels".to_owned(),
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::control::ControlMsg;
    use crate::exporter::{Error, ExporterWrapper};
    use crate::local::exporter as local;
    use crate::message;
    use crate::message::Message;
    use crate::shared::exporter as shared;
    use crate::testing::exporter::TestContext;
    use crate::testing::exporter::TestRuntime;
    use crate::testing::{CtrlMsgCounters, TestMsg};
    use async_trait::async_trait;
    use otap_df_channel::error::RecvError;
    use otap_df_channel::mpsc;
    use serde_json::Value;
    use std::future::Future;
    use std::time::Duration;
    use tokio::time::sleep;

    /// A test exporter that counts message events.
    /// Works with any type of exporter !Send or Send.
    pub struct TestExporter {
        /// Counter for different message types
        pub counter: CtrlMsgCounters,
    }

    impl TestExporter {
        /// Creates a new test node with the given counter
        pub fn new(counter: CtrlMsgCounters) -> Self {
            TestExporter { counter }
        }
    }

    #[async_trait(?Send)]
    impl local::Exporter<TestMsg> for TestExporter {
        async fn start(
            self: Box<Self>,
            mut msg_chan: message::MessageChannel<TestMsg>,
            effect_handler: local::EffectHandler<TestMsg>,
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
                            exporter: effect_handler.exporter_name(),
                            error: "Unknown control message".to_owned(),
                        });
                    }
                }
            }
            Ok(())
        }
    }

    #[async_trait]
    impl shared::Exporter<TestMsg> for TestExporter {
        async fn start(
            self: Box<Self>,
            mut msg_chan: shared::MessageChannel<TestMsg>,
            effect_handler: shared::EffectHandler<TestMsg>,
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
                            exporter: effect_handler.exporter_name(),
                            error: "Unknown control message".to_owned(),
                        });
                    }
                }
            }
            Ok(())
        }
    }

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
    fn test_exporter_local() {
        let test_runtime = TestRuntime::new();
        let exporter = ExporterWrapper::local(
            TestExporter::new(test_runtime.counters()),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure());
    }

    #[test]
    fn test_exporter_shared() {
        let test_runtime = TestRuntime::new();
        let exporter = ExporterWrapper::shared(
            TestExporter::new(test_runtime.counters()),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure());
    }

    fn make_chan() -> (
        mpsc::Sender<ControlMsg>,
        mpsc::Sender<String>,
        message::MessageChannel<String>,
    ) {
        let (control_tx, control_rx) = mpsc::Channel::<ControlMsg>::new(10);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(10);
        (
            control_tx,
            pdata_tx,
            message::MessageChannel::new(
                message::Receiver::Local(control_rx),
                message::Receiver::Local(pdata_rx),
            ),
        )
    }

    #[tokio::test]
    async fn test_control_priority() {
        let (control_tx, pdata_tx, mut channel) = make_chan();

        pdata_tx.send_async("pdata1".to_owned()).await.unwrap();
        control_tx
            .send_async(ControlMsg::Ack { id: 1 })
            .await
            .unwrap();

        // Control message should be received first due to bias
        let msg = channel.recv().await.unwrap();
        assert!(matches!(msg, Message::Control(ControlMsg::Ack { id: 1 })));

        // Then pdata message
        let msg = channel.recv().await.unwrap();
        assert!(matches!(msg, Message::PData(ref s) if s == "pdata1"));
    }

    #[tokio::test]
    async fn test_shutdown_drain() {
        let (control_tx, pdata_tx, mut channel) = make_chan();

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

        // Send more pdata after shutdown is sent, but before receiver likely gets it
        pdata_tx.send_async("pdata3".to_string()).await.unwrap();
        pdata_tx
            .send_async("pdata4_during_drain".to_string())
            .await
            .unwrap();

        // --- Start Receiving ---

        // 1. Should receive pdata1 (drain)
        let msg1 = channel.recv().await.unwrap();
        assert!(matches!(msg1, Message::PData(ref s) if s == "pdata1"));

        // 2. Should receive pdata2 (drain)
        let msg2 = channel.recv().await.unwrap();
        assert!(matches!(msg2, Message::PData(ref s) if s == "pdata2"));

        // 3. Should receive pdata3 (drain)
        let msg3 = channel.recv().await.unwrap();
        assert!(matches!(msg3, Message::PData(ref s) if s == "pdata3"));

        // 4. Should receive pdata4 (drain)
        let msg4 = channel.recv().await.unwrap();
        assert!(matches!(msg4, Message::PData(ref s) if s == "pdata4_during_drain"));

        // Wait for deadline to likely expire
        sleep(Duration::from_millis(120)).await; // Wait longer than deadline

        // Send pdata *after* deadline
        // This might get buffered but shouldn't be received before the shutdown msg
        let _ = pdata_tx
            .send_async("pdata5_after_deadline".to_string())
            .await;

        // 5. Now, should receive the Shutdown message itself
        let msg5 = channel.recv().await.unwrap();
        assert!(matches!(
            msg5,
            Message::Control(ControlMsg::Shutdown { .. })
        ));

        drop(control_tx);
        drop(pdata_tx); // Close channels

        // 6. Check for RecvError after channels closed
        let msg_err = channel.recv().await;
        assert!(matches!(msg_err, Err(RecvError::Closed)));
    }

    #[tokio::test]
    async fn test_shutdown_drain_pdata_closes() {
        let (control_tx, pdata_tx, mut channel) = make_chan();

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

        sleep(Duration::from_millis(10)).await; // Give receiver a chance

        // --- Start Receiving ---

        // 1. Should receive pdata1 (drain)
        let msg1 = channel.recv().await.unwrap();
        assert!(matches!(msg1, Message::PData(ref s) if s == "pdata1"));

        // Close the pdata channel during drain
        drop(pdata_tx);

        // 2. Now, should receive the Shutdown message because pdata channel closed
        let msg2 = channel.recv().await.unwrap();
        assert!(matches!(
            msg2,
            Message::Control(ControlMsg::Shutdown { .. })
        ));

        drop(control_tx);

        // 3. Check for RecvError after channels closed
        let msg_err = channel.recv().await;
        assert!(matches!(msg_err, Err(RecvError::Closed)));
    }

    #[tokio::test]
    async fn test_immediate_shutdown() {
        let (control_tx, pdata_tx, mut channel) = make_chan();

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
        assert!(matches!(
            msg1,
            Message::Control(ControlMsg::Shutdown { .. })
        ));

        // Pdata should be ignored and the recv method should return Closed
        let msg2 = channel.recv().await;
        assert!(matches!(msg2, Err(RecvError::Closed)));
    }

    /// After Shutdown all later control messages are silently dropped (ignored).
    #[tokio::test]
    async fn test_ignore_ctrl_after_shutdown() {
        let (control_tx, pdata_tx, mut chan) = make_chan();

        control_tx
            .send_async(ControlMsg::Shutdown {
                deadline: Duration::from_secs(0),
                reason: "ignore_followups".into(),
            })
            .await
            .unwrap();

        let msg = chan.recv().await.unwrap();
        assert!(matches!(msg, Message::Control(ControlMsg::Shutdown { .. })));

        // Send a control message that should fail as the channel has been closed
        // following the shutdown.
        assert!(
            control_tx
                .send_async(ControlMsg::Ack { id: 99 })
                .await
                .is_err()
        );

        // Send a pdata message that should fail as the channel has been closed
        // following the shutdown.
        assert!(pdata_tx.send_async("pdata1".to_owned()).await.is_err());

        // Another recv should report Closed, proving Ack was discarded.
        assert!(matches!(chan.recv().await, Err(RecvError::Closed)));
    }

    /// Immediate shutdown (deadline == 0) returns Shutdown and then behaves Closed.
    #[tokio::test]
    async fn test_immediate_shutdown_closed_afterwards() {
        let (control_tx, _pdata_tx, mut chan) = make_chan();

        control_tx
            .send_async(ControlMsg::Shutdown {
                deadline: Duration::from_secs(0),
                reason: "now".into(),
            })
            .await
            .unwrap();

        // First recv -> Shutdown
        let first = chan.recv().await.unwrap();
        assert!(matches!(
            first,
            Message::Control(ControlMsg::Shutdown { .. })
        ));

        // Second recv -> channel considered closed
        assert!(matches!(chan.recv().await, Err(RecvError::Closed)));
    }
}
