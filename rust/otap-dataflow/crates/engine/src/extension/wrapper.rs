// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension wrapper, control channel, and bundle types.
//!
//! Extensions are PData-free — they never process pipeline data, only control
//! messages. This module defines [`EffectHandler`], [`ExtensionWrapper`], and
//! [`ExtensionBundle`]. The typestate builder that constructs bundles lives
//! in [`super::builder`].
//!
//! For the local (!Send) and shared (Send) Extension traits, see
//! [`local::extension`](crate::local::extension) and
//! [`shared::extension`](crate::shared::extension).

use crate::capability::factory::{LocalInstanceFactory, SharedInstanceFactory};
use crate::channel_metrics::ChannelMetricsRegistry;
use crate::channel_mode::{LocalMode, SharedMode, wrap_control_channel_metrics};
use crate::config::ExtensionConfig;
use crate::context::PipelineContext;
use crate::control::ExtensionControlMsg;
use crate::entity_context::NodeTelemetryGuard;
use crate::error::Error;
use crate::local::extension as local_ext;
use crate::local::message::LocalReceiver;
use crate::message::Sender;
use crate::shared::extension as shared_ext;
use crate::shared::message::SharedReceiver;
use crate::terminal_state::TerminalState;
use otap_df_channel::error::RecvError;
use otap_df_config::ExtensionId;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_telemetry::otel_debug;
use otap_df_telemetry::reporter::MetricsReporter;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{Sleep, sleep_until};

// ── ControlChannel ──────────────────────────────────────────────────────────

/// A shutdown-aware channel for receiving extension control messages.
///
/// Generic over the receiver type `R` to support both local (!Send) and
/// shared (Send) channels. Concrete types are defined in
/// [`local::extension::ControlChannel`](crate::local::extension::ControlChannel)
/// and [`shared::extension::ControlChannel`](crate::shared::extension::ControlChannel).
///
/// When a `Shutdown` message arrives with a future deadline, the channel
/// continues delivering other control messages until the deadline expires,
/// then returns the `Shutdown`.
#[doc(hidden)]
pub struct ControlChannel<R> {
    control_rx: Option<R>,
    shutting_down_deadline: Option<Instant>,
    pending_shutdown: Option<ExtensionControlMsg>,
}

/// Trait abstracting over local and shared receivers for control messages.
#[doc(hidden)]
pub trait ControlReceiver {
    /// Receives the next message, or returns an error if the channel is closed.
    fn recv(&mut self) -> impl Future<Output = Result<ExtensionControlMsg, RecvError>>;
}

impl ControlReceiver for LocalReceiver<ExtensionControlMsg> {
    async fn recv(&mut self) -> Result<ExtensionControlMsg, RecvError> {
        LocalReceiver::recv(self).await
    }
}

impl ControlReceiver for SharedReceiver<ExtensionControlMsg> {
    async fn recv(&mut self) -> Result<ExtensionControlMsg, RecvError> {
        SharedReceiver::recv(self).await
    }
}

impl<R: ControlReceiver> ControlChannel<R> {
    /// Creates a new `ControlChannel` with the given control receiver.
    #[must_use]
    pub fn new(control_rx: R) -> Self {
        ControlChannel {
            control_rx: Some(control_rx),
            shutting_down_deadline: None,
            pending_shutdown: None,
        }
    }

    /// Asynchronously receives the next control message.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if the channel is closed.
    pub async fn recv(&mut self) -> Result<ExtensionControlMsg, RecvError> {
        let mut sleep_until_deadline: Option<Pin<Box<Sleep>>> = None;

        loop {
            if self.control_rx.is_none() {
                return Err(RecvError::Closed);
            }

            if let Some(dl) = self.shutting_down_deadline {
                if Instant::now() >= dl {
                    let shutdown = self
                        .pending_shutdown
                        .take()
                        .expect("pending_shutdown must exist");
                    self.shutdown();
                    return Ok(shutdown);
                }

                if sleep_until_deadline.is_none() {
                    sleep_until_deadline = Some(Box::pin(sleep_until(dl.into())));
                }

                // Race the deadline against incoming control messages so that
                // Config / CollectTelemetry messages sent during the grace
                // period are still delivered to the extension.
                tokio::select! {
                    biased;
                    _ = sleep_until_deadline.as_mut().expect("set above") => {
                        let shutdown = self
                            .pending_shutdown
                            .take()
                            .expect("pending_shutdown must exist");
                        self.shutdown();
                        return Ok(shutdown);
                    }
                    msg = self.control_rx.as_mut().expect("checked above").recv() => {
                        match msg {
                            Ok(msg) => return Ok(msg),
                            Err(_) => {
                                // Channel closed during grace period — deliver
                                // the pending shutdown instead of propagating
                                // the closed error.
                                let shutdown = self
                                    .pending_shutdown
                                    .take()
                                    .expect("pending_shutdown must exist");
                                self.shutdown();
                                return Ok(shutdown);
                            }
                        }
                    }
                }
            }

            match self
                .control_rx
                .as_mut()
                .expect("checked above")
                .recv()
                .await
            {
                Ok(ExtensionControlMsg::Shutdown { deadline, reason }) => {
                    if deadline.duration_since(Instant::now()).is_zero() {
                        self.shutdown();
                        return Ok(ExtensionControlMsg::Shutdown { deadline, reason });
                    }
                    self.shutting_down_deadline = Some(deadline);
                    self.pending_shutdown =
                        Some(ExtensionControlMsg::Shutdown { deadline, reason });
                    continue;
                }
                Ok(msg) => return Ok(msg),
                Err(e) => return Err(e),
            }
        }
    }

    fn shutdown(&mut self) {
        self.shutting_down_deadline = None;
        let _ = self.control_rx.take().expect("control_rx must exist");
    }
}

// ── EffectHandler ───────────────────────────────────────────────────────────

/// The effect handler for extensions.
///
/// Provides extensions with identity and basic I/O. Extensions manage their
/// own timers directly via `tokio::time` rather than through the engine's
/// timer infrastructure, keeping the extension system fully PData-free.
#[derive(Clone)]
pub struct EffectHandler {
    name: ExtensionId,
    #[allow(dead_code)]
    metrics_reporter: MetricsReporter,
}

impl EffectHandler {
    /// Creates a new `EffectHandler` for the given extension.
    #[must_use]
    pub const fn new(name: ExtensionId, metrics_reporter: MetricsReporter) -> Self {
        EffectHandler {
            name,
            metrics_reporter,
        }
    }

    /// Returns the name of the extension associated with this handler.
    #[must_use]
    pub fn extension_id(&self) -> ExtensionId {
        self.name.clone()
    }
}

// ── Lifecycle ────────────────────────────────────────────────────────────────

/// The lifecycle state of an extension variant (local or shared).
///
/// Generic over the extension trait object type `E`, which is
/// `Rc<dyn local::Extension>` for local variants and
/// `Box<dyn shared::Extension>` for shared variants.
pub enum ExtensionLifecycle<E, R> {
    /// Active extension with an event loop and control channel.
    Active {
        /// The extension trait object with start method.
        extension: E,
        /// Control channel sender.
        control_sender: Sender<ExtensionControlMsg>,
        /// Control channel receiver.
        control_receiver: R,
    },
    /// Passive extension — capabilities only, no task spawned.
    Passive,
}

// ── ExtensionWrapper ────────────────────────────────────────────────────────

/// Wrapper for a single extension variant in the pipeline engine.
///
/// Extensions are NOT generic over PData — they operate exclusively on
/// [`ExtensionControlMsg`], keeping the extension system entirely decoupled
/// from the data-plane type.
///
/// The first layer is local vs shared (enum variant). The second layer is
/// active vs passive ([`ExtensionLifecycle`]). This two-level structure makes
/// impossible states unrepresentable while keeping local and shared
/// concerns cleanly separated.
pub enum ExtensionWrapper {
    /// A local (!Send) extension variant.
    Local {
        /// Extension identifier.
        name: ExtensionId,
        /// User-provided extension configuration.
        user_config: Arc<ExtensionUserConfig>,
        /// Engine runtime configuration for this extension.
        runtime_config: ExtensionConfig,
        /// Node telemetry guard, set during pipeline wiring.
        telemetry: Option<NodeTelemetryGuard>,
        /// The lifecycle state (active with channels, or passive).
        lifecycle: ExtensionLifecycle<
            std::rc::Rc<dyn local_ext::Extension>,
            LocalReceiver<ExtensionControlMsg>,
        >,
        /// Mints instances of the extension's concrete type for
        /// capability consumers. The engine's generated capability
        /// registration glue downcasts back to `E` and wraps in the
        /// requested trait object.
        instance_factory: LocalInstanceFactory,
    },
    /// A shared (Send) extension variant.
    Shared {
        /// Extension identifier.
        name: ExtensionId,
        /// User-provided extension configuration.
        user_config: Arc<ExtensionUserConfig>,
        /// Engine runtime configuration for this extension.
        runtime_config: ExtensionConfig,
        /// Node telemetry guard, set during pipeline wiring.
        telemetry: Option<NodeTelemetryGuard>,
        /// The lifecycle state (active with channels, or passive).
        lifecycle:
            ExtensionLifecycle<Box<dyn shared_ext::Extension>, SharedReceiver<ExtensionControlMsg>>,
        /// Mints instances of the extension's concrete type for
        /// capability consumers.
        instance_factory: SharedInstanceFactory,
    },
}

/// The set of extension wrappers produced by a single extension factory.
///
/// An extension can have at most one local variant and one shared variant.
/// The builder enforces that at least one is present and that dual
/// registrations use different concrete types.
pub struct ExtensionBundle {
    /// The local (!Send) extension variant, if any.
    local: Option<ExtensionWrapper>,
    /// The shared (Send) extension variant, if any.
    shared: Option<ExtensionWrapper>,
}

impl ExtensionWrapper {
    /// Returns the name of this extension.
    #[must_use]
    pub fn name(&self) -> ExtensionId {
        match self {
            ExtensionWrapper::Local { name, .. } | ExtensionWrapper::Shared { name, .. } => {
                name.clone()
            }
        }
    }

    /// Returns the user configuration for this extension.
    #[must_use]
    pub fn user_config(&self) -> Arc<ExtensionUserConfig> {
        match self {
            ExtensionWrapper::Local { user_config, .. }
            | ExtensionWrapper::Shared { user_config, .. } => user_config.clone(),
        }
    }

    /// Returns `true` if this extension is passive (no active lifecycle).
    #[must_use]
    pub fn is_passive(&self) -> bool {
        match self {
            ExtensionWrapper::Local { lifecycle, .. } => {
                matches!(lifecycle, ExtensionLifecycle::Passive)
            }
            ExtensionWrapper::Shared { lifecycle, .. } => {
                matches!(lifecycle, ExtensionLifecycle::Passive)
            }
        }
    }

    /// Returns the shared instance factory, if this is a shared variant.
    #[must_use]
    pub fn shared_instance_factory(&self) -> Option<&SharedInstanceFactory> {
        match self {
            ExtensionWrapper::Shared {
                instance_factory, ..
            } => Some(instance_factory),
            _ => None,
        }
    }

    /// Returns the local instance factory, if this is a local variant.
    #[must_use]
    pub fn local_instance_factory(&self) -> Option<&LocalInstanceFactory> {
        match self {
            ExtensionWrapper::Local {
                instance_factory, ..
            } => Some(instance_factory),
            _ => None,
        }
    }

    pub(crate) fn with_node_telemetry_guard(mut self, guard: NodeTelemetryGuard) -> Self {
        match &mut self {
            ExtensionWrapper::Local { telemetry, .. }
            | ExtensionWrapper::Shared { telemetry, .. } => *telemetry = Some(guard),
        }
        self
    }

    pub(crate) fn with_control_channel_metrics(
        self,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Self {
        match self {
            ExtensionWrapper::Local {
                name,
                user_config,
                runtime_config,
                telemetry,
                lifecycle,
                instance_factory,
            } => {
                let lifecycle = match lifecycle {
                    ExtensionLifecycle::Passive => ExtensionLifecycle::Passive,
                    ExtensionLifecycle::Active {
                        extension,
                        control_sender,
                        control_receiver,
                    } => {
                        let capacity = runtime_config.control_channel.capacity as u64;
                        let (local_sender, local_receiver) = match control_sender {
                            Sender::Local(s) => (s, control_receiver),
                            _ => unreachable!("Local variant always has local sender"),
                        };
                        let (s, r) = wrap_control_channel_metrics::<LocalMode, ExtensionControlMsg>(
                            name.as_ref(),
                            pipeline_ctx,
                            channel_metrics,
                            channel_metrics_enabled,
                            capacity,
                            local_sender,
                            local_receiver,
                        );
                        ExtensionLifecycle::Active {
                            extension,
                            control_sender: Sender::Local(s),
                            control_receiver: r,
                        }
                    }
                };
                ExtensionWrapper::Local {
                    name,
                    user_config,
                    runtime_config,
                    telemetry,
                    lifecycle,
                    instance_factory,
                }
            }
            ExtensionWrapper::Shared {
                name,
                user_config,
                runtime_config,
                telemetry,
                lifecycle,
                instance_factory,
            } => {
                let lifecycle = match lifecycle {
                    ExtensionLifecycle::Passive => ExtensionLifecycle::Passive,
                    ExtensionLifecycle::Active {
                        extension,
                        control_sender,
                        control_receiver,
                    } => {
                        let capacity = runtime_config.control_channel.capacity as u64;
                        let shared_sender = match control_sender {
                            Sender::Shared(s) => s,
                            _ => unreachable!("Shared variant always has shared sender"),
                        };
                        let (s, r) = wrap_control_channel_metrics::<SharedMode, ExtensionControlMsg>(
                            name.as_ref(),
                            pipeline_ctx,
                            channel_metrics,
                            channel_metrics_enabled,
                            capacity,
                            shared_sender,
                            control_receiver,
                        );
                        ExtensionLifecycle::Active {
                            extension,
                            control_sender: Sender::Shared(s),
                            control_receiver: r,
                        }
                    }
                };
                ExtensionWrapper::Shared {
                    name,
                    user_config,
                    runtime_config,
                    telemetry,
                    lifecycle,
                    instance_factory,
                }
            }
        }
    }

    /// Returns the `ExtensionControlSender` for sending control messages,
    /// or `None` if this extension is passive.
    #[allow(dead_code)]
    pub(crate) fn extension_control_sender(
        &self,
    ) -> Option<crate::control::ExtensionControlSender> {
        let (name, sender) = match self {
            ExtensionWrapper::Local {
                name,
                lifecycle: ExtensionLifecycle::Active { control_sender, .. },
                ..
            }
            | ExtensionWrapper::Shared {
                name,
                lifecycle: ExtensionLifecycle::Active { control_sender, .. },
                ..
            } => (name, control_sender),
            _ => return None,
        };
        Some(crate::control::ExtensionControlSender {
            name: name.clone(),
            sender: sender.clone(),
        })
    }

    /// Starts the extension lifecycle.
    ///
    /// # Errors
    ///
    /// Returns an error if this is a passive extension (no active lifecycle).
    pub async fn start(self, metrics_reporter: MetricsReporter) -> Result<TerminalState, Error> {
        match self {
            ExtensionWrapper::Local {
                name,
                lifecycle:
                    ExtensionLifecycle::Active {
                        extension,
                        control_receiver,
                        ..
                    },
                ..
            } => {
                otel_debug!("extension.start.local", name = name.as_ref());
                let effect_handler = EffectHandler::new(name, metrics_reporter);
                extension
                    .start(
                        local_ext::ControlChannel::new(control_receiver),
                        effect_handler,
                    )
                    .await
            }
            ExtensionWrapper::Shared {
                name,
                lifecycle:
                    ExtensionLifecycle::Active {
                        extension,
                        control_receiver,
                        ..
                    },
                ..
            } => {
                otel_debug!("extension.start.shared", name = name.as_ref());
                let effect_handler = EffectHandler::new(name, metrics_reporter);
                extension
                    .start(
                        shared_ext::ControlChannel::new(control_receiver),
                        effect_handler,
                    )
                    .await
            }
            _ => Err(Error::InternalError {
                message: "start() called on passive extension".into(),
            }),
        }
    }
}

impl ExtensionBundle {
    /// Assemble an [`ExtensionBundle`] from already-constructed wrappers.
    /// Called by [`super::builder::ExtensionBundleBuilder::build`].
    pub(super) fn from_parts(
        local: Option<ExtensionWrapper>,
        shared: Option<ExtensionWrapper>,
    ) -> Self {
        ExtensionBundle { local, shared }
    }

    /// Returns a reference to the local extension variant, if any.
    #[must_use]
    pub fn local(&self) -> Option<&ExtensionWrapper> {
        self.local.as_ref()
    }

    /// Returns a reference to the shared extension variant, if any.
    #[must_use]
    pub fn shared(&self) -> Option<&ExtensionWrapper> {
        self.shared.as_ref()
    }

    /// Takes ownership of the local extension variant, if any.
    pub fn take_local(&mut self) -> Option<ExtensionWrapper> {
        self.local.take()
    }

    /// Takes ownership of the shared extension variant, if any.
    pub fn take_shared(&mut self) -> Option<ExtensionWrapper> {
        self.shared.take()
    }

    /// Returns an iterator over the extension wrappers in this set.
    pub fn iter(&self) -> impl Iterator<Item = &ExtensionWrapper> {
        self.local.iter().chain(self.shared.iter())
    }

    /// Register this bundle's capabilities into the given registry.
    ///
    /// Calls the `register_shared` / `register_local` fn pointers on
    /// `capabilities` (produced by the `extension_capabilities!` macro)
    /// with a clone of the wrapper's corresponding instance factory.
    /// The fn pointers fan out one registry entry per listed capability.
    ///
    /// # Errors
    ///
    /// - [`Error::InternalError`](crate::capability::registry::Error::InternalError)
    ///   if `capabilities` advertises an execution model that the
    ///   runtime [`ExtensionBundle`] does not contain (e.g. the
    ///   `extension_capabilities!` macro lists local capabilities but
    ///   the factory's `create()` body never called `.local(...)` on
    ///   the builder). This catches a metadata-vs-bundle drift early
    ///   instead of letting it surface as a downstream
    ///   `Error::ConfigError("…does not provide capability…")` from
    ///   `resolve_bindings`. The shared-only-with-`SharedAsLocal`
    ///   fallback case stays valid: that macro arm advertises an
    ///   empty `local: &[]`, so there is no asymmetry to reject.
    /// - Any error returned by the capability registry
    ///   (e.g. duplicate `(capability, extension)` registration).
    pub fn register_into(
        &self,
        capabilities: &crate::capability::ExtensionCapabilities,
        registry: &mut crate::capability::registry::CapabilityRegistry,
    ) -> Result<(), crate::capability::registry::Error> {
        // Fail fast on metadata-vs-bundle drift: the
        // `extension_capabilities!` macro only checks that the listed
        // types implement the listed capability traits. It cannot see
        // what the factory's `create()` actually built. If the macro
        // advertises an execution model the runtime bundle is
        // missing, the mismatch would silently slip through
        // `register_into` and surface later as a confusing
        // `"extension does not provide capability"` config error
        // pointing at the wrong layer.
        let bundle_name = self
            .local
            .as_ref()
            .or(self.shared.as_ref())
            .map(|w| w.name().to_string())
            .unwrap_or_else(|| "<empty bundle>".to_string());
        if !capabilities.shared.is_empty() && self.shared.is_none() {
            return Err(crate::capability::registry::Error::InternalError {
                message: format!(
                    "extension '{bundle_name}': extension_capabilities! advertises shared \
                     capabilities {caps:?} but the ExtensionBundle has no shared variant \
                     - either add `.shared(...)` to the builder chain or remove the \
                     capabilities from the macro list",
                    caps = capabilities.shared,
                ),
            });
        }
        if !capabilities.local.is_empty() && self.local.is_none() {
            return Err(crate::capability::registry::Error::InternalError {
                message: format!(
                    "extension '{bundle_name}': extension_capabilities! advertises local \
                     capabilities {caps:?} but the ExtensionBundle has no local variant \
                     - either add `.local(...)` to the builder chain or remove the \
                     capabilities from the macro list",
                    caps = capabilities.local,
                ),
            });
        }

        if let Some(shared) = self.shared.as_ref()
            && let Some(factory) = shared.shared_instance_factory()
        {
            (capabilities.register_shared)(shared.name(), factory.clone(), registry)?;
        }
        if let Some(local) = self.local.as_ref()
            && let Some(factory) = local.local_instance_factory()
        {
            (capabilities.register_local)(local.name(), factory.clone(), registry)?;
        }
        Ok(())
    }
}

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
