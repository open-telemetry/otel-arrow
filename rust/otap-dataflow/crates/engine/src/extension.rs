// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension wrapper providing a unified interface over local (`!Send`) and
//! shared (`Send`) extension implementations.
//!
//! Extensions are PData-free — they never process pipeline data, only control
//! messages. This module wraps local and shared variants into a single
//! `ExtensionWrapper` that the engine can start and manage.
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
use crate::control::ExtensionControlMsg;
use crate::entity_context::NodeTelemetryGuard;
use crate::local::extension as local;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::node::NodeId;
use crate::shared::extension as shared;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::terminal_state::TerminalState;
use otap_df_channel::mpsc;
use otap_df_config::node::NodeUserConfig;
use otap_df_telemetry::reporter::MetricsReporter;
use std::sync::Arc;

/// A wrapper for the extension that allows for both `Send` and `!Send` implementations.
///
/// Extensions are NOT generic over PData — they operate exclusively on
/// [`ExtensionControlMsg`], keeping the extension system entirely decoupled
/// from the data-plane type.
pub enum ExtensionWrapper {
    /// An extension with a `!Send` implementation.
    Local {
        /// Index identifier for the node.
        node_id: NodeId,
        /// The user configuration for the node.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the extension.
        runtime_config: ExtensionConfig,
        /// The extension instance.
        extension: Box<dyn local::Extension>,
        /// A sender for control messages.
        control_sender: LocalSender<ExtensionControlMsg>,
        /// A receiver for control messages.
        control_receiver: Option<LocalReceiver<ExtensionControlMsg>>,
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
        extension: Box<dyn shared::Extension>,
        /// A sender for control messages.
        control_sender: SharedSender<ExtensionControlMsg>,
        /// A receiver for control messages.
        control_receiver: Option<SharedReceiver<ExtensionControlMsg>>,
        /// Telemetry guard for node lifecycle cleanup.
        telemetry: Option<NodeTelemetryGuard>,
    },
}

impl ExtensionWrapper {
    /// Creates a new local `ExtensionWrapper` with the given extension and configuration (!Send
    /// implementation).
    pub fn local<E>(
        extension: E,
        node_id: NodeId,
        user_config: Arc<NodeUserConfig>,
        config: &ExtensionConfig,
    ) -> Self
    where
        E: local::Extension + 'static,
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
        E: shared::Extension + 'static,
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

    /// Returns whether this extension uses a shared (Send) implementation.
    #[must_use]
    pub fn is_shared(&self) -> bool {
        match self {
            ExtensionWrapper::Local { .. } => false,
            ExtensionWrapper::Shared { .. } => true,
        }
    }

    /// Returns the node ID of this extension.
    #[must_use]
    pub fn node_id(&self) -> NodeId {
        match self {
            ExtensionWrapper::Local { node_id, .. } => node_id.clone(),
            ExtensionWrapper::Shared { node_id, .. } => node_id.clone(),
        }
    }

    /// Returns the user configuration for this extension.
    #[must_use]
    pub fn user_config(&self) -> Arc<NodeUserConfig> {
        match self {
            ExtensionWrapper::Local { user_config, .. } => user_config.clone(),
            ExtensionWrapper::Shared { user_config, .. } => user_config.clone(),
        }
    }

    /// Collects the extension's trait registrations and inserts them into
    /// the registry under the given name.
    ///
    /// Called by the engine during pipeline build.
    pub fn register_traits(&self, registry: &mut registry::CapabilityRegistry, name: &str) {
        let registrations = match self {
            ExtensionWrapper::Local { extension, .. } => extension.extension_capabilities(),
            ExtensionWrapper::Shared { extension, .. } => extension.extension_capabilities(),
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
                    wrap_control_channel_metrics::<LocalMode, ExtensionControlMsg>(
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
                    wrap_control_channel_metrics::<SharedMode, ExtensionControlMsg>(
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

    /// Returns an `ExtensionControlSender` for sending control messages to this extension.
    pub(crate) fn extension_control_sender(&self) -> crate::control::ExtensionControlSender {
        match self {
            ExtensionWrapper::Local {
                node_id,
                control_sender,
                ..
            } => crate::control::ExtensionControlSender {
                node_id: node_id.clone(),
                sender: crate::message::Sender::Local(control_sender.clone()),
            },
            ExtensionWrapper::Shared {
                node_id,
                control_sender,
                ..
            } => crate::control::ExtensionControlSender {
                node_id: node_id.clone(),
                sender: crate::message::Sender::Shared(control_sender.clone()),
            },
        }
    }

    /// Starts the extension and begins its operation.
    ///
    /// Extensions do NOT receive a `PipelineCtrlMsgSender` — they are fully
    /// PData-free and manage their own timers directly via `tokio::time`.
    pub async fn start(
        self,
        metrics_reporter: MetricsReporter,
    ) -> Result<TerminalState, crate::error::Error> {
        match self {
            ExtensionWrapper::Local {
                node_id,
                extension,
                control_receiver,
                ..
            } => {
                let effect_handler = local::EffectHandler::new(node_id, metrics_reporter);

                let control_receiver =
                    control_receiver.expect("control_receiver missing from ExtensionWrapper");

                let ctrl_chan = local::ControlChannel::new(control_receiver);
                extension.start(ctrl_chan, effect_handler).await
            }
            ExtensionWrapper::Shared {
                node_id,
                extension,
                control_receiver,
                ..
            } => {
                let effect_handler = shared::EffectHandler::new(node_id, metrics_reporter);

                let control_receiver =
                    control_receiver.expect("control_receiver missing from ExtensionWrapper");

                let ctrl_chan = shared::ControlChannel::new(control_receiver);
                extension.start(ctrl_chan, effect_handler).await
            }
        }
    }
}

// ── TelemetryWrapped impl ───────────────────────────────────────────────────

impl crate::TelemetryWrapped for ExtensionWrapper {
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
    use crate::control::ExtensionControlMsg;
    use crate::testing::{CtrlMsgCounters, test_node};
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
    impl local::Extension for TestExtension {
        async fn start(
            self: Box<Self>,
            mut ctrl_chan: local::ControlChannel,
            _effect_handler: local::EffectHandler,
        ) -> Result<TerminalState, crate::error::Error> {
            loop {
                match ctrl_chan.recv().await? {
                    ExtensionControlMsg::TimerTick { .. } => {
                        self.counter.increment_timer_tick();
                    }
                    ExtensionControlMsg::Config { .. } => {
                        self.counter.increment_config();
                    }
                    ExtensionControlMsg::Shutdown { .. } => {
                        self.counter.increment_shutdown();
                        break;
                    }
                    ExtensionControlMsg::CollectTelemetry { .. } => {}
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
    impl shared::Extension for SharedTestExtension {
        async fn start(
            self: Box<Self>,
            mut ctrl_chan: shared::ControlChannel,
            _effect_handler: shared::EffectHandler,
        ) -> Result<TerminalState, crate::error::Error> {
            loop {
                if let ExtensionControlMsg::Shutdown { .. } = ctrl_chan.recv().await? {
                    self.counter.increment_shutdown();
                    break;
                }
            }
            Ok(TerminalState::default())
        }
    }
}
