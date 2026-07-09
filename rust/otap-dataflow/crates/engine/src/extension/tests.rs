// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the extension module (wrapper + builder + control channel).

use super::wrapper::{EffectHandler, ExtensionWrapper};
use crate::channel_metrics::ChannelMetricsRegistry;
use crate::config::ExtensionConfig;
use crate::control::ExtensionControlMsg;
use crate::error::Error;
use crate::local::extension as local_ext;
use crate::message::Sender;
use crate::shared::extension as shared_ext;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::terminal_state::TerminalState;
use crate::testing::CtrlMsgCounters;
use async_trait::async_trait;
use otap_df_config::ExtensionId;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_telemetry::reporter::MetricsReporter;
use serde_json::Value;
use shared_ext::Extension as SharedExtension;
use std::sync::Arc;
use std::time::Instant;

fn test_metrics_reporter() -> MetricsReporter {
    let (tx, _rx) = flume::bounded(1);
    MetricsReporter::new(tx)
}

/// Returns a oneshot receiver that will never receive a shutdown
/// signal: the matching sender is dropped immediately, so the
/// receiver resolves to `Err(_)` and the `ControlChannel` falls
/// through to the regular control path. Used by control-channel
/// unit tests that don't care about the dedicated shutdown channel.
fn never_firing_shutdown_rx() -> tokio::sync::oneshot::Receiver<crate::control::ShutdownPayload> {
    tokio::sync::oneshot::channel().1
}

fn ext_config(name: &'static str) -> (ExtensionId, Arc<ExtensionUserConfig>, ExtensionConfig) {
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
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    assert!(set.local().is_none());
    let w = set.take_shared().unwrap();
    assert!(!w.is_passive());
    assert!(w.extension_control_sender().is_some());
}

#[test]
fn test_shared_passive() {
    let (n, u, c) = ext_config("sp");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .cloned()
        .shared("data".to_string())
        .build()
        .unwrap();
    assert!(set.local().is_none());
    let w = set.take_shared().unwrap();
    assert!(w.is_passive());
    assert!(w.extension_control_sender().is_none());
}

#[test]
fn test_local_active() {
    let (n, u, c) = ext_config("la");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .build()
        .unwrap();
    assert!(set.shared().is_none());
    let w = set.take_local().unwrap();
    assert!(!w.is_passive());
    assert!(w.extension_control_sender().is_some());
}

#[test]
fn test_local_passive() {
    let (n, u, c) = ext_config("lp");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .cloned()
        .local(42u32)
        .build()
        .unwrap();
    assert!(set.shared().is_none());
    let w = set.take_local().unwrap();
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
        .active()
        .local(std::rc::Rc::new(DualExt))
        .shared(DualExt)
        .build();
    assert!(result.is_err());
}

#[test]
fn test_dual_creates_local_and_shared() {
    let (n, u, c) = ext_config("d");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let local = set.take_local().unwrap();
    let shared = set.take_shared().unwrap();
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
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let w = set.take_shared().unwrap();
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
            .active()
            .shared(TestSharedExt::new(ctr.clone()))
            .build()
            .unwrap()
            .take_shared()
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
            .active()
            .local(std::rc::Rc::new(TestLocalExt::new(ctr.clone())))
            .build()
            .unwrap()
            .take_local()
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
    let (_tx, rx) = tokio::sync::mpsc::channel::<ExtensionControlMsg>(8);
    let (shutdown_tx, shutdown_rx) =
        tokio::sync::oneshot::channel::<crate::control::ShutdownPayload>();
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx), shutdown_rx);
    shutdown_tx
        .send(crate::control::ShutdownPayload {
            deadline: Instant::now(),
            reason: "i".into(),
        })
        .expect("dedicated shutdown sender alive");
    assert!(ch.recv().await.unwrap().is_shutdown());
    assert!(ch.recv().await.is_err());
}

#[tokio::test]
async fn test_ctrl_config_then_shutdown() {
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let s = SharedSender::mpsc(tx);
    let (shutdown_tx, shutdown_rx) =
        tokio::sync::oneshot::channel::<crate::control::ShutdownPayload>();
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx), shutdown_rx);
    s.send(ExtensionControlMsg::Config {
        config: Value::String("h".into()),
    })
    .await
    .unwrap();
    assert!(!ch.recv().await.unwrap().is_shutdown());
    shutdown_tx
        .send(crate::control::ShutdownPayload {
            deadline: Instant::now(),
            reason: "d".into(),
        })
        .expect("dedicated shutdown sender alive");
    assert!(ch.recv().await.unwrap().is_shutdown());
    assert!(ch.recv().await.is_err());
}

#[tokio::test]
async fn test_ctrl_delayed_shutdown() {
    // Shutdown is delivered immediately regardless of how far in the
    // future its deadline is — the deadline is the extension's
    // cooperative cleanup budget, not a wait time before delivery.
    let (_tx, rx) = tokio::sync::mpsc::channel::<ExtensionControlMsg>(8);
    let (shutdown_tx, shutdown_rx) =
        tokio::sync::oneshot::channel::<crate::control::ShutdownPayload>();
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx), shutdown_rx);
    let dl = Instant::now() + std::time::Duration::from_secs(60);
    shutdown_tx
        .send(crate::control::ShutdownPayload {
            deadline: dl,
            reason: "dl".into(),
        })
        .expect("dedicated shutdown sender alive");
    let before = Instant::now();
    assert!(ch.recv().await.unwrap().is_shutdown());
    assert!(
        before.elapsed() < std::time::Duration::from_secs(1),
        "Shutdown should be delivered immediately, not after the deadline"
    );
    // After Shutdown the channel is closed.
    assert!(ch.recv().await.is_err());
}

#[tokio::test]
async fn test_ctrl_shutdown_closes_channel() {
    // Messages sent after Shutdown are never observed: once Shutdown
    // is delivered the channel is closed and the extension's event
    // loop terminates.
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let (shutdown_tx, shutdown_rx) =
        tokio::sync::oneshot::channel::<crate::control::ShutdownPayload>();
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx), shutdown_rx);
    let s = SharedSender::mpsc(tx);
    shutdown_tx
        .send(crate::control::ShutdownPayload {
            deadline: Instant::now() + std::time::Duration::from_secs(1),
            reason: "d".into(),
        })
        .expect("dedicated shutdown sender alive");
    s.send(ExtensionControlMsg::Config {
        config: Value::Null,
    })
    .await
    .unwrap();
    assert!(ch.recv().await.unwrap().is_shutdown());
    assert!(ch.recv().await.is_err());
}

#[tokio::test]
async fn test_ctrl_collect_telemetry() {
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let mut ch =
        shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx), never_firing_shutdown_rx());
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
    let mut ch =
        shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx), never_firing_shutdown_rx());
    drop(tx);
    assert!(ch.recv().await.is_err());
}

#[tokio::test]
async fn test_ctrl_sender_send() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(8);
    let sender = crate::control::ExtensionControlSender {
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
    let h = EffectHandler::new("eh".into(), test_metrics_reporter(), None);
    assert_eq!(h.extension_id().as_ref(), "eh");
    let c = h.clone();
    assert_eq!(c.extension_id().as_ref(), "eh");
}

#[test]
fn test_passive_ctrl_metrics_noop() {
    let (n, u, c) = ext_config("pm");
    let w = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .cloned()
        .shared("d".to_string())
        .build()
        .unwrap()
        .take_shared()
        .unwrap();
    let (ctx, _) = crate::testing::test_extension_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let key = ctx.register_extension_entity(
        "pm".into(),
        crate::extension::wrapper::ExtensionVariant::Shared,
    );
    let handle = crate::entity_context::EntityTelemetryHandle::new(ctx.metrics_registry(), key);
    let w = w.with_control_channel_metrics(&handle, &ctx, &mut cm, true);
    assert!(w.is_passive());
}

#[test]
fn test_start_with_telemetry() {
    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let ctr = CtrlMsgCounters::new();
        let (n, u, c) = ext_config("st");
        let w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .shared(TestSharedExt::new(ctr.clone()))
            .build()
            .unwrap()
            .take_shared()
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
        .active()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap()
        .take_shared()
        .unwrap();

    let (ctx, _) = crate::testing::test_extension_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let key = ctx.register_extension_entity(
        "acm".into(),
        crate::extension::wrapper::ExtensionVariant::Shared,
    );
    let handle = crate::entity_context::EntityTelemetryHandle::new(ctx.metrics_registry(), key);
    let w = w.with_control_channel_metrics(&handle, &ctx, &mut cm, true);
    assert!(!w.is_passive());
    assert!(w.extension_control_sender().is_some());
}

#[test]
fn test_dual_passive() {
    let (n, u, c) = ext_config("dp");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .cloned()
        .local(42u32)
        .shared("data".to_string())
        .build()
        .unwrap();
    assert!(set.take_local().unwrap().is_passive());
    assert!(set.take_shared().unwrap().is_passive());
}

#[test]
fn test_dual_passive_fresh() {
    let (n, u, c) = ext_config("dpf");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .constructed()
        .local(|| 42u32)
        .shared(|| "data".to_string())
        .build()
        .unwrap();
    assert!(set.take_local().unwrap().is_passive());
    assert!(set.take_shared().unwrap().is_passive());
}

#[test]
fn test_extension_set_iter() {
    let (n, u, c) = ext_config("it");
    let set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    assert_eq!(set.iter().count(), 2);
}

// ── Background lifecycle tests ───────────────────────────────────────────────

#[test]
fn test_background_shared() {
    // `.background().shared(...).build()` produces a bundle with a single
    // shared variant whose runtime lifecycle is Active (engine-driven event
    // loop). Authors will pair this with `capabilities: None` on the factory.
    let (n, u, c) = ext_config("bg-shared");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .background()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    assert!(set.local().is_none());
    let w = set.take_shared().unwrap();
    // Background reuses the Active event-loop machinery.
    assert!(!w.is_passive());
    assert!(w.extension_control_sender().is_some());
}

#[test]
fn test_background_local() {
    let (n, u, c) = ext_config("bg-local");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .background()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .build()
        .unwrap();
    assert!(set.shared().is_none());
    let w = set.take_local().unwrap();
    assert!(!w.is_passive());
    assert!(w.extension_control_sender().is_some());
}

#[test]
fn test_background_factory_capabilities_none() {
    // Background bundles call `register_into(None, …)` → no-op,
    // contributing zero entries to the registry.
    use crate::capability::registry::CapabilityRegistry;

    let (n, u, c) = ext_config("bg-noreg");
    let bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();

    let mut registry = CapabilityRegistry::new();
    bundle
        .register_into(None, &mut registry)
        .expect("Background register_into is a no-op");
    let _ = registry;
}

// Compile-time invariants enforced by typestate (illustrative; not run):
//
// ```compile_fail
// // Cannot register two variants on `.background()`.
// let (n, u, c) = ext_config("bg");
// let _ = ExtensionWrapper::builder(n, u, &c)
//     .background()
//     .shared(TestSharedExt::new(CtrlMsgCounters::new()))
//     .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
//     .build();
// ```
//
// ```compile_fail
// // Cannot `.build()` a Background bundle without a registration.
// let (n, u, c) = ext_config("bg");
// let _ = ExtensionWrapper::builder(n, u, &c).background().build();
// ```

// ── Extension telemetry wiring tests ─────────────────────────────────────────

#[test]
fn test_extension_attribute_set_carries_pipeline_scope() {
    use crate::attributes::{
        ExtensionAttributeSet, ExtensionScopeAttributeSet, PipelineAttributeSet,
    };
    use otap_df_telemetry::attributes::AttributeSetHandler;

    let attrs = ExtensionAttributeSet {
        extension_id: "ext1".into(),
        extension_variant: "local".into(),
        extension_scope: ExtensionScopeAttributeSet::pipeline(PipelineAttributeSet {
            pipeline_group_id: "grp".into(),
            pipeline_id: "pipeline_a".into(),
            ..PipelineAttributeSet::default()
        }),
    };
    assert_eq!(attrs.schema_name(), "extension.attrs");
    let attr_map: std::collections::HashMap<&'static str, String> = attrs
        .iter_attributes()
        .map(|(k, v)| (k, v.to_string_value()))
        .collect();
    assert!(attr_map.contains_key("extension.id"));
    assert!(attr_map.contains_key("extension.variant"));
    assert_eq!(
        attr_map.get("scope.kind").map(String::as_str),
        Some("pipeline"),
        "extension.attrs must carry scope.kind; got: {attr_map:?}"
    );
    assert_eq!(
        attr_map.get("pipeline.group.id").map(String::as_str),
        Some("grp"),
        "extension.attrs must carry the composed pipeline.group.id; got: {attr_map:?}"
    );
    assert_eq!(
        attr_map.get("pipeline.id").map(String::as_str),
        Some("pipeline_a"),
        "extension.attrs must carry the composed pipeline.id; got: {attr_map:?}"
    );
    assert!(
        !attr_map.contains_key("scope.id"),
        "scope.id was replaced by typed composed pipeline attributes; got: {attr_map:?}"
    );
}

#[test]
fn test_extension_channel_attribute_set_shape() {
    use crate::attributes::{
        ExtensionAttributeSet, ExtensionChannelAttributeSet, ExtensionScopeAttributeSet,
        PipelineAttributeSet,
    };
    use otap_df_telemetry::attributes::AttributeSetHandler;

    let attrs = ExtensionChannelAttributeSet {
        channel_id: "ext1:control".into(),
        extension_attrs: ExtensionAttributeSet {
            extension_id: "ext1".into(),
            extension_variant: "shared".into(),
            extension_scope: ExtensionScopeAttributeSet::pipeline(PipelineAttributeSet {
                pipeline_group_id: "grp".into(),
                pipeline_id: "pipeline_a".into(),
                ..PipelineAttributeSet::default()
            }),
        },
        channel_mode: "local".into(),
        channel_impl: "internal".into(),
    };
    assert_eq!(attrs.schema_name(), "extension.channel.attrs");
    let keys: Vec<&'static str> = attrs.iter_attributes().map(|(k, _)| k).collect();
    // Invariants (channel.kind, channel.type) and node-only fields (node.port)
    // must not be present.
    assert!(keys.contains(&"channel.id"));
    assert!(keys.contains(&"extension.id"));
    assert!(keys.contains(&"channel.mode"));
    assert!(keys.contains(&"channel.impl"));
    assert!(!keys.contains(&"channel.kind"));
    assert!(!keys.contains(&"channel.type"));
    assert!(!keys.contains(&"node.port"));
    assert!(keys.contains(&"scope.kind"));
    assert!(keys.contains(&"pipeline.id"));
    assert!(keys.contains(&"pipeline.group.id"));
    assert!(
        !keys.contains(&"scope.id"),
        "scope.id was replaced by typed composed pipeline attributes; got keys: {keys:?}"
    );
}

#[test]
fn test_register_extension_entity_unregister_roundtrip() {
    let (ctx, registry) = crate::testing::test_extension_ctx();
    let before = registry.entity_count();
    let key = ctx.register_extension_entity(
        "ext1".into(),
        crate::extension::wrapper::ExtensionVariant::Local,
    );
    assert_eq!(registry.entity_count(), before + 1);
    let schema = registry
        .visit_entity(key, |attrs| attrs.schema_name())
        .expect("entity should exist");
    assert_eq!(schema, "extension.attrs");
    assert!(registry.unregister_entity(key));
    assert_eq!(registry.entity_count(), before);
}

#[test]
fn test_register_extension_channel_entity_carries_extension_scope() {
    let (ctx, registry) = crate::testing::test_extension_ctx();
    let key = ctx.register_extension_channel_entity(
        "ext1".into(),
        crate::extension::wrapper::ExtensionVariant::Shared,
        "ext1:control".into(),
        "shared",
        "tokio",
    );
    let schema = registry
        .visit_entity(key, |attrs| attrs.schema_name())
        .expect("entity should exist");
    assert_eq!(schema, "extension.channel.attrs");
    assert!(registry.unregister_entity(key));
}

#[test]
fn test_wire_telemetry_passive_registers_only_extension_entity() {
    let (n, u, c) = ext_config("pwt");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .cloned()
        .shared("d".to_string())
        .build()
        .unwrap();

    let (ctx, registry) = crate::testing::test_extension_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let before = registry.entity_count();

    let _ = bundle.wire_telemetry("pwt".into(), &ctx, &mut cm, true);

    // Passive => no control channel entity registered. Only the extension entity.
    assert_eq!(registry.entity_count(), before + 1);

    // Dropping the bundle drops the guard, which cleans up the extension entity.
    drop(bundle);
    assert_eq!(registry.entity_count(), before);
}

#[test]
fn test_wire_telemetry_active_registers_extension_and_control_channel() {
    let (n, u, c) = ext_config("awt");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();

    let (ctx, registry) = crate::testing::test_extension_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let before = registry.entity_count();

    let _ = bundle.wire_telemetry("awt".into(), &ctx, &mut cm, true);

    // Active shared => extension entity + 1 control-channel entity.
    assert_eq!(registry.entity_count(), before + 2);

    drop(bundle);
    assert_eq!(registry.entity_count(), before);
}

#[test]
fn test_wire_telemetry_dual_active_registers_both_control_channels() {
    let (n, u, c) = ext_config("dwt");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();

    let (ctx, registry) = crate::testing::test_extension_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let before = registry.entity_count();

    let _ = bundle.wire_telemetry("dwt".into(), &ctx, &mut cm, true);

    // Dual active => 2 extension entities (local + shared) + 2 control-channel entities.
    assert_eq!(registry.entity_count(), before + 4);

    drop(bundle);
    assert_eq!(registry.entity_count(), before);
}

#[test]
fn test_wire_telemetry_disabled_still_registers_channel_entity() {
    // With metrics disabled the channel sender/receiver wrappers are not
    // attached, but the channel entity is still registered so that the
    // identity exists in telemetry output.
    let (n, u, c) = ext_config("dis");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();

    let (ctx, registry) = crate::testing::test_extension_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let before = registry.entity_count();

    let _ = bundle.wire_telemetry("dis".into(), &ctx, &mut cm, false);

    assert_eq!(registry.entity_count(), before + 2);

    drop(bundle);
    assert_eq!(registry.entity_count(), before);
}

// When the host's monitor dispatches `CollectTelemetry { metrics_reporter }`,
// the extension must publish its internal metrics via the supplied reporter.
#[otap_df_telemetry_macros::metric_set(name = "test.extension.internal")]
#[derive(Debug, Default, Clone)]
struct TestExtensionInternalMetrics {
    #[metric(unit = "{tick}")]
    collect_invocations: otap_df_telemetry::instrument::Counter<u64>,
}

struct TelemetryReportingLocalExt {
    metrics:
        std::cell::RefCell<otap_df_telemetry::metrics::MetricSet<TestExtensionInternalMetrics>>,
    started: std::cell::Cell<bool>,
}

// `Clone` is required by `ActiveStage::local`. Clones share state via
// interior mutability — only one live instance ever runs.
impl Clone for TelemetryReportingLocalExt {
    fn clone(&self) -> Self {
        Self {
            metrics: std::cell::RefCell::new(self.metrics.borrow().clone()),
            started: std::cell::Cell::new(self.started.get()),
        }
    }
}

#[async_trait(?Send)]
impl crate::local::extension::Extension for TelemetryReportingLocalExt {
    async fn start(
        self: std::rc::Rc<Self>,
        mut ctrl: local_ext::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, Error> {
        self.started.set(true);
        loop {
            match ctrl.recv().await? {
                ExtensionControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    let mut m = self.metrics.borrow_mut();
                    m.collect_invocations.add(1);
                    let _ = metrics_reporter.report(&mut *m);
                }
                ExtensionControlMsg::Shutdown { .. } => break,
                _ => {}
            }
        }
        Ok(TerminalState::default())
    }
}

#[test]
fn active_extension_reports_internal_metrics_on_collect_telemetry() {
    use otap_df_telemetry::reporter::MetricsReporter;

    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let (ctx, _registry) = crate::testing::test_extension_ctx();

        // Pre-register the extension's internal metric set through the
        // host context, the same way real extensions would do at setup.
        let entity = ctx.register_extension_entity(
            "telemetry_ext".into(),
            crate::extension::wrapper::ExtensionVariant::Local,
        );
        let internal_metrics =
            ctx.register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity);

        let ext = std::rc::Rc::new(TelemetryReportingLocalExt {
            metrics: std::cell::RefCell::new(internal_metrics),
            started: std::cell::Cell::new(false),
        });
        let started_probe = ext.clone();

        let (n, u, c) = ext_config("telemetry_ext");
        let w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .local(ext)
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let sender = w.extension_control_sender().unwrap();

        let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });

        // A test-owned reporter so we can observe the snapshot the
        // extension publishes.
        let (snapshot_rx, ext_reporter) = MetricsReporter::create_new_and_receiver(4);

        sender
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: ext_reporter,
            })
            .await
            .unwrap();

        // Wait for the snapshot via async recv (works on a LocalSet
        // because the channel is flume which exposes recv_async).
        let snapshot =
            tokio::time::timeout(std::time::Duration::from_secs(2), snapshot_rx.recv_async())
                .await
                .expect("extension did not publish a snapshot in time")
                .expect("snapshot channel closed without delivery");

        assert!(
            started_probe.started.get(),
            "extension start() must have run before reporting"
        );
        let value_sum: u64 = snapshot
            .get_metrics()
            .iter()
            .map(|v| match v {
                otap_df_telemetry::metrics::MetricValue::U64(n) => *n,
                _ => 0,
            })
            .sum();
        assert!(
            value_sum >= 1,
            "expected the collect_invocations counter to be reported, got snapshot: {:?}",
            snapshot.get_metrics()
        );

        sender
            .send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "done".into(),
            })
            .await
            .unwrap();
        assert!(h.await.unwrap().is_ok());
    }));
}

#[test]
fn two_active_extensions_report_isolated_internal_metrics() {
    // Two active extensions sharing one MetricsReporter must publish
    // under distinct `MetricSetKey`s with no value bleed.
    use otap_df_telemetry::reporter::MetricsReporter;

    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let (ctx, _registry) = crate::testing::test_extension_ctx();

        // Each extension gets its own entity AND its own MetricSet,
        // mirroring how `ExtensionMetricsMonitor::register` wires them.
        let entity_a = ctx.register_extension_entity(
            "ext_a".into(),
            crate::extension::wrapper::ExtensionVariant::Local,
        );
        let metrics_a =
            ctx.register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity_a);
        let key_a = metrics_a.metric_set_key();

        let entity_b = ctx.register_extension_entity(
            "ext_b".into(),
            crate::extension::wrapper::ExtensionVariant::Local,
        );
        let metrics_b =
            ctx.register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity_b);
        let key_b = metrics_b.metric_set_key();

        assert_ne!(
            key_a, key_b,
            "two distinct entities must yield distinct metric-set keys"
        );

        let ext_a = std::rc::Rc::new(TelemetryReportingLocalExt {
            metrics: std::cell::RefCell::new(metrics_a),
            started: std::cell::Cell::new(false),
        });
        let ext_b = std::rc::Rc::new(TelemetryReportingLocalExt {
            metrics: std::cell::RefCell::new(metrics_b),
            started: std::cell::Cell::new(false),
        });

        let (n_a, u_a, c_a) = ext_config("ext_a");
        let w_a = ExtensionWrapper::builder(n_a, u_a, &c_a)
            .active()
            .local(ext_a)
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let sender_a = w_a.extension_control_sender().unwrap();

        let (n_b, u_b, c_b) = ext_config("ext_b");
        let w_b = ExtensionWrapper::builder(n_b, u_b, &c_b)
            .active()
            .local(ext_b)
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let sender_b = w_b.extension_control_sender().unwrap();

        let h_a = tokio::task::spawn_local(async move { w_a.start(test_metrics_reporter()).await });
        let h_b = tokio::task::spawn_local(async move { w_b.start(test_metrics_reporter()).await });

        // ONE shared reporter — both extensions publish through the same
        // channel. Isolation must be enforced by per-MetricSet keys, not
        // by separate transports.
        let (snapshot_rx, shared_reporter) = MetricsReporter::create_new_and_receiver(8);

        // Drive extension A twice and extension B once so values differ.
        sender_a
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: shared_reporter.clone(),
            })
            .await
            .unwrap();
        sender_a
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: shared_reporter.clone(),
            })
            .await
            .unwrap();
        sender_b
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: shared_reporter.clone(),
            })
            .await
            .unwrap();

        // Collect snapshots until we have one with `key_a` and one with
        // `key_b`. Bounded loop guards against hangs while tolerating
        // arbitrary interleaving from the LocalSet scheduler.
        let mut sum_by_key: std::collections::HashMap<_, u64> = std::collections::HashMap::new();
        let deadline = Instant::now() + std::time::Duration::from_secs(2);
        while (!sum_by_key.contains_key(&key_a) || !sum_by_key.contains_key(&key_b))
            && Instant::now() < deadline
        {
            match tokio::time::timeout(
                std::time::Duration::from_millis(200),
                snapshot_rx.recv_async(),
            )
            .await
            {
                Ok(Ok(snap)) => {
                    let k = snap.key();
                    let v: u64 = snap
                        .get_metrics()
                        .iter()
                        .map(|m| match m {
                            otap_df_telemetry::metrics::MetricValue::U64(n) => *n,
                            _ => 0,
                        })
                        .sum();
                    // Counter::add() reports deltas; sum across snapshots
                    // gives total invocations observed for that key.
                    *sum_by_key.entry(k).or_insert(0) += v;
                }
                Ok(Err(_)) | Err(_) => break,
            }
        }

        let a_total = *sum_by_key.get(&key_a).unwrap_or(&0);
        let b_total = *sum_by_key.get(&key_b).unwrap_or(&0);

        // Each extension's reported total equals its own invocation
        // count and no others.
        assert_eq!(a_total, 2, "ext_a should report exactly its 2 invocations");
        assert_eq!(b_total, 1, "ext_b should report exactly its 1 invocation");
        assert_eq!(
            sum_by_key.len(),
            2,
            "only the two registered metric-set keys should appear, got: {sum_by_key:?}"
        );

        sender_a
            .send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "done".into(),
            })
            .await
            .unwrap();
        sender_b
            .send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "done".into(),
            })
            .await
            .unwrap();
        assert!(h_a.await.unwrap().is_ok());
        assert!(h_b.await.unwrap().is_ok());
    }));
}

fn pipeline_ctx_in_controller(
    controller: &crate::context::ControllerContext,
    group: &'static str,
    pipeline: &'static str,
    core_id: usize,
    thread_id: usize,
) -> crate::context::PipelineContext {
    controller.pipeline_context_with(group.into(), pipeline.into(), core_id, 1, thread_id)
}

#[test]
fn extension_entity_isolated_across_pipelines() {
    use crate::extension::wrapper::ExtensionVariant;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    let registry = TelemetryRegistryHandle::new();
    let controller = crate::context::ControllerContext::new(registry.clone());

    let pipe_a = pipeline_ctx_in_controller(&controller, "grp", "pipeline_a", 0, 0);
    let pipe_b = pipeline_ctx_in_controller(&controller, "grp", "pipeline_b", 0, 1);

    let ext_a = pipe_a
        .extension_context()
        .register_extension_entity("same_ext".into(), ExtensionVariant::Local);
    let ext_b = pipe_b
        .extension_context()
        .register_extension_entity("same_ext".into(), ExtensionVariant::Local);

    assert_ne!(
        ext_a, ext_b,
        "two pipelines hosting an extension with the same id+variant must get distinct entity keys",
    );
}

#[test]
fn extension_entity_isolated_across_cores() {
    use crate::extension::wrapper::ExtensionVariant;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    let registry = TelemetryRegistryHandle::new();
    let controller = crate::context::ControllerContext::new(registry.clone());

    // Same pipeline replicated across two cores. Each per-core runtime
    // registers its own extension instances — they must not alias.
    let pipe_core0 = pipeline_ctx_in_controller(&controller, "grp", "pipeline_a", 0, 0);
    let pipe_core1 = pipeline_ctx_in_controller(&controller, "grp", "pipeline_a", 1, 1);

    let ext_core0 = pipe_core0
        .extension_context()
        .register_extension_entity("same_ext".into(), ExtensionVariant::Local);
    let ext_core1 = pipe_core1
        .extension_context()
        .register_extension_entity("same_ext".into(), ExtensionVariant::Local);

    assert_ne!(
        ext_core0, ext_core1,
        "same pipeline on two cores must register distinct extension entities",
    );
}

#[test]
fn extension_entity_isolated_across_pipeline_groups() {
    use crate::extension::wrapper::ExtensionVariant;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    let registry = TelemetryRegistryHandle::new();
    let controller = crate::context::ControllerContext::new(registry.clone());

    let pipe_grp_a = pipeline_ctx_in_controller(&controller, "grp_a", "pipeline_x", 0, 0);
    let pipe_grp_b = pipeline_ctx_in_controller(&controller, "grp_b", "pipeline_x", 0, 1);

    let ext_grp_a = pipe_grp_a
        .extension_context()
        .register_extension_entity("same_ext".into(), ExtensionVariant::Local);
    let ext_grp_b = pipe_grp_b
        .extension_context()
        .register_extension_entity("same_ext".into(), ExtensionVariant::Local);

    assert_ne!(
        ext_grp_a, ext_grp_b,
        "two pipeline groups must isolate their extensions even when the inner pipeline id matches",
    );
}

#[test]
fn extension_channel_entity_isolated_across_pipelines() {
    use crate::extension::wrapper::ExtensionVariant;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    let registry = TelemetryRegistryHandle::new();
    let controller = crate::context::ControllerContext::new(registry.clone());

    let pipe_a = pipeline_ctx_in_controller(&controller, "grp", "pipeline_a", 0, 0);
    let pipe_b = pipeline_ctx_in_controller(&controller, "grp", "pipeline_b", 0, 1);

    let chan_a = pipe_a
        .extension_context()
        .register_extension_channel_entity(
            "same_ext".into(),
            ExtensionVariant::Shared,
            "same_ext:control".into(),
            "shared",
            "tokio",
        );
    let chan_b = pipe_b
        .extension_context()
        .register_extension_channel_entity(
            "same_ext".into(),
            ExtensionVariant::Shared,
            "same_ext:control".into(),
            "shared",
            "tokio",
        );

    assert_ne!(
        chan_a, chan_b,
        "extension control-channel entities must be pipeline-scoped",
    );
}

#[test]
fn extension_attribute_set_carries_pipeline_scope_from_pipeline_context() {
    use crate::extension::wrapper::ExtensionVariant;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    let registry = TelemetryRegistryHandle::new();
    let controller = crate::context::ControllerContext::new(registry.clone());
    let pipe = pipeline_ctx_in_controller(&controller, "grp", "pipeline_a", 7, 0);
    let ext_ctx = pipe.extension_context();
    let attrs = ext_ctx.extension_attribute_set("ext1".into(), ExtensionVariant::Local);

    let key = registry.register_entity(attrs);

    let attr_map = registry
        .visit_entity(key, |a| {
            a.iter_attributes()
                .map(|(k, v)| (k.to_string(), v.to_string_value()))
                .collect::<std::collections::HashMap<_, _>>()
        })
        .expect("entity should exist");

    assert!(
        attr_map.contains_key("scope.kind"),
        "extension.attrs minted from a PipelineContext must carry scope.kind; got: {attr_map:?}"
    );
    assert_eq!(
        attr_map.get("scope.kind").map(String::as_str),
        Some("pipeline"),
        "pipeline-hosted extensions must report scope.kind = \"pipeline\""
    );
    assert_eq!(
        attr_map.get("pipeline.group.id").map(String::as_str),
        Some("grp"),
        "extension.attrs minted from a PipelineContext must carry the composed pipeline.group.id; got: {attr_map:?}"
    );
    assert_eq!(
        attr_map.get("pipeline.id").map(String::as_str),
        Some("pipeline_a"),
        "extension.attrs minted from a PipelineContext must carry the composed pipeline.id; got: {attr_map:?}"
    );
    assert_eq!(
        attr_map.get("core.id").map(String::as_str),
        Some("7"),
        "extension.attrs minted from a PipelineContext must carry the composed core.id; got: {attr_map:?}"
    );
    assert_eq!(
        attr_map.get("deployment.generation").map(String::as_str),
        Some("0"),
        "extension.attrs minted from a PipelineContext must carry the composed deployment.generation; got: {attr_map:?}"
    );
    assert!(
        !attr_map.contains_key("scope.id"),
        "scope.id was replaced by typed composed pipeline attributes; got: {attr_map:?}"
    );
    assert_eq!(
        attr_map.get("extension.id").map(String::as_str),
        Some("ext1")
    );
}

// An extension that panics inside its `CollectTelemetry` handler. Used to
// verify the host treats it as a terminal `JoinPanic` without leaking into
// neighbouring extensions' snapshots.
struct PanicOnCollectExt;

#[async_trait(?Send)]
impl crate::local::extension::Extension for PanicOnCollectExt {
    async fn start(
        self: std::rc::Rc<Self>,
        mut ctrl: local_ext::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, Error> {
        loop {
            match ctrl.recv().await? {
                ExtensionControlMsg::CollectTelemetry { .. } => {
                    panic!("simulated panic in CollectTelemetry handler");
                }
                ExtensionControlMsg::Shutdown { .. } => break,
                _ => {}
            }
        }
        Ok(TerminalState::default())
    }
}

impl Clone for PanicOnCollectExt {
    fn clone(&self) -> Self {
        PanicOnCollectExt
    }
}

#[test]
fn two_pipelines_publish_isolated_internal_metrics_through_shared_reporter() {
    // Live regression: two distinct pipelines (different ids, different
    // logical cores) each host an instance of the same extension type
    // and report through ONE shared `MetricsReporter`. Isolation must
    // be enforced by per-entity `MetricSetKey`s with no value bleed.
    use crate::extension::wrapper::ExtensionVariant;
    use otap_df_telemetry::reporter::MetricsReporter;

    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let registry = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller = crate::context::ControllerContext::new(registry.clone());

        // Two pipelines, distinct ids, distinct logical cores. These
        // are the same coordinates a real deployment would supply when
        // pinning pipelines to different cores.
        let pipe_a = controller.pipeline_context_with("grp".into(), "pipeline_a".into(), 0, 2, 0);
        let pipe_b = controller.pipeline_context_with("grp".into(), "pipeline_b".into(), 1, 2, 1);

        let ext_ctx_a = pipe_a.extension_context();
        let ext_ctx_b = pipe_b.extension_context();

        let entity_a =
            ext_ctx_a.register_extension_entity("same_ext".into(), ExtensionVariant::Local);
        let entity_b =
            ext_ctx_b.register_extension_entity("same_ext".into(), ExtensionVariant::Local);
        assert_ne!(
            entity_a, entity_b,
            "two pipelines on different cores must mint distinct entity keys"
        );

        let metrics_a =
            ext_ctx_a.register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity_a);
        let metrics_b =
            ext_ctx_b.register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity_b);
        let key_a = metrics_a.metric_set_key();
        let key_b = metrics_b.metric_set_key();
        assert_ne!(key_a, key_b);

        let ext_a = std::rc::Rc::new(TelemetryReportingLocalExt {
            metrics: std::cell::RefCell::new(metrics_a),
            started: std::cell::Cell::new(false),
        });
        let ext_b = std::rc::Rc::new(TelemetryReportingLocalExt {
            metrics: std::cell::RefCell::new(metrics_b),
            started: std::cell::Cell::new(false),
        });

        let (n_a, u_a, c_a) = ext_config("same_ext");
        let w_a = ExtensionWrapper::builder(n_a, u_a, &c_a)
            .active()
            .local(ext_a)
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let sender_a = w_a.extension_control_sender().unwrap();

        let (n_b, u_b, c_b) = ext_config("same_ext");
        let w_b = ExtensionWrapper::builder(n_b, u_b, &c_b)
            .active()
            .local(ext_b)
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let sender_b = w_b.extension_control_sender().unwrap();

        let h_a = tokio::task::spawn_local(async move { w_a.start(test_metrics_reporter()).await });
        let h_b = tokio::task::spawn_local(async move { w_b.start(test_metrics_reporter()).await });

        let (snapshot_rx, shared_reporter) = MetricsReporter::create_new_and_receiver(8);

        // 3 invocations to pipeline_a's extension, 1 to pipeline_b's.
        for _ in 0..3 {
            sender_a
                .send(ExtensionControlMsg::CollectTelemetry {
                    metrics_reporter: shared_reporter.clone(),
                })
                .await
                .unwrap();
        }
        sender_b
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: shared_reporter.clone(),
            })
            .await
            .unwrap();

        let mut sum_by_key: std::collections::HashMap<_, u64> = std::collections::HashMap::new();
        let deadline = Instant::now() + std::time::Duration::from_secs(2);
        while (!sum_by_key.contains_key(&key_a) || !sum_by_key.contains_key(&key_b))
            && Instant::now() < deadline
        {
            match tokio::time::timeout(
                std::time::Duration::from_millis(200),
                snapshot_rx.recv_async(),
            )
            .await
            {
                Ok(Ok(snap)) => {
                    let k = snap.key();
                    let v: u64 = snap
                        .get_metrics()
                        .iter()
                        .map(|m| match m {
                            otap_df_telemetry::metrics::MetricValue::U64(n) => *n,
                            _ => 0,
                        })
                        .sum();
                    *sum_by_key.entry(k).or_insert(0) += v;
                }
                Ok(Err(_)) | Err(_) => break,
            }
        }

        assert_eq!(
            *sum_by_key.get(&key_a).unwrap_or(&0),
            3,
            "pipeline_a should report exactly its 3 invocations"
        );
        assert_eq!(
            *sum_by_key.get(&key_b).unwrap_or(&0),
            1,
            "pipeline_b should report exactly its 1 invocation"
        );
        assert_eq!(
            sum_by_key.len(),
            2,
            "only the two registered metric-set keys should appear, got: {sum_by_key:?}"
        );

        sender_a
            .send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "done".into(),
            })
            .await
            .unwrap();
        sender_b
            .send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "done".into(),
            })
            .await
            .unwrap();
        assert!(h_a.await.unwrap().is_ok());
        assert!(h_b.await.unwrap().is_ok());
    }));
}

#[test]
fn panicking_collect_telemetry_handler_does_not_contaminate_neighbour() {
    // One extension panics while handling `CollectTelemetry`; the
    // neighbour must continue publishing its own snapshots through the
    // shared reporter without skew.
    use crate::extension::wrapper::ExtensionVariant;
    use otap_df_telemetry::reporter::MetricsReporter;

    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let (ctx, _registry) = crate::testing::test_extension_ctx();

        let entity_good = ctx.register_extension_entity("good".into(), ExtensionVariant::Local);
        let metrics_good =
            ctx.register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity_good);
        let key_good = metrics_good.metric_set_key();

        let ext_good = std::rc::Rc::new(TelemetryReportingLocalExt {
            metrics: std::cell::RefCell::new(metrics_good),
            started: std::cell::Cell::new(false),
        });
        let ext_bad = std::rc::Rc::new(PanicOnCollectExt);

        let (n_g, u_g, c_g) = ext_config("good");
        let w_good = ExtensionWrapper::builder(n_g, u_g, &c_g)
            .active()
            .local(ext_good)
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let sender_good = w_good.extension_control_sender().unwrap();

        let (n_b, u_b, c_b) = ext_config("bad");
        let w_bad = ExtensionWrapper::builder(n_b, u_b, &c_b)
            .active()
            .local(ext_bad)
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let sender_bad = w_bad.extension_control_sender().unwrap();

        let h_good =
            tokio::task::spawn_local(async move { w_good.start(test_metrics_reporter()).await });

        // Silence panic backtrace from the bad extension while we drive it.
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let h_bad =
            tokio::task::spawn_local(async move { w_bad.start(test_metrics_reporter()).await });

        let (snapshot_rx, shared_reporter) = MetricsReporter::create_new_and_receiver(8);

        // Push the bad one first, then drive the good one twice.
        sender_bad
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: shared_reporter.clone(),
            })
            .await
            .unwrap();
        sender_good
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: shared_reporter.clone(),
            })
            .await
            .unwrap();
        sender_good
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: shared_reporter.clone(),
            })
            .await
            .unwrap();

        // The bad extension's JoinHandle resolves with a panic-style JoinError.
        let bad_result = h_bad.await;
        std::panic::set_hook(prev_hook);
        let bad_panicked = bad_result
            .as_ref()
            .err()
            .map(|e| e.is_panic())
            .unwrap_or(false);
        assert!(
            bad_panicked,
            "bad extension should surface as JoinError(is_panic=true)"
        );

        // Collect snapshots until the good extension has reported its 2 invocations.
        let mut good_total = 0u64;
        let mut seen_keys = std::collections::HashSet::new();
        let deadline = Instant::now() + std::time::Duration::from_secs(2);
        while good_total < 2 && Instant::now() < deadline {
            match tokio::time::timeout(
                std::time::Duration::from_millis(200),
                snapshot_rx.recv_async(),
            )
            .await
            {
                Ok(Ok(snap)) => {
                    let _ = seen_keys.insert(snap.key());
                    if snap.key() == key_good {
                        good_total += snap
                            .get_metrics()
                            .iter()
                            .map(|m| match m {
                                otap_df_telemetry::metrics::MetricValue::U64(n) => *n,
                                _ => 0,
                            })
                            .sum::<u64>();
                    }
                }
                Ok(Err(_)) | Err(_) => break,
            }
        }

        assert_eq!(
            good_total, 2,
            "good extension must publish exactly its 2 invocations despite the neighbour's panic"
        );
        // The panicker registered no metric set, so it must not appear in snapshots.
        assert_eq!(
            seen_keys.len(),
            1,
            "only the good extension's metric set should appear; saw {:?}",
            seen_keys
        );

        // The good extension still shuts down cleanly.
        sender_good
            .send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "done".into(),
            })
            .await
            .unwrap();
        assert!(h_good.await.unwrap().is_ok());
    }));
}

#[test]
fn snapshot_consumed_after_extension_drop_retains_identity() {
    // An extension publishes a snapshot, then is fully torn down
    // (entity unregistered via Drop). The in-flight snapshot must
    // still be consumable with its key/values intact — snapshots are
    // value types, not references into the registry.
    use crate::extension::wrapper::ExtensionVariant;
    use otap_df_telemetry::reporter::MetricsReporter;

    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let (ctx, registry) = crate::testing::test_extension_ctx();

        let entity = ctx.register_extension_entity("ephemeral".into(), ExtensionVariant::Local);
        let metrics = ctx.register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity);
        let key = metrics.metric_set_key();

        let ext = std::rc::Rc::new(TelemetryReportingLocalExt {
            metrics: std::cell::RefCell::new(metrics),
            started: std::cell::Cell::new(false),
        });
        let (n, u, c) = ext_config("ephemeral");
        let w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .local(ext)
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let sender = w.extension_control_sender().unwrap();
        let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });

        let (snapshot_rx, ext_reporter) = MetricsReporter::create_new_and_receiver(4);
        sender
            .send(ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: ext_reporter,
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
        assert!(h.await.unwrap().is_ok());

        // Tear down the registry's entity ownership too, simulating
        // the host releasing the extension after shutdown.
        let _ = registry.unregister_entity(entity);

        // Now consume the snapshot — it must still resolve to the same
        // key and carry the recorded value.
        let snap =
            tokio::time::timeout(std::time::Duration::from_secs(2), snapshot_rx.recv_async())
                .await
                .expect("snapshot should still be deliverable after entity unregistration")
                .expect("snapshot channel closed without delivery");

        assert_eq!(snap.key(), key, "snapshot key must survive extension drop");
        let total: u64 = snap
            .get_metrics()
            .iter()
            .map(|m| match m {
                otap_df_telemetry::metrics::MetricValue::U64(n) => *n,
                _ => 0,
            })
            .sum();
        assert!(
            total >= 1,
            "snapshot recorded value must survive extension drop, got: {:?}",
            snap.get_metrics()
        );
    }));
}

#[test]
fn wire_telemetry_dual_returns_distinct_keys_and_releases_all_entities_per_cycle() {
    // `wire_telemetry` on a dual bundle must mint two distinct entity
    // keys (one per variant). Re-running the wire/drop cycle must
    // return entity count to baseline each time — guards against
    // partial-cleanup leaks under repeated registration.
    let (ctx, registry) = crate::testing::test_extension_ctx();
    let baseline = registry.entity_count();

    for cycle in 0..3 {
        let (n, u, c) = ext_config("dual_cycle");
        let mut bundle = ExtensionWrapper::builder(n, u, &c)
            .active()
            .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
            .shared(TestSharedExt::new(CtrlMsgCounters::new()))
            .build()
            .unwrap();

        let mut cm = ChannelMetricsRegistry::default();
        let keys = bundle.wire_telemetry("dual_cycle".into(), &ctx, &mut cm, true);

        let local_key = keys
            .local
            .unwrap_or_else(|| panic!("cycle {cycle}: dual bundle must assign a local entity key"));
        let shared_key = keys.shared.unwrap_or_else(|| {
            panic!("cycle {cycle}: dual bundle must assign a shared entity key")
        });
        assert_ne!(
            local_key, shared_key,
            "cycle {cycle}: local and shared variants must own distinct entity keys"
        );
        assert_eq!(
            registry.entity_count(),
            baseline + 4,
            "cycle {cycle}: dual active bundle should register 2 extension + 2 channel entities"
        );

        drop(bundle);
        assert_eq!(
            registry.entity_count(),
            baseline,
            "cycle {cycle}: dropping the wired bundle must release every entity it registered"
        );
    }
}

/// `Shutdown` must preempt any backlog on the FIFO control channel —
/// it rides a dedicated oneshot the wrapper checks via biased select.
#[tokio::test(flavor = "current_thread")]
async fn shutdown_preempts_queued_collect_telemetry_on_control_channel() {
    use crate::control::ShutdownPayload;
    use crate::extension::wrapper::ControlChannel;
    use crate::local::message::LocalReceiver;
    use otap_df_channel::mpsc;

    let (tx, rx) = mpsc::Channel::<ExtensionControlMsg>::new(8);
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<ShutdownPayload>();
    let mut control = ControlChannel::new(LocalReceiver::mpsc(rx), shutdown_rx);

    tx.send_async(ExtensionControlMsg::CollectTelemetry {
        metrics_reporter: test_metrics_reporter(),
    })
    .await
    .expect("queue CollectTelemetry");

    let deadline = Instant::now() + std::time::Duration::from_secs(1);
    shutdown_tx
        .send(ShutdownPayload {
            deadline,
            reason: "test".into(),
        })
        .expect("fire dedicated shutdown");

    let first = control.recv().await.expect("recv returns a message");
    assert!(
        matches!(first, ExtensionControlMsg::Shutdown { .. }),
        "Shutdown must be delivered before queued CollectTelemetry — \
         the dedicated shutdown channel must preempt the FIFO control channel"
    );
    assert!(
        control.recv().await.is_err(),
        "after Shutdown the regular control channel must be closed"
    );
}

// Readiness probe wiring tests

use super::readiness::{ReadinessProbeError, ReadinessSignaller};

const PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(6);

#[test]
fn active_without_with_readiness_probe_has_no_probe_or_signaller() {
    let (n, u, c) = ext_config("active_no_probe_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let mut w = bundle.take_shared().unwrap();
    assert!(w.take_readiness_probe().is_none());
    assert!(!w.has_readiness_signaller());

    let (n, u, c) = ext_config("active_no_probe_local");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .build()
        .unwrap();
    let mut w = bundle.take_local().unwrap();
    assert!(w.take_readiness_probe().is_none());
    assert!(!w.has_readiness_signaller());
}

#[test]
fn active_with_readiness_probe_shared_only_plumbs_probe_and_signaller() {
    let (n, u, c) = ext_config("active_probe_shared_only");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();

    assert!(bundle.local().is_none());
    let mut w = bundle.take_shared().unwrap();
    let probe = w.take_readiness_probe().unwrap();
    assert_eq!(probe.timeout(), PROBE_TIMEOUT);
    assert!(w.has_readiness_signaller());
}

#[test]
fn active_with_readiness_probe_local_only_plumbs_probe_and_signaller() {
    let (n, u, c) = ext_config("active_probe_local_only");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .build()
        .unwrap();

    assert!(bundle.shared().is_none());
    let mut w = bundle.take_local().unwrap();
    let probe = w.take_readiness_probe().unwrap();
    assert_eq!(probe.timeout(), PROBE_TIMEOUT);
    assert!(w.has_readiness_signaller());
}

#[test]
fn active_with_readiness_probe_shared_then_local_plumbs_two_pairs() {
    let (n, u, c) = ext_config("active_probe_shared_then_local");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .build()
        .unwrap();

    let mut shared = bundle.take_shared().unwrap();
    let mut local = bundle.take_local().unwrap();

    assert_eq!(
        shared.take_readiness_probe().unwrap().timeout(),
        PROBE_TIMEOUT
    );
    assert_eq!(
        local.take_readiness_probe().unwrap().timeout(),
        PROBE_TIMEOUT
    );
    assert!(shared.has_readiness_signaller());
    assert!(local.has_readiness_signaller());
}

#[test]
fn active_with_readiness_probe_local_then_shared_plumbs_two_pairs() {
    let (n, u, c) = ext_config("active_probe_local_then_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();

    let mut shared = bundle.take_shared().unwrap();
    let mut local = bundle.take_local().unwrap();

    assert!(shared.take_readiness_probe().is_some());
    assert!(local.take_readiness_probe().is_some());
    assert!(shared.has_readiness_signaller());
    assert!(local.has_readiness_signaller());
}

#[test]
fn active_dual_variant_probes_are_independent() {
    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let (n, u, c) = ext_config("active_probe_independent");
        let mut bundle = ExtensionWrapper::builder(n, u, &c)
            .active()
            .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
            .shared(TestSharedExt::new(CtrlMsgCounters::new()))
            .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
            .build()
            .unwrap();
        let mut shared = bundle.take_shared().unwrap();
        let mut local = bundle.take_local().unwrap();

        let s_probe = shared.take_readiness_probe().unwrap();
        let l_probe = local.take_readiness_probe().unwrap();

        let s = shared.extension_control_sender().unwrap();
        let h =
            tokio::task::spawn_local(async move { shared.start(test_metrics_reporter()).await });
        s.send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "drop signaller".into(),
        })
        .await
        .unwrap();
        let _ = h.await.unwrap().unwrap();

        let s_result =
            tokio::time::timeout(std::time::Duration::from_millis(100), s_probe.wait_ready())
                .await
                .unwrap();
        assert_eq!(s_result, Err(ReadinessProbeError::SignallerDropped));

        // Local probe must remain pending while only shared dropped.
        match tokio::time::timeout(std::time::Duration::from_millis(50), l_probe.wait_ready()).await
        {
            Err(_) => {}
            Ok(other) => panic!("expected pending, got {other:?}"),
        }

        drop(local);
    }));
}

#[test]
fn background_without_with_readiness_probe_has_no_probe_or_signaller() {
    let (n, u, c) = ext_config("bg_no_probe_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let mut w = bundle.take_shared().unwrap();
    assert!(w.take_readiness_probe().is_none());
    assert!(!w.has_readiness_signaller());

    let (n, u, c) = ext_config("bg_no_probe_local");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .build()
        .unwrap();
    let mut w = bundle.take_local().unwrap();
    assert!(w.take_readiness_probe().is_none());
    assert!(!w.has_readiness_signaller());
}

#[test]
fn background_with_readiness_probe_shared_plumbs_probe_and_signaller() {
    let (n, u, c) = ext_config("bg_probe_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let mut w = bundle.take_shared().unwrap();
    let probe = w.take_readiness_probe().unwrap();
    assert_eq!(probe.timeout(), PROBE_TIMEOUT);
    assert!(w.has_readiness_signaller());
}

#[test]
fn background_with_readiness_probe_local_plumbs_probe_and_signaller() {
    let (n, u, c) = ext_config("bg_probe_local");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .build()
        .unwrap();
    let mut w = bundle.take_local().unwrap();
    let probe = w.take_readiness_probe().unwrap();
    assert_eq!(probe.timeout(), PROBE_TIMEOUT);
    assert!(w.has_readiness_signaller());
}

#[test]
fn with_readiness_probe_timeout_propagates_to_probe_unchanged() {
    let custom = std::time::Duration::from_millis(123_456);
    let (n, u, c) = ext_config("probe_timeout_propagates");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(custom)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), custom);
}

#[test]
fn take_readiness_probe_is_single_use() {
    let (n, u, c) = ext_config("probe_single_use");
    let mut w = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap()
        .take_shared()
        .unwrap();
    assert!(w.take_readiness_probe().is_some());
    assert!(w.take_readiness_probe().is_none());
}

use super::readiness::DEFAULT_READINESS_TIMEOUT;

#[test]
fn active_with_readiness_probe_uses_default_timeout() {
    let (n, u, c) = ext_config("active_default_timeout_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), DEFAULT_READINESS_TIMEOUT);
}

#[test]
fn active_with_readiness_probe_timeout_at_default_is_accepted() {
    let (n, u, c) = ext_config("active_extended_at_default_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(DEFAULT_READINESS_TIMEOUT)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), DEFAULT_READINESS_TIMEOUT);
}

#[test]
fn active_with_readiness_probe_timeout_above_default_carries_override() {
    let extended = DEFAULT_READINESS_TIMEOUT + std::time::Duration::from_secs(7);
    let (n, u, c) = ext_config("active_extended_above_default_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(extended)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), extended);
}

#[test]
fn active_with_readiness_probe_timeout_below_default_is_accepted() {
    let short = std::time::Duration::from_millis(200);
    let (n, u, c) = ext_config("active_extended_below_default_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(short)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), short);
}

#[test]
fn active_with_readiness_probe_timeout_zero_errors() {
    let (n, u, c) = ext_config("active_extended_zero_shared");
    let result = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(std::time::Duration::ZERO)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build();
    assert!(matches!(
        result,
        Err(Error::ExtensionReadinessZeroTimeout { .. })
    ));
}

#[test]
fn active_with_readiness_probe_timeout_smallest_positive_is_accepted() {
    let smallest = std::time::Duration::from_nanos(1);
    let (n, u, c) = ext_config("active_extended_smallest_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .active()
        .with_readiness_probe_timeout_override(smallest)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), smallest);
}

#[test]
fn background_with_readiness_probe_uses_default_timeout() {
    let (n, u, c) = ext_config("bg_default_timeout_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .with_readiness_probe()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), DEFAULT_READINESS_TIMEOUT);
}

#[test]
fn background_with_readiness_probe_timeout_at_default_is_accepted() {
    let (n, u, c) = ext_config("bg_extended_at_default_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .with_readiness_probe_timeout_override(DEFAULT_READINESS_TIMEOUT)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), DEFAULT_READINESS_TIMEOUT);
}

#[test]
fn background_with_readiness_probe_timeout_above_default_carries_override() {
    let extended = DEFAULT_READINESS_TIMEOUT + std::time::Duration::from_secs(11);
    let (n, u, c) = ext_config("bg_extended_above_default_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .with_readiness_probe_timeout_override(extended)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), extended);
}

#[test]
fn background_with_readiness_probe_timeout_below_default_is_accepted() {
    let short = std::time::Duration::from_millis(200);
    let (n, u, c) = ext_config("bg_extended_below_default_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .with_readiness_probe_timeout_override(short)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), short);
}

#[test]
fn background_with_readiness_probe_timeout_zero_errors() {
    let (n, u, c) = ext_config("bg_extended_zero_shared");
    let result = ExtensionWrapper::builder(n, u, &c)
        .background()
        .with_readiness_probe_timeout_override(std::time::Duration::ZERO)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build();
    assert!(matches!(
        result,
        Err(Error::ExtensionReadinessZeroTimeout { .. })
    ));
}

#[test]
fn background_with_readiness_probe_timeout_smallest_positive_is_accepted() {
    let smallest = std::time::Duration::from_nanos(1);
    let (n, u, c) = ext_config("bg_extended_smallest_shared");
    let mut bundle = ExtensionWrapper::builder(n, u, &c)
        .background()
        .with_readiness_probe_timeout_override(smallest)
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let probe = bundle
        .take_shared()
        .unwrap()
        .take_readiness_probe()
        .unwrap();
    assert_eq!(probe.timeout(), smallest);
}

#[test]
fn effect_handler_signal_ready_is_noop_when_no_signaller() {
    let h = EffectHandler::new("noop_eh".into(), test_metrics_reporter(), None);
    for _ in 0..4 {
        h.signal_ready();
    }
    let c = h.clone();
    c.signal_ready();
}

#[tokio::test]
async fn effect_handler_signal_ready_fires_signaller_when_present() {
    let (sig, probe) = ReadinessSignaller::pair(PROBE_TIMEOUT);
    let inspect = sig.clone();
    let h = EffectHandler::new("fires_eh".into(), test_metrics_reporter(), Some(sig));

    assert!(!inspect.is_ready());
    h.signal_ready();
    assert!(inspect.is_ready());

    tokio::time::timeout(std::time::Duration::from_millis(50), probe.wait_ready())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn effect_handler_signal_ready_idempotent_across_clones_and_calls() {
    let (sig, probe) = ReadinessSignaller::pair(PROBE_TIMEOUT);
    let h = EffectHandler::new("idem_eh".into(), test_metrics_reporter(), Some(sig));
    let c1 = h.clone();
    let c2 = h.clone();

    h.signal_ready();
    h.signal_ready();
    c1.signal_ready();
    c2.signal_ready();
    c2.signal_ready();

    tokio::time::timeout(std::time::Duration::from_millis(50), probe.wait_ready())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn effect_handler_drop_without_signal_ready_yields_signaller_dropped() {
    let (sig, probe) = ReadinessSignaller::pair(std::time::Duration::from_secs(60));
    let h = EffectHandler::new("drop_eh".into(), test_metrics_reporter(), Some(sig));
    let c = h.clone();
    drop(h);
    drop(c);

    let res = tokio::time::timeout(std::time::Duration::from_millis(50), probe.wait_ready())
        .await
        .unwrap();
    assert_eq!(res, Err(ReadinessProbeError::SignallerDropped));
}

#[derive(Clone)]
struct ReadyOnStartSharedExt;

#[async_trait]
impl SharedExtension for ReadyOnStartSharedExt {
    async fn start(
        self: Box<Self>,
        mut ctrl: shared_ext::ControlChannel,
        eh: EffectHandler,
    ) -> Result<TerminalState, Error> {
        eh.signal_ready();
        loop {
            if let ExtensionControlMsg::Shutdown { .. } = ctrl.recv().await? {
                break;
            }
        }
        Ok(TerminalState::default())
    }
}

#[derive(Clone)]
struct ReadyOnStartLocalExt;

#[async_trait(?Send)]
impl crate::local::extension::Extension for ReadyOnStartLocalExt {
    async fn start(
        self: std::rc::Rc<Self>,
        mut ctrl: local_ext::ControlChannel,
        eh: EffectHandler,
    ) -> Result<TerminalState, Error> {
        eh.signal_ready();
        loop {
            if let ExtensionControlMsg::Shutdown { .. } = ctrl.recv().await? {
                break;
            }
        }
        Ok(TerminalState::default())
    }
}

#[test]
fn shared_active_start_fires_engine_probe_via_eh_signal_ready() {
    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let (n, u, c) = ext_config("shared_start_ready");
        let mut w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
            .shared(ReadyOnStartSharedExt)
            .build()
            .unwrap()
            .take_shared()
            .unwrap();
        let probe = w.take_readiness_probe().unwrap();
        let s = w.extension_control_sender().unwrap();

        let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });

        tokio::time::timeout(std::time::Duration::from_millis(200), probe.wait_ready())
            .await
            .unwrap()
            .unwrap();

        s.send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "d".into(),
        })
        .await
        .unwrap();
        let _ = h.await.unwrap().unwrap();
    }));
}

#[test]
fn local_active_start_fires_engine_probe_via_eh_signal_ready() {
    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let (n, u, c) = ext_config("local_start_ready");
        let mut w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .with_readiness_probe_timeout_override(PROBE_TIMEOUT)
            .local(std::rc::Rc::new(ReadyOnStartLocalExt))
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let probe = w.take_readiness_probe().unwrap();
        let s = w.extension_control_sender().unwrap();

        let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });

        tokio::time::timeout(std::time::Duration::from_millis(200), probe.wait_ready())
            .await
            .unwrap()
            .unwrap();

        s.send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "d".into(),
        })
        .await
        .unwrap();
        let _ = h.await.unwrap().unwrap();
    }));
}

#[test]
fn shared_active_start_without_ready_resolves_probe_with_signaller_dropped() {
    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let (n, u, c) = ext_config("shared_start_no_ready");
        let mut w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .with_readiness_probe_timeout_override(std::time::Duration::from_secs(60))
            .shared(TestSharedExt::new(CtrlMsgCounters::new()))
            .build()
            .unwrap()
            .take_shared()
            .unwrap();
        let probe = w.take_readiness_probe().unwrap();
        let s = w.extension_control_sender().unwrap();
        let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });

        s.send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "no ready".into(),
        })
        .await
        .unwrap();
        let _ = h.await.unwrap().unwrap();

        let res = tokio::time::timeout(std::time::Duration::from_millis(100), probe.wait_ready())
            .await
            .unwrap();
        assert_eq!(res, Err(ReadinessProbeError::SignallerDropped));
    }));
}

#[test]
fn shared_active_start_without_opt_in_eh_signal_ready_is_silent_noop() {
    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let (n, u, c) = ext_config("shared_start_no_optin");
        let mut w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .shared(ReadyOnStartSharedExt)
            .build()
            .unwrap()
            .take_shared()
            .unwrap();
        assert!(w.take_readiness_probe().is_none());
        let s = w.extension_control_sender().unwrap();
        let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
        s.send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "d".into(),
        })
        .await
        .unwrap();
        let _ = h.await.unwrap().unwrap();
    }));
}
