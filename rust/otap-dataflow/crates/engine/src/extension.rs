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
use crate::shared::extension as shared_ext;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::terminal_state::TerminalState;
use otap_df_channel::error::RecvError;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_telemetry::otel_debug;
use otap_df_telemetry::reporter::MetricsReporter;
use std::any::TypeId;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{Sleep, sleep_until};

/// Name type for extensions — separate from node names.
pub type ExtensionName = otap_df_config::ExtensionId;

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
/// Provides extensions with identity and basic I/O. Extensions manage their
/// own timers directly via `tokio::time` rather than through the engine's
/// timer infrastructure, keeping the extension system fully PData-free.
#[derive(Clone)]
pub struct EffectHandler {
    name: ExtensionName,
    #[allow(dead_code)]
    metrics_reporter: MetricsReporter,
}

impl EffectHandler {
    /// Creates a new `EffectHandler` for the given extension.
    #[must_use]
    pub const fn new(name: ExtensionName, metrics_reporter: MetricsReporter) -> Self {
        EffectHandler {
            name,
            metrics_reporter,
        }
    }

    /// Returns the name of the extension associated with this handler.
    #[must_use]
    pub fn extension_id(&self) -> ExtensionName {
        self.name.clone()
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
pub struct Active<E>(pub E);

/// Wraps an extension type to signal it is passive (no lifecycle).
///
/// In this PR the wrapped value is consumed by `decompose()` and only its
/// `TypeId` is retained. The value itself will be stored when the capability
/// system is added.
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

// ── ExtensionLifecycle ──────────────────────────────────────────────────────

/// The lifecycle variant of an extension.
///
/// Each active variant bundles the extension trait object with its control
/// channel, making impossible states unrepresentable.
pub(crate) enum ExtensionLifecycle {
    /// No task spawned, no control channel. Capabilities only.
    Passive,
    /// A local (!Send) extension with an active event loop.
    LocalActive {
        extension: std::rc::Rc<dyn local_ext::Extension>,
        control_sender: SharedSender<ExtensionControlMsg>,
        control_receiver: SharedReceiver<ExtensionControlMsg>,
    },
    /// A shared (Send) extension with an active event loop.
    SharedActive {
        extension: Box<dyn shared_ext::Extension>,
        control_sender: SharedSender<ExtensionControlMsg>,
        control_receiver: SharedReceiver<ExtensionControlMsg>,
    },
}

// ── ExtensionWrapper ────────────────────────────────────────────────────────

/// Wrapper for a single extension instance in the pipeline engine.
///
/// Extensions are NOT generic over PData. The lifecycle enum determines
/// whether a task is spawned and whether control channels exist.
///
/// For dual-type extensions (both local and shared), the factory should
/// create two separate `ExtensionWrapper`s — one per variant.
pub struct ExtensionWrapper {
    name: ExtensionName,
    user_config: Arc<ExtensionUserConfig>,
    runtime_config: ExtensionConfig,
    lifecycle: ExtensionLifecycle,
    telemetry: Option<NodeTelemetryGuard>,
}

// ── Builder ──────────────────────────────────────────────────────────────────

/// Builder for `ExtensionWrapper`.
///
/// Call exactly one of `with_local()` or `with_shared()`, then `build()`.
pub struct ExtensionWrapperBuilder {
    name: ExtensionName,
    user_config: Arc<ExtensionUserConfig>,
    runtime_config: ExtensionConfig,
    shared: Option<SharedDecomposed>,
    local: Option<LocalDecomposed>,
}

impl ExtensionWrapperBuilder {
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

    /// Build the `ExtensionWrapper`(s).
    ///
    /// Returns one wrapper for single-variant extensions, or two wrappers
    /// for dual-type extensions (one local, one shared).
    ///
    /// # Panics
    ///
    /// Panics if neither `with_local` nor `with_shared` was called.
    pub fn build(self) -> Vec<ExtensionWrapper> {
        let cap = self.runtime_config.control_channel.capacity;
        let mut wrappers = Vec::new();

        if let Some(l) = self.local {
            let lifecycle = match l.extension {
                Some(ext) => {
                    let (tx, rx) = tokio::sync::mpsc::channel(cap);
                    ExtensionLifecycle::LocalActive {
                        extension: ext,
                        control_sender: SharedSender::mpsc(tx),
                        control_receiver: SharedReceiver::mpsc(rx),
                    }
                }
                None => ExtensionLifecycle::Passive,
            };
            wrappers.push(ExtensionWrapper {
                name: self.name.clone(),
                user_config: self.user_config.clone(),
                runtime_config: self.runtime_config.clone(),
                lifecycle,
                telemetry: None,
            });
        }

        if let Some(s) = self.shared {
            let lifecycle = match s.extension {
                Some(ext) => {
                    let (tx, rx) = tokio::sync::mpsc::channel(cap);
                    ExtensionLifecycle::SharedActive {
                        extension: ext,
                        control_sender: SharedSender::mpsc(tx),
                        control_receiver: SharedReceiver::mpsc(rx),
                    }
                }
                None => ExtensionLifecycle::Passive,
            };
            wrappers.push(ExtensionWrapper {
                name: self.name.clone(),
                user_config: self.user_config.clone(),
                runtime_config: self.runtime_config.clone(),
                lifecycle,
                telemetry: None,
            });
        }

        assert!(
            !wrappers.is_empty(),
            "ExtensionWrapper must have at least one variant (local or shared)"
        );

        for w in &wrappers {
            otel_debug!(
                "extension.builder.build",
                name = w.name.as_ref(),
                lifecycle = match &w.lifecycle {
                    ExtensionLifecycle::Passive => "passive",
                    ExtensionLifecycle::LocalActive { .. } => "local_active",
                    ExtensionLifecycle::SharedActive { .. } => "shared_active",
                },
            );
        }

        wrappers
    }
}

impl ExtensionWrapper {
    /// Start building an `ExtensionWrapper`.
    #[must_use]
    pub fn builder(
        name: ExtensionName,
        user_config: Arc<ExtensionUserConfig>,
        config: &ExtensionConfig,
    ) -> ExtensionWrapperBuilder {
        ExtensionWrapperBuilder {
            name,
            user_config,
            runtime_config: config.clone(),
            shared: None,
            local: None,
        }
    }

    /// Returns the name of this extension.
    #[must_use]
    pub fn name(&self) -> ExtensionName {
        self.name.clone()
    }

    /// Returns the user configuration for this extension.
    #[must_use]
    pub fn user_config(&self) -> Arc<ExtensionUserConfig> {
        self.user_config.clone()
    }

    /// Returns `true` if this extension is passive (no active lifecycle).
    #[must_use]
    pub fn is_passive(&self) -> bool {
        matches!(self.lifecycle, ExtensionLifecycle::Passive)
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
        let capacity = self.runtime_config.control_channel.capacity as u64;
        let name = self.name.as_ref();

        self.lifecycle = match self.lifecycle {
            ExtensionLifecycle::Passive => ExtensionLifecycle::Passive,
            ExtensionLifecycle::LocalActive {
                extension,
                control_sender,
                control_receiver,
            } => {
                let (s, r) = wrap_control_channel_metrics::<SharedMode, ExtensionControlMsg>(
                    name,
                    pipeline_ctx,
                    channel_metrics,
                    channel_metrics_enabled,
                    capacity,
                    control_sender,
                    control_receiver,
                );
                ExtensionLifecycle::LocalActive {
                    extension,
                    control_sender: s,
                    control_receiver: r,
                }
            }
            ExtensionLifecycle::SharedActive {
                extension,
                control_sender,
                control_receiver,
            } => {
                let (s, r) = wrap_control_channel_metrics::<SharedMode, ExtensionControlMsg>(
                    name,
                    pipeline_ctx,
                    channel_metrics,
                    channel_metrics_enabled,
                    capacity,
                    control_sender,
                    control_receiver,
                );
                ExtensionLifecycle::SharedActive {
                    extension,
                    control_sender: s,
                    control_receiver: r,
                }
            }
        };
        self
    }

    /// Returns `ExtensionControlSender`(s) for sending control messages.
    #[allow(dead_code)]
    pub(crate) fn extension_control_senders(&self) -> Vec<crate::control::ExtensionControlSender> {
        match &self.lifecycle {
            ExtensionLifecycle::Passive => Vec::new(),
            ExtensionLifecycle::LocalActive { control_sender, .. }
            | ExtensionLifecycle::SharedActive { control_sender, .. } => {
                vec![crate::control::ExtensionControlSender {
                    name: self.name.clone(),
                    sender: crate::message::Sender::Shared(control_sender.clone()),
                }]
            }
        }
    }

    /// Starts the extension lifecycle.
    pub async fn start(self, metrics_reporter: MetricsReporter) -> Result<TerminalState, Error> {
        let ext_name = self.name.clone();
        let effect_handler = EffectHandler::new(self.name, metrics_reporter);

        match self.lifecycle {
            ExtensionLifecycle::Passive => {
                panic!("start() called on passive extension — this is a bug")
            }
            ExtensionLifecycle::LocalActive {
                extension,
                control_receiver,
                ..
            } => {
                otel_debug!("extension.start.local", name = ext_name.as_ref());
                extension
                    .start(ControlChannel::new(control_receiver), effect_handler)
                    .await
            }
            ExtensionLifecycle::SharedActive {
                extension,
                control_receiver,
                ..
            } => {
                otel_debug!("extension.start.shared", name = ext_name.as_ref());
                extension
                    .start(ControlChannel::new(control_receiver), effect_handler)
                    .await
            }
        }
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

    fn test_metrics_reporter() -> MetricsReporter {
        let (tx, _rx) = flume::bounded(1);
        MetricsReporter::new(tx)
    }

    fn ext_config(
        name: &'static str,
    ) -> (ExtensionName, Arc<ExtensionUserConfig>, ExtensionConfig) {
        (
            name.into(),
            Arc::new(ExtensionUserConfig::new(
                "urn:otap:extension:test".into(),
                Value::Null,
            )),
            ExtensionConfig::new(name),
        )
    }

    #[derive(Clone)]
    struct TestExt {
        counter: CtrlMsgCounters,
    }
    impl TestExt {
        fn new(c: CtrlMsgCounters) -> Self {
            Self { counter: c }
        }
    }

    #[async_trait]
    impl SharedExtension for TestExt {
        async fn start(
            self: Box<Self>,
            mut ctrl: ControlChannel,
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

    #[async_trait(?Send)]
    impl crate::local::extension::Extension for TestExt {
        async fn start(
            self: std::rc::Rc<Self>,
            mut ctrl: ControlChannel,
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
        let w = ExtensionWrapper::builder(n, u, &c)
            .with_shared(Active(TestExt::new(CtrlMsgCounters::new())))
            .build()
            .pop()
            .unwrap();
        assert!(!w.is_passive());
        assert_eq!(w.extension_control_senders().len(), 1);
    }

    #[test]
    fn test_shared_passive() {
        let (n, u, c) = ext_config("sp");
        let w = ExtensionWrapper::builder(n, u, &c)
            .with_shared(Passive("data".to_string()))
            .build()
            .pop()
            .unwrap();
        assert!(w.is_passive());
        assert!(w.extension_control_senders().is_empty());
    }

    #[test]
    fn test_local_active() {
        let (n, u, c) = ext_config("la");
        let w = ExtensionWrapper::builder(n, u, &c)
            .with_local(Active(std::rc::Rc::new(TestExt::new(
                CtrlMsgCounters::new(),
            ))))
            .build()
            .pop()
            .unwrap();
        assert!(!w.is_passive());
        assert_eq!(w.extension_control_senders().len(), 1);
    }

    #[test]
    fn test_local_passive() {
        let (n, u, c) = ext_config("lp");
        let w = ExtensionWrapper::builder(n, u, &c)
            .with_local(Passive(std::rc::Rc::new(42u32)))
            .build()
            .pop()
            .unwrap();
        assert!(w.is_passive());
    }

    #[test]
    #[should_panic(expected = "at least one variant")]
    fn test_empty_panics() {
        let (n, u, c) = ext_config("e");
        let _ = ExtensionWrapper::builder(n, u, &c).build().pop().unwrap();
    }

    #[test]
    fn test_dual_creates_two_wrappers() {
        let e = TestExt::new(CtrlMsgCounters::new());
        let (n, u, c) = ext_config("d");
        let wrappers = ExtensionWrapper::builder(n, u, &c)
            .with_local(Active(std::rc::Rc::new(e.clone())))
            .with_shared(Active(e))
            .build();
        assert_eq!(wrappers.len(), 2);
        // First is local, second is shared — both active
        assert!(!wrappers[0].is_passive());
        assert!(!wrappers[1].is_passive());
        assert_eq!(wrappers[0].extension_control_senders().len(), 1);
        assert_eq!(wrappers[1].extension_control_senders().len(), 1);
    }

    #[test]
    fn test_name_and_config_accessors() {
        let (n, u, c) = ext_config("acc");
        let w = ExtensionWrapper::builder(n, u, &c)
            .with_shared(Active(TestExt::new(CtrlMsgCounters::new())))
            .build()
            .pop()
            .unwrap();
        assert_eq!(w.name().as_ref(), "acc");
        assert_eq!(w.user_config().r#type.as_ref(), "urn:otap:extension:test");
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
                .with_shared(Active(TestExt::new(ctr.clone())))
                .build()
                .pop()
                .unwrap();
            let s = w.extension_control_senders();
            let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
            s[0].send(ExtensionControlMsg::Config {
                config: Value::Null,
            })
            .await
            .unwrap();
            s[0].send(ExtensionControlMsg::Shutdown {
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
                .with_local(Active(std::rc::Rc::new(TestExt::new(ctr.clone()))))
                .build()
                .pop()
                .unwrap();
            let s = w.extension_control_senders();
            let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
            s[0].send(ExtensionControlMsg::Shutdown {
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
        let mut ch = ControlChannel::new(SharedReceiver::mpsc(rx));
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
        let mut ch = ControlChannel::new(SharedReceiver::mpsc(rx));
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
        let mut ch = ControlChannel::new(SharedReceiver::mpsc(rx));
        let dl = Instant::now() + std::time::Duration::from_millis(50);
        SharedSender::mpsc(tx)
            .send(ExtensionControlMsg::Shutdown {
                deadline: dl,
                reason: "dl".into(),
            })
            .await
            .unwrap();
        assert!(ch.recv().await.unwrap().is_shutdown());
        assert!(Instant::now() >= dl);
    }

    #[tokio::test]
    async fn test_ctrl_collect_telemetry() {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let mut ch = ControlChannel::new(SharedReceiver::mpsc(rx));
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
        let mut ch = ControlChannel::new(SharedReceiver::mpsc(rx));
        drop(tx);
        assert!(ch.recv().await.is_err());
    }

    #[tokio::test]
    async fn test_ctrl_sender_send() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let sender = crate::control::ExtensionControlSender {
            name: "st".into(),
            sender: crate::message::Sender::Shared(SharedSender::mpsc(tx)),
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

    #[tokio::test]
    async fn test_effect_handler_info() {
        EffectHandler::new("ehi".into(), test_metrics_reporter())
            .info("msg")
            .await;
    }

    #[test]
    fn test_passive_ctrl_metrics_noop() {
        let (n, u, c) = ext_config("pm");
        let w = ExtensionWrapper::builder(n, u, &c)
            .with_shared(Passive("d".to_string()))
            .build()
            .pop()
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
                .with_shared(Active(TestExt::new(ctr.clone())))
                .build()
                .pop()
                .unwrap();
            let s = w.extension_control_senders();
            let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
            s[0].send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: test_metrics_reporter(),
            })
            .await
            .unwrap();
            s[0].send(ExtensionControlMsg::Config {
                config: Value::Null,
            })
            .await
            .unwrap();
            s[0].send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "d".into(),
            })
            .await
            .unwrap();
            assert!(h.await.unwrap().is_ok());
            ctr.assert(0, 0, 1, 1);
        }));
    }
}
