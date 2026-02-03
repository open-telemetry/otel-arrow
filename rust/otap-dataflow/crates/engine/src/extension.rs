// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension wrapper used to provide a unified interface to the pipeline engine that abstracts over
//! the fact that extension implementations may be `!Send` or `Send`.
//!
//! For more details on the `!Send` implementation of an extension, see [`local::Extension`].
//! See [`shared::Extension`] for the Send implementation.

use crate::channel_metrics::ChannelMetricsRegistry;
use crate::channel_mode::{LocalMode, SharedMode, wrap_control_channel_metrics};
use crate::config::ExtensionConfig;
use crate::context::PipelineContext;
use crate::control::{Controllable, NodeControlMsg, PipelineCtrlMsgSender};
use crate::entity_context::NodeTelemetryGuard;
use crate::error::Error;
use crate::extensions::ExtensionRegistry;
use crate::extensions::registry::ExtensionBundle;
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

/// A wrapper for the extension that allows for both `Send` and `!Send` effect handlers.
///
/// Note: This is useful for creating a single interface for the extension regardless of their
/// 'sendability'.
pub enum ExtensionWrapper<PData> {
    /// An extension with a `!Send` implementation.
    Local {
        /// Index identifier for the node.
        node_id: NodeId,
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the extension.
        runtime_config: ExtensionConfig,
        /// The extension instance.
        extension: Box<dyn local::Extension<PData>>,
        /// Extension traits that this extension provides.
        /// Taken during pipeline initialization to build the central registry.
        extension_traits: Option<ExtensionBundle>,
        /// A sender for control messages.
        control_sender: LocalSender<NodeControlMsg<PData>>,
        /// A receiver for control messages.
        control_receiver: LocalReceiver<NodeControlMsg<PData>>,
        /// Telemetry guard for node lifecycle cleanup.
        telemetry: Option<NodeTelemetryGuard>,
        /// Extension registry for accessing extension traits.
        extension_registry: Option<ExtensionRegistry>,
    },
    /// An extension with a `Send` implementation.
    Shared {
        /// Index identifier for the node.
        node_id: NodeId,
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the extension.
        runtime_config: ExtensionConfig,
        /// The extension instance.
        extension: Box<dyn shared::Extension<PData>>,
        /// Extension traits that this extension provides.
        /// Taken during pipeline initialization to build the central registry.
        extension_traits: Option<ExtensionBundle>,
        /// A sender for control messages.
        control_sender: SharedSender<NodeControlMsg<PData>>,
        /// A receiver for control messages.
        control_receiver: SharedReceiver<NodeControlMsg<PData>>,
        /// Telemetry guard for node lifecycle cleanup.
        telemetry: Option<NodeTelemetryGuard>,
        /// Extension registry for accessing extension traits.
        extension_registry: Option<ExtensionRegistry>,
    },
}

#[async_trait::async_trait(?Send)]
impl<PData> Controllable<PData> for ExtensionWrapper<PData> {
    /// Returns the control message sender for the extension.
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
        extension_traits: ExtensionBundle,
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
            extension_traits: Some(extension_traits),
            control_sender: LocalSender::mpsc(control_sender),
            control_receiver: LocalReceiver::mpsc(control_receiver),
            telemetry: None,
            extension_registry: None,
        }
    }

    /// Creates a new shared `ExtensionWrapper` with the given extension and configuration (Send
    /// implementation).
    pub fn shared<E>(
        extension: E,
        extension_traits: ExtensionBundle,
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
            extension_traits: Some(extension_traits),
            control_sender: SharedSender::mpsc(control_sender),
            control_receiver: SharedReceiver::mpsc(control_receiver),
            telemetry: None,
            extension_registry: None,
        }
    }

    /// Sets the extension registry for this extension.
    pub fn set_extension_registry(&mut self, registry: ExtensionRegistry) {
        match self {
            ExtensionWrapper::Local {
                extension_registry, ..
            } => *extension_registry = Some(registry),
            ExtensionWrapper::Shared {
                extension_registry, ..
            } => *extension_registry = Some(registry),
        }
    }

    pub(crate) fn with_node_telemetry_guard(self, guard: NodeTelemetryGuard) -> Self {
        match self {
            ExtensionWrapper::Local {
                node_id,
                user_config,
                runtime_config,
                extension,
                extension_traits,
                control_sender,
                control_receiver,
                extension_registry,
                ..
            } => ExtensionWrapper::Local {
                node_id,
                user_config,
                runtime_config,
                extension,
                extension_traits,
                control_sender,
                control_receiver,
                telemetry: Some(guard),
                extension_registry,
            },
            ExtensionWrapper::Shared {
                node_id,
                user_config,
                runtime_config,
                extension,
                extension_traits,
                control_sender,
                control_receiver,
                extension_registry,
                ..
            } => ExtensionWrapper::Shared {
                node_id,
                user_config,
                runtime_config,
                extension,
                extension_traits,
                control_sender,
                control_receiver,
                telemetry: Some(guard),
                extension_registry,
            },
        }
    }

    pub(crate) fn take_telemetry_guard(&mut self) -> Option<NodeTelemetryGuard> {
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
                extension_traits,
                telemetry,
                extension_registry,
            } => {
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
                    extension_traits,
                    control_sender,
                    control_receiver,
                    telemetry,
                    extension_registry,
                }
            }
            ExtensionWrapper::Shared {
                node_id,
                runtime_config,
                control_sender,
                control_receiver,
                user_config,
                extension,
                extension_traits,
                telemetry,
                extension_registry,
            } => {
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
                    extension_traits,
                    control_sender,
                    control_receiver,
                    telemetry,
                    extension_registry,
                }
            }
        }
    }

    /// Starts the extension and begins its operation.
    pub async fn start(
        self,
        pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<PData>,
        metrics_reporter: MetricsReporter,
    ) -> Result<TerminalState, Error> {
        match (self, metrics_reporter) {
            (
                ExtensionWrapper::Local {
                    node_id,
                    extension,
                    control_receiver,
                    extension_registry,
                    ..
                },
                metrics_reporter,
            ) => {
                let mut effect_handler = local::EffectHandler::new(node_id, metrics_reporter);
                if let Some(registry) = extension_registry {
                    effect_handler.set_extension_registry(registry);
                }
                effect_handler
                    .core
                    .set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);
                // Extensions only receive control messages, no pdata
                // Create a dummy pdata receiver that will never receive anything
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
                    extension_registry,
                    ..
                },
                metrics_reporter,
            ) => {
                let mut effect_handler = shared::EffectHandler::new(node_id, metrics_reporter);
                if let Some(registry) = extension_registry {
                    effect_handler.set_extension_registry(registry);
                }
                effect_handler
                    .core
                    .set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);
                let message_channel = shared::MessageChannel::new(control_receiver);
                extension.start(message_channel, effect_handler).await
            }
        }
    }

    /// Takes the extension traits bundle from this wrapper, leaving `None` in its place.
    ///
    /// This is called during pipeline initialization to collect all extension traits
    /// into the central registry.
    pub fn take_extension_traits(&mut self) -> Option<ExtensionBundle> {
        match self {
            ExtensionWrapper::Local {
                extension_traits, ..
            } => extension_traits.take(),
            ExtensionWrapper::Shared {
                extension_traits, ..
            } => extension_traits.take(),
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

    /// Sends a control message to the node.
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

#[cfg(test)]
mod tests {
    use crate::config::ExtensionConfig;
    use crate::control::NodeControlMsg;
    use crate::extension::{Error, ExtensionWrapper};
    use crate::extensions::registry::ExtensionBundle;
    use crate::local::extension as local;
    use crate::message;
    use crate::message::Message;
    use crate::node::Node;
    use crate::shared::extension as shared;
    use crate::terminal_state::TerminalState;
    use crate::testing::{CtrlMsgCounters, TestMsg, test_node};
    use async_trait::async_trait;
    use otap_df_config::node::{NodeKind, NodeUserConfig};
    use serde_json::Value;
    use std::sync::Arc;

    /// A test extension that counts message events.
    pub struct TestExtension {
        /// Counter for different message types
        pub counter: CtrlMsgCounters,
    }

    impl TestExtension {
        /// Creates a new test extension with the given counter
        pub fn new(counter: CtrlMsgCounters) -> Self {
            TestExtension { counter }
        }
    }

    #[async_trait(?Send)]
    impl local::Extension<TestMsg> for TestExtension {
        async fn start(
            self: Box<Self>,
            mut msg_chan: message::MessageChannel<TestMsg>,
            _effect_handler: local::EffectHandler<TestMsg>,
        ) -> Result<TerminalState, Error> {
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
                    Message::Control(NodeControlMsg::CollectTelemetry { .. }) => {
                        // Ignore telemetry collection requests in tests
                    }
                    Message::Control(NodeControlMsg::Ack(_)) => {}
                    Message::Control(NodeControlMsg::Nack(_)) => {}
                    Message::Control(NodeControlMsg::DelayedData { .. }) => {}
                    Message::PData(_) => {
                        // Extensions don't process pdata
                    }
                }
            }
            Ok(TerminalState::default())
        }
    }

    #[test]
    fn test_local_extension_wrapper_creation() {
        let counter = CtrlMsgCounters::new();
        let extension = TestExtension::new(counter);
        let node_id = test_node("test_extension");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            NodeKind::Receiver, // Extension is not a config kind yet
            "urn:test:extension".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("test_extension");

        let wrapper = ExtensionWrapper::local(
            extension,
            ExtensionBundle::new(),
            node_id,
            user_config,
            &config,
        );

        assert!(!wrapper.is_shared());
    }

    /// A shared test extension
    pub struct SharedTestExtension {
        pub counter: CtrlMsgCounters,
    }

    impl SharedTestExtension {
        pub fn new(counter: CtrlMsgCounters) -> Self {
            SharedTestExtension { counter }
        }
    }

    #[async_trait]
    impl shared::Extension<TestMsg> for SharedTestExtension {
        async fn start(
            self: Box<Self>,
            mut msg_chan: shared::MessageChannel<TestMsg>,
            _effect_handler: shared::EffectHandler<TestMsg>,
        ) -> Result<TerminalState, Error> {
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
    fn test_shared_extension_wrapper_creation() {
        let counter = CtrlMsgCounters::new();
        let extension = SharedTestExtension::new(counter);
        let node_id = test_node("test_extension");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            NodeKind::Receiver, // Extension is not a config kind yet
            "urn:test:extension".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("test_extension");

        let wrapper = ExtensionWrapper::shared(
            extension,
            ExtensionBundle::new(),
            node_id,
            user_config,
            &config,
        );

        assert!(wrapper.is_shared());
    }
}
