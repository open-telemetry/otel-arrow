// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension wrapper and infrastructure.
//!
//! Extensions are PData-free — they never process pipeline data, only control
//! messages. This module defines [`ControlChannel`], [`EffectHandler`], and
//! the [`ExtensionWrapper`] struct that the engine uses to start and manage
//! extension instances.
//!
//! For the local (!Send) and shared (Send) Extension traits, see
//! [`local::extension`](crate::local::extension) and
//! [`shared::extension`](crate::shared::extension).

use crate::channel_metrics::ChannelMetricsRegistry;
use crate::channel_mode::{SharedMode, wrap_control_channel_metrics};
use crate::config::ExtensionConfig;
use crate::context::PipelineContext;
use crate::control::ExtensionControlMsg;
use crate::entity_context::NodeTelemetryGuard;
use crate::error::Error;
use crate::local::extension as local_ext;
use crate::node::NodeId;
use crate::shared::extension as shared_ext;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::terminal_state::TerminalState;
use otap_df_channel::error::RecvError;
use otap_df_config::node::NodeUserConfig;
use otap_df_telemetry::otel_debug;
use otap_df_telemetry::reporter::MetricsReporter;
use std::any::TypeId;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{Sleep, sleep_until};

// ── ControlChannel ──────────────────────────────────────────────────────────

/// A channel for receiving control messages for extensions.
///
/// When a `Shutdown` message arrives with a future deadline, the channel waits
/// until the deadline expires, then returns the `Shutdown`.
pub struct ControlChannel {
    control_rx: Option<SharedReceiver<ExtensionControlMsg>>,
    shutting_down_deadline: Option<Instant>,
    pending_shutdown: Option<ExtensionControlMsg>,
}

impl ControlChannel {
    /// Creates a new `ControlChannel` with the given control receiver.
    #[must_use]
    pub const fn new(control_rx: SharedReceiver<ExtensionControlMsg>) -> Self {
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

                tokio::select! {
                    biased;
                    _ = sleep_until_deadline.as_mut().expect("set above") => {
                        let shutdown = self.pending_shutdown
                            .take()
                            .expect("pending_shutdown must exist");
                        self.shutdown();
                        return Ok(shutdown);
                    }
                }
            }

            tokio::select! {
                biased;
                ctrl = self.control_rx.as_mut().expect("checked above").recv() => match ctrl {
                    Ok(ExtensionControlMsg::Shutdown { deadline, reason }) => {
                        if deadline.duration_since(Instant::now()).is_zero() {
                            self.shutdown();
                            return Ok(ExtensionControlMsg::Shutdown { deadline, reason });
                        }
                        self.shutting_down_deadline = Some(deadline);
                        self.pending_shutdown = Some(ExtensionControlMsg::Shutdown { deadline, reason });
                        continue;
                    }
                    Ok(msg) => return Ok(msg),
                    Err(e)  => return Err(e),
                },
            }
        }
    }

    fn shutdown(&mut self) {
        self.shutting_down_deadline = None;
        drop(self.control_rx.take().expect("control_rx must exist"));
    }
}

// ── EffectHandler ───────────────────────────────────────────────────────────

/// The effect handler for extensions.
///
/// Provides extensions with node identity and basic I/O. Extensions manage
/// their own timers directly via `tokio::time` rather than through the engine's
/// timer infrastructure, keeping the extension system fully PData-free.
#[derive(Clone)]
pub struct EffectHandler {
    node_id: NodeId,
    #[allow(dead_code)]
    metrics_reporter: MetricsReporter,
}

impl EffectHandler {
    /// Creates a new `EffectHandler` for the given extension node.
    #[must_use]
    pub const fn new(node_id: NodeId, metrics_reporter: MetricsReporter) -> Self {
        EffectHandler {
            node_id,
            metrics_reporter,
        }
    }

    /// Returns the id of the extension associated with this handler.
    #[must_use]
    pub fn extension_id(&self) -> NodeId {
        self.node_id.clone()
    }

    /// Print an info message to stdout.
    pub async fn info(&self, message: &str) {
        use tokio::io::{AsyncWriteExt, stdout};
        let mut out = stdout();
        let _ = out.write_all(message.as_bytes()).await;
        let _ = out.write_all(b"\n").await;
        let _ = out.flush().await;
    }
}

// ── Active / Passive wrappers ────────────────────────────────────────────────

/// Wraps an extension type to signal it has an active event loop.
///
/// The engine spawns a task and creates a control channel for active extensions.
/// The inner type must implement the appropriate `Extension` trait.
pub struct Active<E>(pub E);

/// Wraps an extension type to signal it is passive (no lifecycle).
///
/// No task is spawned, no control channel is created.
pub struct Passive<E>(pub E);

/// Decomposed result of a shared extension provider.
#[doc(hidden)]
pub struct SharedDecomposed {
    pub extension: Option<Box<dyn shared_ext::Extension>>,
    pub type_id: TypeId,
}

/// Decomposed result of a local extension provider.
#[doc(hidden)]
pub struct LocalDecomposed {
    pub extension: Option<std::rc::Rc<dyn local_ext::Extension>>,
    pub type_id: TypeId,
}

/// Sealed trait for shared extension providers (Active or Passive).
pub trait SharedProvider: sealed_provider::SealedShared {
    /// Decompose into type-erased components.
    fn decompose(self) -> SharedDecomposed;
}

/// Sealed trait for local extension providers (Active or Passive).
pub trait LocalProvider: sealed_provider::SealedLocal {
    /// Decompose into type-erased components.
    fn decompose(self) -> LocalDecomposed;
}

mod sealed_provider {
    pub trait SealedShared {}
    pub trait SealedLocal {}
}

// Active<E> shared: requires Extension + Clone + Send
impl<E: shared_ext::Extension + Clone + Send + 'static> sealed_provider::SealedShared
    for Active<E>
{
}

impl<E: shared_ext::Extension + Clone + Send + 'static> SharedProvider for Active<E> {
    fn decompose(self) -> SharedDecomposed {
        let ext: Box<dyn shared_ext::Extension> = Box::new(self.0);
        SharedDecomposed {
            extension: Some(ext),
            type_id: TypeId::of::<E>(),
        }
    }
}

// Passive<E> shared: no Extension bound needed
impl<E: Clone + Send + 'static> sealed_provider::SealedShared for Passive<E> {}

impl<E: Clone + Send + 'static> SharedProvider for Passive<E> {
    fn decompose(self) -> SharedDecomposed {
        SharedDecomposed {
            extension: None,
            type_id: TypeId::of::<E>(),
        }
    }
}

// Active<Rc<E>> local: requires local Extension
impl<E: local_ext::Extension + 'static> sealed_provider::SealedLocal for Active<std::rc::Rc<E>> {}

impl<E: local_ext::Extension + 'static> LocalProvider for Active<std::rc::Rc<E>> {
    fn decompose(self) -> LocalDecomposed {
        let ext: std::rc::Rc<dyn local_ext::Extension> = self.0;
        LocalDecomposed {
            extension: Some(ext),
            type_id: TypeId::of::<E>(),
        }
    }
}

// Passive<Rc<E>> local: no Extension bound needed
impl<E: 'static> sealed_provider::SealedLocal for Passive<std::rc::Rc<E>> {}

impl<E: 'static> LocalProvider for Passive<std::rc::Rc<E>> {
    fn decompose(self) -> LocalDecomposed {
        LocalDecomposed {
            extension: None,
            type_id: TypeId::of::<E>(),
        }
    }
}

// ── ExtensionWrapper ────────────────────────────────────────────────────────

/// Wrapper for extension instances in the pipeline engine.
///
/// Extensions are NOT generic over PData — they operate exclusively on
/// [`ExtensionControlMsg`], keeping the extension system entirely decoupled
/// from the data-plane type.
pub struct ExtensionWrapper {
    node_id: NodeId,
    user_config: Arc<NodeUserConfig>,
    runtime_config: ExtensionConfig,
    shared_extension: Option<Box<dyn shared_ext::Extension>>,
    local_extension: Option<std::rc::Rc<dyn local_ext::Extension>>,
    control_sender: Option<SharedSender<ExtensionControlMsg>>,
    control_receiver: Option<SharedReceiver<ExtensionControlMsg>>,
    shared_control_sender: Option<SharedSender<ExtensionControlMsg>>,
    shared_control_receiver: Option<SharedReceiver<ExtensionControlMsg>>,
    telemetry: Option<NodeTelemetryGuard>,
}

// ── Builder ──────────────────────────────────────────────────────────────────

/// Builder for `ExtensionWrapper`.
///
/// At least one variant (local or shared) must be added before calling `build()`.
pub struct ExtensionWrapperBuilder {
    node_id: NodeId,
    user_config: Arc<NodeUserConfig>,
    runtime_config: ExtensionConfig,
    shared_extension: Option<Box<dyn shared_ext::Extension>>,
    local_extension: Option<std::rc::Rc<dyn local_ext::Extension>>,
    shared_type_id: Option<TypeId>,
    local_type_id: Option<TypeId>,
}

impl ExtensionWrapperBuilder {
    /// Add a **local** (!Send) extension variant.
    ///
    /// Use `Active(Rc::new(ext))` for extensions with an event loop,
    /// or `Passive(Rc::new(ext))` for capability-only extensions.
    pub fn with_local(mut self, provider: impl LocalProvider) -> Self {
        let decomposed = provider.decompose();
        otel_debug!(
            "extension.builder.with_local",
            node_id = self.node_id.name.as_ref(),
            active = decomposed.extension.is_some(),
        );
        self.local_extension = decomposed.extension;
        self.local_type_id = Some(decomposed.type_id);
        self
    }

    /// Add a **shared** (Send) extension variant.
    ///
    /// Use `Active(ext)` for extensions with an event loop,
    /// or `Passive(ext)` for capability-only extensions.
    pub fn with_shared(mut self, provider: impl SharedProvider) -> Self {
        let decomposed = provider.decompose();
        otel_debug!(
            "extension.builder.with_shared",
            node_id = self.node_id.name.as_ref(),
            active = decomposed.extension.is_some(),
        );
        self.shared_extension = decomposed.extension;
        self.shared_type_id = Some(decomposed.type_id);
        self
    }

    /// Build the `ExtensionWrapper`.
    ///
    /// # Panics
    ///
    /// - Panics if neither `with_local` nor `with_shared` was called.
    /// - Panics if both were called with the same concrete type.
    pub fn build(self) -> ExtensionWrapper {
        let has_local = self.local_extension.is_some() || self.local_type_id.is_some();
        let has_shared = self.shared_extension.is_some() || self.shared_type_id.is_some();
        assert!(
            has_local || has_shared,
            "ExtensionWrapper must have at least one variant (local or shared)"
        );

        // TypeId guard: when both variants are provided, they must be different types.
        if let (Some(local_tid), Some(shared_tid)) = (self.local_type_id, self.shared_type_id) {
            assert!(
                local_tid != shared_tid,
                "with_local() and with_shared() called with the same concrete type — \
                 use with_shared() alone when a single type should serve both \
                 local and shared consumers"
            );
        }

        let has_local_lifecycle = self.local_extension.is_some();
        let has_shared_lifecycle = self.shared_extension.is_some();
        let has_any_lifecycle = has_local_lifecycle || has_shared_lifecycle;
        let has_both_lifecycles = has_local_lifecycle && has_shared_lifecycle;

        let (control_sender, control_receiver) = if has_any_lifecycle {
            let (tx, rx) = tokio::sync::mpsc::channel(self.runtime_config.control_channel.capacity);
            (Some(SharedSender::mpsc(tx)), Some(SharedReceiver::mpsc(rx)))
        } else {
            (None, None)
        };

        // Dual control channels when both variants have active lifecycles.
        let (shared_control_sender, shared_control_receiver) = if has_both_lifecycles {
            let (tx, rx) = tokio::sync::mpsc::channel(self.runtime_config.control_channel.capacity);
            (Some(SharedSender::mpsc(tx)), Some(SharedReceiver::mpsc(rx)))
        } else {
            (None, None)
        };

        otel_debug!(
            "extension.builder.build",
            node_id = self.node_id.name.as_ref(),
            has_local_lifecycle = has_local_lifecycle,
            has_shared_lifecycle = has_shared_lifecycle,
        );

        ExtensionWrapper {
            node_id: self.node_id,
            user_config: self.user_config,
            runtime_config: self.runtime_config,
            shared_extension: self.shared_extension,
            local_extension: self.local_extension,
            control_sender,
            control_receiver,
            shared_control_sender,
            shared_control_receiver,
            telemetry: None,
        }
    }
}

impl ExtensionWrapper {
    /// Start building an `ExtensionWrapper`.
    #[must_use]
    pub fn builder(
        node_id: NodeId,
        user_config: Arc<NodeUserConfig>,
        config: &ExtensionConfig,
    ) -> ExtensionWrapperBuilder {
        ExtensionWrapperBuilder {
            node_id,
            user_config,
            runtime_config: config.clone(),
            shared_extension: None,
            local_extension: None,
            shared_type_id: None,
            local_type_id: None,
        }
    }

    /// Returns the node ID of this extension.
    #[must_use]
    pub fn node_id(&self) -> NodeId {
        self.node_id.clone()
    }

    /// Returns the user configuration for this extension.
    #[must_use]
    pub fn user_config(&self) -> Arc<NodeUserConfig> {
        self.user_config.clone()
    }

    /// Returns `true` if this extension is passive (no active lifecycle).
    #[must_use]
    pub fn is_passive(&self) -> bool {
        self.local_extension.is_none() && self.shared_extension.is_none()
    }

    pub(crate) fn with_node_telemetry_guard(mut self, guard: NodeTelemetryGuard) -> Self {
        self.telemetry = Some(guard);
        self
    }

    pub(crate) fn with_control_channel_metrics(
        mut self,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Self {
        if self.control_sender.is_none() {
            return self;
        }

        let control_sender = self.control_sender.take().expect("checked above");
        let control_receiver = self.control_receiver.take().expect("must exist");
        let (wrapped_sender, wrapped_receiver) =
            wrap_control_channel_metrics::<SharedMode, ExtensionControlMsg>(
                &self.node_id,
                pipeline_ctx,
                channel_metrics,
                channel_metrics_enabled,
                self.runtime_config.control_channel.capacity as u64,
                control_sender,
                control_receiver,
            );
        self.control_sender = Some(wrapped_sender);
        self.control_receiver = Some(wrapped_receiver);

        if let (Some(shared_sender), Some(shared_receiver)) = (
            self.shared_control_sender.take(),
            self.shared_control_receiver.take(),
        ) {
            let (wrapped_sender, wrapped_receiver) =
                wrap_control_channel_metrics::<SharedMode, ExtensionControlMsg>(
                    &self.node_id,
                    pipeline_ctx,
                    channel_metrics,
                    channel_metrics_enabled,
                    self.runtime_config.control_channel.capacity as u64,
                    shared_sender,
                    shared_receiver,
                );
            self.shared_control_sender = Some(wrapped_sender);
            self.shared_control_receiver = Some(wrapped_receiver);
        }

        self
    }

    /// Returns `ExtensionControlSender`(s) for sending control messages.
    #[allow(dead_code)] // Used by runtime pipeline in a follow-up PR.
    pub(crate) fn extension_control_senders(&self) -> Vec<crate::control::ExtensionControlSender> {
        let mut senders = Vec::new();
        if let Some(ref sender) = self.control_sender {
            senders.push(crate::control::ExtensionControlSender {
                node_id: self.node_id.clone(),
                sender: crate::message::Sender::Shared(sender.clone()),
            });
        }
        if let Some(ref shared_sender) = self.shared_control_sender {
            senders.push(crate::control::ExtensionControlSender {
                node_id: self.node_id.clone(),
                sender: crate::message::Sender::Shared(shared_sender.clone()),
            });
        }
        senders
    }

    /// Starts the extension lifecycle(s).
    ///
    /// For dual-lifecycle extensions, spawns the shared variant as a background
    /// task and awaits the local variant on the current thread.
    pub async fn start(self, metrics_reporter: MetricsReporter) -> Result<TerminalState, Error> {
        let node_name = self.node_id.name.clone();
        let effect_handler = EffectHandler::new(self.node_id, metrics_reporter);
        let control_receiver = self
            .control_receiver
            .expect("start() called on passive extension — this is a bug");
        let ctrl_chan = ControlChannel::new(control_receiver);

        match (self.local_extension, self.shared_extension) {
            (Some(_), Some(_)) => {
                // The engine starts each active lifecycle independently — it
                // should never call start() on a wrapper that still holds both.
                // The runtime pipeline will take each variant separately.
                unreachable!(
                    "extension `{}` has both active lifecycles — the engine must \
                     start each variant independently, not through a single start() call",
                    node_name
                );
            }
            (Some(local_ext), None) => {
                otel_debug!("extension.start.local", node_id = node_name.as_ref(),);
                local_ext.start(ctrl_chan, effect_handler).await
            }
            (None, Some(shared_ext)) => {
                otel_debug!("extension.start.shared", node_id = node_name.as_ref(),);
                shared_ext.start(ctrl_chan, effect_handler).await
            }
            (None, None) => {
                panic!("ExtensionWrapper has no extension instance — this is a bug")
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
    use crate::shared::extension::Extension as SharedExtension;
    use crate::testing::{CtrlMsgCounters, test_node};
    use async_trait::async_trait;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_telemetry::reporter::MetricsReporter;
    use serde_json::Value;

    fn test_metrics_reporter() -> MetricsReporter {
        let (tx, _rx) = flume::bounded(1);
        MetricsReporter::new(tx)
    }

    #[derive(Clone)]
    struct TestExtension {
        counter: CtrlMsgCounters,
    }

    impl TestExtension {
        fn new(counter: CtrlMsgCounters) -> Self {
            TestExtension { counter }
        }
    }

    #[async_trait]
    impl SharedExtension for TestExtension {
        async fn start(
            self: Box<Self>,
            mut ctrl_chan: ControlChannel,
            _effect_handler: EffectHandler,
        ) -> Result<TerminalState, Error> {
            loop {
                match ctrl_chan.recv().await? {
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

    #[async_trait(?Send)]
    impl crate::local::extension::Extension for TestExtension {
        async fn start(
            self: std::rc::Rc<Self>,
            mut ctrl_chan: ControlChannel,
            _effect_handler: EffectHandler,
        ) -> Result<TerminalState, Error> {
            loop {
                if let ExtensionControlMsg::Shutdown { .. } = ctrl_chan.recv().await? {
                    break;
                }
            }
            Ok(TerminalState::default())
        }
    }

    #[test]
    fn test_shared_active_wrapper_creation() {
        let counter = CtrlMsgCounters::new();
        let extension = TestExtension::new(counter);
        let node_id = test_node("test_extension");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("test_extension");

        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_shared(Active(extension))
            .build();

        assert!(!wrapper.is_passive());
    }

    #[test]
    fn test_shared_passive_wrapper_creation() {
        let node_id = test_node("test_passive");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("test_passive");

        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_shared(Passive(String::from("passive_data")))
            .build();

        assert!(wrapper.is_passive());
    }

    #[test]
    #[should_panic(expected = "at least one variant")]
    fn test_empty_builder_panics() {
        let node_id = test_node("empty");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("empty");

        let _ = ExtensionWrapper::builder(node_id, user_config, &config).build();
    }

    #[test]
    #[should_panic(expected = "same concrete type")]
    fn test_same_type_dual_registration_panics() {
        let counter = CtrlMsgCounters::new();
        let ext = TestExtension::new(counter);
        let node_id = test_node("dual_same");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("dual_same");

        let _ = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_local(Active(std::rc::Rc::new(ext.clone())))
            .with_shared(Active(ext))
            .build();
    }

    #[test]
    fn test_local_active_wrapper_creation() {
        let counter = CtrlMsgCounters::new();
        let ext = TestExtension::new(counter);
        let node_id = test_node("local_active");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("local_active");

        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_local(Active(std::rc::Rc::new(ext)))
            .build();

        assert!(!wrapper.is_passive());
    }

    #[test]
    fn test_local_passive_wrapper_creation() {
        let node_id = test_node("local_passive");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("local_passive");

        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_local(Passive(std::rc::Rc::new(42u32)))
            .build();

        assert!(wrapper.is_passive());
    }

    #[test]
    fn test_dual_type_different_types_succeeds() {
        let counter = CtrlMsgCounters::new();
        let ext = TestExtension::new(counter);
        let node_id = test_node("dual_diff");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("dual_diff");

        // Local passive with a different type (u32), shared active with TestExtension.
        // Different TypeIds should not panic.
        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_local(Passive(std::rc::Rc::new(42u32)))
            .with_shared(Active(ext))
            .build();

        // Shared is active, so not fully passive.
        assert!(!wrapper.is_passive());
    }

    #[test]
    fn test_node_id_and_user_config_accessors() {
        let counter = CtrlMsgCounters::new();
        let ext = TestExtension::new(counter);
        let node_id = test_node("accessors");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("accessors");

        let wrapper = ExtensionWrapper::builder(node_id, user_config.clone(), &config)
            .with_shared(Active(ext))
            .build();

        assert_eq!(wrapper.node_id().name.as_ref(), "accessors");
        assert_eq!(
            wrapper.user_config().r#type.as_ref(),
            "urn:otap:extension:test"
        );
    }

    #[test]
    fn test_extension_control_senders_active() {
        let counter = CtrlMsgCounters::new();
        let ext = TestExtension::new(counter);
        let node_id = test_node("ctrl_senders");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("ctrl_senders");

        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_shared(Active(ext))
            .build();

        let senders = wrapper.extension_control_senders();
        assert_eq!(senders.len(), 1);
    }

    #[test]
    fn test_extension_control_senders_passive_empty() {
        let node_id = test_node("ctrl_passive");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("ctrl_passive");

        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_shared(Passive(String::from("data")))
            .build();

        let senders = wrapper.extension_control_senders();
        assert!(senders.is_empty());
    }

    #[test]
    fn test_extension_control_senders_dual_active() {
        // Use a genuinely different local type to satisfy the TypeId guard.
        #[derive(Clone)]
        struct LocalTestExtension;

        #[async_trait(?Send)]
        impl crate::local::extension::Extension for LocalTestExtension {
            async fn start(
                self: std::rc::Rc<Self>,
                mut ctrl_chan: ControlChannel,
                _effect_handler: EffectHandler,
            ) -> Result<TerminalState, Error> {
                loop {
                    if let ExtensionControlMsg::Shutdown { .. } = ctrl_chan.recv().await? {
                        break;
                    }
                }
                Ok(TerminalState::default())
            }
        }

        let counter = CtrlMsgCounters::new();
        let ext = TestExtension::new(counter);
        let node_id = test_node("ctrl_dual");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("ctrl_dual");

        // Different types: Rc<LocalTestExtension> (local) vs TestExtension (shared)
        // Both active -> two control senders
        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_local(Active(std::rc::Rc::new(LocalTestExtension)))
            .with_shared(Active(ext))
            .build();

        let senders = wrapper.extension_control_senders();
        assert_eq!(senders.len(), 2);
    }

    #[test]
    fn test_extension_control_msg_is_shutdown() {
        let shutdown = ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "test".into(),
        };
        assert!(shutdown.is_shutdown());

        let config = ExtensionControlMsg::Config {
            config: Value::Null,
        };
        assert!(!config.is_shutdown());
    }

    #[test]
    fn test_shared_active_start_and_shutdown() {
        let (rt, local_set) = crate::testing::setup_test_runtime();
        rt.block_on(local_set.run_until(async {
            let counter = CtrlMsgCounters::new();
            let ext = TestExtension::new(counter.clone());
            let node_id = test_node("start_shared");
            let user_config = Arc::new(NodeUserConfig::with_user_config(
                "urn:otap:extension:test".into(),
                Value::Null,
            ));
            let config = ExtensionConfig::new("start_shared");

            let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
                .with_shared(Active(ext))
                .build();

            let senders = wrapper.extension_control_senders();
            assert_eq!(senders.len(), 1);

            let metrics_reporter = test_metrics_reporter();
            let handle =
                tokio::task::spawn_local(async move { wrapper.start(metrics_reporter).await });

            // Send config then shutdown
            senders[0]
                .send(ExtensionControlMsg::Config {
                    config: Value::Null,
                })
                .await
                .unwrap();
            senders[0]
                .send(ExtensionControlMsg::Shutdown {
                    deadline: Instant::now(),
                    reason: "done".into(),
                })
                .await
                .unwrap();

            let result = handle.await.unwrap();
            assert!(result.is_ok());
            counter.assert(0, 0, 1, 1);
        }));
    }

    #[test]
    fn test_local_active_start_and_shutdown() {
        let (rt, local_set) = crate::testing::setup_test_runtime();
        rt.block_on(local_set.run_until(async {
            let counter = CtrlMsgCounters::new();
            let ext = TestExtension::new(counter.clone());
            let node_id = test_node("start_local");
            let user_config = Arc::new(NodeUserConfig::with_user_config(
                "urn:otap:extension:test".into(),
                Value::Null,
            ));
            let config = ExtensionConfig::new("start_local");

            let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
                .with_local(Active(std::rc::Rc::new(ext)))
                .build();

            let senders = wrapper.extension_control_senders();
            assert_eq!(senders.len(), 1);

            let metrics_reporter = test_metrics_reporter();
            let handle =
                tokio::task::spawn_local(async move { wrapper.start(metrics_reporter).await });

            senders[0]
                .send(ExtensionControlMsg::Shutdown {
                    deadline: Instant::now(),
                    reason: "done".into(),
                })
                .await
                .unwrap();

            let result = handle.await.unwrap();
            assert!(result.is_ok());
            // Local impl breaks on shutdown without incrementing counters
            counter.assert(0, 0, 0, 0);
        }));
    }

    #[tokio::test]
    async fn test_control_channel_immediate_shutdown() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let sender = SharedSender::mpsc(tx);
        let receiver = SharedReceiver::mpsc(rx);
        let mut chan = ControlChannel::new(receiver);

        sender
            .send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "immediate".into(),
            })
            .await
            .unwrap();

        let msg = chan.recv().await.unwrap();
        assert!(msg.is_shutdown());

        // After shutdown, channel should be closed
        let err = chan.recv().await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_control_channel_config_before_shutdown() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let sender = SharedSender::mpsc(tx);
        let receiver = SharedReceiver::mpsc(rx);
        let mut chan = ControlChannel::new(receiver);

        sender
            .send(ExtensionControlMsg::Config {
                config: Value::String("hello".into()),
            })
            .await
            .unwrap();
        sender
            .send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "done".into(),
            })
            .await
            .unwrap();

        // Config arrives first
        let msg1 = chan.recv().await.unwrap();
        assert!(!msg1.is_shutdown());

        // Then shutdown
        let msg2 = chan.recv().await.unwrap();
        assert!(msg2.is_shutdown());

        // Then closed
        assert!(chan.recv().await.is_err());
    }

    #[test]
    fn test_effect_handler_extension_id() {
        let node_id = test_node("eh_test");
        let metrics_reporter = test_metrics_reporter();
        let handler = EffectHandler::new(node_id, metrics_reporter);
        assert_eq!(handler.extension_id().name.as_ref(), "eh_test");
    }

    #[tokio::test]
    async fn test_effect_handler_info() {
        let node_id = test_node("eh_info");
        let metrics_reporter = test_metrics_reporter();
        let handler = EffectHandler::new(node_id, metrics_reporter);
        // Just verify it doesn't panic; output goes to stdout.
        handler.info("test message").await;
    }

    #[tokio::test]
    async fn test_control_channel_delayed_shutdown() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let sender = SharedSender::mpsc(tx);
        let receiver = SharedReceiver::mpsc(rx);
        let mut chan = ControlChannel::new(receiver);

        // Send shutdown with a short future deadline
        let deadline = Instant::now() + std::time::Duration::from_millis(50);
        sender
            .send(ExtensionControlMsg::Shutdown {
                deadline,
                reason: "delayed".into(),
            })
            .await
            .unwrap();

        // Should wait until the deadline, then return shutdown
        let msg = chan.recv().await.unwrap();
        assert!(msg.is_shutdown());
        assert!(Instant::now() >= deadline);

        // Channel should be closed after shutdown
        assert!(chan.recv().await.is_err());
    }

    #[tokio::test]
    async fn test_control_channel_collect_telemetry() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let sender = SharedSender::mpsc(tx);
        let receiver = SharedReceiver::mpsc(rx);
        let mut chan = ControlChannel::new(receiver);

        let reporter = test_metrics_reporter();
        sender
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: reporter,
            })
            .await
            .unwrap();

        let msg = chan.recv().await.unwrap();
        assert!(!msg.is_shutdown());
        assert!(matches!(msg, ExtensionControlMsg::CollectTelemetry { .. }));
    }

    #[tokio::test]
    async fn test_control_channel_closed_immediately() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let receiver = SharedReceiver::mpsc(rx);
        let mut chan = ControlChannel::new(receiver);

        // Drop sender to close the channel
        drop(tx);

        let result = chan.recv().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extension_control_sender_send() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let sender = crate::control::ExtensionControlSender {
            node_id: test_node("sender_test"),
            sender: crate::message::Sender::Shared(SharedSender::mpsc(tx)),
        };

        sender
            .send(ExtensionControlMsg::Config {
                config: Value::String("hello".into()),
            })
            .await
            .unwrap();

        let msg = rx.recv().await.unwrap();
        assert!(matches!(msg, ExtensionControlMsg::Config { .. }));
    }

    #[test]
    fn test_shared_active_start_and_shutdown_with_collect_telemetry() {
        let (rt, local_set) = crate::testing::setup_test_runtime();
        rt.block_on(local_set.run_until(async {
            let counter = CtrlMsgCounters::new();
            let ext = TestExtension::new(counter.clone());
            let node_id = test_node("start_telem");
            let user_config = Arc::new(NodeUserConfig::with_user_config(
                "urn:otap:extension:test".into(),
                Value::Null,
            ));
            let config = ExtensionConfig::new("start_telem");

            let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
                .with_shared(Active(ext))
                .build();

            let senders = wrapper.extension_control_senders();
            let metrics_reporter = test_metrics_reporter();
            let handle =
                tokio::task::spawn_local(async move { wrapper.start(metrics_reporter).await });

            // Send CollectTelemetry, Config, then Shutdown
            senders[0]
                .send(ExtensionControlMsg::CollectTelemetry {
                    metrics_reporter: test_metrics_reporter(),
                })
                .await
                .unwrap();
            senders[0]
                .send(ExtensionControlMsg::Config {
                    config: Value::Null,
                })
                .await
                .unwrap();
            senders[0]
                .send(ExtensionControlMsg::Shutdown {
                    deadline: Instant::now(),
                    reason: "done".into(),
                })
                .await
                .unwrap();

            let result = handle.await.unwrap();
            assert!(result.is_ok());
            counter.assert(0, 0, 1, 1);
        }));
    }

    #[test]
    fn test_passive_extension_with_control_channel_metrics_noop() {
        let node_id = test_node("passive_metrics");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("passive_metrics");

        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_shared(Passive(String::from("data")))
            .build();

        // Passive has no control channels, so with_control_channel_metrics is a noop.
        let (ctx, _registry) = crate::testing::test_pipeline_ctx();
        let mut channel_metrics = ChannelMetricsRegistry::default();
        let wrapper = wrapper.with_control_channel_metrics(&ctx, &mut channel_metrics, true);

        assert!(wrapper.is_passive());
    }

    #[test]
    fn test_effect_handler_clone() {
        let node_id = test_node("eh_clone");
        let metrics_reporter = test_metrics_reporter();
        let handler = EffectHandler::new(node_id, metrics_reporter);
        let cloned = handler.clone();
        assert_eq!(
            handler.extension_id().name.as_ref(),
            cloned.extension_id().name.as_ref()
        );
    }

    #[test]
    fn test_dual_passive_different_types() {
        let node_id = test_node("dual_passive");
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            "urn:otap:extension:test".into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new("dual_passive");

        // Both passive, different types
        let wrapper = ExtensionWrapper::builder(node_id, user_config, &config)
            .with_local(Passive(std::rc::Rc::new(42u32)))
            .with_shared(Passive(String::from("data")))
            .build();

        assert!(wrapper.is_passive());
        assert!(wrapper.extension_control_senders().is_empty());
    }
}
