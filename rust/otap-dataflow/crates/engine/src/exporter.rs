// SPDX-License-Identifier: Apache-2.0

//! Exporter wrapper used to provide a unified interface to the pipeline engine that abstracts over
//! the fact that exporter implementations may be `!Send` or `Send`.
//!
//! For more details on the `!Send` implementation of an exporter, see [`local::Exporter`].
//! See [`shared::Exporter`] for the Send implementation.

use crate::config::ExporterConfig;
use crate::control::{NodeControlMsg, Controllable, PipelineCtrlMsgSender};
use crate::error::Error;
use crate::local::exporter as local;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message;
use crate::message::{Receiver, Sender};
use crate::node::{Node, NodeWithPDataReceiver};
use crate::shared::exporter as shared;
use crate::shared::message::{SharedReceiver, SharedSender};
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use otap_df_config::NodeId;
use otap_df_config::node::NodeUserConfig;
use std::sync::Arc;

/// A wrapper for the exporter that allows for both `Send` and `!Send` effect handlers.
///
/// Note: This is useful for creating a single interface for the exporter regardless of their
/// 'sendability'.
pub enum ExporterWrapper<PData> {
    /// An exporter with a `!Send` implementation.
    Local {
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The exporter instance.
        exporter: Box<dyn local::Exporter<PData>>,
        /// The effect handler instance for the exporter.
        effect_handler: local::EffectHandler<PData>,
        /// A sender for control messages.
        control_sender: LocalSender<NodeControlMsg>,
        /// A receiver for control messages.
        control_receiver: Option<LocalReceiver<NodeControlMsg>>,
        /// Receiver for PData messages.
        pdata_receiver: Option<Receiver<PData>>,
    },
    /// An exporter with a `Send` implementation.
    Shared {
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The exporter instance.
        exporter: Box<dyn shared::Exporter<PData>>,
        /// The effect handler instance for the exporter.
        effect_handler: shared::EffectHandler<PData>,
        /// A sender for control messages.
        control_sender: SharedSender<NodeControlMsg>,
        /// A receiver for control messages.
        control_receiver: Option<SharedReceiver<NodeControlMsg>>,
        /// Receiver for PData messages.
        pdata_receiver: Option<SharedReceiver<PData>>,
    },
}

#[async_trait::async_trait(?Send)]
impl<PData> Controllable for ExporterWrapper<PData> {
    /// Sends a control message to the node.
    async fn send_control_msg(&self, msg: NodeControlMsg) -> Result<(), SendError<NodeControlMsg>> {
        self.control_sender().send(msg).await
    }

    /// Returns the control message sender for the exporter.
    fn control_sender(&self) -> Sender<NodeControlMsg> {
        match self {
            ExporterWrapper::Local { control_sender, .. } => Sender::Local(control_sender.clone()),
            ExporterWrapper::Shared { control_sender, .. } => {
                Sender::Shared(control_sender.clone())
            }
        }
    }
}

impl<PData> ExporterWrapper<PData> {
    /// Creates a new local `ExporterWrapper` with the given exporter and configuration (!Send
    /// implementation).
    pub fn local<E>(exporter: E, user_config: Arc<NodeUserConfig>, config: &ExporterConfig) -> Self
    where
        E: local::Exporter<PData> + 'static,
    {
        let (control_sender, control_receiver) =
            mpsc::Channel::new(config.control_channel.capacity);

        ExporterWrapper::Local {
            user_config,
            effect_handler: local::EffectHandler::new(config.name.clone()),
            exporter: Box::new(exporter),
            control_sender: LocalSender::MpscSender(control_sender),
            control_receiver: Some(LocalReceiver::MpscReceiver(control_receiver)),
            pdata_receiver: None, // This will be set later
        }
    }

    /// Creates a new shared `ExporterWrapper` with the given exporter and configuration (Send
    /// implementation).
    pub fn shared<E>(exporter: E, user_config: Arc<NodeUserConfig>, config: &ExporterConfig) -> Self
    where
        E: shared::Exporter<PData> + 'static,
    {
        let (control_sender, control_receiver) =
            tokio::sync::mpsc::channel(config.control_channel.capacity);

        ExporterWrapper::Shared {
            user_config,
            effect_handler: shared::EffectHandler::new(config.name.clone()),
            exporter: Box::new(exporter),
            control_sender: SharedSender::MpscSender(control_sender),
            control_receiver: Some(SharedReceiver::MpscReceiver(control_receiver)),
            pdata_receiver: None, // This will be set later
        }
    }

    /// Starts the exporter and begins exporting incoming data.
    pub async fn start(self, pipeline_ctrl_msg_tx: PipelineCtrlMsgSender) -> Result<(), Error<PData>> {
        match self {
            ExporterWrapper::Local {
                mut effect_handler,
                exporter,
                control_receiver,
                pdata_receiver,
                ..
            } => {
                let control_rx = control_receiver.ok_or_else(|| Error::ExporterError {
                    exporter: effect_handler.exporter_id(),
                    error: "Control receiver not initialized".to_owned(),
                })?;
                let pdata_rx = pdata_receiver.ok_or_else(|| Error::ExporterError {
                    exporter: effect_handler.exporter_id(),
                    error: "PData receiver not initialized".to_owned(),
                })?;
                effect_handler.core.set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);
                let message_channel =
                    message::MessageChannel::new(Receiver::Local(control_rx), pdata_rx);
                exporter.start(message_channel, effect_handler).await
            }
            ExporterWrapper::Shared {
                mut effect_handler,
                exporter,
                control_receiver,
                pdata_receiver,
                ..
            } => {
                let control_rx = control_receiver.ok_or_else(|| Error::ExporterError {
                    exporter: effect_handler.exporter_id(),
                    error: "Control receiver not initialized".to_owned(),
                })?;
                let pdata_rx = pdata_receiver.ok_or_else(|| Error::ExporterError {
                    exporter: effect_handler.exporter_id(),
                    error: "PData receiver not initialized".to_owned(),
                })?;
                effect_handler.core.set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);
                let message_channel = shared::MessageChannel::new(control_rx, pdata_rx);
                exporter.start(message_channel, effect_handler).await
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<PData> Node for ExporterWrapper<PData> {
    fn is_shared(&self) -> bool {
        match self {
            ExporterWrapper::Local { .. } => false,
            ExporterWrapper::Shared { .. } => true,
        }
    }

    fn user_config(&self) -> Arc<NodeUserConfig> {
        match self {
            ExporterWrapper::Local {
                user_config: config,
                ..
            } => config.clone(),
            ExporterWrapper::Shared {
                user_config: config,
                ..
            } => config.clone(),
        }
    }

    /// Sends a control message to the node.
    async fn send_control_msg(&self, msg: NodeControlMsg) -> Result<(), SendError<NodeControlMsg>> {
        match self {
            ExporterWrapper::Local { control_sender, .. } => control_sender.send(msg).await,
            ExporterWrapper::Shared { control_sender, .. } => control_sender.send(msg).await,
        }
    }
}

impl<PData> NodeWithPDataReceiver<PData> for ExporterWrapper<PData> {
    fn set_pdata_receiver(
        &mut self,
        node_id: NodeId,
        receiver: Receiver<PData>,
    ) -> Result<(), Error<PData>> {
        match (self, receiver) {
            (ExporterWrapper::Local { pdata_receiver, .. }, receiver) => {
                *pdata_receiver = Some(receiver);
                Ok(())
            }
            (ExporterWrapper::Shared { pdata_receiver, .. }, Receiver::Shared(receiver)) => {
                *pdata_receiver = Some(receiver);
                Ok(())
            }
            (ExporterWrapper::Shared { .. }, _) => Err(Error::ExporterError {
                exporter: node_id,
                error: "Expected a shared sender for PData".to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::control::NodeControlMsg;
    use crate::exporter::{Error, ExporterWrapper};
    use crate::local::exporter as local;
    use crate::local::message::LocalReceiver;
    use crate::message;
    use crate::message::Message;
    use crate::shared::exporter as shared;
    use crate::testing::exporter::TestContext;
    use crate::testing::exporter::TestRuntime;
    use crate::testing::{CtrlMsgCounters, TestMsg};
    use async_trait::async_trait;
    use otap_df_channel::error::RecvError;
    use otap_df_channel::mpsc;
    use otap_df_config::node::NodeUserConfig;
    use serde_json::Value;
    use std::future::Future;
    use std::sync::Arc;
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
                    Message::Control(NodeControlMsg::TimerTick { .. }) => {
                        self.counter.increment_timer_tick();
                    }
                    Message::Control(NodeControlMsg::Config { .. }) => {
                        self.counter.increment_config();
                    }
                    Message::Control(NodeControlMsg::Shutdown { .. }) => {
                        self.counter.increment_shutdown();
                        break;
                    }
                    Message::PData(_message) => {
                        self.counter.increment_message();
                    }
                    _ => {
                        return Err(Error::ExporterError {
                            exporter: effect_handler.exporter_id(),
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
                    Message::Control(NodeControlMsg::TimerTick { .. }) => {
                        self.counter.increment_timer_tick();
                    }
                    Message::Control(NodeControlMsg::Config { .. }) => {
                        self.counter.increment_config();
                    }
                    Message::Control(NodeControlMsg::Shutdown { .. }) => {
                        self.counter.increment_shutdown();
                        break;
                    }
                    Message::PData(_message) => {
                        self.counter.increment_message();
                    }
                    _ => {
                        return Err(Error::ExporterError {
                            exporter: effect_handler.exporter_id(),
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
    fn validation_procedure<T>() -> impl FnOnce(
        TestContext<TestMsg>,
        Result<(), Error<T>>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx, _| {
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
        let user_config = Arc::new(NodeUserConfig::new_exporter_config("test_exporter"));
        let exporter = ExporterWrapper::local(
            TestExporter::new(test_runtime.counters()),
            user_config,
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
        let user_config = Arc::new(NodeUserConfig::new_exporter_config("test_exporter"));
        let exporter = ExporterWrapper::shared(
            TestExporter::new(test_runtime.counters()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure());
    }

    fn make_chan() -> (
        mpsc::Sender<NodeControlMsg>,
        mpsc::Sender<String>,
        message::MessageChannel<String>,
    ) {
        let (control_tx, control_rx) = mpsc::Channel::<NodeControlMsg>::new(10);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(10);
        (
            control_tx,
            pdata_tx,
            message::MessageChannel::new(
                message::Receiver::Local(LocalReceiver::MpscReceiver(control_rx)),
                message::Receiver::Local(LocalReceiver::MpscReceiver(pdata_rx)),
            ),
        )
    }

    #[tokio::test]
    async fn test_control_priority() {
        let (control_tx, pdata_tx, mut channel) = make_chan();

        pdata_tx.send_async("pdata1".to_owned()).await.unwrap();
        control_tx
            .send_async(NodeControlMsg::Ack { id: 1 })
            .await
            .unwrap();

        // Control message should be received first due to bias
        let msg = channel.recv().await.unwrap();
        assert!(matches!(msg, Message::Control(NodeControlMsg::Ack { id: 1 })));

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
            .send_async(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(100), // 100ms deadline
                reason: "Test Shutdown".to_string(),
            })
            .await
            .unwrap();

        // Send more pdata after shutdown is sent, but before the receiver likely gets it
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
        // println!("msg5 = {:?}", msg5);
        assert!(matches!(
            msg5,
            Message::Control(NodeControlMsg::Shutdown { .. })
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
            .send_async(NodeControlMsg::Shutdown {
                deadline: Duration::from_secs(5), // Long deadline
                reason: "Test Shutdown PData Closes".to_string(),
            })
            .await
            .unwrap();

        sleep(Duration::from_millis(10)).await; // Give the receiver a chance

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
            Message::Control(NodeControlMsg::Shutdown { .. })
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
            .send_async(NodeControlMsg::Shutdown {
                deadline: Duration::from_secs(0), // Immediate deadline
                reason: "Immediate Shutdown".to_string(),
            })
            .await
            .unwrap();

        // Should immediately receive the shutdown message, no draining
        let msg1 = channel.recv().await.unwrap();
        assert!(matches!(
            msg1,
            Message::Control(NodeControlMsg::Shutdown { .. })
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
            .send_async(NodeControlMsg::Shutdown {
                deadline: Duration::from_secs(0),
                reason: "ignore_followups".into(),
            })
            .await
            .unwrap();

        let msg = chan.recv().await.unwrap();
        assert!(matches!(msg, Message::Control(NodeControlMsg::Shutdown { .. })));

        // Send a control message that should fail as the channel has been closed
        // following the shutdown.
        assert!(
            control_tx
                .send_async(NodeControlMsg::Ack { id: 99 })
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
            .send_async(NodeControlMsg::Shutdown {
                deadline: Duration::from_secs(0),
                reason: "now".into(),
            })
            .await
            .unwrap();

        // First recv -> Shutdown
        let first = chan.recv().await.unwrap();
        assert!(matches!(
            first,
            Message::Control(NodeControlMsg::Shutdown { .. })
        ));

        // Second recv -> channel considered closed
        assert!(matches!(chan.recv().await, Err(RecvError::Closed)));
    }
}
