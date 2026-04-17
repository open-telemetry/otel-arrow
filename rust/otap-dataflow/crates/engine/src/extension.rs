// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension wrapper and infrastructure.
//!
//! Extensions are PData-free — they never process pipeline data, only control
//! messages. This module defines [`EffectHandler`], [`ExtensionWrapper`],
//! [`ExtensionBundle`], and the builder infrastructure. Control channels
//! are defined in [`local::extension`](crate::local::extension) and
//! [`shared::extension`](crate::shared::extension).
//!
//! For the local (!Send) and shared (Send) Extension traits, see
//! [`local::extension`](crate::local::extension) and
//! [`shared::extension`](crate::shared::extension).

use crate::channel_metrics::ChannelMetricsRegistry;
use crate::channel_mode::{LocalMode, SharedMode, wrap_control_channel_metrics};
use crate::config::ExtensionConfig;
use crate::context::PipelineContext;
use crate::control::ExtensionControlMsg;
use crate::entity_context::NodeTelemetryGuard;
use crate::error::Error;
use crate::local::extension as local_ext;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::Sender;
use crate::shared::extension as shared_ext;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::terminal_state::TerminalState;
use otap_df_channel::error::RecvError;
use otap_df_channel::mpsc;
use otap_df_config::ExtensionId;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_telemetry::otel_debug;
use otap_df_telemetry::reporter::MetricsReporter;
use std::any::TypeId;
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

// ── Active / Passive wrappers ────────────────────────────────────────────────

/// Wraps an extension type to signal it has an active event loop.
pub struct Active<E>(pub E);

/// Wraps an extension type to signal it is passive (no lifecycle).
///
/// The wrapped value is consumed by `decompose()` and only its
/// `TypeId` is retained. The value itself will be stored when the capability
/// system is added.
pub struct Passive<E>(pub E);

/// Decomposed result of a shared extension provider.
#[doc(hidden)]
pub struct SharedDecomposed {
    pub(crate) extension: Option<Box<dyn shared_ext::Extension>>,
    /// Used by the same-type guard and the capability system.
    pub(crate) type_id: TypeId,
}

/// Decomposed result of a local extension provider.
#[doc(hidden)]
pub struct LocalDecomposed {
    pub(crate) extension: Option<std::rc::Rc<dyn local_ext::Extension>>,
    /// Used by the same-type guard and the capability system.
    pub(crate) type_id: TypeId,
}

/// Sealed trait for shared extension providers.
pub trait SharedProvider: sealed_provider::SealedShared {
    /// Decompose into type-erased components.
    fn decompose(self) -> SharedDecomposed;
}

/// Sealed trait for local extension providers.
pub trait LocalProvider: sealed_provider::SealedLocal {
    /// Decompose into type-erased components.
    fn decompose(self) -> LocalDecomposed;
}

mod sealed_provider {
    pub trait SealedShared {}
    pub trait SealedLocal {}
}

// Clone is required by the capability system for `Box<dyn CloneAnySend>`.
impl<E: shared_ext::Extension + Clone + Send + 'static> sealed_provider::SealedShared
    for Active<E>
{
}
impl<E: shared_ext::Extension + Clone + Send + 'static> SharedProvider for Active<E> {
    fn decompose(self) -> SharedDecomposed {
        SharedDecomposed {
            extension: Some(Box::new(self.0)),
            type_id: TypeId::of::<E>(),
        }
    }
}

impl<E: Clone + Send + 'static> sealed_provider::SealedShared for Passive<E> {}
impl<E: Clone + Send + 'static> SharedProvider for Passive<E> {
    fn decompose(self) -> SharedDecomposed {
        SharedDecomposed {
            extension: None,
            type_id: TypeId::of::<E>(),
        }
    }
}

impl<E: local_ext::Extension + 'static> sealed_provider::SealedLocal for Active<std::rc::Rc<E>> {}
impl<E: local_ext::Extension + 'static> LocalProvider for Active<std::rc::Rc<E>> {
    fn decompose(self) -> LocalDecomposed {
        LocalDecomposed {
            extension: Some(self.0),
            type_id: TypeId::of::<E>(),
        }
    }
}

impl<E: 'static> sealed_provider::SealedLocal for Passive<std::rc::Rc<E>> {}
impl<E: 'static> LocalProvider for Passive<std::rc::Rc<E>> {
    fn decompose(self) -> LocalDecomposed {
        LocalDecomposed {
            extension: None,
            type_id: TypeId::of::<E>(),
        }
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

// ── Builder ──────────────────────────────────────────────────────────────────

/// Builder for [`ExtensionBundle`].
///
/// At least one variant (local or shared) must be added before calling `build()`.
/// Both variants can be provided for dual-mode extensions.
pub struct ExtensionBundleBuilder {
    name: ExtensionId,
    user_config: Arc<ExtensionUserConfig>,
    runtime_config: ExtensionConfig,
    shared: Option<SharedDecomposed>,
    local: Option<LocalDecomposed>,
}

impl ExtensionBundleBuilder {
    /// Add a **local** (!Send) extension variant.
    pub fn with_local(mut self, provider: impl LocalProvider) -> Self {
        self.local = Some(provider.decompose());
        self
    }

    /// Add a **shared** (Send) extension variant.
    pub fn with_shared(mut self, provider: impl SharedProvider) -> Self {
        self.shared = Some(provider.decompose());
        self
    }

    /// Build the [`ExtensionBundle`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Neither `with_local` nor `with_shared` was called.
    /// - Both variants use the same concrete type (dual registration requires
    ///   distinct local and shared implementations).
    pub fn build(self) -> Result<ExtensionBundle, Error> {
        if let (Some(local), Some(shared)) = (&self.local, &self.shared) {
            if local.type_id == shared.type_id {
                return Err(Error::InternalError {
                    message: "local and shared variants must use different concrete types; \
                              use with_local() or with_shared() alone for single-variant extensions"
                        .into(),
                });
            }
        }

        let cap = self.runtime_config.control_channel.capacity;

        let local = self.local.map(|l| {
            let lifecycle = match l.extension {
                Some(ext) => {
                    let (tx, rx) = mpsc::Channel::new(cap);
                    ExtensionLifecycle::Active {
                        extension: ext,
                        control_sender: Sender::Local(LocalSender::mpsc(tx)),
                        control_receiver: LocalReceiver::mpsc(rx),
                    }
                }
                None => ExtensionLifecycle::Passive,
            };
            ExtensionWrapper::Local {
                name: self.name.clone(),
                user_config: self.user_config.clone(),
                runtime_config: self.runtime_config.clone(),
                telemetry: None,
                lifecycle,
            }
        });

        let shared = self.shared.map(|s| {
            let lifecycle = match s.extension {
                Some(ext) => {
                    let (tx, rx) = tokio::sync::mpsc::channel(cap);
                    ExtensionLifecycle::Active {
                        extension: ext,
                        control_sender: Sender::Shared(SharedSender::mpsc(tx)),
                        control_receiver: SharedReceiver::mpsc(rx),
                    }
                }
                None => ExtensionLifecycle::Passive,
            };
            ExtensionWrapper::Shared {
                name: self.name.clone(),
                user_config: self.user_config.clone(),
                runtime_config: self.runtime_config.clone(),
                telemetry: None,
                lifecycle,
            }
        });

        if local.is_none() && shared.is_none() {
            return Err(Error::InternalError {
                message: "ExtensionBundle must have at least one variant (local or shared)".into(),
            });
        }

        for w in local.iter().chain(shared.iter()) {
            let name = w.name();
            otel_debug!(
                "extension.builder.build",
                name = name.as_ref(),
                variant = match w {
                    ExtensionWrapper::Local { .. } => "local",
                    ExtensionWrapper::Shared { .. } => "shared",
                },
                lifecycle = if w.is_passive() { "passive" } else { "active" },
            );
        }

        Ok(ExtensionBundle { local, shared })
    }
}

impl ExtensionWrapper {
    /// Start building an [`ExtensionBundle`].
    #[must_use]
    pub fn builder(
        name: ExtensionId,
        user_config: Arc<ExtensionUserConfig>,
        config: &ExtensionConfig,
    ) -> ExtensionBundleBuilder {
        ExtensionBundleBuilder {
            name,
            user_config,
            runtime_config: config.clone(),
            shared: None,
            local: None,
        }
    }

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
                }
            }
            ExtensionWrapper::Shared {
                name,
                user_config,
                runtime_config,
                telemetry,
                lifecycle,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::ExtensionControlMsg;
    use crate::shared::extension::Extension as SharedExtension;
    use crate::testing::CtrlMsgCounters;
    use async_trait::async_trait;
    use serde_json::Value;
    use std::time::Instant;

    fn test_metrics_reporter() -> MetricsReporter {
        let (tx, _rx) = flume::bounded(1);
        MetricsReporter::new(tx)
    }

    fn ext_config(name: &'static str) -> (ExtensionId, Arc<ExtensionUserConfig>, ExtensionConfig) {
        (
            name.into(),
            Arc::new(ExtensionUserConfig::new(
                "urn:otap:test".into(),
                Value::Null,
            )),
            ExtensionConfig::new(name),
        )
    }

    #[derive(Clone)]
    struct TestSharedExt {
        counter: CtrlMsgCounters,
    }
    impl TestSharedExt {
        fn new(c: CtrlMsgCounters) -> Self {
            Self { counter: c }
        }
    }

    #[async_trait]
    impl SharedExtension for TestSharedExt {
        async fn start(
            self: Box<Self>,
            mut ctrl: shared_ext::ControlChannel,
            _eh: EffectHandler,
        ) -> Result<TerminalState, Error> {
            loop {
                match ctrl.recv().await? {
                    ExtensionControlMsg::Config { .. } => self.counter.increment_config(),
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

    #[derive(Clone)]
    struct TestLocalExt {
        _counter: CtrlMsgCounters,
    }
    impl TestLocalExt {
        fn new(c: CtrlMsgCounters) -> Self {
            Self { _counter: c }
        }
    }

    #[async_trait(?Send)]
    impl crate::local::extension::Extension for TestLocalExt {
        async fn start(
            self: std::rc::Rc<Self>,
            mut ctrl: local_ext::ControlChannel,
            _eh: EffectHandler,
        ) -> Result<TerminalState, Error> {
            loop {
                if let ExtensionControlMsg::Shutdown { .. } = ctrl.recv().await? {
                    break;
                }
            }
            Ok(TerminalState::default())
        }
    }

    #[test]
    fn test_shared_active() {
        let (n, u, c) = ext_config("sa");
        let set = ExtensionWrapper::builder(n, u, &c)
            .with_shared(Active(TestSharedExt::new(CtrlMsgCounters::new())))
            .build()
            .unwrap();
        assert!(set.local.is_none());
        let w = set.shared.unwrap();
        assert!(!w.is_passive());
        assert!(w.extension_control_sender().is_some());
    }

    #[test]
    fn test_shared_passive() {
        let (n, u, c) = ext_config("sp");
        let set = ExtensionWrapper::builder(n, u, &c)
            .with_shared(Passive("data".to_string()))
            .build()
            .unwrap();
        assert!(set.local.is_none());
        let w = set.shared.unwrap();
        assert!(w.is_passive());
        assert!(w.extension_control_sender().is_none());
    }

    #[test]
    fn test_local_active() {
        let (n, u, c) = ext_config("la");
        let set = ExtensionWrapper::builder(n, u, &c)
            .with_local(Active(std::rc::Rc::new(TestLocalExt::new(
                CtrlMsgCounters::new(),
            ))))
            .build()
            .unwrap();
        assert!(set.shared.is_none());
        let w = set.local.unwrap();
        assert!(!w.is_passive());
        assert!(w.extension_control_sender().is_some());
    }

    #[test]
    fn test_local_passive() {
        let (n, u, c) = ext_config("lp");
        let set = ExtensionWrapper::builder(n, u, &c)
            .with_local(Passive(std::rc::Rc::new(42u32)))
            .build()
            .unwrap();
        assert!(set.shared.is_none());
        let w = set.local.unwrap();
        assert!(w.is_passive());
    }

    #[test]
    fn test_empty_returns_error() {
        let (n, u, c) = ext_config("e");
        let result = ExtensionWrapper::builder(n, u, &c).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_same_type_dual_returns_error() {
        // TestSharedExt implements only shared::Extension, so we need
        // a type that implements both to test the same-type guard.
        // Since TestLocalExt and TestSharedExt are different types,
        // we can't easily trigger this. Instead, create a dual-impl type.
        #[derive(Clone)]
        struct DualExt;

        #[async_trait]
        impl SharedExtension for DualExt {
            async fn start(
                self: Box<Self>,
                _ctrl: shared_ext::ControlChannel,
                _eh: EffectHandler,
            ) -> Result<TerminalState, Error> {
                Ok(TerminalState::default())
            }
        }

        #[async_trait(?Send)]
        impl crate::local::extension::Extension for DualExt {
            async fn start(
                self: std::rc::Rc<Self>,
                _ctrl: local_ext::ControlChannel,
                _eh: EffectHandler,
            ) -> Result<TerminalState, Error> {
                Ok(TerminalState::default())
            }
        }

        let (n, u, c) = ext_config("st");
        let result = ExtensionWrapper::builder(n, u, &c)
            .with_local(Active(std::rc::Rc::new(DualExt)))
            .with_shared(Active(DualExt))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_dual_creates_local_and_shared() {
        let (n, u, c) = ext_config("d");
        let set = ExtensionWrapper::builder(n, u, &c)
            .with_local(Active(std::rc::Rc::new(TestLocalExt::new(
                CtrlMsgCounters::new(),
            ))))
            .with_shared(Active(TestSharedExt::new(CtrlMsgCounters::new())))
            .build()
            .unwrap();
        let local = set.local.unwrap();
        let shared = set.shared.unwrap();
        assert!(!local.is_passive());
        assert!(!shared.is_passive());
        assert!(local.extension_control_sender().is_some());
        assert!(shared.extension_control_sender().is_some());
        assert!(matches!(local, ExtensionWrapper::Local { .. }));
        assert!(matches!(shared, ExtensionWrapper::Shared { .. }));
    }

    #[test]
    fn test_name_and_config_accessors() {
        let (n, u, c) = ext_config("acc");
        let set = ExtensionWrapper::builder(n, u, &c)
            .with_shared(Active(TestSharedExt::new(CtrlMsgCounters::new())))
            .build()
            .unwrap();
        let w = set.shared.unwrap();
        assert_eq!(w.name().as_ref(), "acc");
        assert_eq!(w.user_config().r#type.as_ref(), "urn:otap:test");
    }

    #[test]
    fn test_control_msg_is_shutdown() {
        assert!(
            ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "t".into()
            }
            .is_shutdown()
        );
        assert!(
            !ExtensionControlMsg::Config {
                config: Value::Null
            }
            .is_shutdown()
        );
    }

    #[test]
    fn test_shared_start_shutdown() {
        let (rt, ls) = crate::testing::setup_test_runtime();
        rt.block_on(ls.run_until(async {
            let ctr = CtrlMsgCounters::new();
            let (n, u, c) = ext_config("ss");
            let w = ExtensionWrapper::builder(n, u, &c)
                .with_shared(Active(TestSharedExt::new(ctr.clone())))
                .build()
                .unwrap()
                .shared
                .unwrap();
            let s = w.extension_control_sender().unwrap();
            let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
            s.send(ExtensionControlMsg::Config {
                config: Value::Null,
            })
            .await
            .unwrap();
            s.send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "d".into(),
            })
            .await
            .unwrap();
            assert!(h.await.unwrap().is_ok());
            ctr.assert(0, 0, 1, 1);
        }));
    }

    #[test]
    fn test_local_start_shutdown() {
        let (rt, ls) = crate::testing::setup_test_runtime();
        rt.block_on(ls.run_until(async {
            let ctr = CtrlMsgCounters::new();
            let (n, u, c) = ext_config("ls");
            let w = ExtensionWrapper::builder(n, u, &c)
                .with_local(Active(std::rc::Rc::new(TestLocalExt::new(ctr.clone()))))
                .build()
                .unwrap()
                .local
                .unwrap();
            let s = w.extension_control_sender().unwrap();
            let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
            s.send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "d".into(),
            })
            .await
            .unwrap();
            assert!(h.await.unwrap().is_ok());
            ctr.assert(0, 0, 0, 0);
        }));
    }

    #[tokio::test]
    async fn test_ctrl_immediate_shutdown() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
        SharedSender::mpsc(tx)
            .send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "i".into(),
            })
            .await
            .unwrap();
        assert!(ch.recv().await.unwrap().is_shutdown());
        assert!(ch.recv().await.is_err());
    }

    #[tokio::test]
    async fn test_ctrl_config_then_shutdown() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let s = SharedSender::mpsc(tx);
        let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
        s.send(ExtensionControlMsg::Config {
            config: Value::String("h".into()),
        })
        .await
        .unwrap();
        s.send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "d".into(),
        })
        .await
        .unwrap();
        assert!(!ch.recv().await.unwrap().is_shutdown());
        assert!(ch.recv().await.unwrap().is_shutdown());
        assert!(ch.recv().await.is_err());
    }

    #[tokio::test]
    async fn test_ctrl_delayed_shutdown() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
        let dl = Instant::now() + std::time::Duration::from_millis(50);
        let sender = SharedSender::mpsc(tx);
        sender
            .send(ExtensionControlMsg::Shutdown {
                deadline: dl,
                reason: "dl".into(),
            })
            .await
            .unwrap();
        // Keep sender alive so control_rx doesn't close before the deadline.
        assert!(ch.recv().await.unwrap().is_shutdown());
        assert!(Instant::now() >= dl);
        drop(sender);
    }

    #[tokio::test]
    async fn test_ctrl_delayed_shutdown_sender_dropped() {
        // Edge case: sender drops during grace period. The pending shutdown
        // should still be delivered instead of returning RecvError::Closed.
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
        let dl = Instant::now() + std::time::Duration::from_millis(100);
        SharedSender::mpsc(tx)
            .send(ExtensionControlMsg::Shutdown {
                deadline: dl,
                reason: "dl".into(),
            })
            .await
            .unwrap();
        // Sender is dropped here — channel closes during grace period.
        // recv() should still return the pending shutdown, not Err(Closed).
        assert!(ch.recv().await.unwrap().is_shutdown());
    }

    #[tokio::test]
    async fn test_ctrl_collect_telemetry() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
        SharedSender::mpsc(tx)
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: test_metrics_reporter(),
            })
            .await
            .unwrap();
        assert!(matches!(
            ch.recv().await.unwrap(),
            ExtensionControlMsg::CollectTelemetry { .. }
        ));
    }

    #[tokio::test]
    async fn test_ctrl_closed() {
        let (tx, rx) = tokio::sync::mpsc::channel::<ExtensionControlMsg>(8);
        let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
        drop(tx);
        assert!(ch.recv().await.is_err());
    }

    #[tokio::test]
    async fn test_ctrl_sender_send() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let sender = crate::control::ExtensionControlSender {
            name: "st".into(),
            sender: Sender::Shared(SharedSender::mpsc(tx)),
        };
        sender
            .send(ExtensionControlMsg::Config {
                config: Value::Null,
            })
            .await
            .unwrap();
        assert!(matches!(
            rx.recv().await.unwrap(),
            ExtensionControlMsg::Config { .. }
        ));
    }

    #[test]
    fn test_effect_handler() {
        let h = EffectHandler::new("eh".into(), test_metrics_reporter());
        assert_eq!(h.extension_id().as_ref(), "eh");
        let c = h.clone();
        assert_eq!(c.extension_id().as_ref(), "eh");
    }

    #[test]
    fn test_passive_ctrl_metrics_noop() {
        let (n, u, c) = ext_config("pm");
        let w = ExtensionWrapper::builder(n, u, &c)
            .with_shared(Passive("d".to_string()))
            .build()
            .unwrap()
            .shared
            .unwrap();
        let (ctx, _) = crate::testing::test_pipeline_ctx();
        let mut cm = ChannelMetricsRegistry::default();
        let w = w.with_control_channel_metrics(&ctx, &mut cm, true);
        assert!(w.is_passive());
    }

    #[test]
    fn test_start_with_telemetry() {
        let (rt, ls) = crate::testing::setup_test_runtime();
        rt.block_on(ls.run_until(async {
            let ctr = CtrlMsgCounters::new();
            let (n, u, c) = ext_config("st");
            let w = ExtensionWrapper::builder(n, u, &c)
                .with_shared(Active(TestSharedExt::new(ctr.clone())))
                .build()
                .unwrap()
                .shared
                .unwrap();
            let s = w.extension_control_sender().unwrap();
            let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
            s.send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: test_metrics_reporter(),
            })
            .await
            .unwrap();
            s.send(ExtensionControlMsg::Config {
                config: Value::Null,
            })
            .await
            .unwrap();
            s.send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "d".into(),
            })
            .await
            .unwrap();
            assert!(h.await.unwrap().is_ok());
            ctr.assert(0, 0, 1, 1);
        }));
    }

    #[test]
    fn test_active_with_control_channel_metrics() {
        let (n, u, c) = ext_config("acm");
        let w = ExtensionWrapper::builder(n, u, &c)
            .with_shared(Active(TestSharedExt::new(CtrlMsgCounters::new())))
            .build()
            .unwrap()
            .shared
            .unwrap();

        let (ctx, _) = crate::testing::test_pipeline_ctx();
        let mut cm = ChannelMetricsRegistry::default();
        let w = w.with_control_channel_metrics(&ctx, &mut cm, true);
        assert!(!w.is_passive());
        assert!(w.extension_control_sender().is_some());
    }

    #[test]
    fn test_dual_passive() {
        let (n, u, c) = ext_config("dp");
        let set = ExtensionWrapper::builder(n, u, &c)
            .with_local(Passive(std::rc::Rc::new(42u32)))
            .with_shared(Passive("data".to_string()))
            .build()
            .unwrap();
        assert!(set.local.unwrap().is_passive());
        assert!(set.shared.unwrap().is_passive());
    }

    #[test]
    fn test_dual_mixed_active_passive() {
        let (n, u, c) = ext_config("dmap");
        let set = ExtensionWrapper::builder(n, u, &c)
            .with_local(Passive(std::rc::Rc::new(42u32)))
            .with_shared(Active(TestSharedExt::new(CtrlMsgCounters::new())))
            .build()
            .unwrap();
        assert!(set.local.unwrap().is_passive());
        assert!(!set.shared.unwrap().is_passive());
    }

    #[test]
    fn test_extension_set_iter() {
        let (n, u, c) = ext_config("it");
        let set = ExtensionWrapper::builder(n, u, &c)
            .with_local(Active(std::rc::Rc::new(TestLocalExt::new(
                CtrlMsgCounters::new(),
            ))))
            .with_shared(Active(TestSharedExt::new(CtrlMsgCounters::new())))
            .build()
            .unwrap();
        assert_eq!(set.iter().count(), 2);
    }
}
