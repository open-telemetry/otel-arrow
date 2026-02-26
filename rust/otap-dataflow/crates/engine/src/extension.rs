// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension wrapper used to provide a unified interface to the pipeline engine that abstracts over
//! the fact that extension implementations may be `!Send` or `Send`.
//!
//! Extensions are **not** generic over `PData` — they sit outside the data-flow graph
//! and never process pipeline data.  They use the PData-free [`ExtensionControlMsg`]
//! for their control channels.
//!
//! For more details on the `!Send` implementation of an extension, see [`local::extension::Extension`].
//! See [`shared::extension::Extension`] for the Send implementation.

use crate::channel_metrics::ChannelMetricsRegistry;
use crate::channel_mode::{LocalMode, SharedMode, wrap_control_channel_metrics};
use crate::config::ExtensionConfig;
use crate::context::PipelineContext;
use crate::control::ExtensionControlMsg;
use crate::entity_context::NodeTelemetryGuard;
use crate::error::Error;
use crate::extensions::ExtensionHandles;
use crate::local::extension as local;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::node::NodeId;
use crate::shared::extension as shared;
use crate::shared::message::{SharedReceiver, SharedSender};
use otap_df_channel::mpsc;
use otap_df_config::node::NodeUserConfig;
use otap_df_telemetry::reporter::MetricsReporter;
use std::sync::Arc;

/// A wrapper for extensions that allows for both `Send` and `!Send` effect handlers.
///
/// Unlike the pipeline node wrappers, `ExtensionWrapper` is **not** generic over `PData`.
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
        control_receiver: LocalReceiver<ExtensionControlMsg>,
        /// Service handles produced by this extension, to be merged into the registry.
        handles: Option<ExtensionHandles>,
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
        control_receiver: SharedReceiver<ExtensionControlMsg>,
        /// Service handles produced by this extension, to be merged into the registry.
        handles: Option<ExtensionHandles>,
        /// Telemetry guard for node lifecycle cleanup.
        telemetry: Option<NodeTelemetryGuard>,
    },
}

impl ExtensionWrapper {
    /// Creates a new local `ExtensionWrapper` with the given extension and configuration.
    pub fn local<E>(
        extension: E,
        handles: ExtensionHandles,
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
            control_receiver: LocalReceiver::mpsc(control_receiver),
            handles: Some(handles),
            telemetry: None,
        }
    }

    /// Creates a new shared `ExtensionWrapper` with the given extension and configuration.
    pub fn shared<E>(
        extension: E,
        handles: ExtensionHandles,
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
            control_receiver: SharedReceiver::mpsc(control_receiver),
            handles: Some(handles),
            telemetry: None,
        }
    }

    /// Takes the service handles out of this wrapper.
    ///
    /// Called during pipeline build to transfer handles into the
    /// `ExtensionRegistryBuilder` (crate-internal).
    /// Returns `None` if handles have already been taken.
    pub fn take_handles(&mut self) -> Option<ExtensionHandles> {
        match self {
            ExtensionWrapper::Local { handles, .. } => handles.take(),
            ExtensionWrapper::Shared { handles, .. } => handles.take(),
        }
    }

    /// Returns the node id of the extension.
    pub fn node_id(&self) -> NodeId {
        match self {
            ExtensionWrapper::Local { node_id, .. } => node_id.clone(),
            ExtensionWrapper::Shared { node_id, .. } => node_id.clone(),
        }
    }

    /// Returns the extension name (from the user config URN).
    pub fn name(&self) -> &str {
        match self {
            ExtensionWrapper::Local { user_config, .. } => user_config.r#type.as_ref(),
            ExtensionWrapper::Shared { user_config, .. } => user_config.r#type.as_ref(),
        }
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
                handles,
                ..
            } => ExtensionWrapper::Local {
                node_id,
                user_config,
                runtime_config,
                extension,
                control_sender,
                control_receiver,
                handles,
                telemetry: Some(guard),
            },
            ExtensionWrapper::Shared {
                node_id,
                user_config,
                runtime_config,
                extension,
                control_sender,
                control_receiver,
                handles,
                ..
            } => ExtensionWrapper::Shared {
                node_id,
                user_config,
                runtime_config,
                extension,
                control_sender,
                control_receiver,
                handles,
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
                handles,
                telemetry,
            } => {
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
                    control_receiver,
                    handles,
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
                handles,
                telemetry,
            } => {
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
                    control_receiver,
                    handles,
                    telemetry,
                }
            }
        }
    }

    /// Starts the extension's background work.
    ///
    /// The extension task runs independently, processing only control messages.
    /// Unlike pipeline node wrappers, extensions do not receive a `PipelineCtrlMsgSender`
    /// — they manage their own timers via `tokio::time` if needed.
    pub async fn start(self, metrics_reporter: MetricsReporter) -> Result<(), Error> {
        match self {
            ExtensionWrapper::Local {
                node_id,
                extension,
                control_receiver,
                ..
            } => {
                let effect_handler = local::EffectHandler::new(node_id, metrics_reporter);
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
                let ctrl_chan = shared::ControlChannel::new(control_receiver);
                extension.start(ctrl_chan, effect_handler).await
            }
        }
    }
}
