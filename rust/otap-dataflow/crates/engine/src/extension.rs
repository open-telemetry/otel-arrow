// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension wrapper providing a unified interface over local (`!Send`) and
//! shared (`Send`) extension implementations.
//!
//! For the extension lifecycle traits, see [`local::extension`](crate::local::extension)
//! and [`shared::extension`](crate::shared::extension).
//!
//! For the registry and sealed trait infrastructure, see
//! [`registry`](registry).
//!
//! For built-in extension traits, see
//! [`bearer_token_provider`](bearer_token_provider).

pub mod registry;

/// Extension traits that components can implement to expose capabilities.
pub mod bearer_token_provider;

use crate::channel_metrics::ChannelMetricsRegistry;
use crate::channel_mode::{LocalMode, SharedMode, wrap_control_channel_metrics};
use crate::config::ExtensionConfig;
use crate::context::PipelineContext;
use crate::control::{Controllable, NodeControlMsg, PipelineCtrlMsgSender};
use crate::entity_context::NodeTelemetryGuard;
use crate::local::extension as local;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message;
use crate::message::{Receiver, Sender};
use crate::node::{Node, NodeId};
use crate::shared::extension as shared;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::terminal_state::TerminalState;
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use otap_df_config::node::NodeUserConfig;
use otap_df_telemetry::reporter::MetricsReporter;
use std::sync::Arc;

/// A wrapper for the extension that allows for both `Send` and `!Send` implementations.
///
/// Note: This is useful for creating a single interface for the extension regardless of their
/// 'sendability'.
pub enum ExtensionWrapper<PData> {
    /// An extension with a `!Send` implementation.
    Local {
        /// Index identifier for the node.
        node_id: NodeId,
        /// The user configuration for the node.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the extension.
        runtime_config: ExtensionConfig,
        /// The extension instance.
        extension: Box<dyn local::Extension<PData>>,
        /// A sender for control messages.
        control_sender: LocalSender<NodeControlMsg<PData>>,
        /// A receiver for control messages.
        control_receiver: Option<LocalReceiver<NodeControlMsg<PData>>>,
        /// Telemetry guard for node lifecycle cleanup.
        telemetry: Option<NodeTelemetryGuard>,
    },
    /// An extension with a `Send` implementation.
    Shared {
        /// Index identifier for the node.
        node_id: NodeId,
        /// The user configuration for the node.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the extension.
        runtime_config: ExtensionConfig,
        /// The extension instance.
        extension: Box<dyn shared::Extension<PData>>,
        /// A sender for control messages.
        control_sender: SharedSender<NodeControlMsg<PData>>,
        /// A receiver for control messages.
        control_receiver: Option<SharedReceiver<NodeControlMsg<PData>>>,
        /// Telemetry guard for node lifecycle cleanup.
        telemetry: Option<NodeTelemetryGuard>,
    },
}

#[async_trait::async_trait(?Send)]
impl<PData> Controllable<PData> for ExtensionWrapper<PData> {
    fn control_sender(&self) -> Sender<NodeControlMsg<PData>> {
        match self {
            ExtensionWrapper::Local { control_sender, .. } => Sender::Local(control_sender.clone()),
            ExtensionWrapper::Shared { control_sender, .. } => {
                Sender::Shared(control_sender.clone())
            }
        }
    }
}

impl<PData> ExtensionWrapper<PData> {
    /// Creates a new local `ExtensionWrapper` with the given extension and configuration (!Send
    /// implementation).
    pub fn local<E>(
        extension: E,
        node_id: NodeId,
        user_config: Arc<NodeUserConfig>,
        config: &ExtensionConfig,
    ) -> Self
    where
        E: local::Extension<PData> + 'static,
    {
        let (control_sender, control_receiver) =
            mpsc::Channel::new(config.control_channel.capacity);

        ExtensionWrapper::Local {
            node_id,
            user_config,
            runtime_config: config.clone(),
            extension: Box::new(extension),
            control_sender: LocalSender::mpsc(control_sender),
            control_receiver: Some(LocalReceiver::mpsc(control_receiver)),
            telemetry: None,
        }
    }

    /// Creates a new shared `ExtensionWrapper` with the given extension and configuration (Send
    /// implementation).
    pub fn shared<E>(
        extension: E,
        node_id: NodeId,
        user_config: Arc<NodeUserConfig>,
        config: &ExtensionConfig,
    ) -> Self
    where
        E: shared::Extension<PData> + 'static,
    {
        let (control_sender, control_receiver) =
            tokio::sync::mpsc::channel(config.control_channel.capacity);

        ExtensionWrapper::Shared {
            node_id,
            user_config,
            runtime_config: config.clone(),
            extension: Box::new(extension),
            control_sender: SharedSender::mpsc(control_sender),
            control_receiver: Some(SharedReceiver::mpsc(control_receiver)),
            telemetry: None,
        }
    }

    /// Collects the extension's trait registrations and inserts them into
    /// the registry under the given name.
    ///
    /// Called by the engine during pipeline build.
    pub fn register_traits(&self, registry: &mut registry::ExtensionRegistry, name: &str) {
        let registrations = match self {
            ExtensionWrapper::Local { extension, .. } => extension.extension_traits(),
            ExtensionWrapper::Shared { extension, .. } => extension.extension_traits(),
        };
        registry.register_all(name, registrations);
    }

    pub(crate) fn with_node_telemetry_guard(self, guard: NodeTelemetryGuard) -> Self {
        match self {
            ExtensionWrapper::Local {
                node_id,
                user_config,
                runtime_config,
                extension,
                control_sender,
                control_receiver,
                ..
            } => ExtensionWrapper::Local {
                node_id,
                user_config,
                runtime_config,
                extension,
                control_sender,
                control_receiver,
                telemetry: Some(guard),
            },
            ExtensionWrapper::Shared {
                node_id,
                user_config,
                runtime_config,
                extension,
                control_sender,
                control_receiver,
                ..
            } => ExtensionWrapper::Shared {
                node_id,
                user_config,
                runtime_config,
                extension,
                control_sender,
                control_receiver,
                telemetry: Some(guard),
            },
        }
    }

    pub(crate) const fn take_telemetry_guard(&mut self) -> Option<NodeTelemetryGuard> {
        match self {
            ExtensionWrapper::Local { telemetry, .. } => telemetry.take(),
            ExtensionWrapper::Shared { telemetry, .. } => telemetry.take(),
        }
    }

    pub(crate) fn with_control_channel_metrics(
        self,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Self {
        match self {
            ExtensionWrapper::Local {
                node_id,
                runtime_config,
                control_sender,
                control_receiver,
                user_config,
                extension,
                telemetry,
                ..
            } => {
                let control_receiver = control_receiver.expect("control_receiver already taken");

                let (control_sender, control_receiver) =
                    wrap_control_channel_metrics::<LocalMode, PData>(
                        &node_id,
                        pipeline_ctx,
                        channel_metrics,
                        channel_metrics_enabled,
                        runtime_config.control_channel.capacity as u64,
                        control_sender,
                        control_receiver,
                    );

                ExtensionWrapper::Local {
                    node_id,
                    user_config,
                    runtime_config,
                    extension,
                    control_sender,
                    control_receiver: Some(control_receiver),
                    telemetry,
                }
            }
            ExtensionWrapper::Shared {
                node_id,
                runtime_config,
                control_sender,
                control_receiver,
                user_config,
                extension,
                telemetry,
                ..
            } => {
                let control_receiver = control_receiver.expect("control_receiver already taken");

                let (control_sender, control_receiver) =
                    wrap_control_channel_metrics::<SharedMode, PData>(
                        &node_id,
                        pipeline_ctx,
                        channel_metrics,
                        channel_metrics_enabled,
                        runtime_config.control_channel.capacity as u64,
                        control_sender,
                        control_receiver,
                    );

                ExtensionWrapper::Shared {
                    node_id,
                    user_config,
                    runtime_config,
                    extension,
                    control_sender,
                    control_receiver: Some(control_receiver),
                    telemetry,
                }
            }
        }
    }

    /// Starts the extension and begins its operation.
    pub async fn start(
        self,
        pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<PData>,
        metrics_reporter: MetricsReporter,
    ) -> Result<TerminalState, crate::error::Error> {
        match (self, metrics_reporter) {
            (
                ExtensionWrapper::Local {
                    node_id,
                    extension,
                    control_receiver,
                    ..
                },
                metrics_reporter,
            ) => {
                let mut effect_handler = local::EffectHandler::new(node_id, metrics_reporter);
                effect_handler
                    .core
                    .set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);

                let control_receiver =
                    control_receiver.expect("control_receiver missing from ExtensionWrapper");

                // Extensions only receive control messages, no pdata.
                // Create a dummy pdata receiver that will never produce data.
                let (_dummy_tx, dummy_rx) = mpsc::Channel::<PData>::new(1);
                let message_channel = message::MessageChannel::new(
                    Receiver::Local(control_receiver),
                    Receiver::Local(LocalReceiver::mpsc(dummy_rx)),
                );
                extension.start(message_channel, effect_handler).await
            }
            (
                ExtensionWrapper::Shared {
                    node_id,
                    extension,
                    control_receiver,
                    ..
                },
                metrics_reporter,
            ) => {
                let mut effect_handler = shared::EffectHandler::new(node_id, metrics_reporter);
                effect_handler
                    .core
                    .set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);

                let control_receiver =
                    control_receiver.expect("control_receiver missing from ExtensionWrapper");

                let message_channel = shared::MessageChannel::new(control_receiver);
                extension.start(message_channel, effect_handler).await
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<PData> Node<PData> for ExtensionWrapper<PData> {
    fn is_shared(&self) -> bool {
        match self {
            ExtensionWrapper::Local { .. } => false,
            ExtensionWrapper::Shared { .. } => true,
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            ExtensionWrapper::Local { node_id, .. } => node_id.clone(),
            ExtensionWrapper::Shared { node_id, .. } => node_id.clone(),
        }
    }

    fn user_config(&self) -> Arc<NodeUserConfig> {
        match self {
            ExtensionWrapper::Local {
                user_config: config,
                ..
            } => config.clone(),
            ExtensionWrapper::Shared {
                user_config: config,
                ..
            } => config.clone(),
        }
    }

    async fn send_control_msg(
        &self,
        msg: NodeControlMsg<PData>,
    ) -> Result<(), SendError<NodeControlMsg<PData>>> {
        match self {
            ExtensionWrapper::Local { control_sender, .. } => control_sender.send(msg).await,
            ExtensionWrapper::Shared { control_sender, .. } => control_sender.send(msg).await,
        }
    }
}

// ── TelemetryWrapped impl ───────────────────────────────────────────────────

impl<PData> crate::TelemetryWrapped for ExtensionWrapper<PData> {
    fn with_control_channel_metrics(
        self,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Self {
        ExtensionWrapper::with_control_channel_metrics(
            self,
            pipeline_ctx,
            channel_metrics,
            channel_metrics_enabled,
        )
    }

    fn with_node_telemetry_guard(self, guard: NodeTelemetryGuard) -> Self {
        ExtensionWrapper::with_node_telemetry_guard(self, guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::NodeControlMsg;
    use crate::message::Message;
    use crate::testing::{CtrlMsgCounters, TestMsg, test_node};
    use async_trait::async_trait;
    use otap_df_config::node::NodeUserConfig;
    use serde_json::Value;

    #[derive(Clone)]
    struct TestExtension {
        counter: CtrlMsgCounters,
    }

    impl TestExtension {
        fn new(counter: CtrlMsgCounters) -> Self {
            TestExtension { counter }
        }
    }

    #[async_trait(?Send)]
    impl local::Extension<TestMsg> for TestExtension {
        async fn start(
            self: Box<Self>,
            mut msg_chan: message::MessageChannel<TestMsg>,
            _effect_handler: local::EffectHandler<TestMsg>,
        ) -> Result<TerminalState, crate::error::Error> {
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
                    Message::Control(NodeControlMsg::CollectTelemetry { .. }) => {}
                    Message::Control(NodeControlMsg::Ack(_)) => {}
                    Message::Control(NodeControlMsg::Nack(_)) => {}
                    Message::Control(NodeControlMsg::DelayedData { .. }) => {}
                    Message::PData(_) => {}
                }
            }
            Ok(TerminalState::default())
        }
    }

    #[test]
    fn test_extension_wrapper_local_creation() {
        let counter = CtrlMsgCounters::new();
        let extension = TestExtension::new(counter);
        let node_id = test_node("test_extension");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("test_extension");

        let wrapper = ExtensionWrapper::local(extension, node_id, user_config, &config);

        assert!(!wrapper.is_shared());
    }

    #[test]
    fn test_extension_wrapper_shared_creation() {
        let counter = CtrlMsgCounters::new();
        let node_id = test_node("test_extension_shared");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("test_extension_shared");

        let shared_ext = SharedTestExtension::new(counter);
        let wrapper = ExtensionWrapper::shared(shared_ext, node_id, user_config, &config);

        assert!(wrapper.is_shared());
    }

    #[derive(Clone)]
    struct SharedTestExtension {
        counter: CtrlMsgCounters,
    }

    impl SharedTestExtension {
        fn new(counter: CtrlMsgCounters) -> Self {
            SharedTestExtension { counter }
        }
    }

    #[async_trait]
    impl shared::Extension<TestMsg> for SharedTestExtension {
        async fn start(
            self: Box<Self>,
            mut msg_chan: shared::MessageChannel<TestMsg>,
            _effect_handler: shared::EffectHandler<TestMsg>,
        ) -> Result<TerminalState, crate::error::Error> {
            loop {
                if let Message::Control(NodeControlMsg::Shutdown { .. }) = msg_chan.recv().await? {
                    self.counter.increment_shutdown();
                    break;
                }
            }
            Ok(TerminalState::default())
        }
    }
}
