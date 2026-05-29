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
    // Shutdown is delivered immediately regardless of how far in the
    // future its deadline is — the deadline is the extension's
    // cooperative cleanup budget, not a wait time before delivery.
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
    let dl = Instant::now() + std::time::Duration::from_secs(60);
    let sender = SharedSender::mpsc(tx);
    sender
        .send(ExtensionControlMsg::Shutdown {
            deadline: dl,
            reason: "dl".into(),
        })
        .await
        .unwrap();
    let before = Instant::now();
    assert!(ch.recv().await.unwrap().is_shutdown());
    assert!(
        before.elapsed() < std::time::Duration::from_secs(1),
        "Shutdown should be delivered immediately, not after the deadline"
    );
    // After Shutdown the channel is closed.
    assert!(ch.recv().await.is_err());
    drop(sender);
}

#[tokio::test]
async fn test_ctrl_shutdown_closes_channel() {
    // Messages sent after Shutdown are never observed: once Shutdown
    // is delivered the channel is closed and the extension's event
    // loop terminates.
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
    let s = SharedSender::mpsc(tx);
    s.send(ExtensionControlMsg::Shutdown {
        deadline: Instant::now() + std::time::Duration::from_secs(1),
        reason: "d".into(),
    })
    .await
    .unwrap();
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
        .passive()
        .cloned()
        .shared("d".to_string())
        .build()
        .unwrap()
        .take_shared()
        .unwrap();
    let (ctx, _) = crate::testing::test_extension_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let key = ctx.register_extension_entity("pm".into(), crate::extension::wrapper::ExtensionVariant::Shared);
    let handle =
        crate::entity_context::EntityTelemetryHandle::new(ctx.metrics_registry(), key);
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
    let key = ctx.register_extension_entity("acm".into(), crate::extension::wrapper::ExtensionVariant::Shared);
    let handle =
        crate::entity_context::EntityTelemetryHandle::new(ctx.metrics_registry(), key);
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
    // The convention paired with `.background()`: an ExtensionFactory built
    // around the bundle sets `capabilities: None`. `register_into` skips
    // capability registration, so the bundle contributes zero entries to the
    // CapabilityRegistry.
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
    // Registry remains empty — Background extensions contribute zero entries.
    // (No `is_empty` accessor is exposed; instead we rely on the absence of
    // any entries being observable via `get_local`/`get_shared` returning
    // `None` for any (TypeId, ExtensionId) we might query. The strongest
    // available assertion here: the no-op call returns `Ok(())`.)
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
fn test_extension_attribute_set_is_scope_agnostic() {
    use crate::attributes::ExtensionAttributeSet;
    use otap_df_telemetry::attributes::AttributeSetHandler;

    let attrs = ExtensionAttributeSet {
        extension_id: "ext1".into(),
        extension_variant: "local".into(),
    };
    assert_eq!(attrs.schema_name(), "extension.attrs");
    let keys: Vec<&'static str> = attrs.iter_attributes().map(|(k, _)| k).collect();
    assert_eq!(keys, vec!["extension.id", "extension.variant"]);
}

#[test]
fn test_extension_channel_attribute_set_shape() {
    use crate::attributes::{ExtensionAttributeSet, ExtensionChannelAttributeSet};
    use otap_df_telemetry::attributes::AttributeSetHandler;

    let attrs = ExtensionChannelAttributeSet {
        channel_id: "ext1:control".into(),
        extension_attrs: ExtensionAttributeSet {
            extension_id: "ext1".into(),
            extension_variant: "shared".into(),
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
}

#[test]
fn test_register_extension_entity_unregister_roundtrip() {
    let (ctx, registry) = crate::testing::test_extension_ctx();
    let before = registry.entity_count();
    let key = ctx.register_extension_entity("ext1".into(), crate::extension::wrapper::ExtensionVariant::Local);
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

// Active extension contract: when the host's monitor dispatches
// `CollectTelemetry { metrics_reporter }`, the extension must use that
// supplied reporter to publish its own internal metrics. This test
// stands in for the runtime: it sends a `CollectTelemetry` whose
// reporter the test owns the receiver of, and asserts the extension's
// snapshot lands on that channel.
#[otap_df_telemetry_macros::metric_set(name = "test.extension.internal")]
#[derive(Debug, Default, Clone)]
struct TestExtensionInternalMetrics {
    #[metric(unit = "{tick}")]
    collect_invocations: otap_df_telemetry::instrument::Counter<u64>,
}

struct TelemetryReportingLocalExt {
    metrics: std::cell::RefCell<otap_df_telemetry::metrics::MetricSet<TestExtensionInternalMetrics>>,
    started: std::cell::Cell<bool>,
}

// `Clone` is required by `ActiveStage::local`. The clones share state
// via interior mutability, which is fine for this test since only one
// live instance ever runs.
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
        let internal_metrics = ctx
            .register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity);

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

        let h = tokio::task::spawn_local(async move {
            w.start(test_metrics_reporter()).await
        });

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
        let snapshot = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            snapshot_rx.recv_async(),
        )
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
    // Cross-contamination guard: when two active extensions report via
    // the same MetricsReporter, the collector must receive two snapshots
    // with distinct `MetricSetKey`s — one per extension — and the values
    // recorded by extension A must NOT appear under extension B's key
    // (or vice versa). This is the invariant that lets the host monitor
    // safely share a single reporter across all extensions.
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
        let metrics_a = ctx
            .register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity_a);
        let key_a = metrics_a.metric_set_key();

        let entity_b = ctx.register_extension_entity(
            "ext_b".into(),
            crate::extension::wrapper::ExtensionVariant::Local,
        );
        let metrics_b = ctx
            .register_metric_set_for_entity::<TestExtensionInternalMetrics>(entity_b);
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

        let h_a = tokio::task::spawn_local(async move {
            w_a.start(test_metrics_reporter()).await
        });
        let h_b = tokio::task::spawn_local(async move {
            w_b.start(test_metrics_reporter()).await
        });

        // ONE shared reporter — both extensions publish through the same
        // channel. Isolation must be enforced by per-MetricSet keys, not
        // by separate transports.
        let (snapshot_rx, shared_reporter) =
            MetricsReporter::create_new_and_receiver(8);

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
        let mut sum_by_key: std::collections::HashMap<_, u64> =
            std::collections::HashMap::new();
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

        // Each extension's reported total equals its own invocation count
        // and no others. If contamination existed (e.g., a single shared
        // counter, or wrong key routing), A's two invocations would leak
        // into B's total, or vice versa.
        assert_eq!(a_total, 2, "ext_a should report exactly its 2 invocations");
        assert_eq!(b_total, 1, "ext_b should report exactly its 1 invocation");
        // Only A's and B's keys ever appear — no spurious third entry.
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
