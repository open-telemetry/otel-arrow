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
    let (ctx, _) = crate::testing::test_pipeline_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let key = ctx.register_extension_entity("pm".into());
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

    let (ctx, _) = crate::testing::test_pipeline_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let key = ctx.register_extension_entity("acm".into());
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
    };
    assert_eq!(attrs.schema_name(), "extension.attrs");
    // extension_id is the only attribute; no pipeline/engine attributes baked in.
    let keys: Vec<&'static str> = attrs.iter_attributes().map(|(k, _)| k).collect();
    assert_eq!(keys, vec!["extension.id"]);
}

#[test]
fn test_extension_channel_attribute_set_shape() {
    use crate::attributes::{ExtensionAttributeSet, ExtensionChannelAttributeSet};
    use otap_df_telemetry::attributes::AttributeSetHandler;

    let attrs = ExtensionChannelAttributeSet {
        channel_id: "ext1:control".into(),
        extension_attrs: ExtensionAttributeSet {
            extension_id: "ext1".into(),
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
    let (ctx, registry) = crate::testing::test_pipeline_ctx();
    let before = registry.entity_count();
    let key = ctx.register_extension_entity("ext1".into());
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
    let (ctx, registry) = crate::testing::test_pipeline_ctx();
    let key = ctx.register_extension_channel_entity(
        "ext1".into(),
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

    let (ctx, registry) = crate::testing::test_pipeline_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let before = registry.entity_count();

    let key = ctx.register_extension_entity("pwt".into());
    let handle =
        crate::entity_context::EntityTelemetryHandle::new(ctx.metrics_registry(), key);
    bundle.wire_telemetry(handle, &ctx, &mut cm, true);

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

    let (ctx, registry) = crate::testing::test_pipeline_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let before = registry.entity_count();

    let key = ctx.register_extension_entity("awt".into());
    let handle =
        crate::entity_context::EntityTelemetryHandle::new(ctx.metrics_registry(), key);
    bundle.wire_telemetry(handle, &ctx, &mut cm, true);

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

    let (ctx, registry) = crate::testing::test_pipeline_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let before = registry.entity_count();

    let key = ctx.register_extension_entity("dwt".into());
    let handle =
        crate::entity_context::EntityTelemetryHandle::new(ctx.metrics_registry(), key);
    bundle.wire_telemetry(handle, &ctx, &mut cm, true);

    // Dual active => extension entity + 2 control-channel entities
    // (one for local variant, one for shared variant), both tracked on the
    // shared handle.
    assert_eq!(registry.entity_count(), before + 3);

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

    let (ctx, registry) = crate::testing::test_pipeline_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let before = registry.entity_count();

    let key = ctx.register_extension_entity("dis".into());
    let handle =
        crate::entity_context::EntityTelemetryHandle::new(ctx.metrics_registry(), key);
    bundle.wire_telemetry(handle, &ctx, &mut cm, false);

    assert_eq!(registry.entity_count(), before + 2);

    drop(bundle);
    assert_eq!(registry.entity_count(), before);
}
